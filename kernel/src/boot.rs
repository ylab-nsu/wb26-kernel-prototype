use alloc::vec::Vec;

use crate::arch::{AddressSpace, Mapping};

pub struct BootContext {
    pub address_space: AddressSpace,
    pub mappings: Vec<Mapping>,
}
