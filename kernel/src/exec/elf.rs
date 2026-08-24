//! ELF64 image loading into a fresh address space.
//!
//! Naming follows Linux `fs/binfmt_elf.c`: `elf_load` is the binfmt entry for
//! one image, `load_elf_phdrs` fetches the program-header table, `elf_map`
//! maps a single `PT_LOAD` (file part + bss part).
//!
//! Phase-1 scope: static `ET_EXEC`, no `PT_INTERP`/`ET_DYN` yet (rejected with
//! dedicated errors). Mapping is zero-copy: the file already sits in RAM, so a
//! `PT_LOAD` maps the file's physical pages directly (no copy, no
//! `write_user`). `PT_GNU_STACK`/`PT_PHDR` are analysed for later phases but
//! do not drive any action yet.

use object::read::elf::{FileHeader, ProgramHeader};
use object::{elf, LittleEndian};

use crate::arch::traits::{TargetAddressSpace, TargetPhysicalAllocator};
use crate::arch::{AddressSpace, KERNEL_LAYOUT, PhysicalAddress, PhysicalAllocator, VirtualAddress};
use crate::exec::image::Image;
use crate::exec::{align_down, align_up, ExecError};
use crate::vm::{MappingFlags, MappingPermissions};

type Endian = LittleEndian;type Elf64 = elf::FileHeader64<Endian>;
type Phdr = elf::ProgramHeader64<Endian>;

/// Result of mapping one ELF image.
pub struct LoadInfo {
    pub entry: u64,
    /// First free address after the highest segment (future `brk`).
    pub brk: u64,
    pub stack_exec: bool,
    /// File offset of the phdr table (for a later `AT_PHDR`).
    pub phoff: u64,
    pub phent: u64,
    pub phnum: u64,
}

/// Read-only facts collected in the analysis pass.
struct Analysis {
    min_vaddr: u64,
    max_vaddr: u64,
    stack_exec: bool,
    load_count: u32,
}

/// `p_flags` -> mapping permissions.
fn prot_from(p_flags: u32) -> MappingPermissions {
    MappingPermissions::new()
        .with_read(p_flags & elf::PF_R.0 != 0)
        .with_write(p_flags & elf::PF_W.0 != 0)
        .with_execute(p_flags & elf::PF_X.0 != 0)
}

/// Validate the ELF header. Any failure is an ENOEXEC analogue.
fn validate_ehdr(hdr: &Elf64, endian: Endian) -> Result<(), ExecError> {
    if hdr.e_ident.magic != elf::ELFMAG {
        return Err(ExecError::BadMagic);
    }
    if hdr.e_ident.class != elf::ELFCLASS64 {
        return Err(ExecError::UnsupportedClass);
    }
    if hdr.e_ident.data != elf::ELFDATA2LSB {
        return Err(ExecError::UnsupportedEndian);
    }
    if hdr.e_machine.get(endian) != elf::EM_RISCV {
        return Err(ExecError::UnsupportedMachine);
    }
    match hdr.e_type.get(endian) {
        t if t == elf::ET_EXEC => {}
        t if t == elf::ET_DYN => return Err(ExecError::DynUnsupported),
        _ => return Err(ExecError::UnsupportedType),
    }
    if hdr.e_phentsize.get(endian) as usize != core::mem::size_of::<Phdr>() {
        return Err(ExecError::BadPhdrEntSize);
    }
    let phnum = hdr.e_phnum.get(endian);
    if phnum == 0 || phnum > 1024 {
        return Err(ExecError::BadPhdrCount);
    }
    Ok(())
}

/// Fetch the program-header table from the image.
fn load_elf_phdrs(
    image: &mut Image,
    elf_ex: &Elf64,
    endian: Endian,
) -> Result<&'static [Phdr], ExecError> {
    elf_ex
        .program_headers(endian, *image)
        .map_err(|_| ExecError::ImageFault)
}

/// Single analysis pass over the program headers.
fn analyze_phdrs(phdrs: &[Phdr], endian: Endian) -> Result<Analysis, ExecError> {
    let mut analysis = Analysis {
        min_vaddr: u64::MAX,
        max_vaddr: 0,
        stack_exec: false,
        load_count: 0,
    };

    for phdr in phdrs {
        let p_type = phdr.p_type(endian);
        if p_type == elf::PT_LOAD {
            let file_size = phdr.p_filesz(endian);
            let mem_size = phdr.p_memsz(endian);
            if mem_size < file_size {
                return Err(ExecError::MemszLtFilesz);
            }
            // Linker invariant required by page-granular mapping.
            if phdr.p_vaddr(endian) % 4096 != phdr.p_offset(endian) % 4096 {
                return Err(ExecError::MisalignedSegment);
            }
            analysis.min_vaddr = analysis.min_vaddr.min(align_down(phdr.p_vaddr(endian), 4096));
            analysis.max_vaddr = analysis
                .max_vaddr
                .max(align_up(phdr.p_vaddr(endian) + mem_size, 4096));
            analysis.load_count += 1;
        } else if p_type == elf::PT_INTERP {
            return Err(ExecError::InterpUnsupported);
        } else if p_type == elf::PT_GNU_STACK {
            analysis.stack_exec = phdr.p_flags(endian).0 & elf::PF_X.0 != 0;
        }
    }

    if analysis.load_count == 0 {
        return Err(ExecError::NoLoadSegments);
    }
    Ok(analysis)
}

/// Map one `PT_LOAD` segment, zero-copy from the RAM-resident file image.
///
/// The file part maps the file's own physical pages directly into the address
/// space (the bytes are already in RAM). The bss part (`memsz > filesz`) gets
/// fresh zeroed anonymous pages.
fn elf_map(
    image: &mut Image,
    address_space: &mut AddressSpace,
    phdr: &Phdr,
    endian: Endian,
    load_bias: u64,
) -> Result<(), ExecError> {
    let vaddr = load_bias + phdr.p_vaddr(endian);
    let file_offset = phdr.p_offset(endian);
    let file_size = phdr.p_filesz(endian);
    let mem_size = phdr.p_memsz(endian);
    let perms = prot_from(phdr.p_flags(endian).0);

    let vstart = align_down(vaddr, 4096);
    let page_offset = vaddr - vstart; // == file_offset % PAGE (checked in analyse)

    // File-backed part: map the file's physical pages directly. Zero-copy —
    // the bytes are already in RAM, so the segment is installed with its final
    // permissions right away (there is no `protect` in the kernel VM API yet).
    let file_map_len = align_up(page_offset + file_size, 4096);
    if file_map_len > 0 {
        let file_page_base = align_down(file_offset, 4096);
        let phys_addr = image.phys_addr(file_page_base);
        let phys = PhysicalAddress::try_from(phys_addr as usize)
            .map_err(|_| ExecError::ImageFault)?;
        let alloc = PhysicalAllocator::alloc_contiguous_at(phys, file_map_len as usize)?;

        address_space.map(
            VirtualAddress::try_from(vstart as usize).unwrap(),
            alloc,
            perms,
            MappingFlags::new().with_user(true),
        );
    }

    // bss: `mem_size > file_size`. Whole extra pages get their own zeroed
    // anonymous mapping with the final permissions.
    let bss_start = vaddr + file_size;
    let bss_end = vaddr + mem_size;
    let bss_page_start = align_up(bss_start, 4096);
    let bss_page_end = align_up(bss_end, 4096);
    if bss_page_start < bss_page_end {
        let alloc = PhysicalAllocator::alloc_contiguous((bss_page_end - bss_page_start) as usize)?;
        address_space.map(
            VirtualAddress::try_from(bss_page_start as usize).unwrap(),
            alloc,
            perms,
            MappingFlags::new().with_user(true),
        );
    }

    info!(
        "exec: segment {:#018x}..{:#018x} mapped (filesz={:#x} memsz={:#x})",
        vaddr,
        vaddr + mem_size,
        file_size,
        mem_size,
    );
    Ok(())
}

// Temporary MMU tables and per-thread kernel stacks live in `.page_table_pool`
// after `__epage_table_pool`; `KERNEL_LAYOUT` does not know them.
unsafe extern "C" {
    #[link_name = "__s_temp_mmu_table"]
    static S_TEMP_MMU_TABLE: u8;
    #[link_name = "__e_temp_mmu_table"]
    static E_TEMP_MMU_TABLE: u8;
    #[link_name = "__s_temp_kernel_stacks"]
    static S_TEMP_KERNEL_STACKS: u8;
    #[link_name = "__e_temp_kernel_stacks"]
    static E_TEMP_KERNEL_STACKS: u8;
}

/// Map the kernel half into a fresh process address space, sharing the
/// physical pages that `map_kernel_sections` already reserved for the boot AS.
///
/// The boot-time mapping owns/reserves the kernel pages; here we only take
/// references (refcount++) so traps/syscalls still find the kernel after a
/// switch. The mappings are owned by `address_space` itself and are torn down
/// when the address space dies.
pub fn map_kernel_shared(address_space: &mut AddressSpace) -> Result<(), ExecError> {
    let layout = &KERNEL_LAYOUT;
    let va_offset = layout.kernel_va_offset;

    let sections: [(&str, usize, usize, MappingPermissions); 9] = [
        ("text", layout.stext, layout.etext, MappingPermissions::rx()),
        ("rodata", layout.srodata, layout.erodata, MappingPermissions::ro()),
        ("data", layout.sdata, layout.edata, MappingPermissions::rw()),
        ("bss", layout.sbss, layout.ebss, MappingPermissions::rw()),
        ("heap", layout.sheap, layout.eheap, MappingPermissions::rw()),
        (
            "page_table_pool",
            layout.spage_table_pool,
            layout.epage_table_pool,
            MappingPermissions::rw(),
        ),
        (
            "temp_mmu_table",
            unsafe { &S_TEMP_MMU_TABLE as *const u8 as usize },
            unsafe { &E_TEMP_MMU_TABLE as *const u8 as usize },
            MappingPermissions::rw(),
        ),
        (
            "kernel_stacks",
            unsafe { &S_TEMP_KERNEL_STACKS as *const u8 as usize },
            unsafe { &E_TEMP_KERNEL_STACKS as *const u8 as usize },
            MappingPermissions::rw(),
        ),
        ("stack", layout.estack, layout.sstack, MappingPermissions::rw()),
    ];

    for (name, start, end, perms) in sections {
        if end <= start {
            continue;
        }
        let phys = PhysicalAddress::try_from(start - va_offset)
            .map_err(|_| ExecError::ImageFault)?;
        // Pages already reserved by boot (kernel sections) get one more
        // reference; pages the kernel occupies but never marked (temporary
        // MMU tables, per-thread stacks) get reserved on the spot.
        let alloc = match PhysicalAllocator::retain(phys, end - start) {
            Ok(a) => a,
            Err(_) => PhysicalAllocator::alloc_contiguous_at(phys, end - start)?,
        };
        address_space.map(
            VirtualAddress::try_from(start).unwrap(),
            alloc,
            perms,
            MappingFlags::new(),
        );
        info!(
            "exec: shared kernel {name} {start:#018x}..{end:#018x}",
        );
    }

    Ok(())
}

/// Parse, validate and map an ELF image into `address_space`; summarise the
/// result for a later stack builder.
///
/// `load_bias` is 0 for `ET_EXEC`; non-zero biases are the `ET_DYN`/PIE hook.
pub fn elf_load(
    image: &mut Image,
    address_space: &mut AddressSpace,
    load_bias: u64,
) -> Result<LoadInfo, ExecError> {
    let endian = LittleEndian;

    let elf_ex = Elf64::parse(*image).map_err(|_| ExecError::BadMagic)?;
    validate_ehdr(elf_ex, endian)?;

    let phdrs = load_elf_phdrs(image, elf_ex, endian)?;
    let analysis = analyze_phdrs(phdrs, endian)?;
    info!(
        "exec: image span {:#018x}..{:#018x}, {} PT_LOAD segments",
        analysis.min_vaddr,
        analysis.max_vaddr,
        analysis.load_count,
    );

    let mut brk = 0u64;
    for phdr in phdrs {
        if phdr.p_type(endian) != elf::PT_LOAD {
            continue;
        }
        elf_map(image, address_space, phdr, endian, load_bias)?;
        brk = brk.max(align_up(
            load_bias + phdr.p_vaddr(endian) + phdr.p_memsz(endian),
            4096,
        ));
    }

    Ok(LoadInfo {
        entry: load_bias + elf_ex.e_entry.get(endian),
        brk,
        stack_exec: analysis.stack_exec,
        phoff: elf_ex.e_phoff.get(endian),
        phent: elf_ex.e_phentsize.get(endian) as u64,
        phnum: elf_ex.e_phnum.get(endian) as u64,
    })
}
