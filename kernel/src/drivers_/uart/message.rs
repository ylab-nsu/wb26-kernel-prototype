use heapless::Vec;

pub const MAX_RECEVIED_BYTES: usize = 16;

pub enum UartDriverMessage {
    Receive {
		data: Vec<u8, MAX_RECEVIED_BYTES>,
    },
    Send,
    LineStatus,
    ModemStatus,
}