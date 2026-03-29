#[no_mangle]
pub extern "C" fn main() {
    println!("Hello, world!");
    let mut a = 0i64;
    for _ in 0i64..20000000 {
        a += 1;
    }
    println!("! Result: {}", a);
}