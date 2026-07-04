use bitmap_allocator::BitAlloc64K;

use crate::{
    allocator::{bitmap::BitmapMemoryAllocator, AllocatorError},
    arch::{
        riscv::alloc::RiscvPhysicalAllocation, riscv::memory::PAGE_SIZE,
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
