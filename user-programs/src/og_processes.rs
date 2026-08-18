use crate::syscalls::print_number;

#[export_name = "og_process1"]
pub extern "C" fn og_process1() {
    let mut i: i32 = 10000;
    loop {
        sbi_println!("og_process1 calls print_number({i})");
        print_number(i);
        for _ in 1..500_000 {}
        i += 1;
    }
}

#[export_name = "og_process2"]
pub extern "C" fn og_process2() {
    let mut i: i32 = 10000;
    loop {
        sbi_println!("og_process2 calls print_number({i})");
        print_number(i);
        for _ in 1..1_000_000 {}
        i += 1;
    }
}

#[export_name = "og_process3"]
pub extern "C" fn og_process3() {
    let mut i: i32 = 10000;
    loop {
        sbi_println!("og_process3 calls print_number({i})");
        print_number(i);
        for _ in 1..2000_000 {}
        i += 1;
    }
}
