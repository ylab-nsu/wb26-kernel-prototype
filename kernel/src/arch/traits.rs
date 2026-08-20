use crate::{
    allocator::AllocatorError,
    arch::{
        Mapping, PhysicalAddress, PhysicalAllocation, PlatformDuration, PlatformInstant,
        VirtualAddress,
    },
    vm::{MappingFlags, MappingPermissions},
};

pub trait TargetPlatform {
    fn init();
    fn ipi();
    fn sleep();
    fn shutdown();
    fn wfi();
    unsafe fn ei();
    fn di();
    fn micros() -> u64;
    // ...

    // interrupts/excs (hooks)
    // timers

    // todo: remove after switching to a sane memory manager
    fn get_user_va_offset() -> usize;
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

    // todo another interface
    fn set_arg0(&mut self, value: usize);
}

pub trait TargetContext: Default + Clone {
    fn with_ra(self, ra: usize) -> Self;
    fn with_sp(self, sp: usize) -> Self;
}

pub trait TargetInstant: Copy {
    fn now() -> Self;
}

pub struct TargetTimerCallbackContext {
    pub handle_time: PlatformInstant,
    pub target_time: PlatformInstant,
}

pub type TargetTimerImmediateCallback = fn(TargetTimerCallbackContext);

#[derive(Clone)]
pub enum TargetTimerCallback {
    Reschedule,
    // Inside interrupt
    Immediate {
        callback: TargetTimerImmediateCallback,
    },
    // TODO: Elsewhere
    Soft {
        callback: fn(),
    },
}

impl TargetTimerCallback {
    pub fn immediate(callback: TargetTimerImmediateCallback) -> Self {
        TargetTimerCallback::Immediate { callback }
    }
}

pub trait TargetTimerQueue {
    fn add_oneshot_timer(delta: PlatformDuration, callback: TargetTimerCallback);
    fn add_repeating_timer(interval: PlatformDuration, callback: TargetTimerCallback);
}
