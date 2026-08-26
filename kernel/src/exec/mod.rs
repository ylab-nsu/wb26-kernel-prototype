//! exec module: load an executable image into a fresh address space.
//!
//! Port of the `exec` harness from `baremetal-test-exec`. Phase 1 covers only
//! ELF parsing + segment mapping (zero-copy from a RAM-resident file); the
//! stack builder (`create_elf_tables`) and the full `load_elf_binary` pipeline
//! come later once the address-space API gains `write_user`/`protect`.
//!
//! Naming follows Linux (`fs/exec.c`, `fs/binfmt_elf.c`):
//! - `elf` — the ELF loader (`load_elf_phdrs`, `elf_map`),
//! - `image` — abstracts the file bytes (`bprm->file` analogue).

pub mod elf;
pub mod image;
pub mod test;

use crate::allocator::AllocatorError;

pub const fn align_down(value: u64, align: u64) -> u64 {
    value & !(align - 1)
}

pub const fn align_up(value: u64, align: u64) -> u64 {
    align_down(value + align - 1, align)
}

pub const USER_STACK_SIZE: u64 = 16 * 4096;
pub const USER_STACK_TOP: u64 = 0x8C00_0000;

// errno values returned to user space (see `linux errno.h`).
const ENOEXEC: i32 = 8;
const ENOMEM: i32 = 12;
const EFAULT: i32 = 14;

/// Load failures. Port-able to errno via [`ExecError::errno`].
#[derive(Debug, Clone, Copy)]
pub enum ExecError {
    BadMagic,
    UnsupportedClass,
    UnsupportedEndian,
    UnsupportedMachine,
    UnsupportedType,
    DynUnsupported,
    BadPhdrEntSize,
    BadPhdrCount,
    NoLoadSegments,
    MemszLtFilesz,
    MisalignedSegment,
    InterpUnsupported,
    ImageFault,
    OutOfMemory,
    OutOfMappings,
}

impl ExecError {
    pub fn as_str(self) -> &'static str {
        match self {
            ExecError::BadMagic => "not an ELF file",
            ExecError::UnsupportedClass => "not a 64-bit ELF",
            ExecError::UnsupportedEndian => "not little-endian",
            ExecError::UnsupportedMachine => "wrong machine (need EM_RISCV)",
            ExecError::UnsupportedType => "not an executable (ET_EXEC)",
            ExecError::DynUnsupported => "ET_DYN/PIE not supported yet",
            ExecError::BadPhdrEntSize => "bad program header entry size",
            ExecError::BadPhdrCount => "bad program header count",
            ExecError::NoLoadSegments => "no PT_LOAD segments",
            ExecError::MemszLtFilesz => "segment memsz < filesz",
            ExecError::MisalignedSegment => "vaddr/offset page incongruence",
            ExecError::InterpUnsupported => "PT_INTERP not supported yet",
            ExecError::ImageFault => "file image read fault",
            ExecError::OutOfMemory => "out of physical memory",
            ExecError::OutOfMappings => "out of page-table mappings",
        }
    }

    /// The errno a real `execve` would surface to user space.
    pub fn errno(self) -> i32 {
        match self {
            // Malformed/unrecognised executable -> ENOEXEC.
            ExecError::BadMagic
            | ExecError::UnsupportedClass
            | ExecError::UnsupportedEndian
            | ExecError::UnsupportedMachine
            | ExecError::UnsupportedType
            | ExecError::DynUnsupported
            | ExecError::BadPhdrEntSize
            | ExecError::BadPhdrCount
            | ExecError::NoLoadSegments
            | ExecError::MemszLtFilesz
            | ExecError::MisalignedSegment
            | ExecError::InterpUnsupported => ENOEXEC,
            // Physical memory / mapping resources exhausted -> ENOMEM.
            ExecError::OutOfMemory | ExecError::OutOfMappings => ENOMEM,
            // Reads/writes against the image -> EFAULT.
            ExecError::ImageFault => EFAULT,
        }
    }
}

impl From<AllocatorError> for ExecError {
    fn from(e: AllocatorError) -> Self {
        match e {
            AllocatorError::NotEnoughMemory | AllocatorError::AlreadyOccupied => {
                ExecError::OutOfMemory
            }
            _ => ExecError::OutOfMemory,
        }
    }
}
