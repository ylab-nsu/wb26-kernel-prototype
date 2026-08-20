use crate::arch::traits::TargetPciBus;
use crate::arch::PciBus;

// Standard PCI Configuration Space offsets
const PCI_VENDOR_DEVICE_REG: u16 = 0x00;
const PCI_COMMAND_REG: u16 = 0x04;
const PCI_CLASS_REVISION_REG: u16 = 0x08;
const PCI_BAR0_REG: u16 = 0x10;

// PCI Command Register
const PCI_COMMAND_MEMORY_SPACE: u16 = 1 << 1;

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

#[derive(Clone, Copy, Debug)]
struct PciAddress {
    bus: u8,
    device: u8,
    function: u8,
}

#[derive(Clone, Copy, Debug)]
struct PciDevice {
    address: PciAddress,
    vendor_id: u16,
    device_id: u16,
    revision: u8,
    prog_if: u8,
    subclass: u8,
    class: u8,
}

#[derive(Clone, Copy, Debug)]
struct MemoryBar {
    size: usize,
}

/// Reads the main fields from PCI Configuration Space
///
/// Returns None if there is no PCI function at this BDF
fn read_pci_device(address: PciAddress) -> Option<PciDevice> {
    let id = PciBus::pci_read32(
        address.bus,
        address.device,
        address.function,
        PCI_VENDOR_DEVICE_REG,
    );

    let vendor_id = id as u16;

    // 0xffff means that there is no PCI function at this BDF.
    if vendor_id == 0xffff {
        return None;
    }

    let device_id = (id >> 16) as u16;

    let class_revision = PciBus::pci_read32(
        address.bus,
        address.device,
        address.function,
        PCI_CLASS_REVISION_REG,
    );

    Some(PciDevice {
        address,
        vendor_id,
        device_id,
        revision: class_revision as u8,
        prog_if: (class_revision >> 8) as u8,
        subclass: (class_revision >> 16) as u8,
        class: (class_revision >> 24) as u8,
    })
}

// Determines the type and required size of BAR0
fn probe_bar0(device: &PciDevice) -> Option<MemoryBar> {
    let address = device.address;

    let command = PciBus::pci_read16(
        address.bus,
        address.device,
        address.function,
        PCI_COMMAND_REG,
    );

    PciBus::pci_write16(
        address.bus,
        address.device,
        address.function,
        PCI_COMMAND_REG,
        command & !PCI_COMMAND_MEMORY_SPACE,
    );

    let original = PciBus::pci_read32(address.bus, address.device, address.function, PCI_BAR0_REG);

    if original & PCI_BAR_IO_SPACE != 0 {
        error!("SDHCI BAR0 is an I/O BAR");

        PciBus::pci_write16(
            address.bus,
            address.device,
            address.function,
            PCI_COMMAND_REG,
            command,
        );

        return None;
    }

    let memory_type = original & PCI_BAR_MEMORY_TYPE_MASK;

    if memory_type != PCI_BAR_MEMORY_TYPE_32 {
        error!("Unsupported SDHCI BAR0 memory type: {memory_type:#x}");

        PciBus::pci_write16(
            address.bus,
            address.device,
            address.function,
            PCI_COMMAND_REG,
            command,
        );

        return None;
    }

    PciBus::pci_write32(
        address.bus,
        address.device,
        address.function,
        PCI_BAR0_REG,
        u32::MAX,
    );

    let mask = PciBus::pci_read32(address.bus, address.device, address.function, PCI_BAR0_REG);

    PciBus::pci_write32(
        address.bus,
        address.device,
        address.function,
        PCI_BAR0_REG,
        original,
    );

    PciBus::pci_write16(
        address.bus,
        address.device,
        address.function,
        PCI_COMMAND_REG,
        command,
    );

    let address_mask = mask & PCI_BAR_ADDRESS_MASK;

    if address_mask == 0 {
        error!("SDHCI BAR0 is not implemented");
        return None;
    }

    let size = (!address_mask).wrapping_add(1) as usize;

    if size == 0 || !size.is_power_of_two() {
        error!("Invalid SDHCI BAR0 size: {size:#x}");
        return None;
    }

    info!("SDHCI BAR0 before probing: {original:#010x}");
    info!("SDHCI BAR0 mask: {mask:#010x}");
    info!("SDHCI BAR0 size: {size:#x}");

    Some(MemoryBar { size })
}

pub extern "C" fn driver_task() -> ! {
    info!("Scanning PCI bus {PCI_BUS}...");

    for device_number in 0..PCI_DEVICES_PER_BUS {
        for function_number in 0..PCI_FUNCTIONS_PER_DEVICE {
            let address = PciAddress {
                bus: PCI_BUS,
                device: device_number,
                function: function_number,
            };

            let Some(device) = read_pci_device(address) else {
                continue;
            };

            info!(
                "PCI {:02x}:{:02x}.{} \
                 vendor={:04x} device={:04x} \
                 class={:02x}:{:02x}:{:02x} \
                 revision={:02x}",
                device.address.bus,
                device.address.device,
                device.address.function,
                device.vendor_id,
                device.device_id,
                device.class,
                device.subclass,
                device.prog_if,
                device.revision,
            );

            if device.class != SDHCI_CLASS || device.subclass != SDHCI_SUBCLASS {
                continue;
            }

            info!(
                "Found SDHCI at {:02x}:{:02x}.{}",
                device.address.bus, device.address.device, device.address.function,
            );

            let Some(bar) = probe_bar0(&device) else {
                error!("Could not probe SDHCI BAR0");
                continue;
            };

            info!("SDHCI BAR0 requires {:#x} bytes", bar.size);
        }
    }

    info!("PCI scan completed");

    loop {
        core::hint::spin_loop();
    }
}
