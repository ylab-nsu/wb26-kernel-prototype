#![allow(unused)]

use crate::arch::traits::TargetPlatform;
use crate::Platform;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicUsize, Ordering};

pub type Mutex<T> = spin::mutex::Mutex<T, spin::relax::Spin>;
type MutexGuard<'a, T> = spin::mutex::MutexGuard<'a, T, spin::relax::Spin>;
pub type RwLock<T> = spin::rwlock::RwLock<T, spin::relax::Spin>;
pub type Once<T> = spin::once::Once<T, spin::relax::Spin>;
pub type LazyLock<T> = spin::lazylock::LazyLock<T, fn() -> T, spin::relax::Spin>;
pub type Barrier = spin::barrier::Barrier<spin::relax::Spin>;

struct CriticalSection;
critical_section::set_impl!(CriticalSection);

const NO_OWNER: usize = usize::MAX;
static OWNER: AtomicUsize = AtomicUsize::new(NO_OWNER);

#[inline]
fn hart_id() -> usize {
    0 // For now OS only supports single CPU core
}

unsafe impl critical_section::Impl for CriticalSection {
    unsafe fn acquire() -> critical_section::RawRestoreState {
        let was_enabled = Platform::interrupts_enabled();
        unsafe { Platform::disable_interrupts() };

        let me = hart_id();
        if OWNER.load(Ordering::Relaxed) == me {
            return was_enabled as u8;
        }
        while OWNER
            .compare_exchange_weak(NO_OWNER, me, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        was_enabled as u8 | 0b10
    }

    unsafe fn release(state: critical_section::RawRestoreState) {
        if state & 0b10 != 0 {
            OWNER.store(NO_OWNER, Ordering::Release);
        }
        if state & 0b01 != 0 {
            unsafe { Platform::enable_interrupts() };
        }
    }
}

pub struct InterruptMutex<T> {
    inner: Mutex<T>,
}

pub struct InterruptMutexGuard<'a, T: 'a + ?Sized> {
    guard: MutexGuard<'a, T>,
    critical_section: critical_section::RestoreState,
}

impl<T> InterruptMutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            inner: Mutex::new(value),
        }
    }

    pub fn lock(&self) -> InterruptMutexGuard<'_, T> {
        let state = unsafe { critical_section::acquire() };
        InterruptMutexGuard {
            guard: self.inner.lock(),
            critical_section: state,
        }
    }

    pub fn try_lock(&self) -> Option<InterruptMutexGuard<'_, T>> {
        let state = unsafe { critical_section::acquire() };
        if let Some(guard) = self.inner.try_lock() {
            Some(InterruptMutexGuard {
                guard,
                critical_section: state,
            })
        } else {
            unsafe { critical_section::release(state) }
            None
        }
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }
}

impl<T: ?Sized> Drop for InterruptMutexGuard<'_, T> {
    fn drop(&mut self) {
        unsafe { critical_section::release(self.critical_section) }
    }
}

impl<T: ?Sized> Deref for InterruptMutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.guard
    }
}

impl<T: ?Sized> DerefMut for InterruptMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.guard
    }
}

unsafe impl<T> Send for InterruptMutex<T> {}
unsafe impl<T> Sync for InterruptMutex<T> {}
