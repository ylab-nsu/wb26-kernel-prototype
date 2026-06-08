use core::ops::{Index, IndexMut};

use crate::{
    arch::common::page_table::pool::PageTableRef,
    vm::{MappingFlags, MappingPermissions},
};

pub mod alloc;
pub mod pool;

pub enum PageTableEntryStateInner {
    Invalid,
    Node {
        phys_addr: usize,
    },
    Leaf {
        phys_addr: usize,
        permissions: MappingPermissions,
        flags: MappingFlags,
    },
}

pub enum PageTableEntryState<P: TargetPageTable> {
    Invalid,
    Node {
        page_table: PageTableRef<P>,
    },
    Leaf {
        phys_addr: usize,
        permissions: MappingPermissions,
        flags: MappingFlags,
    },
}

pub trait TargetPageTableEntry: Sized {
    fn read_state(&self) -> PageTableEntryStateInner;
    fn write_state(&mut self, value: PageTableEntryStateInner);
}

pub trait TargetPageTable:
    Index<usize, Output: TargetPageTableEntry> + IndexMut<usize, Output: TargetPageTableEntry>
{
    const PAGE_TABLE_ENTRIES: usize;

    const DEFAULT: Self;
}
