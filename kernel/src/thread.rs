use crate::arch::traits::TargetTrapFrame;
use crate::arch::TrapFrame;
use alloc::vec::Vec;


#[repr(C)]
#[derive(Debug)]
pub struct Page(pub [usize; 512]);

#[repr(C)]
#[derive(Debug)]
pub struct PagePool {
    pub kernel_stack_pages: [Page; 64],
}

extern "C" {
    #[link_name = "__s_temp_kernel_stacks"]
    static mut MMU_TABLE: [Page; 64];
}

pub struct Thread {
    pub id: usize,
    pub kernel_sp: usize,
    pub frame: &'static mut TrapFrame,
}

impl Thread {
    unsafe fn new(id: usize) -> Self {
        let _left = &mut MMU_TABLE[id * 4] as *mut Page as usize;
        let right = &mut MMU_TABLE[(id + 1) * 4] as *mut Page as usize;

        let trap_frame_addr =
            (right - size_of::<TrapFrame>()) / size_of::<TrapFrame>() * size_of::<TrapFrame>();
        debug_assert!(trap_frame_addr != 0);
        debug_assert!(trap_frame_addr % align_of::<TrapFrame>() == 0);
        let trap_frame = trap_frame_addr as *mut TrapFrame;

        Thread {
            id,
            kernel_sp: trap_frame_addr,
            frame: &mut *trap_frame,
        }
    }
}

pub static mut PROCESSES: Vec<Thread> = Vec::new();

pub static mut CURRENT_THREAD: usize = 0;

pub static mut NEXT_STACK: usize = 0x47000000;
pub const MAX_STACK: usize = 0x48000000;

pub fn spawn(f: extern "C" fn() -> !, sp: usize) {
    unsafe {
        let thread = Thread::new(PROCESSES.len() + 1);
        *thread.frame = TrapFrame::default().with_pc(f as usize).with_sp(sp);
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
        PROCESSES
            .last_mut()
            .unwrap()
            .frame
            .set_arg0(prog.entry as usize);
    }
}

pub fn setup_threads() {
    let time = riscv::register::time::read64();
    println!("Current time: {}", time);
    // sbi::timer::set_timer(time + 10_000_000).expect("Can't set timer");

    unsafe {
        let pr0 = Thread::new(0);
        PROCESSES.push(pr0);
    }

    for prog in USER_PROGRAMS {
        spawn_user_program(prog);
    }
}
