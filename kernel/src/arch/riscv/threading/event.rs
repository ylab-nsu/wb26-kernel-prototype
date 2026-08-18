pub use crate::arch::riscv::time::{TickDuration, TickInstant};
use crate::sync::Mutex;
use alloc::collections::{binary_heap::PeekMut, BinaryHeap};
use core::cmp::{Eq, Ord, PartialEq, PartialOrd};

static TIMERS: Mutex<BinaryHeap<Timer>> = Mutex::new(BinaryHeap::new());

enum TimerType {
    OneShot,
    Repeating,
}

type TimerCallback = fn(&TickInstant);

// Fire time is passed into the callback
struct Timer {
    callback: TimerCallback,
    target_time: TickInstant,
    start_time: TickInstant,
    inner: TimerType,
}

impl Ord for Timer {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        other
            .target_time
            .cmp(&self.target_time)
            .then(other.start_time.cmp(&self.start_time))
    }
}

impl PartialOrd for Timer {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Timer {
    fn eq(&self, other: &Self) -> bool {
        self.target_time == other.target_time && self.start_time == other.start_time
    }
}

impl Eq for Timer {}

pub fn add_timer_at(
    start_time: TickInstant,
    interval: TickDuration,
    callback: TimerCallback,
    repeat: bool,
) {
    TIMERS.lock().push(Timer {
        target_time: start_time + interval,
        start_time: start_time,
        callback,
        inner: if repeat {
            TimerType::Repeating
        } else {
            TimerType::OneShot
        },
    });
}

pub fn add_timer(interval: TickDuration, callback: TimerCallback, repeat: bool) {
    add_timer_at(TickInstant::now(), interval, callback, repeat);
}

pub fn peek_next_interrupt_time() -> Option<TickInstant> {
    TIMERS.lock().peek().map(|e| e.target_time)
}

pub fn drain_events_by_time(time: TickInstant) {
    let mut timers = TIMERS.lock();
    while let Some(mut event) = timers.peek_mut() {
        if event.target_time <= time {
            (event.callback)(&time);
            match event.inner {
                TimerType::Repeating => {
                    let interval = event.target_time - event.start_time;
                    event.start_time = time;
                    event.target_time = time + interval;
                }
                TimerType::OneShot => {
                    PeekMut::pop(event);
                }
            }
        } else {
            break;
        }
    }
}
