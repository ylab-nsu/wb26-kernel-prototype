use crate::arch::traits::TargetDebugWriter;
use riscv::interrupt;

pub struct SbiWriter;

impl TargetDebugWriter for SbiWriter {
    fn new() -> Self {
        SbiWriter
    }
}

impl core::fmt::Write for SbiWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        interrupt::free(|| {
            for b in s.bytes() {
                sbi::debug_console::write_byte(b).unwrap();
            }
        });

        Ok(())
    }
}
