use crate::layout::kernel_layout;
use crate::page_pool::PAGE_POOL;
use core::ptr::addr_of;
use riscv::register::satp;

pub unsafe fn set_satp(number: usize) {
    let a = kernel_layout.kernel_va_offset;
    let b = addr_of!(PAGE_POOL.mmu_pages[number]) as usize;
    let c = (b - a) >> 12;
    satp::set(
        satp::Mode::Sv39,
        number,
        c, // (addr_of!(MMU_TABLE[number]) as usize - kernel_layout.kernel_va_offset) >> 12,
    );
}

#[inline]
fn pte_value(addr: usize, flags: usize) -> usize {
    addr >> 12 << 10 | flags
}

pub fn init_mmu() {
    unsafe {
        riscv::register::sstatus::set_sum();
        let kernel_pa = pte_value(0x80000000, 0b101111);
        PAGE_POOL.mmu_pages[0].0[510] = kernel_pa;
        PAGE_POOL.mmu_pages[1].0[510] = kernel_pa;

        PAGE_POOL.mmu_pages[0].0[1] = kernel_pa;
        let user_pa = pte_value(0x80000000, 0b011111);
        PAGE_POOL.mmu_pages[1].0[1] = user_pa;
        set_satp(0);
        riscv::asm::sfence_vma_all();
    }
}
