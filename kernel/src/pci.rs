use alloc::vec::Vec;
use crate::arch::traits::{ TargetPciBus, TargetInterruptController };
use crate::arch::{ PciBus, InterruptController };

// Standard PCI Configuration Space offsets
pub const PCI_VENDOR_DEVICE_REG: u16 = 0x00;
pub const PCI_COMMAND_REG: u16 = 0x04;
pub const PCI_CLASS_REVISION_REG: u16 = 0x08;
pub const PCI_BAR0_REG: u16 = 0x10;
pub const PCI_HEADER_TYPE_REG: u16 = 0x1C;
pub const PCI_INTERRUPT_LINE_REG: u16 = 0x3C;
pub const PCI_INTERRUPT_PIN_REG: u16 = 0x3D;

// PCI Command Register
pub const PCI_COMMAND_MEMORY_SPACE: u16 = 1 << 1;
pub const PCI_COMMAND_IO_SPACE: u16 = 1 << 0;
pub const PCI_COMMAND_INTERRUPT_DISABLE: u16 = 1 << 10;
pub const PCI_COMMAND_BUS_MASTER: u16 = 1 << 2;

// Memory BAR fields
pub const PCI_BAR_IO_SPACE: u32 = 1;
pub const PCI_BAR_MEMORY_TYPE_MASK: u32 = 0b11 << 1;
pub const PCI_BAR_MEMORY_TYPE_32: u32 = 0b00 << 1;
pub const PCI_BAR_ADDRESS_MASK: u32 = 0xffff_fff0;

// Standard PCI class codes for an SD Host Controller
pub const SDHCI_CLASS: u8 = 0x08;
pub const SDHCI_SUBCLASS: u8 = 0x05;

pub const PCI_BUS: u8 = 0;
pub const PCI_DEVICES_PER_BUS: u8 = 32;
pub const PCI_FUNCTIONS_PER_DEVICE: u8 = 8;

// Bus-specific data about device attached to it
pub struct PciDeviceInfo {
    pub bus: u8,
    pub dev: u8,
    pub func: u8,
}

#[derive(Clone, Copy, Debug)]
struct MemoryBar {
    size: usize,
    flags: u32,
}

pub static mut PCI_DEVICES: Vec<PciDeviceInfo> = Vec::new();

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

// TODO: Accept closure as interrupt handler
pub fn pci_enable_interrupt(bus: u8, dev: u8, func: u8) {
    let command = PciBus::pci_read16(bus, dev, func, PCI_COMMAND_REG);

    PciBus::pci_write16(
        bus,
        dev,
        func,
        PCI_COMMAND_REG,
        command & !PCI_COMMAND_INTERRUPT_DISABLE,
    );

    // TODO: Determine which irq number does pci device use
    InterruptController::enable_irq(32);
    InterruptController::enable_irq(33);
    InterruptController::enable_irq(34);
    InterruptController::enable_irq(35);
}

pub fn pci_enable_bus_mastering(bus: u8, dev: u8, func: u8) {
    let command = PciBus::pci_read16(bus, dev, func, PCI_COMMAND_REG);

    PciBus::pci_write16(
        bus,
        dev,
        func,
        PCI_COMMAND_REG,
        command | PCI_COMMAND_BUS_MASTER,
    );
}

pub fn pci_function_is_present(bus: u8, dev: u8, func: u8) -> bool {
    let id = PciBus::pci_read32(bus, dev, func, PCI_VENDOR_DEVICE_REG);

    let vendor_id = id as u16;

    // 0xffff means that there is no PCI function at this BDF.
    vendor_id != 0xFFFF
}

fn probe_bar0(bus: u8, dev: u8, func: u8) -> Option<MemoryBar> {
    let original = PciBus::pci_read32(bus, dev, func, PCI_BAR0_REG);

    if original & PCI_BAR_IO_SPACE != 0 {
        error!("PCI {}:{}:{} BAR0 is an I/O BAR", bus, dev, func);
        return None;
    }

    let memory_type = original & PCI_BAR_MEMORY_TYPE_MASK;

    // This minimal implementation supports only a 32-bit Memory BAR
    if memory_type != PCI_BAR_MEMORY_TYPE_32 {
        error!(
            "Unsupported PCI {}:{}:{} BAR0 memory type: {memory_type:#x}",
            bus, dev, func
        );
        return None;
    }

    PciBus::pci_write32(bus, dev, func, PCI_BAR0_REG, u32::MAX);
    let mask = PciBus::pci_read32(bus, dev, func, PCI_BAR0_REG);

    PciBus::pci_write32(bus, dev, func, PCI_BAR0_REG, original);

    let address_mask = mask & PCI_BAR_ADDRESS_MASK;

    if address_mask == 0 {
        error!("PCI {}:{}:{} BAR0 is not implemented", bus, dev, func);
        return None;
    }

    let size = (!address_mask).wrapping_add(1) as usize;

    if size == 0 || !size.is_power_of_two() {
        error!(
            "Invalid PCI {}:{}:{} BAR0 size: {size:#x}",
            bus, dev, func
        );
        return None;
    }

    info!(
        "PCI {}:{}:{} BAR0 before probing: {original:#010x}",
        bus, dev, func
    );
    info!("PCI {}:{}:{} BAR0 mask: {mask:#010x}", bus, dev, func);
    info!("PCI {}:{}:{} BAR0 size: {size:#x}", bus, dev, func);

    Some(MemoryBar {
        size,
        flags: original & !PCI_BAR_ADDRESS_MASK,
    })
}

fn align_up(address: usize, alignment: usize) -> Option<usize> {
    if alignment == 0 || !alignment.is_power_of_two() {
        return None;
    }

    address
        .checked_add(alignment - 1)
        .map(|value| value & !(alignment - 1))
}

fn allocate_mmio(next_mmio_address: &mut usize, size: usize) -> Option<usize> {
    let mmio_end = PciBus::mmio_base().checked_add(PciBus::mmio_size())?;
    let address = align_up(*next_mmio_address, size)?;
    let end = address.checked_add(size)?;

    if address < PciBus::mmio_base() || end > mmio_end {
        return None;
    }

    *next_mmio_address = end;
    Some(address)
}

fn configure_bar0(
    bus: u8,
    dev: u8,
    func: u8,
    next_mmio_address: &mut usize,
) -> Option<usize> {
    let bar = probe_bar0(bus, dev, func)?;
    let bar_address = allocate_mmio(next_mmio_address, bar.size)?;

    if bar_address > u32::MAX as usize {
        error!("BAR0 address does not fit into a 32-bit BAR");
        return None;
    }

    let bar_value = (bar_address as u32) | bar.flags;
    PciBus::pci_write32(bus, dev, func, PCI_BAR0_REG, bar_value);

    let actual_bar = PciBus::pci_read32(bus, dev, func, PCI_BAR0_REG);
    let actual_address = (actual_bar & PCI_BAR_ADDRESS_MASK) as usize;

    if actual_address != bar_address {
        error!(
            "BAR0 verification failed for PCI {}:{}:{}: requested={bar_address:#x}, actual={actual_address:#x}",
            bus, dev, func
        );
        return None;
    }

    info!(
        "PCI {}:{}:{} BAR0 assigned: {actual_address:#x}",
        bus, dev, func
    );
    Some(actual_address)
}

pub fn map_all_bars(
    bus: u8,
    dev: u8,
    func: u8,
    next_mmio_address: &mut usize,
) -> Result<(), ()> {
    let header = PciBus::pci_read8(bus, dev, func, PCI_HEADER_TYPE_REG);

    // Bit 7 is the multifunction flag; bits 6:0 contain the header type.
    match header & 0x7f {
        0x00 => configure_bar0(bus, dev, func, next_mmio_address)
            .map(|_| ())
            .ok_or(()),
        0x01 => {
            error!("PCI header type 1 is not implemented");
            Err(())
        }
        header_type => {
            error!(
                "Unknown header type {header_type:#x} on PCI {}:{}:{}",
                bus, dev, func
            );
            Err(())
        }
    }
}

pub fn register_pci_device(bus: u8, dev: u8, func: u8) {
    unsafe {
        PCI_DEVICES.push(PciDeviceInfo { bus, dev, func });
    }

    // TODO: Match devices with drivers through the kernel device subsystem.
}

pub fn spawn_driver(bus: u8, dev: u8, func: u8) {
    let class_revision = PciBus::pci_read32(bus, dev, func, PCI_CLASS_REVISION_REG);
    let class = (class_revision >> 24) as u8;
    let subclass = (class_revision >> 16) as u8;

    if class == SDHCI_CLASS && subclass == SDHCI_SUBCLASS {
        info!("Found SDHCI at {:02x}:{:02x}.{}", bus, dev, func);

        // TODO: Actually run the matched driver instead of becoming it.
        crate::sdhci::driver_task();
    }
}

pub extern "C" fn driver_task() -> ! {
    info!("Scanning PCI bus {PCI_BUS}...");

    let mut next_mmio_address = PciBus::mmio_base();

    for device_number in 0..PCI_DEVICES_PER_BUS {
        for function_number in 0..PCI_FUNCTIONS_PER_DEVICE {
            if !pci_function_is_present(PCI_BUS, device_number, function_number) {
                continue;
            }

            let class_revision = PciBus::pci_read32(
                PCI_BUS,
                device_number,
                function_number,
                PCI_CLASS_REVISION_REG,
            );
            let class = (class_revision >> 24) as u8;
            let subclass = (class_revision >> 16) as u8;

            // TODO: Remove skipping of all devices except SDHCI
            if class != SDHCI_CLASS || subclass != SDHCI_SUBCLASS {
                continue;
            }

            // Disable MMIO and PIO decoding while BARs are configured
            pci_disable_device(PCI_BUS, device_number, function_number);

            if map_all_bars(
                PCI_BUS,
                device_number,
                function_number,
                &mut next_mmio_address,
            )
            .is_err()
            {
                error!(
                    "Could not configure BARs for PCI {:02x}:{:02x}.{}",
                    PCI_BUS, device_number, function_number
                );
                continue;
            }

            register_pci_device(PCI_BUS, device_number, function_number);

            // TODO: Replace this with driver matching after the driver
            // subsystem is implemented.
            spawn_driver(PCI_BUS, device_number, function_number);

            // TODO: Check Header Type and stop after function 0 for a
            // non-multifunction device.
        }
    }

    loop {
        core::hint::spin_loop();
    }
}
