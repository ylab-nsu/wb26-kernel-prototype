use core::cell::RefCell;
use core::fmt::Write;
use core::ptr::{read_volatile, write_volatile};
use core::range;

use crate::drivers_::uart::buffer::RingBuffer;
use crate::drivers_::uart::uart16550::UART16550;
use crate::drivers_::uart::{self, DataBits, Parity, StopBits, TriggerLevel, UART};
use crate::threading::scheduler::reschedule;
use alloc::format;
use critical_section::Mutex;
use heapless::{mpmc, String};
use riscv::_export::critical_section;
use riscv::result;

use super::buffer::{RingBufferReadError, RingBufferWriteError};
use super::registers::{read_reg, write_reg, Masks, Register};

const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const BLUE: &str = "\x1b[34m";
const RESET: &str = "\x1b[0m";

enum TestType {
    Passed,
    Failed,
}

fn form(message: &str, result: TestType) -> String<128> {
    let mut output = String::<128>::new();

    match result {
        TestType::Failed => {
            write!(&mut output, "{BLUE}{message}{RESET}: {RED}failed{RESET}").unwrap();
        }

        TestType::Passed => {
            write!(&mut output, "{BLUE}{message}{RESET}: {GREEN}passed{RESET}").unwrap();
        }
    }

    output
}

fn ringbuffer_rw_test() {
    let mut rb: RingBuffer<10> = RingBuffer::new();
    assert!(
        rb.write(b'a').is_ok(),
        "{}",
        form("ringbuffer_rw_test", TestType::Failed)
    );

    let result: Result<u8, RingBufferReadError> = rb.read();
    assert!(
        result.is_ok(),
        "{}",
        form("ringbuffer_rw_test", TestType::Failed)
    );

    assert!(
        result.unwrap() == b'a',
        "{}",
        form("ringbuffer_rw_test", TestType::Failed)
    );

    println!("{}", form("ringbuffer_rw_test", TestType::Passed));
}

fn ringbuffer_empty_test() {
    let mut rb: RingBuffer<10> = RingBuffer::new();
    let res = rb.read();
    assert!(
        rb.read().is_ok() == false,
        "{}",
        form("ringbuffer_empty_test", TestType::Failed)
    );
    println!("{}", form("ringbuffer_empty_test", TestType::Passed));
}

fn ringbuffer_full_test() {
    let mut rb: RingBuffer<1> = RingBuffer::new();
    if rb.write(b'a').is_err() {}
    assert!(
        rb.write(b'a').is_ok() == false,
        "{}",
        form("ringbuffer_full_test", TestType::Failed)
    );
    println!("{}", form("ringbuffer_full_test", TestType::Passed));
}

fn ringbuffer_fifo_order_test() {
    let mut rb: RingBuffer<10> = RingBuffer::new();
    rb.write(b'a');
    rb.write(b'b');
    rb.write(b'c');
    assert!(
        rb.read().unwrap() == b'a',
        "{}",
        form("ringbuffer_fifo_order_test", TestType::Failed)
    );
    assert!(
        rb.read().unwrap() == b'b',
        "{}",
        form("ringbuffer_fifo_order_test", TestType::Failed)
    );
    assert!(
        rb.read().unwrap() == b'c',
        "{}",
        form("ringbuffer_fifo_order_test", TestType::Failed)
    );
    println!("{}", form("ringbuffer_fifo_order_test", TestType::Passed));
}

fn registers_offset_test() {
    assert!(
        Register::Rbr.offset() == 0,
        "{}",
        form("registers_offset_test", TestType::Failed)
    );
    assert!(
        Register::Thr.offset() == 0,
        "{}",
        form("registers_offset_test", TestType::Failed)
    );
    assert!(
        Register::Dll.offset() == 0,
        "{}",
        form("registers_offset_test", TestType::Failed)
    );
    assert!(
        Register::Ier.offset() == 1,
        "{}",
        form("registers_offset_test", TestType::Failed)
    );
    assert!(
        Register::Dlm.offset() == 1,
        "{}",
        form("registers_offset_test", TestType::Failed)
    );
    assert!(
        Register::Iir.offset() == 2,
        "{}",
        form("registers_offset_test", TestType::Failed)
    );
    assert!(
        Register::Fcr.offset() == 2,
        "{}",
        form("registers_offset_test", TestType::Failed)
    );
    assert!(
        Register::Lcr.offset() == 3,
        "{}",
        form("registers_offset_test", TestType::Failed)
    );
    assert!(
        Register::Mcr.offset() == 4,
        "{}",
        form("registers_offset_test", TestType::Failed)
    );
    assert!(
        Register::Lsr.offset() == 5,
        "{}",
        form("registers_offset_test", TestType::Failed)
    );
    assert!(
        Register::Msr.offset() == 6,
        "{}",
        form("registers_offset_test", TestType::Failed)
    );
    assert!(
        Register::Scr.offset() == 7,
        "{}",
        form("registers_offset_test", TestType::Failed)
    );
    println!("{}", form("registers_offset_test", TestType::Passed));
}

fn registers_masks_test() {
    assert!(
        Masks::LSR_DATA_READY == 0b0000_0001,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::LSR_THR_EMPTY == 0b0010_0000,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::IER_RX_AVAILABLE == 0b0000_0001,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::IER_TX_EMPTY == 0b0000_0010,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::IIR_REASON == 0b0000_1110,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::IIR_NO_INTERRUPT == 0b0000_0001,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::IIR_MODEM_STATUS == 0b0000_0000,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::IIR_THR_EMPTY == 0b0000_0010,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::IIR_RECEIVED_DATA_AVAILABLE == 0b0000_0100,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::IIR_LINE_STATUS == 0b0000_0110,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::IIR_CHARACTER_TIMEOUT == 0b0000_1100,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::FCR_ENABLE_FIFO == 0b0000_0001,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::FCR_RX_CLEAR_FIFO == 0b0000_0010,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::FCR_TX_CLEAR_FIFO == 0b0000_0100,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::FCR_RX_TRIGGER_1 == 0b0000_0000,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::FCR_RX_TRIGGER_4 == 0b0100_0000,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::FCR_RX_TRIGGER_8 == 0b1000_0000,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::FCR_RX_TRIGGER_14 == 0b1100_0000,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::LCR_DLAB == 0b1000_0000,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::LCR_DATA_BITS_5 == 0b0000_0000,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::LCR_DATA_BITS_6 == 0b0000_0001,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::LCR_DATA_BITS_7 == 0b0000_0010,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::LCR_DATA_BITS_8 == 0b0000_0011,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::LCR_STOP_BITS_1 == 0b0000_0000,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::LCR_STOP_BITS_2 == 0b0000_0100,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::LCR_PARITY_NONE == 0b0000_0000,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::LCR_PARITY_EVEN == 0b0001_1000,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    assert!(
        Masks::LCR_PARITY_ODD == 0b0000_1000,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );
    assert!(
        Masks::MCR_LOOPBACK == 0b0001_0000,
        "{}",
        form("registers_masks_test", TestType::Failed)
    );

    println!("{}", form("registers_masks_test", TestType::Passed));
}

fn registers_rw_test() {
    const ADDR: usize = 0x1000_0000;

    write_reg(ADDR, Register::Ier, 0x03);
    let value = read_reg(ADDR, Register::Ier);

    assert_eq!(
        value,
        0x03,
        "{}",
        form("registers_rw_test", TestType::Failed)
    );

    write_reg(ADDR, Register::Ier, 0x00);
    let value = read_reg(ADDR, Register::Ier);

    assert_eq!(
        value,
        0x00,
        "{}",
        form("registers_rw_test", TestType::Failed)
    );

    write_reg(ADDR, Register::Lcr, 0x00);
    let value = read_reg(ADDR, Register::Lcr);

    assert_eq!(
        value,
        0x00,
        "{}",
        form("registers_rw_test", TestType::Failed)
    );

    write_reg(ADDR, Register::Lcr, 0x03);
    let value = read_reg(ADDR, Register::Lcr);

    assert_eq!(
        value,
        0x03,
        "{}",
        form("registers_rw_test", TestType::Failed)
    );

    write_reg(ADDR, Register::Mcr, 0x00);
    let value = read_reg(ADDR, Register::Mcr);

    assert_eq!(
        value,
        0x00,
        "{}",
        form("registers_rw_test", TestType::Failed)
    );

    write_reg(ADDR, Register::Mcr, 0x01);
    let value = read_reg(ADDR, Register::Mcr);

    assert_eq!(
        value,
        0x01,
        "{}",
        form("registers_rw_test", TestType::Failed)
    );

    write_reg(ADDR, Register::Scr, 0x00);
    let value = read_reg(ADDR, Register::Scr);

    assert_eq!(
        value,
        0x00,
        "{}",
        form("registers_rw_test", TestType::Failed)
    );

    write_reg(ADDR, Register::Scr, 0xAA);
    let value = read_reg(ADDR, Register::Scr);

    assert_eq!(
        value,
        0xAA,
        "{}",
        form("registers_rw_test", TestType::Failed)
    );

    write_reg(ADDR, Register::Scr, 0x55);
    let value = read_reg(ADDR, Register::Scr);

    assert_eq!(
        value,
        0x55,
        "{}",
        form("registers_rw_test", TestType::Failed)
    );

    println!("{}", form("registers_rw_test", TestType::Passed));
}

fn uart16550_new_test() {
    let uart = UART16550::<256>::new(0x1000_0000);
    assert!(
        uart.get_addr() == 0x1000_0000,
        "{}",
        form("uart16550_new_test", TestType::Failed)
    );

    println!("{}", form("uart16550_new_test", TestType::Passed));
}

fn uart16550_init_test() {
    const ADDR: usize = 0x1000_0000;
    let uart = UART16550::<256>::new(ADDR);
    uart.init(TriggerLevel::Four);
    println!("{}", form("uart16550_init_test", TestType::Passed));
}

fn uart16550_loopback_test() {
    println!("{}", form("uart16550_init_test", TestType::Passed));
    const ADDR: usize = 0x1000_0000;
    let uart = UART16550::<256>::new(ADDR);
    uart.enable_loopback();
    write_reg(ADDR, Register::Thr, b'a');

    while read_reg(ADDR, Register::Lsr) & Masks::LSR_DATA_READY == 0 {}

    let value = read_reg(ADDR, Register::Rbr);
    uart.disable_loopback();

    assert!(
        value == b'a',
        "{}",
        form("uart16550_loopback_test", TestType::Failed)
    );
    println!("{}", form("uart16550_loopback_test", TestType::Passed));
}

fn uart16550_rx_interrupt_test() {
    const ADDR: usize = 0x1000_0000;

    let uart = UART16550::<256>::new(ADDR);

    // Сначала выключаем RX interrupt
    uart.disable_rx_interrupt();

    // Включаем
    uart.enable_rx_interrupt();

    let ier = read_reg(ADDR, Register::Ier);

    assert!(
        ier & Masks::IER_RX_AVAILABLE != 0,
        "{}",
        form("uart16550_rx_interrupt_test", TestType::Failed)
    );

    println!("{}", form("uart16550_rx_interrupt_test", TestType::Passed));
}

fn uart16550_tx_interrupt_test() {
    const ADDR: usize = 0x1000_0000;

    let uart = UART16550::<256>::new(ADDR);

    uart.disable_tx_interrupt();
    uart.enable_tx_interrupt();

    let ier = read_reg(ADDR, Register::Ier);

    assert!(
        ier & Masks::IER_TX_EMPTY != 0,
        "{}",
        form("uart16550_tx_interrupt_test", TestType::Failed)
    );

    println!("{}", form("uart16550_tx_interrupt_test", TestType::Passed));
    uart.disable_tx_interrupt();
}

pub extern "C" fn uart16550_rw_test() -> ! {
    let tests: &[&[u8]] = &[
        b"A",
        b"Hello",
        b"Hello UART!",
        b"1234567890123456",
        b"abcdefghijklmnopqrstuvwxyz",
        b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    ];

    let mut test_index = 0;

    // Настройка UART один раз
    critical_section::with(|cs| {
        let mut uart = UART.borrow(cs).borrow_mut();

        uart.init(TriggerLevel::One);
        uart.enable_rx_interrupt();
    });

    loop {
        if test_index < tests.len() {
            let data = tests[test_index];

            // Отправляем данные
            let write_result = critical_section::with(|cs| {
                let mut uart = UART.borrow(cs).borrow_mut();
                uart.enable_loopback();
                let res = uart.write(data);
                res
            });

         
            // Читаем столько же байт, сколько отправили
            let mut received = [0u8; 64];
            let mut received_len = 0;

            while received_len < data.len() {
                let byte = critical_section::with(|cs| UART.borrow(cs).borrow_mut().read());

                match byte {
                    Some(byte) => {
                        received[received_len] = byte;
                        received_len += 1;
                    }

                    None => {
                        reschedule();
                    }
                }
            }


 			let byte = critical_section::with(|cs| UART.borrow(cs).borrow_mut().disable_loopback());
			assert!(
                write_result.is_ok(),
                "{}",
                form("uart16550_rw_test:", TestType::Failed)
            );

            // Проверяем весь пакет
            assert_eq!(
                &received[..received_len],
                data,
                "{}",
                form("uart16550_rw_test: read", TestType::Failed)
            );

            // Следующий размер
            test_index += 1;
			if test_index == tests.len(){
				let byte = critical_section::with(|cs| UART.borrow(cs).borrow_mut().disable_rx_interrupt());
				let byte = critical_section::with(|cs| UART.borrow(cs).borrow_mut().disable_tx_interrupt());
           	 	println!("{}", form("uart16550_rw_test", TestType::Passed));
			}

        }
    }
}


fn uart16550_trigger_level_test(level: TriggerLevel, expected: usize) {
    const ADDR: usize = 0x1000_0000;

    let uart = UART16550::<256>::new(ADDR);

    uart.init(level);
	uart.enable_rx_interrupt();
    uart.enable_loopback();

    for _ in 0..(expected - 1) {
        write_reg(ADDR, Register::Thr, b'A');
    }

    let iir = read_reg(ADDR, Register::Iir);
	uart.disable_loopback();

    assert!(
        iir & Masks::IIR_REASON != Masks::IIR_RECEIVED_DATA_AVAILABLE,
        "{}",
        form("uart16550_trigger_level_test", TestType::Failed)
    );

    // Отправляем последний байт
	uart.enable_loopback();
    write_reg(ADDR, Register::Thr, b'A');

    let iir = read_reg(ADDR, Register::Iir);

	uart.disable_loopback();
	uart.disable_rx_interrupt();
	uart.disable_tx_interrupt();
    assert!(
        iir & Masks::IIR_REASON == Masks::IIR_RECEIVED_DATA_AVAILABLE,
        "{}",
        form("uart16550_trigger_level_test", TestType::Failed)
    );

	let message = format!("uart16550_trigger_level_test{}", expected);
    println!("{}", form(&message, TestType::Passed));

}

fn uart16550_baud_rate_test() {
    const ADDR: usize = 0x1000_0000;

    let mut uart = UART16550::<256>::new(ADDR);

    // 115200 baud
    uart.set_baud_rate(115_200);

    // Включаем DLAB, чтобы получить доступ к DLL/DLM
    let lcr = read_reg(ADDR, Register::Lcr);

    write_reg(
        ADDR,
        Register::Lcr,
        lcr | Masks::LCR_DLAB
    );

    let dll = read_reg(ADDR, Register::Dll);
    let dlm = read_reg(ADDR, Register::Dlm);

    // Возвращаем LCR
    write_reg(
        ADDR,
        Register::Lcr,
        lcr & !Masks::LCR_DLAB
    );

    // divisor = 2
    assert_eq!(
        dll,
        0x02,
        "{}",
        form("uart16550_baud_rate_test: DLL", TestType::Failed)
    );

    assert_eq!(
        dlm,
        0x00,
        "{}",
        form("uart16550_baud_rate_test: DLM", TestType::Failed)
    );

    println!(
        "{}",
        form("uart16550_baud_rate_test", TestType::Passed)
    );
}

fn uart16550_line_config_test() {
    const ADDR: usize = 0x1000_0000;

    let mut uart = UART16550::<256>::new(ADDR);

    // 8 data bits, no parity, 1 stop bit
    uart.set_line_config(
        DataBits::Eight,
        Parity::None,
        StopBits::One,
    );

    let lcr = read_reg(ADDR, Register::Lcr);

    assert_eq!(
        lcr & 0b11,
        Masks::LCR_DATA_BITS_8,
        "{}",
        form("uart16550_line_config: data bits", TestType::Failed)
    );

    assert_eq!(
        lcr & (1 << 2),
        Masks::LCR_STOP_BITS_1,
        "{}",
        form("uart16550_line_config: stop bits", TestType::Failed)
    );

    assert_eq!(
        lcr & (0b111 << 3),
        Masks::LCR_PARITY_NONE,
        "{}",
        form("uart16550_line_config: parity", TestType::Failed)
    );

    // 7 data bits, even parity, 2 stop bits
    uart.set_line_config(
        DataBits::Seven,
        Parity::Even,
        StopBits::Two,
    );

    let lcr = read_reg(ADDR, Register::Lcr);

    assert_eq!(
        lcr & 0b11,
        Masks::LCR_DATA_BITS_7,
        "{}",
        form("uart16550_line_config: data bits", TestType::Failed)
    );

    assert_eq!(
        lcr & (1 << 2),
        Masks::LCR_STOP_BITS_2,
        "{}",
        form("uart16550_line_config: stop bits", TestType::Failed)
    );

    assert_eq!(
        lcr & (0b111 << 3),
        Masks::LCR_PARITY_EVEN,
        "{}",
        form("uart16550_line_config: parity", TestType::Failed)
    );

    // 6 data bits, odd parity, 1 stop bit
    uart.set_line_config(
        DataBits::Six,
        Parity::Odd,
        StopBits::One,
    );

    let lcr = read_reg(ADDR, Register::Lcr);

    assert_eq!(
        lcr & 0b11,
        Masks::LCR_DATA_BITS_6,
        "{}",
        form("uart16550_line_config: data bits", TestType::Failed)
    );

    assert_eq!(
        lcr & (1 << 2),
        Masks::LCR_STOP_BITS_1,
        "{}",
        form("uart16550_line_config: stop bits", TestType::Failed)
    );

    assert_eq!(
        lcr & (0b111 << 3),
        Masks::LCR_PARITY_ODD,
        "{}",
        form("uart16550_line_config: parity", TestType::Failed)
    );

    println!(
        "{}",
        form("uart16550_line_config_test", TestType::Passed)
    );
}


pub fn start_test() -> () {
    //  Ring buffer
    ringbuffer_rw_test();
    ringbuffer_empty_test();
    ringbuffer_full_test();
    ringbuffer_fifo_order_test();

    //Registers
    registers_offset_test();
    registers_masks_test();
    registers_rw_test();

    // UART16550
    uart16550_init_test();
    uart16550_new_test();
    uart16550_loopback_test();
    uart16550_rx_interrupt_test();
    uart16550_tx_interrupt_test();
	uart16550_trigger_level_test(TriggerLevel::One, 1);
	uart16550_trigger_level_test(TriggerLevel::Four, 4);
	uart16550_trigger_level_test(TriggerLevel::Eight, 8);
	uart16550_trigger_level_test(TriggerLevel::Fourteen, 14);
	uart16550_baud_rate_test();
	uart16550_line_config_test();

}


#[inline]
pub fn read_cycle() -> u64 {
    let cycles: u64;

    unsafe {
        core::arch::asm!(
            "rdcycle {}",
            out(reg) cycles,
        );
    }

    cycles
}