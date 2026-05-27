use crate::threading::scheduler::reschedule;
use heapless::mpmc;

#[allow(deprecated)]
pub static TEST_DRIVER_QUEUE: mpmc::Queue<i32, 16> = mpmc::Queue::new();

pub extern "C" fn driver_task() -> ! {
    loop {
        match TEST_DRIVER_QUEUE.dequeue() {
            None => {
                println!("Driver task yields");
                reschedule();
                println!("Driver back to work");
            }
            Some(number) => {
                println!("Got message to print number {number}");
            }
        }
    }
}
