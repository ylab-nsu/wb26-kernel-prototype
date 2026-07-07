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
    pub estack: usize,
    pub sstack: usize,
}

extern "C" {
    #[link_name = "__kernel_layout"]
    pub static kernel_layout: KernelLayout;
}

unsafe fn print(name: &str, start: usize, end: usize) {
    let pages = ((end - start) / 4096) + 1;
    let start_page = start >> 12;

    println!("Size of {} is {} pages", name, pages);
    for i in 0..pages {
        println!(
            "Page {:x}",
            (start_page + i) - (kernel_layout.kernel_va_offset >> 12)
        );
    }
}

pub fn print_kernel_layout() {
    unsafe {
        println!("{:x?}", kernel_layout);
        print("rodata", kernel_layout.srodata, kernel_layout.erodata);
        debug!("Page table pool {:x} {:x}", kernel_layout.spage_table_pool, kernel_layout.epage_table_pool);
        print("page_table_pool", kernel_layout.spage_table_pool, kernel_layout.epage_table_pool);
    }
}

// unsafe {
//     println!("{:x?}", kernel_layout);
//     let start_page = kernel_layout.srodata >> 12;
//
//
//     for i in 0..=5 {
//         println!("Page {:x}", kernel_layout.srodata + i);
//     }
//     println!("Size of rodata is {:} pages", (kernel_layout.erodata - kernel_layout.srodata) / 4096);
//     println!("Size of text is {:} pages", (kernel_layout.etext - kernel_layout.stext) / 4096);
//     println!("Size of bss is {:} pages", (kernel_layout.ebss - kernel_layout.sbss) / 4096);
// }