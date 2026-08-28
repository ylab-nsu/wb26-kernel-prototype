use crate::arch::traits::TargetTrapFrame;
use crate::drivers::driver_task;
use crate::threading::thread::{
    create_empty_thread, get_thread, spawn_kernel, spawn_user, THREADS_INDEXES,
};
use crate::drivers_::uart::{terminal_task, uart_driver, uart16550_rw_test};
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
    "og_process1: .dc.a __user_og_process1",
    "og_process2: .dc.a __user_og_process2",
    "og_process3: .dc.a __user_og_process3",
    "scull_user: .dc.a __user_scull_user",
);

extern "C" {
    static crt0: extern "C" fn() -> !;
    static user1: extern "C" fn();
    static og_process1: extern "C" fn();
    static og_process2: extern "C" fn();
    static og_process3: extern "C" fn();
    static scull_user: extern "C" fn();
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
        get_thread(id).user_frame.set_arg0(prog.entry as usize);
    }
}

pub fn setup_threads() {
    let time = riscv::register::time::read64();
    info!("Current time: {}", time);
    println!();
    println!();
    // sbi::timer::set_timer(time + 10_000_000).expect("Can't set timer");

    create_empty_thread();

    let driver_task_id = spawn_kernel(driver_task);
    let uart_id = spawn_kernel(uart_driver);
	// terminal task
	//let terminal_id = spawn_kernel(terminal_task);
	//let test_id = spawn_kernel(uart16550_rw_test);
	
    unsafe {
        THREADS_INDEXES.driver_task = driver_task_id;
        THREADS_INDEXES.user_start = driver_task_id + 1;
    }

    let user_programs: [UserProgram; _] = [
        UserProgram {
            entry: unsafe { user1 },
            stack_size: 64 * 1024,
        },
        UserProgram {
            entry: unsafe { og_process1 },
            stack_size: 64 * 1024,
        },
        UserProgram {
            entry: unsafe { og_process2 },
            stack_size: 64 * 1024,
        },
        UserProgram {
            entry: unsafe { og_process3 },
            stack_size: 64 * 1024,
        },
        UserProgram {
            entry: unsafe { scull_user },
            stack_size: 64 * 1024,
        },
    ];

    // for prog in user_programs {
    //     spawn_user_program(&prog);
    // }
}
