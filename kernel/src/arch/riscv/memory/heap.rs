use core::{mem::{MaybeUninit, zeroed}, ptr::addr_of};
use embedded_alloc::LlffHeap as Heap;

use crate::arch::riscv::{memory::layout::KERNEL_LAYOUT, vm::PAGE_SIZE};

#[global_allocator]
static HEAP: Heap = Heap::empty();

const HEAP_SIZE: usize = 256 * PAGE_SIZE; // 1 MB

#[link_section = ".heap"]
#[used]
static HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [unsafe { zeroed() }; HEAP_SIZE];

pub fn init_heap() {
    let heap_addr = addr_of!(HEAP_MEM) as usize;

    debug_assert!(heap_addr >= KERNEL_LAYOUT.sheap);
    debug_assert!(heap_addr + HEAP_SIZE <= KERNEL_LAYOUT.eheap);

    unsafe { HEAP.init(heap_addr, HEAP_SIZE) };

    debug!("Initialized heap at address 0x{heap_addr:x} with size 0x{HEAP_SIZE:x}");
}
