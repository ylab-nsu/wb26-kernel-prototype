use crate::arch::traits::{TargetAddressSpace, TargetPlatform};
use crate::arch::Platform;
use crate::threading::thread::{ThreadRawRunning, ThreadRefClaimed};

#[allow(improper_ctypes)]
extern "C" {
    fn _switch_thread(
        prev_context: ThreadRawRunning,
        schedule_prev: bool,
        next_context: ThreadRawRunning,
    ) -> usize;
}

pub unsafe fn switch_to(next: ThreadRefClaimed, schedule_prev: bool) {
    // TODO: need to disable interrupts here and save previous state
    let next = next.run();
    let prev = Platform::get_current_thread_raw();
    info!("Switching threads from {} to {}", prev.as_ref().id, next.id);

    unsafe {
        next.address_space
            .as_ref()
            .expect("Trying to switch to thread without address space")
            .switch();
    }

    let next_raw = next.into_raw();
    unsafe {
        Platform::set_current_thread_raw(&next_raw);
    }

    unsafe {
        _switch_thread(prev, schedule_prev, next_raw);
    }
}

#[export_name = "_finish_switch_rust"]
extern "C" fn finish_switch(prev: ThreadRawRunning, schedule_prev: bool) {
    let prev = prev.into_ref();
    let prev = unsafe { prev.stop() };
    if schedule_prev {
        // Will add to the scheduler queue when the scheduler is changed to use queues
        let _ = prev;
    }
    // TODO: need to restore interrupts state
}
