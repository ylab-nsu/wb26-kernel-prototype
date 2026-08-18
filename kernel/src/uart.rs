use crate::arch::traits::TargetPlatform;
use crate::arch::Platform;
use crate::threading::scheduler::reschedule;
use heapless::mpmc;
use riscv::_export::critical_section;

pub enum UartDriverMessage {
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
pub static UART_DRIVER_QUEUE: mpmc::Queue<UartDriverMessage, 32> = mpmc::Queue::new();

pub fn put_into_queue(message: UartDriverMessage, queue: &mpmc::QueueView<UartDriverMessage>) {
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
}

pub extern "C" fn uart_driver() -> ! {
    loop {
        let message = critical_section::with(|_| UART_DRIVER_QUEUE.dequeue());
        match message {
            None => {
                info!("UART driver yields");
                reschedule();
                info!("UART driver back to work");
            }
            _ => {}
        }
    }
}
