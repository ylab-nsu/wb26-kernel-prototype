use crate::pci::*;
use crate::tests::mocks::pci::MockPciBus;
use crate::tests::mocks::plic::MockInterruptController;
use crate::arch::traits::{ TargetPciBus, TargetInterruptController };

fn test_align_up() {
    assert_eq!(align_up(0x1000, 0x1000), Some(0x1000));
    assert_eq!(align_up(0x1001, 0x1000), Some(0x2000));
    assert_eq!(align_up(0x15F0, 0x0100), Some(0x1600));
    assert_eq!(align_up(0x1000, 0), None); // Zero alignment invalid
    assert_eq!(align_up(0x1000, 3), None); // Non-power-of-two invalid
}

fn test_pci_function_is_present() {
    MockPciBus::reset();
    MockInterruptController::reset();

    // Non-existent device returns 0xFFFF_FFFF
    MockPciBus::set_reg32(0, 0, 0, PCI_VENDOR_DEVICE_REG, 0xFFFF_FFFF);
    assert!(!pci_function_is_present(0, 0, 0));

    // Valid device ID (Vendor 0x8086, Device 0x100E)
    MockPciBus::set_reg32(0, 1, 0, PCI_VENDOR_DEVICE_REG, 0x100E_8086);
    assert!(pci_function_is_present(0, 1, 0));
}

fn test_pci_enable_disable_device() {
    MockPciBus::reset();
    MockInterruptController::reset();

    MockPciBus::set_reg16(0, 1, 0, PCI_COMMAND_REG, 0x0000);

    // Test enabling
    pci_enable_device(0, 1, 0);
    let cmd = MockPciBus::pci_read16(0, 1, 0, PCI_COMMAND_REG);
    assert_ne!(cmd & PCI_COMMAND_MEMORY_SPACE, 0);
    assert_ne!(cmd & PCI_COMMAND_IO_SPACE, 0);

    // Test disabling
    pci_disable_device(0, 1, 0);
    let cmd_disabled = MockPciBus::pci_read16(0, 1, 0, PCI_COMMAND_REG);
    assert_eq!(cmd_disabled & PCI_COMMAND_MEMORY_SPACE, 0);
    assert_eq!(cmd_disabled & PCI_COMMAND_IO_SPACE, 0);
}

fn test_pci_enable_bus_mastering() {
    MockPciBus::reset();
    MockInterruptController::reset();
    MockPciBus::set_reg16(0, 1, 0, PCI_COMMAND_REG, 0x0000);

    pci_enable_bus_mastering(0, 1, 0);
    let cmd = MockPciBus::pci_read16(0, 1, 0, PCI_COMMAND_REG);
    assert_ne!(cmd & PCI_COMMAND_BUS_MASTER, 0);
}

fn test_pci_enable_interrupt() {
    MockPciBus::reset();
    MockInterruptController::reset();
    MockPciBus::set_reg16(0, 1, 0, PCI_COMMAND_REG, PCI_COMMAND_INTERRUPT_DISABLE);

    pci_enable_interrupt(0, 1, 0);

    let cmd = MockPciBus::pci_read16(0, 1, 0, PCI_COMMAND_REG);
    assert_eq!(cmd & PCI_COMMAND_INTERRUPT_DISABLE, 0);
    assert_eq!(MockInterruptController::get_enabled_irqs(), alloc::vec![32, 33, 34, 35]);
}

fn test_allocate_mmio() {
    MockPciBus::reset();
    MockInterruptController::reset();
    let mut next_mmio = 0x1000_0000;

    // Valid 4KB allocation
    let addr1 = allocate_mmio(&mut next_mmio, 0x1000);
    assert_eq!(addr1, Some(0x1000_0000));
    assert_eq!(next_mmio, 0x1000_1000);

    // Valid 64KB allocation with alignment auto-adjusted
    let addr2 = allocate_mmio(&mut next_mmio, 0x10000);
    assert_eq!(addr2, Some(0x1001_0000));
    assert_eq!(next_mmio, 0x1002_0000);

    // Excessive allocation exceeding MMIO window boundary
    let mut out_of_bounds = 0x1FFF_F000;
    let addr_fail = allocate_mmio(&mut out_of_bounds, 0x10000);
    assert_eq!(addr_fail, None);
}

fn test_probe_bar0_success() {
    MockPciBus::reset();
    MockInterruptController::reset();
    // Setup 32-bit Memory BAR with 8KB (0x2000) size
    MockPciBus::set_reg32(0, 1, 0, PCI_BAR0_REG, PCI_BAR_MEMORY_TYPE_32);
    MockPciBus::set_bar0_size(0, 1, 0, 0x2000);

    let bar = probe_bar0(0, 1, 0).expect("BAR0 probing failed");
    assert_eq!(bar.size, 0x2000);
    assert_eq!(bar.flags, PCI_BAR_MEMORY_TYPE_32);
}

fn test_probe_bar0_rejects_io_bar() {
    MockPciBus::reset();
    MockInterruptController::reset();
    // Set bit 0 (IO Space indicator)
    MockPciBus::set_reg32(0, 1, 0, PCI_BAR0_REG, PCI_BAR_IO_SPACE);
    assert!(probe_bar0(0, 1, 0).is_none());
}

fn test_probe_bar0_rejects_unsupported_64bit_bar() {
    MockPciBus::reset();
    MockInterruptController::reset();
    // Set bits 2:1 to 0b10 (64-bit BAR type)
    let bar_64bit = 0b10 << 1;
    MockPciBus::set_reg32(0, 1, 0, PCI_BAR0_REG, bar_64bit);
    assert!(probe_bar0(0, 1, 0).is_none());
}

fn test_configure_bar0() {
    MockPciBus::reset();
    MockInterruptController::reset();
    MockPciBus::set_reg32(0, 2, 0, PCI_BAR0_REG, PCI_BAR_MEMORY_TYPE_32);
    MockPciBus::set_bar0_size(0, 2, 0, 0x4000); // 16KB BAR

    let mut next_mmio = 0x1000_0000;
    let assigned_addr = configure_bar0(0, 2, 0, &mut next_mmio);

    assert_eq!(assigned_addr, Some(0x1000_0000));
    assert_eq!(next_mmio, 0x1000_4000);
    assert_eq!(
        MockPciBus::pci_read32(0, 2, 0, PCI_BAR0_REG),
        0x1000_0000 | PCI_BAR_MEMORY_TYPE_32
    );
}

fn test_map_all_bars() {
    MockPciBus::reset();
    MockInterruptController::reset();

    // Header Type 0x00 (Standard device, non-multifunction)
    MockPciBus::set_reg32(0, 1, 0, PCI_HEADER_TYPE_REG, 0x00);
    MockPciBus::set_reg32(0, 1, 0, PCI_BAR0_REG, PCI_BAR_MEMORY_TYPE_32);
    MockPciBus::set_bar0_size(0, 1, 0, 0x1000);

    let mut next_mmio = 0x1000_0000;
    assert!(map_all_bars(0, 1, 0, &mut next_mmio).is_ok());

    // Header Type 0x01 (PCI-to-PCI Bridge - unsupported)
    MockPciBus::set_reg32(0, 2, 0, PCI_HEADER_TYPE_REG, 0x01);
    assert!(map_all_bars(0, 2, 0, &mut next_mmio).is_err());
}

fn test_register_pci_device() {
    MockPciBus::reset();
    MockInterruptController::reset();
    register_pci_device(0, 5, 2);

    unsafe {
        assert_eq!(PCI_DEVICES.len(), 1);
        assert_eq!(PCI_DEVICES[0].bus, 0);
        assert_eq!(PCI_DEVICES[0].dev, 5);
        assert_eq!(PCI_DEVICES[0].func, 2);
    }
}

pub fn run_tests() {
    test_align_up();
    test_pci_function_is_present();
    test_pci_enable_disable_device();
    test_pci_enable_bus_mastering();
    test_pci_enable_interrupt();
    test_allocate_mmio();
    test_probe_bar0_success();
    test_probe_bar0_rejects_io_bar();
    test_probe_bar0_rejects_unsupported_64bit_bar();
    test_configure_bar0();
    test_map_all_bars();
    test_register_pci_device();
}
