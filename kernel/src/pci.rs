use crate::arch::traits::TargetPciBus;
use crate::arch::PciBus;

const PCI_COMMAND_REG: u16 = 0x04;
const PCI_CLASS_REG: u16 = 0x08;
const PCI_BAR0_REG: u16 = 0x10;

pub extern "C" fn driver_task() -> ! {
    for dev in 0..23 {
        for func in 0..8 {
            let id = PciBus::pci_read16(0, dev, func, 0x00);
            if (id  == 0xFFFF) {
                continue; // No device present
            }

            // Check Base Class (0x08: System Peripheral) and Subclass (0x05: SD Host Controller)
            let class_rev = PciBus::pci_read32(0, dev, func, PCI_CLASS_REG);
            let class_code = class_rev >> 16;

            if (class_code == 0x0805) {
                info!("Found SD-HCI");

                let bar0 = PciBus::pci_read32(0, dev, func, PCI_BAR0_REG) as usize;
                info!("SD-HCI BAR: {bar0:x}");
            }
        }
    }

    loop { }
}
