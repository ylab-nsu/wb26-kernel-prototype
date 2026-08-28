use crate::allocator::AllocatorError;
use crate::arch::{ PhysicalAllocation, PhysicalAllocator };
use crate::arch::traits::{ TargetPhysicalAllocation, TargetPhysicalAllocator };
use crate::threading::scheduler::reschedule;
use heapless::{ mpmc, Vec };
use riscv::_export::critical_section;
use bitfield_struct::{ bitfield, bitenum };
use crate::sdhci::sdhci_command::*;
use crate::sdhci::{ SdhciCommandDesc, SdhciCommandType, SdhciResponseType, SdhciError, SdhciSlot, SdhciTransferMode, SdhciTransferingDirection, SdhciBlockSize, SdhciOperatingVoltage };

const R3_COMMAND_PRESET: SdhciCommandDesc = SdhciCommandDesc::new()
    .with_response_type(SdhciResponseType::ResponseLen48)
    .with_crc_enable(false)
    .with_index_check_enable(false);

const R2_COMMAND_PRESET: SdhciCommandDesc = SdhciCommandDesc::new()
    .with_response_type(SdhciResponseType::ResponseLen136)
    .with_crc_enable(true)
    .with_index_check_enable(false);

const R1_COMMAND_PRESET: SdhciCommandDesc = SdhciCommandDesc::new()
    .with_response_type(SdhciResponseType::ResponseLen48)
    .with_crc_enable(true)
    .with_index_check_enable(true);

const NO_RESPONSE_PRESET: SdhciCommandDesc = SdhciCommandDesc::new()
    .with_response_type(SdhciResponseType::NoResponse)
    .with_crc_enable(false)
    .with_index_check_enable(false);

const ADMA2_VALID: u16 = 1 << 0;
const ADMA2_END: u16 = 1 << 1;
const ADMA2_TRANSFER: u16 = 0b10 << 4;
const ADMA2_DESCRIPTOR_CAPACITY: usize = 16;
const MAX_BLOCKS_PER_TRANSFER: u16 = ADMA2_DESCRIPTOR_CAPACITY as u16;
const EMMC_TEST_PATTERN: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];
const EMMC_TEST_SAMPLE_SIZE: usize = 16;

#[repr(C, align(4))]
#[derive(Copy, Clone)]
struct Adma2Descriptor {
    attributes: u16,
    length: u16,
    address: u32,
}

struct Adma2DescriptorTable {
    allocation: PhysicalAllocation,
    len: usize,
    capacity: usize,
}

impl Adma2DescriptorTable {
    fn new(capacity: usize) -> Result<Self, AllocatorError> {
        let size = core::mem::size_of::<Adma2Descriptor>()
            .checked_mul(capacity)
            .ok_or(AllocatorError::NotEnoughMemory)?;

        let allocation = PhysicalAllocator::alloc_contiguous_aligned(
            size,
            core::mem::align_of::<Adma2Descriptor>(),
        )?;
        let address: usize = allocation.addr().try_into().unwrap();

        debug!(
            "Allocated ADMA2 descriptor table: address=0x{address:X}, capacity={capacity}, requested_size={size}, allocated_size={}",
            allocation.size(),
        );
        assert!(address < (1 << 32));

        Ok(Self {
            allocation,
            len: 0,
            capacity,
        })
    }

    fn clear(&mut self) {
        self.len = 0;
    }

    fn push_transfer(&mut self, buffer_address: u32, length: u16) -> bool {
        if self.len == self.capacity {
            return false;
        }

        let descriptor_index = self.len;

        unsafe {
            let descriptor = (self.physical_address() as *mut Adma2Descriptor)
                .add(descriptor_index);

            descriptor.write(Adma2Descriptor {
                attributes: ADMA2_VALID | ADMA2_TRANSFER,
                length,
                address: buffer_address,
            });
        }

        self.len += 1;

        debug!(
            "ADMA2 descriptor[{descriptor_index}]: buffer=0x{buffer_address:08X}, length={length}",
        );

        true
    }

    fn finish(&mut self) -> bool {
        if self.len == 0 {
            return false;
        }

        unsafe {
            let descriptor = (self.physical_address() as *mut Adma2Descriptor)
                .add(self.len - 1);
            let mut descriptor_value = descriptor.read();
            descriptor_value.attributes |= ADMA2_END;
            descriptor.write(descriptor_value);
        }

        true
    }

    fn physical_address(&self) -> usize {
        self.allocation.addr().try_into().unwrap()
    }
}

#[derive(Copy, Clone)]
pub enum EmmcOp {
    Read {
        block_index: u32,
        block_count: u16,
        address: usize,
    },
    Write {
        block_index: u32,
        block_count: u16,
        address: usize,
    },
}

#[derive(Copy, Clone, PartialEq)]
enum EmmcDirection {
    Read,
    Write,
}

impl EmmcOp {
    fn direction(&self) -> EmmcDirection {
        match self {
            EmmcOp::Read { .. } => EmmcDirection::Read,
            EmmcOp::Write { .. } => EmmcDirection::Write,
        }
    }

    fn block_index(&self) -> u32 {
        match self {
            EmmcOp::Read { block_index, .. }
            | EmmcOp::Write { block_index, .. } => *block_index,
        }
    }

    fn block_count(&self) -> u16 {
        match self {
            EmmcOp::Read { block_count, .. }
            | EmmcOp::Write { block_count, .. } => *block_count,
        }
    }
}

struct EmmcBatch {
    direction: EmmcDirection,
    first_block: u32,
    block_count: u16,
    operations: Vec<EmmcOp, ADMA2_DESCRIPTOR_CAPACITY>,
}

impl EmmcBatch {
    fn new(first: EmmcOp) -> Self {
        let mut operations = Vec::new();
        assert!(operations.push(first).is_ok());

        EmmcBatch {
            direction: first.direction(),
            first_block: first.block_index(),
            block_count: first.block_count(),
            operations,
        }
    }

    fn can_append(&self, operation: &EmmcOp) -> bool {
        let Some(next_block) = self.first_block.checked_add(self.block_count as u32) else {
            return false;
        };
        let Some(block_count) = self.block_count.checked_add(operation.block_count()) else {
            return false;
        };

        self.direction == operation.direction()
            && next_block == operation.block_index()
            && block_count <= MAX_BLOCKS_PER_TRANSFER
    }

    fn push(&mut self, operation: EmmcOp) -> bool {
        if !self.can_append(&operation) {
            return false;
        }

        let operation_block_count = operation.block_count();
        if self.operations.push(operation).is_err() {
            return false;
        }

        self.block_count += operation_block_count;
        true
    }
}

pub enum EmmcDriverMessage {
    Interrupt,
    Op(EmmcOp),
}

#[allow(deprecated)]
pub static EMMC_DRIVER_QUEUE: mpmc::Queue<EmmcOp, 32> = mpmc::Queue::new();
pub static mut INTERRUPT_RECEIVED: bool = false;
pub static mut EMMC_DRIVER_ANY: bool = true;

pub fn put_into_queue(message: EmmcDriverMessage, queue: &mpmc::QueueView<EmmcOp>) {
    match message {
        // Out-of-band message
        EmmcDriverMessage::Interrupt => {
            unsafe {
                INTERRUPT_RECEIVED = true;
                EMMC_DRIVER_ANY = true;
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

fn take_next_operation(lookahead: &mut Option<EmmcOp>) -> Option<EmmcOp> {
    lookahead
        .take()
        .or_else(|| critical_section::with(|_| EMMC_DRIVER_QUEUE.dequeue()))
}

fn build_batch(first: EmmcOp, lookahead: &mut Option<EmmcOp>) -> EmmcBatch {
    let mut batch = EmmcBatch::new(first);

    while batch.block_count < MAX_BLOCKS_PER_TRANSFER {
        let Some(operation) = take_next_operation(lookahead) else {
            break;
        };

        if !batch.push(operation) {
            *lookahead = Some(operation);
            break;
        }
    }

    batch
}

#[derive(Debug)]
#[repr(u8)]
#[bitenum(all = false)]
enum AccessMode {
    #[fallback]
    Byte = 0,
    Sector = 2,
}

#[derive(Debug)]
pub enum EmmcUnrecoverableError {
    SdhciError(SdhciError),
    AllocatorError(AllocatorError),
    Incompatible,                   // Card is incompatible with host
}

#[derive(Debug)]
pub enum EmmcRecoverableError {
    InvalidConditions(Ocr),         // The host is compatible with card, but must change it's mode
}

#[derive(Debug)]
pub enum EmmcError {
    Recoverable(EmmcRecoverableError),
    Unrecoverable(EmmcUnrecoverableError),
}

impl From<SdhciError> for EmmcUnrecoverableError {
    fn from(err: SdhciError) -> Self {
        return EmmcUnrecoverableError::SdhciError(err);
    }
}
impl From<AllocatorError> for EmmcUnrecoverableError {
    fn from(err: AllocatorError) -> Self {
        return EmmcUnrecoverableError::AllocatorError(err);
    }
}
impl From<SdhciError> for EmmcError {
    fn from(err: SdhciError) -> Self {
        return EmmcError::Unrecoverable(EmmcUnrecoverableError::SdhciError(err));
    }
}
impl From<AllocatorError> for EmmcError {
    fn from(err: AllocatorError) -> Self {
        return EmmcError::Unrecoverable(EmmcUnrecoverableError::AllocatorError(err));
    }
}
impl From<EmmcRecoverableError> for EmmcError {
    fn from(err: EmmcRecoverableError) -> Self {
        return EmmcError::Recoverable(err);
    }
}
impl From<EmmcUnrecoverableError> for EmmcError {
    fn from(err: EmmcUnrecoverableError) -> Self {
        return EmmcError::Unrecoverable(err);
    }
}

// Operation Conditions Register
#[bitfield(u32)]
pub struct Ocr {
    #[bits(7)]
    __: usize,
    from_1_70_v_to_1_95_v: bool,
    v2_0: bool,
    v2_1: bool,
    v2_2: bool,
    v2_3: bool,
    v2_4: bool,
    v2_5: bool,
    v2_6: bool,
    v2_7: bool,
    v2_8: bool,
    v2_9: bool,
    v3_0: bool,
    v3_1: bool,
    v3_2: bool,
    v3_3: bool,
    v3_4: bool,
    v3_5: bool,
    #[bits(5)]
    __: usize,
    #[bits(2)]
    access_mode: AccessMode,
    powered_up: bool,
}

impl Ocr {
    // If two ocr values are compatible
    fn compatible_with(&self, other: &Self) -> bool {
        // No bitmasks today in order to be able to change fields order 
        self.from_1_70_v_to_1_95_v() && other.from_1_70_v_to_1_95_v() || 
        self.v2_0() && other.v2_0() || 
        self.v2_1() && other.v2_1() || 
        self.v2_2() && other.v2_2() || 
        self.v2_3() && other.v2_3() || 
        self.v2_4() && other.v2_4() || 
        self.v2_5() && other.v2_5() || 
        self.v2_6() && other.v2_6() || 
        self.v2_7() && other.v2_7() || 
        self.v2_8() && other.v2_8() || 
        self.v2_9() && other.v2_9() || 
        self.v3_0() && other.v3_0() || 
        self.v3_1() && other.v3_1() || 
        self.v3_2() && other.v3_2() || 
        self.v3_3() && other.v3_3() || 
        self.v3_4() && other.v3_4() || 
        self.v3_5() && other.v3_5()
    }

    // If current slot voltage is compatible with OCR
    fn runs_in_compatible_conditions(&self, slot: &SdhciSlot) -> bool {
        let power_ctl = slot.sdhci_interface.power_control();

        match power_ctl.voltage() {
            SdhciOperatingVoltage::V1_8 => self.from_1_70_v_to_1_95_v(),
            SdhciOperatingVoltage::V3_0 => self.v3_0(),
            SdhciOperatingVoltage::V3_3 => self.v3_3(),
        }
    }
}

struct Emmc {
    sdhci_slot: SdhciSlot,
    block_size: u16,                    // in bytes
    bounce_buffer: PhysicalAllocation,  // buffer in first 4GB of phys memory
    adma_descriptor_table: Adma2DescriptorTable,
    access_mode: AccessMode,
    rca: u16,
}

impl Emmc {
    // Init card and retrieve info about it
    fn new(mut sdhci_slot: SdhciSlot) -> Result<Self, EmmcError> {
        // Enable all interrupts statuses and clear them
        sdhci_slot.clear_all_interrupt_statuses();
        sdhci_slot.enable_all_interrupt_statuses();               

        // Enable transfer complete interrupts signal and error signals
        let normal_interrupt_signal_enable = sdhci_slot.sdhci_interface.normal_interrupt_signal_enable()
            .with_transfer_complete(true);
        sdhci_slot.sdhci_interface.set_normal_interrupt_signal_enable(normal_interrupt_signal_enable);
        sdhci_slot.enable_all_error_signals();

        // Set 400KHz as default frequency
        // TODO: Check for errors
        sdhci_slot.set_sdclock_frequency(400)?;

        let command = SdhciCommand {
            command_desc: NO_RESPONSE_PRESET
                .with_data_present(false)
                .with_command_type(SdhciCommandType::Normal)
                .with_index(0),
            argument: 0x0,
            kind: SdhciCommandKind::NonDatCommand,
        };
        sdhci_slot.send_command(&command)?;
        sdhci_slot.wait_for_command_complete()?;

        let rca: u16 = 1;
        let slot_ocr = slot_capabilities_to_ocr(&sdhci_slot);
        let slot_ocr_as_bits = slot_ocr.into_bits();

        // Send CMD1 - receive OCR
        let command = SdhciCommand {
            command_desc: R3_COMMAND_PRESET
                .with_data_present(false)
                .with_command_type(SdhciCommandType::Normal)
                .with_index(1),
            argument: slot_ocr_as_bits,
            kind: SdhciCommandKind::NonDatCommand,
        };

        // Wait until card isn't busy with power up procedure
        let ocr = loop {
            sdhci_slot.send_command(&command)?;
            sdhci_slot.wait_for_command_complete()?;
            let ocr = Ocr::from_bits(sdhci_slot.read_response_normalized() as u32);

            // If card finished power up procedure - break
            if ocr.powered_up() {
                break ocr;
            }
        };

        info!("Successful CMD1: {:x}", ocr.into_bits());

        // If slot is incompatible with card
        if !ocr.compatible_with(&slot_ocr) {
            return Err(EmmcError::Unrecoverable(EmmcUnrecoverableError::Incompatible));
        }
        // If slot is compatible, but must change its' voltage
        if !ocr.runs_in_compatible_conditions(&sdhci_slot) {
            return Err(EmmcError::Recoverable(EmmcRecoverableError::InvalidConditions(ocr)));
        }

        let access_mode = ocr.access_mode();

        // Send CMD2 - receive CID
        let command = SdhciCommand {
            command_desc: R2_COMMAND_PRESET
                .with_data_present(false)
                .with_command_type(SdhciCommandType::Normal)
                .with_index(2),
            argument: 0x0,
            kind: SdhciCommandKind::NonDatCommand,
        };
        sdhci_slot.send_command(&command)?;
        sdhci_slot.wait_for_command_complete()?;
        let cid = sdhci_slot.read_response_normalized();

        info!("Slot {} CID: {:x}", sdhci_slot.slot_num, cid);

        // Send CMD3 - set RCA
        let command = SdhciCommand {
            command_desc: R1_COMMAND_PRESET
                .with_data_present(false)
                .with_command_type(SdhciCommandType::Normal)
                .with_index(3),
            argument: (rca as u32) << 16,
            kind: SdhciCommandKind::NonDatCommand,
        };
        sdhci_slot.send_command(&command)?;
        sdhci_slot.wait_for_command_complete()?;
        let status = sdhci_slot.read_response_normalized();

        // TODO: Struct/bitfield for status interpreting
        info!("Slot {} state {}", sdhci_slot.slot_num, (status >> 8) & 0b1111);

        let command = SdhciCommand {
            command_desc: R1_COMMAND_PRESET
                .with_data_present(false)
                .with_command_type(SdhciCommandType::Normal)
                .with_index(7),
            argument: (rca as u32) << 16,
            kind: SdhciCommandKind::NonDatCommand,
        };
        sdhci_slot.send_command(&command)?;
        sdhci_slot.wait_for_command_complete()?;
        let status = sdhci_slot.read_response_normalized();
        info!("Slot {} state {}", sdhci_slot.slot_num, (status >> 8) & 0b1111);

        // Allocate bounce buffer
        let buff = PhysicalAllocator::alloc_contiguous_aligned(
            512 * MAX_BLOCKS_PER_TRANSFER as usize,
            512,
        )?;
        let buff_addr: usize = buff.addr().try_into().unwrap();

        // NOTE: Bounce buffer must be in upper 4GB of memory
        debug!("Allocated buffer for DMA: 0x{buff_addr:X}");
        assert!(buff_addr < (1 << 32));

        let adma_descriptor_table =
            Adma2DescriptorTable::new(ADMA2_DESCRIPTOR_CAPACITY)?;

        // TODO: Get block size as minimum between slot and eMMC capabilities
        Ok(Emmc { sdhci_slot, block_size: 512, bounce_buffer: buff, adma_descriptor_table, access_mode, rca })
    }

    // Keep the previous SDMA path so it can be used as a fallback on
    // controllers without ADMA2 support. This path intentionally stays
    // single-block for now, matching the old implementation.
    #[allow(dead_code)]
    fn serve_op(&mut self, op: EmmcOp) -> Result<(), EmmcError> {
        let bounce_buffer_address: usize = self.bounce_buffer.addr().try_into().unwrap();
        assert!(bounce_buffer_address < (1 << 32));

        match op {
            EmmcOp::Read {
                block_index,
                block_count,
                address: _,
            } => {
                assert_eq!(block_count, 1);

                let data_transfer_kind = SdhciDataTransferKind::DmaTransfer(
                    SdhciDmaTransfer::Sdma {
                        address: bounce_buffer_address as u32,
                    },
                );
                let kind = SdhciCommandKind::DataTransfer(SdhciDataTransfer {
                    block_size: SdhciBlockSize::new()
                        .with_transfer_block_size(self.block_size)
                        .with_host_sdma_buffer_boundary(0),
                    block_count: 1,
                    transfer_mode: SdhciTransferMode::new()
                        .with_dma_enable(true)
                        .with_block_count_enable(false)
                        .with_auto_cmd12_enable(false)
                        .with_data_transfer_direction_select(SdhciTransferingDirection::Read)
                        .with_multi_block_select(false),
                    data_transfer_kind,
                });
                let command = SdhciCommand {
                    argument: block_index,
                    command_desc: R1_COMMAND_PRESET
                        .with_data_present(true)
                        .with_command_type(SdhciCommandType::Normal)
                        .with_index(17),
                    kind,
                };

                self.sdhci_slot.send_command(&command)?;
                self.sdhci_slot.wait_for_command_complete()?;

                debug!("Reading from slot {} using SDMA", self.sdhci_slot.slot_num);
                Ok(())
            },
            EmmcOp::Write {
                block_index,
                block_count,
                address,
            } => {
                assert_eq!(block_count, 1);

                unsafe {
                    core::ptr::copy_nonoverlapping(
                        address as *const u8,
                        bounce_buffer_address as *mut u8,
                        self.block_size as usize,
                    );
                }

                let data_transfer_kind = SdhciDataTransferKind::DmaTransfer(
                    SdhciDmaTransfer::Sdma {
                        address: bounce_buffer_address as u32,
                    },
                );
                let kind = SdhciCommandKind::DataTransfer(SdhciDataTransfer {
                    block_size: SdhciBlockSize::new()
                        .with_transfer_block_size(self.block_size)
                        .with_host_sdma_buffer_boundary(0),
                    block_count: 1,
                    transfer_mode: SdhciTransferMode::new()
                        .with_dma_enable(true)
                        .with_block_count_enable(false)
                        .with_auto_cmd12_enable(false)
                        .with_data_transfer_direction_select(SdhciTransferingDirection::Write)
                        .with_multi_block_select(false),
                    data_transfer_kind,
                });
                let command = SdhciCommand {
                    command_desc: R1_COMMAND_PRESET
                        .with_data_present(true)
                        .with_command_type(SdhciCommandType::Normal)
                        .with_index(24),
                    argument: block_index,
                    kind,
                };

                self.sdhci_slot.send_command(&command)?;
                self.sdhci_slot.wait_for_command_complete()?;

                debug!("Writing to slot {} using SDMA", self.sdhci_slot.slot_num);
                Ok(())
            },
        }
    }

    fn prepare_adma2_descriptors(&mut self, block_count: u16) -> u64 {
        let buffer_address: usize = self.bounce_buffer.addr().try_into().unwrap();
        let descriptor_table_address = self.adma_descriptor_table.physical_address();

        assert!(block_count > 0);
        assert!(block_count <= MAX_BLOCKS_PER_TRANSFER);

        self.adma_descriptor_table.clear();

        for index in 0..block_count as usize {
            let descriptor_buffer_address =
                buffer_address + index * self.block_size as usize;

            assert!(descriptor_buffer_address < (1 << 32));
            assert!(self.adma_descriptor_table.push_transfer(
                descriptor_buffer_address as u32,
                self.block_size,
            ));
        }

        assert!(self.adma_descriptor_table.finish());

        debug!(
            "ADMA2 table ready: address=0x{descriptor_table_address:X}, descriptors={}, total_length={}",
            self.adma_descriptor_table.len,
            self.block_size as usize * block_count as usize,
        );

        descriptor_table_address as u64
    }

    fn set_block_count(&mut self, block_count: u16) -> Result<(), EmmcError> {
        let command = SdhciCommand {
            command_desc: R1_COMMAND_PRESET
                .with_data_present(false)
                .with_command_type(SdhciCommandType::Normal)
                .with_index(23),
            argument: block_count as u32,
            kind: SdhciCommandKind::NonDatCommand,
        };

        self.sdhci_slot.send_command(&command)?;
        self.sdhci_slot.wait_for_command_complete()?;

        Ok(())
    }

    fn prepare_batch_write(&self, batch: &EmmcBatch) {
        let bounce_buffer_address: usize = self.bounce_buffer.addr().try_into().unwrap();
        let mut offset = 0;

        for operation in batch.operations.iter() {
            let EmmcOp::Write { block_count, address, .. } = operation else {
                unreachable!();
            };
            let transfer_size = self.block_size as usize * *block_count as usize;

            unsafe {
                core::ptr::copy_nonoverlapping(
                    *address as *const u8,
                    (bounce_buffer_address + offset) as *mut u8,
                    transfer_size,
                );
            }

            offset += transfer_size;
        }
    }

    fn complete_batch_read(&self, batch: &EmmcBatch) {
        let bounce_buffer_address: usize = self.bounce_buffer.addr().try_into().unwrap();
        let mut offset = 0;

        for operation in batch.operations.iter() {
            let EmmcOp::Read { block_count, address, .. } = operation else {
                unreachable!();
            };
            let transfer_size = self.block_size as usize * *block_count as usize;

            unsafe {
                core::ptr::copy_nonoverlapping(
                    (bounce_buffer_address + offset) as *const u8,
                    *address as *mut u8,
                    transfer_size,
                );
            }

            offset += transfer_size;
        }
    }

    fn serve_batch(&mut self, batch: &EmmcBatch) -> Result<(), EmmcError> {
        assert!(batch.block_count > 0);
        assert!(batch.block_count <= MAX_BLOCKS_PER_TRANSFER);

        let multi_block = batch.block_count > 1;

        if multi_block {
            self.set_block_count(batch.block_count)?;
        }

        let transfer_direction = match batch.direction {
            EmmcDirection::Read => SdhciTransferingDirection::Read,
            EmmcDirection::Write => {
                self.prepare_batch_write(batch);
                SdhciTransferingDirection::Write
            },
        };

        let command_index = match (batch.direction, multi_block) {
            (EmmcDirection::Read, false) => 17,
            (EmmcDirection::Read, true) => 18,
            (EmmcDirection::Write, false) => 24,
            (EmmcDirection::Write, true) => 25,
        };

        let descriptor_table_address =
            self.prepare_adma2_descriptors(batch.block_count);
        let data_transfer_kind = SdhciDataTransferKind::DmaTransfer(
            SdhciDmaTransfer::Adma2 {
                descriptor_table_address,
            },
        );
        let kind = SdhciCommandKind::DataTransfer(SdhciDataTransfer {
            block_size: SdhciBlockSize::new()
                .with_transfer_block_size(self.block_size)
                .with_host_sdma_buffer_boundary(0),
            block_count: batch.block_count,
            transfer_mode: SdhciTransferMode::new()
                .with_dma_enable(true)
                .with_block_count_enable(multi_block)
                .with_auto_cmd12_enable(false)
                .with_data_transfer_direction_select(transfer_direction)
                .with_multi_block_select(multi_block),
            data_transfer_kind,
        });
        let command = SdhciCommand {
            argument: batch.first_block,
            command_desc: R1_COMMAND_PRESET
                .with_data_present(true)
                .with_command_type(SdhciCommandType::Normal)
                .with_index(command_index),
            kind,
        };

        self.sdhci_slot.send_command(&command)?;
        self.sdhci_slot.wait_for_command_complete()?;

        Ok(())
    }

    fn print_read_result(&self, batch: &EmmcBatch) {
        for operation in batch.operations.iter() {
            let EmmcOp::Read {
                block_index,
                block_count,
                address,
            } = operation
            else {
                continue;
            };

            let transfer_size =
                self.block_size as usize * *block_count as usize;

            if transfer_size < EMMC_TEST_SAMPLE_SIZE {
                continue;
            }

            let data = unsafe {
                core::slice::from_raw_parts(
                    *address as *const u8,
                    EMMC_TEST_SAMPLE_SIZE,
                )
            };

            let pattern_ok = data
                .iter()
                .enumerate()
                .all(|(index, byte)| {
                    *byte == EMMC_TEST_PATTERN[index % EMMC_TEST_PATTERN.len()]
                });

            info!("================ eMMC READ ================");
            info!("Block {} data:", block_index);
            info!(
                "{:02X} {:02X} {:02X} {:02X}  {:02X} {:02X} {:02X} {:02X}  {:02X} {:02X} {:02X} {:02X}  {:02X} {:02X} {:02X} {:02X}",
                data[0], data[1], data[2], data[3],
                data[4], data[5], data[6], data[7],
                data[8], data[9], data[10], data[11],
                data[12], data[13], data[14], data[15],
            );
            info!(
                "Expected: DE AD BE EF repeated -> {}",
                if pattern_ok { "OK" } else { "MISMATCH" },
            );
            info!("===========================================");
        }
    }
}

// Build desired Operation Conditions from slot capabilities
fn slot_capabilities_to_ocr(slot: &SdhciSlot) -> Ocr {
    let slot_cap = slot.sdhci_interface.capabilities();
    Ocr::new()
        .with_from_1_70_v_to_1_95_v(slot_cap.voltage_1_8_support())
        .with_v2_0(false)
        .with_v2_1(false)
        .with_v2_2(false)
        .with_v2_3(false)
        .with_v2_4(false)
        .with_v2_5(false)
        .with_v2_6(false)
        .with_v2_7(false)
        .with_v2_8(false)
        .with_v2_9(false)
        .with_v3_0(slot_cap.voltage_3_0_support())
        .with_v3_1(false)
        .with_v3_2(false)
        .with_v3_3(slot_cap.voltage_3_3_support())
        .with_v3_4(false)
        .with_v3_5(false)
        .with_access_mode(AccessMode::Sector) // NOTE: Slot is always capable of sector access
        .with_powered_up(false)
}

fn main_routine(mut slot: SdhciSlot) -> Result<(), EmmcError> {
    let mut emmc = Emmc::new(slot)?;

    let mut current: Option<EmmcBatch> = None;
    let mut lookahead: Option<EmmcOp> = None;

    // TODO: Check CSD:
    // enable high speed and widest bus possible
    // check block size

    #[cfg(any(feature = "emmc-read-test", feature = "emmc-write-test", feature = "emmc-mass-write-test"))]
    unsafe {
        let buff = PhysicalAllocator::alloc_contiguous_aligned(512, 512).unwrap();
        let buff_addr: usize = buff.addr().try_into().unwrap();

        debug!("Allocated buffer for TEST: 0x{buff_addr:X}");
        assert!(buff_addr < (1 << 32));

        #[cfg(feature = "emmc-read-test")]
        {
            put_into_queue(EmmcDriverMessage::Op(EmmcOp::Read { block_index: 0, block_count: 1, address: buff_addr }), EMMC_DRIVER_QUEUE.as_view());
        }
        #[cfg(feature = "emmc-write-test")]
        {
            let buff = core::slice::from_raw_parts_mut(buff_addr as *mut u8, 512);
            for byte in buff.iter_mut() {
                *byte = 128;
            }
            put_into_queue(EmmcDriverMessage::Op(EmmcOp::Write { block_index: 0, block_count: 1, address: buff_addr }), EMMC_DRIVER_QUEUE.as_view());
            put_into_queue(EmmcDriverMessage::Op(EmmcOp::Read { block_index: 0, block_count: 1, address: buff_addr }), EMMC_DRIVER_QUEUE.as_view());
        }
        #[cfg(feature = "emmc-mass-write-test")]
        {
            let buff = core::slice::from_raw_parts_mut(buff_addr as *mut u8, 512);
            for byte in buff.iter_mut() {
                *byte = 128;
            }
            for i in 0..32 {
                put_into_queue(EmmcDriverMessage::Op(EmmcOp::Write { block_index: i, block_count: 1, address: buff_addr }), EMMC_DRIVER_QUEUE.as_view());
            }
        }
    }

    loop {
        unsafe {
            if INTERRUPT_RECEIVED {
                match slot.wait_for_transfer_complete() {
                    Ok(_) => { 
                        if let Some(batch) = current.as_ref() {
                            match batch.direction {
                                EmmcDirection::Read => {
                                    emmc.complete_batch_read(batch);
                                    #[cfg(any(feature = "emmc-read-test", feature = "emmc-write-test", feature = "emmc-mass-write-test"))]
                                    {
                                        emmc.print_read_result(batch);
                                    }
                                },
                                EmmcDirection::Write => {},
                            }
                        }
                    },
                    Err(err) => error!("DMA error: {:?}", err),
                }

                current = None;
                INTERRUPT_RECEIVED = false;
            }
        }

        if current.is_none() {
            let op = take_next_operation(&mut lookahead);
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
                    let batch = build_batch(op, &mut lookahead);
                    match emmc.serve_batch(&batch) {
                        Ok(_) => current = Some(batch),
                        Err(err) => {
                            error!("Slot {} error: {:?}", slot.slot_num, err);
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
}

fn try_recover(mut slot: SdhciSlot, err: EmmcRecoverableError) -> Result<(), EmmcUnrecoverableError> {
    match err {
        EmmcRecoverableError::InvalidConditions(emmc_ocr) => {
            let slot_cap = slot.sdhci_interface.capabilities();

            // TODO: Check for errors
            if slot_cap.voltage_3_3_support() && emmc_ocr.v3_3() {
                slot.power_up(SdhciOperatingVoltage::V3_3)?;
                Ok(())
            }
            else if slot_cap.voltage_3_0_support() && emmc_ocr.v3_0() {
                slot.power_up(SdhciOperatingVoltage::V3_0)?;
                Ok(())
            }
            else if slot_cap.voltage_1_8_support() && emmc_ocr.from_1_70_v_to_1_95_v() {
                slot.power_up(SdhciOperatingVoltage::V1_8)?;
                Ok(())
            }
            else {
                Err(EmmcUnrecoverableError::Incompatible)
            }
        }
    }
}

pub extern "C" fn driver_task() -> ! {
    unsafe {
        // TODO: Get this structure from bus-driver using some id of device, to which driver is attached
        // If there is no sdhci host or no 0th slot - panic
        let slot = crate::sdhci::HOST.as_mut().unwrap().slots[0].unwrap();

        loop {
            let res = main_routine(slot);

            let res = match res {
                Err(EmmcError::Recoverable(err)) => try_recover(slot, err),
                Err(EmmcError::Unrecoverable(err)) => Err(err),
                Ok(_) => Ok(()),
            };

            // Unrecoverable error encountered
            if let Err(err) = res {
                error!("SDHCI slot {} unrecoverable error: {:?}", slot.slot_num, err);
                break;
            }
        }
    }

    loop { }
}
