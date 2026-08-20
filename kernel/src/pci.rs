use crate::arch::traits::TargetPciBus;
use crate::arch::PciBus;

// Standard PCI Configuration Space offsets.
const PCI_VENDOR_DEVICE_REG: u16 = 0x00;
const PCI_CLASS_REVISION_REG: u16 = 0x08;

// Standard PCI class codes for an SD Host Controller.
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

/// Reads the main fields from PCI Configuration Space.
///
/// Returns None if there is no PCI function at this BDF.
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

            if device.class == SDHCI_CLASS
                && device.subclass == SDHCI_SUBCLASS
            {
                info!(
                    "Found SDHCI at {:02x}:{:02x}.{}",
                    device.address.bus,
                    device.address.device,
                    device.address.function,
                );
            }
        }
    }

    info!("PCI scan completed");

    loop {
        core::hint::spin_loop();
    }
}
