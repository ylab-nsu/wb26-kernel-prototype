use core::fmt::Write;
use core::ptr;
use riscv::_export::critical_section;

pub(crate) struct MyPrinter;

impl Write for MyPrinter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        // uart_print(s);
        // return Ok(());

        critical_section::with(|_| {
            for b in s.bytes() {
                sbi::debug_console::write_byte(b).unwrap();
            }
        });

        // let sa = format!("{:x}\n", s.len());
        // for b in sa.bytes() {
        //     sbi::debug_console::write_byte(b).unwrap();
        // }

        // let ptr = s.as_ptr() as usize;
        // let addr = if ptr > 0xffff_ffff_0000_0000 {
        //     ptr - 0xffff_ffff_0000_0000
        // } else {
        //     ptr
        // };
        //
        // unsafe {
        //     sbi::debug_console::write(
        //         PhysicalAddress::new(addr),
        //         PhysicalAddress::new(0),
        //         s.len()
        //     ).unwrap();
        // }

        // for b in s.bytes() {
        //     // sbi::debug_console::write_byte(b);
        //     if let Err(_) = sbi::debug_console::write_byte(b) {
        //         return Err(core::fmt::Error);
        //     }
        //     for _i in 0..100_000 {}
        // }

        Ok(())
    }
}

fn _uart_print(message: &str) {
    const UART: *mut u8 = 0x10000000 as *mut u8;

    for b in message.bytes() {
        unsafe {
            if b.is_ascii() {
                ptr::write_volatile(UART, b);
            }
        }

        for _i in 0..100_000 {}
    }

    // for c in message.chars() {
    //     if c.is_ascii() {
    //         unsafe {
    //             ptr::write_volatile(UART, c as u8);
    //         }
    //     } else {
    //         unsafe {
    //             ptr::write_volatile(UART, 0x21);
    //         }
    //     }
    // }
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
