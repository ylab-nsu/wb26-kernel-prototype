pub mod cards;
mod sdhci_command;
mod sdhci_interface;

use sdhci_command::*;
use sdhci_interface::*;
use crate::pci::{ pci_enable_device, pci_enable_interrupt, pci_enable_bus_mastering, PCI_BAR0_REG };
use crate::arch::traits::TargetPciBus;
use crate::arch::PciBus;

// Bus-specific data about device attached to it
#[derive(Copy, Clone)]
struct SdhciSlot {
    slot_num: u8,
    sdhci_interface: SdhciInterface,
}

// Slots amount is limited by PCI BARs amount, there are six of them
pub struct Sdhci {
    pci_bus: u8,
    pci_dev: u8,
    pci_func: u8,
    slots: [Option<SdhciSlot>; 6],
}

#[derive(Debug)]
pub enum SdhciError {
    Timeout,
    Interrupted(SdhciErrorInterruptStatus),
    Incompatible,
}

impl SdhciSlot {
    fn new(slot_num: u8, sdhci_interface: SdhciInterface) -> Self {
        SdhciSlot { slot_num, sdhci_interface }
    }

    fn init(&mut self) -> Result<(), SdhciError> {
        self.soft_reset()?;

        // Set timeout to (timeout_clock_freq * 2^27)
        self.sdhci_interface.set_timeout_control(0b1110);

        // If there is card on startup - attach it
        if self.is_card_presented() {
            self.attach_card()?;
        }

        Ok(())
    }

    // Identify card and spawn a driver for it
    fn attach_card(&mut self) -> Result<(), SdhciError> {
        info!("Attaching card to sdhci slot {}", self.slot_num);

        info!("Setting slot {} to identification state", self.slot_num);
        self.power_up(SdhciOperatingVoltage::V3_3)?;
        self.set_sdclock_frequency(400)?;

        // TODO: Probe for card type instead of treating it as a emmc unconditionally
        // TODO: Register attached card in kernel device subsystem
        // TODO: Replace with driver matching after drivers subsystem implementation
        cards::emmc::driver_task();
    }

    fn power_up(&mut self, voltage: SdhciOperatingVoltage) -> Result<(), SdhciError> {
        info!("Slot {} target voltage: {}",
            self.slot_num,
            match voltage {
                SdhciOperatingVoltage::V1_8 => "1.8V",
                SdhciOperatingVoltage::V3_0 => "3.0V",
                SdhciOperatingVoltage::V3_3 => "3.3V",
            });


        let caps = self.sdhci_interface.capabilities();
        let voltage_supported = match voltage {
            SdhciOperatingVoltage::V1_8 => caps.voltage_1_8_support(),
            SdhciOperatingVoltage::V3_0 => caps.voltage_3_0_support(),
            SdhciOperatingVoltage::V3_3 => caps.voltage_3_3_support(),
        };

        if !voltage_supported {
            return Err(SdhciError::Incompatible);
        }

        let power_ctl = self.sdhci_interface.power_control();

        // Power off
        self.sdhci_interface.set_power_control(power_ctl.with_sd_bus_power(false));

        // Power on with changed voltage
        self.sdhci_interface.set_power_control(power_ctl
            .with_sd_bus_power(true)
            .with_voltage(voltage));

        Ok(())
    }

    // Enables sdclock on given frequency
    // freq in KHz
    fn set_sdclock_frequency(&mut self, freq: u32) -> Result<(), SdhciError> {
        info!("Slot {} target freq: {freq}KHz", self.slot_num);

        let base_freq = self.sdhci_interface.capabilities().base_clock_freq();

        info!("Slot {} base freq: {base_freq}MHz", self.slot_num);

        // Divide by 2, because actual divisor is (divisor * 2)
        let divisor = (base_freq as u32 * 1000 / freq / 2).next_power_of_two() as u8;

        info!("Slot {} frequency divisor: {}", self.slot_num, divisor as u32 * 2);
        
        let clock_ctl = self.sdhci_interface.clock_control();

        let clock_ctl = clock_ctl.with_sd_clock_enable(false);
        self.sdhci_interface.set_clock_control(clock_ctl);

        let clock_ctl = clock_ctl
            .with_freq_divisor(divisor)
            .with_internal_clock_enable(true);
        self.sdhci_interface.set_clock_control(clock_ctl);

        let mut clock_ctl = clock_ctl;
        let mut stable = false;
        for _ in 0..1000 {
            clock_ctl = self.sdhci_interface.clock_control();
            if clock_ctl.internal_clock_stable() {
                stable = true;
                break;
            }
        }

        match stable {
            true => {
                self.sdhci_interface.set_clock_control(clock_ctl.with_sd_clock_enable(true));
                Ok(())
            },
            false => Err(SdhciError::Timeout),
        }
    }

    fn is_card_presented(&self) -> bool {
        let present_state = self.sdhci_interface.present_state();

        present_state.card_inserted()
    }

    fn soft_reset(&mut self) -> Result<(), SdhciError> {
        let soft_reset_reg = self.sdhci_interface.soft_reset_control();
        self.sdhci_interface.set_soft_reset_control(soft_reset_reg.with_for_all(true));

        // TODO: Setup timer for timing out
        // Wait for the controller to clear the reset bit
        let mut reset = false;
        for _ in 0..1000 {
            if !self.sdhci_interface.soft_reset_control().for_all() {
                reset = true;
                break;
            }
        }

        match reset {
            true => Ok(()),
            false => Err(SdhciError::Timeout),
        }
    }

    fn dump_capabilities(&self) {
        let caps = self.sdhci_interface.capabilities();

        info!("Slot {} capabilities:", self.slot_num);

        info!("\tTimeout clock freq: {}{}", 
            caps.timeout_clock_freq(),
            match caps.timeout_clock_unit() {
                SdhciTimeoutClockUnit::KHz => "KHz",
                SdhciTimeoutClockUnit::MHz => "MHz",
            });
        info!("\tBase clock freq: {}MHz", caps.base_clock_freq());
        info!("\tMax block length: {}",
            match caps.max_block_length() {
                SdhciMaxBlockLength::B512 => "512B",
                SdhciMaxBlockLength::B1024 => "1024B",
                SdhciMaxBlockLength::B2048 => "2048B",
            });
        info!("\tADMA2 support: {}", caps.adma2_support());
        info!("\tHigh speed support: {}", caps.high_speed_support());
        info!("\tSDMA support: {}", caps.sdma_support());
        info!("\tSuspend resume support: {}", caps.suspend_resume_support());
        info!("\t3.3V support: {}", caps.voltage_3_3_support());
        info!("\t3.0V support: {}", caps.voltage_3_0_support());
        info!("\t1.8V support: {}", caps.voltage_1_8_support());
        info!("\t64 bit systm bus support: {}", caps.system_bus_64());
    }

    fn enable_all_interrupt_statuses(&mut self) {
        let normal_interrupt_status_enable = SdhciNormalInterruptStatusEnable::new()
            .with_command_complete(true)
            .with_transfer_complete(true)
            .with_block_gap_event(true)
            .with_dma_interrupt(true)
            .with_buffer_write_ready(true)
            .with_buffer_read_ready(true)
            .with_card_insertion(true)
            .with_card_removal(true)
            .with_card_interrupt(true);
        self.sdhci_interface.set_normal_interrupt_status_enable(normal_interrupt_status_enable);

        let error_interrupt_status_enable = SdhciErrorInterruptStatusEnable::new()
            .with_command_timeout(true)
            .with_command_crc(true)
            .with_command_end_bit(true)
            .with_command_index(true)
            .with_data_timeout(true)
            .with_data_crc(true)
            .with_data_end_bit(true)
            .with_current_limit(true)
            .with_auto_cmd12(true)
            .with_adma(true);
        self.sdhci_interface.set_error_interrupt_status_enable(error_interrupt_status_enable);
    }

    fn enable_all_error_signals(&mut self) {
        let error_interrupt_signal_enable = SdhciErrorInterruptSignalEnable::new()
            .with_command_timeout(true)
            .with_command_crc(true)
            .with_command_end_bit(true)
            .with_command_index(true)
            .with_data_timeout(true)
            .with_data_crc(true)
            .with_data_end_bit(true)
            .with_current_limit(true)
            .with_auto_cmd12(true)
            .with_adma(true);
        self.sdhci_interface.set_error_interrupt_signal_enable(error_interrupt_signal_enable);
    }

    fn clear_all_interrupt_statuses(&mut self) {
        let normal_interrupt_status = SdhciNormalInterruptStatus::new()
            .with_command_complete(true)
            .with_transfer_complete(true)
            .with_block_gap_event(true)
            .with_dma_interrupt(true)
            .with_buffer_write_ready(true)
            .with_buffer_read_ready(true)
            .with_card_insertion(true)
            .with_card_removal(true);
        self.sdhci_interface.set_normal_interrupt_status(normal_interrupt_status);

        let error_interrupt_status = SdhciErrorInterruptStatus::new()
            .with_command_timeout(true)
            .with_command_crc(true)
            .with_command_end_bit(true)
            .with_command_index(true)
            .with_data_timeout(true)
            .with_data_crc(true)
            .with_data_end_bit(true)
            .with_current_limit(true)
            .with_auto_cmd12(true)
            .with_adma(true);
        self.sdhci_interface.set_error_interrupt_status(error_interrupt_status);
    }

    fn send_command(&mut self, command: &SdhciCommand) -> Result<(), SdhciError> {
        // Wait till CMD and DAT (if needed) lines are free
        // TODO: Set up some timer for timeout
        let mut lines_free = false;
        for _ in 0..1000 {
            let present_state = self.sdhci_interface.present_state();
            if !present_state.command_inhibit_cmd() && (!command.command_desc.data_present() || !present_state.command_inhibit_dat()) {
                lines_free = true;
                break;
            }
        }
        if !lines_free {
            return Err(SdhciError::Timeout);
        }

        info!("Issuing CMD{} on slot {}", command.command_desc.index(), self.slot_num);

        match &command.kind {
            SdhciCommandKind::NonDatCommand => {},
            SdhciCommandKind::DataTransfer(transfer) => {
                self.sdhci_interface.set_block_size(transfer.block_size);
                self.sdhci_interface.set_block_count(transfer.block_count);
                self.sdhci_interface.set_transfer_mode(transfer.transfer_mode);
                match &transfer.data_transfer_kind {
                    SdhciDataTransferKind::CpuTransfer => {},
                    SdhciDataTransferKind::DmaTransfer(transfer) => {
                        self.sdhci_interface.set_sdma_address(transfer.sdma_address);
                    },
                }
            },
        }

        self.sdhci_interface.set_argument(command.argument);
        self.sdhci_interface.set_command(command.command_desc);

        Ok(())
    }

    fn read_from_buffer(&self, buff: &mut [u8]) -> Result<(), SdhciError> {
        // TODO: Set up some timer for timeout
        let mut buffer_read_ready = false;
        for _ in 0..1000 {
            if self.sdhci_interface.present_state().buffer_read_enable() {
                buffer_read_ready = true;
                break;
            }
        }
        if !buffer_read_ready {
            return Err(SdhciError::Timeout);
        }

        info!("Reading from slot {}", self.slot_num);

        for i in 0..(buff.len() / 4) {
            let word = self.sdhci_interface.buffer_data_port();
            let word = word.to_le_bytes();
            for j in 0..word.len() {
                buff[i * 4 + j] = word[j];
            }
        }

        Ok(())
    }

    fn wait_for_command_complete(&mut self) -> Result<(), SdhciError> {
        loop {
            let normal_int_status = self.sdhci_interface.normal_interrupt_status();

            if normal_int_status.command_complete() {
                let normal_int_status = SdhciNormalInterruptStatus::new().with_command_complete(true);
                self.sdhci_interface.set_normal_interrupt_status(normal_int_status);

                return Ok(());
            }
            else if normal_int_status.error_interrupt() {
                let error_int_status = self.sdhci_interface.error_interrupt_status();
                self.sdhci_interface.set_error_interrupt_status(error_int_status);

                return Err(SdhciError::Interrupted(error_int_status));
            }
        }
    }

    fn wait_for_transfer_complete(&mut self) -> Result<(), SdhciError> {
        loop {
            let normal_int_status = self.sdhci_interface.normal_interrupt_status();

            if normal_int_status.error_interrupt() {
                let error_int_status = self.sdhci_interface.error_interrupt_status();
                self.sdhci_interface.set_error_interrupt_status(error_int_status);

                return Err(SdhciError::Interrupted(error_int_status));
            }
            else if normal_int_status.transfer_complete() {
                let normal_int_status = SdhciNormalInterruptStatus::new().with_transfer_complete(true);
                self.sdhci_interface.set_normal_interrupt_status(normal_int_status);

                return Ok(());
            }
        }
    }

    fn read_response_normalized(&self) -> u128 {
        let mut out: u128 = 0;

        let parts = self.sdhci_interface.response();
        for i in 0..4 {
            out |= (parts[i] as u128) << (i * 32);
        }

        out
    }
}

impl Sdhci {
    fn new(bus: u8, dev: u8, func: u8) -> Self {
        // TODO: Check all BARs, not just first
        let sdhci_base = PciBus::pci_read32(bus, dev, func, PCI_BAR0_REG) & !(0x0F as u32);

        Sdhci { 
            pci_bus: bus,
            pci_dev: dev,
            pci_func: func,
            slots: [Some(SdhciSlot::new(0, SdhciInterface::from_raw_pointer(sdhci_base as *mut SdhciSlotRegisters))), None, None, None, None, None] 
        }
    }

    fn init(&mut self) {
        pci_enable_bus_mastering(self.pci_bus, self.pci_dev, self.pci_func);
        pci_enable_interrupt(self.pci_bus, self.pci_dev, self.pci_func);
        pci_enable_device(self.pci_bus, self.pci_dev, self.pci_func);

        // Init all slots
        for (i, slot) in self.slots.iter_mut().enumerate() {
            if let Some(slot) = slot {
                match slot.init() {
                    Ok(_) => {
                        info!("SDHCI: Slot {} initialized successfully", i);
                    },
                    Err(err) => {
                        error!("SDHCI: Slot {} fault: {:?}", i, err);
                        // TODO: Invalidate slot
                    }
                }
            }
        }
    }
}

pub static mut HOST: Option<Sdhci> = None;

pub extern "C" fn driver_task() -> ! {
    unsafe {
        // TODO: Get this structure from bus-driver using some id of device, to which driver is attached
        let pci_info = &crate::pci::PCI_DEVICES[0];

        HOST = Some(Sdhci::new(pci_info.bus, pci_info.dev, pci_info.func));
        HOST.as_mut().unwrap().init();

    }

    loop { }
}
