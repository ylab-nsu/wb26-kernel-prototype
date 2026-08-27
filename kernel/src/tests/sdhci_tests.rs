use crate::tests::mocks::sdhci_interface::{ MockState, MockSdhciInterface };
use crate::sdhci::sdhci_interface::*;
use crate::sdhci::sdhci_command::*;
use crate::sdhci::*;
use crate::run_test;

fn test_soft_reset_success() {
    let mut state = MockState::default();
    // Make the soft reset polling loop exit immediately
    state.soft_reset_control = state.soft_reset_control.with_for_all(false);
    
    let interface = MockSdhciInterface::new(&raw mut state);
    let mut slot = SdhciSlot::new(0, interface);

    let result = slot.soft_reset();

    assert!(result.is_ok());
    assert!(state.written_soft_reset_control.is_some());
    assert!(state.written_soft_reset_control.unwrap().for_all());
}

fn test_soft_reset_timeout() {
    let mut state = MockState::default();
    // Force timeout by keeping the bit high
    state.soft_reset_control = state.soft_reset_control.with_for_all(true);
    
    let interface = MockSdhciInterface::new(&raw mut state);
    let mut slot = SdhciSlot::new(0, interface);

    let result = slot.soft_reset();

    assert!(matches!(result, Err(SdhciError::Timeout)));
}

fn test_power_up_supported_voltage() {
    let mut state = MockState::default();
    
    // Set 3.3V capability to true in the register struct
    state.capabilities = state.capabilities.with_voltage_3_3_support(true);
    
    let interface = MockSdhciInterface::new(&raw mut state);
    let mut slot = SdhciSlot::new(0, interface);

    let result = slot.power_up(SdhciOperatingVoltage::V3_3);

    assert!(result.is_ok());
    assert!(state.written_power_control.is_some());
    assert_eq!(state.written_power_control.unwrap().voltage(), SdhciOperatingVoltage::V3_3);
}

fn test_power_up_incompatible_voltage() {
    let mut state = MockState::default();
    
    // 1.8V is false by default in zeroed memory
    let interface = MockSdhciInterface::new(&raw mut state);
    let mut slot = SdhciSlot::new(0, interface);

    let result = slot.power_up(SdhciOperatingVoltage::V1_8);

    assert!(matches!(result, Err(SdhciError::Incompatible)));
    assert!(state.written_power_control.is_none());
}

fn test_set_sdclock_frequency_success() {
    let mut state = MockState::default();
    
    // Provide a base clock frequency (e.g. 50MHz) and mark stable
    state.capabilities = state.capabilities.with_base_clock_freq(50);
    state.clock_control = state.clock_control.with_internal_clock_stable(true);
    
    let interface = MockSdhciInterface::new(&raw mut state);
    let mut slot = SdhciSlot::new(0, interface);

    let result = slot.set_sdclock_frequency(400);

    assert!(result.is_ok());
    assert!(state.written_clock_control.is_some());
}

fn test_wait_for_command_complete_success() {
    let mut state = MockState::default();
    
    // Exit loop immediately
    state.normal_interrupt_status = state.normal_interrupt_status
        .with_command_complete(true)
        .with_error_interrupt(false);
    
    let interface = MockSdhciInterface::new(&raw mut state);
    let mut slot = SdhciSlot::new(0, interface);

    let result = slot.wait_for_command_complete();

    assert!(result.is_ok());
    assert!(state.written_normal_interrupt_status.is_some());
    assert!(state.written_normal_interrupt_status.unwrap().command_complete()); // Check that driver
                                                                                // cleared bit
}

fn test_read_response_normalized() {
    let mut state = MockState::default();
    
    // Fake the response registers
    state.response = [0x11111111, 0x22222222, 0x33333333, 0x44444444];
    
    let interface = MockSdhciInterface::new(&raw mut state);
    let slot = SdhciSlot::new(0, interface);

    let response = slot.read_response_normalized();

    assert_eq!(response, 0x44444444_33333333_22222222_11111111);
}

fn test_send_command_success_no_data() {
    let mut state = MockState::default();
    
    // Both CMD and DAT lines are free
    state.present_state = state.present_state
        .with_command_inhibit_cmd(false)
        .with_command_inhibit_dat(false);
        
    let interface = MockSdhciInterface::new(&raw mut state);
    let mut slot = SdhciSlot::new(0, interface);

    // Construct a command without data
    let mut cmd_desc: SdhciCommandDesc = unsafe { core::mem::zeroed() };
    cmd_desc = cmd_desc.with_data_present(false);

    let command = SdhciCommand {
        command_desc: cmd_desc,
        argument: 0xDEADBEEF, // Distinct argument to trace
        kind: SdhciCommandKind::NonDatCommand,
    };

    let result = slot.send_command(&command);

    assert!(result.is_ok());
    assert_eq!(state.written_argument, Some(0xDEADBEEF));
    assert!(state.written_command.is_some());
}

fn test_send_command_timeout_cmd_inhibit() {
    let mut state = MockState::default();
    
    // CMD line is currently inhibited/busy
    state.present_state = state.present_state
        .with_command_inhibit_cmd(true)
        .with_command_inhibit_dat(false);
        
    let interface = MockSdhciInterface::new(&raw mut state);
    let mut slot = SdhciSlot::new(0, interface);

    let mut cmd_desc: SdhciCommandDesc = unsafe { core::mem::zeroed() };
    cmd_desc = cmd_desc.with_data_present(false);

    let command = SdhciCommand {
        command_desc: cmd_desc,
        argument: 0x0,
        kind: SdhciCommandKind::NonDatCommand,
    };

    let result = slot.send_command(&command);

    assert!(matches!(result, Err(SdhciError::Timeout)));
    assert!(state.written_command.is_none(), "Should not write command if timed out");
}

fn test_send_command_timeout_dat_inhibit_on_data_command() {
    let mut state = MockState::default();
    
    // CMD line is free, but DAT line is inhibited
    state.present_state = state.present_state
        .with_command_inhibit_cmd(false)
        .with_command_inhibit_dat(true);
        
    let interface = MockSdhciInterface::new(&mut state);
    let mut slot = SdhciSlot::new(0, interface);

    // We set data_present to true, which forces the driver to also check DAT line
    let mut cmd_desc: SdhciCommandDesc = unsafe { core::mem::zeroed() };
    cmd_desc = cmd_desc.with_data_present(true);

    // Provide a dummy zeroed data transfer block
    let transfer = unsafe { core::mem::zeroed() };

    let command = SdhciCommand {
        command_desc: cmd_desc,
        argument: 0x0,
        kind: SdhciCommandKind::DataTransfer(transfer),
    };

    let result = slot.send_command(&command);

    assert!(matches!(result, Err(SdhciError::Timeout)));
    assert!(state.written_command.is_none(), "Should not write command if timed out waiting for DAT lines");
}

pub fn run_tests() {
    run_test!(test_soft_reset_success);
    run_test!(test_soft_reset_timeout);
    run_test!(test_power_up_supported_voltage);
    run_test!(test_power_up_incompatible_voltage);
    run_test!(test_set_sdclock_frequency_success);
    run_test!(test_wait_for_command_complete_success);
    run_test!(test_read_response_normalized);
    run_test!(test_send_command_success_no_data);
    run_test!(test_send_command_timeout_cmd_inhibit);
    run_test!(test_send_command_timeout_dat_inhibit_on_data_command);
}
