#![no_std]
#![no_main]

#[macro_use]
mod print;
mod user1;
mod og_processes;


use core::arch::asm;
use core::panic::PanicInfo;

#[export_name = "crt0"]
pub fn crt0() -> ! {
    let main: extern "C" fn();
    unsafe {
        asm!(
            "mv {0}, a0",
            out(reg) main,
        );
    }
    main();
    println!("Main terminated");
    loop {riscv::asm::wfi()} // No exit() for now
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        riscv::asm::wfi();
    }
}
