use crate::pci::*;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use crate::arch::traits::TargetPciBus;

// --- MOCK STORAGE & STATE ---
static mut MOCK_REGS: BTreeMap<(u8, u8, u8, u16), u32> = BTreeMap::new();
static mut MOCK_BAR_MASKS: BTreeMap<(u8, u8, u8, u16), u32> = BTreeMap::new();
static mut MOCK_MMIO_BASE: usize = 0x1000_0000;
static mut MOCK_MMIO_SIZE: usize = 0x1000_0000;

pub struct MockPciBus;

impl MockPciBus {
    pub fn reset() {
        unsafe {
            MOCK_REGS.clear();
            MOCK_BAR_MASKS.clear();
            MOCK_MMIO_BASE = 0x1000_0000;
            MOCK_MMIO_SIZE = 0x1000_0000;
            PCI_DEVICES.clear();
        }
    }

    pub fn set_reg32(bus: u8, dev: u8, func: u8, off: u16, val: u32) {
        unsafe {
            MOCK_REGS.insert((bus, dev, func, off), val);
        }
    }

    pub fn set_reg16(bus: u8, dev: u8, func: u8, off: u16, val: u16) {
        let aligned_off = off & !3;
        let shift = ((off & 3) * 8) as u32;
        unsafe {
            let current = MOCK_REGS.get(&(bus, dev, func, aligned_off)).copied().unwrap_or(0);
            let mask = !(0xFFFFu32 << shift);
            let updated = (current & mask) | ((val as u32) << shift);
            MOCK_REGS.insert((bus, dev, func, aligned_off), updated);
        }
    }

    pub fn set_bar0_size(bus: u8, dev: u8, func: u8, size: usize) {
        let mask = (!(size - 1) as u32) & PCI_BAR_ADDRESS_MASK;
        unsafe {
            MOCK_BAR_MASKS.insert((bus, dev, func, PCI_BAR0_REG), mask);
        }
    }
}

impl TargetPciBus for MockPciBus {
    fn mmio_base() -> usize {
        unsafe { MOCK_MMIO_BASE }
    }

    fn mmio_size() -> usize {
        unsafe { MOCK_MMIO_SIZE }
    }

    fn pci_read32(bus: u8, dev: u8, func: u8, off: u16) -> u32 {
        let aligned_off = off & !3;
        unsafe {
            *MOCK_REGS.get(&(bus, dev, func, aligned_off)).unwrap_or(&0xFFFF_FFFF)
        }
    }

    fn pci_write32(bus: u8, dev: u8, func: u8, off: u16, val: u32) {
        let aligned_off = off & !3;
        unsafe {
            if aligned_off == PCI_BAR0_REG && val == u32::MAX {
                // Simulate hardware returning the size mask on probe (write u32::MAX)
                let mask = MOCK_BAR_MASKS
                    .get(&(bus, dev, func, PCI_BAR0_REG))
                    .copied()
                    .unwrap_or(0xFFFF_F000); // Default to 4KB mask
                let flags = MOCK_REGS.get(&(bus, dev, func, PCI_BAR0_REG)).copied().unwrap_or(0) & !PCI_BAR_ADDRESS_MASK;
                MOCK_REGS.insert((bus, dev, func, PCI_BAR0_REG), mask | flags);
            } else {
                MOCK_REGS.insert((bus, dev, func, aligned_off), val);
            }
        }
    }

    fn pci_read16(bus: u8, dev: u8, func: u8, off: u16) -> u16 {
        let val32 = Self::pci_read32(bus, dev, func, off);
        let shift = ((off & 3) * 8) as u32;
        (val32 >> shift) as u16
    }

    fn pci_write16(bus: u8, dev: u8, func: u8, off: u16, val: u16) {
        let shift = ((off & 3) * 8) as u32;
        let aligned_off = off & !3;
        let current = Self::pci_read32(bus, dev, func, aligned_off);
        let mask = !(0xFFFFu32 << shift);
        let updated = (current & mask) | ((val as u32) << shift);
        Self::pci_write32(bus, dev, func, aligned_off, updated);
    }

    fn pci_read8(bus: u8, dev: u8, func: u8, off: u16) -> u8 {
        let val32 = Self::pci_read32(bus, dev, func, off);
        let shift = ((off & 3) * 8) as u32;
        (val32 >> shift) as u8
    }

    fn pci_write8(bus: u8, dev: u8, func: u8, off: u16, val: u8) {
        let shift = ((off & 3) * 8) as u32;
        let aligned_off = off & !3;
        let current = Self::pci_read32(bus, dev, func, aligned_off);
        let mask = !(0xFFu32 << shift);
        let updated = (current & mask) | ((val as u32) << shift);
        Self::pci_write32(bus, dev, func, aligned_off, updated);
    }
}
