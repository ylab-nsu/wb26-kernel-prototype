use crate::arch::AddressSpace;

#[derive(Debug)]
pub enum MapperError {
    Ivalid,
    NotSupported
}

pub struct Mapping {
    pub vaddr: usize,
    pub paddr: usize,
    pub length: isize,
    pub flags: MappingFlags,
    pub address_space_ref: *mut AddressSpace,
}

impl Mapping {
    pub fn unmap(self) {
        core::mem::drop(self);
    }
}

impl Drop for Mapping {
    fn drop(&mut self) {
        println!("Drop Mapping");
        // unsafe { (*self.address_space_ref).unmap(self) };
    }
}

use bitfield_struct::bitfield;

#[bitfield(u8)]
pub struct MappingPermissions {
    pub read: bool,
    pub write: bool,
    pub execute: bool,

    #[bits(5)]
    __: usize,
}

impl MappingPermissions {
    pub const fn ro() -> Self {
        Self::new()
            .with_read(true)
    }

    pub const fn rw() -> Self {
        Self::new()
            .with_read(true)
            .with_write(true)
    }

    pub const fn rx() -> Self {
        Self::new()
            .with_read(true)
            .with_execute(true)
    }

    pub const fn rwx() -> Self {
        Self::new()
            .with_read(true)
            .with_write(true)
            .with_execute(true)
    }
}

#[bitfield(u8)]
pub struct MappingFlags {
    pub user: bool,
    pub global: bool,
    pub accessed: bool,
    pub dirty: bool,

    #[bits(4)]
    __: usize,
}
