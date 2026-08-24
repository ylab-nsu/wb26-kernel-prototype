use crate::{
    allocator::AllocatorError,
    arch::{Mapping, PhysicalAddress, PhysicalAllocation, PlatformDuration, VirtualAddress},
    timers::{TimerCallback, TimerHandle},
    vm::{MappingFlags, MappingPermissions},
};
use alloc::sync::Arc;

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

pub trait TargetTimerQueue {
    //! Trait with functions to add timers to the timer queue
    //!
    //! # Examples of timers
    //! ```
    //! // Simple repeating timer
    //! TimerQueue::add_repeating_timer(
    //!     Duration::from_secs(1).into(),
    //!     TimerCallback::immediate(|_| info!("1 Second timer")),
    //! );
    //! // One shot timer
    //! TimerQueue::add_oneshot_timer(
    //!     Duration::from_secs(10).into(),
    //!     TimerCallback::immediate(|_| info!("10 second oneshot timer")),
    //! );
    //! // Repeating timer with inner state
    //! TimerQueue::add_repeating_timer(
    //!     Duration::from_secs(3).into(),
    //!     TimerCallback::immediate(|_| {
    //!         static COUNT: Mutex<u32> = Mutex::new(0);
    //!         let mut count = COUNT.lock();
    //!         *count += 1;
    //!         info!(
    //!             "3 Second stateful timer {}",
    //!             count
    //!         )
    //!     }),
    //! );
    //! // Reschedule timer
    //! TimerQueue::add_repeating_timer(Duration::from_secs(1).into(), TimerCallback::Reschedule);
    //! // One shot repeating timer
    //! fn oneshot_repeating_callback(_: TimerCallbackContext) {
    //!     info!("One shot repeating timer");
    //!     TimerQueue::add_oneshot_timer(
    //!         Duration::from_secs(2).into(),
    //!         TimerCallback::immediate(oneshot_repeating_callback),
    //!     );
    //! }
    //! TimerQueue::add_oneshot_timer(
    //!     Duration::from_secs(2).into(),
    //!     TimerCallback::immediate(oneshot_repeating_callback),
    //! );
    //! // One shot timer with capture
    //! let to_capture = 5;
    //! TimerQueue::add_oneshot_timer(
    //!     Duration::from_secs(4).into(),
    //!     TimerCallback::immediate(move |_| {
    //!         info!(
    //!             "-------------------------- One shot timer with capture {}",
    //!             to_capture
    //!         );
    //!     }),
    //! );
    //! // One shot timer with mutable capture
    //! let mut to_capture_mutable = 10;
    //! TimerQueue::add_repeating_timer(
    //!     Duration::from_secs(4).into(),
    //!     TimerCallback::immediate(move |_| {
    //!         info!(
    //!             "-------------------------- One shot timer with mut capture {}",
    //!             to_capture_mutable
    //!         );
    //!         to_capture_mutable += 1;
    //!     }),
    //! );
    //! // Stop timer with handle
    //! let handle = TimerQueue::add_repeating_timer(Duration::from_secs(2).into(), TimerCallback::immediate(|_| {
    //!     info!("1 second timer");
    //! }));
    //! TimerQueue::add_oneshot_timer(Duration::from_secs(5).into(), TimerCallback::immediate( move |_| {
    //!     handle.stop();
    //!     info!("timer stopped");
    //! }));
    //! ```
    fn add_oneshot_timer(delta: PlatformDuration, callback: TimerCallback) -> Arc<TimerHandle>;
    fn add_repeating_timer(interval: PlatformDuration, callback: TimerCallback)
        -> Arc<TimerHandle>;
}
