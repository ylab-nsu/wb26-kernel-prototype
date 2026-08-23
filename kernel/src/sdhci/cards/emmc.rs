use bitfield_struct::{ bitfield, bitenum };
use core::ptr::{read_volatile, write_volatile};
use crate::sdhci::{ Command, CommandType, ResponseType, CommandError, Slot };

const R3_COMMAND_PRESET: Command = Command::new()
    .with_response_type(ResponseType::ResponseLen48)
    .with_crc_enable(false)
    .with_index_check_enable(false);

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
            // Send CMD1
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

        Ok(())
    }
}

pub extern "C" fn driver_task() -> ! {
    unsafe {
        let slot = crate::sdhci::host.as_mut().unwrap().slots[0];

        match slot {
            Some(slot) => {
                let mut emmc = Emmc::new(slot);

                emmc.init();
            },
            None => error!("There is no 0th slot in sdhci"),
        }
    }

    loop { }
}
