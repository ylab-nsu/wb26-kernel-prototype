use alloc::vec::Vec;
use riscv::interrupt::{Exception, Interrupt, Trap};
use riscv::interrupt::Interrupt::SupervisorTimer;
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

#[export_name = "_handle_trap_rust"]
extern "C" fn handle_trap(frame: &mut TrapFrame) {
    // println!("Current SP: {:p}", frame);

    let time = riscv::register::time::read64();
    sbi::timer::set_timer(time + 10_000_000).expect("Can't set timer");
    // sbi::timer::set_timer(time + 1_000).expect("Can't set timer");

    let x: Trap<Interrupt, Exception> = riscv::register::scause::read().cause().try_into().unwrap();

    println!("Cause: {x:?}");

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

    unsafe { CURRENT_THREAD = next_thread }

    match x {
        Trap::Exception(cause) => {
            println!("exception:{cause:?}");
            match cause {
                Exception::InstructionFault => {
                    sbi::timer::set_timer(u64::MAX).expect("Can't set timer");
                    let epc = unsafe { riscv::register::sepc::read() };
                    panic!("InstructionFault {epc:x} {}", curr.frame.pc);
                }
                _ => {}
            }
        }
        Trap::Interrupt(SupervisorTimer) => {
            // println!("Yooohooo timer!!!!");
        }
        Trap::Interrupt(cause) => {
            println!("interrupt:{cause:?}");
        }
    }

    // Trap::from(cause);
}

pub fn process1() -> ! {
    loop {
        println!("Process1 1");
        for _ in 1..5_000 {}
        println!("Process1 2");
        for _ in 1..5_000 {}
        println!("Process1 3");
        for _ in 1..5_000 {}
        println!("Process1 4");
        for _ in 1..5_000 {}
        println!("Process1 5");
        for _ in 1..5_000 {}
        println!("Process1 6");
        for _ in 1..5_000 {}
    }
}

pub fn process2() -> ! {
    loop {
        println!("Process2 1");
        for _ in 1..5_000 {}
        println!("Process2 2");
        for _ in 1..5_000 {}
        println!("Process2 3");
        for _ in 1..5_000 {}
        println!("Process2 4");
        for _ in 1..5_000 {}
        println!("Process2 5");
        for _ in 1..5_000 {}
        println!("Process2 6");
        for _ in 1..5_000 {}

        // spawn(process3, 0x8500_0000)
    }
}

pub fn process3() -> ! {
    loop {
        println!("Process3 1");
        for _ in 1..5_000_000 {}
        println!("Process3 2");
        for _ in 1..5_000_000 {}
        println!("Process3 3");
        for _ in 1..5_000_000 {}
        println!("Process3 4");
        for _ in 1..5_000_000 {}
        println!("Process3 5");
        for _ in 1..5_000_000 {}
        println!("Process3 6");
        for _ in 1..5_000_000 {}
    }
}

pub fn spawn(f: fn() -> !, sp: usize) {
    unsafe {
        let thread = Thread {
            id: PROCESSES.len() + 1,
            frame: TrapFrame::default().with_pc(f as usize).with_sp(sp),
        };

        PROCESSES.push(thread);
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
}

pub fn enable_threading() {
    unsafe {
        riscv::interrupt::enable();
        riscv::interrupt::enable_interrupt(SupervisorTimer);
        // riscv::register::sie::set_stimer();
    };
}