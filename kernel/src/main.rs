#![no_std]
#![no_main]
#![warn(unsafe_op_in_unsafe_fn)]

// #[macro_use]
extern crate alloc;
mod asm;
mod heap;
mod paging;
#[macro_use]
mod print;
mod device_tree;
mod layout;
mod mmu;
mod page_pool;
mod threading;

use core::arch::asm;
use core::panic::PanicInfo;

// use crate::paging::alloc::{set_global_physical_page_allocator, PhysicalPage, PhysicalPageAllocator};
// use fdt_rs::prelude::FallibleIterator;

#[export_name = "_main"]
fn main(_hart_id: usize, _dtc: usize) -> ! {
    println!("Initializing heap...");
    heap::init_heap();
    mmu::init_mmu();
    threading::init::setup_trap();
    threading::init::setup_threads();
    // device_tree::handle_device_tree(_dtc);

    println!("I am virtual {:x?}!", riscv::register::satp::read());

    layout::print_kernel_layout();

    unsafe { asm!("csrw sscratch, sp") };
    threading::init::enable_threading();

    // let ppa = PhysicalPageAllocator::new(0x1000);
    //
    // set_global_physical_page_allocator(ppa);
    //
    // {
    //     let p = PhysicalPage::new();
    //     println!("{:x?}", p);
    // }

    loop {
        riscv::asm::wfi();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Something went wrong.");
    println!("{}", info);
    println!("Shutting down...");
    riscv::asm::wfi();
    println!("After shutting down...");
    // sbi::system_reset::system_reset(ResetType::, ResetReason::SystemFailure).unwrap();

    loop {
        riscv::asm::wfi();
    }
}
