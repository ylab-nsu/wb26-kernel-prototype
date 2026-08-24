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
/// An address space is a hierarchy of page tables rooted at `root_page_table`.
/// It is a shared, reference-counted object: the root page table carries its own
/// refcount (`PageTableRef`), and the pages this AS has mapped are held by
/// retain references in `pages`. Cloning the AS (sharing it between threads)
/// takes one more reference on the root table and one more reference on every
/// held page — no new counters are introduced; the existing page/table refcounts
/// drive the whole lifetime.
///
/// Ownership cascade (top-down only, `AS -> page -> mapping`):
/// - pages are owned by this AS through `pages` (retain refs);
/// - each page owns its mappings (`PageMapping`, attached in the physical
///   allocator) and dies with the page;
/// - nothing references upward: pages/mappings do not know their AS.
///
/// When the last thread holding this AS dies, the AS drops, which releases the
/// root table and every held page. A page dies only when the last owning AS is
/// gone, so no manual PTE teardown is ever required.
pub struct Sv39AddressSpace {
    root_page_table: PageTableRef<Sv39PageTable>,
    /// Retain references to every page this AS has mapped. Keeps the pages
    /// alive while the AS lives; dropped (released) when the AS dies.
    pages: Vec<PhysicalAllocation>,
}

impl Clone for Sv39AddressSpace {
    /// Share this AS with another thread: one more reference on the root table
    /// and one more reference on every held page. Existing refcounts only.
    fn clone(&self) -> Self {
        let root_page_table = self.root_page_table.clone();
        let pages = self.pages.iter().map(|p| p.clone()).collect();
        Sv39AddressSpace {
            root_page_table,
            pages,
        }
    }
}

impl Sv39AddressSpace {
    pub fn new() -> Self {
        let root_page_table = PAGE_TABLE_POOL.alloc_page_table();

        Sv39AddressSpace {
            root_page_table,
            pages: Vec::new(),
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
}

impl TargetAddressSpace for Sv39AddressSpace {
    fn map(
        &mut self,
        virt_addr: VirtualAddress,
        phys_alloc: PhysicalAllocation,
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

        // The page owns this mapping (reverse mapping). The AS keeps ownership
        // of the page itself via a retain reference so it lives while the AS
        // lives; when the page dies, its mappings die with it.
        let vaddr_usize: usize = virt_addr.try_into().unwrap();
        phys_alloc.attach_mapping(vaddr_usize, permissions, flags);
        self.pages.push(phys_alloc);

        Sv39Mapping {
            vaddr: virt_addr,
            addr: phys_addr,
            size,
            permissions,
            flags,
        }
    }

    unsafe fn switch(&self) {
        let pa = self.root_page_table.get_phys_addr();
        let ppn = pa.into_bits() >> 12;
        debug!("Switch to new address space with PPN 0x{ppn:x}");

        satp::set(satp::Mode::Sv39, 0, ppn as usize);
        sfence_vma_all();
    }
}

/// Lightweight read-only view of a mapping, returned from [`TargetAddressSpace::map`].
///
/// This does NOT own anything: ownership lives in the page (reverse mapping,
/// kept alive by the AS via its `pages` list). It exists to satisfy the
/// `TargetMapping` interface (virtual/physical address, size, permissions).
#[derive(Clone, Copy)]
pub struct Sv39Mapping {
    vaddr: VirtualAddress,
    addr: PhysicalAddress,
    size: usize,
    permissions: MappingPermissions,
    flags: MappingFlags,
}

impl TargetMapping for Sv39Mapping {
    fn virt_addr(&self) -> VirtualAddress {
        self.vaddr
    }

    fn phys_addr(&self) -> PhysicalAddress {
        self.addr
    }

    fn size(&self) -> usize {
        self.size
    }

    fn permissions(&self) -> MappingPermissions {
        self.permissions
    }

    fn flags(&self) -> MappingFlags {
        self.flags
    }
}
