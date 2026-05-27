use core::arch::asm;

pub(crate) fn print_number(number: i32) {
    unsafe {
        asm!(
        "ecall",
        in("a0") number,
        in("a7") 1,
        options(nostack),
        );
    }
}
