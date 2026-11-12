use core::future::Future;

use alloc::sync::Arc;
use crossbeam_queue::SegQueue;

use crate::executor::{futures::{yield_now, sleep}, task::Task};

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
    sleep(1_000_000).await;
    debug!("One compound");
    yield_now().await;
    debug!("Two compound");
    sleep(1_000_000).await;
    debug!("Finish compound");
}

async fn test() {
    debug!("Start");
    yield_now().await;
    debug!("One");
    yield_now().await;
    debug!("Two");
    sleep(1_000_000).await;
    debug!("Finish");
}

pub extern "C" fn executor() -> ! {
    Executor::get().spawn(test());

    Executor::get().spawn(async {
        debug!("Start2");
        sleep(500_000).await;
        debug!("One2");
        compound().await;
        debug!("Two2");
        sleep(2_000_000).await;
        debug!("Finish2");
    });

    Executor::worker(&Executor::get().task_queue);

    loop {}
}
