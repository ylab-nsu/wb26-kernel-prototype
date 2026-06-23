pub mod bitmap;

use core::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum AllocatorError {
    NotEnoughMemory,
    InvalidAlignment,
    InvalidAddress,
    AlreadyOccupied,
}

impl Display for AllocatorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let msg = match self {
            AllocatorError::NotEnoughMemory => "not enough memory",
            AllocatorError::InvalidAlignment => "invalid alignment",
            AllocatorError::InvalidAddress => "invalid address",
            AllocatorError::AlreadyOccupied => "memory is already occupied",
        };

        f.write_str(msg)
    }
}

impl Error for AllocatorError {}
