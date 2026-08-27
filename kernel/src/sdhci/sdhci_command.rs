use crate::sdhci::{ SdhciCommandDesc, SdhciBlockSize, SdhciTransferMode };

pub struct SdhciCommand {
    pub argument: u32,
    pub command_desc: SdhciCommandDesc,
    pub kind: SdhciCommandKind,
}

pub enum SdhciCommandKind {
    NonDatCommand,
    DataTransfer(SdhciDataTransfer)
}

pub struct SdhciDataTransfer {
    pub block_size: SdhciBlockSize,
    pub block_count: u16,
    pub transfer_mode: SdhciTransferMode,
    pub data_transfer_kind: SdhciDataTransferKind,
}

pub enum SdhciDataTransferKind {
    CpuTransfer,
    DmaTransfer(SdhciDmaTransfer),
}

pub enum SdhciDmaTransfer {
    Sdma {
        address: u32,
    },
    Adma2 {
        descriptor_table_address: u64,
    },
}
