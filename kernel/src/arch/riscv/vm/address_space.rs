use alloc::sync::Arc;
use alloc::vec::Vec;
use riscv::{asm::sfence_vma_all, register::satp};

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

/// A process address space.
///
/// An address space is a hierarchy of page tables rooted at `root_page_table`,
/// plus the list of mappings it owns. Each mapping holds an `Arc` reference to
/// its physical allocation, so the pages stay alive exactly as long as a
/// mapping references them. Sharing the AS between threads (`Clone`) takes one
/// more reference on the root table and clones every mapping (`Arc::clone`).
///
/// When the last thread holding this AS dies, the AS drops, releasing the root
/// table and every mapping; a physical allocation is freed once the last
/// mapping holding it is gone.
pub struct Sv39AddressSpace {
    root_page_table: PageTableRef<Sv39PageTable>,
    /// The mappings this AS has installed. Keeps each backing allocation alive
    /// while the AS lives; removed (dropped) by [`TargetAddressSpace::unmap`].
    mappings: Vec<Sv39Mapping>,
}

impl Clone for Sv39AddressSpace {
    /// Share this AS with another thread: one more reference on the root table
    /// and one more reference on every held allocation (Arc::clone).
    fn clone(&self) -> Self {
        let root_page_table = self.root_page_table.clone();
        let mappings = self.mappings.clone();
        Sv39AddressSpace {
            root_page_table,
            mappings,
        }
    }
}

impl Sv39AddressSpace {
    pub fn new() -> Self {
        let root_page_table = PAGE_TABLE_POOL.alloc_page_table();

        Sv39AddressSpace {
            root_page_table,
            mappings: Vec::new(),
        }
    }

    fn get_l2_page_table(&self, virt_addr: VirtualAddress) -> PageTableRef<Sv39PageTable> {
        let l0 = &self.root_page_table;
        let l1 = l0.get_next_level_table(virt_addr.vpn_2()).unwrap();
        let l2 = l1.get_next_level_table(virt_addr.vpn_1()).unwrap();

        l2
    }

    fn map_page(
        &mut self,
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

    fn unmap_page(&mut self, virt_addr: VirtualAddress) {
        self.get_l2_page_table(virt_addr).write_invalid(virt_addr.vpn_0());
    }
}

impl TargetAddressSpace for Sv39AddressSpace {
    fn map(
        &mut self,
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

            self.map_page(va, pa, permissions, flags);
        }

        let mapping = Sv39Mapping {
            vaddr: virt_addr,
            alloc: phys_alloc,
            permissions,
            flags,
        };
        self.mappings.push(mapping.clone());
        mapping
    }

    unsafe fn unmap(&mut self, mapping: &Mapping) {
        for offset in (0..mapping.size()).step_by(PAGE_SIZE) {
            let va = mapping.virt_addr().byte_add(offset);
            self.unmap_page(va);
        }

        let vaddr = mapping.virt_addr();
        self.mappings.retain(|m| m.vaddr != vaddr);
    }

    fn map_shared(&mut self, src: &Mapping, dest_vaddr: VirtualAddress) -> Mapping {
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
        debug!("Switch to new address space with PPN 0x{ppn:x}");

        satp::set(satp::Mode::Sv39, 0, ppn as usize);
        sfence_vma_all();
    }
}

/// A mapping installed into an address space: a virtual range backed by a
/// physical allocation. Owns its allocation through an `Arc`, so the pages
/// live exactly as long as some mapping (or the AS) references them.
#[derive(Clone)]
pub struct Sv39Mapping {
    vaddr: VirtualAddress,
    alloc: Arc<PhysicalAllocation>,
    permissions: MappingPermissions,
    flags: MappingFlags,
}

impl Sv39Mapping {
    pub fn new(
        vaddr: VirtualAddress,
        alloc: Arc<PhysicalAllocation>,
        permissions: MappingPermissions,
        flags: MappingFlags,
    ) -> Self {
        Sv39Mapping {
            vaddr,
            alloc,
            permissions,
            flags,
        }
    }

    /// The backing allocation this mapping references (for sharing).
    pub fn alloc(&self) -> &Arc<PhysicalAllocation> {
        &self.alloc
    }
}

impl TargetMapping for Sv39Mapping {
    fn virt_addr(&self) -> VirtualAddress {
        self.vaddr
    }

    fn phys_addr(&self) -> PhysicalAddress {
        self.alloc.addr()
    }

    fn size(&self) -> usize {
        self.alloc.size()
    }

    fn permissions(&self) -> MappingPermissions {
        self.permissions
    }

    fn flags(&self) -> MappingFlags {
        self.flags
    }
}