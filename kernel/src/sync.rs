#![allow(unused)]

use crate::arch::traits::TargetPlatform;
use crate::Platform;
use core::sync::atomic::{AtomicUsize, Ordering};

pub type Mutex<T> = spin::mutex::Mutex<T, spin::relax::Spin>;
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
