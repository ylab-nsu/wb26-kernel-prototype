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

pub struct Thread {
    pub id: usize,
    // pub kernel_sp: usize,
    pub frame: &'static mut TrapFrame,
    pub valid: bool,
    pub is_kernel: bool,
}

impl Thread {
    unsafe fn new(id: usize) -> Self {
        let _left = unsafe { &mut KERNEL_STACKS[id * 16] as *mut Page as usize };
        let right = unsafe { &mut KERNEL_STACKS[(id + 1) * 16] as *mut Page as usize };

        let trap_frame_addr =
            (right - size_of::<TrapFrame>()) / size_of::<TrapFrame>() * size_of::<TrapFrame>();
        debug_assert!(trap_frame_addr != 0);
        debug_assert!(trap_frame_addr % align_of::<TrapFrame>() == 0);
        let trap_frame = trap_frame_addr as *mut TrapFrame;

        Thread {
            id,
            // kernel_sp: trap_frame_addr,
            frame: unsafe { &mut *trap_frame },
            valid: true,
            is_kernel: false,
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

pub(crate) fn create_empty_process() -> usize {
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
        *thread.frame = TrapFrame::default().with_pc(f as usize).with_sp(user_sp);
        PROCESSES.push(thread);
        id
    }
}

pub fn spawn_kernel(f: extern "C" fn() -> !) -> usize {
    unsafe {
        let id = PROCESSES.len();
        let mut thread = Thread::new(id);
        *thread.frame = TrapFrame::default().with_pc(f as usize);
        // thread.kernel_sp = 0;
        thread.is_kernel = true;
        // thread.kernel_sp = 0;
        PROCESSES.push(thread);
        id
    }
}
