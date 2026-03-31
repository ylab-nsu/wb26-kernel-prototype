#[repr(C)]
#[derive(Debug)]
struct KernelLayout {
    kernel_va_offset: usize,
    stext: usize,
    etext: usize,
    srodata: usize,
    erodata: usize,
    sdata: usize,
    edata: usize,
    sbss: usize,
    ebss: usize,
}

extern "C" {
    #[link_name = "__kernel_layout"]
    static kernel_layout: KernelLayout;
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