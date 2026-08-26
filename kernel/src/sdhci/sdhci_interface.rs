use bitfield_struct::{ bitfield, bitenum };
use core::ptr::{read_volatile, write_volatile};

#[bitfield(u16)]
pub struct SdhciBlockSize {
    #[bits(12)]
    pub transfer_block_size: u16,
    #[bits(3)]
    pub host_sdma_buffer_boundary: u8,
    #[bits(1)]
    __: usize,
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
pub enum SdhciTransferingDirection {
    #[fallback]
    Write = 0,
    Read = 1,
}

#[bitfield(u16)]
pub struct SdhciTransferMode {
    pub dma_enable: bool,
    pub block_count_enable: bool,
    pub auto_cmd12_enable: bool,
    __: bool,
    #[bits(1)]
    pub data_transfer_direction_select: SdhciTransferingDirection,
    pub multi_block_select: bool,

    #[bits(10)]
    __: usize,
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
pub enum SdhciResponseType {
    #[fallback]
    NoResponse = 0,
    ResponseLen136 = 1,
    ResponseLen48 = 2,
    ResponseLen48CheckBusy = 3, // Response lenght 48 check Busy after response
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
pub enum SdhciCommandType {
    #[fallback]
    Normal = 0,
    Suspend = 1,
    Resume = 2,
    Abort = 3,
}

#[bitfield(u16)]
pub struct SdhciCommandDesc {
    #[bits(2)]
    pub response_type: SdhciResponseType,
    #[bits(1)]
    __: usize,
    pub crc_enable: bool,
    pub index_check_enable: bool,
    pub data_present: bool,
    #[bits(2)]
    pub command_type: SdhciCommandType,
    #[bits(6)]
    pub index: u8,
    #[bits(2)]
    __: usize
}

#[bitfield(u32)]
pub struct SdhciPresentState {
    pub command_inhibit_cmd: bool,
    pub command_inhibit_dat: bool,
    pub dat_line_active: bool,
    #[bits(5)]
    __: usize,
    pub write_transfer_active: bool,
    pub read_transfer_active: bool,
    pub buffer_write_enable: bool,
    pub buffer_read_enable: bool,
    #[bits(4)]
    __: usize,
    pub card_inserted: bool,
    pub card_state_stable: bool,
    pub card_detect_pin_level: bool,
    pub write_protect_switch_pin_level: bool, // NOTE: 0 - write protected, 1 - write enabled
    pub dat0_line_level: bool,
    pub dat1_line_level: bool,
    pub dat2_line_level: bool,
    pub dat3_line_level: bool,
    pub cmd_line_level: bool,
    #[bits(7)]
    __: usize,
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
pub enum SdhciTransferWidth {
    #[fallback]
    OneBit = 0,
    FourBit = 1,
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
pub enum SdhciDmaMode {
    #[fallback]
    SDMA = 0,
    ADMA2_32 = 2,
    ADMA2_64 = 3,
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
pub enum SdhciCardDetection {
    #[fallback]
    Nornal = 0,
    Test = 1,
}

#[bitfield(u8)]
pub struct SdhciHostControl {
    pub led_enable: bool,
    #[bits(1)]
    pub data_transfer_width: SdhciTransferWidth,
    pub high_speed_enable: bool,
    #[bits(2)]
    pub dma_mode: SdhciDmaMode,
    #[bits(1)]
    __: usize,
    pub card_detect_test_level: bool,
    #[bits(1)]
    pub card_detect_signal: SdhciCardDetection // NOTE: Disable interrupts before changing
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
pub enum SdhciOperatingVoltage {
    #[fallback]
    V1_8 = 5,
    V3_0 = 6,
    V3_3 = 7,
}

#[bitfield(u8)]
pub struct SdhciPowerControl {
    pub sd_bus_power: bool,
    #[bits(3)]
    pub voltage: SdhciOperatingVoltage,
    #[bits(4)]
    __: usize,
}

#[bitfield(u8)]
pub struct SdhciBlockGapControl {
    pub stop_at_block_gap_req: bool,
    pub continue_request: bool,
    pub read_wait_control: bool,
    pub interrupt_at_block_gap: bool,
    #[bits(4)]
    __: usize,
}

#[bitfield(u8)]
pub struct SdhciWakeupControl {
    pub on_card_interrupt: bool,
    pub on_card_insertion: bool,
    pub on_card_removal: bool,
    #[bits(5)]
    __: usize,
}

#[bitfield(u16)]
pub struct SdhciClockControl {
    pub internal_clock_enable: bool,
    pub internal_clock_stable: bool,
    pub sd_clock_enable: bool,
    #[bits(5)]
    __: usize,
    pub freq_divisor: u8 // NOTE: Actual divisor used is (freq_divisor * 2)
}

#[bitfield(u8)]
pub struct SdhciSoftResetControl {
    pub for_all: bool,
    pub for_cmd_line: bool,
    pub for_dat_line: bool,
    #[bits(5)]
    __: usize,
}

#[bitfield(u16)]
pub struct SdhciNormalInterruptStatus {
    pub command_complete: bool,
    pub transfer_complete: bool,
    pub block_gap_event: bool,
    pub dma_interrupt: bool,
    pub buffer_write_ready: bool,
    pub buffer_read_ready: bool,
    pub card_insertion: bool,
    pub card_removal: bool,
    pub card_interrupt: bool,
    #[bits(6)]
    __: usize,
    pub error_interrupt: bool,
}

#[bitfield(u16)]
pub struct SdhciErrorInterruptStatus {
    pub command_timeout: bool,
    pub command_crc: bool,
    pub command_end_bit: bool,
    pub command_index: bool,
    pub data_timeout: bool,
    pub data_crc: bool,
    pub data_end_bit: bool,
    pub current_limit: bool,
    pub auto_cmd12: bool,
    pub adma: bool,
    #[bits(2)]
    __: usize,
    #[bits(4)]
    pub vendor_specific: u8,
}

#[bitfield(u16)]
pub struct SdhciNormalInterruptStatusEnable {
    pub command_complete: bool,
    pub transfer_complete: bool,
    pub block_gap_event: bool,
    pub dma_interrupt: bool,
    pub buffer_write_ready: bool,
    pub buffer_read_ready: bool,
    pub card_insertion: bool,
    pub card_removal: bool,
    pub card_interrupt: bool,
    #[bits(7)]
    __: usize,
}

#[bitfield(u16)]
pub struct SdhciErrorInterruptStatusEnable {
    pub command_timeout: bool,
    pub command_crc: bool,
    pub command_end_bit: bool,
    pub command_index: bool,
    pub data_timeout: bool,
    pub data_crc: bool,
    pub data_end_bit: bool,
    pub current_limit: bool,
    pub auto_cmd12: bool,
    pub adma: bool,
    #[bits(2)]
    __: usize,
    #[bits(4)]
    pub vendor_specific: u8,
}

#[bitfield(u16)]
pub struct SdhciNormalInterruptSignalEnable {
    pub command_complete: bool,
    pub transfer_complete: bool,
    pub block_gap_event: bool,
    pub dma_interrupt: bool,
    pub buffer_write_ready: bool,
    pub buffer_read_ready: bool,
    pub card_insertion: bool,
    pub card_removal: bool,
    pub card_interrupt: bool,
    #[bits(7)]
    __: usize,
}

#[bitfield(u16)]
pub struct SdhciErrorInterruptSignalEnable {
    pub command_timeout: bool,
    pub command_crc: bool,
    pub command_end_bit: bool,
    pub command_index: bool,
    pub data_timeout: bool,
    pub data_crc: bool,
    pub data_end_bit: bool,
    pub current_limit: bool,
    pub auto_cmd12: bool,
    pub adma: bool,
    #[bits(2)]
    __: usize,
    #[bits(4)]
    pub vendor_specific: u8,
}

#[bitfield(u16)]
pub struct SdhciAutoCmd12ErrorStatus {
    pub not_executed: bool,
    pub timeout: bool,
    pub crc: bool,
    pub end_bit: bool,
    pub index: bool,
    #[bits(2)]
    __: usize,
    pub command_not_issued_by_error: bool,
    #[bits(8)]
    __: usize,
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
pub enum SdhciTimeoutClockUnit {
    #[fallback]
    KHz = 0,
    MHz= 1,
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
pub enum SdhciMaxBlockLength {
    #[fallback]
    B512 = 0,
    B1024 = 1,
    B2048 = 2,
}

#[bitfield(u64)]
pub struct SdhciCapabilities {
    #[bits(6)]
    pub timeout_clock_freq: u8,
    #[bits(1)]
    __: usize,
    #[bits(1)]
    pub timeout_clock_unit: SdhciTimeoutClockUnit,
    #[bits(6)]
    pub base_clock_freq: u8,
    #[bits(2)]
    __: usize,
    #[bits(2)]
    pub max_block_length: SdhciMaxBlockLength,
    #[bits(1)]
    __: usize,
    pub adma2_support: bool,
    #[bits(1)]
    __: usize,
    pub high_speed_support: bool,
    pub sdma_support: bool,
    pub suspend_resume_support: bool,
    pub voltage_3_3_support: bool,
    pub voltage_3_0_support: bool,
    pub voltage_1_8_support: bool,
    #[bits(1)]
    __: usize,
    pub system_bus_64: bool,
    #[bits(35)]
    __: usize,
}

// NOTE: Actual max current is (max_currenr_n_mV * 4)mA
#[bitfield(u64)]
pub struct SdhciMaximumCurrentCapabilities {
    pub max_current_3_3_v: u8,
    pub max_current_3_0_v: u8,
    pub max_current_1_8_v: u8,
    #[bits(40)]
    __: usize,
}

#[bitfield(u16)]
pub struct SdhciSlotInterruptStatus {
    pub per_slot_interrupt_signal: u8,
    __: u8,
}

#[bitfield(u16)]
pub struct SdhciVersion {
    pub specification_version: u8,
    pub vendor_version: u8,
}

#[bitfield(u16)]
pub struct SdhciForceEventForErrorInterrupt {
    pub command_timeout: bool,
    pub command_crc: bool,
    pub command_end_bit: bool,
    pub command_index: bool,
    pub data_timeout: bool,
    pub data_crc: bool,
    pub data_end_bit: bool,
    pub current_limit: bool,
    pub auto_cmd12: bool,
    pub adma: bool,
    #[bits(2)]
    __: usize,
    #[bits(4)]
    pub vendor_specific: u8,
}

#[bitfield(u16)]
pub struct SdhciForceEventForCmd12Error {
    pub not_executed: bool,
    pub timeout: bool,
    pub crc: bool,
    pub end_bit: bool,
    pub index: bool,
    #[bits(2)]
    __: usize,
    pub command_not_issued_by_error: bool,
    #[bits(8)]
    __: usize,
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
pub enum SdhciAdmaErrorState {
    #[fallback]
    StStop = 0,
    StFds = 1,
    StTfr = 3,
}

#[bitfield(u8)]
pub struct SdhciAdmaErrorStatus {
    #[bits(2)]
    pub adma_error_state: SdhciAdmaErrorState,
    pub adma_length_mismatch: bool,
    #[bits(5)]
    __: usize,
}

#[repr(C, packed(1))]
pub struct SdhciSlotRegisters {
    pub sdma_address: u32,
    pub block_size: SdhciBlockSize,
    pub block_count: u16,
    pub argument: u32,
    pub transfer_mode: SdhciTransferMode,
    pub command: SdhciCommandDesc,
    pub response: [u32; 4],
    pub buffer_data_port: u32,
    pub present_state: SdhciPresentState,
    pub host_control: SdhciHostControl,
    pub power_control: SdhciPowerControl,
    pub block_gap_control: SdhciBlockGapControl,
    pub wakeup_control: SdhciWakeupControl,
    pub clock_control: SdhciClockControl,
    pub timeout_control: u8, // NOTE: Refer to spec
    pub soft_reset_control: SdhciSoftResetControl,
    pub normal_interrupt_status: SdhciNormalInterruptStatus,
    pub error_interrupt_status: SdhciErrorInterruptStatus,
    pub normal_interrupt_status_enable: SdhciNormalInterruptStatusEnable,
    pub error_interrupt_status_enable: SdhciErrorInterruptStatusEnable,
    pub normal_interrupt_signal_enable: SdhciNormalInterruptSignalEnable,
    pub error_interrupt_signal_enable: SdhciErrorInterruptSignalEnable,
    pub auto_cmd12_error_status: SdhciAutoCmd12ErrorStatus,
    __1: u16,
    pub capabilities: SdhciCapabilities,
    pub maximum_current_capabilities: SdhciMaximumCurrentCapabilities,
    pub force_event_for_cmd12_error: SdhciForceEventForCmd12Error,
    pub force_event_for_error_interrupt: SdhciForceEventForErrorInterrupt,
    pub adma_error_status: SdhciAdmaErrorStatus,
    __2: u8,
    __3: u16,
    pub adma_address: u64,
    __4: [u32; 39],
    pub slot_interrupt_status: SdhciSlotInterruptStatus,
    pub host_controller_version: SdhciVersion,
}

#[derive(Copy, Clone)]
pub struct SdhciInterface {
    regs: *mut SdhciSlotRegisters,
}

impl SdhciInterface {
    pub fn from_raw_pointer(slot_regs: *mut SdhciSlotRegisters) -> Self {
        SdhciInterface { regs: slot_regs }
    }

    pub fn sdma_address(&self) -> u32 {
        unsafe {
            let ptr = &raw const (*self.regs).sdma_address;
            read_volatile(ptr)
        }
    }
    pub fn block_size(&self) -> SdhciBlockSize {
        unsafe {
            let ptr = &raw const (*self.regs).block_size;
            read_volatile(ptr)
        }
    }
    pub fn block_count(&self) -> u16 {
        unsafe {
            let ptr = &raw const (*self.regs).block_count;
            read_volatile(ptr)
        }
    }
    pub fn argument(&self) -> u32 {
        unsafe {
            let ptr = &raw const (*self.regs).argument;
            read_volatile(ptr)
        }
    }
    pub fn transfer_mode(&self) -> SdhciTransferMode {
        unsafe {
            let ptr = &raw const (*self.regs).transfer_mode;
            read_volatile(ptr)
        }
    }
    pub fn command(&self) -> SdhciCommandDesc {
        unsafe {
            let ptr = &raw const (*self.regs).command;
            read_volatile(ptr)
        }
    }
    pub fn response(&self) -> [u32; 4] {
        unsafe {
            let ptr = &raw const (*self.regs).response;
            read_volatile(ptr)
        }
    }
    pub fn buffer_data_port(&self) -> u32 {
        unsafe {
            let ptr = &raw const (*self.regs).buffer_data_port;
            read_volatile(ptr)
        }
    }
    pub fn present_state(&self) -> SdhciPresentState {
        unsafe {
            let ptr = &raw const (*self.regs).present_state;
            read_volatile(ptr)
        }
    }
    pub fn host_control(&self) -> SdhciHostControl {
        unsafe {
            let ptr = &raw const (*self.regs).host_control;
            read_volatile(ptr)
        }
    }
    pub fn power_control(&self) -> SdhciPowerControl {
        unsafe {
            let ptr = &raw const (*self.regs).power_control;
            read_volatile(ptr)
        }
    }
    pub fn block_gap_control(&self) -> SdhciBlockGapControl {
        unsafe {
            let ptr = &raw const (*self.regs).block_gap_control;
            read_volatile(ptr)
        }
    }
    pub fn wakeup_control(&self) -> SdhciWakeupControl {
        unsafe {
            let ptr = &raw const (*self.regs).wakeup_control;
            read_volatile(ptr)
        }
    }
    pub fn clock_control(&self) -> SdhciClockControl {
        unsafe {
            let ptr = &raw const (*self.regs).clock_control;
            read_volatile(ptr)
        }
    }
    pub fn timeout_control(&self) -> u8 {
        unsafe {
            let ptr = &raw const (*self.regs).timeout_control;
            read_volatile(ptr)
        }
    }
    pub fn soft_reset_control(&self) -> SdhciSoftResetControl {
        unsafe {
            let ptr = &raw const (*self.regs).soft_reset_control;
            read_volatile(ptr)
        }
    }
    pub fn normal_interrupt_status(&self) -> SdhciNormalInterruptStatus {
        unsafe {
            let ptr = &raw const (*self.regs).normal_interrupt_status;
            read_volatile(ptr)
        }
    }
    pub fn error_interrupt_status(&self) -> SdhciErrorInterruptStatus {
        unsafe {
            let ptr = &raw const (*self.regs).error_interrupt_status;
            read_volatile(ptr)
        }
    }
    pub fn normal_interrupt_status_enable(&self) -> SdhciNormalInterruptStatusEnable {
        unsafe {
            let ptr = &raw const (*self.regs).normal_interrupt_status_enable;
            read_volatile(ptr)
        }
    }
    pub fn error_interrupt_status_enable(&self) -> SdhciErrorInterruptStatusEnable {
        unsafe {
            let ptr = &raw const (*self.regs).error_interrupt_status_enable;
            read_volatile(ptr)
        }
    }
    pub fn normal_interrupt_signal_enable(&self) -> SdhciNormalInterruptSignalEnable {
        unsafe {
            let ptr = &raw const (*self.regs).normal_interrupt_signal_enable;
            read_volatile(ptr)
        }
    }
    pub fn error_interrupt_signal_enable(&self) -> SdhciErrorInterruptSignalEnable {
        unsafe {
            let ptr = &raw const (*self.regs).error_interrupt_signal_enable;
            read_volatile(ptr)
        }
    }
    pub fn auto_cmd12_error_status(&self) -> SdhciAutoCmd12ErrorStatus {
        unsafe {
            let ptr = &raw const (*self.regs).auto_cmd12_error_status;
            read_volatile(ptr)
        }
    }
    pub fn capabilities(&self) -> SdhciCapabilities {
        unsafe {
            let ptr = &raw const (*self.regs).capabilities;
            read_volatile(ptr)
        }
    }
    pub fn maximum_current_capabilities(&self) -> SdhciMaximumCurrentCapabilities {
        unsafe {
            let ptr = &raw const (*self.regs).maximum_current_capabilities;
            read_volatile(ptr)
        }
    }
    pub fn force_event_for_cmd12_error(&self) -> SdhciForceEventForCmd12Error {
        unsafe {
            let ptr = &raw const (*self.regs).force_event_for_cmd12_error;
            read_volatile(ptr)
        }
    }
    pub fn force_event_for_error_interrupt(&self) -> SdhciForceEventForErrorInterrupt {
        unsafe {
            let ptr = &raw const (*self.regs).force_event_for_error_interrupt;
            read_volatile(ptr)
        }
    }
    pub fn adma_error_status(&self) -> SdhciAdmaErrorStatus {
        unsafe {
            let ptr = &raw const (*self.regs).adma_error_status;
            read_volatile(ptr)
        }
    }
    pub fn adma_address(&self) -> u64 {
        unsafe {
            let ptr = &raw const (*self.regs).adma_address;
            read_volatile(ptr)
        }
    }
    pub fn slot_interrupt_status(&self) -> SdhciSlotInterruptStatus {
        unsafe {
            let ptr = &raw const (*self.regs).slot_interrupt_status;
            read_volatile(ptr)
        }
    }
    pub fn host_controller_version(&self) -> SdhciVersion {
        unsafe {
            let ptr = &raw const (*self.regs).host_controller_version;
            read_volatile(ptr)
        }
    }

    pub fn set_sdma_address(&mut self, val: u32) {
        unsafe {
            let ptr = &raw mut (*self.regs).sdma_address;
            write_volatile(ptr,val);
        }
    }
    pub fn set_block_size(&mut self, val: SdhciBlockSize) {
        unsafe {
            let ptr = &raw mut (*self.regs).block_size;
            write_volatile(ptr,val);
        }
    }
    pub fn set_block_count(&mut self, val: u16) {
        unsafe {
            let ptr = &raw mut (*self.regs).block_count;
            write_volatile(ptr,val);
        }
    }
    pub fn set_argument(&mut self, val: u32) {
        unsafe {
            let ptr = &raw mut (*self.regs).argument;
            write_volatile(ptr,val);
        }
    }
    pub fn set_transfer_mode(&mut self, val: SdhciTransferMode) {
        unsafe {
            let ptr = &raw mut (*self.regs).transfer_mode;
            write_volatile(ptr,val);
        }
    }
    pub fn set_command(&mut self, val: SdhciCommandDesc) {
        unsafe {
            let ptr = &raw mut (*self.regs).command;
            write_volatile(ptr,val);
        }
    }
    pub fn set_response(&mut self, val: [u32; 4]) {
        unsafe {
            let ptr = &raw mut (*self.regs).response;
            write_volatile(ptr,val);
        }
    }
    pub fn set_buffer_data_port(&mut self, val: u32) {
        unsafe {
            let ptr = &raw mut (*self.regs).buffer_data_port;
            write_volatile(ptr,val);
        }
    }
    pub fn set_present_state(&mut self, val: SdhciPresentState) {
        unsafe {
            let ptr = &raw mut (*self.regs).present_state;
            write_volatile(ptr,val);
        }
    }
    pub fn set_host_control(&mut self, val: SdhciHostControl) {
        unsafe {
            let ptr = &raw mut (*self.regs).host_control;
            write_volatile(ptr,val);
        }
    }
    pub fn set_power_control(&mut self, val: SdhciPowerControl) {
        unsafe {
            let ptr = &raw mut (*self.regs).power_control;
            write_volatile(ptr,val);
        }
    }
    pub fn set_block_gap_control(&mut self, val: SdhciBlockGapControl) {
        unsafe {
            let ptr = &raw mut (*self.regs).block_gap_control;
            write_volatile(ptr,val);
        }
    }
    pub fn set_wakeup_control(&mut self, val: SdhciWakeupControl) {
        unsafe {
            let ptr = &raw mut (*self.regs).wakeup_control;
            write_volatile(ptr,val);
        }
    }
    pub fn set_clock_control(&mut self, val: SdhciClockControl) {
        unsafe {
            let ptr = &raw mut (*self.regs).clock_control;
            write_volatile(ptr,val);
        }
    }
    pub fn set_timeout_control(&mut self, val: u8) {
        unsafe {
            let ptr = &raw mut (*self.regs).timeout_control;
            write_volatile(ptr,val);
        }
    }
    pub fn set_soft_reset_control(&mut self, val: SdhciSoftResetControl) {
        unsafe {
            let ptr = &raw mut (*self.regs).soft_reset_control;
            write_volatile(ptr,val);
        }
    }
    pub fn set_normal_interrupt_status(&mut self, val: SdhciNormalInterruptStatus) {
        unsafe {
            let ptr = &raw mut (*self.regs).normal_interrupt_status;
            write_volatile(ptr,val);
        }
    }
    pub fn set_error_interrupt_status(&mut self, val: SdhciErrorInterruptStatus) {
        unsafe {
            let ptr = &raw mut (*self.regs).error_interrupt_status;
            write_volatile(ptr,val);
        }
    }
    pub fn set_normal_interrupt_status_enable(&mut self, val: SdhciNormalInterruptStatusEnable) {
        unsafe {
            let ptr = &raw mut (*self.regs).normal_interrupt_status_enable;
            write_volatile(ptr,val);
        }
    }
    pub fn set_error_interrupt_status_enable(&mut self, val: SdhciErrorInterruptStatusEnable) {
        unsafe {
            let ptr = &raw mut (*self.regs).error_interrupt_status_enable;
            write_volatile(ptr,val);
        }
    }
    pub fn set_normal_interrupt_signal_enable(&mut self, val: SdhciNormalInterruptSignalEnable) {
        unsafe {
            let ptr = &raw mut (*self.regs).normal_interrupt_signal_enable;
            write_volatile(ptr,val);
        }
    }
    pub fn set_error_interrupt_signal_enable(&mut self, val: SdhciErrorInterruptSignalEnable) {
        unsafe {
            let ptr = &raw mut (*self.regs).error_interrupt_signal_enable;
            write_volatile(ptr,val);
        }
    }
    pub fn set_auto_cmd12_error_status(&mut self, val: SdhciAutoCmd12ErrorStatus) {
        unsafe {
            let ptr = &raw mut (*self.regs).auto_cmd12_error_status;
            write_volatile(ptr,val);
        }
    }
    pub fn set_capabilities(&mut self, val: SdhciCapabilities) {
        unsafe {
            let ptr = &raw mut (*self.regs).capabilities;
            write_volatile(ptr,val);
        }
    }
    pub fn set_maximum_current_capabilities(&mut self, val: SdhciMaximumCurrentCapabilities) {
        unsafe {
            let ptr = &raw mut (*self.regs).maximum_current_capabilities;
            write_volatile(ptr,val);
        }
    }
    pub fn set_force_event_for_cmd12_error(&mut self, val: SdhciForceEventForCmd12Error) {
        unsafe {
            let ptr = &raw mut (*self.regs).force_event_for_cmd12_error;
            write_volatile(ptr,val);
        }
    }
    pub fn set_force_event_for_error_interrupt(&mut self, val: SdhciForceEventForErrorInterrupt) {
        unsafe {
            let ptr = &raw mut (*self.regs).force_event_for_error_interrupt;
            write_volatile(ptr,val);
        }
    }
    pub fn set_adma_error_status(&mut self, val: SdhciAdmaErrorStatus) {
        unsafe {
            let ptr = &raw mut (*self.regs).adma_error_status;
            write_volatile(ptr,val);
        }
    }
    pub fn set_adma_address(&mut self, val: u64) {
        unsafe {
            let ptr = &raw mut (*self.regs).adma_address;
            write_volatile(ptr,val);
        }
    }
    pub fn set_slot_interrupt_status(&mut self, val: SdhciSlotInterruptStatus) {
        unsafe {
            let ptr = &raw mut (*self.regs).slot_interrupt_status;
            write_volatile(ptr,val);
        }
    }
    pub fn set_host_controller_version(&mut self, val: SdhciVersion) {
        unsafe {
            let ptr = &raw mut (*self.regs).host_controller_version;
            write_volatile(ptr,val);
        }
    }
}
