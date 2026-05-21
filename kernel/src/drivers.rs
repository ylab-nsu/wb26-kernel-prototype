pub(crate) extern "C" fn driver_task() -> ! {
    loop {
        // Demonstration that driver works in S-mode and can read registers
        let sstatus = riscv::register::sstatus::read();
        println!("Me driver me read sstatus: {sstatus:x?}");
        for _ in 1..1_000_000 {}
        let satp = riscv::register::satp::read();
        println!("Me driver me read satp: {satp:x?}");
        for _ in 1..1_000_000 {}
        let stvec = riscv::register::stvec::read();
        println!("Me driver me read stvec: {stvec:x?}");
        for _ in 1..1_000_000 {}
    }
}
