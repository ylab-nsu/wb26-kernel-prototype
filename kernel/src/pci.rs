use crate::arch::traits::TargetPciBus;
use crate::arch::PciBus;
use core::ptr::{read_volatile, write_volatile};

// Standard PCI Configuration Space offsets
const PCI_VENDOR_DEVICE_REG: u16 = 0x00;
const PCI_COMMAND_REG: u16 = 0x04;
const PCI_CLASS_REVISION_REG: u16 = 0x08;
const PCI_BAR0_REG: u16 = 0x10;
const PCI_HEADER_TYPE_REG: u16 = 0x1C;

// PCI Command Register
const PCI_COMMAND_MEMORY_SPACE: u16 = 1 << 1;
const PCI_COMMAND_IO_SPACE: u16 = 1 << 0;

// Memory BAR fields
const PCI_BAR_IO_SPACE: u32 = 1;
const PCI_BAR_MEMORY_TYPE_MASK: u32 = 0b11 << 1;
const PCI_BAR_MEMORY_TYPE_32: u32 = 0b00 << 1;
const PCI_BAR_ADDRESS_MASK: u32 = 0xffff_fff0;

// Standard PCI class codes for an SD Host Controller
const SDHCI_CLASS: u8 = 0x08;
const SDHCI_SUBCLASS: u8 = 0x05;

const PCI_BUS: u8 = 0;
const PCI_DEVICES_PER_BUS: u8 = 32;
const PCI_FUNCTIONS_PER_DEVICE: u8 = 8;

pub fn pci_disable_device(bus: u8, dev: u8, func: u8) {
    let command = PciBus::pci_read16(bus, dev, func, PCI_COMMAND_REG);

    PciBus::pci_write16(
        bus,
        dev,
        func,
        PCI_COMMAND_REG,
        command & !PCI_COMMAND_MEMORY_SPACE & !PCI_COMMAND_IO_SPACE,
    );
}

pub fn pci_enable_device(bus: u8, dev: u8, func: u8) {
    let command = PciBus::pci_read16(bus, dev, func, PCI_COMMAND_REG);

    PciBus::pci_write16(
        bus,
        dev,
        func,
        PCI_COMMAND_REG,
        command | PCI_COMMAND_MEMORY_SPACE | PCI_COMMAND_IO_SPACE,
    );
}

pub fn pci_function_is_present(bus: u8, dev: u8, func: u8) -> bool {
    let id = PciBus::pci_read32(bus, dev, func, PCI_VENDOR_DEVICE_REG);

    let vendor_id = id as u16;

    // 0xffff means that there is no PCI function at this BDF.
    vendor_id != 0xFFFF
}

pub fn map_all_bars(bus: u8, dev: u8, func: u8) -> Result<(), ()> {
    // Determine header type
    let header = PciBus::pci_read8(bus, dev, func, PCI_HEADER_TYPE_REG);

    match header & 0xEF {
        0x00 => {
            // TODO: Implement proper bars mapping
            PciBus::pci_write32(bus, dev, func, PCI_BAR0_REG, PciBus::mmio_base() as u32);

            Ok(())
        }
        0x01 => {
            error!("PCI header type 1 not implemented");
            Err(())
        }
        _ => {
            error!("Unknown header type on PCI {}:{}:{}", bus, dev, func);
            Err(())
        }
    }
}

pub fn register_pci_device(bus: u8, dev: u8, func: u8) {
    // TODO: Change to proper device registration after drivers subsystem implementation
}

pub fn spawn_driver(bus: u8, dev: u8, func: u8) {
    let class_revision = PciBus::pci_read32(bus, dev, func, PCI_CLASS_REVISION_REG);

    if class_revision >> 16 == 0x0805 {
        info!("Found SDHCI");

        pci_enable_device(bus, dev, func);

        // TODO: Actually run driver instead of "becoming" it
        let sdhci_base = PciBus::mmio_base();

        unsafe {
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
    }
}

pub extern "C" fn driver_task() -> ! {
    info!("Scanning PCI bus {PCI_BUS}...");

    for device_number in 0..PCI_DEVICES_PER_BUS {
        for function_number in 0..PCI_FUNCTIONS_PER_DEVICE {
            // TODO: Remove skipping of all except sdhci
            let class_revision = PciBus::pci_read32(PCI_BUS, device_number, function_number, PCI_CLASS_REVISION_REG);
            if class_revision >> 16 != 0x0805 {
                continue;
            }

            // Disable MMIO or PIO for device on startup
            pci_disable_device(PCI_BUS, device_number, function_number);

            if pci_function_is_present(PCI_BUS, device_number, function_number) {
                if let Err(_) = map_all_bars(PCI_BUS, device_number, function_number) {
                    continue;
                }

                register_pci_device(PCI_BUS, device_number, function_number);

                // TODO: Replace with driver matching after drivers subsystem implementation
                spawn_driver(PCI_BUS, device_number, function_number);
            }

            // TODO: Check Header register and break if device isn't multifunctional
        }
    }

    loop {}
}
