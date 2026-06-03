use crate::arch::traits::TargetTrapFrame;
use crate::drivers::driver_task;
use crate::threading::thread::{
    create_empty_process, get_process, spawn_kernel, spawn_user, PROCESSES_INDEXES,
};
use core::arch::global_asm;

pub static mut NEXT_USER_STACK: usize = 0x47000000;
pub const MAX_USER_STACK: usize = 0x48000000;

pub struct UserProgram {
    pub entry: extern "C" fn(),
    pub stack_size: usize,
}

global_asm!(
    ".section .rodata.usersymaddrs, \"a\"",
    ".align 2",
    "crt0:     .dc.a __user_crt0",
    "user1:    .dc.a __user_user1",
    "process1: .dc.a __user_process1",
    "process2: .dc.a __user_process2",
    "process3: .dc.a __user_process3",
);

extern "C" {
    static crt0: extern "C" fn() -> !;
    static user1: extern "C" fn();
    static process1: extern "C" fn();
    static process2: extern "C" fn();
    static process3: extern "C" fn();
}

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
    let id = spawn_user(unsafe { crt0 }, stack_end);
    unsafe {
        get_process(id).user_frame.set_arg0(prog.entry as usize);
    }
}

pub fn setup_threads() {
    let time = riscv::register::time::read64();
    println!("Current time: {}", time);
    // sbi::timer::set_timer(time + 10_000_000).expect("Can't set timer");

    create_empty_process();

    let driver_task_id = spawn_kernel(driver_task);
    unsafe {
        PROCESSES_INDEXES.driver_task = driver_task_id;
        PROCESSES_INDEXES.user_start = driver_task_id + 1;
    }

    let user_programs: [UserProgram; _] = [
        UserProgram {
            entry: unsafe { user1 },
            stack_size: 64 * 1024,
        },
        UserProgram {
            entry: unsafe { process1 },
            stack_size: 64 * 1024,
        },
        UserProgram {
            entry: unsafe { process2 },
            stack_size: 64 * 1024,
        },
        UserProgram {
            entry: unsafe { process3 },
            stack_size: 64 * 1024,
        },
    ];

    for prog in user_programs {
        spawn_user_program(&prog);
    }
}
