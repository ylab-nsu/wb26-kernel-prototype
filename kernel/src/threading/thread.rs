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

#[repr(C)]
#[derive(Debug, Default, Clone)]
pub(crate) struct Context {
    pub(crate) ra: usize,
    pub(crate) sp: usize,
    pub(crate) s0: usize,
    pub(crate) s1: usize,
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

impl Context {
    fn with_ra(mut self, ra: usize) -> Self {
        self.ra = ra;
        self
    }

    fn with_sp(mut self, sp: usize) -> Self {
        self.sp = sp;
        self
    }
}

extern "C" {
    fn _initial_return_trap() -> !;
}

pub(crate) struct Thread {
    pub(crate) id: usize,
    pub(crate) context: &'static mut Context,
    pub(crate) user_frame: &'static mut TrapFrame, // Cannot be used for kernel threads
    pub(crate) valid: bool,
    pub(crate) is_kernel: bool,
}

impl Thread {
    unsafe fn new(id: usize) -> Self {
        let _left = unsafe { &mut PAGE_POOL.kernel_stack_pages[id * 16] as *mut Page as usize };
        let right =
            unsafe { &mut PAGE_POOL.kernel_stack_pages[(id + 1) * 16] as *mut Page as usize };

        let context_addr =
            (right - size_of::<Context>()) / align_of::<Context>() * align_of::<Context>();
        debug_assert!(context_addr != 0);
        debug_assert!(context_addr % align_of::<Context>() == 0);
        let context = context_addr as *mut Context;

        let trap_frame_addr = (context_addr - size_of::<TrapFrame>()) / align_of::<TrapFrame>()
            * align_of::<TrapFrame>();
        debug_assert!(trap_frame_addr != 0);
        debug_assert!(trap_frame_addr % align_of::<TrapFrame>() == 0);
        let trap_frame = trap_frame_addr as *mut TrapFrame;

        Thread {
            id,
            context: unsafe { &mut *context },
            user_frame: unsafe { &mut *trap_frame },
            valid: true,
            is_kernel: false,
        }
    }
}

pub(crate) struct ProcessesIndexes {
    pub(crate) driver_task: usize,
    pub(crate) user_start: usize,
}

static mut PROCESSES: Vec<Thread> = Vec::new();

static mut CURRENT_THREAD: usize = 0;

pub(crate) static mut PROCESSES_INDEXES: ProcessesIndexes = ProcessesIndexes {
    // These will be initialized later, but before scheduling started
    driver_task: 1,
    user_start: 1,
};

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

pub(crate) fn create_empty_process() -> usize {
    unsafe {
        let id = PROCESSES.len();
        let mut pr0 = Thread::new(id);
        pr0.valid = false;
        PROCESSES.push(pr0);
        id
    }
}

pub(crate) fn spawn_user(f: extern "C" fn() -> !, user_sp: usize) -> usize {
    unsafe {
        let id = PROCESSES.len();
        let thread = Thread::new(id);
        *thread.user_frame = TrapFrame::default().with_pc(f as usize).with_sp(user_sp);
        *thread.context = Context::default()
            .with_ra(_initial_return_trap as *const () as usize)
            .with_sp((thread.user_frame as *mut TrapFrame) as usize);
        PROCESSES.push(thread);
        id
    }
}

pub(crate) fn spawn_kernel(f: extern "C" fn() -> !) -> usize {
    unsafe {
        let id = PROCESSES.len();
        let mut thread = Thread::new(id);
        *thread.user_frame = TrapFrame::default().with_pc(f as usize).with_sp(0);
        *thread.context = Context::default()
            .with_ra(_initial_return_trap as *const () as usize)
            .with_sp(thread.user_frame as *mut TrapFrame as usize);
        thread.is_kernel = true;
        PROCESSES.push(thread);
        id
    }
}
