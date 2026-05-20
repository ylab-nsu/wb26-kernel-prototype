use crate::threading::thread;
use core::arch::global_asm;
use riscv::interrupt::Interrupt;
use riscv::register::mtvec::TrapMode;
use riscv::register::stvec::Stvec;

static mut NEXT_USER_STACK: usize = 0x47000000;
const MAX_USER_STACK: usize = 0x48000000;

struct UserProgram {
    entry: extern "C" fn(),
    stack_size: usize,
}
struct UserProgram2 {
    entry: extern "C" fn() -> !,
    stack_size: usize,
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

fn spawn_user_program(prog: &UserProgram) {
    let stack_end;
    unsafe {
        stack_end = NEXT_USER_STACK + prog.stack_size;
        if stack_end > MAX_USER_STACK {
            panic!("Stack area is exceeded");
        }
        NEXT_USER_STACK = stack_end;
    }
    // spawn(USER_PROGRAMS[0].entry, stack_end);
    let id = thread::spawn(unsafe { crt0 }, stack_end);
    unsafe {
        thread::get_process(id).frame.a0 = prog.entry as usize;
    }
}

pub(crate) fn setup_trap() {
    extern "C" {
        fn _start_trap();
    }
    unsafe {
        riscv::register::stvec::write(Stvec::new(
            _start_trap as *const () as usize,
            TrapMode::Direct,
        ))
    }
}

pub(crate) fn setup_threads() {
    let time = riscv::register::time::read64();
    println!("Current time: {}", time);
    // sbi::timer::set_timer(time + 10_000_000).expect("Can't set timer");

    unsafe {
        thread::create_empty_process();
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

pub(crate) fn enable_threading() {
    unsafe {
        riscv::interrupt::enable();
        riscv::interrupt::enable_interrupt(Interrupt::SupervisorTimer);
        // riscv::register::sie::set_stimer();
    };
}
