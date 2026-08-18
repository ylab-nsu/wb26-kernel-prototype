use crate::arch::traits::TargetPlatform;
use crate::arch::Platform;
use crate::threading::scheduler::reschedule;
use heapless::mpmc;
use riscv::_export::critical_section;

pub enum TestDriverMessage {
    PrintNumber {
        number: usize,
    },
    PrintString {
        user_addr: usize,
        len: usize,
    },
    WriteScull {
        user_src: usize,
        scull_dst: usize,
        count: usize,
    },
    ReadScull {
        user_dst: usize,
        scull_src: usize,
        count: usize,
    },
}

#[allow(deprecated)]
pub static TEST_DRIVER_QUEUE: mpmc::Queue<TestDriverMessage, 32> = mpmc::Queue::new();
pub static mut TEST_DRIVER_ANY: bool = false;

static mut SCULL_AREA: [u8; 1024] = [0; _];

pub fn put_into_queue(message: TestDriverMessage, queue: &mpmc::QueueView<TestDriverMessage>) {
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
                info!("Cannot put element into queue, park current thread and reschedule");
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
                info!("Driver task yields");
                unsafe {
                    TEST_DRIVER_ANY = false;
                }
                reschedule();
                info!("Driver back to work");
            }
            Some(TestDriverMessage::PrintNumber { number }) => info!("PrintNumber: {}", number),
            Some(TestDriverMessage::PrintString { user_addr, len }) => unsafe {
                let ptr = user_addr.wrapping_sub(Platform::get_user_va_offset()) as *const u8;
                let s = core::slice::from_raw_parts(ptr, len);
                info!("PrintString");
                print!("`");
                print!("{}", core::str::from_utf8_unchecked(s));
                println!("`");
            },
            Some(TestDriverMessage::WriteScull {
                user_src,
                scull_dst,
                count: len,
            }) => unsafe {
                info!("WriteScull something");
                assert!(scull_dst + len <= SCULL_AREA.len());
                let src = user_src.wrapping_sub(Platform::get_user_va_offset()) as *const u8;
                let dst = SCULL_AREA.as_mut_ptr().add(scull_dst);
                core::ptr::copy(src, dst, len);
            },
            Some(TestDriverMessage::ReadScull {
                user_dst,
                scull_src,
                count: len,
            }) => unsafe {
                info!("ReadScull something");
                assert!(scull_src + len <= SCULL_AREA.len());
                let dst = user_dst.wrapping_sub(Platform::get_user_va_offset()) as *mut u8;
                let src = SCULL_AREA.as_ptr().add(scull_src);
                core::ptr::copy(src, dst, len);
            },
        }
    }
}
