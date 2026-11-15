use crate::arch::traits::{TargetContext, TargetPlatform, TargetTrapFrame};
use crate::arch::{AddressSpace, Context, Platform, TrapFrame};
use crate::sync::current_core_id;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use core::cell::{Cell, UnsafeCell};
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::num::NonZeroUsize;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use derive_setters::Setters;
use pinned_init::{cast_init, init, InPlaceInit, Init};

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

#[deny(dead_code)]
#[derive(Default, Setters)]
pub struct ThreadBuilder {
    #[setters(strip_option)]
    initial_pc: Option<NonZeroUsize>,
    #[setters(strip_option)]
    initial_user_sp: Option<NonZeroUsize>,
    initial_arg0: usize,
    initial_arg1: usize,
    initial_arg2: usize,
    initial_arg3: usize,
    #[setters(strip_option)]
    kernel_stack: Option<NonNull<u8>>,
    #[setters(strip_option)]
    id: Option<usize>,
    #[setters(strip_option)]
    address_space: Option<Arc<AddressSpace>>,
    not_valid: bool,
    is_user: bool,
}

pub fn pseudo_allocate_kernel_stack(id: usize) -> NonNull<u8> {
    // TODO: add a better kernel stack allocator
    NonNull::from(unsafe { &mut KERNEL_STACKS[(id + 1) * 16] }).cast()
}

impl ThreadBuilder {
    pub fn build(self) -> ThreadRef {
        let ThreadBuilder {
            initial_pc,
            initial_user_sp,
            initial_arg0,
            initial_arg1,
            initial_arg2,
            initial_arg3,
            kernel_stack,
            id,
            address_space,
            not_valid,
            is_user,
        } = self;

        let id = id.expect("id must be set for a thread");
        let kernel_stack = kernel_stack
            .expect("kernel_stack must be set for a thread")
            .as_ptr();
        // TODO: make sure stack pointer is aligned by 16 bytes for riscv

        let trap_alignment: usize = align_of::<TrapFrame>();
        let trap_frame = kernel_stack.wrapping_byte_sub(size_of::<TrapFrame>());
        let offset = trap_frame.align_offset(trap_alignment);
        let trap_frame = trap_frame.wrapping_byte_sub((trap_alignment - offset) % trap_alignment);
        let trap_frame = trap_frame as *mut TrapFrame;

        debug_assert!(!trap_frame.is_null());
        debug_assert!(trap_frame.is_aligned());

        let initial_user_sp: usize = match (initial_user_sp, is_user) {
            (Some(user_sp), true) => user_sp.get(),
            (Some(_), false) => panic!("user_sp must not be set for a kernel thread"),
            (None, true) => panic!("user_sp must be set for a user thread"),
            (None, false) => 0,
        };

        unsafe {
            trap_frame.write(
                TrapFrame::default()
                    .with_pc(usize::from(
                        initial_pc.expect("initial_pc must be set for a thread"),
                    ))
                    .with_sp(initial_user_sp)
                    .with_arg0(initial_arg0)
                    .with_arg1(initial_arg1)
                    .with_arg2(initial_arg2)
                    .with_arg3(initial_arg3)
                    .with_usermode(is_user),
            );
        }

        let address_space = match (address_space, not_valid) {
            (Some(t), _) => Some(t),
            (None, false) => {
                let t = Arc::new(AddressSpace::new());

                Some(t)
            }
            (None, true) => None,
        };

        ThreadRef(
            Arc::init(init!(Thread {
                id,
                context <- unsafe_cell_init(
                    Context::init(_initial_return_trap as *const () as usize, trap_frame.addr())
                ),
                address_space: address_space,
                valid: !not_valid,
                is_user,
                claimed_lock: AtomicUsize::new(usize::MAX),
                used_lock: AtomicBool::new(false),
            }))
            .expect("out of memory while creating a thread"), // Will not work on stable Rust
        )
    }
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
    pub is_user: bool,
    claimed_lock: AtomicUsize,
    used_lock: AtomicBool,
}

impl Thread {
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
    unsafe { Platform::get_current_thread_raw().as_ref().id }
}

pub unsafe fn get_threads_count() -> usize {
    unsafe { THREADS.len() }
}

pub unsafe fn get_thread(id: usize) -> ThreadRef {
    unsafe { THREADS.get_mut(id).unwrap().clone() }
}
// Don't worry, all this THREADS array thing will be removed soon
pub unsafe fn push_thread(thread: ThreadRef) {
    unsafe {
        THREADS.push(thread);
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
    pub fn try_claim(self) -> Result<ClaimedThreadRef, ThreadRef> {
        match self.0.claimed_lock.compare_exchange(
            usize::MAX,
            current_core_id(),
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            Ok(_) => Ok(ClaimedThreadRef(
                unsafe { convert_arc!(self.0) },
                PhantomData,
            )),
            Err(_) => Err(self),
        }
    }

    pub fn downgrade(&self) -> ThreadWeak {
        ThreadWeak(Arc::downgrade(&self.0))
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
pub struct ThreadWeak(Weak<Thread>);

impl ThreadWeak {
    pub fn upgrade(&self) -> Option<ThreadRef> {
        self.0.upgrade().map(ThreadRef)
    }
}

// !Clone !Sync Send
#[must_use = ""]
#[repr(transparent)]
pub struct ClaimedThreadRef(Arc<CurrentThread>, PhantomData<Cell<()>>);

impl ClaimedThreadRef {
    pub fn release(self) -> ThreadRef {
        let this = ManuallyDrop::new(self);
        this.0.claimed_lock.store(usize::MAX, Ordering::Release);
        ThreadRef(unsafe { convert_arc!(core::ptr::read(&this.0)) })
    }

    pub fn run(self) -> RunningThreadRef {
        while self.used_lock.load(Ordering::Relaxed) {
            core::hint::spin_loop();
        }
        let _ = self.used_lock.load(Ordering::Acquire);
        self.used_lock.store(true, Ordering::Release);
        let this = ManuallyDrop::new(self);
        RunningThreadRef(unsafe { core::ptr::read(&this.0) }, PhantomData)
    }

    pub fn copy_ref(&self) -> ThreadRef {
        ThreadRef(unsafe { convert_arc!(self.0.clone()) })
    }

    pub fn downgrade(&self) -> ThreadWeak {
        ThreadWeak(unsafe { convert_weak!(Arc::downgrade(&self.0)) })
    }
}

impl Deref for ClaimedThreadRef {
    type Target = CurrentThread;
    fn deref(&self) -> &CurrentThread {
        &self.0
    }
}

impl Drop for ClaimedThreadRef {
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
pub struct RunningThreadRef(Arc<CurrentThread>, PhantomData<Cell<()>>);

impl RunningThreadRef {
    pub unsafe fn stop(self) -> ThreadRef {
        (*self.0).claimed_lock.store(usize::MAX, Ordering::Release);
        (*self.0).used_lock.store(false, Ordering::Release);
        let this = ManuallyDrop::new(self);
        ThreadRef(unsafe { convert_arc!(core::ptr::read(&this.0)) })
    }

    pub fn copy_ref(&self) -> ThreadRef {
        ThreadRef(unsafe { convert_arc!(self.0.clone()) })
    }

    pub fn downgrade(&self) -> ThreadWeak {
        ThreadWeak(unsafe { convert_weak!(Arc::downgrade(&self.0)) })
    }

    pub fn into_raw(self) -> ThreadRawRunning {
        let this = ManuallyDrop::new(self);
        let arc = unsafe { core::ptr::read(&this.0) };
        unsafe { ThreadRawRunning(NonNull::new_unchecked(Arc::into_raw(arc).cast_mut())) }
    }
}

impl Deref for RunningThreadRef {
    type Target = CurrentThread;
    fn deref(&self) -> &CurrentThread {
        &self.0
    }
}

impl Drop for RunningThreadRef {
    fn drop(&mut self) {
        /*const*/
        {
            panic!("ThreadRefRunning must be consumed!")
        }
    }
}

pub fn with_current<R>(f: impl FnOnce(&CurrentThread) -> R) -> R {
    let raw = unsafe { Platform::get_current_thread_raw() };
    let thread: &CurrentThread = raw.as_ref();
    f(thread)
}

// !Clone !Copy !Sync !Send
#[must_use = ""]
#[repr(transparent)]
pub struct ThreadRawRunning(NonNull<CurrentThread>);

impl ThreadRawRunning {
    pub fn into_ref(self) -> RunningThreadRef {
        RunningThreadRef(unsafe { Arc::from_raw(self.0.as_ptr()) }, PhantomData)
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

#[inline]
fn unsafe_cell_init<T>(init: impl Init<T>) -> impl Init<UnsafeCell<T>> {
    unsafe { cast_init(init) }
}
