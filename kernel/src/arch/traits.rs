use core::fmt::Write;

use crate::vm::{MapperError, Mapping, MappingFlags};

pub trait TargetPlatform {
    type PlatformWriter;

    fn init();
    fn get_writer() -> impl Write;
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
