use crate::arch::PlatformInstant;
use alloc::boxed::Box;

pub struct TimerCallbackContext {
    pub handle_time: PlatformInstant,
    pub target_time: PlatformInstant,
}

pub trait TimerCallbackFunction: FnMut(TimerCallbackContext) + Send {}
impl<T: FnMut(TimerCallbackContext) + Send> TimerCallbackFunction for T {}
pub type TimerCallbackFunctionBoxed = Box<dyn TimerCallbackFunction>;

pub enum TimerCallback {
    Reschedule,
    // Inside interrupt
    Immediate {
        callback: TimerCallbackFunctionBoxed,
    },
    // TODO: Elsewhere
    Soft {
        callback: fn(),
    },
}

impl TimerCallback {
    pub fn immediate<T: TimerCallbackFunction + 'static>(callback: T) -> Self {
        TimerCallback::Immediate {
            callback: Box::new(callback),
        }
    }

    pub fn reschedule() -> Self {
        TimerCallback::Reschedule
    }

    pub fn soft(_: fn()) -> Self {
        todo!("implement timer soft callbacks");
    }
}
