use crate::syscalls::print_number;

#[export_name = "user1"]
pub extern "C" fn user1() {
    println!("Hello, world!");
    let mut a = 0i64;
    for _ in 0i64..20000000 {
        a += 1;
    }
    println!("! Result: {}", a);
    print_number(a as i32);
    print_number(a as i32);
    print_number(a as i32);
}
