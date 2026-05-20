use crate::threading::thread;
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
    let id = thread::spawn(CRT0[0].entry, stack_end);
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

    for prog in USER_PROGRAMS {
        spawn_user_program(prog);
    }
}

pub(crate) fn enable_threading() {
    unsafe {
        riscv::interrupt::enable();
        riscv::interrupt::enable_interrupt(Interrupt::SupervisorTimer);
        // riscv::register::sie::set_stimer();
    };
}
