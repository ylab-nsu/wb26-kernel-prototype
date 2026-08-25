use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

struct OneshotFuture {
    complete: bool,
}

impl Future for OneshotFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.complete {
            Poll::Ready(())
        } else {
            self.get_mut().complete = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

pub fn yield_now() -> impl Future<Output = ()> {
    OneshotFuture { complete: false }
}
