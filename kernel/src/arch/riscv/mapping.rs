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

pub fn map_kernel_sections(address_space: &mut AddressSpace) -> Vec<Mapping> {
    info!("Mapping sections into kernel address space");

    let sections = [
        (
            "text",
            KERNEL_LAYOUT.stext,
            KERNEL_LAYOUT.etext,
            MappingPermissions::rx(),
            MappingFlags::new(),
        ),
        (
            "rodata",
            KERNEL_LAYOUT.srodata,
            KERNEL_LAYOUT.erodata,
            MappingPermissions::ro(),
            MappingFlags::new(),
        ),
        (
            "data",
            KERNEL_LAYOUT.sdata,
            KERNEL_LAYOUT.edata,
            MappingPermissions::rw(),
            MappingFlags::new(),
        ),
        (
            "bss",
            KERNEL_LAYOUT.sbss,
            KERNEL_LAYOUT.ebss,
            MappingPermissions::rw(),
            MappingFlags::new(),
        ),
        // (
        //     "uninit",
        //     KERNEL_LAYOUT.suninit,
        //     KERNEL_LAYOUT.euninit,
        //     MappingPermissions::rx(),
        //     MappingFlags::new(),
        // ),
        (
            "heap",
            KERNEL_LAYOUT.sheap,
            KERNEL_LAYOUT.eheap,
            MappingPermissions::rw(),
            MappingFlags::new(),
        ),
        (
            "page_table_pool",
            KERNEL_LAYOUT.spage_table_pool,
            KERNEL_LAYOUT.epage_table_pool,
            MappingPermissions::rw(),
            MappingFlags::new(),
        ),
        (
            "stack",
            KERNEL_LAYOUT.estack,
            KERNEL_LAYOUT.sstack,
            MappingPermissions::rw(),
            MappingFlags::new(),
        ),
    ];

    let mappings: Vec<Mapping> = sections
        .into_iter()
        .map(|(name, virt_start, virt_end, permissions, flags)| {
            map_section(
                name,
                virt_start,
                virt_end,
                permissions,
                flags,
                KERNEL_LAYOUT.kernel_va_offset,
                address_space,
            )
        })
        .collect();

    mappings
}
