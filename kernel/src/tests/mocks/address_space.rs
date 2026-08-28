use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::arch::traits::{TargetAddressSpace, TargetMapping, TargetPhysicalAllocation};
use crate::arch::{Mapping, PhysicalAllocation, VirtualAddress};
use crate::vm::{MappingFlags, MappingPermissions};

/// A recorded map/unmap/map_shared call.
pub struct MockRecord {
    pub vaddr: usize,
    pub size: usize,
    pub phys_addr: usize,
    pub perms: MappingPermissions,
    pub flags: MappingFlags,
}

/// Records every mapping call instead of touching real page tables.
pub struct MockAddressSpace {
    pub records: Vec<MockRecord>,
}

impl MockAddressSpace {
    pub fn new() -> Self {
        MockAddressSpace {
            records: Vec::new(),
        }
    }
}

impl TargetAddressSpace for MockAddressSpace {
    fn map(
        &mut self,
        vaddr: VirtualAddress,
        alloc: Arc<PhysicalAllocation>,
        perms: MappingPermissions,
        flags: MappingFlags,
    ) -> Mapping {
        self.records.push(MockRecord {
            vaddr: vaddr.try_into().unwrap(),
            size: alloc.size(),
            phys_addr: alloc.addr().into_bits() as usize,
            perms,
            flags,
        });
        Mapping::new(vaddr, alloc, perms, flags)
    }

    unsafe fn unmap(&mut self, _mapping: &Mapping) {}

    fn map_shared(&mut self, src: &Mapping, dest_vaddr: VirtualAddress) -> Mapping {
        self.records.push(MockRecord {
            vaddr: dest_vaddr.try_into().unwrap(),
            size: src.size(),
            phys_addr: src.phys_addr().into_bits() as usize,
            perms: src.permissions(),
            flags: src.flags(),
        });
        Mapping::new(dest_vaddr, src.alloc().clone(), src.permissions(), src.flags())
    }

    unsafe fn switch(&self) {}
}