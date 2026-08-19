use crate::arch::traits::TargetPlatform;
use crate::arch::Platform;
use crate::threading::scheduler::reschedule;
use crate::uart2::UART;
use heapless::mpmc;
use riscv::_export::critical_section;


pub enum UartDriverMessage {
    Read {
        addr: usize,
    },
    Write {
        addr: usize,
    },
    Flush,
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

	let mut uart = UART::new(0x10000000);
	uart.putc('A' as u8);
	loop {
		let c = uart.getc();
		uart.putc(c);
	}

    loop {
        let message: Option<UartDriverMessage> = critical_section::with(|_| UART_DRIVER_QUEUE.dequeue());
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
