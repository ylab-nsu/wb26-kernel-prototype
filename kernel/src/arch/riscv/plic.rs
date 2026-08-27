use crate::arch::traits::TargetInterruptController;

pub struct RiscvInterruptController;

pub const PLIC_BASE: usize = 0x0C00_0000;
pub const PLIC_PRIORITY: usize = PLIC_BASE;
pub const PLIC_ENABLE_HART0_SMODE: usize = PLIC_BASE + 0x2080;
pub const PLIC_THRESHOLD_HART0_SMODE: usize = PLIC_BASE + 0x20_1000;
pub const PLIC_CLAIM_HART0_SMODE: usize = PLIC_BASE + 0x20_1004;

impl TargetInterruptController for RiscvInterruptController {
    fn enable_irq(irq: u32) {
        unsafe {
            // 1. Set the priority of the IRQ to > 0 (e.g., 1)
            let priority_addr = (PLIC_PRIORITY + (irq as usize) * 4) as *mut u32;
            core::ptr::write_volatile(priority_addr, 1);

            // 2. Enable the IRQ for Hart 0 S-mode
            // Each bit in this array represents an IRQ. IRQ 32 is bit 0 of word 1.
            let enable_word = irq / 32;
            let enable_bit = irq % 32;
            let enable_addr = (PLIC_ENABLE_HART0_SMODE + (enable_word as usize) * 4) as *mut u32;
            let current_enable = core::ptr::read_volatile(enable_addr);
            core::ptr::write_volatile(enable_addr, current_enable | (1 << enable_bit));

            // 3. Set the priority threshold to 0 (accept any interrupt > 0)
            let threshold_addr = PLIC_THRESHOLD_HART0_SMODE as *mut u32;
            core::ptr::write_volatile(threshold_addr, 0);
        }
    }
}

