use crate::arch::{ PhysicalAllocation, PhysicalAllocator, PhysicalAddress };
use crate::arch::traits::{ TargetPhysicalAllocation, TargetPhysicalAllocator, TargetAddress };
use crate::threading::scheduler::reschedule;
use heapless::mpmc;
use riscv::_export::critical_section;
use bitfield_struct::{ bitfield, bitenum };
use core::ptr::{read_volatile, write_volatile};
use crate::sdhci::{ Command, CommandType, ResponseType, CommandError, Slot, TransferMode, TransferingDirection, BlockSize };

const R3_COMMAND_PRESET: Command = Command::new()
    .with_response_type(ResponseType::ResponseLen48)
    .with_crc_enable(false)
    .with_index_check_enable(false);

const R2_COMMAND_PRESET: Command = Command::new()
    .with_response_type(ResponseType::ResponseLen136)
    .with_crc_enable(true)
    .with_index_check_enable(false);

const R1_COMMAND_PRESET: Command = Command::new()
    .with_response_type(ResponseType::ResponseLen48)
    .with_crc_enable(true)
    .with_index_check_enable(true);

pub enum EmmcDriverMessage {
    Interrupt,
    Read {
        block_index: u32,
        address: usize,
    },
    Write {
        block_index: u32,
        address: usize,
    },
}

#[allow(deprecated)]
pub static EMMC_DRIVER_QUEUE: mpmc::Queue<EmmcDriverMessage, 32> = mpmc::Queue::new();
pub static mut EMMC_DRIVER_ANY: bool = true;

pub fn put_into_queue(message: EmmcDriverMessage, queue: &mpmc::QueueView<EmmcDriverMessage>) {
    let mut message = message;
    loop {
        match critical_section::with(|_| {
            let mut res = queue.enqueue(message);
            if let Err(v) = res {
                res = queue.enqueue(v);
            }
            res
        }) {
            Ok(_) => break,
            Err(v) => {
                message = v;
                info!("Cannot put element into queue, park current thread and reschedule");
                reschedule();
            }
        }
    }
    unsafe { EMMC_DRIVER_ANY = true };
}

enum AddressingMode {
    Unknown,
    Byte,
    Sector,
}

struct Emmc {
    sdhci_slot: Slot,
    addressing_mode: AddressingMode,
}

impl Emmc {

    fn new(sdhci_slot: Slot) -> Self {
        Emmc { sdhci_slot, addressing_mode: AddressingMode::Unknown }
    }

    // Init card to transfer mode
    fn init(&mut self) -> Result<(), CommandError> {
        let sdhci_slot = self.sdhci_slot;

        // Enable all interrupts statuses and clear them
        sdhci_slot.clear_all_interrupt_statuses();
        sdhci_slot.enable_all_interrupt_statuses();               

        let ocr = loop {
            // Send CMD1 - receive OCR
            let command = R3_COMMAND_PRESET
                .with_data_present(false)
                .with_command_type(CommandType::Normal)
                .with_index(1);
            let argument = 0x80200080; // OCR register value for eMMC with sector addressing
                                       // mode
            let ocr = sdhci_slot.issue_non_dat_command(argument, command)?;

            if ocr & (1 << 31) != 0 {
                break ocr;
            }
        };

        info!("Successful CMD1: {:x}", ocr);
        // TODO: Correct voltage if card isn't compatible with 3.3V
        self.addressing_mode = if ocr & (1 << 30) == 1 { AddressingMode::Sector } else { AddressingMode::Byte };

        // Send CMD2 - receive CID
        let command = R2_COMMAND_PRESET
            .with_data_present(false)
            .with_command_type(CommandType::Normal)
            .with_index(2);
        let argument = 0x0;
        let cid = sdhci_slot.issue_non_dat_command(argument, command)?;

        info!("Slot {} CID: {:x}", sdhci_slot.slot_num, cid);

        // Send CMD3 - set RCA
        let command = R1_COMMAND_PRESET
            .with_data_present(false)
            .with_command_type(CommandType::Normal)
            .with_index(3);
        let argument = 0x1 << 16;
        let status = sdhci_slot.issue_non_dat_command(argument, command)?;

        info!("Slot {} state {}", sdhci_slot.slot_num, (status >> 8) & 0b1111);

        // TODO: Check CSD and enable high speed and widest bus possible

        let command = R1_COMMAND_PRESET
            .with_data_present(false)
            .with_command_type(CommandType::Normal)
            .with_index(7);
        let argument = 0x1 << 16;
        let status = sdhci_slot.issue_non_dat_command(argument, command)?;
        info!("Slot {} state {}", sdhci_slot.slot_num, (status >> 8) & 0b1111);

        Ok(())
    }
}

pub extern "C" fn driver_task() -> ! {
    unsafe {
        let buff = PhysicalAllocator::alloc_contiguous(512);
        let slot = crate::sdhci::host.as_mut().unwrap().slots[0];

        if buff.is_ok() && slot.is_some() {
            let buff: usize = buff.unwrap().addr().try_into().unwrap();
            let slot = slot.unwrap();

            debug!("Allocated buffer for DMA: 0x{buff:X}");
            assert!(buff < (1 << 32));
            let buff = buff as u32;

            let mut emmc = Emmc::new(slot);

            match emmc.init() {
                Ok(_) => {
                    let mut current: Option<EmmcDriverMessage> = None;
                    let pending_requests: mpmc::Queue<EmmcDriverMessage, 32> = mpmc::Queue::new();

                    let block_size = BlockSize::new()
                        .with_transfer_block_size(0x0200)
                        .with_host_sdma_buffer_boundary(0b111);
                    let transfer_mode = TransferMode::new()
                        .with_dma_enable(false)
                        .with_block_count_enable(false)
                        .with_auto_cmd12_enable(false)
                        .with_data_transfer_direction_select(TransferingDirection::Read)
                        .with_multi_block_select(false);
                    let command = R1_COMMAND_PRESET
                        .with_data_present(true)
                        .with_command_type(CommandType::Normal)
                        .with_index(17);
                    let argument = 0x0;
                    let mut buffer: [u32; 512 / 4] = [1; _];

                    let res = slot.issue_cpu_read_data_transfer(
                        block_size,
                        1,
                        argument,
                        transfer_mode,
                        command,
                        &mut buffer
                        );

                    match res {
                        Ok(_) => {
                            info!("Read: {:x?}", buffer);
                        },
                        Err(CommandError::IssuanceTimeout) => error!("Slot {} issuance timeout", slot.slot_num),
                        Err(CommandError::InterruptedError(err)) => error!("Slot {} inrerrupt error: {}", slot.slot_num, err.into_bits()),
                    }


                    // Enable transfer complete interrupts signal and error signals
                    unsafe {
                        let normal_interrupt_signal_enable = &raw mut (*slot.regs).normal_interrupt_signal_enable;
                        let normal_interrupt_signal_enable_val = read_volatile(normal_interrupt_signal_enable)
                            .with_transfer_complete(true);
                        write_volatile(normal_interrupt_signal_enable, normal_interrupt_signal_enable_val);
                        slot.enable_all_error_signals();
                    }

                    // TODO: REMOVE!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
                    put_into_queue(EmmcDriverMessage::Read {block_index: 0, address: 0 }, EMMC_DRIVER_QUEUE.as_view());

                    loop { 
                        let message = critical_section::with(|_| EMMC_DRIVER_QUEUE.dequeue());
                        match message {
                            None => {
                                info!("Driver task yields");
                                unsafe {
                                    EMMC_DRIVER_ANY = false;
                                }
                                reschedule();
                                info!("Driver back to work");
                            },
                            Some(EmmcDriverMessage::Read {block_index: block_index, address: address }) => {
                                if current.is_some() {
                                    // TODO: Put into pending
                                }
                                else {
                                    current = Some(EmmcDriverMessage::Read {block_index: block_index, address: address });

                                    let block_size = BlockSize::new()
                                        .with_transfer_block_size(0x0200)
                                        .with_host_sdma_buffer_boundary(0b111);
                                    let transfer_mode = TransferMode::new()
                                        .with_dma_enable(true)
                                        .with_block_count_enable(false)
                                        .with_auto_cmd12_enable(false)
                                        .with_data_transfer_direction_select(TransferingDirection::Read)
                                        .with_multi_block_select(false);
                                    let command = R1_COMMAND_PRESET
                                        .with_data_present(true)
                                        .with_command_type(CommandType::Normal)
                                        .with_index(17);
                                    
                                    let res = slot.issue_dma_command(
                                        buff,
                                        block_size,
                                        1, 
                                        block_index,
                                        transfer_mode,
                                        command
                                        );

                                    match res {
                                        Ok(_) => info!("Reading from slot {}", slot.slot_num),
                                        Err(CommandError::IssuanceTimeout) => {
                                            error!("Slot {} issuance timeout", slot.slot_num);
                                            current = None;
                                        },
                                        Err(CommandError::InterruptedError(err)) => {
                                            error!("Slot {} inrerrupt error: {}", slot.slot_num, err.into_bits());
                                            current = None;
                                        },
                                    }
                                }
                            },
                            Some(EmmcDriverMessage::Interrupt) => {
                                match slot.wait_for_transfer_complete() {
                                    Ok(_) => { 
                                        info!("DMA completed");
                                        match current {
                                            Some(EmmcDriverMessage::Read { block_index: _, address: _} ) => {
                                                unsafe {
                                                    let data_ptr = buff as *mut u32;
                                                    let data = core::slice::from_raw_parts(data_ptr, 512 / 4);
                                                    info!("READ: {:x?}", data);
                                                }
                                            },
                                            _ => {},
                                        }
                                    },
                                    Err(err) => error!("DMA error: {}", err.into_bits()),
                                }
                                current = None;
                            },
                            Some(_) => {
                            },
                        }
                    }
                },
                Err(CommandError::IssuanceTimeout) => error!("Slot {} issuance timeout", slot.slot_num),
                Err(CommandError::InterruptedError(err)) => error!("Slot {} inrerrupt error: {}", slot.slot_num, err.into_bits()),

            }
        }
    }

    loop { }
}
