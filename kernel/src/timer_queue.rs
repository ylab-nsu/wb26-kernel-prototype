use core::sync::atomic::{AtomicBool, Ordering};

use alloc::{
    boxed::Box,
    collections::binary_heap::BinaryHeap,
    sync::{Arc, Weak},
};
use heapless::Vec;

use crate::{
    arch::{time::Alarm, traits::TargetAlarm},
    sync::InterruptMutex,
    time::{Duration, Instant},
};

pub struct TimerQueue {
    inner: InterruptMutex<TimerQueueInner>,
}

struct TimerQueueInner {
    queue: BinaryHeap<ScheduledTimer>,
    alarm: &'static Alarm,
}

pub struct CancelHandle {
    cancel_flag: Weak<AtomicBool>,
}

pub struct TimerCallbackContext {
    pub handle_time: Instant,
    pub target_time: Instant,
}

pub enum TimerCallback {
    /// Triggers a reschedule on return from interrupt handler
    Reschedule,
    /// Calls a callback function inside the interrupt handler
    Immediate(Box<dyn FnMut(TimerCallbackContext)>),
}

pub enum ScheduleKind {
    OneShot,
    Repeating(Duration),
}

unsafe impl Send for TimerQueue {}
unsafe impl Sync for TimerQueue {}

impl TimerQueue {
    pub const fn new(alarm: &'static Alarm) -> Self {
        Self {
            inner: InterruptMutex::new(TimerQueueInner {
                queue: BinaryHeap::new(),
                alarm,
            }),
        }
    }

    pub fn register(
        &self,
        callback: TimerCallback,
        delay: Duration,
        schedule_kind: ScheduleKind,
    ) -> CancelHandle {
        let current_time = Instant::now();

        let mut inner = self.inner.lock();
        let cancel_flag = Arc::new(AtomicBool::new(false));
        inner.queue.push(ScheduledTimer {
            callback,
            target_time: current_time + delay,
            last_fire_time: current_time,
            schedule_kind,
            cancel_flag: cancel_flag.clone(),
        });
        CancelHandle {
            cancel_flag: Arc::downgrade(&cancel_flag),
        }
    }

    pub fn handle_timers(&self) -> bool {
        let current_time = Instant::now();

        let mut reschedule = false;
        loop {
            let mut ready_timers: Vec<ScheduledTimer, 16> = Vec::new();
            {
                let queue = &mut self.inner.lock().queue;
                loop {
                    let is_ready = queue
                        .peek()
                        .map(|event| event.target_time <= current_time)
                        .unwrap_or(false);

                    if !is_ready {
                        break;
                    }

                    let event = queue.pop().unwrap();

                    if let Err(event) = ready_timers.push(event) {
                        queue.push(event);
                        break;
                    }
                }
            };

            if ready_timers.is_empty() {
                break;
            }

            let mut timers_to_reinsert: Vec<ScheduledTimer, 16> = Vec::new();

            for mut event in ready_timers {
                if event.cancel_flag.load(Ordering::Relaxed) {
                    continue;
                }
                match &mut event.callback {
                    TimerCallback::Immediate(callback) => {
                        let ctx = TimerCallbackContext {
                            target_time: event.target_time,
                            handle_time: current_time,
                        };
                        (callback)(ctx);
                    }
                    TimerCallback::Reschedule => {
                        reschedule = true;
                    }
                }

                match event.schedule_kind {
                    ScheduleKind::Repeating(interval) => {
                        event.last_fire_time = current_time;
                        event.target_time = event.target_time + interval;
                        let _ = timers_to_reinsert.push(event);
                    }
                    _ => {}
                }
            }

            if !timers_to_reinsert.is_empty() {
                let queue = &mut self.inner.lock().queue;
                for event in timers_to_reinsert {
                    queue.push(event);
                }
            }
        }

        let inner = self.inner.lock();
        if let Some(timer) = inner.queue.peek() {
            inner
                .alarm
                .schedule(timer.target_time.as_ticks())
                .expect("Could not schedule an alarm");
        }
        reschedule
    }
}

impl CancelHandle {
    pub fn cancel(&self) {
        if let Some(cancel_flag) = self.cancel_flag.upgrade() {
            cancel_flag.store(true, Ordering::Relaxed);
        }
    }
}

struct ScheduledTimer {
    callback: TimerCallback,
    target_time: Instant,
    last_fire_time: Instant,
    schedule_kind: ScheduleKind,
    cancel_flag: Arc<AtomicBool>,
}

impl Ord for ScheduledTimer {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        other
            .target_time
            .cmp(&self.target_time)
            .then(other.last_fire_time.cmp(&self.last_fire_time))
    }
}

impl PartialOrd for ScheduledTimer {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ScheduledTimer {
    fn eq(&self, other: &Self) -> bool {
        self.target_time == other.target_time && self.last_fire_time == other.last_fire_time
    }
}

impl Eq for ScheduledTimer {}
