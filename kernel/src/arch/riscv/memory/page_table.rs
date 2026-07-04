use core::ops::{Index, IndexMut};

use bitfield_struct::bitfield;

use crate::{
    arch::{
        common::page_table::{PageTableEntryStateInner, TargetPageTable, TargetPageTableEntry},
        riscv::memory::Sv39PhysicalAddress,
    },
    vm::{MappingFlags, MappingPermissions},
};

const PAGE_TABLE_ENTRIES: usize = 512;

#[bitfield(u64)]
pub struct Sv39PageTableEntry {
    valid: bool,
    read: bool,
    write: bool,
    execute: bool,
    user: bool,
    global: bool,
    accessed: bool,
    dirty: bool,

    #[bits(2)]
    __: usize,

    #[bits(9)]
    ppn_0: usize,
    #[bits(9)]
    ppn_1: usize,
    #[bits(26)]
    ppn_2: usize,

    #[bits(7)]
    __: usize,

    #[bits(2)]
    pbmt: usize,
    n: bool,
}

impl Sv39PageTableEntry {
    pub const fn with_mapping_flags(self, flags: MappingFlags) -> Self {
        self.with_user(flags.user())
            .with_global(flags.global())
            .with_accessed(flags.accessed())
            .with_dirty(flags.dirty())
    }

    pub const fn with_mapping_permissions(self, permissions: MappingPermissions) -> Self {
        self.with_read(permissions.read())
            .with_write(permissions.write())
            .with_execute(permissions.execute())
    }

    pub const fn with_phys_addr(self, phys_addr: Sv39PhysicalAddress) -> Self {
        self.with_ppn_0(phys_addr.ppn_0())
            .with_ppn_1(phys_addr.ppn_1())
            .with_ppn_2(phys_addr.ppn_2())
    }

    pub const fn mapping_flags(&self) -> MappingFlags {
        MappingFlags::new()
            .with_user(self.user())
            .with_global(self.global())
            .with_accessed(self.accessed())
            .with_dirty(self.dirty())
    }

    pub const fn mapping_permissions(&self) -> MappingPermissions {
        MappingPermissions::new()
            .with_read(self.read())
            .with_write(self.write())
            .with_execute(self.execute())
    }

    pub const fn phys_addr(&self) -> Sv39PhysicalAddress {
        Sv39PhysicalAddress::new()
            .with_ppn_0(self.ppn_0())
            .with_ppn_1(self.ppn_1())
            .with_ppn_2(self.ppn_2())
    }
}

impl TargetPageTableEntry for Sv39PageTableEntry {
    fn read_state(&self) -> PageTableEntryStateInner {
        if !self.valid() {
            return PageTableEntryStateInner::Invalid;
        } else if !(self.read() || self.write() || self.execute()) {
            let phys_addr = Sv39PhysicalAddress::new()
                .with_ppn_0(self.ppn_0())
                .with_ppn_1(self.ppn_1())
                .with_ppn_2(self.ppn_2());
            return PageTableEntryStateInner::Node { phys_addr };
        } else {
            PageTableEntryStateInner::Leaf {
                phys_addr: self.phys_addr(),
                permissions: self.mapping_permissions(),
                flags: self.mapping_flags(),
            }
        }
    }

    fn write_state(&mut self, value: PageTableEntryStateInner) {
        match value {
            PageTableEntryStateInner::Invalid => {
                self.0 = Sv39PageTableEntry::new().into_bits();
            }
            PageTableEntryStateInner::Node { phys_addr } => {
                let new_pte = Sv39PageTableEntry::new()
                    .with_valid(true)
                    .with_ppn_0(phys_addr.ppn_0())
                    .with_ppn_1(phys_addr.ppn_1())
                    .with_ppn_2(phys_addr.ppn_2());

                self.0 = new_pte.into_bits();
            }
            PageTableEntryStateInner::Leaf {
                phys_addr,
                flags,
                permissions,
            } => {
                let new_pte = Sv39PageTableEntry::new()
                    .with_valid(true)
                    .with_mapping_permissions(permissions)
                    .with_mapping_flags(flags)
                    .with_phys_addr(phys_addr);

                self.0 = new_pte.into_bits();
            }
        }
    }
}

#[repr(C)]
#[repr(align(4096))]
#[derive(Debug)]
pub struct Sv39PageTable(pub [Sv39PageTableEntry; PAGE_TABLE_ENTRIES]);

impl TargetPageTable for Sv39PageTable {
    const DEFAULT: Self = Sv39PageTable([const { Sv39PageTableEntry(0) }; PAGE_TABLE_ENTRIES]);

    const PAGE_TABLE_ENTRIES: usize = PAGE_TABLE_ENTRIES;
}

impl Index<usize> for Sv39PageTable {
    type Output = Sv39PageTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl IndexMut<usize> for Sv39PageTable {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}
