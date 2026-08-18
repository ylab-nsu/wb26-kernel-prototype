use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::arch::{Platform, traits::TargetPlatform};

struct SleepFuture {
    target_time: u64,
}

impl Future for SleepFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let current_time = Platform::micros();

        if current_time > self.target_time {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

pub fn sleep(micros: u64) -> impl Future<Output = ()> {
    let current_time = Platform::micros();

    SleepFuture {
        target_time: current_time + micros,
    }
}
