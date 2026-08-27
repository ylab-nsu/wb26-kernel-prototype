mod buffer;
mod driver;
mod message;
mod registers;
mod test;
mod uart16550;

pub use driver::get_interrupt_reason_from;
pub use driver::put_into_queue;
pub use driver::UART_DRIVER_QUEUE;
pub use driver::{terminal_task, uart_driver};
pub use message::UartDriverMessage;
pub use registers::{DataBits, Parity, StopBits, TriggerLevel};
pub use test::{start_test, uart16550_rw_test};
pub use uart16550::UART;
