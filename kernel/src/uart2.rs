use core::ptr::{addr_of_mut, read_volatile, write_volatile};

use heapless::spsc;


const BUFFER_SIZE: usize = 1024;
// LSR bits
const LSR_DATA_READY: u8 = 1 << 0;
const LSR_THR_EMPTY: u8 = 1 << 5;

// IER bits
const IER_RX_AVAILABLE: u8 = 1 << 0;
const IER_TX_EMPTY: u8 = 1 << 1;

// IIR
const IIR_NO_INTERRUPT: u8 = 1 << 0;
const IIR_TX_EMPTY: u8 = 0b001 << 1;
const IIR_RX_AVAILABLE: u8 = 0b010 << 1;


pub struct UART{
	regs: *mut Registers,
	//rx_buff: spsc::Queue<u8, BUFFER_SIZE>,
	//tx_buff: spsc::Queue<u8, BUFFER_SIZE>,
}
   
#[repr(C)]
pub struct Registers{
 	thr_rbr: u8, // +0x00
    ier: u8,     // +0x01
    iir_fcr: u8, // +0x02
    lcr: u8,     // +0x03
    mcr: u8,     // +0x04
    lsr: u8,     // +0x05
    msr: u8,     // +0x06
    scr: u8,     // +0x07
}

impl UART {
	pub fn new(addr: usize) -> Self {
		UART { regs: addr as *mut Registers}
	}

    /// Отправляет один байт
    pub fn putc(&self, byte: u8) {
        while self.lsr() & LSR_THR_EMPTY == 0 {}

        // Пишем байт в THR
        unsafe {
            write_volatile(addr_of_mut!((*self.regs).thr_rbr), byte);
        }
    }

	pub fn getc(&self) -> u8 {
		while self.lsr() & LSR_DATA_READY == 0 {} // Waiting 
		unsafe  {read_volatile(addr_of_mut!((*self.regs).thr_rbr)) }
	}

    fn lsr(&self) -> u8 {
        unsafe {
            core::ptr::read_volatile(&(*self.regs).lsr)
        }
    }
}