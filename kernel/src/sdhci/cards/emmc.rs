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
        let slot = crate::sdhci::host.as_mut().unwrap().slots[0];

        match slot {
            Some(slot) => {
                let mut emmc = Emmc::new(slot);

                match emmc.init() {
                    Ok(_) => {
                        let block_size = BlockSize::new()
                            .with_transfer_block_size(0x0200)
                            .with_host_sdma_buffer_boundary(0);
                        let transfer_mode = TransferMode::new()
                            .with_dma_enable(false)
                            .with_block_count_enable(false)
                            .with_auto_cmd12_enable(true)
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
                            Ok(_) => info!("Read: {:x?}", buffer),
                            Err(CommandError::IssuanceTimeout) => error!("Slot {} issuance timeout", slot.slot_num),
                            Err(CommandError::InterruptedError(err)) => error!("Slot {} inrerrupt error: {}", slot.slot_num, err.into_bits()),
                        }

                    },
                    Err(CommandError::IssuanceTimeout) => error!("Slot {} issuance timeout", slot.slot_num),
                    Err(CommandError::InterruptedError(err)) => error!("Slot {} inrerrupt error: {}", slot.slot_num, err.into_bits()),

                }
            },
            None => error!("There is no 0th slot in sdhci"),
        }
    }

    loop { }
}
