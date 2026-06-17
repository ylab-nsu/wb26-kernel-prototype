use crate::vm::{MapperError, Mapping, MappingFlags};

pub trait TargetPlatform {
    fn init();
    fn ipi();
    fn sleep();
    fn shutdown();
    fn wfi();
    // ...

    // interrupts/excs (hooks)
    // timers
}

pub trait TargetAddressSpace {
    type PhysicalAddress;
    type VirtualAddress;

    fn map(&mut self, vaddr: Self::VirtualAddress, paddr: Self::PhysicalAddress, flags: MappingFlags) -> Result<Mapping, MapperError>;
    
    unsafe fn unmap(&mut self, mapping: &Mapping);

    unsafe fn switch(&self);
}

pub trait TargetDebugWriter: core::fmt::Write {
    fn new() -> Self;
}

pub trait TargetAddress:
    PartialEq
    + Eq
    + PartialOrd
    + Ord
    + Copy
    + Sized
    + TryFrom<usize, Error: core::fmt::Debug>
    + TryInto<usize, Error: core::fmt::Debug>
    + core::fmt::Binary
    + core::fmt::LowerHex
    + core::fmt::UpperHex
    + core::fmt::Octal
    + core::fmt::Pointer
{
    fn byte_add(self, count: usize) -> Self;
    fn byte_sub(self, count: usize) -> Self;
    fn byte_offset(self, count: isize) -> Self;
    fn byte_offset_from(&self, origin: Self) -> isize;
    fn byte_offset_from_unsigned(self, origin: Self) -> usize;
}
