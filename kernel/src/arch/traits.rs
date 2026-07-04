use crate::{
    allocator::AllocatorError, arch::{Mapping, PhysicalAddress, PhysicalAllocation, VirtualAddress}, vm::{MappingFlags, MappingPermissions},
};

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
    fn map(
        &mut self,
        virt_addr: VirtualAddress,
        phys_alloc: PhysicalAllocation,
        permissions: MappingPermissions,
        flags: MappingFlags,
    ) -> Mapping;

    unsafe fn switch(&self);
}

pub trait TargetMapping {
    fn virt_addr(&self) -> VirtualAddress;
    fn phys_addr(&self) -> PhysicalAddress;
    fn size(&self) -> usize;
    fn permissions(&self) -> MappingPermissions;
    fn flags(&self) -> MappingFlags;
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

pub trait TargetPhysicalAllocator {
    fn alloc_contiguous(size: usize) -> Result<PhysicalAllocation, AllocatorError>;
    fn alloc_contiguous_aligned(size: usize, alignment: usize) -> Result<PhysicalAllocation, AllocatorError>;
    fn alloc_contiguous_at(addr: PhysicalAddress, size: usize) -> Result<PhysicalAllocation, AllocatorError>;
}

pub trait TargetPhysicalAllocation: core::fmt::Debug + core::fmt::Display {
    fn addr(&self) -> PhysicalAddress;
    fn size(&self) -> usize;
}
