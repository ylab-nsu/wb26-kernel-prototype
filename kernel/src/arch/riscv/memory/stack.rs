use core::{arch::asm, mem::MaybeUninit, ptr::addr_of};

use crate::arch::riscv::{memory::layout::KERNEL_LAYOUT, vm::PAGE_SIZE};

const STACK_SIZE: usize = 256 * PAGE_SIZE; // 1 MB

#[link_section = ".stack"]
#[used]
static STACK_MEM: [MaybeUninit<u8>; STACK_SIZE] = [MaybeUninit::uninit(); STACK_SIZE];

fn read_sp() -> usize {
    let sp: usize;
    unsafe {
        asm!("mv {}, sp", out(reg) sp);
    }

    sp
}

pub fn init_stack() {
    let stack_addr = addr_of!(STACK_MEM) as usize;

    debug_assert!(stack_addr >= KERNEL_LAYOUT.estack);
    debug_assert!(stack_addr + STACK_SIZE <= KERNEL_LAYOUT.sstack);

    let sp = read_sp();

    debug_assert!(sp >= KERNEL_LAYOUT.estack);
    debug_assert!(sp <= KERNEL_LAYOUT.sstack);
}
