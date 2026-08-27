use alloc::vec::Vec;

use crate::{
    arch::{
        riscv::memory::layout::KERNEL_LAYOUT,
        traits::{TargetAddressSpace, TargetPhysicalAllocator},
        AddressSpace, Mapping, PhysicalAddress, PhysicalAllocator, VirtualAddress,
    },
    vm::{MappingFlags, MappingPermissions},
};

fn map_section(
    name: &str,
    virt_start: usize,
    virt_end: usize,
    permissions: MappingPermissions,
    flags: MappingFlags,
    va_offset: usize,
    address_space: &mut AddressSpace,
) -> Mapping {
    let size = virt_end - virt_start;
    let phys_addr = PhysicalAddress::try_from(virt_start - va_offset).unwrap();
    let phys_alloc = PhysicalAllocator::alloc_contiguous_at(phys_addr, size).unwrap();

    info!("Map {name} from 0x{virt_start:x} to 0x{phys_addr:x}, size: {size:x}, perms: {permissions}, flags: {flags}");

    address_space.map(
        VirtualAddress::try_from(virt_start).unwrap(),
        phys_alloc,
        permissions,
        flags,
    )
}

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

pub fn map_kernel_sections(address_space: &mut AddressSpace) -> Vec<Mapping> {
    info!("Mapping sections into kernel address space");

    kernel_sections()
        .into_iter()
        .map(|(name, start, end, perms)| {
            map_section(
                name,
                start,
                end,
                perms,
                MappingFlags::new(),
                KERNEL_LAYOUT.kernel_va_offset,
                address_space,
            )
        })
        .collect()
}
