use crate::syscalls::{print_str, read_scull, sched_yield, write_scull};

#[export_name = "scull_user"]
pub extern "C" fn scull_user() {
    let mut arr: [u8; 128] = [0; _];
    for i in 0..128 {
        arr[i] = i as u8;
    }
    write_scull(&arr, 50);
    println!("scull_user wants to write data");
    sched_yield();

    let mut in_buf: [u8; _] = [64; 20];
    read_scull(&mut in_buf, 137);
    println!("scull_user wants to read data");
    let in_buf_copy = in_buf.clone();
    print_str(unsafe { core::str::from_utf8_unchecked(&in_buf_copy) });
    sched_yield();

    println!("scull_user should already have data");
    print_str(unsafe { core::str::from_utf8_unchecked(&in_buf) });
    sched_yield();

    return;
}
