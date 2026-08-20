use core::ptr::{read_volatile, write_volatile};
use crate::arch::traits::TargetPciBus;

pub struct RiscvPciBus;

const ECAM_BASE: usize = 0x30000000;

impl RiscvPciBus {
    // Compute memory-mapped PCI config address (Bus 0, Device 0..31, Function 0..7)
    #[inline]
    fn pci_ecam_addr(bus: u8, dev: u8, func: u8, off: u16) -> usize {
        (ECAM_BASE |
        ((bus as usize) << 20) |
        ((dev as usize) << 15) |
        ((func as usize) << 12) |
        (off & 0xFFF) as usize)
    }
}

impl TargetPciBus for RiscvPciBus {
    fn pci_read16(bus: u8, dev: u8, func: u8, off: u16) -> u16 {
        unsafe { read_volatile(RiscvPciBus::pci_ecam_addr(bus, dev, func, off) as *const u16) }
    }

    fn pci_read32(bus: u8, dev: u8, func: u8, off: u16) -> u32 {
        unsafe { read_volatile(RiscvPciBus::pci_ecam_addr(bus, dev, func, off) as *const u32) }
    }

    fn pci_write16(bus: u8, dev: u8, func: u8, off: u16, val: u16) {
        unsafe { write_volatile(RiscvPciBus::pci_ecam_addr(bus, dev, func, off) as *mut u16, val) }
    }

    fn pci_write32(bus: u8, dev: u8, func: u8, off: u16, val: u32) {
        unsafe { write_volatile(RiscvPciBus::pci_ecam_addr(bus, dev, func, off) as *mut u32, val) }
    }

    fn pci_read8(bus: u8, dev: u8, func: u8, off: u16) -> u8{
        unsafe { read_volatile(RiscvPciBus::pci_ecam_addr(bus, dev, func, off) as *const u8) }
    }

    fn pci_write8(bus: u8, dev: u8, func: u8, off: u16, val: u8) {
        unsafe { write_volatile(RiscvPciBus::pci_ecam_addr(bus, dev, func, off) as *mut u8, val)}
    }
}
