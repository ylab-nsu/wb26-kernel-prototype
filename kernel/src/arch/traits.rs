use alloc::sync::Arc;

use crate::{
    allocator::AllocatorError,
    arch::{Mapping, PhysicalAddress, PhysicalAllocation, VirtualAddress},
    vm::{MappingFlags, MappingPermissions},
};

pub trait TargetPlatform {
    fn init();
    fn ipi();
    fn sleep();
    fn shutdown();
    fn wfi();
    fn disable_interrupts();
    unsafe fn enable_interrupts();
    fn interrupts_enabled() -> bool;
    unsafe fn set_interrupts_enabled(enable: bool);
    fn interrupt_free<F, R>(f: F) -> R
    where
        F: FnOnce() -> R;
    fn micros() -> u64;
    // ...

    // interrupts/excs (hooks)
    // timers

    // todo: remove after switching to a sane memory manager
    fn get_user_va_offset() -> usize;
}

pub trait TargetAddressSpace {
    fn map(
        &self,
        virt_addr: VirtualAddress,
        phys_alloc: Arc<PhysicalAllocation>,
        permissions: MappingPermissions,
        flags: MappingFlags,
    ) -> Mapping;

    unsafe fn unmap(&self, mapping: &impl TargetMapping);

    fn map_shared(&self, src: &impl TargetMapping, dest_vaddr: VirtualAddress) -> Mapping;

    unsafe fn switch(&self);
}

pub trait TargetMapping {
    fn virt_addr(&self) -> VirtualAddress;
    fn phys_addr(&self) -> PhysicalAddress;
    fn size(&self) -> usize;
    fn permissions(&self) -> MappingPermissions;
    fn flags(&self) -> MappingFlags;
    fn alloc(&self) -> &Arc<PhysicalAllocation>;
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
    fn alloc_contiguous_aligned(
        size: usize,
        alignment: usize,
    ) -> Result<PhysicalAllocation, AllocatorError>;
    fn alloc_contiguous_at(
        addr: PhysicalAddress,
        size: usize,
    ) -> Result<PhysicalAllocation, AllocatorError>;
}

pub trait TargetPhysicalAllocation: core::fmt::Debug + core::fmt::Display {
    fn addr(&self) -> PhysicalAddress;
    fn size(&self) -> usize;
}

pub trait TargetTrapFrame: Default + Clone {
    fn with_pc(self, pc: usize) -> Self;
    fn with_sp(self, sp: usize) -> Self;
    fn with_arg0(self, arg0: usize) -> Self;

    // todo another interface
    fn set_pc(&mut self, value: usize);
    fn set_sp(&mut self, value: usize);
    fn set_arg0(&mut self, value: usize);
}

/// Memory where callee-saved registers are stored when switching threads.
/// After being initialized, can only be used as the argument for `switch_thread`.
/// The first switch to a context corresponds to the start of a new thread.
pub trait TargetContext: Clone {
    /// Must provide valid initial state (destination) so after `switch_thread`
    /// a new thread will correctly continue its execution.
    ///
    /// The recommended destination is the point in the trap handler
    /// right after the Rust code executed and returned into assembly
    /// (_initial_return_trap).
    unsafe fn new(initial_pc: usize, initial_sp: usize) -> Self;
    /// Get data pointer to give to assembly code that stores and reads registers.
    fn get_ptr(&mut self) -> *mut core::ffi::c_void;
}
