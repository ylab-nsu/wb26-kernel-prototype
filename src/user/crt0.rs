use core::arch::asm;

pub fn crt0() -> ! {
    let main: extern "C" fn();
    unsafe {
        asm!(
            "mv {0}, a0",
            out(reg) main,
        );
    }
    main();
    // println!("Main terminated");
    loop {riscv::asm::wfi()} // No exit() for now
}
