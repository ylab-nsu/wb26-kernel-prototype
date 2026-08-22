use bitfield_struct::{ bitfield, bitenum };
use core::ptr::{read_volatile, write_volatile};
use crate::pci::{ pci_enable_device, PCI_BAR0_REG };
use crate::arch::traits::TargetPciBus;
use crate::arch::PciBus;

pub static mut sdhci_pci_addr: Option<(u8, u8, u8)> = None;

#[bitfield(u16)]
struct BlockSize {
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
enum TransferingDirection {
    #[fallback]
    WRITE = 0,
    READ = 1,
}

#[bitfield(u16)]
struct TransferMode {
    dma_enable: bool,
    block_count_enable: bool,
    auto_cmd12_enable: bool,
    __: bool,
    #[bits(1)]
    data_transfer_direction_select: TransferingDirection,
    multi_block_select: bool,

    #[bits(10)]
    __: usize,
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
enum ResponseType {
    #[fallback]
    NO_RESPONSE = 0,
    RESPONSE_LEN_136 = 1,
    RESPONSE_LEN_48 = 2,
    RESPONSE_LEN_48_CBAR = 3, // Response lenght 48 check Busy after response
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
enum CommandType {
    #[fallback]
    NORMAL = 0,
    SUSPEND = 1,
    RESUME = 2,
    ABORT = 3,
}

#[bitfield(u16)]
struct Command {
    #[bits(2)]
    response_type: ResponseType,
    #[bits(1)]
    __: usize,
    crc_enable: bool,
    index_check_enable: bool,
    data_present: bool,
    #[bits(2)]
    command_type: CommandType,
    #[bits(6)]
    index: u8,
    #[bits(2)]
    __: usize
}

#[bitfield(u32)]
struct PresentState {
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
enum TransferWidth {
    #[fallback]
    ONE_BIT = 0,
    FOUR_BIT = 1,
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
enum DmaMode {
    #[fallback]
    SDMA = 0,
    ADMA2_32 = 2,
    ADMA2_64 = 3,
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
enum CardDetection {
    #[fallback]
    NORMAL = 0,
    TEST = 1,
}

#[bitfield(u8)]
struct HostControl {
    led_enable: bool,
    #[bits(1)]
    data_transfer_width: TransferWidth,
    high_speed_enable: bool,
    #[bits(2)]
    dma_mode: DmaMode,
    #[bits(1)]
    __: usize,
    card_detect_test_level: bool,
    #[bits(1)]
    card_detect_signal: CardDetection // NOTE: Disable interrupts before changing
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
enum Voltage {
    #[fallback]
    V1_8 = 5,
    V3_0 = 6,
    V3_3 = 7,
}

#[bitfield(u8)]
struct PowerControl {
    sd_bus_power: bool,
    #[bits(3)]
    voltage: Voltage,
    #[bits(4)]
    __: usize,
}

#[bitfield(u8)]
struct BlockGapControl {
    stop_at_block_gap_req: bool,
    continue_request: bool,
    read_wait_control: bool,
    interrupt_at_block_gap: bool,
    #[bits(4)]
    __: usize,
}

#[bitfield(u8)]
struct WakeupControl {
    on_card_interrupt: bool,
    on_card_insertion: bool,
    on_card_removal: bool,
    #[bits(5)]
    __: usize,
}

#[bitfield(u16)]
struct ClockControl {
    internal_clock_enable: bool,
    internal_clock_stable: bool,
    sd_clock_enable: bool,
    #[bits(5)]
    __: usize,
    freq_divisor: u8 // NOTE: Actual divisor used is (freq_divisor * 2)
}

#[bitfield(u8)]
struct SoftResetControl {
    for_all: bool,
    for_cmd_line: bool,
    for_dat_line: bool,
    #[bits(5)]
    __: usize,
}

#[bitfield(u16)]
struct NormalInterruptStatus {
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
struct ErrorInterruptStatus {
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
struct NormalInterruptStatusEnable {
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
struct ErrorInterruptStatusEnable {
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
struct NormalInterruptSignalEnable {
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
struct ErrorInterruptSignalEnable {
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
struct AutoCmd12ErrorStatus {
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
enum TimeoutClockUnit {
    #[fallback]
    KHz = 0,
    MHz= 1,
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
enum MaxBlockLength {
    #[fallback]
    B512 = 0,
    B1024 = 1,
    B2048 = 2,
}

#[bitfield(u64)]
struct Capabilities {
    #[bits(6)]
    timeout_clock_freq: u8,
    #[bits(1)]
    __: usize,
    #[bits(1)]
    timeout_clock_unit: TimeoutClockUnit,
    #[bits(6)]
    base_clock_freq: u8,
    #[bits(2)]
    __: usize,
    #[bits(2)]
    max_block_length: MaxBlockLength,
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
struct MaximumCurrentCapabilities {
    max_current_3_3V: u8,
    max_current_3_0V: u8,
    max_current_1_8V: u8,
    #[bits(40)]
    __: usize,
}

#[bitfield(u16)]
struct SlotInterruptStatus {
    per_slot_interrupt_signal: u8,
    __: u8,
}

#[bitfield(u16)]
struct HostControllerVersion {
    specification_version: u8,
    vendor_version: u8,
}

#[bitfield(u16)]
struct ForceEventForErrorInterrupt {
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
struct ForceEventForCmd12Error {
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
enum AdmaErrorState {
    #[fallback]
    ST_STOP = 0,
    ST_FDS = 1,
    ST_TFR = 3,
}

#[bitfield(u8)]
struct AdmaErrorStatus {
    #[bits(2)]
    adma_error_state: AdmaErrorState,
    adma_length_mismatch: bool,
    #[bits(5)]
    __: usize,
}

#[repr(C, packed(1))]
struct SlotRegisters {
    sdma_address: u32,
    block_size: BlockSize,
    block_count: u16,
    argument: u32,
    transfer_mode: TransferMode,
    command: Command,
    response: u128,
    buffer_data_port: u32,
    present_state: PresentState,
    host_control: HostControl,
    power_control: PowerControl,
    block_gap_control: BlockGapControl,
    wakeup_control: WakeupControl,
    clock_control: ClockControl,
    timeout_control: u8, // NOTE: Refer to spec
    soft_reset_control: SoftResetControl,
    normal_interrupt_status: NormalInterruptStatus,
    error_interrupt_status: ErrorInterruptStatus,
    normal_interrupt_status_enable: NormalInterruptStatusEnable,
    error_interrupt_status_enable: ErrorInterruptStatusEnable,
    normal_interrupt_signal_enable: NormalInterruptSignalEnable,
    error_interrupt_signal_enable: ErrorInterruptSignalEnable,
    auto_cmd12_error_status: AutoCmd12ErrorStatus,
    __1: u16,
    capabilities: Capabilities,
    maximum_current_capabilities: MaximumCurrentCapabilities,
    force_event_for_error_interrupt: ForceEventForErrorInterrupt,
    force_event_for_cmd12_error: ForceEventForCmd12Error,
    adma_error_status: AdmaErrorStatus,
    __2: u8,
    __3: u16,
    adma_address: u64,
    __4: [u32; 39],
    host_controller_version: HostControllerVersion,
    slot_interrupt_status: SlotInterruptStatus,
}

// Slots amount is limited by PCI BARs amount, there are six of them
struct Sdhci {
    slots: [Option<*mut SlotRegisters>; 6],
}

impl Sdhci {
    fn is_card_presented(&self, slot: u8) -> Result<bool, &'static str> {
        if let Some(slot_regs) = self.slots[slot as usize] {
            unsafe {
                let present_state = &raw mut ((*slot_regs).present_state);
                let present_state = read_volatile(present_state);

                Ok(present_state.card_inserted())
            }
        }
        else {
            Err("There is no such slot")
        }
    }

    fn soft_reset(&self, slot: u8) -> Result<(), &'static str> {
        if let Some(slot_regs) = self.slots[slot as usize] {
            unsafe {
                let soft_reset_reg = &raw mut ((*slot_regs).soft_reset_control);
                let soft_reset_reg_state = read_volatile(soft_reset_reg);
                write_volatile(soft_reset_reg, soft_reset_reg_state.with_for_all(true));

                // TODO: Setup interrupt or something
                // Wait for the controller to clear the reset bit
                for _ in 0..1000 {
                    if !read_volatile(soft_reset_reg).for_all() {
                        break;
                    }
                }

                Ok(())
            }
        }
        else {
            Err("There is no such slot")
        }
    }

    fn dump_capabilities(&self, slot: u8) -> Result<(), &'static str> {
        if let Some(slot_regs) = self.slots[slot as usize] {
            unsafe {
                let caps = &raw mut ((*slot_regs).capabilities);
                let caps = read_volatile(caps);

                info!("Timeout clock freq: {}{}", 
                    caps.timeout_clock_freq(),
                    match caps.timeout_clock_unit() {
                        TimeoutClockUnit::KHz => "KHz",
                        TimeoutClockUnit::MHz => "MHz",
                    });
                info!("Base clock freq: {}MHz", caps.base_clock_freq());
                info!("Max block length: {}",
                    match caps.max_block_length() {
                        MaxBlockLength::B512 => "512B",
                        MaxBlockLength::B1024 => "1024B",
                        MaxBlockLength::B2048 => "2048B",
                    });
                info!("ADMA2 support: {}", caps.adma2_support());
                info!("High speed support: {}", caps.high_speed_support());
                info!("SDMA support: {}", caps.sdma_support());
                info!("Suspend resume support: {}", caps.suspend_resume_support());
                info!("3.3V support: {}", caps.voltage_3_3_support());
                info!("3.0V support: {}", caps.voltage_3_0_support());
                info!("1.8V support: {}", caps.voltage_1_8_support());
                info!("64 bit systm bus support: {}", caps.system_bus_64());
                
                Ok(())
            }
        }
        else {
            Err("There is no such slot")
        }
    }

    /*
    fn set_normal_speed_frequency(&self, slot: u8) -> Result<(), &'static str> {
        if let Some(slot_regs) = self.slots[slot as usize] {
            unsafe {
                // Read base clock frequency
                let caps = &raw mut ((*slot_regs).capabilities);
                let base_clock_freq = read_volatile(caps).base_clock_freq(); // MHz

                // For normal speed we need 25 MHz frequency
                let divisor = (base_clock_freq / 25).next_power_of_two() as u8;
            }
        }
        else {
            Err("There is no such slot")
        }
    }
    */
}

pub extern "C" fn driver_task() -> ! {

    unsafe {
        if let Some((bus, dev, func)) = sdhci_pci_addr {
            debug!("Size of reg set: {}", core::mem::size_of::<SlotRegisters>());

            pci_enable_device(bus, dev, func);

            // TODO: Check all BARs, not just first
            let sdhci_base = PciBus::pci_read32(bus, dev, func, PCI_BAR0_REG) & !(0x0F as u32);
            let sdhci_data = Sdhci { slots: [Some(sdhci_base as *mut SlotRegisters), None, None, None, None, None] };

            // Soft reset sdhci as 
            if let Err(err) = sdhci_data.soft_reset(0) {
                error!("SDHCI Driver fault: {}", err);
            }

            match sdhci_data.is_card_presented(0) {
                Ok(card_inserted) => info!("Card Inserted: {}", card_inserted),
                Err(err) => error!("SDHCI Driver fault: {}", err),
            }

            if let Err(err) = sdhci_data.dump_capabilities(0) {
                error!("SDHCI Driver fault: {}", err);
            }

            // TODO: Set frequency divisor
            // TODO: Set timeout_control

            // TODO: Enable slot (enable clock, power, ...)

        }
        else {
            error!("No sdhci address on pci bus on sdhci bus driver start");
        }
    }

    loop { }
}
