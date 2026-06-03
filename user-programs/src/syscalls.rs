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

// WriteScull { user_src: usize, scull_dst: usize, len: usize },
// ReadScull { user_dst: usize, scull_src: usize, len: usize },

pub fn write_scull(src: &[u8], scull_dst: usize) {
    unsafe {
        asm!(
        "ecall",
        in("a0") src.as_ptr(),
        in("a1") scull_dst,
        in("a2") src.len(),
        in("a7") 3,
        options(nostack),
        );
    }
}

pub fn read_scull(dst: &mut [u8], scull_src: usize) {
    unsafe {
        asm!(
        "ecall",
        in("a0") dst.as_ptr(),
        in("a1") scull_src,
        in("a2") dst.len(),
        in("a7") 4,
        options(nostack),
        );
    }
}

pub fn sched_yield() {
    unsafe {
        asm!(
        "ecall",
        in("a7") 5,
        options(nostack),
        );
    }
}
