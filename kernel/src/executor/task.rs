use core::{future::Future, pin::Pin, task::Context};

use alloc::{boxed::Box, sync::Arc};
use futures_task::{waker, ArcWake};

use crate::{executor::Executor, sync::Mutex};

type BoxedFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

pub struct Task {
    future: Mutex<BoxedFuture>,
}

impl Task {
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Self {
            future: Mutex::new(Box::pin(future)),
        }
    }

    pub fn poll(self: Arc<Self>) {
        let waker = waker(self.clone());
        let mut cx = Context::from_waker(&waker);

        let mut future = self.future.try_lock().unwrap_or_else(|| {
            unreachable!(
                "failed to acquire future lock, perhaps someone is running it right now =_="
            )
        });

        let result = future.as_mut().poll(&mut cx);

        if result.is_ready() {
            debug!("Future is ready!");
        }
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        Executor::get().schedule_task(arc_self.clone());
    }
}
