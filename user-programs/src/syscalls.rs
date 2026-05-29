use core::arch::asm;

pub fn print_number(number: i32) {
    unsafe {
        asm!(
        "ecall",
        in("a0") number,
        in("a7") 1,
        options(nostack),
        );
    }
}

pub fn print_str(s: &str) {
    unsafe {
        asm!(
        "ecall",
        in("a0") s.as_ptr(),
        in("a1") s.len(),
        in("a7") 2,
        options(nostack),
        );
    }
}
