use bitfield_struct::{ bitfield, bitenum };
use core::ptr::{read_volatile, write_volatile};
use crate::pci::{ pci_enable_device, PCI_BAR0_REG };
use crate::arch::traits::TargetPciBus;
use crate::arch::PciBus;

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
    Write = 0,
    Read = 1,
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
    NoResponse = 0,
    ResponseLen136 = 1,
    ResponseLen48 = 2,
    ResponseLen48CheckBusy = 3, // Response lenght 48 check Busy after response
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
enum CommandType {
    #[fallback]
    Normal = 0,
    Suspend = 1,
    Resume = 2,
    Abort = 3,
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
    OneBit = 0,
    FourBit = 1,
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
    Nornal = 0,
    Test = 1,
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
    max_current_3_3_v: u8,
    max_current_3_0_v: u8,
    max_current_1_8_v: u8,
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
    StStop = 0,
    StFds = 1,
    StTfr = 3,
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

// Bus-specific data about device attached to it
struct Slot {
    slot_num: u8,
    regs: *mut SlotRegisters,
}

// Slots amount is limited by PCI BARs amount, there are six of them
struct Sdhci {
    pci_bus: u8,
    pci_dev: u8,
    pci_func: u8,
    slots: [Option<Slot>; 6],
}

impl Slot {
    fn new(slot_num: u8, regs: *mut SlotRegisters) -> Self {
        Slot { slot_num, regs }
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
        self.power_up(Voltage::V3_3)?;
        self.set_sdclock_frequency(400)?;

        // TODO: Probe for card type

        Ok(())
    }

    fn power_up(&self, voltage: Voltage) -> Result<(), &'static str> {
        unsafe {
            info!("Slot {} target voltage: {}",
                self.slot_num,
                match voltage {
                    Voltage::V1_8 => "1.8V",
                    Voltage::V3_0 => "3.0V",
                    Voltage::V3_3 => "3.3V",
                });


            let caps = &raw mut (*self.regs).capabilities;
            let voltage_supported = match voltage {
                Voltage::V1_8 => read_volatile(caps).voltage_1_8_support(),
                Voltage::V3_0 => read_volatile(caps).voltage_3_0_support(),
                Voltage::V3_3 => read_volatile(caps).voltage_3_3_support(),
            };

            if !voltage_supported {
                return Err("Slot doesnt support target voltage");
            }

            let power_ctl_ptr = &raw mut (*self.regs).power_control;
            let power_ctl = read_volatile(power_ctl_ptr);
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
                    TimeoutClockUnit::KHz => "KHz",
                    TimeoutClockUnit::MHz => "MHz",
                });
            info!("\tBase clock freq: {}MHz", caps.base_clock_freq());
            info!("\tMax block length: {}",
                match caps.max_block_length() {
                    MaxBlockLength::B512 => "512B",
                    MaxBlockLength::B1024 => "1024B",
                    MaxBlockLength::B2048 => "2048B",
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

impl Sdhci {
    fn new(bus: u8, dev: u8, func: u8) -> Self {
        // TODO: Check all BARs, not just first
        let sdhci_base = PciBus::pci_read32(bus, dev, func, PCI_BAR0_REG) & !(0x0F as u32);

        Sdhci { 
            pci_bus: bus,
            pci_dev: dev,
            pci_func: func,
            slots: [Some(Slot::new(0, sdhci_base as *mut SlotRegisters)), None, None, None, None, None] 
        }
    }

    fn init(&mut self) {
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

pub extern "C" fn driver_task() -> ! {

    unsafe {
        // TODO: Get this structure from bus-driver using some id of device, to which driver is attached
        let pci_info = &crate::pci::PCI_DEVICES[0];

        let mut host = Sdhci::new(pci_info.bus, pci_info.dev, pci_info.func);
        host.init();

        match &host.slots[0] {
            Some(slot) => {
                slot.dump_capabilities();

                // TODO: Enable slot (enable clock, power, ...)
            },
            None => error!("There is no 0th slot in sdhci"),
        }
    }

    loop { }
}
