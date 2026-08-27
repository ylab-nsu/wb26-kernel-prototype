use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::{
    arch::{
        riscv::memory::layout::KERNEL_LAYOUT,
        traits::{TargetAddressSpace, TargetPhysicalAllocator},
        AddressSpace, Mapping, PhysicalAddress, PhysicalAllocator, VirtualAddress,
    },
    sync::Once,
    vm::{MappingFlags, MappingPermissions},
};

// Temporary MMU tables and per-thread kernel stacks live in `.page_table_pool`
// after `__epage_table_pool`; `KERNEL_LAYOUT` does not know them.
unsafe extern "C" {
    #[link_name = "__s_temp_mmu_table"]
    static S_TEMP_MMU_TABLE: u8;
    #[link_name = "__e_temp_mmu_table"]
    static E_TEMP_MMU_TABLE: u8;
    #[link_name = "__s_temp_kernel_stacks"]
    static S_TEMP_KERNEL_STACKS: u8;
    #[link_name = "__e_temp_kernel_stacks"]
    static E_TEMP_KERNEL_STACKS: u8;
}

/// The kernel's runtime regions, one source of truth for the boot address
/// space and for sharing the kernel half into process address spaces.
pub fn kernel_sections() -> [(&'static str, usize, usize, MappingPermissions); 9] {
    let layout = &KERNEL_LAYOUT;
    [
        ("text", layout.stext, layout.etext, MappingPermissions::rx()),
        ("rodata", layout.srodata, layout.erodata, MappingPermissions::ro()),
        ("data", layout.sdata, layout.edata, MappingPermissions::rw()),
        ("bss", layout.sbss, layout.ebss, MappingPermissions::rw()),
        ("heap", layout.sheap, layout.eheap, MappingPermissions::rw()),
        (
            "page_table_pool",
            layout.spage_table_pool,
            layout.epage_table_pool,
            MappingPermissions::rw(),
        ),
        (
            "temp_mmu_table",
            unsafe { &S_TEMP_MMU_TABLE as *const u8 as usize },
            unsafe { &E_TEMP_MMU_TABLE as *const u8 as usize },
            MappingPermissions::rw(),
        ),
        (
            "kernel_stacks",
            unsafe { &S_TEMP_KERNEL_STACKS as *const u8 as usize },
            unsafe { &E_TEMP_KERNEL_STACKS as *const u8 as usize },
            MappingPermissions::rw(),
        ),
        ("stack", layout.estack, layout.sstack, MappingPermissions::rw()),
    ]
}

/// Canonical mappings of the kernel half, written once at boot. Process
/// address spaces share the same backing allocations by copying these mappings
/// (`map_shared`), so the kernel half is read-only afterwards.
pub static KERNEL_MAPPINGS: Once<Vec<Mapping>> = Once::new();

pub fn map_kernel_sections(address_space: &mut AddressSpace) {
    info!("Mapping sections into kernel address space");
    let va_offset = KERNEL_LAYOUT.kernel_va_offset;

    let mut registry = Vec::new();
    for (name, start, end, perms) in kernel_sections() {
        let size = end - start;
        if size == 0 {
            continue;
        }

        let phys = PhysicalAddress::try_from(start - va_offset).unwrap();
        let alloc = PhysicalAllocator::alloc_contiguous_at(phys, size).unwrap();

        info!("Map {name} from 0x{start:x} to 0x{phys:x}, size: {size:x}, perms: {perms}");

        let mapping = address_space.map(
            VirtualAddress::try_from(start).unwrap(),
            Arc::new(alloc),
            perms,
            MappingFlags::new(),
        );
        registry.push(mapping);
    }
    KERNEL_MAPPINGS.call_once(|| registry);
}