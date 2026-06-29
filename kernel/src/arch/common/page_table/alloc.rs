use bitmap_allocator::{BitAlloc, BitAlloc4K};

use crate::{
    arch::common::page_table::pool::PAGE_TABLE_POOL_ENTRIES,
    sync::{LazyLock, Mutex},
};

pub static PAGE_TABLE_ALLOCATOR: PageTableAllocator = PageTableAllocator::new();

pub struct PageTableAllocator(LazyLock<Mutex<BitAlloc4K>>);

impl PageTableAllocator {
    const fn new() -> PageTableAllocator {
        PageTableAllocator(LazyLock::new(|| {
            let mut alloc = BitAlloc4K::DEFAULT;
            alloc.insert(0..PAGE_TABLE_POOL_ENTRIES);

            Mutex::new(alloc)
        }))
    }

    pub fn alloc(&self) -> Option<usize> {
        self.0.lock().alloc()
    }

    pub fn dealloc(&self, idx: usize) -> Option<()> {
        if self.0.lock().dealloc(idx) {
            Some(())
        } else {
            None
        }
    }
}
