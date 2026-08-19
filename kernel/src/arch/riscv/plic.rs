use core::ptr::{read_volatile, write_volatile};

const PLIC_BASE: usize = 0x0c00_0000;

// UART 16550 IRQ on QEMU virt
pub const UART_IRQ: usize = 10;

// ------------------------------------------------------------
// PLIC S-mode context
// ------------------------------------------------------------

const PLIC_S_ENABLE: usize = PLIC_BASE + 0x2080;

const PLIC_S_THRESHOLD: usize = PLIC_BASE + 0x201000;

const PLIC_S_CLAIM: usize = PLIC_BASE + 0x201004;

// ------------------------------------------------------------
// Priority
// ------------------------------------------------------------

pub fn plic_set_priority(irq: usize, priority: u32) {
    unsafe {
        write_volatile((PLIC_BASE + irq * 4) as *mut u32, priority);
    }
}

// ------------------------------------------------------------
// Enable interrupt
// ------------------------------------------------------------

pub fn plic_enable(irq: usize) {
    unsafe {
        let addr = PLIC_S_ENABLE as *mut u32;

        let mut value = read_volatile(addr);

        value |= 1 << irq;

        write_volatile(addr, value);
    }
}

// ------------------------------------------------------------
// Set threshold
// ------------------------------------------------------------

pub fn plic_set_threshold(threshold: u32) {
    unsafe {
        write_volatile(PLIC_S_THRESHOLD as *mut u32, threshold);
    }
}

// ------------------------------------------------------------
// Claim interrupt
// ------------------------------------------------------------

pub fn plic_claim() -> u32 {
    unsafe { read_volatile(PLIC_S_CLAIM as *const u32) }
}

// ------------------------------------------------------------
// Complete interrupt
// ------------------------------------------------------------

pub fn plic_complete(irq: u32) {
    unsafe {
        write_volatile(PLIC_S_CLAIM as *mut u32, irq);
    }
}

// ------------------------------------------------------------
// Initialization
// ------------------------------------------------------------

pub fn plic_init() {
    // Give UART interrupt priority 1
    plic_set_priority(UART_IRQ, 1);

    // Accept interrupts with priority > 0
    plic_set_threshold(0);

    // Enable UART IRQ
    plic_enable(UART_IRQ);
}
