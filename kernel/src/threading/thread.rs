use crate::arch::traits::{TargetContext, TargetTrapFrame};
use crate::arch::{AddressSpace, Context, TrapFrame};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cell::UnsafeCell;

#[repr(C)]
#[derive(Debug)]
pub struct Page(pub [usize; 512]);

extern "C" {
    #[link_name = "__s_temp_kernel_stacks"]
    static mut KERNEL_STACKS: [Page; 512];
}

extern "C" {
    fn _initial_return_trap() -> !;
}

pub struct Thread {
    pub id: usize,
    pub context: UnsafeCell<Context>,
    pub address_space: Option<Arc<AddressSpace>>,
    pub valid: bool,
    pub is_kernel: bool,
}

// struct PreStart<'a, T: TargetTrapFrame> {
//     frame: &'a mut T,
// }

// struct PreStart<'a, T: TargetTrapFrame>(&'a mut T);
//
// impl<T: TargetTrapFrame> PreStart<'_, T> {
//     fn set_pc(&mut self, v: usize) {
//         self.0.set_pc(v);
//     }
//     fn set_sp(&mut self, v: usize) {
//         self.0.set_sp(v);
//     }
//     fn set_arg0(&mut self, v: usize) {
//         self.0.set_arg0(v);
//     }
// }

impl Thread {
    unsafe fn new(
        id: usize,
        is_kernel: bool,
        address_space: Arc<AddressSpace>,
        initial_pc: usize,
        initial_user_sp: usize,
        arg0: usize,
        // frame_init: F,
    ) -> Self
// where
    //     F: FnOnce(PreStart<'_, TrapFrame>),
    {
        let _left = unsafe { &mut KERNEL_STACKS[id * 16] as *mut Page as usize };
        let right = unsafe { &mut KERNEL_STACKS[(id + 1) * 16] as *mut Page as usize };

        let trap_frame_addr =
            (right - size_of::<TrapFrame>()) / align_of::<TrapFrame>() * align_of::<TrapFrame>();
        debug_assert!(trap_frame_addr != 0);
        debug_assert!(trap_frame_addr % align_of::<TrapFrame>() == 0);
        let trap_frame = trap_frame_addr as *mut TrapFrame;
        unsafe {
            trap_frame.write(
                TrapFrame::default()
                    .with_pc(initial_pc)
                    .with_sp(initial_user_sp)
                    .with_arg0(arg0),
            );
            // frame_init(PreStart(&mut *trap_frame));
        }

        Thread {
            id,
            context: UnsafeCell::new(unsafe {
                Context::new(_initial_return_trap as *const () as usize, trap_frame_addr)
            }),
            address_space: Some(address_space),
            valid: true,
            is_kernel,
        }
    }

    pub fn terminate(&mut self) {
        self.valid = false;
        self.address_space.take();
    }
}

pub struct ThreadsIndexes {
    pub driver_task: usize,
    pub user_start: usize,
}

static mut THREADS: Vec<Arc<Thread>> = Vec::new();

static mut CURRENT_THREAD: usize = 0;

pub static mut THREADS_INDEXES: ThreadsIndexes = ThreadsIndexes {
    // These will be initialized later, but before scheduling started
    driver_task: 1,
    user_start: 1,
};

pub unsafe fn get_current_thread_id() -> usize {
    unsafe { CURRENT_THREAD }
}

pub unsafe fn set_current_thread_id(id: usize) {
    unsafe { CURRENT_THREAD = id }
}

pub unsafe fn get_threads_count() -> usize {
    unsafe { THREADS.len() }
}

pub unsafe fn get_thread(id: usize) -> Arc<Thread> {
    unsafe { THREADS.get_mut(id).unwrap().clone() }
}

pub fn create_invalid_thread() -> usize {
    unsafe {
        let id = THREADS.len();
        let mut pr0 = Thread::new(id, true, Arc::new(AddressSpace::new()), 0, 0, 0);
        pr0.valid = false;
        THREADS.push(Arc::new(pr0));
        id
    }
}

pub fn spawn_user(
    f: extern "C" fn() -> !,
    user_sp: usize,
    address_space: Arc<AddressSpace>,
    user_main: usize,
) -> usize {
    unsafe {
        let id = THREADS.len();
        let thread = Thread::new(id, false, address_space, f as usize, user_sp, user_main);
        THREADS.push(Arc::new(thread));
        id
    }
}

pub fn spawn_kernel(f: extern "C" fn() -> !, address_space: Arc<AddressSpace>) -> usize {
    unsafe {
        let id = THREADS.len();
        let thread = Thread::new(id, true, address_space, f as usize, 0, 0);
        THREADS.push(Arc::new(thread));
        id
    }
}
