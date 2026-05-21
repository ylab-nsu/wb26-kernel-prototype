const MMU_PAGES: usize = 16;
const KERNEL_STACK_PAGES: usize = 512;

#[repr(C)]
#[derive(Debug)]
pub(crate) struct Page(pub(crate) [usize; 512]);

#[repr(C)]
#[derive(Debug)]
pub(crate) struct PagePool {
    pub(crate) mmu_pages: [Page; MMU_PAGES],
    pub(crate) kernel_stack_pages: [Page; KERNEL_STACK_PAGES],
}

#[link_section = ".page_pool"]
pub(crate) static mut PAGE_POOL: PagePool = unsafe { core::mem::zeroed() };
