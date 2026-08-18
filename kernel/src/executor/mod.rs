use core::{future::Future, pin::Pin, task::{Context, Poll, RawWaker, RawWakerVTable, Waker}};

use alloc::{boxed::Box, collections::VecDeque};

use crate::{executor::futures::{oneshot::yield_now, sleep::sleep}, sync::Mutex};

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
    future: Pin<Box<dyn Future<Output = ()> + Send>>,
}

static TASKS: Mutex<VecDeque<Task>> = Mutex::new(VecDeque::new());

pub fn spawn(future: impl Future<Output = ()> + Send + 'static) {
    let mut tasks = TASKS.lock();

    tasks.push_front(Task {
        future: Box::pin(future),
    });
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

    loop {
        // heapless::
        let mut tasks = TASKS.lock();

        let waker = Waker::noop();
        let mut cx = Context::from_waker(waker);

        if let Some(mut task) = tasks.pop_back() {
            match task.future.as_mut().poll(&mut cx) {
                Poll::Ready(_) => {
                    debug!("Ready");
                    // break;
                }
                Poll::Pending => {
                    // debug!("Not ready");
                    tasks.push_front(task);
                }
            }
        } else {
            debug!("Finish exec");
            break;
        }
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
