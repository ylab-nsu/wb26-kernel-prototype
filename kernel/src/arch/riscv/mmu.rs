use crate::arch::riscv::memory::layout::KERNEL_LAYOUT;
use core::ptr::addr_of;
use riscv::register::satp;

#[repr(C, align(4096))]
struct PageTable(pub [usize; 512]);

extern "C" {
    #[link_name = "__s_temp_mmu_table"]
    static mut MMU_TABLE: [PageTable; 16];
}

pub unsafe fn set_satp(number: usize) {
    // todo!();
    let a = KERNEL_LAYOUT.kernel_va_offset;
    let b = addr_of!(MMU_TABLE[number]) as usize;
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
        MMU_TABLE[0].0[510] = kernel_pa;
        MMU_TABLE[1].0[510] = kernel_pa;

        MMU_TABLE[0].0[1] = kernel_pa;
        let user_pa = pte_value(0x80000000, 0b011111);
        MMU_TABLE[1].0[1] = user_pa;
        set_satp(0);
        riscv::asm::sfence_vma_all();
    }
}
