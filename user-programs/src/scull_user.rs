use crate::syscalls::{print_str, read_scull, sched_yield, write_scull};

// User of SCULL device (Simple Character Utility for Loading Localities)

#[export_name = "scull_user"]
pub extern "C" fn scull_user() {
    let mut arr: [u8; 128] = [0; _];
    for i in 0..128 {
        arr[i] = i as u8;
    }
    sbi_println!("scull_user wants to write data to device");
    write_scull(&arr, 50);
    sched_yield();

    let mut in_buf: [u8; _] = [64; 20];
    sbi_println!("scull_user wants to read data from device");
    read_scull(&mut in_buf, 137);
    sbi_println!("but before we yield, the read syscall is still in queue");
    sbi_println!("if we clone the buffer and print, the data is still uninitialized");
    let in_buf_copy = in_buf.clone();
    print_str(unsafe { core::str::from_utf8_unchecked(&in_buf_copy) });
    sched_yield(); // In current implementation you should yield for your syscalls to be processed

    sbi_println!("scull_user now has right data in the buffer");
    print_str(unsafe { core::str::from_utf8_unchecked(&in_buf) });
    sched_yield();

    return;
}
