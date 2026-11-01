use crate::arch::traits::{TargetContext, TargetPlatform, TargetTrapFrame};
use crate::arch::{AddressSpace, Context, Platform, TrapFrame};
use crate::sync::current_core_id;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::cell::{Cell, UnsafeCell};
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

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

const _: () = assert!(
    core::mem::offset_of!(Thread, context) == 0,
    "If this assert fails, you can just put `#[repr(C)]` on struct Thread. \
    It isn't there yet because I'm curious to see how it can happen."
);

#[derive(Debug)]
pub struct Thread {
    pub context: UnsafeCell<Context>,
    pub id: usize,
    pub address_space: Option<Arc<AddressSpace>>,
    pub valid: bool,
    pub is_kernel: bool,
    claimed_lock: AtomicUsize,
    used_lock: AtomicBool,
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
        is_user: bool,
        address_space: Arc<AddressSpace>,
        initial_pc: usize,
        initial_user_sp: usize,
        arg0: usize,
        // frame_init: F,
    ) -> ThreadRefSetup
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
                    .with_arg0(arg0)
                    .with_usermode(!is_user),
            );
            // frame_init(PreStart(&mut *trap_frame));
        }

        let ret = ThreadRefSetup(
            Arc::new(Thread {
                id,
                context: UnsafeCell::new(unsafe {
                    Context::new(_initial_return_trap as *const () as usize, trap_frame_addr)
                }),
                address_space: Some(address_space),
                valid: true,
                is_kernel: is_user,
                claimed_lock: AtomicUsize::new(usize::MAX),
                used_lock: AtomicBool::new(false),
            }),
            PhantomData,
        );

        ret
    }

    pub fn terminate(&mut self) {
        self.valid = false;
        self.address_space.take();
    }
}

/// Thread wrapper for exclusive access when the thread is being executed
/// on this CPU. Provides methods that allow access to exclusive fields.
/// Some accesses may still need some synchronization such as disabling
/// interrupts.
#[repr(transparent)]
#[derive(Debug)]
pub struct CurrentThread(Thread);

impl Deref for CurrentThread {
    type Target = Thread;
    fn deref(&self) -> &Thread {
        &self.0
    }
}

impl CurrentThread {
    // exclusive methods
}

macro_rules! convert_arc {
    ($arc:expr) => {{
        Arc::from_raw(Arc::into_raw($arc) as *const _)
    }};
}

macro_rules! convert_weak {
    ($weak:expr) => {{
        Weak::from_raw(Weak::into_raw($weak) as *const _)
    }};
}

// Check that two types certainly have the same layout
// Even if it failed, the conversion would be valid,
// but may be not zero-cost
const _: () = {
    assert!(size_of::<Arc<Thread>>() == size_of::<Arc<CurrentThread>>());
    assert!(align_of::<Arc<Thread>>() == align_of::<Arc<CurrentThread>>());
};

pub struct ThreadsIndexes {
    pub driver_task: usize,
    pub user_start: usize,
}

static mut THREADS: Vec<ThreadRef> = Vec::new();

pub static mut THREADS_INDEXES: ThreadsIndexes = ThreadsIndexes {
    // These will be initialized later, but before scheduling started
    driver_task: 1,
    user_start: 1,
};

pub unsafe fn get_current_thread_id() -> usize {
    Platform::get_current_thread_raw().as_ref().id
}

pub unsafe fn get_threads_count() -> usize {
    unsafe { THREADS.len() }
}

pub unsafe fn get_thread(id: usize) -> ThreadRef {
    unsafe { THREADS.get_mut(id).unwrap().clone() }
}

pub unsafe fn create_initial_invalid_thread() -> usize {
    let id = unsafe {
        let id = THREADS.len();
        let mut pr0 = Thread::new(id, false, Arc::new(AddressSpace::new()), 0, 0, 0);
        pr0.valid = false;
        THREADS.push(pr0.finish());
        id
    };
    let thread = get_thread(id);
    let thread = thread.try_claim().unwrap();
    let thread = thread.run();
    unsafe { Platform::set_current_thread_raw(&thread.into_raw()) }
    id
}

pub fn spawn_user(
    f: extern "C" fn() -> !,
    user_sp: usize,
    address_space: Arc<AddressSpace>,
    user_main: usize,
) -> usize {
    unsafe {
        let id = THREADS.len();
        let thread = Thread::new(id, true, address_space, f as usize, user_sp, user_main);
        THREADS.push(thread.finish());
        id
    }
}

pub fn spawn_kernel(f: extern "C" fn() -> !, address_space: Arc<AddressSpace>) -> usize {
    unsafe {
        let id = THREADS.len();
        let thread = Thread::new(id, false, address_space, f as usize, 0, 0);
        THREADS.push(thread.finish());
        id
    }
}

/**
Thread wrappers
**/

// !Clone !Sync Send
#[must_use]
#[repr(transparent)]
#[derive(Debug)]
pub struct ThreadRefSetup(Arc<Thread>, PhantomData<Cell<()>>);

impl ThreadRefSetup {
    pub fn finish(self) -> ThreadRef {
        ThreadRef(self.0)
    }
}

impl Deref for ThreadRefSetup {
    type Target = Thread;
    fn deref(&self) -> &Thread {
        &self.0
    }
}

impl DerefMut for ThreadRefSetup {
    fn deref_mut(&mut self) -> &mut Thread {
        debug_assert_eq!(Arc::strong_count(&self.0), 1);
        debug_assert_eq!(Arc::weak_count(&self.0), 0);
        unsafe { &mut *(Arc::as_ptr(&self.0) as *mut Thread) }
    }
}

// Clone Sync Send
#[repr(transparent)]
#[derive(Clone, Debug)]
pub struct ThreadRef(Arc<Thread>);

impl ThreadRef {
    pub fn try_claim(self) -> Result<ThreadRefClaimed, ThreadRef> {
        match self.0.claimed_lock.compare_exchange(
            usize::MAX,
            current_core_id(),
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            Ok(_) => Ok(ThreadRefClaimed(
                unsafe { convert_arc!(self.0) },
                PhantomData,
            )),
            Err(_) => Err(self),
        }
    }

    pub fn downgrade(&self) -> ThreadRefWeak {
        ThreadRefWeak(Arc::downgrade(&self.0))
    }
}

impl Deref for ThreadRef {
    type Target = Thread;
    fn deref(&self) -> &Thread {
        &self.0
    }
}

// Clone Sync Send
#[repr(transparent)]
#[derive(Clone)]
pub struct ThreadRefWeak(Weak<Thread>);

impl ThreadRefWeak {
    pub fn upgrade(&self) -> Option<ThreadRef> {
        self.0.upgrade().map(ThreadRef)
    }
}

// !Clone !Sync Send
#[must_use = ""]
#[repr(transparent)]
pub struct ThreadRefClaimed(Arc<CurrentThread>, PhantomData<Cell<()>>);

impl ThreadRefClaimed {
    pub fn release(self) -> ThreadRef {
        let this = ManuallyDrop::new(self);
        this.0.claimed_lock.store(usize::MAX, Ordering::Release);
        ThreadRef(unsafe { convert_arc!(core::ptr::read(&this.0)) })
    }

    pub fn run(self) -> ThreadRefRunning {
        while self.used_lock.load(Ordering::Relaxed) {
            core::hint::spin_loop();
        }
        let _ = self.used_lock.load(Ordering::Acquire);
        self.used_lock.store(true, Ordering::Release);
        let this = ManuallyDrop::new(self);
        ThreadRefRunning(unsafe { core::ptr::read(&this.0) }, PhantomData)
    }

    pub fn copy_ref(&self) -> ThreadRef {
        ThreadRef(unsafe { convert_arc!(self.0.clone()) })
    }

    pub fn downgrade(&self) -> ThreadRefWeak {
        ThreadRefWeak(unsafe { convert_weak!(Arc::downgrade(&self.0)) })
    }
}

impl Deref for ThreadRefClaimed {
    type Target = Thread;
    fn deref(&self) -> &Thread {
        &self.0
    }
}

impl Drop for ThreadRefClaimed {
    fn drop(&mut self) {
        /*const*/
        {
            panic!("ThreadRefClaimed must be consumed!")
        }
    }
}

// Clone !Sync !Send
#[must_use = ""]
#[repr(transparent)]
pub struct ThreadRefRunning(Arc<CurrentThread>, PhantomData<Cell<()>>);

impl ThreadRefRunning {
    pub unsafe fn stop(self) -> ThreadRef {
        (*self.0).claimed_lock.store(usize::MAX, Ordering::Release);
        (*self.0).used_lock.store(false, Ordering::Release);
        let this = ManuallyDrop::new(self);
        ThreadRef(unsafe { convert_arc!(core::ptr::read(&this.0)) })
    }

    pub fn copy_ref(&self) -> ThreadRef {
        ThreadRef(unsafe { convert_arc!(self.0.clone()) })
    }

    pub fn downgrade(&self) -> ThreadRefWeak {
        ThreadRefWeak(unsafe { convert_weak!(Arc::downgrade(&self.0)) })
    }

    pub fn into_raw(self) -> ThreadRawRunning {
        let this = ManuallyDrop::new(self);
        let arc = unsafe { core::ptr::read(&this.0) };
        unsafe { ThreadRawRunning(NonNull::new_unchecked(Arc::into_raw(arc).cast_mut())) }
    }
}

impl Deref for ThreadRefRunning {
    type Target = Thread;
    fn deref(&self) -> &Thread {
        &self.0
    }
}

impl Drop for ThreadRefRunning {
    fn drop(&mut self) {
        /*const*/
        {
            panic!("ThreadRefRunning must be consumed!")
        }
    }
}

pub fn with_current<R>(f: impl FnOnce(&CurrentThread) -> R) -> R {
    let raw = Platform::get_current_thread_raw();
    let thread: &CurrentThread = raw.as_ref();
    f(thread)
}

// !Clone !Copy !Sync !Send
#[must_use = ""]
#[repr(transparent)]
pub struct ThreadRawRunning(NonNull<CurrentThread>);

impl ThreadRawRunning {
    pub fn into_ref(self) -> ThreadRefRunning {
        ThreadRefRunning(unsafe { Arc::from_raw(self.0.as_ptr()) }, PhantomData)
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut CurrentThread {
        self.0.as_ptr()
    }

    #[inline]
    pub unsafe fn from_ptr(ptr: *mut CurrentThread) -> Self {
        Self(unsafe { NonNull::new_unchecked(ptr) })
    }

    #[inline]
    pub fn as_ref(&self) -> &CurrentThread {
        unsafe { self.0.as_ptr().as_ref_unchecked() }
    }
}
