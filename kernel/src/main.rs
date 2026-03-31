#![no_std]
#![no_main]

#[macro_use]
extern crate alloc;
mod asm;
mod heap;
mod paging;
#[macro_use]
mod print;
mod thread;
mod layout;
mod device_tree;

use core::any::Any;
use core::arch::asm;
use core::fmt::Write;
use core::ops::{Deref, DerefMut, Index};
use core::panic::PanicInfo;

// use crate::paging::alloc::{set_global_physical_page_allocator, PhysicalPage, PhysicalPageAllocator};
use fdt_rs::prelude::FallibleIterator;

#[export_name = "_main"]
fn main(_hart_id: usize, _dtc: usize) -> ! {
    println!("Initializing heap...");
    heap::init_heap();
    thread::setup_trap();
    thread::setup_threads();
    // device_tree::handle_device_tree(_dtc);

    println!("I am virtual {:x?}!", riscv::register::satp::read());

    layout::print_kernel_layout();

    unsafe { asm!("csrw sscratch, sp") };
    thread::enable_threading();

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
