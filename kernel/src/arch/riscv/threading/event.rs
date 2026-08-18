use crate::arch::riscv::time::{TickDuration, TickInstant};
use crate::arch::traits::TargetTimerQueue;
use crate::sync::Mutex;
use alloc::collections::{binary_heap::PeekMut, BinaryHeap};
use core::cmp::{Eq, Ord, PartialEq, PartialOrd};

static TIMERS: Mutex<BinaryHeap<Timer>> = Mutex::new(BinaryHeap::new());

enum TimerType {
    OneShot,
    Repeating,
}

type TimerCallback = fn(TickInstant);

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

pub struct TimerQueue;

impl TargetTimerQueue for TimerQueue {
    type TargetDuration = TickDuration;
    type TargetInstant = TickInstant;
    type TargetTimerCallback = TimerCallback;

    fn add_timer_at(
        start_time: Self::TargetInstant,
        interval: Self::TargetDuration,
        callback: Self::TargetTimerCallback,
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

    fn fire_timers_ready_by_time(time: Self::TargetInstant) {
        let mut timers = TIMERS.lock();
        while let Some(mut event) = timers.peek_mut() {
            if event.target_time <= time {
                (event.callback)(time);
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

    fn get_next_fire_time() -> Option<Self::TargetInstant> {
        TIMERS.lock().peek().map(|e| e.target_time)
    }
}
