use core::arch::asm;

use fdt::Fdt;

use crate::{
    arch::{AddressSpace, riscv::{
        alloc, mapping::map_kernel_sections, memory::{self, layout::KERNEL_LAYOUT},
    }}, kernel_main, thread,
};

#[export_name = "__riscv_main"]
fn riscv_main(_hart_id: usize, dtc: usize) -> ! {
    debug!("Initializing heap...");
    memory::heap::init_heap();
    memory::stack::init_stack();
    // mmu::init_mmu();
    thread::setup_trap();
    thread::setup_threads();
    // device_tree::handle_device_tree(_dtc);

    let device_tree = unsafe { Fdt::from_ptr(dtc as *const u8).expect("Can't parse device tree") };

    alloc::phys::init_physical_allocator(&device_tree);

    println!("I am virtual {:x?}!", riscv::register::satp::read());

    debug!("{KERNEL_LAYOUT:#x?}");

    unsafe { asm!("csrw sscratch, sp") };
    // thread::enable_threading();

    let mut address_space = AddressSpace::new();
    
    let _mappings = map_kernel_sections(&mut address_space);

    kernel_main();

    // loop {
    //     riscv::asm::wfi();
    // }
}
