#[repr(C)]
#[derive(Debug)]
pub struct KernelLayout {
    pub kernel_va_offset: usize,
    pub stext: usize,
    pub etext: usize,
    pub srodata: usize,
    pub erodata: usize,
    pub sdata: usize,
    pub edata: usize,
    pub sbss: usize,
    pub ebss: usize,
    pub suninit: usize,
    pub euninit: usize,
    pub sheap: usize,
    pub eheap: usize,
    pub spage_table_pool: usize,
    pub epage_table_pool: usize,
    pub sstack_protector: usize,
    pub estack_protector: usize,
    pub estack: usize,
    pub sstack: usize,
    pub user_va_offset: usize,
}

unsafe extern "C" {
    #[link_name = "__kernel_layout"]
    pub safe static KERNEL_LAYOUT: KernelLayout;
}
