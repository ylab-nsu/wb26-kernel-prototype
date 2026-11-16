use crate::drivers::TEST_DRIVER_ANY;
use crate::threading::switch::switch_to;
use crate::threading::thread::{
    get_current_thread_id, get_thread, get_threads_count, THREADS_INDEXES,
};

static mut LAST_USER: usize = unsafe { THREADS_INDEXES.user_start - 1 };

#[export_name = "_reschedule_rust"]
pub extern "C" fn reschedule() {
    let next_thread = unsafe {
        if TEST_DRIVER_ANY {
            THREADS_INDEXES.driver_task
        } else {
            let mut checked = LAST_USER;
            let next_thread = loop {
                checked = if checked < get_threads_count() - 1 {
                    checked + 1
                } else {
                    THREADS_INDEXES.user_start
                };
                if get_thread(checked).valid {
                    break checked;
                }
            };
            LAST_USER = next_thread;
            next_thread
        }
    };

    if next_thread == unsafe { get_current_thread_id() } {
        debug!("Want to switch to myself, do not switch");
        return;
    }

    let next = unsafe { get_thread(next_thread) };
    let next = next.try_claim().unwrap(); // No multithreading yet, can unwrap

    unsafe {
        switch_to(next, true);
    }
}
