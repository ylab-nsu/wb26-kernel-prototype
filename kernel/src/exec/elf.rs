//! ELF64 image loading into a fresh address space.
//!
//! Three phases, mirroring Linux `fs/binfmt_elf.c`:
//! - [`read_and_validate`] parses and validates the ELF header and the
//!   program-header table (the file is read exactly twice),
//! - [`load`] maps the kernel half, the user stack and every `PT_LOAD` segment
//!   into a fresh address space, producing a [`UserProgram`],
//! - [`run_elf`] is the public entry: it chains both phases and spawns the
//!   program via `spawn_user_program`.
//!
//! Scope: static `ET_EXEC`, no `PT_INTERP`/`ET_DYN` yet (rejected with
//! dedicated errors). Mapping is zero-copy: the file already sits in RAM, so a
//! `PT_LOAD` maps the file's physical pages directly (no copy, no
//! `write_user`). Non-`PT_LOAD` program headers (`PHDR`, `GNU_STACK`,
//! `RISCV_ATTRIBUTES`, ...) are ignored.

use object::read::elf::{FileHeader, ProgramHeader};
use object::{elf, LittleEndian};

use crate::arch::traits::{TargetAddressSpace, TargetPhysicalAllocation, TargetPhysicalAllocator};
use crate::arch::{
    kernel_sections, AddressSpace, KERNEL_LAYOUT, PhysicalAddress, PhysicalAllocator,
    VirtualAddress,
};
use crate::exec::image::Image;
use crate::exec::{align_down, align_up, ExecError, USER_STACK_SIZE, USER_STACK_TOP};
use crate::threading::init::{spawn_user_program, UserProgram};
use crate::vm::{MappingFlags, MappingPermissions};

type Endian = LittleEndian;type Elf64 = elf::FileHeader64<Endian>;
type Phdr = elf::ProgramHeader64<Endian>;

/// Parsed and validated ELF metadata, produced by [`read_and_validate`] and
/// consumed by [`load`]. `phdrs` points into the image RAM and is reused so the
/// file is read exactly twice: the ELF header, then the program-header table.
struct ExecMeta {
    entry: u64,
    phdrs: &'static [Phdr],
    /// First free address after the highest segment (future `brk`).
    brk: u64,
}

/// Read-only facts collected in the analysis pass.
struct PhdrsAnalysis {
    min_vaddr: u64,
    max_vaddr: u64,
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
fn validate_elf_hdr(hdr: &Elf64, endian: Endian) -> Result<(), ExecError> {
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
fn load_program_hdrs(
    image: &mut Image,
    elf_hdr: &Elf64,
    endian: Endian,
) -> Result<&'static [Phdr], ExecError> {
    elf_hdr
        .program_headers(endian, *image)
        .map_err(|_| ExecError::ImageFault)
}

fn validate_program_hdr(program_hdr: &Phdr, endian: Endian) -> Result<(), ExecError> {
    let file_size = program_hdr.p_filesz(endian);
    let mem_size = program_hdr.p_memsz(endian);

    if mem_size < file_size {
        return Err(ExecError::MemszLtFilesz);
    }

    // Linker invariant required by page-granular mapping.
    if program_hdr.p_vaddr(endian) % 4096 != program_hdr.p_offset(endian) % 4096 {
        return Err(ExecError::MisalignedSegment);
    }

    Ok(())
}

fn vaddr_alloc_bounds(program_hdr: &Phdr, endian: Endian) -> (u64, u64) {
    let vaddr_alloc_start = align_down(
        program_hdr.p_vaddr(endian), 
        4096
    );
    let vaddr_alloc_end = align_up(
        program_hdr.p_vaddr(endian) + program_hdr.p_memsz(endian), 
        4096
    );
    
    (vaddr_alloc_start, vaddr_alloc_end)
}

/// Single analysis pass over the program headers.
fn analyze_program_hdrs(program_hdrs: &[Phdr], endian: Endian) -> Result<PhdrsAnalysis, ExecError> {
    let mut analysis = PhdrsAnalysis {
        min_vaddr: u64::MAX,
        max_vaddr: 0,
        load_count: 0,
    };

    for program_hdr in program_hdrs {
        match program_hdr.p_type(endian) {
            elf::PT_LOAD => {
                validate_program_hdr(program_hdr, endian)?;
                let (vaddr_alloc_start, vaddr_alloc_end) = vaddr_alloc_bounds(program_hdr, endian);
                analysis.min_vaddr = analysis.min_vaddr.min(vaddr_alloc_start);
                analysis.max_vaddr = analysis.max_vaddr.max(vaddr_alloc_end);
                analysis.load_count += 1;
            }
            elf::PT_INTERP => {
                return Err(ExecError::InterpUnsupported);
            }
            // PHDR, GNU_STACK, RISCV_ATTRIBUTES, GNU_RELRO, ... are ignored for
            // a static ET_EXEC (only PT_INTERP is unsupported).
            _ => {}
        }
    }

    if analysis.load_count == 0 {
        return Err(ExecError::NoLoadSegments);
    }
    Ok(analysis)
}

/// Load one `PT_LOAD` segment into a freshly allocated region: copy the file
/// bytes, zero-fill the bss tail, then map the region into the address space.
fn map_segment(
    image: &mut Image,
    address_space: &mut AddressSpace,
    phdr: &Phdr,
    endian: Endian,
) -> Result<(), ExecError> {
    let vaddr = phdr.p_vaddr(endian);
    let filesz = phdr.p_filesz(endian);
    let memsz = phdr.p_memsz(endian);
    if memsz == 0 {
        return Ok(());
    }
    let perms = prot_from(phdr.p_flags(endian).0);

    let (vaddr_alloc_start, vaddr_alloc_end) = vaddr_alloc_bounds(phdr, endian);

    let alloc = PhysicalAllocator::alloc_contiguous(
        (vaddr_alloc_end - vaddr_alloc_start) as usize
    )?;
    let phys_alloc_start = alloc.addr().into_bits() as usize;
    let segment_offset = (vaddr - vaddr_alloc_start) as usize;

    // Copy the file bytes once, read through the abstract image.
    if filesz > 0 {
        image.read_into(
            phdr.p_offset(endian),
            (phys_alloc_start + segment_offset) as *mut u8,
            filesz as usize
        )?;
    }

    // Zero only the bss remainder, leaving file bytes untouched.
    if memsz > filesz {
        unsafe {
            core::ptr::write_bytes(
                (phys_alloc_start + segment_offset + filesz as usize) as *mut u8,
                0,
                (memsz - filesz) as usize,
            );
        }
    }

    address_space.map(
        VirtualAddress::try_from(vaddr_alloc_start as usize).unwrap(),
        alloc,
        perms,
        MappingFlags::new().with_user(true),
    );
    Ok(())
}

/// Map the kernel half into a fresh process address space, sharing the
/// physical pages that `map_kernel_sections` already reserved for the boot AS.
///
/// The boot-time mapping owns/reserves the kernel pages; here we only take
/// references (refcount++) so traps/syscalls still find the kernel after a
/// switch. The mappings are owned by `address_space` itself and are torn down
/// when the address space dies.
pub fn map_kernel_shared(address_space: &mut AddressSpace) -> Result<(), ExecError> {
    for (name, start, end, perms) in kernel_sections() {
        if end <= start {
            continue;
        }
        let phys = PhysicalAddress::try_from(start - KERNEL_LAYOUT.kernel_va_offset)
            .map_err(|_| ExecError::ImageFault)?;
        // Boot (map_kernel_sections) already reserved every kernel page, so
        // we only take one more reference to share them with this process.
        let alloc = PhysicalAllocator::retain(phys, end - start)?;
        address_space.map(
            VirtualAddress::try_from(start).unwrap(),
            alloc,
            perms,
            MappingFlags::new(),
        );
        info!("exec: shared kernel {name} {start:#018x}..{end:#018x}");
    }

    Ok(())
}

fn init_stack(address_space: &mut AddressSpace) {
    let stack_alloc = PhysicalAllocator::alloc_contiguous(USER_STACK_SIZE as usize)
        .expect("alloc stack");

    address_space.map(
        VirtualAddress::try_from((USER_STACK_TOP - USER_STACK_SIZE) as usize).unwrap(),
        stack_alloc,
        MappingPermissions::rw(),
        MappingFlags::new().with_user(true),
    );
}

/// Phase 1: parse the ELF header and the program-header table and validate
/// them. The file is read exactly twice (ELF header, then phdr table); the
/// resulting [`ExecMeta`] is reused by [`load`] — no further reads.
fn read_and_validate(image: &mut Image) -> Result<ExecMeta, ExecError> {
    let endian = LittleEndian;

    let elf_hdr = Elf64::parse(*image).map_err(|_| ExecError::BadMagic)?;
    validate_elf_hdr(elf_hdr, endian)?;

    let phdrs = load_program_hdrs(image, elf_hdr, endian)?;
    let analysis = analyze_program_hdrs(phdrs, endian)?;

    info!(
        "exec: image span {:x}..{:x}, {} PT_LOAD segments",
        analysis.min_vaddr,
        analysis.max_vaddr,
        analysis.load_count,
    );

    Ok(ExecMeta {
        entry: elf_hdr.e_entry.get(endian),
        phdrs,
        brk: analysis.max_vaddr,
    })
}

/// Phase 2: create a fresh address space, map the kernel half, the user stack
/// and every `PT_LOAD` segment. Produces a [`UserProgram`] ready to spawn.
fn load(meta: &ExecMeta, image: &mut Image) -> Result<UserProgram, ExecError> {
    let mut address_space = AddressSpace::new();

    map_kernel_shared(&mut address_space)?;
    init_stack(&mut address_space);

    for phdr in meta.phdrs {
        if phdr.p_type(LittleEndian) == elf::PT_LOAD {
            map_segment(image, &mut address_space, phdr, LittleEndian)?;
        }
    }

    Ok(UserProgram {
        entry: meta.entry as usize,
        sp: USER_STACK_TOP as usize,
        address_space,
    })
}

/// Run an ELF image: read+validate, load into a fresh address space, spawn a
/// user thread. Returns the spawned thread id.
pub fn run_elf(image: &mut Image) -> Result<usize, ExecError> {
    let meta = read_and_validate(image)?;
    let program = load(&meta, image)?;
    info!("exec: entry={:#018x} brk={:#018x}", meta.entry, meta.brk);
    Ok(spawn_user_program(program))
}
