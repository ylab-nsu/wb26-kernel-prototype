use core::ptr::{read_volatile, write_volatile};

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Register {
    Rbr,
    Thr,
    Dll,
    Ier,
    Dlm,
    Iir,
    Fcr,
    Lcr,
    Mcr,
    Lsr,
    Msr,
    Scr,
}

pub enum TriggerLevel{
	One,
	Four,
	Eight,
	Fourteen,
}

pub enum DataBits {
    Five,
    Six,
    Seven,
    Eight,
}

pub enum StopBits {
    One,
    Two,
}

pub enum Parity {
    None,
    Odd,
    Even,
}

impl Register {
    pub fn offset(self) -> usize {
        match self {
            Register::Rbr => 0,
            Register::Dll => 0,
            Register::Thr => 0,
            Register::Ier => 1,
            Register::Dlm => 1,
            Register::Iir => 2,
            Register::Fcr => 2,
            Register::Lcr => 3,
            Register::Mcr => 4,
            Register::Lsr => 5,
            Register::Msr => 6,
            Register::Scr => 7,
        }
    }
}

pub fn read_reg(addr: usize, reg: Register) -> u8 {
    unsafe {
        let reg_ptr = (addr + reg.offset()) as *mut u8;
        read_volatile(reg_ptr)
    }
}

pub fn write_reg(addr: usize, reg: Register, value: u8) {
    unsafe {
        let reg_ptr = (addr + reg.offset()) as *mut u8;
        write_volatile(reg_ptr, value);
    }
}

pub struct Masks;
pub const TX_FIFO_SIZE: usize = 16;

impl Masks {
    // LSR bits
    pub const LSR_DATA_READY: u8 = 1 << 0;
    pub const LSR_THR_EMPTY: u8 = 1 << 5;
	pub const LSR_OVERRUN_ERROR: u8 = 1 << 1;


    // IER bits
    pub const IER_RX_AVAILABLE: u8 = 1 << 0;
    pub const IER_TX_EMPTY: u8 = 1 << 1;

    // IIR bits
    pub const IIR_REASON: u8 = 0b111 << 1;
    pub const IIR_NO_INTERRUPT: u8 = 0b001 << 0;

    pub const IIR_MODEM_STATUS: u8 = 0b000 << 1;
    pub const IIR_THR_EMPTY: u8 = 0b001 << 1;
    pub const IIR_RECEIVED_DATA_AVAILABLE: u8 = 0b010 << 1;
    pub const IIR_LINE_STATUS: u8 = 0b011 << 1;
    pub const IIR_CHARACTER_TIMEOUT: u8 = 0b110 << 1;

    // FCR bits
    pub const FCR_ENABLE_FIFO: u8 = 1 << 0;
    pub const FCR_RX_CLEAR_FIFO: u8 = 1 << 1;
    pub const FCR_TX_CLEAR_FIFO: u8 = 1 << 2;
	pub const FCR_RX_TRIGGER_1: u8 = 0b00 << 6;
	pub const FCR_RX_TRIGGER_4: u8 = 0b01 << 6;
	pub const FCR_RX_TRIGGER_8: u8 = 0b10 << 6;
	pub const FCR_RX_TRIGGER_14: u8 = 0b11 << 6;

	// LCR bits
	pub const LCR_DLAB: u8 = 1 << 7;
	pub const LCR_DATA_BITS_5: u8 = 0b00;
	pub const LCR_DATA_BITS_6: u8 = 0b01;
	pub const LCR_DATA_BITS_7: u8 = 0b10;
	pub const LCR_DATA_BITS_8: u8 = 0b11;
	pub const LCR_STOP_BITS_1: u8 = 0;
	pub const LCR_STOP_BITS_2: u8 = 1 << 2;
	pub const LCR_PARITY_NONE: u8 = 0;
	pub const LCR_PARITY_EVEN: u8 = 0b11 << 3;
	pub const LCR_PARITY_ODD:  u8 = 0b01 << 3;



	// MCR bits
	pub const MCR_LOOPBACK: u8 = 1 << 4;

}
