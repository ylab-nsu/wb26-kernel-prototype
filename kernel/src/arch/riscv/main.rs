use core::arch::asm;

use crate::{
    arch::{PhysicalAddress, PhysicalAllocator, riscv::memory::{self, layout::KERNEL_LAYOUT}}, kernel_main, thread,
};

#[export_name = "__riscv_main"]
fn riscv_main(_hart_id: usize, _dtc: usize) -> ! {
    println!("Initializing heap...");
    memory::heap::init_heap();
    // mmu::init_mmu();
    thread::setup_trap();
    thread::setup_threads();
    // device_tree::handle_device_tree(_dtc);

    unsafe { PhysicalAllocator::init(PhysicalAddress::from_bits(0x8000_0000), 4096 * 4096) };

    println!("I am virtual {:x?}!", riscv::register::satp::read());

    debug!("{KERNEL_LAYOUT:#x?}");

    unsafe { asm!("csrw sscratch, sp") };
    // thread::enable_threading();

    kernel_main();

    // loop {
    //     riscv::asm::wfi();
    // }
}
