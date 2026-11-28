use crate::arch::riscv::memory::layout::KERNEL_LAYOUT;
use crate::arch::traits::TargetPlatform;
use crate::threading::thread::{CurrentThread, RunningThreadRaw};
use core::arch::asm;
use core::ptr::NonNull;

pub struct RiscvPlatform;

impl TargetPlatform for RiscvPlatform {
    fn init() {
        todo!()
    }

    fn ipi() {
        todo!()
    }

    fn sleep() {
        todo!()
    }

    fn shutdown() {
        todo!()
    }

    fn wfi() {
        riscv::asm::wfi();
    }

    fn disable_interrupts() {
        riscv::interrupt::disable()
    }

    unsafe fn enable_interrupts() {
        unsafe { riscv::interrupt::enable() }
    }

    fn interrupts_enabled() -> bool {
        riscv::register::sstatus::read().sie()
    }

    unsafe fn set_interrupts_enabled(enable: bool) {
        match enable {
            true => unsafe { Self::enable_interrupts() },
            false => Self::disable_interrupts(),
        }
    }

    fn interrupt_free<F, R>(f: F) -> R
    where
        F: FnOnce() -> R,
    {
        riscv::interrupt::free(f)
    }

    fn micros() -> u64 {
        riscv::register::time::read64() / 10
    }

    fn get_user_va_offset() -> usize {
        unsafe { KERNEL_LAYOUT.user_va_offset }
    }

    #[inline]
    unsafe fn get_current_thread_raw() -> RunningThreadRaw {
        let ptr: *mut CurrentThread;
        unsafe { asm!("mv {}, tp", out(reg) ptr, options(pure, nomem, nostack)) };
        unsafe { RunningThreadRaw::from_ptr(NonNull::new(ptr).unwrap()) }
    }

    #[inline]
    unsafe fn set_current_thread_raw(raw: &RunningThreadRaw) {
        let ptr = raw.as_ptr();
        unsafe { asm!("mv tp, {}", in(reg) ptr.as_ptr(), options(nomem, nostack)) };
    }
}
