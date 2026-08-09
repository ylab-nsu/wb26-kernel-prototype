use core::{
    fmt::Debug,
    mem::MaybeUninit,
    ops::Index,
    ptr::{self, NonNull},
};

use alloc::vec::Vec;
use sync_unsafe_cell::SyncUnsafeCell;

use crate::{
    arch::{
        PhysicalAddress, common::page_table::{
            PageTableEntryState, PageTableEntryStateInner, TargetPageTable, TargetPageTableEntry, alloc::PAGE_TABLE_ALLOCATOR,
        }, traits::TargetAddress,
    }, sync::{Mutex, Once}, vm::{MappingFlags, MappingPermissions},
};

pub const PAGE_TABLE_POOL_ENTRIES: usize = 4;

#[derive(Debug)]
pub struct PageTableDescriptor {
    pub num_refs: usize,
}

#[repr(C)]
pub struct PageTablePool<P: TargetPageTable> {
    page_tables: [MaybeUninit<SyncUnsafeCell<P>>; PAGE_TABLE_POOL_ENTRIES],
    descriptors: [MaybeUninit<Mutex<PageTableDescriptor>>; PAGE_TABLE_POOL_ENTRIES],
    page_tables_phys_addr: Once<PhysicalAddress>,
}

impl<P: TargetPageTable> Debug for PageTablePool<P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let descriptors: Vec<usize> = self
            .descriptors
            .iter()
            .map(|x| unsafe { x.assume_init_ref() }.lock().num_refs)
            .collect();

        f.debug_struct("PageTablePool")
            .field("descriptors", &descriptors)
            .field("page_tables_phys_addr", &self.page_tables_phys_addr)
            .finish()
    }
}

impl<P: TargetPageTable> PageTablePool<P> {
    pub const unsafe fn new() -> Self {
        PageTablePool {
            page_tables: [const { MaybeUninit::uninit() }; PAGE_TABLE_POOL_ENTRIES],
            descriptors: [const { MaybeUninit::uninit() }; PAGE_TABLE_POOL_ENTRIES],
            page_tables_phys_addr: Once::new(),
        }
    }

    pub unsafe fn set_pool_phys_address(&self, addr: PhysicalAddress) {
        self.page_tables_phys_addr.call_once(|| addr);
    }

    unsafe fn get_page_table_ptr(&self, index: usize) -> *mut P {
        self.page_tables[index].assume_init_ref().get()
    }

    unsafe fn get_page_table(&self, index: usize) -> &P {
        // unsafe { self.page_tables[index].assume_init_ref().into_inner() }
        unsafe { &*self.get_page_table_ptr(index) }
    }

    unsafe fn get_page_table_mut(&self, index: usize) -> &mut P {
        // unsafe { &mut *core::ptr::from_ref(self.get_page_table(index)).cast_mut() }
        unsafe { &mut *self.get_page_table_ptr(index) }
    }

    unsafe fn get_descriptor(&self, index: usize) -> &Mutex<PageTableDescriptor> {
        unsafe { self.descriptors[index].assume_init_ref() }
    }

    unsafe fn get_page_table_addr_from_index(&self, index: usize) -> PhysicalAddress {
        let page_tables_phys_addr = unsafe { self.page_tables_phys_addr.get_unchecked() };
        page_tables_phys_addr.byte_add(index * core::mem::size_of::<P>())
    }

    unsafe fn get_index_from_page_table_addr(&self, phys_addr: PhysicalAddress) -> usize {
        let page_tables_phys_addr = unsafe { self.page_tables_phys_addr.get_unchecked() };
        phys_addr.byte_offset_from_unsigned(*page_tables_phys_addr) / core::mem::size_of::<P>()
    }

    unsafe fn create_ref_from_index(&self, index: usize) -> PageTableRef<P> {
        let new_ref = PageTableRef {
            pool: NonNull::from_ref(self),
            index,
        };

        new_ref.increment_ref_counter();

        new_ref
    }

    unsafe fn create_ref_from_page_table_addr(
        &self,
        phys_addr: PhysicalAddress,
    ) -> PageTableRef<P> {
        let index = unsafe { self.get_index_from_page_table_addr(phys_addr) };

        unsafe { self.create_ref_from_index(index) }
    }
}

impl<P: TargetPageTable> PageTablePool<P> {
    pub fn alloc_page_table(&self) -> PageTableRef<P> {
        let table_idx = PAGE_TABLE_ALLOCATOR.alloc().expect("Can't allocate new page table");

        debug!("Allocated new page table at {table_idx}");

        // self.page_tables[table_idx].write(P::DEFAULT);
        let page_table_ptr = ptr::from_ref(self.page_tables.index(table_idx)).cast_mut();
        unsafe { (*page_table_ptr).write(SyncUnsafeCell::new(P::DEFAULT)) };

        let descriptor = Mutex::new(PageTableDescriptor { num_refs: 1 });

        // self.descriptors[table_idx].write(descriptor);
        let descriptor_ptr = ptr::from_ref(self.descriptors.index(table_idx)).cast_mut();
        unsafe { (*descriptor_ptr).write(descriptor) };

        PageTableRef {
            pool: NonNull::from_ref(self),
            index: table_idx,
        }
    }
}

pub struct PageTableRef<P: TargetPageTable> {
    pool: NonNull<PageTablePool<P>>,
    index: usize,
}

impl<P: TargetPageTable> PageTableRef<P> {
    pub fn get_phys_addr(&self) -> PhysicalAddress {
        let pool_ref = self.get_pool_ref();
        unsafe { pool_ref.get_page_table_addr_from_index(self.index) }
    }

    fn get_pool_ref(&self) -> &PageTablePool<P> {
        unsafe { self.pool.as_ref() }
    }

    unsafe fn get_page_table(&self) -> &P {
        let pool_ref = self.get_pool_ref();
        unsafe { pool_ref.get_page_table(self.index) }
    }

    unsafe fn get_page_table_mut(&self) -> &mut P {
        let pool_ref = self.get_pool_ref();
        unsafe { pool_ref.get_page_table_mut(self.index) }
    }

    fn get_descriptor(&self) -> &Mutex<PageTableDescriptor> {
        let pool_ref = self.get_pool_ref();
        unsafe { pool_ref.get_descriptor(self.index) }
    }

    unsafe fn increment_ref_counter(&self) {
        self.get_descriptor().lock().num_refs += 1;
    }

    unsafe fn decrement_ref_counter(&self) {
        self.get_descriptor().lock().num_refs -= 1;
    }
}

impl<P: TargetPageTable> PageTableRef<P> {
    pub fn read_state(&self, index: usize) -> PageTableEntryState<P> {
        // Maybe needs lock
        let state = unsafe { self.get_page_table() }.index(index).read_state();

        match state {
            PageTableEntryStateInner::Invalid => PageTableEntryState::Invalid,
            PageTableEntryStateInner::Node { phys_addr } => {
                let pool_ref = self.get_pool_ref();

                let page_table = unsafe { pool_ref.create_ref_from_page_table_addr(phys_addr) };

                PageTableEntryState::Node { page_table }
            }
            PageTableEntryStateInner::Leaf {
                phys_addr,
                permissions,
                flags,
            } => PageTableEntryState::Leaf {
                phys_addr,
                permissions,
                flags,
            },
        }
    }

    pub fn write_invalid(&self, index: usize) {
        self.write_state(index, PageTableEntryState::Invalid);
    }

    pub fn write_node(&self, index: usize, other: PageTableRef<P>) {
        self.write_state(index, PageTableEntryState::Node { page_table: other });
    }

    pub fn write_leaf(
        &self,
        index: usize,
        phys_addr: PhysicalAddress,
        permissions: MappingPermissions,
        flags: MappingFlags,
    ) {
        self.write_state(
            index,
            PageTableEntryState::Leaf {
                phys_addr,
                permissions,
                flags,
            },
        );
    }

    pub fn write_state(&self, index: usize, state: PageTableEntryState<P>) {
        let _lock = self.get_descriptor().lock();

        let page_table = unsafe { self.get_page_table_mut() };
        let old_inner_state = page_table.index(index).read_state();

        let new_inner_state = self.get_new_inner_state(old_inner_state, state);

        page_table.index_mut(index).write_state(new_inner_state);
    }

    fn get_new_inner_state(
        &self,
        old_inner_state: PageTableEntryStateInner,
        new_state: PageTableEntryState<P>,
    ) -> PageTableEntryStateInner {
        match old_inner_state {
            PageTableEntryStateInner::Invalid => match new_state {
                PageTableEntryState::Invalid => todo!(),
                PageTableEntryState::Node { page_table } => {
                    unsafe { page_table.increment_ref_counter() };

                    PageTableEntryStateInner::Node {
                        phys_addr: page_table.get_phys_addr(),
                    }
                }
                PageTableEntryState::Leaf {
                    phys_addr,
                    permissions,
                    flags,
                } => PageTableEntryStateInner::Leaf {
                    phys_addr,
                    permissions,
                    flags,
                },
            },
            PageTableEntryStateInner::Node { phys_addr } => match new_state {
                PageTableEntryState::Invalid => {
                    let pool_ref = self.get_pool_ref();

                    let page_table = unsafe { pool_ref.create_ref_from_page_table_addr(phys_addr) };

                    unsafe { page_table.decrement_ref_counter() };

                    PageTableEntryStateInner::Invalid
                }
                _ => todo!(),
            },
            PageTableEntryStateInner::Leaf { .. } => match new_state {
                PageTableEntryState::Invalid => PageTableEntryStateInner::Invalid,
                _ => todo!(),
            },
        }
    }

    pub fn get_next_level_table(&self, index: usize) -> Option<PageTableRef<P>> {
        match self.read_state(index) {
            PageTableEntryState::Invalid => {
                let page_table = self.get_pool_ref().alloc_page_table();
                self.write_node(index, page_table.clone());

                Some(page_table)
            }
            PageTableEntryState::Node { page_table } => Some(page_table),
            PageTableEntryState::Leaf { .. } => None,
        }
    }
}

impl<P: TargetPageTable> Clone for PageTableRef<P> {
    fn clone(&self) -> Self {
        unsafe { self.increment_ref_counter() };

        Self { ..*self }
    }
}

impl<P: TargetPageTable> Drop for PageTableRef<P> {
    // impl Drop for PT DES!!!
    fn drop(&mut self) {
        let do_deallocate = {
            let mut descriptor = self.get_descriptor().lock();
            descriptor.num_refs -= 1;

            let left = descriptor.num_refs;
            // debug!("Drop Ref, left: {left}");

            descriptor.num_refs == 0
        };

        if do_deallocate {
            self.invalidate_table();
            PAGE_TABLE_ALLOCATOR.dealloc(self.index);
            unsafe {
                core::ptr::drop_in_place(core::ptr::from_ref(self.get_descriptor()).cast_mut())
            };
        }
    }
}

// TRAIT BOUND BRUH MOMENT
impl<P: TargetPageTable> PageTableRef<P> {
    fn invalidate_table(&self) {
        for i in 0..P::PAGE_TABLE_ENTRIES {
            match self.read_state(i) {
                PageTableEntryState::Node { .. } => self.write_invalid(i),
                _ => {}
            }
        }
    }
}
