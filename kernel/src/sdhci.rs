use core::ptr::{read_volatile, write_volatile};
use crate::pci::{ pci_enable_device, PCI_BAR0_REG };
use crate::arch::traits::TargetPciBus;
use crate::arch::PciBus;

pub static mut sdhci_pci_addr: Option<(u8, u8, u8)> = None;

pub extern "C" fn driver_task() -> ! {

    unsafe {
        if let Some((bus, dev, func)) = sdhci_pci_addr {
            pci_enable_device(bus, dev, func);

            let sdhci_base = PciBus::pci_read32(bus, dev, func, PCI_BAR0_REG) & !(0x0F as u32);

            // 1. Check Host Version (Offset 0xFE, 16-bit read)
            let version_ptr = (sdhci_base + 0xFE) as *const u16;
            let version = core::ptr::read_volatile(version_ptr);

            // 2. Check Present State (Offset 0x24, 32-bit read)
            let present_state_ptr = (sdhci_base + 0x24) as *const u32;
            let present_state = core::ptr::read_volatile(present_state_ptr);
            let card_inserted = (present_state & (1 << 16)) != 0;

            // 3. Trigger Software Reset (Offset 0x2F, 8-bit write/read)
            let reset_ptr = (sdhci_base + 0x2F) as *mut u8;
            core::ptr::write_volatile(reset_ptr, 0x01); // 0x01 = Reset All

            // Wait for the controller to clear the reset bit
            let mut reset_cleared = false;
            for _ in 0..1000 {
                if core::ptr::read_volatile(reset_ptr) & 0x01 == 0 {
                    reset_cleared = true;
                    break;
                }
            }

            info!("SDHCI Version Reg: {:#06x}", version);
            info!("Card Inserted: {}", card_inserted);
            info!("Reset Successful: {}", reset_cleared);
        }
        else {
            error!("No sdhci address on pci bus on sdhci bus driver start");
        }
    }

    loop { }
}
