use heapless::Vec;

use crate::drivers_::uart::uart16550::RX_BUFFER_COUNT;
use crate::threading::scheduler::reschedule;
use heapless::mpmc;
use riscv::_export::critical_section;

use super::message::{UartDriverMessage, MAX_RECEVIED_BYTES};
use super::uart16550::UART;
use super::registers::{read_reg, Register, Masks};
use super::test::{read_cycle, read_instret};

static mut RBR_COUNT: usize = 0;
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
				unsafe {
					RBR_COUNT+=1;
				}
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

pub fn put_into_queue(
    message: UartDriverMessage,
    queue: &mpmc::QueueView<UartDriverMessage>,
) -> Result<(), UartDriverMessage> {
    critical_section::with(|_| {
        queue.enqueue(message)
    })
}

static mut MESSAGE_COUNT: usize = 0;

pub extern "C" fn uart_driver() -> ! {
    loop {
        let message: Option<UartDriverMessage> =
            critical_section::with(|_| UART_DRIVER_QUEUE.dequeue());
        match message {
			
            Some(UartDriverMessage::Receive { data }) => {
				unsafe {
					MESSAGE_COUNT += data.len();
				}
				critical_section::with(|cs|{
					//println!("RX interrupt");
					let mut uart = UART.borrow(cs).borrow_mut();
					
					//let start = read_instret();

					//let end = read_instret();

					//println!("empty: {} instr", end - start);

				
					//let start =  read_instret();
					uart.handle_rx_interrupt(&data);
					//let end = read_instret();
					
					//println!("handle_rx_interrupt: {} instr", end - start);
				});
            }
            Some(UartDriverMessage::Send) => {
				critical_section::with(|cs|{
					//println!("TX interrupt");
					let mut uart = UART.borrow(cs).borrow_mut();
					//let start =  read_cycle();
					uart.handle_tx_interrupt();
					//let end = read_cycle();
					//println!("handle_tx_interrupt: {} cycles", end - start);
				});
            }
            Some(UartDriverMessage::LineStatus) => {println!("PANIC!");}
            Some(UartDriverMessage::ModemStatus) => {}
            None => {
		
                reschedule();
                //info!("UART driver back to work");
            }
        }
    }
}

static mut TERMINAL_COUNT: usize = 0;
pub extern "C" fn terminal_task() -> ! {
    let mut buffer = [0u8; 512];
    let mut len = 0;

    loop {
        let byte = critical_section::with(|cs| {
            UART.borrow(cs).borrow_mut().read()
        });

		if byte.is_some() {
			unsafe {
				TERMINAL_COUNT += 1;
			}
		}

        match byte {
            Some(b'\n') | Some(b'\r') => {
                if len > 0 {
				// 	println!(
				// 	"RBR={} MESSAGE={} RX_BUFFER={} TERMINAL={}",
				// 	unsafe { RBR_COUNT },
				// 	unsafe { MESSAGE_COUNT },
				// 	unsafe { RX_BUFFER_COUNT },
				// 	unsafe { TERMINAL_COUNT },
				// );
                    critical_section::with(|cs| {
                        let mut uart = UART.borrow(cs).borrow_mut();

                        let _ = uart.write(&buffer[..len]);
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