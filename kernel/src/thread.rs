use crate::mmu::set_satp;
use alloc::vec::Vec;
use riscv::interrupt::Interrupt::SupervisorTimer;
use riscv::interrupt::{Exception, Interrupt, Trap};
use riscv::register::mtvec::TrapMode;
use riscv::register::stvec::Stvec;

#[repr(C)]
#[derive(Debug, Default, Clone)]
struct TrapFrame {
    pc: usize,
    ra: usize,
    sp: usize,
    gp: usize,
    tp: usize,
    t0: usize,
    t1: usize,
    t2: usize,
    s0: usize,
    s1: usize,
    a0: usize,
    a1: usize,
    a2: usize,
    a3: usize,
    a4: usize,
    a5: usize,
    a6: usize,
    a7: usize,
    s2: usize,
    s3: usize,
    s4: usize,
    s5: usize,
    s6: usize,
    s7: usize,
    s8: usize,
    s9: usize,
    s10: usize,
    s11: usize,
    t3: usize,
    t4: usize,
    t5: usize,
    t6: usize,
}

impl TrapFrame {
    fn with_pc(mut self, pc: usize) -> Self {
        self.pc = pc;
        self
    }

    fn with_sp(mut self, sp: usize) -> Self {
        self.sp = sp;
        self
    }
}

struct Thread {
    id: usize,
    frame: TrapFrame,
}

static mut PROCESSES: Vec<Thread> = Vec::new();

static mut CURRENT_THREAD: usize = 0;

static mut NEXT_STACK: usize = 0x47000000;
const MAX_STACK: usize = 0x48000000;

fn reschedule(frame: &mut TrapFrame) {
    let time = riscv::register::time::read64();
    sbi::timer::set_timer(time + 10_000_000).expect("Can't set timer");
    // sbi::timer::set_timer(time + 1_000).expect("Can't set timer");

    let next_thread = unsafe {
        if CURRENT_THREAD < PROCESSES.len() - 1 {
            CURRENT_THREAD + 1
        } else {
            1
        }
    };

    let (curr, next) = unsafe {
        let curr = PROCESSES.get_mut(CURRENT_THREAD).unwrap();
        let next = PROCESSES.get_mut(next_thread).unwrap();

        // println!("CURRENT_THREAD: {CURRENT_THREAD}");

        (curr, next)
    };

    if frame.pc == 0 {
        let epc = unsafe { riscv::register::sepc::read() };
        println!("frame.pc is zero, epc={:x}", epc);
    }

    curr.frame = frame.clone();
    *frame = next.frame.clone();

    unsafe {
        CURRENT_THREAD = next_thread;
        riscv::register::sstatus::set_spp(riscv::register::sstatus::SPP::User);
        set_satp(1);
    };
}

#[export_name = "_handle_trap_rust"]
extern "C" fn handle_trap(frame: &mut TrapFrame) {
    // println!("Current SP: {:p}", frame);
    let x: Trap<Interrupt, Exception> = riscv::register::scause::read().cause().try_into().unwrap();
    println!("Cause: {x:?}");

    match x {
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            reschedule(frame);
        }

        Trap::Exception(Exception::UserEnvCall) => {
            if (frame.a6 == 0) {
                println!("Received non-SBI UserEnvCall");
            } else {
                println!("    Redirecting UserEnvCall to SBI");
                unsafe {
                    core::arch::asm!(
                        "ecall",
                        inlateout("a0") frame.a0 => frame.a0,
                        inlateout("a1") frame.a1 => frame.a1,
                        in("a2") frame.a2,
                        in("a3") frame.a3,
                        in("a4") frame.a4,
                        in("a5") frame.a5,
                        in("a6") frame.a6,
                        in("a7") frame.a7,
                    );
                }
            }
            frame.pc += 4;
        }

        Trap::Exception(Exception::InstructionFault) => {
            sbi::timer::set_timer(u64::MAX).expect("Can't set timer");
            let epc = unsafe { riscv::register::sepc::read() };
            panic!("InstructionFault {epc:x} {}", frame.pc);
        }

        Trap::Interrupt(cause) => {
            println!("interrupt:{cause:?}");
        }
        Trap::Exception(cause) => {
            println!("exception:{cause:?}");
        }
    }
}

pub fn spawn(f: extern "C" fn() -> !, sp: usize) {
    unsafe {
        let thread = Thread {
            id: PROCESSES.len() + 1,
            frame: TrapFrame::default().with_pc(f as usize).with_sp(sp),
        };

        PROCESSES.push(thread);
    }
}

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
        stack_end = NEXT_STACK + prog.stack_size;
        if stack_end > MAX_STACK {
            panic!("Stack area is exceeded");
        }
        NEXT_STACK = stack_end;
    }
    // spawn(USER_PROGRAMS[0].entry, stack_end);
    spawn(CRT0[0].entry, stack_end);
    unsafe {
        PROCESSES.last_mut().unwrap().frame.a0 = prog.entry as usize;
    }
}

pub fn setup_trap() {
    extern "C" {
        fn _start_trap();
    }
    unsafe { riscv::register::stvec::write(Stvec::new(_start_trap as usize, TrapMode::Direct)) }
}

pub fn setup_threads() {
    let time = riscv::register::time::read64();
    println!("Current time: {}", time);
    // sbi::timer::set_timer(time + 10_000_000).expect("Can't set timer");

    let pr0 = Thread {
        id: 0,
        frame: TrapFrame::default(),
    };

    unsafe {
        PROCESSES.push(pr0);
    }

    // Doesn't work
    // extern "C" {
    //     static __eustack: usize;
    //     static __sustack: usize;
    // }
    //
    // unsafe {
    //     NEXT_STACK = addr_of!(__eustack) as usize;
    //     MAX_STACK = addr_of!(__sustack) as usize;
    // }
    for prog in USER_PROGRAMS {
        spawn_user_program(prog);
    }
}

pub fn enable_threading() {
    unsafe {
        riscv::interrupt::enable();
        riscv::interrupt::enable_interrupt(SupervisorTimer);
        // riscv::register::sie::set_stimer();
    };
}
