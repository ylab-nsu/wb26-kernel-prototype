use crate::page_pool::{Page, PAGE_POOL};
use alloc::vec::Vec;

#[repr(C)]
#[derive(Debug, Default, Clone)]
pub(crate) struct TrapFrame {
    pub(crate) pc: usize,
    pub(crate) ra: usize,
    pub(crate) sp: usize,
    pub(crate) gp: usize,
    pub(crate) tp: usize,
    pub(crate) t0: usize,
    pub(crate) t1: usize,
    pub(crate) t2: usize,
    pub(crate) s0: usize,
    pub(crate) s1: usize,
    pub(crate) a0: usize,
    pub(crate) a1: usize,
    pub(crate) a2: usize,
    pub(crate) a3: usize,
    pub(crate) a4: usize,
    pub(crate) a5: usize,
    pub(crate) a6: usize,
    pub(crate) a7: usize,
    pub(crate) s2: usize,
    pub(crate) s3: usize,
    pub(crate) s4: usize,
    pub(crate) s5: usize,
    pub(crate) s6: usize,
    pub(crate) s7: usize,
    pub(crate) s8: usize,
    pub(crate) s9: usize,
    pub(crate) s10: usize,
    pub(crate) s11: usize,
    pub(crate) t3: usize,
    pub(crate) t4: usize,
    pub(crate) t5: usize,
    pub(crate) t6: usize,
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

pub(crate) struct Thread {
    pub(crate) id: usize,
    pub(crate) kernel_sp: usize,
    pub(crate) frame: &'static mut TrapFrame,
}

impl Thread {
    unsafe fn new(id: usize) -> Self {
        let _left = unsafe { &mut PAGE_POOL.kernel_stack_pages[id * 4] as *mut Page as usize };
        let right =
            unsafe { &mut PAGE_POOL.kernel_stack_pages[(id + 1) * 4] as *mut Page as usize };

        let trap_frame_addr =
            (right - size_of::<TrapFrame>()) / size_of::<TrapFrame>() * size_of::<TrapFrame>();
        debug_assert!(trap_frame_addr != 0);
        debug_assert!(trap_frame_addr % align_of::<TrapFrame>() == 0);
        let trap_frame = trap_frame_addr as *mut TrapFrame;

        Thread {
            id,
            kernel_sp: trap_frame_addr,
            frame: unsafe { &mut *trap_frame },
        }
    }
}

static mut PROCESSES: Vec<Thread> = Vec::new();

static mut CURRENT_THREAD: usize = 0;

pub(crate) unsafe fn get_current_thread_id() -> usize {
    unsafe { CURRENT_THREAD }
}

pub(crate) unsafe fn set_current_thread_id(id: usize) {
    unsafe { CURRENT_THREAD = id }
}

pub(crate) unsafe fn get_process_count() -> usize {
    unsafe { PROCESSES.len() }
}

pub(crate) unsafe fn get_process(id: usize) -> &'static mut Thread {
    unsafe { PROCESSES.get_mut(id).unwrap() }
}

pub(crate) unsafe fn create_empty_process() -> usize {
    unsafe {
        let id = PROCESSES.len();
        let pr0 = Thread::new(id);
        PROCESSES.push(pr0);
        id
    }
}

pub(crate) fn spawn(f: extern "C" fn() -> !, sp: usize) -> usize {
    unsafe {
        let id = PROCESSES.len();
        let thread = Thread::new(id);
        *thread.frame = TrapFrame::default().with_pc(f as usize).with_sp(sp);
        PROCESSES.push(thread);
        id
    }
}
