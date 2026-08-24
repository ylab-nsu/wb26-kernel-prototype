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
pub mod vm;
mod drivers_;

use core::panic::PanicInfo;
use core::ptr::read;
use core::result;
use riscv::_export::critical_section;

use crate::arch::{traits::TargetPlatform, Platform};
use crate::drivers_::uart::UART;
use crate::drivers_::uart::{TriggerLevel, DataBits, StopBits, Parity};

use crate::boot::BootContext;
use crate::threading::init::setup_threads;

pub fn kernel_main(_ctx: BootContext) -> ! {
    info!("Starting kernel (kernel_main())");
	
    setup_threads();

	critical_section::with(|cs|{
		let mut uart = UART.borrow(cs).borrow_mut();
		uart.init(TriggerLevel::Fourteen);
		uart.enable_rx_interrupt();
		uart.set_baud_rate(115_200);
		uart.set_line_config(DataBits::Eight, Parity::Even, StopBits::One);

		if uart.write(b"Hello world\n").is_err(){
			println!("Write error");
		};
	});

    unsafe {
         Platform::ei();
    }

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
