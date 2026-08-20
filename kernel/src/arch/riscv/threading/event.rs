use crate::arch::riscv::time::{TickDuration, TickInstant};
use crate::arch::traits::{
    TargetInstant, TargetTimerCallback, TargetTimerCallbackContext, TargetTimerImmediateCallback,
    TargetTimerQueue,
};
use crate::sync::Mutex;
use alloc::collections::{binary_heap::PeekMut, BinaryHeap};
use core::cmp::{Eq, Ord, PartialEq, PartialOrd};
use heapless::Vec;
use riscv::_export::critical_section;

struct Timers {
    queue: BinaryHeap<Timer>,
}

impl Timers {
    const fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
        }
    }

    fn set_timer(&mut self, target_time: Option<TickInstant>) {
        if let Some(next_time) = target_time {
            sbi::timer::set_timer(next_time.get_ticks()).expect("Can't set timer");
        } else {
            sbi::timer::set_timer(u64::MAX).expect("Can't set timer");
        }
    }

    fn set_timer_from_queue(&mut self) {
        self.set_timer(self.get_next_fire_time_from_queue());
    }

    fn get_next_fire_time_from_queue(&self) -> Option<TickInstant> {
        self.queue.peek().map(|e| e.target_time)
    }
}

static TIMERS: Mutex<Timers> = Mutex::new(Timers::new());

enum TimerKind {
    OneShot,
    Repeating { interval: TickDuration },
}

struct Timer {
    callback: TargetTimerCallback,
    target_time: TickInstant,
    start_or_last_fire_time: TickInstant,
    kind: TimerKind,
}

impl Ord for Timer {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        other.target_time.cmp(&self.target_time).then(
            other
                .start_or_last_fire_time
                .cmp(&self.start_or_last_fire_time),
        )
    }
}

impl PartialOrd for Timer {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Timer {
    fn eq(&self, other: &Self) -> bool {
        self.target_time == other.target_time
            && self.start_or_last_fire_time == other.start_or_last_fire_time
    }
}

impl Eq for Timer {}

enum PreparedTimerCallback {
    Reschedule,
    Immediate {
        callback: TargetTimerImmediateCallback,
        ctx: TargetTimerCallbackContext,
    },
}

fn fire_timers_ready_by_time(time: TickInstant) -> bool {
    let mut reschedule = false;
    loop {
        let mut callbacks_to_call: Vec<PreparedTimerCallback, 16> = Vec::new();
        {
            let mut timers = TIMERS.lock();
            while let Some(mut event) = timers.queue.peek_mut() {
                if event.target_time <= time {
                    if callbacks_to_call
                        .push(match event.callback {
                            TargetTimerCallback::Reschedule => PreparedTimerCallback::Reschedule,
                            TargetTimerCallback::Immediate { callback } => {
                                PreparedTimerCallback::Immediate {
                                    callback,
                                    ctx: TargetTimerCallbackContext {
                                        target_time: event.target_time,
                                        handle_time: time,
                                    },
                                }
                            }
                            TargetTimerCallback::Soft { callback: _ } => {
                                todo!("Implement soft timers")
                            }
                        })
                        .is_err()
                    {
                        break;
                    }
                    match event.kind {
                        TimerKind::Repeating { interval } => {
                            event.start_or_last_fire_time = time;
                            event.target_time = event.target_time + interval;
                        }
                        TimerKind::OneShot => {
                            PeekMut::pop(event);
                        }
                    }
                } else {
                    break;
                }
            }
        };
        if callbacks_to_call.is_empty() {
            break;
        }
        for prepared_callback in callbacks_to_call {
            match prepared_callback {
                PreparedTimerCallback::Reschedule => {
                    reschedule = true;
                }
                PreparedTimerCallback::Immediate { callback, ctx } => {
                    (callback)(ctx);
                }
            }
        }
    }
    TIMERS.lock().set_timer_from_queue();
    return reschedule;
}

pub fn fire_ready_timers() -> bool {
    fire_timers_ready_by_time(TickInstant::now())
}

fn add_timer(
    start_time: TickInstant,
    target_time: TickInstant,
    callback: TargetTimerCallback,
    interval: Option<TickDuration>,
) {
    critical_section::with(|_| {
        let mut timers = TIMERS.lock();
        match timers.queue.peek() {
            None => {
                timers.set_timer(Some(target_time));
            }
            Some(e) if e.target_time > target_time => {
                timers.set_timer(Some(target_time));
            }
            _ => (),
        };
        let timer_type =
            interval.map_or(TimerKind::OneShot, |i| TimerKind::Repeating { interval: i });
        timers.queue.push(Timer {
            target_time: target_time,
            start_or_last_fire_time: start_time,
            callback,
            kind: timer_type,
        });
    });
}

pub struct TimerQueue;

impl TargetTimerQueue for TimerQueue {
    fn add_oneshot_timer(delta: TickDuration, callback: TargetTimerCallback) {
        let start_time = TickInstant::now();
        add_timer(start_time, start_time + delta, callback, None);
    }
    fn add_repeating_timer(interval: TickDuration, callback: TargetTimerCallback) {
        let start_time = TickInstant::now();
        add_timer(
            TickInstant::now(),
            start_time + interval,
            callback,
            Some(interval),
        );
    }
}
