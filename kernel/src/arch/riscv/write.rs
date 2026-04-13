use core::fmt::Write;
use riscv::_export::critical_section;

pub struct SbiWriter;

impl SbiWriter {
    pub fn new() -> Self {
        SbiWriter
    }
}

impl Write for SbiWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        critical_section::with(|_| {
            for b in s.bytes() {
                sbi::debug_console::write_byte(b).unwrap();
            }
        });

        Ok(())
    }
}
