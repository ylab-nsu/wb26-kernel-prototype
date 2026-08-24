use crate::arch::{PlatformDuration, PlatformInstant};
use alloc::boxed::Box;
use alloc::sync::Arc;
use core::sync::atomic::AtomicBool;

pub struct TimerCallbackContext {
    pub handle_time: PlatformInstant,
    pub target_time: PlatformInstant,
}

pub enum TimerCallback {
    OneShot {
        callback: Box<dyn FnOnce(TimerCallbackContext) + Send>,
    },
    Repeating { 
        callback: Box<dyn FnMut(TimerCallbackContext) + Send>,
        interval: PlatformDuration 
    },
}

impl TimerCallback {
    pub fn one_shot<T: FnOnce(TimerCallbackContext) + Send + 'static>(callback: T) -> Self {
        TimerCallback::OneShot {
            callback: Box::new(callback),
        }
    }

    pub fn repeating<T: FnMut(TimerCallbackContext) + Send + 'static>(callback: T, interval: PlatformDuration) -> Self {
        TimerCallback::Repeating {
            callback: Box::new(callback),
            interval
        }
    }
}


pub enum TimerKind {
    Reschedule,
    // Inside interrupt
    Immediate {
        callback: TimerCallback
    },
    // TODO: Elsewhere
    Soft,
}

pub struct Timer {
    pub target_time: PlatformInstant,
    pub start_or_last_fire_time: PlatformInstant,
    pub kind: TimerKind,
    pub handle: Arc<TimerHandle>,
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

pub struct TimerHandle {
    to_stop: AtomicBool,
}

impl TimerHandle {
    pub fn new() -> Self {
        TimerHandle {
            to_stop: AtomicBool::new(false),
        }
    }

    pub fn stop(&self) {
        self.to_stop
            .store(true, core::sync::atomic::Ordering::Release)
    }

    pub fn is_stoped(&self) -> bool {
        self.to_stop.load(core::sync::atomic::Ordering::Acquire)
    }
}
