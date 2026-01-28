use crate::println;
use core::mem::MaybeUninit;

static mut PHYSICAL_PAGE_ALLOCATOR: MaybeUninit<PhysicalPageAllocator> = MaybeUninit::uninit();

type PhysicalPageNumber = usize;

#[derive(Debug, Default)]
pub struct PhysicalPage(PhysicalPageNumber);

impl PhysicalPage {
    pub fn new() -> PhysicalPage {
        unsafe {
            PHYSICAL_PAGE_ALLOCATOR
                .assume_init_mut()
                .allocate()
                .unwrap()
        }
    }

    pub fn number(&self) -> PhysicalPageNumber {
        self.0
    }
}

impl Drop for PhysicalPage {
    fn drop(&mut self) {
        unsafe { PHYSICAL_PAGE_ALLOCATOR.assume_init_mut().free(self) }
    }
}

pub fn set_global_physical_page_allocator(ppa: PhysicalPageAllocator) {
    unsafe {
        PHYSICAL_PAGE_ALLOCATOR.write(ppa);
    }
}

trait PageAllocator {
    type Page;

    fn allocate(&mut self) -> Option<Self::Page>;
    fn allocate_at(&mut self, number: PhysicalPageNumber) -> Option<Self::Page>;
    fn free(&mut self, page: &mut Self::Page);
}

#[derive(Debug, Default, Clone)]
pub struct PhysicalPageAllocator {
    base: PhysicalPageNumber,
    alloc: (),
}

impl PhysicalPageAllocator {
    pub fn new(base: PhysicalPageNumber) -> PhysicalPageAllocator {
        PhysicalPageAllocator { base, alloc: () }
    }
}

// impl PageAllocator for PhysicalPageAllocator {
//     type Page = u64;
//
//     fn allocate(&mut self) -> Option<PhysicalPage> {
//         self.base += 1;
//         Some(PhysicalPage(self.base))
//     }
//
//     fn allocate_at(&mut self, number: PhysicalPageNumber) -> Option<PhysicalPage> {
//         Some(PhysicalPage(number))
//     }
//
//     fn free(&mut self, page: &mut PhysicalPage) {
//         println!("Dropping PhysicalPage {:x}", page.number());
//     }
// }

impl PageAllocator for PhysicalPageAllocator {
    type Page = PhysicalPage;

    fn allocate(&mut self) -> Option<Self::Page> {
        self.base += 1;
        Some(PhysicalPage(self.base))
    }

    fn allocate_at(&mut self, number: PhysicalPageNumber) -> Option<Self::Page> {
        Some(PhysicalPage(number))
    }

    fn free(&mut self, page: &mut Self::Page) {
        println!("Dropping PhysicalPage {:x}", page.number());
    }
}
