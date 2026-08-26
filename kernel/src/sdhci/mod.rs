pub mod cards;
mod sdhci_command;

use sdhci_command::*;
use bitfield_struct::{ bitfield, bitenum };
use core::ptr::{read_volatile, write_volatile};
use crate::pci::{ pci_enable_device, pci_enable_interrupt, pci_enable_bus_mastering, PCI_BAR0_REG };
use crate::arch::traits::TargetPciBus;
use crate::arch::PciBus;

#[bitfield(u16)]
struct SdhciBlockSize {
    #[bits(12)]
    transfer_block_size: u16,
    #[bits(3)]
    host_sdma_buffer_boundary: u8,
    #[bits(1)]
    __: usize,
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
enum SdhciTransferingDirection {
    #[fallback]
    Write = 0,
    Read = 1,
}

#[bitfield(u16)]
struct SdhciTransferMode {
    dma_enable: bool,
    block_count_enable: bool,
    auto_cmd12_enable: bool,
    __: bool,
    #[bits(1)]
    data_transfer_direction_select: SdhciTransferingDirection,
    multi_block_select: bool,

    #[bits(10)]
    __: usize,
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
enum SdhciResponseType {
    #[fallback]
    NoResponse = 0,
    ResponseLen136 = 1,
    ResponseLen48 = 2,
    ResponseLen48CheckBusy = 3, // Response lenght 48 check Busy after response
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
enum SdhciCommandType {
    #[fallback]
    Normal = 0,
    Suspend = 1,
    Resume = 2,
    Abort = 3,
}

#[bitfield(u16)]
struct SdhciCommandDesc {
    #[bits(2)]
    response_type: SdhciResponseType,
    #[bits(1)]
    __: usize,
    crc_enable: bool,
    index_check_enable: bool,
    data_present: bool,
    #[bits(2)]
    command_type: SdhciCommandType,
    #[bits(6)]
    index: u8,
    #[bits(2)]
    __: usize
}

#[bitfield(u32)]
struct SdhciPresentState {
    command_inhibit_cmd: bool,
    command_inhibit_dat: bool,
    dat_line_active: bool,
    #[bits(5)]
    __: usize,
    write_transfer_active: bool,
    read_transfer_active: bool,
    buffer_write_enable: bool,
    buffer_read_enable: bool,
    #[bits(4)]
    __: usize,
    card_inserted: bool,
    card_state_stable: bool,
    card_detect_pin_level: bool,
    write_protect_switch_pin_level: bool, // NOTE: 0 - write protected, 1 - write enabled
    dat0_line_level: bool,
    dat1_line_level: bool,
    dat2_line_level: bool,
    dat3_line_level: bool,
    cmd_line_level: bool,
    #[bits(7)]
    __: usize,
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
enum SdhciTransferWidth {
    #[fallback]
    OneBit = 0,
    FourBit = 1,
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
enum SdhciDmaMode {
    #[fallback]
    SDMA = 0,
    ADMA2_32 = 2,
    ADMA2_64 = 3,
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
enum SdhciCardDetection {
    #[fallback]
    Nornal = 0,
    Test = 1,
}

#[bitfield(u8)]
struct SdhciHostControl {
    led_enable: bool,
    #[bits(1)]
    data_transfer_width: SdhciTransferWidth,
    high_speed_enable: bool,
    #[bits(2)]
    dma_mode: SdhciDmaMode,
    #[bits(1)]
    __: usize,
    card_detect_test_level: bool,
    #[bits(1)]
    card_detect_signal: SdhciCardDetection // NOTE: Disable interrupts before changing
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
enum SdhciOperatingVoltage {
    #[fallback]
    V1_8 = 5,
    V3_0 = 6,
    V3_3 = 7,
}

#[bitfield(u8)]
struct SdhciPowerControl {
    sd_bus_power: bool,
    #[bits(3)]
    voltage: SdhciOperatingVoltage,
    #[bits(4)]
    __: usize,
}

#[bitfield(u8)]
struct SdhciBlockGapControl {
    stop_at_block_gap_req: bool,
    continue_request: bool,
    read_wait_control: bool,
    interrupt_at_block_gap: bool,
    #[bits(4)]
    __: usize,
}

#[bitfield(u8)]
struct SdhciWakeupControl {
    on_card_interrupt: bool,
    on_card_insertion: bool,
    on_card_removal: bool,
    #[bits(5)]
    __: usize,
}

#[bitfield(u16)]
struct SdhciClockControl {
    internal_clock_enable: bool,
    internal_clock_stable: bool,
    sd_clock_enable: bool,
    #[bits(5)]
    __: usize,
    freq_divisor: u8 // NOTE: Actual divisor used is (freq_divisor * 2)
}

#[bitfield(u8)]
struct SdhciSoftResetControl {
    for_all: bool,
    for_cmd_line: bool,
    for_dat_line: bool,
    #[bits(5)]
    __: usize,
}

#[bitfield(u16)]
struct SdhciNormalInterruptStatus {
    command_complete: bool,
    transfer_complete: bool,
    block_gap_event: bool,
    dma_interrupt: bool,
    buffer_write_ready: bool,
    buffer_read_ready: bool,
    card_insertion: bool,
    card_removal: bool,
    card_interrupt: bool,
    #[bits(6)]
    __: usize,
    error_interrupt: bool,
}

#[bitfield(u16)]
struct SdhciErrorInterruptStatus {
    command_timeout: bool,
    command_crc: bool,
    command_end_bit: bool,
    command_index: bool,
    data_timeout: bool,
    data_crc: bool,
    data_end_bit: bool,
    current_limit: bool,
    auto_cmd12: bool,
    adma: bool,
    #[bits(2)]
    __: usize,
    #[bits(4)]
    vendor_specific: u8,
}

#[bitfield(u16)]
struct SdhciNormalInterruptStatusEnable {
    command_complete: bool,
    transfer_complete: bool,
    block_gap_event: bool,
    dma_interrupt: bool,
    buffer_write_ready: bool,
    buffer_read_ready: bool,
    card_insertion: bool,
    card_removal: bool,
    card_interrupt: bool,
    #[bits(7)]
    __: usize,
}

#[bitfield(u16)]
struct SdhciErrorInterruptStatusEnable {
    command_timeout: bool,
    command_crc: bool,
    command_end_bit: bool,
    command_index: bool,
    data_timeout: bool,
    data_crc: bool,
    data_end_bit: bool,
    current_limit: bool,
    auto_cmd12: bool,
    adma: bool,
    #[bits(2)]
    __: usize,
    #[bits(4)]
    vendor_specific: u8,
}

#[bitfield(u16)]
struct SdhciNormalInterruptSignalEnable {
    command_complete: bool,
    transfer_complete: bool,
    block_gap_event: bool,
    dma_interrupt: bool,
    buffer_write_ready: bool,
    buffer_read_ready: bool,
    card_insertion: bool,
    card_removal: bool,
    card_interrupt: bool,
    #[bits(7)]
    __: usize,
}

#[bitfield(u16)]
struct SdhciErrorInterruptSignalEnable {
    command_timeout: bool,
    command_crc: bool,
    command_end_bit: bool,
    command_index: bool,
    data_timeout: bool,
    data_crc: bool,
    data_end_bit: bool,
    current_limit: bool,
    auto_cmd12: bool,
    adma: bool,
    #[bits(2)]
    __: usize,
    #[bits(4)]
    vendor_specific: u8,
}

#[bitfield(u16)]
struct SdhciAutoCmd12ErrorStatus {
    not_executed: bool,
    timeout: bool,
    crc: bool,
    end_bit: bool,
    index: bool,
    #[bits(2)]
    __: usize,
    command_not_issued_by_error: bool,
    #[bits(8)]
    __: usize,
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
enum SdhciTimeoutClockUnit {
    #[fallback]
    KHz = 0,
    MHz= 1,
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
enum SdhciMaxBlockLength {
    #[fallback]
    B512 = 0,
    B1024 = 1,
    B2048 = 2,
}

#[bitfield(u64)]
struct SdhciCapabilities {
    #[bits(6)]
    timeout_clock_freq: u8,
    #[bits(1)]
    __: usize,
    #[bits(1)]
    timeout_clock_unit: SdhciTimeoutClockUnit,
    #[bits(6)]
    base_clock_freq: u8,
    #[bits(2)]
    __: usize,
    #[bits(2)]
    max_block_length: SdhciMaxBlockLength,
    #[bits(1)]
    __: usize,
    adma2_support: bool,
    #[bits(1)]
    __: usize,
    high_speed_support: bool,
    sdma_support: bool,
    suspend_resume_support: bool,
    voltage_3_3_support: bool,
    voltage_3_0_support: bool,
    voltage_1_8_support: bool,
    #[bits(1)]
    __: usize,
    system_bus_64: bool,
    #[bits(35)]
    __: usize,
}

// NOTE: Actual max current is (max_currenr_n_mV * 4)mA
#[bitfield(u64)]
struct SdhciMaximumCurrentCapabilities {
    max_current_3_3_v: u8,
    max_current_3_0_v: u8,
    max_current_1_8_v: u8,
    #[bits(40)]
    __: usize,
}

#[bitfield(u16)]
struct SdhciSlotInterruptStatus {
    per_slot_interrupt_signal: u8,
    __: u8,
}

#[bitfield(u16)]
struct SdhciVersion {
    specification_version: u8,
    vendor_version: u8,
}

#[bitfield(u16)]
struct SdhciForceEventForErrorInterrupt {
    command_timeout: bool,
    command_crc: bool,
    command_end_bit: bool,
    command_index: bool,
    data_timeout: bool,
    data_crc: bool,
    data_end_bit: bool,
    current_limit: bool,
    auto_cmd12: bool,
    adma: bool,
    #[bits(2)]
    __: usize,
    #[bits(4)]
    vendor_specific: u8,
}

#[bitfield(u16)]
struct SdhciForceEventForCmd12Error {
    not_executed: bool,
    timeout: bool,
    crc: bool,
    end_bit: bool,
    index: bool,
    #[bits(2)]
    __: usize,
    command_not_issued_by_error: bool,
    #[bits(8)]
    __: usize,
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
enum SdhciAdmaErrorState {
    #[fallback]
    StStop = 0,
    StFds = 1,
    StTfr = 3,
}

#[bitfield(u8)]
struct SdhciAdmaErrorStatus {
    #[bits(2)]
    adma_error_state: SdhciAdmaErrorState,
    adma_length_mismatch: bool,
    #[bits(5)]
    __: usize,
}

#[repr(C, packed(1))]
struct SdhciSlotRegisters {
    sdma_address: u32,
    block_size: SdhciBlockSize,
    block_count: u16,
    argument: u32,
    transfer_mode: SdhciTransferMode,
    command: SdhciCommandDesc,
    response: [u32; 4],
    buffer_data_port: u32,
    present_state: SdhciPresentState,
    host_control: SdhciHostControl,
    power_control: SdhciPowerControl,
    block_gap_control: SdhciBlockGapControl,
    wakeup_control: SdhciWakeupControl,
    clock_control: SdhciClockControl,
    timeout_control: u8, // NOTE: Refer to spec
    soft_reset_control: SdhciSoftResetControl,
    normal_interrupt_status: SdhciNormalInterruptStatus,
    error_interrupt_status: SdhciErrorInterruptStatus,
    normal_interrupt_status_enable: SdhciNormalInterruptStatusEnable,
    error_interrupt_status_enable: SdhciErrorInterruptStatusEnable,
    normal_interrupt_signal_enable: SdhciNormalInterruptSignalEnable,
    error_interrupt_signal_enable: SdhciErrorInterruptSignalEnable,
    auto_cmd12_error_status: SdhciAutoCmd12ErrorStatus,
    __1: u16,
    capabilities: SdhciCapabilities,
    maximum_current_capabilities: SdhciMaximumCurrentCapabilities,
    force_event_for_cmd12_error: SdhciForceEventForCmd12Error,
    force_event_for_error_interrupt: SdhciForceEventForErrorInterrupt,
    adma_error_status: SdhciAdmaErrorStatus,
    __2: u8,
    __3: u16,
    adma_address: u64,
    __4: [u32; 39],
    slot_interrupt_status: SdhciSlotInterruptStatus,
    host_controller_version: SdhciVersion,
}

// Bus-specific data about device attached to it
#[derive(Copy, Clone)]
struct SdhciSlot {
    slot_num: u8,
    regs: *mut SdhciSlotRegisters,
}

// Slots amount is limited by PCI BARs amount, there are six of them
struct Sdhci {
    pci_bus: u8,
    pci_dev: u8,
    pci_func: u8,
    slots: [Option<SdhciSlot>; 6],
}

#[derive(Debug)]
enum SdhciError {
    Timeout,
    Interrupted(SdhciErrorInterruptStatus),
}

impl SdhciSlot {
    fn new(slot_num: u8, regs: *mut SdhciSlotRegisters) -> Self {
        SdhciSlot { slot_num, regs }
    }

    fn init(&mut self) -> Result<(), &'static str> {
        self.soft_reset()?;

        // Set timeout to (timeout_clock_freq * 2^27)
        unsafe {
            let timeout_ctl = &raw mut (*self.regs).timeout_control;
            write_volatile(timeout_ctl, 0b1110);
        }

        // If there is card on startup - attach it
        if self.is_card_presented() {
            self.attach_card()?;
        }

        Ok(())
    }

    // Identify card and spawn a driver for it
    fn attach_card(&mut self) -> Result<(), &'static str> {
        info!("Attaching card to sdhci slot {}", self.slot_num);

        info!("Setting slot {} to identification state", self.slot_num);
        self.power_up(SdhciOperatingVoltage::V3_3)?;
        self.set_sdclock_frequency(400)?;

        // TODO: Probe for card type instead of treating it as a emmc unconditionally
        // TODO: Register attached card in kernel device subsystem
        // TODO: Replace with driver matching after drivers subsystem implementation
        cards::emmc::driver_task();

        Ok(())
    }

    fn power_up(&self, voltage: SdhciOperatingVoltage) -> Result<(), &'static str> {
        unsafe {
            info!("Slot {} target voltage: {}",
                self.slot_num,
                match voltage {
                    SdhciOperatingVoltage::V1_8 => "1.8V",
                    SdhciOperatingVoltage::V3_0 => "3.0V",
                    SdhciOperatingVoltage::V3_3 => "3.3V",
                });


            let caps = &raw mut (*self.regs).capabilities;
            let voltage_supported = match voltage {
                SdhciOperatingVoltage::V1_8 => read_volatile(caps).voltage_1_8_support(),
                SdhciOperatingVoltage::V3_0 => read_volatile(caps).voltage_3_0_support(),
                SdhciOperatingVoltage::V3_3 => read_volatile(caps).voltage_3_3_support(),
            };

            if !voltage_supported {
                return Err("Slot doesnt support target voltage");
            }

            let power_ctl_ptr = &raw mut (*self.regs).power_control;
            let power_ctl = read_volatile(power_ctl_ptr);

            // Power off
            write_volatile(power_ctl_ptr, power_ctl.with_sd_bus_power(false));

            // Power on with changed voltage
            write_volatile(power_ctl_ptr, power_ctl
                .with_sd_bus_power(true)
                .with_voltage(voltage));

            Ok(())
        }
    }

    // Enables sdclock on given frequency
    // freq in KHz
    fn set_sdclock_frequency(&self, freq: u32) -> Result<(), &'static str> {
        unsafe {
            info!("Slot {} target freq: {freq}KHz", self.slot_num);

            let caps = &raw mut (*self.regs).capabilities;
            let base_freq = read_volatile(caps).base_clock_freq();

            info!("Slot {} base freq: {base_freq}MHz", self.slot_num);

            // Divide by 2, because actual divisor is (divisor * 2)
            let divisor = (base_freq as u32 * 1000 / freq / 2).next_power_of_two() as u8;

            info!("Slot {} frequency divisor: {}", self.slot_num, divisor as u32 * 2);
            
            let clock_ctl_ptr = &raw mut (*self.regs).clock_control;
            let mut clock_ctl = read_volatile(clock_ctl_ptr);

            clock_ctl = clock_ctl.with_sd_clock_enable(false);
            write_volatile(clock_ctl_ptr, clock_ctl);

            clock_ctl = clock_ctl
                .with_freq_divisor(divisor)
                .with_internal_clock_enable(true);
            write_volatile(clock_ctl_ptr, clock_ctl);

            let mut stable = false;
            for _ in 0..1000 {
                clock_ctl = read_volatile(clock_ctl_ptr);
                if clock_ctl.internal_clock_stable() {
                    stable = true;
                    break;
                }
            }

            match stable {
                true => {
                    write_volatile(clock_ctl_ptr, clock_ctl.with_sd_clock_enable(true));
                    Ok(())
                },
                false => Err("Clock stabilization timeout"),
            }
        }
    }

    fn is_card_presented(&self) -> bool {
        unsafe {
            let present_state = &raw mut (*self.regs).present_state;
            let present_state = read_volatile(present_state);

            present_state.card_inserted()
        }
    }

    fn soft_reset(&self) -> Result<(), &'static str> {
        unsafe {
            let soft_reset_reg = &raw mut (*self.regs).soft_reset_control;
            let soft_reset_reg_state = read_volatile(soft_reset_reg);
            write_volatile(soft_reset_reg, soft_reset_reg_state.with_for_all(true));

            // TODO: Setup timer for timing out
            // Wait for the controller to clear the reset bit
            let mut reset = false;
            for _ in 0..1000 {
                if !read_volatile(soft_reset_reg).for_all() {
                    reset = true;
                    break;
                }
            }

            match reset {
                true => Ok(()),
                false => Err("Reset timeout"),
            }
        }
    }

    fn dump_capabilities(&self) {
        unsafe {
            let caps = &raw mut (*self.regs).capabilities;
            let caps = read_volatile(caps);

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
    }

    fn enable_all_interrupt_statuses(&self) {
        unsafe {
            let normal_interrupt_status_enable = &raw mut (*self.regs).normal_interrupt_status_enable;
            let normal_interrupt_status_enable_val = SdhciNormalInterruptStatusEnable::new()
                .with_command_complete(true)
                .with_transfer_complete(true)
                .with_block_gap_event(true)
                .with_dma_interrupt(true)
                .with_buffer_write_ready(true)
                .with_buffer_read_ready(true)
                .with_card_insertion(true)
                .with_card_removal(true)
                .with_card_interrupt(true);
            write_volatile(normal_interrupt_status_enable, normal_interrupt_status_enable_val);

            let error_interrupt_status_enable = &raw mut (*self.regs).error_interrupt_status_enable;
            let error_interrupt_status_enable_val = SdhciErrorInterruptStatusEnable::new()
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
            write_volatile(error_interrupt_status_enable, error_interrupt_status_enable_val);
        }
    }

    fn enable_all_error_signals(&self) {
        unsafe {
            let error_interrupt_signal_enable = &raw mut (*self.regs).error_interrupt_signal_enable;
            let error_interrupt_signal_enable_val = SdhciErrorInterruptSignalEnable::new()
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
            write_volatile(error_interrupt_signal_enable, error_interrupt_signal_enable_val);
        }
    }

    fn clear_all_interrupt_statuses(&self) {
        unsafe {
            let normal_interrupt_status = &raw mut (*self.regs).normal_interrupt_status;
            let normal_interrupt_status_val = SdhciNormalInterruptStatus::new()
                .with_command_complete(true)
                .with_transfer_complete(true)
                .with_block_gap_event(true)
                .with_dma_interrupt(true)
                .with_buffer_write_ready(true)
                .with_buffer_read_ready(true)
                .with_card_insertion(true)
                .with_card_removal(true);
            write_volatile(normal_interrupt_status, normal_interrupt_status_val);

            let error_interrupt_status = &raw mut (*self.regs).error_interrupt_status;
            let error_interrupt_status_val = SdhciErrorInterruptStatus::new()
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
            write_volatile(error_interrupt_status, error_interrupt_status_val);
        }
    }

    fn send_command(&self, command: &SdhciCommand) -> Result<(), SdhciError> {
        // Wait till CMD and DAT (if needed) lines are free
        unsafe {
            let present_state = &raw mut (*self.regs).present_state;

            // TODO: Set up some timer for timeout
            let mut lines_free = false;
            for _ in 0..1000 {
                let present_state_val = read_volatile(present_state);
                if !present_state_val.command_inhibit_cmd() && (!command.command_desc.data_present() || !present_state_val.command_inhibit_dat()) {
                    lines_free = true;
                    break;
                }
            }
            if !lines_free {
                return Err(SdhciError::Timeout);
            }

            info!("Issuing CMD{} on slot {}", command.command_desc.index(), self.slot_num);

            let sdma_address_ptr = &raw mut (*self.regs).sdma_address;
            let block_size_ptr = &raw mut (*self.regs).block_size;
            let block_count_ptr = &raw mut (*self.regs).block_count;
            let argument_ptr = &raw mut (*self.regs).argument;
            let transfer_mode_ptr = &raw mut (*self.regs).transfer_mode;
            let command_ptr = &raw mut (*self.regs).command;

            match &command.kind {
                SdhciCommandKind::NonDatCommand => {},
                SdhciCommandKind::DataTransfer(transfer) => {
                    write_volatile(block_size_ptr, transfer.block_size);
                    write_volatile(block_count_ptr, transfer.block_count);
                    write_volatile(transfer_mode_ptr, transfer.transfer_mode);
                    match &transfer.data_transfer_kind {
                        SdhciDataTransferKind::CpuTransfer => {},
                        SdhciDataTransferKind::DmaTransfer(transfer) => {
                            write_volatile(sdma_address_ptr, transfer.sdma_address);
                        },
                    }
                },
            }

            write_volatile(argument_ptr, command.argument);
            write_volatile(command_ptr, command.command_desc);

            Ok(())
        }
    }

    fn read_from_buffer(&self, buff: &mut [u8]) -> Result<(), SdhciError> {
        unsafe {
            let present_state = &raw mut (*self.regs).present_state;

            // TODO: Set up some timer for timeout
            let mut buffer_read_ready = false;
            for _ in 0..1000 {
                if read_volatile(present_state).buffer_read_enable() {
                    buffer_read_ready = true;
                    break;
                }
            }
            if !buffer_read_ready {
                return Err(SdhciError::Timeout);
            }

            info!("Reading from slot {}", self.slot_num);

            let buffer_reg = &raw mut (*self.regs).buffer_data_port;
            for i in 0..(buff.len() / 4) {
                let word = read_volatile(buffer_reg);
                let word = word.to_le_bytes();
                for j in 0..word.len() {
                    buff[i * 4 + j] = word[j];
                }
            }

            Ok(())
        }
    }

    fn wait_for_command_complete(&self) -> Result<(), SdhciError> {
        unsafe {
            let normal_int_status = &raw mut (*self.regs).normal_interrupt_status;
            let error_int_status = &raw mut (*self.regs).error_interrupt_status;

            loop {
                let normal_int_status_val = read_volatile(normal_int_status);

                if normal_int_status_val.command_complete() {
                    let normal_int_status_val = SdhciNormalInterruptStatus::new().with_command_complete(true);
                    write_volatile(normal_int_status, normal_int_status_val);

                    return Ok(());
                }
                else if normal_int_status_val.error_interrupt() {
                    let error_int_status_val = read_volatile(error_int_status);
                    write_volatile(error_int_status, error_int_status_val);

                    return Err(SdhciError::Interrupted(error_int_status_val));
                }
            }
        }
    }

    fn wait_for_transfer_complete(&self) -> Result<(), SdhciError> {
        unsafe {
            let normal_int_status = &raw mut (*self.regs).normal_interrupt_status;
            let error_int_status = &raw mut (*self.regs).error_interrupt_status;

            loop {
                let normal_int_status_val = read_volatile(normal_int_status);

                if normal_int_status_val.error_interrupt() {
                    let error_int_status_val = read_volatile(error_int_status);
                    write_volatile(error_int_status, error_int_status_val);

                    return Err(SdhciError::Interrupted(error_int_status_val));
                }
                else if normal_int_status_val.transfer_complete() {
                    let normal_int_status_val = SdhciNormalInterruptStatus::new().with_transfer_complete(true);
                    write_volatile(normal_int_status, normal_int_status_val);

                    return Ok(());
                }
            }
        }
    }

    fn read_response_normalized(&self) -> u128 {
        unsafe {
            let mut out: u128 = 0;

            for i in 0..4 {
                let part = &raw const (*self.regs).response[i];
                let part = read_volatile(part) as u128;
                out |= part << (i * 32);
            }

            out
        }
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
            slots: [Some(SdhciSlot::new(0, sdhci_base as *mut SdhciSlotRegisters)), None, None, None, None, None] 
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
                        error!("SDHCI: Slot {} fault: {}", i, err);
                        // TODO: Invalidate slot
                    }
                }
            }
        }
    }
}

pub static mut host: Option<Sdhci> = None;

pub extern "C" fn driver_task() -> ! {
    unsafe {
        // TODO: Get this structure from bus-driver using some id of device, to which driver is attached
        let pci_info = &crate::pci::PCI_DEVICES[0];

        host = Some(Sdhci::new(pci_info.bus, pci_info.dev, pci_info.func));
        host.as_mut().unwrap().init();

    }

    loop { }
}
