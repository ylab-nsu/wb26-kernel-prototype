use crate::arch::traits::TargetTrapFrame;
use crate::arch::TrapFrame;
use alloc::vec::Vec;

#[repr(C)]
#[derive(Debug)]
pub struct Page(pub [usize; 512]);

extern "C" {
    #[link_name = "__s_temp_kernel_stacks"]
    static mut KERNEL_STACKS: [Page; 512];
}

#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct Context {
    pub ra: usize,
    pub sp: usize,
    pub s0: usize,
    pub s1: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
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

pub struct Thread {
    pub id: usize,
    pub context: &'static mut Context,
    pub user_frame: &'static mut TrapFrame, // Cannot be used for kernel threads
    pub valid: bool,
    pub is_kernel: bool,
}

impl Thread {
    unsafe fn new(id: usize) -> Self {
        let _left = unsafe { &mut KERNEL_STACKS[id * 16] as *mut Page as usize };
        let right = unsafe { &mut KERNEL_STACKS[(id + 1) * 16] as *mut Page as usize };

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

static mut PROCESSES: Vec<Thread> = Vec::new();

static mut CURRENT_THREAD: usize = 0;

pub unsafe fn get_current_thread_id() -> usize {
    unsafe { CURRENT_THREAD }
}

pub unsafe fn set_current_thread_id(id: usize) {
    unsafe { CURRENT_THREAD = id }
}

pub unsafe fn get_process_count() -> usize {
    unsafe { PROCESSES.len() }
}

pub unsafe fn get_process(id: usize) -> &'static mut Thread {
    unsafe { PROCESSES.get_mut(id).unwrap() }
}

pub fn create_empty_process() -> usize {
    unsafe {
        let id = PROCESSES.len();
        let mut pr0 = Thread::new(id);
        pr0.valid = false;
        PROCESSES.push(pr0);
        id
    }
}

pub fn spawn_user(f: extern "C" fn() -> !, user_sp: usize) -> usize {
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

pub fn spawn_kernel(f: extern "C" fn() -> !) -> usize {
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
