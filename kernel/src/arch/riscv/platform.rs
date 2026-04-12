use crate::arch::{riscv::write::MyPrinter, traits::TargetPlatform};

pub struct RiscvPlatform;

impl TargetPlatform for RiscvPlatform {
    type PlatformWriter = MyPrinter;

    fn init() {
        todo!()
    }

    fn get_writer() -> impl core::fmt::Write {
        MyPrinter
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
        todo!()
    }
}
