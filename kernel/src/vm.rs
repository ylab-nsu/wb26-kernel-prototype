use crate::arch::AddressSpace;

#[derive(Debug)]
pub enum MapperError {
    Ivalid,
    NotSupported
}

pub struct MappingFlags;

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
