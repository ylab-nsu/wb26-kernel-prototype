//! exec module: load an executable image into a fresh address space.
//!
//! - `elf` — the ELF loader,
//! - `image` — abstracts the file bytes.

pub mod elf;
pub mod image;
pub mod test;

use crate::allocator::AllocatorError;

/// Align `value` down to a multiple of `align`.
pub const fn align_down(value: u64, align: u64) -> u64 {
    value & !(align - 1)
}

/// Align `value` up to a multiple of `align`.
pub const fn align_up(value: u64, align: u64) -> u64 {
    align_down(value + align - 1, align)
}

pub const USER_STACK_SIZE: u64 = 16 * 4096;
pub const USER_STACK_TOP: u64 = 0x8C00_0000;

/// Load failures.
#[derive(Debug, Clone, Copy)]
pub enum ExecError {
    BadMagic,
    UnsupportedClass,
    UnsupportedEndian,
    UnsupportedMachine,
    UnsupportedType,
    DynamicUnsupported,
    BadPhdrEntSize,
    BadPhdrCount,
    NoLoadSegments,
    MemszLtFilesz,
    MisalignedSegment,
    InterpreterUnsupported,
    ImageFault,
    OutOfMemory,
    OutOfMappings,
    InvalidAddress,
}

impl From<AllocatorError> for ExecError {
    fn from(e: AllocatorError) -> Self {
        match e {
            AllocatorError::NotEnoughMemory | AllocatorError::AlreadyOccupied => {
                ExecError::OutOfMemory
            }
            AllocatorError::InvalidAddress => ExecError::InvalidAddress,
            _ => ExecError::OutOfMemory,
        }
    }
}
