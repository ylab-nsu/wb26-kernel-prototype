use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use crate::arch::{
    traits::{TargetInstant, TargetTimerQueue},
    PlatformDuration, PlatformInstant, TimerQueue,
};
use crate::timers::{TimerCallback, TimerHandle};
use alloc::sync::Weak;

struct SleepFuture {
    target_time: PlatformInstant,
    is_registered: bool,
    timer_handle: Option<Weak<TimerHandle>>,
}

impl SleepFuture {
    pub fn new(target_time: PlatformInstant) -> Self {
        SleepFuture {
            target_time: target_time,
            is_registered: false,
            timer_handle: None,
        }
    }
}

impl Future for SleepFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current_time = PlatformInstant::now();

        if current_time > self.target_time {
            return Poll::Ready(());
        } else if !self.is_registered {
            self.is_registered = true;
            let waker = cx.waker().clone();

            let handle = TimerQueue::add_timer(
                self.target_time,
                TimerCallback::one_shot(move |_| {
                    waker.wake();
                }),
            );
            self.timer_handle = Some(handle);
        }
        Poll::Pending
    }
}

impl Drop for SleepFuture {
    fn drop(&mut self) {
        if let Some(handle) = self.timer_handle.take() {
            handle.upgrade().map(|handle| handle.stop());
        }
    }
}

pub fn sleep<T: Into<PlatformDuration>>(duration: T) -> impl Future<Output = ()> {
    let current_time = PlatformInstant::now();

    SleepFuture::new(current_time + duration.into())
}

impl Into<PlatformDuration> for u32 {
    fn into(self) -> PlatformDuration {
        Duration::from_micros(self as u64).into()
    }
}
