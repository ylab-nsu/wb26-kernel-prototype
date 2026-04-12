use crate::{arch::traits::TargetAddressSpace, vm::{MapperError, Mapping, MappingFlags}};

pub struct RiscvSv39AddressSpace;

impl TargetAddressSpace for RiscvSv39AddressSpace {
    type PhysicalAddress = usize;

    type VirtualAddress = usize;

    fn map(&mut self, vaddr: Self::VirtualAddress, paddr: Self::PhysicalAddress, flags: MappingFlags) -> Result<Mapping, MapperError> {
        let new_mapping = Mapping {
            vaddr,
            paddr,
            length: 2,
            flags,
            address_space_ref: core::ptr::null_mut(),
        };

        // self.0.push(vaddr as u8);
        // self.0 += 1;

        println!("Create Mapping");
        Ok(new_mapping)
    }

    unsafe fn unmap(&mut self, mapping: &Mapping) {
        // self.0 -= 1;

        // if let Some(v) = self.0.pop() {
        //     println!("POp {}", v);
        // } else {
        //     println!("No Pop");
        // }

        println!("Unmap Mapping");
    }

    unsafe fn switch(&self) {
        todo!()
    }
}


impl Drop for RiscvSv39AddressSpace {
    fn drop(&mut self) {
        println!("Drop AddressSpace");
    }
}
