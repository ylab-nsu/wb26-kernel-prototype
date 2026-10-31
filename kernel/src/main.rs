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
pub mod threading;
pub mod time;
pub mod timer_queue;
pub mod vm;
pub mod services;

use alloc::sync::Arc;
use core::panic::PanicInfo;

use crate::arch::{traits::TargetPlatform, Platform};
use crate::boot::BootContext;
use crate::threading::init::setup_threads;
use crate::threading::trap::ExceptionKind;
use crate::services::interrupt::TrapHandler;
use crate::syscall::handle_syscall;
use crate::time::Duration;
use crate::timer_queue::{TimerCallback, TimerKind, TimerQueue};

static TIMER_QUEUE: TimerQueue = TimerQueue::new(arch::time::Alarm::get());

fn setup_reschedule_timer() {
    TIMER_QUEUE.enqueue(
        TimerCallback::Reschedule,
        Duration::MIN,
        TimerKind::Repeating(Duration::from_secs(1)),
    );
}

fn timer_handler() -> bool {
    TIMER_QUEUE.handle_timers()
}

fn exception_handler(e: ExceptionKind) {
    match e {
        ExceptionKind::Unknown => panic!("Unknown exception occured"),
        ExceptionKind::InstructionFault{ address } => panic!("InstructionFault {address:x}"),
    };
}

pub fn kernel_main(ctx: BootContext) -> ! {
    info!("Starting kernel (kernel_main())");

    info!("Initializing interrupts service");
    let interrupts_service = crate::services::interrupt::init().unwrap();
    let syscall_int_handle = interrupts_service.register_trap_handler(TrapHandler::SysCall(handle_syscall)).unwrap();
    let timer_int_handle = interrupts_service.register_trap_handler(TrapHandler::Timer(timer_handler)).unwrap();
    let exception_handle = interrupts_service.register_trap_handler(TrapHandler::Exception(exception_handler)).unwrap();

    let BootContext { address_space } = ctx;
    let kernel_address_space = Arc::new(address_space);

    setup_threads(kernel_address_space);
    setup_reschedule_timer();

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
