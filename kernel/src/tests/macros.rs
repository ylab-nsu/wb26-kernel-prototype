
#[macro_export]
macro_rules! run_test {
    ($func:ident) => {{
        println!("Test {} is running", stringify!($func));
        
        $func();
        
        println!("Test {} passed", stringify!($func));
    }};
}

