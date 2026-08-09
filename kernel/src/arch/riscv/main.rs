use core::arch::asm;

use fdt::Fdt;

use crate::{
    arch::{
        AddressSpace, riscv::{
            alloc,
            mapping::map_kernel_sections,
            memory::{self, layout::KERNEL_LAYOUT},
            vm,
        }, traits::TargetAddressSpace,
    }, boot::BootContext, kernel_main, thread,
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

    vm::address_space::init_page_table_pool();

    println!("I am virtual {:x?}!", riscv::register::satp::read());

    debug!("{KERNEL_LAYOUT:#x?}");

    unsafe { asm!("csrw sscratch, sp") };
    // thread::enable_threading();

    let mut address_space = AddressSpace::new();

    let mappings = map_kernel_sections(&mut address_space);

    unsafe {
        address_space.switch();
    }

    let boot_context = BootContext {
        address_space,
        mappings,
    };

    kernel_main(boot_context);

    // loop {
    //     riscv::asm::wfi();
    // }
}
