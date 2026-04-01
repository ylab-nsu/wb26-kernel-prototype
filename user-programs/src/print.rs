use core::fmt::Write;
use core::ptr;
use riscv::_export::critical_section;

pub struct MyPrinter;

impl Write for MyPrinter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        critical_section::with(|_| {
            for b in s.bytes() {
                sbi::debug_console::write_byte(b).unwrap();
            }
        });

        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        use crate::print::MyPrinter;
        write!(MyPrinter, $($arg)*).ok();
    }};
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        use crate::print::MyPrinter;
        writeln!(MyPrinter, $($arg)*).ok();
    }};
}
