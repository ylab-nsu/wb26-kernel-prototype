//! ELF64 image loading into a fresh address space.
//!
//! Three phases:
//! - [`read_and_validate`] parses and validates the ELF header and the
//!   program-header table (the file is read exactly twice),
//! - [`load`] maps the kernel half, the user stack and every `PT_LOAD` segment
//!   into a fresh address space, producing a [`UserProgram`],
//! - [`run_elf`] is the public entry: it chains both phases and spawns the
//!   program via `spawn_user_program`.
//!
//! Scope: static `ET_EXEC`, no `PT_INTERP`/`ET_DYN` yet (rejected with
//! dedicated errors). Non-`PT_LOAD` program headers are ignored.

use alloc::sync::Arc;

use object::read::elf::{FileHeader, ProgramHeader};
use object::{elf, LittleEndian};

use crate::arch::traits::{
    TargetAddressSpace, TargetMapping, TargetPhysicalAllocation, TargetPhysicalAllocator,
};
use crate::arch::{AddressSpace, KERNEL_MAPPINGS, PhysicalAllocator, VirtualAddress};
use crate::exec::image::Image;
use crate::exec::{align_down, align_up, ExecError, USER_STACK_SIZE, USER_STACK_TOP};
use crate::threading::init::{spawn_user_program, UserProgram};
use crate::vm::{MappingFlags, MappingPermissions};

pub(crate) type Endian = LittleEndian;
pub(crate) type Elf64 = elf::FileHeader64<Endian>;
pub(crate) type Phdr = elf::ProgramHeader64<Endian>;

/// Parsed and validated ELF metadata.
pub(crate) struct ExecMeta {
    entry: u64,
    phdrs: &'static [Phdr],
    /// First free address after the highest segment (future `brk`).
    brk: u64,
}

/// Read-only facts collected in the analysis pass.
pub(crate) struct PhdrsAnalysis {
    pub(crate) min_vaddr: u64,
    pub(crate) max_vaddr: u64,
    pub(crate) load_count: u32,
}

/// `p_flags` -> mapping permissions.
pub(crate) fn prot_from(p_flags: u32) -> MappingPermissions {
    MappingPermissions::new()
        .with_read(p_flags & elf::PF_R.0 != 0)
        .with_write(p_flags & elf::PF_W.0 != 0)
        .with_execute(p_flags & elf::PF_X.0 != 0)
}

/// Validate the ELF header.
pub(crate) fn validate_ehdr(ehdr: &Elf64, endian: Endian) -> Result<(), ExecError> {
    if ehdr.e_ident.magic != elf::ELFMAG {
        return Err(ExecError::BadMagic);
    }
    if ehdr.e_ident.class != elf::ELFCLASS64 {
        return Err(ExecError::UnsupportedClass);
    }
    if ehdr.e_ident.data != elf::ELFDATA2LSB {
        return Err(ExecError::UnsupportedEndian);
    }
    if ehdr.e_machine.get(endian) != elf::EM_RISCV {
        return Err(ExecError::UnsupportedMachine);
    }
    match ehdr.e_type.get(endian) {
        t if t == elf::ET_EXEC => {}
        t if t == elf::ET_DYN => return Err(ExecError::DynamicUnsupported),
        _ => return Err(ExecError::UnsupportedType),
    }
    if ehdr.e_phentsize.get(endian) as usize != core::mem::size_of::<Phdr>() {
        return Err(ExecError::BadPhdrEntSize);
    }
    let phnum = ehdr.e_phnum.get(endian);
    if phnum == 0 || phnum > 1024 {
        return Err(ExecError::BadPhdrCount);
    }
    Ok(())
}

/// Fetch the program-header table from the image.
pub(crate) fn load_phdrs(
    image: &mut Image,
    ehdr: &Elf64,
    endian: Endian,
) -> Result<&'static [Phdr], ExecError> {
    ehdr
        .program_headers(endian, *image)
        .map_err(|_| ExecError::ImageFault)
}

/// Validate one program header.
pub(crate) fn validate_phdr(phdr: &Phdr, endian: Endian) -> Result<(), ExecError> {
    let file_size = phdr.p_filesz(endian);
    let mem_size = phdr.p_memsz(endian);

    if mem_size < file_size {
        return Err(ExecError::MemszLtFilesz);
    }

    if phdr.p_vaddr(endian) % 4096 != phdr.p_offset(endian) % 4096 {
        return Err(ExecError::MisalignedSegment);
    }

    Ok(())
}

/// Page-aligned bounds of the region to allocate for a segment.
pub(crate) fn vaddr_alloc_bounds(phdr: &Phdr, endian: Endian) -> (u64, u64) {
    let vaddr_alloc_start = align_down(
        phdr.p_vaddr(endian), 
        4096
    );
    let vaddr_alloc_end = align_up(
        phdr.p_vaddr(endian) + phdr.p_memsz(endian), 
        4096
    );
    
    (vaddr_alloc_start, vaddr_alloc_end)
}

/// Single pass collecting image span and load count.
pub(crate) fn analyze_phdrs(phdrs: &[Phdr], endian: Endian) -> Result<PhdrsAnalysis, ExecError> {
    let mut analysis = PhdrsAnalysis {
        min_vaddr: u64::MAX,
        max_vaddr: 0,
        load_count: 0,
    };

    for phdr in phdrs {
        match phdr.p_type(endian) {
            elf::PT_LOAD => {
                validate_phdr(phdr, endian)?;
                let (vaddr_alloc_start, vaddr_alloc_end) = vaddr_alloc_bounds(phdr, endian);
                analysis.min_vaddr = analysis.min_vaddr.min(vaddr_alloc_start);
                analysis.max_vaddr = analysis.max_vaddr.max(vaddr_alloc_end);
                analysis.load_count += 1;
            }
            elf::PT_INTERP => {
                return Err(ExecError::InterpreterUnsupported);
            }
            _ => {}
        }
    }

    if analysis.load_count == 0 {
        return Err(ExecError::NoLoadSegments);
    }
    Ok(analysis)
}

/// Load one `PT_LOAD`: copy file bytes, zero bss, map.
pub(crate) fn map_segment<A: TargetAddressSpace>(
    image: &mut Image,
    address_space: &mut A,
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

    if filesz > 0 {
        image.read_into(
            phdr.p_offset(endian),
            (phys_alloc_start + segment_offset) as *mut u8,
            filesz as usize
        )?;
    }

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
        Arc::new(alloc),
        perms,
        MappingFlags::new().with_user(true),
    );
    Ok(())
}

/// Share the kernel half into a process address space.
pub fn map_kernel_shared(address_space: &mut AddressSpace) -> Result<(), ExecError> {
    for mapping in KERNEL_MAPPINGS
        .get()
        .expect("kernel mappings not initialized")
    {
        let vaddr: usize = mapping.virt_addr().try_into().unwrap();
        address_space.map_shared(mapping, mapping.virt_addr());
        info!("exec: shared kernel {vaddr:#018x} (size {:#x})", mapping.size());
    }

    Ok(())
}

/// Map the user stack.
pub(crate) fn init_stack<A: TargetAddressSpace>(address_space: &mut A) {
    let stack_alloc = PhysicalAllocator::alloc_contiguous(USER_STACK_SIZE as usize)
        .expect("alloc stack");

    address_space.map(
        VirtualAddress::try_from((USER_STACK_TOP - USER_STACK_SIZE) as usize).unwrap(),
        Arc::new(stack_alloc),
        MappingPermissions::rw(),
        MappingFlags::new().with_user(true),
    );
}

/// Parse and validate the ELF header and program headers.
pub(crate) fn read_and_validate(image: &mut Image) -> Result<ExecMeta, ExecError> {
    let endian = LittleEndian;

    let ehdr = Elf64::parse(*image).map_err(|_| ExecError::BadMagic)?;
    validate_ehdr(ehdr, endian)?;

    let phdrs = load_phdrs(image, ehdr, endian)?;
    let analysis = analyze_phdrs(phdrs, endian)?;

    info!(
        "exec: image span {:x}..{:x}, {} PT_LOAD segments",
        analysis.min_vaddr,
        analysis.max_vaddr,
        analysis.load_count,
    );

    Ok(ExecMeta {
        entry: ehdr.e_entry.get(endian),
        phdrs,
        brk: analysis.max_vaddr,
    })
}

/// Load the image into a fresh address space: map the stack and every
/// `PT_LOAD` segment. The kernel half is mapped separately (see `run_elf`).
pub(crate) fn load<A: TargetAddressSpace>(
    meta: &ExecMeta,
    image: &mut Image,
    address_space: A,
) -> Result<UserProgram<A>, ExecError> {
    let mut address_space = address_space;
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

/// Run an ELF image and spawn a user thread.
pub fn run_elf(image: &mut Image) -> Result<usize, ExecError> {
    let meta = read_and_validate(image)?;

    let mut address_space = AddressSpace::new();
    map_kernel_shared(&mut address_space)?;

    let program = load(&meta, image, address_space)?;
    info!("exec: entry={:#018x} brk={:#018x}", meta.entry, meta.brk);
    Ok(spawn_user_program(program))
}
