use core::fmt::Write;

pub struct MyPrinter;

impl Write for MyPrinter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        // critical_section::with(|_| {
            for b in s.bytes() {
                sbi::debug_console::write_byte(b).unwrap();
            }
        // });

        Ok(())
    }
}
