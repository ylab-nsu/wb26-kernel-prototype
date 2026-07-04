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

    fn micros() -> u64 {
        riscv::register::time::read64() / 10
    }
}
