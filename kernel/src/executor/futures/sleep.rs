use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::arch::{
    traits::{TargetInstant, TargetTimerQueue},
    PlatformDuration, PlatformInstant, TimerQueue,
};
use crate::timers::TimerCallback;

struct SleepFuture {
    target_time: PlatformInstant,
    is_registered: bool,
}

impl SleepFuture {
    pub fn new(target_time: PlatformInstant) -> Self {
        SleepFuture {
            target_time: target_time,
            is_registered: false,
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
            TimerQueue::add_oneshot_timer(
                self.target_time - current_time,
                TimerCallback::one_shot(move |_| {
                    waker.wake();
                }),
            );
        }
        Poll::Pending
    }
}

pub fn sleep<T: Into<PlatformDuration>>(duration: T) -> impl Future<Output = ()> {
    let current_time = PlatformInstant::now();

    SleepFuture::new(current_time + duration.into())
}
