use crate::arch::riscv::memory::layout::KERNEL_LAYOUT;
use crate::arch::riscv::vm::PAGE_SIZE;
use core::mem::MaybeUninit;
use embedded_alloc::LlffHeap as Heap;
use sync_unsafe_cell::SyncUnsafeCell;

#[global_allocator]
static HEAP: Heap = Heap::empty();

const HEAP_SIZE: usize = 256 * PAGE_SIZE; // 1 MB

#[link_section = ".heap"]
static HEAP_MEM: SyncUnsafeCell<[MaybeUninit<u8>; HEAP_SIZE]> =
    SyncUnsafeCell::new([MaybeUninit::uninit(); HEAP_SIZE]);

pub fn init_heap() {
    let heap_addr = HEAP_MEM.get() as usize;

    debug_assert!(heap_addr >= KERNEL_LAYOUT.sheap);
    debug_assert!(heap_addr + HEAP_SIZE <= KERNEL_LAYOUT.eheap);

    unsafe { HEAP.init(heap_addr, HEAP_SIZE) };

    debug!("Initialized heap at address 0x{heap_addr:x} with size 0x{HEAP_SIZE:x}");
}
