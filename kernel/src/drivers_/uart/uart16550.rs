use core::cell::RefCell;
use core::ptr::{read_volatile, write_volatile};
use core::range;

use crate::threading::scheduler::reschedule;
use critical_section::Mutex;
use heapless::mpmc;
use riscv::_export::critical_section;
use riscv::result;

use super::buffer::{RingBuffer, RingBufferReadError, RingBufferWriteError};
use super::registers::{
    read_reg, write_reg, DataBits, Masks, Parity, Register, StopBits, TriggerLevel, TX_FIFO_SIZE
};

const UART_CLOCK: usize = 3_686_400;

pub struct UART16550<const N: usize> {
    addr: usize,
    rx_buffer: RingBuffer<N>,
    tx_buffer: RingBuffer<N>,
}
pub static mut RX_BUFFER_COUNT: usize = 0;
impl<const N: usize> UART16550<N> {
    pub const fn new(addr: usize) -> Self {
        UART16550 {
            addr: addr,
            rx_buffer: RingBuffer::new(),
            tx_buffer: RingBuffer::new(),
        }
    }

    pub fn init(&self, trigger_level: TriggerLevel) -> () {
        let trigger_mask = match trigger_level {
            TriggerLevel::One => Masks::FCR_RX_TRIGGER_1,
            TriggerLevel::Four => Masks::FCR_RX_TRIGGER_4,
            TriggerLevel::Eight => Masks::FCR_RX_TRIGGER_8,
            TriggerLevel::Fourteen => Masks::FCR_RX_TRIGGER_14,
        };
        write_reg(
            self.addr,
            Register::Fcr,
            Masks::FCR_ENABLE_FIFO 
                | Masks::FCR_RX_CLEAR_FIFO 
                | Masks::FCR_TX_CLEAR_FIFO 
                | trigger_mask,          
        );
    }

	pub fn get_addr(&self) -> usize {
		self.addr
	}

	pub fn enable_loopback(&self) -> () {
		let mcr = read_reg(self.addr, Register::Mcr);
		write_reg(self.addr, Register::Mcr, mcr | Masks::MCR_LOOPBACK);
	}

	pub fn disable_loopback(&self) -> () {
		let mcr = read_reg(self.addr, Register::Mcr);
		write_reg(self.addr, Register::Mcr, mcr & !Masks::MCR_LOOPBACK);
	}

    pub fn enable_rx_interrupt(&self) {
        let ier = read_reg(self.addr, Register::Ier);
        write_reg(self.addr, Register::Ier, ier | Masks::IER_RX_AVAILABLE);
    }

    pub fn disable_rx_interrupt(&self) {
        let ier = read_reg(self.addr, Register::Ier);
        write_reg(self.addr, Register::Ier, ier & !Masks::IER_RX_AVAILABLE);
    }

    pub fn enable_tx_interrupt(&self) {
        let ier = read_reg(self.addr, Register::Ier);
        write_reg(self.addr, Register::Ier, ier | Masks::IER_TX_EMPTY);
    }

    pub fn disable_tx_interrupt(&self) {
        let ier = read_reg(self.addr, Register::Ier);
        write_reg(self.addr, Register::Ier, ier & !Masks::IER_TX_EMPTY);
    }

    pub fn handle_msr(&mut self) -> () {
        read_reg(self.addr, Register::Msr);
    }

    pub fn handle_lsr(&mut self) -> () {
        read_reg(self.addr, Register::Lsr);
    }

    pub fn read(&mut self) -> Option<u8> {
        match self.rx_buffer.read() {
            Ok(byte) => Some(byte),
            Err(RingBufferReadError::Empty) => Option::None,
        }
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), RingBufferWriteError> {
        for &byte in data {
            self.tx_buffer.write(byte)?;
        }

        self.fill_tx_fifo();

        if !self.tx_buffer.is_empty() {
            self.enable_tx_interrupt();
        }

        Ok(())
    }

    pub fn set_baud_rate(&mut self, baud_rate: usize) -> () {
        let divisor = UART_CLOCK / (16 * baud_rate);
        let lcr = read_reg(self.addr, Register::Lcr);
        write_reg(self.addr, Register::Lcr, lcr | Masks::LCR_DLAB);

        let dll = (divisor & 0x00FF) as u8;
        let dlm = ((divisor >> 8) & 0x00FF) as u8;

        write_reg(self.addr, Register::Dll, dll);
        write_reg(self.addr, Register::Dlm, dlm);

        write_reg(self.addr, Register::Lcr, lcr & !Masks::LCR_DLAB);
    }

    pub fn set_line_config(
        &mut self,
        data_bits: DataBits,
        parity: Parity,
		stop_bits: StopBits,
    ) -> () {
		let mut lcr = read_reg(self.addr,Register::Lcr);
		lcr &= !0b11;
		lcr |= match data_bits {
			DataBits::Five => Masks::LCR_DATA_BITS_5,
			DataBits::Six => Masks::LCR_DATA_BITS_6,
			DataBits::Seven => Masks::LCR_DATA_BITS_7,
			DataBits::Eight => Masks::LCR_DATA_BITS_8,
		};

		lcr &= !(1 << 2);

		lcr |= match stop_bits {
			StopBits::One => Masks::LCR_STOP_BITS_1,
			StopBits::Two => Masks::LCR_STOP_BITS_2,
		};

		lcr &= !(0b111 << 3); // + очищаем sp

		lcr |= match parity {
			Parity::None => Masks::LCR_PARITY_NONE,
			Parity::Even =>	Masks::LCR_PARITY_EVEN,
			Parity::Odd =>  Masks::LCR_PARITY_ODD,
		};

		write_reg(self.addr, Register::Lcr, lcr);
    }


	pub fn handle_tx_interrupt(&mut self) {

        // FIFO освободился → снова заполняем его
        self.fill_tx_fifo();

        // Если software buffer полностью пуст,
        // больше TX interrupt нам не нужен.
        if self.tx_buffer.is_empty() {
            self.disable_tx_interrupt();
        }
    }

	
	pub fn handle_rx_interrupt(&mut self, data: &[u8]) -> () {
        for &byte in data {
            if self.rx_buffer.write(byte).is_err() {
                break;
            }

			unsafe {
				RX_BUFFER_COUNT +=1;
			}
        }
    }


	fn fill_tx_fifo(&mut self) {
	
		// Заполняем аппаратный TX FIFO
        for _ in 0..TX_FIFO_SIZE {

			// UART готов принять данные
			let lsr = read_reg(self.addr, Register::Lsr);

			if lsr & Masks::LSR_THR_EMPTY == 0 {
				return;
			}

            // Больше нечего отправлять
            if self.tx_buffer.is_empty() {
                break;
            }
          
            match self.tx_buffer.read() {
                Ok(byte) => {
                    write_reg(
                        self.addr,
                        Register::Thr,
                        byte
                    );
                }

                Err(RingBufferReadError::Empty) => {
                    break;
                }
            }
        }
	}


}
pub static UART: Mutex<RefCell<UART16550<512>>> =
    Mutex::new(RefCell::new(UART16550::new(0x1000_0000)));
