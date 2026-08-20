use core::fmt::Error;
use core::ptr::{addr_of_mut, read_volatile, write_volatile};

use heapless::Deque;
use heapless::HistoryBuf;
use spin::mutex;

use crate::sync::Mutex;

pub enum RingBufferWriteError{
	Overflow,
}

pub enum RingBufferReadError{
	Empty,
}

struct RingBuffer<const N: usize>{
	buffer: [u8; N],
	read_idx: usize,
	write_idx: usize,
	len: usize,
}

impl<const N: usize> RingBuffer<N> {
	pub const fn new() -> Self{
		RingBuffer { 
			buffer: [0; N], 
			read_idx: 0, 
			write_idx: 0,
			len: 0,
		}
	}

	pub fn write(&mut self, byte: u8) -> Result<(), RingBufferWriteError> {
		if self.len == N {
			Err(RingBufferWriteError::Overflow)
		}
		else {
			if self.write_idx == N {
			self.write_idx = 0;
			}
			self.buffer[self.write_idx] = byte; 
			self.write_idx += 1;
			self.len += 1;
			Ok(())
		}
	}

	pub fn read(&mut self) -> Result<u8, RingBufferReadError>{
		if self.len == 0 {
			Err(RingBufferReadError::Empty)
		}else{
			if self.read_idx == N {
				self.read_idx = 0;
			}
			let byte =self.buffer[self.read_idx];
			self.read_idx += 1;
			self.len -= 1;
			Ok(byte)
		}

	}
}

struct Flags;
const BUFFER_SIZE: usize = 1024;

impl Flags {
	// LSR bits
	const LSR_DATA_READY: 	u8 = 1 << 0;
	const LSR_THR_EMPTY: 	u8 = 1 << 5;

	// IER bits
	const IER_RX_AVAILABLE: u8 = 1 << 0;
	const IER_TX_EMPTY: 	u8 = 1 << 1;

	// IIR bits
	const IIR_REASON: 						u8 = 0b111 << 1;
	const IIR_NO_INTERRUPT:					u8 = 0b001 << 0;
	const IIR_LINE_STATUS: 					u8 = 0b011 << 1;
	const IIR_RECEIVED_DATA_AVAILABLE:		u8 = 0b010 << 1;
	const IIR_CHARACTER_TIMEOUT: 			u8 = 0b110 << 1;
	const IIR_THR_EMPTY: 					u8 = 0b001 << 1;
	const IIR_MODEM_STATUS: 				u8 = 0b000 << 1;

	// FCR bits
	const FCR_ENABLE_FIFO: 		u8 = 1 << 0;
	const FCR_RX_CLEAR_FIFO: 	u8 = 1 << 1;
	const FCR_TX_CLEAR_FIFO: 	u8 = 1 << 2;
}

pub struct UART{
	regs: usize,
	rx_buff: RingBuffer<BUFFER_SIZE>,
	tx_buff:  RingBuffer<BUFFER_SIZE>,
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
	pub const fn new(addr: usize) -> Self {
		UART { 
			regs: addr,
			rx_buff: RingBuffer::new(),
			tx_buff: RingBuffer::new(),
		}
	}

	const fn get_regs_ptr(&self) -> *mut Registers{
		self.regs as *mut Registers
	}

	pub fn init(&self) {
		// Configuring FIFO
		unsafe {
			write_volatile(
				addr_of_mut!((*self.get_regs_ptr()).iir_fcr),
				Flags::FCR_ENABLE_FIFO | Flags::FCR_RX_CLEAR_FIFO | Flags::FCR_TX_CLEAR_FIFO
			);
		}
	}

    /// Отправляет один байт
    pub fn write(&mut self, data: &[u8]) ->Result<(), RingBufferWriteError> {
		for &byte in data{
			self.tx_buff.write(byte)?;
		}
		self.enable_tx_interrupt();
		Ok(())
    }

	// Читает один байт
	pub fn read(&mut self) -> Result<u8, RingBufferReadError> {
		self.rx_buff.read()
	}

	pub fn enable_rx_interrupt(&self) {
		unsafe {
			let ier = read_volatile(addr_of_mut!((*self.get_regs_ptr()).ier));
			write_volatile(
				addr_of_mut!((*self.get_regs_ptr()).ier), 
				ier | Flags::IER_RX_AVAILABLE,
			);
		}
	}

	pub fn disable_rx_interrupt(&self) {
		unsafe {
			let ier = read_volatile(addr_of_mut!((*self.get_regs_ptr()).ier));
			write_volatile(
				addr_of_mut!((*self.get_regs_ptr()).ier), 
				ier & !Flags::IER_RX_AVAILABLE,
			);
		}
	}

	pub fn enable_tx_interrupt(&self){
		unsafe {
			let ier = read_volatile(addr_of_mut!((*self.get_regs_ptr()).ier));
			write_volatile(
				addr_of_mut!((*self.get_regs_ptr()).ier),
				ier | Flags::IER_TX_EMPTY,	 
			);
		}
	}

	pub fn disable_tx_interrupt(&self){
		unsafe {
			let ier = read_volatile(addr_of_mut!((*self.get_regs_ptr()).ier));
			write_volatile(
				addr_of_mut!((*self.get_regs_ptr()).ier),
				ier & !Flags::IER_TX_EMPTY,	 
			);
		}
	}

    fn lsr(&self) -> u8 {
        unsafe {
         	read_volatile(&(*self.get_regs_ptr()).lsr)
        }
    }
	
	fn iir(&self) -> u8{
		unsafe {
			read_volatile(addr_of_mut!((*self.get_regs_ptr()).iir_fcr))
		}
	}

	fn msr(&self) -> u8{
		unsafe {
			read_volatile(addr_of_mut!((*self.get_regs_ptr()).msr))
		}
	}

	fn read_rbr(&self) -> u8 {
		unsafe{
			read_volatile(addr_of_mut!((*self.get_regs_ptr()).thr_rbr))
		}
	}

	fn write_thr(&self, byte: u8) -> (){
		unsafe {
			write_volatile(addr_of_mut!((*self.get_regs_ptr()).thr_rbr), byte);
		}
	}

	pub fn handle_interrupt(&mut self) -> (){
		loop {
			let iir = self.iir(); 
			if iir & Flags::IIR_NO_INTERRUPT != 0 {
				break;
			}

			let reason = iir & Flags::IIR_REASON;
			match reason {
				Flags::IIR_RECEIVED_DATA_AVAILABLE =>{
					while self.lsr() & Flags::LSR_DATA_READY != 0 {
						let byte = self.read_rbr();
						match self.rx_buff.write(byte) {
							Ok(())=>{}
							Err(RingBufferWriteError::Overflow) => {}							
						}
					}
				}

				Flags::IIR_CHARACTER_TIMEOUT => {
					while self.lsr() & Flags::LSR_DATA_READY != 0 {
							let byte = self.read_rbr();

							match self.rx_buff.write(byte) {
								Ok(()) => {}

								Err(RingBufferWriteError::Overflow) => {}
							}
						}
				}

				Flags::IIR_THR_EMPTY => {
					while self.lsr() & Flags::LSR_THR_EMPTY != 0 {
						match self.tx_buff.read() {
							Ok(byte)=>{
								self.write_thr(byte);
							}
							Err(RingBufferReadError::Empty) => {
								self.disable_tx_interrupt();
								break;
							}
						}
					}
					
				}

				Flags::IIR_LINE_STATUS => {
					let lsr = self.lsr();
				}
			
				Flags::IIR_MODEM_STATUS => {
					let mst = self.msr();
				}

				_ => {}

			}
		}
	}
		
}


pub static UART : Mutex<UART> = Mutex::new(UART::new(0x10000000));

pub fn init_uart() -> (){
	let uart = UART.lock();
	uart.init();
	uart.enable_rx_interrupt();
}

pub fn uart_write(data: &[u8]) -> Result<(), RingBufferWriteError> {
	let mut uart = UART.lock();
	uart.write(data)
}

pub fn uart_read() -> Option<u8>{
	let mut uart = UART.lock();
	let result =uart.read(); 
	match result {
		Ok(byte)=>{
			Option::Some(byte)
		}
		Err(RingBufferReadError::Empty) => {Option::None}
	} 
}