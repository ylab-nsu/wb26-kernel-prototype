use core::{future::Future, pin::Pin, sync::atomic::AtomicUsize, task::{Context, Poll, RawWaker, RawWakerVTable, Waker}};

use alloc::{boxed::Box, collections::VecDeque, sync::Arc, task::Wake};

use crate::{executor::futures::{oneshot::yield_now, sleep::sleep}, sync::Mutex};
use riscv::_export::critical_section;

pub mod futures;

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

// fn no_op(_: *const ()) {}
// fn no_op_clone(_: *const ()) -> RawWaker {
//     noop_raw_waker()
// }

// static RWVT: RawWakerVTable = RawWakerVTable::new(no_op_clone, no_op, no_op, no_op);

// #[inline]
// fn noop_raw_waker() -> RawWaker {
//     RawWaker::new(core::ptr::null(), &RWVT)
// }

struct Task {
    future: Mutex<Option<Pin<Box<dyn Future<Output = ()> + Send>>>>
}

impl Task {
    fn schedule(self: Arc<Self>) {
        critical_section::with(|_| {
            WAKED_TASKS.lock().push_back(self)
        })
    }

    fn poll(self: &Arc<Self>) {
        let waker: Waker = self.clone().into();
        let mut cx = Context::from_waker(&waker);

        let mut future = self.future.lock();
        if let Some(mfuture) = future.as_mut() {
            if mfuture.as_mut().poll(&mut cx).is_ready() {
                *future = None;
                info!("Task finished");
                ACTIVE_TASKS.fetch_sub(1, core::sync::atomic::Ordering::AcqRel);
            }
        }
    }
}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        self.schedule();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.clone().schedule();
    }
}

static WAKED_TASKS: Mutex<VecDeque<Arc<Task>>> = Mutex::new(VecDeque::new());
static ACTIVE_TASKS: AtomicUsize = AtomicUsize::new(0);

pub fn spawn(future: impl Future<Output = ()> + Send + 'static) {
    let task = Arc::new(Task {
        future: Mutex::new(Some(Box::pin(future))),
    });

    ACTIVE_TASKS.fetch_add(1, core::sync::atomic::Ordering::AcqRel);

    task.schedule();
}

pub extern "C" fn executor() -> ! {
    spawn(test());

    spawn(async {
        debug!("Start2");
        sleep(500_000).await;
        debug!("One2");
        compound().await;
        debug!("Two2");
        sleep(2_000_000).await;
        debug!("Finish2");
    });

    let mut run = true;
    while run {
        // heapless::
        critical_section::with(|_| {
            let task = WAKED_TASKS.lock().pop_front();

            if let Some(task) = task {
                task.poll();
            } else if ACTIVE_TASKS.load(core::sync::atomic::Ordering::Acquire) == 0 {
                info!("No tasks left");
                run = false;
            }
        })
    }

    loop {
        
    }
}

// pub fn executor() {
//     let future = test();

//     let waker = Waker::noop();
//     // let waker = unsafe { Waker::from_raw(noop_raw_waker()) };
//     let mut ctx = Context::from_waker(waker);
//     let mut pinned_future = pin!(future);
//     let mut pinned_future = Box::pin(future);

//     loop {
//         match pinned_future.as_mut().poll(&mut ctx) {
//             Poll::Ready(_) => {
//                 debug!("Ready");
//                 break;
//             }
//             Poll::Pending => {
//                 // debug!("Not ready");
//             }
//         }
//     }
// }
