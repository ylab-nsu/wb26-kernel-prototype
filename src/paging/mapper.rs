// #[repr(C)]
// struct PageTable([u64; 512]);
//
// impl PageTable {
//     unsafe fn from_addr(addr: usize) -> &'static mut PageTable {
//         if addr % 0xFFF != 0 {
//             panic!("Page table addr must be aligned by page size");
//         }
//
//         &mut *(addr as *mut PageTable)
//     }
//
//     unsafe fn from_addr_zeroed(addr: usize) -> &'static mut PageTable {
//         let table = PageTable::from_addr(addr);
//         table.0.fill(0);
//         table
//     }
// }

// use alloc::boxed::Box;
//
// type VirtualAddress = usize;
// type PhysicalAddress = usize;
// type SizeQuantum = usize;
//
// enum MappingTableEntry {
//     None,
//     Subtable(Box<MappingTable>),
//     Mapping(PhysicalAddress, SizeQuantum),
// }

// Size quantum
// 1 4Kib page for riscv
// 16 bytes pagraph for cdm
// Length is # of paragraphs for cdm
// Length is # of pages for risccv

// struct MappingTable {
//     entries: [MappingTableEntry; 512],
// }
//
// impl MappingTable {
//     fn sync(&mut self) {
//         for (idx, entry) in self.entries.iter_mut().enumerate() {
//             match entry {
//                 MappingTableEntry::None => {}
//                 MappingTableEntry::Subtable(s) => {
//                     let ppn = s.page.ppage;
//                     s.sync();
//                 }
//                 MappingTableEntry::Page => {}
//             }
//         }
//     }
// }

// trait Mapper2 {
//     fn sync(table: &mut MappingTable);
// }
//
// struct Mapping<M: Mapper2> {
//     mapper: M,
//     root_table: MappingTable,
// }
//
// impl Mapping {
//     fn map(&mut self, vaddr: usize, paddr: usize) {
//         let page_table_index = (vaddr >> 12) & 512;
//         match self.root_table.entries[page_table_index] {
//             MappingTableEntry::None => {}
//             MappingTableEntry::Subtable(_) => {}
//             MappingTableEntry::Page => {}
//         }
//     }
// }

trait Mapper {
    fn map(&mut self, vaddr: usize, paddr: usize, size: usize);
    fn unmap(&mut self, vaddr: usize);
    fn get_mapping(&self, vaddr: usize) -> Option<()>;
}
