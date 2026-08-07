use crate::arch::traits::TargetPlatform;

pub struct RiscvPlatform;

impl TargetPlatform for RiscvPlatform {
    fn init() {
        todo!()
    }

    fn ipi() {
        todo!()
    }

    fn sleep() {
        todo!()
    }

    fn shutdown() {
        todo!()
    }

    fn wfi() {
        riscv::asm::wfi();
    }

    unsafe fn ei() {
        riscv::interrupt::enable();
    }

    fn di() {
        riscv::interrupt::disable();
    }

    fn micros() -> u64 {
        riscv::register::time::read64() / 10
    }
}
