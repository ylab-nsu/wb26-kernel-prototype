use crate::syscalls::print_number;

#[export_name = "process1"]
pub extern "C" fn process1() {
    loop {
        // println!("Process1 1");
        print_number(1001);
        println!("1001");
        for _ in 1..200_000 {}
        // println!("Process1 2");
        // print_number(1002);
        for _ in 1..200_000 {}
        // println!("Process1 3");
        // print_number(1003);
        for _ in 1..200_000 {}
        // println!("Process1 4");
        // print_number(1004);
        for _ in 1..200_000 {}
        // println!("Process1 5");
        // print_number(1005);
        for _ in 1..200_000 {}
        // println!("Process1 6");
        // print_number(1006);
        for _ in 1..200_000 {}
    }
}

#[export_name = "process2"]
pub extern "C" fn process2() {
    loop {
        // println!("Process2 1");
        print_number(2001);
        println!("2001");
        for _ in 1..500_000 {}
        // println!("Process2 2");
        // print_number(2002);
        for _ in 1..500_000 {}
        // println!("Process2 3");
        print_number(2003);
        println!("2003");
        for _ in 1..500_000 {}
        // println!("Process2 4");
        // print_number(2004);
        for _ in 1..500_000 {}
        // println!("Process2 5");
        print_number(2005);
        println!("2005");
        for _ in 1..500_000 {}
        // println!("Process2 6");
        // print_number(2006);
        for _ in 1..500_000 {}

        // spawn(process3, 0x8500_0000)
    }
}

#[export_name = "process3"]
pub extern "C" fn process3() {
    loop {
        // println!("Process3 1");
        print_number(3001);
        println!("3001");
        for _ in 1..2_000_000 {}
        // println!("Process3 2");
        print_number(3002);
        println!("3002");
        for _ in 1..2_000_000 {}
        // println!("Process3 3");
        print_number(3003);
        println!("3003");
        for _ in 1..2_000_000 {}
        // println!("Process3 4");
        print_number(3004);
        println!("3004");
        for _ in 1..2_000_000 {}
        // println!("Process3 5");
        print_number(3005);
        println!("3005");
        for _ in 1..2_000_000 {}
        // println!("Process3 6");
        print_number(3006);
        println!("3006");
        for _ in 1..2_000_000 {}
    }
}
