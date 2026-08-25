use crate::allocator::AllocatorError;
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

#[derive(Copy, Clone)]
pub enum EmmcOp {
    Read {
        block_index: u32,
        address: usize,
    },
    Write {
        block_index: u32,
        address: usize,
    },
}

pub enum EmmcDriverMessage {
    Interrupt,
    Op(EmmcOp),
}

#[allow(deprecated)]
pub static mut interupt_received: bool = false;
pub static EMMC_DRIVER_QUEUE: mpmc::Queue<EmmcOp, 32> = mpmc::Queue::new();
pub static mut EMMC_DRIVER_ANY: bool = true;

pub fn put_into_queue(message: EmmcDriverMessage, queue: &mpmc::QueueView<EmmcOp>) {
    let mut message = message;
    match message {
        // Out-of-band message
        EmmcDriverMessage::Interrupt => {
            unsafe {
                interupt_received = true;
            }
        },
        // In-band message
        EmmcDriverMessage::Op(op) => {
            let mut op = op;
            loop {
                match critical_section::with(|_| {
                    let mut res = queue.enqueue(op);
                    if let Err(v) = res {
                        res = queue.enqueue(v);
                    }
                    res
                }) {
                    Ok(_) => break,
                    Err(v) => {
                        op = v;
                        info!("Cannot put element into queue, park current thread and reschedule");
                        reschedule();
                    }
                }
            }
            unsafe { EMMC_DRIVER_ANY = true };
        },
    }
}

enum AddressingMode {
    Byte,
    Sector,
}

#[derive(Debug)]
pub enum EmmcInitializationError {
    CommandError(CommandError),
    AllocatorError(AllocatorError),
}

impl From<CommandError> for EmmcInitializationError {
    fn from(err: CommandError) -> Self {
        return EmmcInitializationError::CommandError(err);
    }
}
impl From<AllocatorError> for EmmcInitializationError {
    fn from(err: AllocatorError) -> Self {
        return EmmcInitializationError::AllocatorError(err);
    }
}

struct Emmc {
    sdhci_slot: Slot,
    block_size: u16,    // in bytes
    bounce_buffer: u32, // buffer in first 4GB of phys memory
    addressing_mode: AddressingMode,
    rca: u16,
}

impl Emmc {

    // Init card and retrieve info about it
    fn new(sdhci_slot: Slot) -> Result<Self, EmmcInitializationError> {
        // Enable all interrupts statuses and clear them
        sdhci_slot.clear_all_interrupt_statuses();
        sdhci_slot.enable_all_interrupt_statuses();               

        let rca: u16 = 1;

        // Wait untill card isn't busy with power initialization
        let ocr = loop {
            // Send CMD1 - receive OCR
            let command = R3_COMMAND_PRESET
                .with_data_present(false)
                .with_command_type(CommandType::Normal)
                .with_index(1);
            // TODO: Read compatibilities register and build argument based on it
            let argument = 0x80200080; // Sector addressing mode + 3.3V + 1.8V
            let ocr = sdhci_slot.issue_non_dat_command(argument, command)?;

            // If isn't busy - break
            if ocr & (1 << 31) != 0 {
                break ocr;
            }
        };

        info!("Successful CMD1: {:x}", ocr);
        // TODO: Check if card went to inactive mode because of incompatible voltage
        let addressing_mode = if ocr & (1 << 30) == 1 { AddressingMode::Sector } else { AddressingMode::Byte };

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

        // TODO: Struct/bitfield for status interpreting
        info!("Slot {} state {}", sdhci_slot.slot_num, (status >> 8) & 0b1111);

        // TODO: Check CSD:
        // enable high speed and widest bus possible
        // check block size

        let command = R1_COMMAND_PRESET
            .with_data_present(false)
            .with_command_type(CommandType::Normal)
            .with_index(7);
        let argument = 0x1 << 16;
        let status = sdhci_slot.issue_non_dat_command(argument, command)?;
        info!("Slot {} state {}", sdhci_slot.slot_num, (status >> 8) & 0b1111);

        // Allocate bounce buffer
        let buff = PhysicalAllocator::alloc_contiguous(512)?;
        let buff: usize = buff.addr().try_into().unwrap();

        debug!("Allocated buffer for DMA: 0x{buff:X}");
        // NOTE: Bounce buffer must be in upper 4GB of memory
        assert!(buff < (1 << 32));
        let buff = buff as u32;

        // TODO: Get block size as minimum between slot and eMMC capabilities
        Ok(Emmc { sdhci_slot, block_size: 512, bounce_buffer: buff, addressing_mode, rca })
    }

    fn serve_op(&self, op: EmmcOp) -> Result<(), CommandError> {
        match op {
            EmmcOp::Read {block_index: block_index, address: address } => {
                let block_size = BlockSize::new()
                    .with_transfer_block_size(self.block_size)
                    .with_host_sdma_buffer_boundary(0);
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
                
                let res = self.sdhci_slot.issue_dma_command(
                    self.bounce_buffer,
                    block_size,
                    1, 
                    block_index,
                    transfer_mode,
                    command
                    );

                if res.is_ok() {
                    info!("Reading from slot {}", self.sdhci_slot.slot_num);
                }

                res
            },
            EmmcOp::Write {block_index: block_index, address: address } => {
                // TODO: Copy data from buffer to bounce buffer

                /*
                let block_size = BlockSize::new()
                    .with_transfer_block_size(0x0200)
                    .with_host_sdma_buffer_boundary(0b111);
                let transfer_mode = TransferMode::new()
                    .with_dma_enable(true)
                    .with_block_count_enable(false)
                    .with_auto_cmd12_enable(false)
                    .with_data_transfer_direction_select(TransferingDirection::Write)
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

                if res.is_ok() {
                    info!("Reading from slot {}", slot.slot_num);
                }
                */

                Ok(())
            },
        }
    }
}


pub extern "C" fn driver_task() -> ! {
    unsafe {
        let slot = crate::sdhci::host.as_mut().unwrap().slots[0];

        if let Some(slot) = slot {
            let mut emmc = Emmc::new(slot);

            match emmc {
                Ok(emmc) => {
                    let mut current: Option<EmmcOp> = None;

                    // Enable transfer complete interrupts signal and error signals
                    unsafe {
                        let normal_interrupt_signal_enable = &raw mut (*slot.regs).normal_interrupt_signal_enable;
                        let normal_interrupt_signal_enable_val = read_volatile(normal_interrupt_signal_enable)
                            .with_transfer_complete(true);
                        write_volatile(normal_interrupt_signal_enable, normal_interrupt_signal_enable_val);
                        slot.enable_all_error_signals();
                    }

                    // TODO: REMOVE!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
                    put_into_queue(EmmcDriverMessage::Op(EmmcOp::Read {block_index: 0, address: 0 }), EMMC_DRIVER_QUEUE.as_view());

                    loop { 
                        if interupt_received {
                            match slot.wait_for_transfer_complete() {
                                Ok(_) => { 
                                    info!("DMA completed");
                                    match current {
                                        Some(EmmcOp::Read { block_index: _, address: _} ) => {
                                            // TODO: Copy data from bounce buffer to address
                                            unsafe {
                                                let data_ptr = emmc.bounce_buffer as *mut u8;
                                                let data = core::slice::from_raw_parts(data_ptr, 512);
                                                info!("READ: {:x?}", data);
                                            }
                                        },
                                        Some(EmmcOp::Write { block_index: _, address: _}) => {
                                            info!("Written data");
                                        },
                                        None => {},
                                    }
                                    current = None;
                                    interupt_received = false;
                                },
                                Err(err) => error!("DMA error: {}", err.into_bits()),
                            }
                        }

                        if current.is_none() {
                            let op = critical_section::with(|_| EMMC_DRIVER_QUEUE.dequeue());
                            match op {
                                None => {
                                    info!("Driver task yields");
                                    unsafe {
                                        EMMC_DRIVER_ANY = false;
                                    }
                                    reschedule();
                                    info!("Driver back to work");
                                },
                                Some(op) => {
                                    current = Some(op);
                                    match emmc.serve_op(op) {
                                        Ok(_) => {},
                                        Err(CommandError::IssuanceTimeout) => {
                                            error!("Slot {} issuance timeout", slot.slot_num);
                                            current = None;
                                        },
                                        Err(CommandError::InterruptedError(err)) => {
                                            error!("Slot {} inrerrupt error: {}", slot.slot_num, err.into_bits());
                                            current = None;
                                        },
                                    }
                                },
                            }
                        }
                        else {
                            info!("Driver task yields");
                            unsafe {
                                EMMC_DRIVER_ANY = false;
                            }
                            reschedule();
                            info!("Driver back to work");
                        }
                    }
                },
                Err(err) => error!("Slot {} error: {:?}", slot.slot_num, err),
            }
        }
    }

    loop { }
}
