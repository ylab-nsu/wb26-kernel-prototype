#![no_std]
#![no_main]
#![warn(unsafe_op_in_unsafe_fn)]

extern crate alloc;

#[macro_use]
mod print;

pub mod allocator;
pub mod arch;
pub mod boot;
pub mod drivers;
pub mod sync;
mod syscall;
mod exception;
pub mod threading;
pub mod time;
pub mod timer_queue;
pub mod vm;
pub mod services;

use alloc::{
    sync::Arc,
    boxed::Box,
};
use core::panic::PanicInfo;

use crate::arch::{
    traits::TargetPlatform, Platform,
    traits::TargetInterruptController,
    InterruptController,
};
use crate::boot::BootContext;
use crate::threading::init::setup_threads;
use crate::services::interrupt::{
    TrapHandler,
    InterruptService,
};
use crate::syscall::handle_syscall;
use crate::exception::handle_exception;
use crate::time::Duration;
use crate::timer_queue::{TimerCallback, TimerKind, TimerQueue};
use static_cell::StaticCell;

static TIMER_QUEUE: StaticCell<TimerQueue> = StaticCell::new();
static INTERRUPT_SERVICE: StaticCell<InterruptService> = StaticCell::new();

pub fn kernel_main(ctx: BootContext) -> ! {
    info!("Starting kernel (kernel_main())");

    let BootContext { address_space } = ctx;
    let kernel_address_space = Arc::new(address_space);

    // Init all kernel services
    let timer_queue = TIMER_QUEUE.init(TimerQueue::new(arch::time::Alarm::get()));
    let interrupt_service = INTERRUPT_SERVICE.init(InterruptService::new());

    info!("Initializing interrupts service");

    let syscall_handler = Box::new(|ctx| handle_syscall(ctx));
    let timer_handler = Box::new(|| timer_queue.handle_timers());
    let exception_handler = Box::new(|e| handle_exception(e));
    let global_trap_handler = Box::new(|cause| interrupt_service.handle_trap(cause));

    let _syscall_int_handle = interrupt_service
        .register_trap_handler(TrapHandler::SysCall(syscall_handler))
        .unwrap();
    let _timer_int_handle = interrupt_service
        .register_trap_handler(TrapHandler::Timer(timer_handler))
        .unwrap();
    let _exception_handle = interrupt_service
        .register_trap_handler(TrapHandler::Exception(exception_handler))
        .unwrap();
    InterruptController::get()
        .set_global_trap_handler(global_trap_handler)
        .unwrap();

    info!("Initializing timer queue");
    timer_queue.enqueue(
        TimerCallback::Reschedule,
        Duration::MIN,
        TimerKind::Repeating(Duration::from_secs(1)),
    );

    setup_threads(kernel_address_space);

    unsafe {
        Platform::enable_interrupts();
    }

    error!("You returned to the thread 0, this shouldn't happen!");

    loop {
        Platform::wfi();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("Something went wrong.");
    error!("{}", info);
    error!("Shutting down...");
    riscv::asm::wfi();
    error!("After shutting down...");
    // sbi::system_reset::system_reset(ResetType::, ResetReason::SystemFailure).unwrap();

    loop {
        riscv::asm::wfi();
    }
}
