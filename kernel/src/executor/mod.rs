//! Async executor implementation.
//!
//! ### Implementation details:
//! * executor doesn't handle cases when future is polled after completion;
//! * executor doesn't handle cases when more than one worker thread tries to execute same task;
//! * executor doesn't support migrating futures between tasks.

use core::future::Future;

use alloc::sync::Arc;
use crossbeam_queue::SegQueue;

use crate::{executor::{futures::{sleep, yield_now}, task::Task}, time::Duration};

pub mod futures;

mod task;

pub struct Executor {
    task_queue: SegQueue<Arc<Task>>,
}

impl Executor {
    pub fn get() -> &'static Executor {
        static EXECUTOR: Executor = Executor::new();

        &EXECUTOR
    }
}

impl Executor {
    pub const fn new() -> Self {
        Self {
            task_queue: SegQueue::new(),
        }
    }

    pub fn spawn<F>(&self, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let task = Arc::new(Task::new(future));
        self.schedule_task(task);
    }

    fn schedule_task(&self, task: Arc<Task>) {
        self.task_queue.push(task);
    }

    fn worker(queue: &'static SegQueue<Arc<Task>>) {
        loop {
            match queue.pop() {
                Some(task) => task.poll(),
                None => continue,
            }
        }
    }
}

async fn compound() {
    debug!("Start compound");
    sleep(Duration::from_secs(1)).await;
    debug!("One compound");
    yield_now().await;
    debug!("Two compound");
    sleep(Duration::from_secs(1)).await;
    debug!("Finish compound");
}

async fn test() {
    debug!("Start");
    yield_now().await;
    debug!("One");
    yield_now().await;
    debug!("Two");
    sleep(Duration::from_secs(1)).await;
    debug!("Finish");
}

pub extern "C" fn executor() -> ! {
    Executor::get().spawn(test());

    Executor::get().spawn(async {
        debug!("Start2");
        sleep(Duration::from_millis(500)).await;
        debug!("One2");
        compound().await;
        debug!("Two2");
        sleep(Duration::from_secs(2)).await;
        debug!("Finish2");
    });

    Executor::worker(&Executor::get().task_queue);

    loop {}
}
