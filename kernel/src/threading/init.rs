use crate::arch::traits::TargetTrapFrame;
use crate::threading::thread::{create_empty_process, get_process, spawn};

pub static mut NEXT_USER_STACK: usize = 0x47000000;
pub const MAX_USER_STACK: usize = 0x48000000;

pub struct UserProgram {
    pub entry: extern "C" fn(),
    pub stack_size: usize,
}
pub struct UserProgram2 {
    pub entry: extern "C" fn() -> !,
    pub stack_size: usize,
}

unsafe extern "C" {
    #[link_name = "__user_user1"]
    safe fn user1();
    #[link_name = "__user_process1"]
    safe fn process1();
    #[link_name = "__user_process2"]
    safe fn process2();
    #[link_name = "__user_process3"]
    safe fn process3();
    #[link_name = "__user_crt0"]
    safe fn crt0() -> !;
}

// const CRT0: extern "C" fn() -> ! = _crt0;
const CRT0: &[UserProgram2] = &[UserProgram2 {
    entry: crt0,
    stack_size: 0,
}];

const USER_PROGRAMS: &[UserProgram] = &[
    UserProgram {
        entry: user1,
        stack_size: 64 * 1024,
    },
    UserProgram {
        entry: process1,
        stack_size: 64 * 1024,
    },
    UserProgram {
        entry: process2,
        stack_size: 64 * 1024,
    },
    UserProgram {
        entry: process3,
        stack_size: 64 * 1024,
    },
];

pub fn spawn_user_program(prog: &UserProgram) {
    let stack_end;
    unsafe {
        stack_end = NEXT_USER_STACK + prog.stack_size;
        if stack_end > MAX_USER_STACK {
            panic!("Stack area is exceeded");
        }
        NEXT_USER_STACK = stack_end;
    }
    // spawn(USER_PROGRAMS[0].entry, stack_end);
    let id = spawn(CRT0[0].entry, stack_end);
    unsafe {
        get_process(id).frame.set_arg0(prog.entry as usize);
    }
}

pub fn setup_threads() {
    let time = riscv::register::time::read64();
    println!("Current time: {}", time);
    // sbi::timer::set_timer(time + 10_000_000).expect("Can't set timer");

    unsafe {
        create_empty_process();
    }

    for prog in USER_PROGRAMS {
        spawn_user_program(prog);
    }
}
