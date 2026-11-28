use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{
    time::{Duration, Instant},
    timer_queue::CancelHandle,
    TIMER_QUEUE,
};

struct SleepFuture {
    target_time: Instant,
    timer_handle: Option<CancelHandle>,
}

impl Future for SleepFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current_time = Instant::now();

        if current_time > self.target_time {
            Poll::Ready(())
        } else {
            if self.timer_handle.is_none() {
                let waker = cx.waker().clone();

                let handle = TIMER_QUEUE.enqueue_oneshot(
                    move |ctx| {
                        debug!(
                            "SleepFuture timer expired, target_time: {:?}",
                            ctx.target_time
                        );
                        waker.wake_by_ref()
                    },
                    self.target_time - current_time,
                );

                self.get_mut().timer_handle = Some(handle);
            }

            Poll::Pending
        }
    }
}

impl Drop for SleepFuture {
    fn drop(&mut self) {
        if let Some(handle) = &self.timer_handle {
            handle.cancel();
        }
    }
}

pub fn sleep(duration: Duration) -> impl Future<Output = ()> {
    SleepFuture {
        target_time: Instant::now() + duration,
        timer_handle: None,
    }
}
