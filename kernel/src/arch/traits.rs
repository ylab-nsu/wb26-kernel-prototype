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
