#![no_std]
#![no_main]

#[macro_use]
mod print;
mod og_processes;
mod syscalls;
mod user1;

use core::arch::asm;
use core::panic::PanicInfo;

#[export_name = "crt0"]
pub fn crt0() -> ! {
    let main: extern "C" fn();
    unsafe {
        asm!(
            "mv {0}, a0",
            out(reg) main,
            options(nostack),
        );
    }
    main();
    println!("Main terminated");
    loop {} // No exit() for now
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
