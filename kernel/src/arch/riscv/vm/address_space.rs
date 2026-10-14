use alloc::sync::Arc;
use alloc::vec::Vec;
use riscv::{asm::sfence_vma_all, register::satp};

use crate::sync::Mutex;
use crate::{
    arch::{
        common::page_table::pool::{PageTablePool, PageTableRef},
        riscv::{
            memory::layout::KERNEL_LAYOUT,
            vm::{page_table::Sv39PageTable, PAGE_SIZE},
        },
        traits::{TargetAddress, TargetAddressSpace, TargetMapping, TargetPhysicalAllocation},
        Mapping, PhysicalAddress, PhysicalAllocation, VirtualAddress,
    },
    vm::{MappingFlags, MappingPermissions},
};

#[link_section = ".page_table_pool"]
pub static PAGE_TABLE_POOL: PageTablePool<Sv39PageTable> = unsafe { PageTablePool::new() };

pub fn init_page_table_pool() {
    let page_table_pool_phys_addr =
        PhysicalAddress::try_from(KERNEL_LAYOUT.spage_table_pool - KERNEL_LAYOUT.kernel_va_offset)
            .unwrap();
    unsafe { PAGE_TABLE_POOL.set_pool_phys_address(page_table_pool_phys_addr) };
}

pub struct Sv39AddressSpace {
    root_page_table: PageTableRef<Sv39PageTable>,
    mappings: Mutex<Vec<Sv39Mapping>>,
}

impl Sv39AddressSpace {
    pub fn new() -> Self {
        let root_page_table = PAGE_TABLE_POOL.alloc_page_table();

        Sv39AddressSpace {
            root_page_table,
            mappings: Mutex::new(Vec::new()),
        }
    }

    fn get_l2_page_table(&self, virt_addr: VirtualAddress) -> PageTableRef<Sv39PageTable> {
        let l0 = &self.root_page_table;
        let l1 = l0.get_next_level_table(virt_addr.vpn_2()).unwrap();
        let l2 = l1.get_next_level_table(virt_addr.vpn_1()).unwrap();

        l2
    }

    fn map_page(
        &self,
        virt_addr: VirtualAddress,
        phys_addr: PhysicalAddress,
        permissions: MappingPermissions,
        flags: MappingFlags,
    ) {
        self.get_l2_page_table(virt_addr).write_leaf(
            virt_addr.vpn_0(),
            phys_addr,
            permissions,
            flags,
        );
    }

    fn unmap_page(&self, virt_addr: VirtualAddress) {
        self.get_l2_page_table(virt_addr)
            .write_invalid(virt_addr.vpn_0());
    }
}

impl TargetAddressSpace for Sv39AddressSpace {
    fn map(
        &self,
        virt_addr: VirtualAddress,
        phys_alloc: Arc<PhysicalAllocation>,
        permissions: MappingPermissions,
        flags: MappingFlags,
    ) -> Mapping {
        let size = phys_alloc.size();
        let phys_addr = phys_alloc.addr();

        for offset in (0..size).step_by(PAGE_SIZE) {
            let va = virt_addr.byte_add(offset);
            let pa = phys_addr.byte_add(offset);

            // debug!("Map {va:p} {pa:p}");
            self.map_page(va, pa, permissions, flags);
        }

        let mapping = Sv39Mapping {
            virt_addr,
            phys_alloc,
            permissions,
            flags,
        };
        self.mappings.lock().push(mapping.clone());
        mapping
    }

    // No one ever tested this method
    unsafe fn unmap(&self, mapping: &impl TargetMapping) {
        for offset in (0..mapping.size()).step_by(PAGE_SIZE) {
            let va = mapping.virt_addr().byte_add(offset);
            self.unmap_page(va);
        }

        let vaddr = mapping.virt_addr();
        self.mappings.lock().retain(|m| m.virt_addr != vaddr);
    }

    fn map_shared(&self, src: &impl TargetMapping, dest_vaddr: VirtualAddress) -> Mapping {
        self.map(
            dest_vaddr,
            src.alloc().clone(),
            src.permissions(),
            src.flags(),
        )
    }

    unsafe fn switch(&self) {
        let pa = self.root_page_table.get_phys_addr();
        let ppn = pa.into_bits() >> 12;
        debug!("Switch to an address space with PPN 0x{ppn:x}");

        satp::set(satp::Mode::Sv39, 0, ppn as usize);
        sfence_vma_all();
    }
}

#[derive(Clone)]
pub struct Sv39Mapping {
    virt_addr: VirtualAddress,
    phys_alloc: Arc<PhysicalAllocation>,
    permissions: MappingPermissions,
    flags: MappingFlags,
}

impl TargetMapping for Sv39Mapping {
    fn virt_addr(&self) -> VirtualAddress {
        self.virt_addr
    }

    fn phys_addr(&self) -> PhysicalAddress {
        self.phys_alloc.addr()
    }

    fn size(&self) -> usize {
        self.phys_alloc.size()
    }

    fn permissions(&self) -> MappingPermissions {
        self.permissions
    }

    fn flags(&self) -> MappingFlags {
        self.flags
    }

    fn alloc(&self) -> &Arc<PhysicalAllocation> {
        &self.phys_alloc
    }
}

// Mapping is now inside its address space. We have unmap, but it doesn't work

// impl Drop for Sv39Mapping {
//     fn drop(&mut self) {
//         let address_space = unsafe { self.address_space.as_mut() };
//         unsafe { address_space.unmap(self.virt_addr(), self.size()) };
//     }
// }
