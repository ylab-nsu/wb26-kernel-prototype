use heapless::Vec;

use crate::threading::scheduler::reschedule;
use heapless::mpmc;
use riscv::_export::critical_section;

use super::message::{UartDriverMessage, MAX_RECEVIED_BYTES};
use super::uart16550::UART;
use super::registers::{read_reg, Register, Masks};

pub fn get_interrupt_reason_from(addr:usize) -> Option<UartDriverMessage>{
	let iir = read_reg(addr, Register::Iir);

    if iir & Masks::IIR_NO_INTERRUPT != 0 {
        return Option::None;
    }

    let reason = iir & Masks::IIR_REASON;
    match reason {
        Masks::IIR_RECEIVED_DATA_AVAILABLE | Masks::IIR_CHARACTER_TIMEOUT => {
            let mut buffer: Vec<u8, MAX_RECEVIED_BYTES> = Vec::new();
            while read_reg(addr, Register::Lsr) & Masks::LSR_DATA_READY != 0 {
                let byte = read_reg(addr, Register::Rbr);
				if buffer.push(byte).is_err() {
					break;
				}
			}
            Some(UartDriverMessage::Receive { data: buffer})
        }
        Masks::IIR_THR_EMPTY => Some(UartDriverMessage::Send),
        Masks::IIR_LINE_STATUS => Some(UartDriverMessage::LineStatus),
        Masks::IIR_MODEM_STATUS => Some(UartDriverMessage::ModemStatus),
        _ => Option::None,
    }
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
                //info!("Cannot put element into queue, park current thread and reschedule");
                reschedule();
            }
        }
    }
}

// TO DO СДЕЛАТЬ ГЛОБАЛЬНУЮ ОЧЕРЕДЬ
pub extern "C" fn uart_driver() -> ! {
    loop {
        let message: Option<UartDriverMessage> =
            critical_section::with(|_| UART_DRIVER_QUEUE.dequeue());
        match message {
			
            Some(UartDriverMessage::Receive { data }) => {
				println!("RX IRQ!");
				critical_section::with(|cs|{
					let mut uart = UART.borrow(cs).borrow_mut();
					uart.handle_rx_interrupt(&data);
				});
            }
            Some(UartDriverMessage::Send) => {
				println!("TX IRQ!");
				critical_section::with(|cs|{
					let mut uart = UART.borrow(cs).borrow_mut();
					uart.handle_tx_interrupt();
				});
            }
            Some(UartDriverMessage::LineStatus) => {}
            Some(UartDriverMessage::ModemStatus) => {}
            None => {
                //info!("UART driver yields");
                reschedule();
                //info!("UART driver back to work");
            }
        }
    }
}

pub extern "C" fn terminal_task() -> ! {
    let mut buffer = [0u8; 256];
    let mut len = 0;

    loop {
        let byte = critical_section::with(|cs| {
            UART.borrow(cs).borrow_mut().read()
        });

        match byte {
            Some(b'\n') | Some(b'\r') => {
                if len > 0 {
                    critical_section::with(|cs| {
                        let mut uart = UART.borrow(cs).borrow_mut();

                        let _ = uart.write(&buffer[..len]);
                        let _ = uart.write(b"\r\n");
                    });

                    len = 0;
                }
            }

            Some(byte) => {
                if len < buffer.len() {
                    buffer[len] = byte;
                    len += 1;
                }
            }

            None => {
                reschedule();
            }
        }
    }
}