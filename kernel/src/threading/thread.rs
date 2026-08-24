use crate::arch::traits::{TargetContext, TargetTrapFrame};
use crate::arch::{AddressSpace, Context, TrapFrame};
use alloc::vec::Vec;

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
    pub context: &'static mut Context,
    pub user_frame: &'static mut TrapFrame, // Cannot be used for kernel threads
    /// `None` once the thread has been terminated (releases its reference on
    /// the address space, letting the refcount cascade free it if this was the
    /// last user).
    pub address_space: Option<AddressSpace>,
    pub valid: bool,
    pub is_kernel: bool,
}

impl Thread {
    unsafe fn new(id: usize, address_space: AddressSpace) -> Self {
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
            address_space: Some(address_space),
            valid: true,
            is_kernel: false,
        }
    }

    /// Release the thread's reference on its address space (and mark it
    /// invalid). If this was the last thread holding the address space, the
    /// refcount cascade frees the root page table and every page it held.
    pub fn terminate(&mut self) {
        self.valid = false;
        self.address_space.take();
    }
}

pub struct ThreadsIndexes {
    pub driver_task: usize,
    pub user_start: usize,
}

static mut THREADS: Vec<Thread> = Vec::new();

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

pub unsafe fn get_thread(id: usize) -> &'static mut Thread {
    unsafe { THREADS.get_mut(id).unwrap() }
}

pub fn create_empty_thread() -> usize {
    unsafe {
        let id = THREADS.len();
        let mut pr0 = Thread::new(id, AddressSpace::new());
        pr0.valid = false;
        THREADS.push(pr0);
        id
    }
}

pub fn spawn_user(f: extern "C" fn() -> !, user_sp: usize) -> usize {
    unsafe {
        let id = THREADS.len();
        let thread = Thread::new(id, AddressSpace::new());
        *thread.user_frame = TrapFrame::default().with_pc(f as usize).with_sp(user_sp);
        *thread.context = Context::default()
            .with_ra(_initial_return_trap as *const () as usize)
            .with_sp((thread.user_frame as *mut TrapFrame) as usize);
        THREADS.push(thread);
        id
    }
}

/// Spawn a user thread that runs in its own `AddressSpace`, entered directly
/// at `entry` (no crt0 trampoline). Ownership of the address space moves into
/// the thread.
pub fn spawn_user_in(entry: usize, user_sp: usize, address_space: AddressSpace) -> usize {
    unsafe {
        let id = THREADS.len();
        let thread = Thread::new(id, address_space);
        *thread.user_frame = TrapFrame::default().with_pc(entry).with_sp(user_sp);
        *thread.context = Context::default()
            .with_ra(_initial_return_trap as *const () as usize)
            .with_sp((thread.user_frame as *mut TrapFrame) as usize);
        THREADS.push(thread);
        id
    }
}

/// Spawn a user thread that shares an *existing* address space with other
/// threads. Cloning the AS takes one more reference on its root page table and
/// on every page it holds (existing refcounts only), so the AS and its pages
/// live until the last thread using them dies.
pub fn spawn_user_in_shared(entry: usize, user_sp: usize, address_space: &AddressSpace) -> usize {
    unsafe {
        let id = THREADS.len();
        let thread = Thread::new(id, address_space.clone());
        *thread.user_frame = TrapFrame::default().with_pc(entry).with_sp(user_sp);
        *thread.context = Context::default()
            .with_ra(_initial_return_trap as *const () as usize)
            .with_sp((thread.user_frame as *mut TrapFrame) as usize);
        THREADS.push(thread);
        id
    }
}

pub fn spawn_kernel(f: extern "C" fn() -> !) -> usize {
    unsafe {
        let id = THREADS.len();
        let mut thread = Thread::new(id, AddressSpace::new());
        *thread.user_frame = TrapFrame::default().with_pc(f as usize).with_sp(0);
        *thread.context = Context::default()
            .with_ra(_initial_return_trap as *const () as usize)
            .with_sp(thread.user_frame as *mut TrapFrame as usize);
        thread.is_kernel = true;
        THREADS.push(thread);
        id
    }
}
