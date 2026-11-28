use crate::drivers::TEST_DRIVER_ANY;
use crate::threading::switch::switch_to;
use crate::threading::thread::{get_current_thread_id, with_current, Thread, ThreadRef};
use cordyceps::MpscQueue;
use core::sync::atomic::Ordering;

static SCHEDULER_STUB: Thread = unsafe { Thread::new_stub() };
static SCHEDULER_QUEUE: MpscQueue<Thread> =
    unsafe { MpscQueue::new_with_static_stub(&SCHEDULER_STUB) };

#[export_name = "_reschedule_rust"]
pub extern "C" fn reschedule() {
    let prev_is_valid = with_current(|current| current.valid);

    let next = match dequeue_thread() {
        Some(thread) => thread,
        None => {
            // TODO: maybe previous thread doesn't want to ever run again, but how it can tell us?
            if prev_is_valid {
                info!("Scheduler queue is empty, returning back to previous thread");
                return;
            } else {
                panic!("Scheduler queue is empty, and previous thread is invalid");
            }
        }
    };

    let next = next.try_claim().unwrap(); // No multithreading yet, can unwrap

    unsafe {
        switch_to(next, prev_is_valid);
    }
}

#[inline]
pub fn schedule_thread(thread: ThreadRef) {
    thread
        .is_enqueued
        .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
        .expect("Thread is already scheduled in another queue");
    SCHEDULER_QUEUE.enqueue(thread);
}

#[inline]
fn dequeue_thread() -> Option<ThreadRef> {
    match SCHEDULER_QUEUE.dequeue() {
        Some(thread) => {
            debug_assert!(thread.is_enqueued.load(Ordering::Relaxed) == true);
            thread.is_enqueued.store(false, Ordering::Release);
            Some(thread)
        }
        None => None,
    }
}
