use crate::sdhci::sdhci_interface::*;

#[derive(Copy, Clone)]
pub struct MockState {
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
    pub capabilities: SdhciCapabilities,
    pub maximum_current_capabilities: SdhciMaximumCurrentCapabilities,
    pub force_event_for_cmd12_error: SdhciForceEventForCmd12Error,
    pub force_event_for_error_interrupt: SdhciForceEventForErrorInterrupt,
    pub adma_error_status: SdhciAdmaErrorStatus,
    pub adma_address: u64,
    pub slot_interrupt_status: SdhciSlotInterruptStatus,
    pub host_controller_version: SdhciVersion,

    pub written_sdma_address: Option<u32>,
    pub written_block_size: Option<SdhciBlockSize>,
    pub written_block_count: Option<u16>,
    pub written_argument: Option<u32>,
    pub written_transfer_mode: Option<SdhciTransferMode>,
    pub written_command: Option<SdhciCommandDesc>,
    pub written_response: Option<[u32; 4]>,
    pub written_buffer_data_port: Option<u32>,
    pub written_present_state: Option<SdhciPresentState>,
    pub written_host_control: Option<SdhciHostControl>,
    pub written_power_control: Option<SdhciPowerControl>,
    pub written_block_gap_control: Option<SdhciBlockGapControl>,
    pub written_wakeup_control: Option<SdhciWakeupControl>,
    pub written_clock_control: Option<SdhciClockControl>,
    pub written_timeout_control: Option<u8>, // NOTE: Refer to spec
    pub written_soft_reset_control: Option<SdhciSoftResetControl>,
    pub written_normal_interrupt_status: Option<SdhciNormalInterruptStatus>,
    pub written_error_interrupt_status: Option<SdhciErrorInterruptStatus>,
    pub written_normal_interrupt_status_enable: Option<SdhciNormalInterruptStatusEnable>,
    pub written_error_interrupt_status_enable: Option<SdhciErrorInterruptStatusEnable>,
    pub written_normal_interrupt_signal_enable: Option<SdhciNormalInterruptSignalEnable>,
    pub written_error_interrupt_signal_enable: Option<SdhciErrorInterruptSignalEnable>,
    pub written_auto_cmd12_error_status: Option<SdhciAutoCmd12ErrorStatus>,
    pub written_capabilities: Option<SdhciCapabilities>,
    pub written_maximum_current_capabilities: Option<SdhciMaximumCurrentCapabilities>,
    pub written_force_event_for_cmd12_error: Option<SdhciForceEventForCmd12Error>,
    pub written_force_event_for_error_interrupt: Option<SdhciForceEventForErrorInterrupt>,
    pub written_adma_error_status: Option<SdhciAdmaErrorStatus>,
    pub written_adma_address: Option<u64>,
    pub written_slot_interrupt_status: Option<SdhciSlotInterruptStatus>,
    pub written_host_controller_version: Option<SdhciVersion>,
}

#[derive(Copy, Clone)]
pub struct MockSdhciInterface {
    pub state: *mut MockState,
}

impl MockSdhciInterface {
    pub fn from_raw_pointer(slot_regs: *mut SdhciSlotRegisters) -> Self {
        unimplemented!("No implementation of from_raw_pointer in sdhci mock");
    }

    pub fn new(state: *mut MockState) -> Self {
        MockSdhciInterface { state }
    }

    pub fn sdma_address(&self) -> u32 {
        unsafe {
            (*self.state).sdma_address
        }
    }
    pub fn block_size(&self) -> SdhciBlockSize {
        unsafe {
            (*self.state).block_size
        }
    }
    pub fn block_count(&self) -> u16 {
        unsafe {
            (*self.state).block_count
        }
    }
    pub fn argument(&self) -> u32 {
        unsafe {
            (*self.state).argument
        }
    }
    pub fn transfer_mode(&self) -> SdhciTransferMode {
        unsafe {
            (*self.state).transfer_mode
        }
    }
    pub fn command(&self) -> SdhciCommandDesc {
        unsafe {
            (*self.state).command
        }
    }
    pub fn response(&self) -> [u32; 4] {
        unsafe {
            (*self.state).response
        }
    }
    pub fn buffer_data_port(&self) -> u32 {
        unsafe {
            (*self.state).buffer_data_port
        }
    }
    pub fn present_state(&self) -> SdhciPresentState {
        unsafe {
            (*self.state).present_state
        }
    }
    pub fn host_control(&self) -> SdhciHostControl {
        unsafe {
            (*self.state).host_control
        }
    }
    pub fn power_control(&self) -> SdhciPowerControl {
        unsafe {
            (*self.state).power_control
        }
    }
    pub fn block_gap_control(&self) -> SdhciBlockGapControl {
        unsafe {
            (*self.state).block_gap_control
        }
    }
    pub fn wakeup_control(&self) -> SdhciWakeupControl {
        unsafe {
            (*self.state).wakeup_control
        }
    }
    pub fn clock_control(&self) -> SdhciClockControl {
        unsafe {
            (*self.state).clock_control
        }
    }
    pub fn timeout_control(&self) -> u8 {
        unsafe {
            (*self.state).timeout_control
        }
    }
    pub fn soft_reset_control(&self) -> SdhciSoftResetControl {
        unsafe {
            (*self.state).soft_reset_control
        }
    }
    pub fn normal_interrupt_status(&self) -> SdhciNormalInterruptStatus {
        unsafe {
            (*self.state).normal_interrupt_status
        }
    }
    pub fn error_interrupt_status(&self) -> SdhciErrorInterruptStatus {
        unsafe {
            (*self.state).error_interrupt_status
        }
    }
    pub fn normal_interrupt_status_enable(&self) -> SdhciNormalInterruptStatusEnable {
        unsafe {
            (*self.state).normal_interrupt_status_enable
        }
    }
    pub fn error_interrupt_status_enable(&self) -> SdhciErrorInterruptStatusEnable {
        unsafe {
            (*self.state).error_interrupt_status_enable
        }
    }
    pub fn normal_interrupt_signal_enable(&self) -> SdhciNormalInterruptSignalEnable {
        unsafe {
            (*self.state).normal_interrupt_signal_enable
        }
    }
    pub fn error_interrupt_signal_enable(&self) -> SdhciErrorInterruptSignalEnable {
        unsafe {
            (*self.state).error_interrupt_signal_enable
        }
    }
    pub fn auto_cmd12_error_status(&self) -> SdhciAutoCmd12ErrorStatus {
        unsafe {
            (*self.state).auto_cmd12_error_status
        }
    }
    pub fn capabilities(&self) -> SdhciCapabilities {
        unsafe {
            (*self.state).capabilities
        }
    }
    pub fn maximum_current_capabilities(&self) -> SdhciMaximumCurrentCapabilities {
        unsafe {
            (*self.state).maximum_current_capabilities
        }
    }
    pub fn force_event_for_cmd12_error(&self) -> SdhciForceEventForCmd12Error {
        unsafe {
            (*self.state).force_event_for_cmd12_error
        }
    }
    pub fn force_event_for_error_interrupt(&self) -> SdhciForceEventForErrorInterrupt {
        unsafe {
            (*self.state).force_event_for_error_interrupt
        }
    }
    pub fn adma_error_status(&self) -> SdhciAdmaErrorStatus {
        unsafe {
            (*self.state).adma_error_status
        }
    }
    pub fn adma_address(&self) -> u64 {
        unsafe {
            (*self.state).adma_address
        }
    }
    pub fn slot_interrupt_status(&self) -> SdhciSlotInterruptStatus {
        unsafe {
            (*self.state).slot_interrupt_status
        }
    }
    pub fn host_controller_version(&self) -> SdhciVersion {
        unsafe {
            (*self.state).host_controller_version
        }
    }

    pub fn set_sdma_address(&mut self, val: u32) {
        unsafe {
            (*self.state).written_sdma_address = Some(val);
        }
    }
    pub fn set_block_size(&mut self, val: SdhciBlockSize) {
        unsafe {
            (*self.state).written_block_size = Some(val);
        }
    }
    pub fn set_block_count(&mut self, val: u16) {
        unsafe {
            (*self.state).written_block_count = Some(val);
        }
    }
    pub fn set_argument(&mut self, val: u32) {
        unsafe {
            (*self.state).written_argument = Some(val);
        }
    }
    pub fn set_transfer_mode(&mut self, val: SdhciTransferMode) {
        unsafe {
            (*self.state).written_transfer_mode = Some(val);
        }
    }
    pub fn set_command(&mut self, val: SdhciCommandDesc) {
        unsafe {
            (*self.state).written_command = Some(val);
        }
    }
    pub fn set_response(&mut self, val: [u32; 4]) {
        unsafe {
            (*self.state).written_response = Some(val);
        }
    }
    pub fn set_buffer_data_port(&mut self, val: u32) {
        unsafe {
            (*self.state).written_buffer_data_port = Some(val);
        }
    }
    pub fn set_present_state(&mut self, val: SdhciPresentState) {
        unsafe {
            (*self.state).written_present_state = Some(val);
        }
    }
    pub fn set_host_control(&mut self, val: SdhciHostControl) {
        unsafe {
            (*self.state).written_host_control = Some(val);
        }
    }
    pub fn set_power_control(&mut self, val: SdhciPowerControl) {
        unsafe {
            (*self.state).written_power_control = Some(val);
        }
    }
    pub fn set_block_gap_control(&mut self, val: SdhciBlockGapControl) {
        unsafe {
            (*self.state).written_block_gap_control = Some(val);
        }
    }
    pub fn set_wakeup_control(&mut self, val: SdhciWakeupControl) {
        unsafe {
            (*self.state).written_wakeup_control = Some(val);
        }
    }
    pub fn set_clock_control(&mut self, val: SdhciClockControl) {
        unsafe {
            (*self.state).written_clock_control = Some(val);
        }
    }
    pub fn set_timeout_control(&mut self, val: u8) {
        unsafe {
            (*self.state).written_timeout_control = Some(val);
        }
    }
    pub fn set_soft_reset_control(&mut self, val: SdhciSoftResetControl) {
        unsafe {
            (*self.state).written_soft_reset_control = Some(val);
        }
    }
    pub fn set_normal_interrupt_status(&mut self, val: SdhciNormalInterruptStatus) {
        unsafe {
            (*self.state).written_normal_interrupt_status = Some(val);
        }
    }
    pub fn set_error_interrupt_status(&mut self, val: SdhciErrorInterruptStatus) {
        unsafe {
            (*self.state).written_error_interrupt_status = Some(val);
        }
    }
    pub fn set_normal_interrupt_status_enable(&mut self, val: SdhciNormalInterruptStatusEnable) {
        unsafe {
            (*self.state).written_normal_interrupt_status_enable = Some(val);
        }
    }
    pub fn set_error_interrupt_status_enable(&mut self, val: SdhciErrorInterruptStatusEnable) {
        unsafe {
            (*self.state).written_error_interrupt_status_enable = Some(val);
        }
    }
    pub fn set_normal_interrupt_signal_enable(&mut self, val: SdhciNormalInterruptSignalEnable) {
        unsafe {
            (*self.state).written_normal_interrupt_signal_enable = Some(val);
        }
    }
    pub fn set_error_interrupt_signal_enable(&mut self, val: SdhciErrorInterruptSignalEnable) {
        unsafe {
            (*self.state).written_error_interrupt_signal_enable = Some(val);
        }
    }
    pub fn set_auto_cmd12_error_status(&mut self, val: SdhciAutoCmd12ErrorStatus) {
        unsafe {
            (*self.state).written_auto_cmd12_error_status = Some(val);
        }
    }
    pub fn set_capabilities(&mut self, val: SdhciCapabilities) {
        unsafe {
            (*self.state).written_capabilities = Some(val);
        }
    }
    pub fn set_maximum_current_capabilities(&mut self, val: SdhciMaximumCurrentCapabilities) {
        unsafe {
            (*self.state).written_maximum_current_capabilities = Some(val);
        }
    }
    pub fn set_force_event_for_cmd12_error(&mut self, val: SdhciForceEventForCmd12Error) {
        unsafe {
            (*self.state).written_force_event_for_cmd12_error = Some(val);
        }
    }
    pub fn set_force_event_for_error_interrupt(&mut self, val: SdhciForceEventForErrorInterrupt) {
        unsafe {
            (*self.state).written_force_event_for_error_interrupt = Some(val);
        }
    }
    pub fn set_adma_error_status(&mut self, val: SdhciAdmaErrorStatus) {
        unsafe {
            (*self.state).written_adma_error_status = Some(val);
        }
    }
    pub fn set_adma_address(&mut self, val: u64) {
        unsafe {
            (*self.state).written_adma_address = Some(val);
        }
    }
    pub fn set_slot_interrupt_status(&mut self, val: SdhciSlotInterruptStatus) {
        unsafe {
            (*self.state).written_slot_interrupt_status = Some(val);
        }
    }
    pub fn set_host_controller_version(&mut self, val: SdhciVersion) {
        unsafe {
            (*self.state).written_host_controller_version = Some(val);
        }
    }
}

impl Default for MockState {
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
