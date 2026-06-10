#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        use $crate::arch::traits::TargetDebugWriter;
        write!($crate::arch::DebugWriter::new(), $($arg)*).ok();
    }};
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        use $crate::arch::traits::TargetDebugWriter;
        writeln!($crate::arch::DebugWriter::new(), $($arg)*).ok();
    }};
}

macro_rules! rich_println {
    ($prefix:tt, $($arg:tt)*) => {
        println!("{:8} {}:{}:{} - {}", $prefix, file!(), line!(), column!(), format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        rich_println!("[INFO]", $($arg)*);
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        rich_println!("[WARN]", $($arg)*);
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        rich_println!("[ERROR]", $($arg)*);
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        rich_println!("[DEBUG]", $($arg)*);
    };
}
