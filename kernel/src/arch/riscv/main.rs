use crate::arch::riscv::mapping::map_kernel_sections;
use crate::arch::riscv::memory::heap::init_heap;
use crate::arch::riscv::memory::stack::init_stack;
use crate::arch::riscv::mmu::init_mmu;
use crate::arch::riscv::threading::trap::setup_trap;
use crate::arch::riscv::plic::{init_plic};
use crate::arch::riscv::{alloc, vm};
use crate::arch::AddressSpace;
use crate::boot::BootContext;
use crate::{arch::riscv::memory::layout::KERNEL_LAYOUT, kernel_main};
use core::arch::asm;
use fdt::Fdt;
use riscv::interrupt::Interrupt::{SupervisorExternal, SupervisorTimer};

#[export_name = "__riscv_main"]
fn riscv_main(_hart_id: usize, dtc: usize) -> ! {
    debug!("Initializing heap...");
    init_heap();
    init_stack();
    // mmu::init_mmu();
    setup_trap();
    // device_tree::handle_device_tree(_dtc);


    let device_tree = unsafe { Fdt::from_ptr(dtc as *const u8).expect("Can't parse device tree") };

    alloc::phys::init_physical_allocator(&device_tree);

    vm::address_space::init_page_table_pool();

    println!("I am virtual {:x?}!", riscv::register::satp::read());

    debug!("{KERNEL_LAYOUT:#x?}");

    unsafe { asm!("csrw sscratch, sp", options(nostack)) };
    init_mmu();
	init_plic(); // Возможно стоит перенести в другое место 
    unsafe {
        riscv::interrupt::enable_interrupt(SupervisorTimer);
		riscv::interrupt::enable_interrupt(SupervisorExternal);
    }

    let mut address_space = AddressSpace::new();

    let mappings = map_kernel_sections(&mut address_space);

    // unsafe {
    //     address_space.switch();
    // }

    let boot_context = BootContext {
        address_space,
        mappings,
    };

    let time = riscv::register::time::read64();
    sbi::timer::set_timer(time + 1_000_000).expect("Can't set timer");

    kernel_main(boot_context);

    // loop {
    //     riscv::asm::wfi();
    // }
}
