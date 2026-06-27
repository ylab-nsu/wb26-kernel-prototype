use core::arch::asm;

use crate::{heap, kernel_main, layout, thread};

#[export_name = "__riscv_main"]
fn riscv_main(_hart_id: usize, _dtc: usize) -> ! {
    println!("Initializing heap...");
    heap::init_heap();
    // mmu::init_mmu();
    thread::setup_trap();
    thread::setup_threads();
    // device_tree::handle_device_tree(_dtc);
    
    unsafe { PhysicalAllocator::init(PhysicalAddress::from_bits(0x8000_0000), 4096 * 4096) };
    
    println!("I am virtual {:x?}!", riscv::register::satp::read());

    layout::print_kernel_layout();

    unsafe { asm!("csrw sscratch, sp") };
    // thread::enable_threading();

    kernel_main();

    // loop {
    //     riscv::asm::wfi();
    // }
}
