#[repr(C)]
#[derive(Debug)]
pub(crate) struct KernelLayout {
    pub(crate) kernel_va_offset: usize,
    pub(crate) stext: usize,
    pub(crate) etext: usize,
    pub(crate) srodata: usize,
    pub(crate) erodata: usize,
    pub(crate) sdata: usize,
    pub(crate) edata: usize,
    pub(crate) sbss: usize,
    pub(crate) ebss: usize,
    pub(crate) spage_pool: usize,
    pub(crate) epage_pool: usize,
    pub(crate) user_va_offset: usize,
}

extern "C" {
    #[link_name = "__kernel_layout"]
    pub(crate) static kernel_layout: KernelLayout;
}

unsafe fn print(name: &str, start: usize, end: usize) {
    let pages = ((end - start) / 4096) + 1;
    let start_page = start >> 12;

    println!("Size of {} is {} pages", name, pages);
    for i in 0..pages {
        println!(
            "Page {:x}",
            (start_page + i) - (unsafe { kernel_layout.kernel_va_offset } >> 12)
        );
    }
}

pub(crate) fn print_kernel_layout() {
    unsafe {
        println!("{:x?}", kernel_layout);
        print("rodata", kernel_layout.srodata, kernel_layout.erodata);
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
