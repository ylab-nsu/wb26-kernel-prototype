use crate::arch::traits::TargetPlatform;
use crate::arch::Platform;
use crate::threading::scheduler::reschedule;
use heapless::mpmc;
use riscv::_export::critical_section;

pub enum TestDriverMessage {
    PrintNumber { number: usize },
    PrintString { user_addr: usize, len: usize },
}

#[allow(deprecated)]
pub static TEST_DRIVER_QUEUE: mpmc::Queue<TestDriverMessage, 32> = mpmc::Queue::new();
pub static mut TEST_DRIVER_ANY: bool = false;

pub fn put_into_queue(
    message: TestDriverMessage,
    queue: &mpmc::QueueView<TestDriverMessage>,
) {
    let mut message = message;
    loop {
        match critical_section::with(|_| {
            let mut res = queue.enqueue(message);
            if let Err(v) = res {
                res = queue.enqueue(v);
            }
            res
        }) {
            Ok(_) => break,
            Err(v) => {
                message = v;
                println!("Cannot put element into queue, reschedule");
                reschedule();
            }
        }
    }
    unsafe { TEST_DRIVER_ANY = true };
}

pub extern "C" fn driver_task() -> ! {
    loop {
        let message = critical_section::with(|_| TEST_DRIVER_QUEUE.dequeue());
        match message {
            None => {
                println!("Driver task yields");
                unsafe {
                    TEST_DRIVER_ANY = false;
                }
                reschedule();
                println!("Driver back to work");
            }
            Some(TestDriverMessage::PrintNumber { number }) => println!("PrintNumber: {number}"),
            Some(TestDriverMessage::PrintString { user_addr, len }) => unsafe {
                // user_addr.wrapping_sub(kernel_layout.user_va_offset) as *const u8
                let offset = Platform::get_user_va_offset();
                let addr = user_addr.wrapping_sub(offset);
                let ptr = addr as *const u8;
                let s = core::slice::from_raw_parts(ptr, len);
                println!("PrintString:");
                print!("{}", core::str::from_utf8_unchecked(s));
            },
        }
    }
}
