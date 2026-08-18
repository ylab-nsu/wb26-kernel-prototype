use crate::syscalls;
use core::fmt::Write;

pub struct SyscallPrinter;

impl Write for SyscallPrinter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        syscalls::print_str(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        use crate::print::SyscallPrinter;
        write!(SyscallPrinter, $($arg)*).ok();
    }};
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        use crate::print::SyscallPrinter;
        writeln!(SyscallPrinter, $($arg)*).ok();
    }};
}

//////////

pub struct SbiPrinter;

impl Write for SbiPrinter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            sbi::debug_console::write_byte(b).unwrap();
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! sbi_print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        use crate::print::SbiPrinter;
        write!(SbiPrinter, $($arg)*).ok();
    }};
}

#[macro_export]
macro_rules! sbi_println {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        use crate::print::SbiPrinter;
        writeln!(SbiPrinter, $($arg)*).ok();
    }};
}
