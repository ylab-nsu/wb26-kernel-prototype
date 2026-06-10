use riscv::_export::critical_section;

use crate::arch::traits::TargetDebugWriter;

pub struct SbiWriter;

impl TargetDebugWriter for SbiWriter {
    fn new() -> Self {
        SbiWriter
    }
}

impl core::fmt::Write for SbiWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        critical_section::with(|_| {
            for b in s.bytes() {
                sbi::debug_console::write_byte(b).unwrap();
            }
        });

        Ok(())
    }
}
