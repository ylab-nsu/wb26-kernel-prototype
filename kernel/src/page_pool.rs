const MMU_PAGES: usize = 16;
const KERNEL_STACK_PAGES: usize = 16;

#[repr(C)]
#[derive(Debug)]
pub struct Page(pub [usize; 512]);

#[repr(C)]
#[derive(Debug)]
pub struct PagePool {
    pub mmu_pages: [Page; MMU_PAGES],
    pub kernel_stack_pages: [Page; KERNEL_STACK_PAGES],
}

#[link_section = ".page_pool"]
pub static mut PAGE_POOL: PagePool = unsafe { core::mem::zeroed() };
