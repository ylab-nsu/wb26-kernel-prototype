mod buffer;
mod driver;
mod message;
mod registers;
mod uart16550;

pub use uart16550::UART;
pub use driver::{uart_driver, terminal_task};
pub use driver::put_into_queue;
pub use driver::UART_DRIVER_QUEUE;
pub use driver::get_interrupt_reason_from;
pub use message::UartDriverMessage;