#[export_name="process1"]
pub extern "C" fn process1() {
    loop {
        println!("Process1 1");
        for _ in 1..200_000 {}
        println!("Process1 2");
        for _ in 1..200_000 {}
        println!("Process1 3");
        for _ in 1..200_000 {}
        println!("Process1 4");
        for _ in 1..200_000 {}
        println!("Process1 5");
        for _ in 1..200_000 {}
        println!("Process1 6");
        for _ in 1..200_000 {}
    }
}

#[export_name="process2"]
pub extern "C" fn process2() {
    loop {
        println!("Process2 1");
        for _ in 1..500_000 {}
        println!("Process2 2");
        for _ in 1..500_000 {}
        println!("Process2 3");
        for _ in 1..500_000 {}
        println!("Process2 4");
        for _ in 1..500_000 {}
        println!("Process2 5");
        for _ in 1..500_000 {}
        println!("Process2 6");
        for _ in 1..500_000 {}

        // spawn(process3, 0x8500_0000)
    }
}

#[export_name="process3"]
pub extern "C" fn process3() {
    loop {
        println!("Process3 1");
        for _ in 1..2_000_000 {}
        println!("Process3 2");
        for _ in 1..2_000_000 {}
        println!("Process3 3");
        for _ in 1..2_000_000 {}
        println!("Process3 4");
        for _ in 1..2_000_000 {}
        println!("Process3 5");
        for _ in 1..2_000_000 {}
        println!("Process3 6");
        for _ in 1..2_000_000 {}
    }
}