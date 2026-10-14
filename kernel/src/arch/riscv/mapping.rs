use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::arch::traits::TargetMapping;
use crate::{
    arch::{
        riscv::memory::layout::KERNEL_LAYOUT,
        traits::{TargetAddressSpace, TargetPhysicalAllocator},
        AddressSpace, Mapping, PhysicalAddress, PhysicalAllocator, VirtualAddress,
    },
    sync::Once,
    vm::{MappingFlags, MappingPermissions},
};

/// The kernel's runtime regions, one source of truth for the boot address
/// space and for sharing the kernel half into process address spaces.
pub fn kernel_sections() -> [(&'static str, usize, usize, MappingPermissions); 8] {
    let layout = &KERNEL_LAYOUT;
    [
        ("text", layout.stext, layout.etext, MappingPermissions::rx()),
        (
            "rodata",
            layout.srodata,
            layout.erodata,
            MappingPermissions::ro(),
        ),
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
            "kernel_stacks",
            layout.s_temp_kernel_stacks,
            layout.e_temp_kernel_stacks,
            MappingPermissions::rw(),
        ),
        (
            "stack",
            layout.estack,
            layout.sstack,
            MappingPermissions::rw(),
        ),
    ]
}

/// Canonical mappings of the kernel half, written once at boot. Process
/// address spaces share the same backing allocations by copying these mappings
/// (`map_shared`), so the kernel half is read-only afterwards.
pub static KERNEL_MAPPINGS: Once<Vec<Mapping>> = Once::new();

pub fn map_kernel_sections(address_space: &AddressSpace) {
    match KERNEL_MAPPINGS.get() {
        None => {
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
        Some(mappings) => {
            for mapping in mappings {
                let vaddr: usize = mapping.virt_addr().try_into().unwrap();
                address_space.map_shared(mapping, mapping.virt_addr());
                info!(
                    "exec: shared kernel {vaddr:#018x} (size {:#x})",
                    mapping.size()
                );
            }
        }
    }
}
