use core::arch::global_asm;

global_asm!(include_str!("trap.s"));
global_asm!(include_str!("init_mmu.s"));
global_asm!(include_str!("entry.s"));
global_asm!(include_str!("kernel_layout.s"));
