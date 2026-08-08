use bitmap_allocator::BitAlloc64K;
use fdt::Fdt;

use crate::{
    allocator::{bitmap::BitmapMemoryAllocator, AllocatorError},
    arch::{
        riscv::alloc::RiscvPhysicalAllocation, riscv::vm::PAGE_SIZE,
        traits::TargetPhysicalAllocator, PhysicalAddress, PhysicalAllocation,
    },
    sync::{Mutex, Once},
};

type RiscvPhysicalAllocatorInner = BitmapMemoryAllocator<PhysicalAddress, BitAlloc64K, PAGE_SIZE>;

static PHYSICAL_MEMORY_ALLOCATOR: Once<Mutex<RiscvPhysicalAllocatorInner>> = Once::new();

pub struct RiscvPhysicalAllocator;

impl RiscvPhysicalAllocator {
    pub(in crate::arch::riscv) unsafe fn init(base: PhysicalAddress, size: usize) {
        PHYSICAL_MEMORY_ALLOCATOR
            .call_once(|| unsafe { Mutex::new(BitmapMemoryAllocator::new(base, size)) });
    }

    pub(in crate::arch::riscv) unsafe fn dealloc_contiguous(addr: PhysicalAddress, size: usize) {
        Self::get_global_allocator()
            .lock()
            .dealloc_contiguous(addr, size);
    }

    fn get_global_allocator() -> &'static Mutex<RiscvPhysicalAllocatorInner> {
        unsafe { PHYSICAL_MEMORY_ALLOCATOR.get_unchecked() }
    }

    fn mark_invalid(addr: PhysicalAddress, size: usize) {
        Self::get_global_allocator()
            .lock()
            .alloc_contiguous_at(addr, size)
            .unwrap();
    }
}

impl TargetPhysicalAllocator for RiscvPhysicalAllocator {
    fn alloc_contiguous(size: usize) -> Result<PhysicalAllocation, AllocatorError> {
        let (addr, size) = Self::get_global_allocator().lock().alloc_contiguous(size)?;

        Ok(RiscvPhysicalAllocation { addr, size })
    }

    fn alloc_contiguous_aligned(
        size: usize,
        alignment: usize,
    ) -> Result<PhysicalAllocation, AllocatorError> {
        let (addr, size) = Self::get_global_allocator()
            .lock()
            .alloc_contiguous_aligned(size, alignment)?;

        Ok(RiscvPhysicalAllocation { addr, size })
    }

    fn alloc_contiguous_at(
        addr: PhysicalAddress,
        size: usize,
    ) -> Result<PhysicalAllocation, AllocatorError> {
        let (addr, size) = Self::get_global_allocator()
            .lock()
            .alloc_contiguous_at(addr, size)?;

        Ok(RiscvPhysicalAllocation { addr, size })
    }
}

pub fn init_physical_allocator(fdt: &Fdt) {
    let memory = fdt
        .memory()
        .regions()
        .next()
        .expect("Memory regions are not defined");

    let base = PhysicalAddress::try_from(memory.starting_address as usize).unwrap();
    let size = memory.size.unwrap();

    unsafe {
        RiscvPhysicalAllocator::init(base, size);
    };
    info!("Initialized physical memory allocator at address {base:p} with size 0x{size:x}");

    if fdt.memory_reservations().count() != 0 {
        panic!("Memory reservasion blocks in DTB are not supported!");
    }

    if let Some(reserved_regions) = fdt.find_node("/reserved-memory") {
        for child in reserved_regions.children() {
            let region = child.reg().unwrap().next().unwrap();

            let addr = PhysicalAddress::try_from(region.starting_address as usize).unwrap();
            let size = region.size.unwrap();

            RiscvPhysicalAllocator::mark_invalid(addr, size);
            info!("Reserved memory at {addr:p} with size 0x{size:x}");
        }
    } else {
        info!("No reserved memory regions found");
    }
}
