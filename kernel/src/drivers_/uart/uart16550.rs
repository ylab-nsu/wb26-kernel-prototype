use core::cell::RefCell;
use core::ptr::{read_volatile, write_volatile};
use core::range;

use crate::threading::scheduler::reschedule;
use critical_section::Mutex;
use heapless::mpmc;
use riscv::_export::critical_section;
use riscv::result;

use super::buffer::{RingBuffer, RingBufferReadError, RingBufferWriteError};
use super::registers::{read_reg, write_reg, Masks, Register};

pub struct UART16550<const N: usize> {
    addr: usize,
    rx_buffer: RingBuffer<N>,
    tx_buffer: RingBuffer<N>,
}

impl<const N: usize> UART16550<N> {
    pub const fn new(addr: usize) -> Self {
        UART16550 {
            addr: addr,
            rx_buffer: RingBuffer::new(),
            tx_buffer: RingBuffer::new(),
        }
    }

    pub fn init(&self) -> () {
        write_reg(
            self.addr,
            Register::Fcr,
            Masks::FCR_ENABLE_FIFO | Masks::FCR_RX_CLEAR_FIFO | Masks::FCR_TX_CLEAR_FIFO,
        );
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

    pub fn send_data(&mut self) -> () {
        while read_reg(self.addr, Register::Lsr) & Masks::LSR_THR_EMPTY != 0 {
            let result = self.tx_buffer.read();
            match result {
                Ok(byte) => write_reg(self.addr, Register::Thr, byte),
                Err(RingBufferReadError::Empty) => {
                    self.disable_tx_interrupt();
                    break;
                }
            }
        }
    }

    pub fn receive_data(&mut self, data: &[u8]) -> () {
        for &byte in data {
            if self.rx_buffer.write(byte).is_err() {
                break;
            }
        }
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
        self.send_data();

        if !self.tx_buffer.is_empty() {
            self.enable_tx_interrupt();
        }

        Ok(())
    }
}
pub static UART: Mutex<RefCell<UART16550<256>>> =
    Mutex::new(RefCell::new(UART16550::new(0x1000_0000)));
