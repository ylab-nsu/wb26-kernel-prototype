use alloc::vec::Vec;

use crate::{
    arch::{
        riscv::alloc::phys::{add_mapping, mappings_of, remove_mapping, PageMapping},
        traits::TargetPhysicalAllocation,
        PhysicalAddress, PhysicalAllocator,
    },
    vm::{MappingFlags, MappingPermissions},
};

pub mod phys;

pub struct RiscvPhysicalAllocation {
    addr: PhysicalAddress,
    size: usize,
}

impl RiscvPhysicalAllocation {
    pub(crate) fn new(addr: PhysicalAddress, size: usize) -> Self {
        RiscvPhysicalAllocation { addr, size }
    }

    /// Attach a mapping (virtual position/perms) to every page of this range.
    /// Mappings are owned by the pages and die with them.
    pub fn attach_mapping(&self, vaddr: usize, permissions: MappingPermissions, flags: MappingFlags) {
        add_mapping(
            self.addr,
            self.size,
            PageMapping {
                phys_addr: self.addr.into_bits() as usize,
                vaddr,
                permissions,
                flags,
            },
        );
    }

    /// Read the mappings attached to the first page of this range.
    pub fn mappings(&self) -> Vec<PageMapping> {
        mappings_of(self.addr)
    }

    /// Detach the mapping at `vaddr` from every page of this range.
    pub fn detach_mapping(&self, vaddr: usize) {
        remove_mapping(self.addr, self.size, vaddr);
    }
}

impl Clone for RiscvPhysicalAllocation {
    /// Take one more reference on the same physical range (refcount++).
    /// Used when the same pages are shared into another address space.
    fn clone(&self) -> Self {
        PhysicalAllocator::retain(self.addr, self.size).expect("retain failed");
        RiscvPhysicalAllocation {
            addr: self.addr,
            size: self.size,
        }
    }
}

impl core::fmt::Debug for RiscvPhysicalAllocation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RiscvPhysicalAllocation")
            .field("addr", &format_args!("{:p}", self.addr))
            .field("size", &self.size)
            .finish()
    }
}

impl core::fmt::Display for RiscvPhysicalAllocation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{} bytes at {:p}", self.size, self.addr))
    }
}

impl TargetPhysicalAllocation for RiscvPhysicalAllocation {
    fn addr(&self) -> PhysicalAddress {
        self.addr
    }

    fn size(&self) -> usize {
        self.size
    }
}

impl Drop for RiscvPhysicalAllocation {
    fn drop(&mut self) {
        unsafe { PhysicalAllocator::dealloc_contiguous(self.addr, self.size) };
    }
}
