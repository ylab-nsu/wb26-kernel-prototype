.section .init, "ax"
.global _start

_start:
    .option push
    .option norelax # to prevent an unsupported R_RISCV_ALIGN relocation from being generated
1:
    auipc ra, %pcrel_hi(1f)
    ld ra, %pcrel_lo(1b)(ra)
    jr ra
    .align  3
1:
    .dword _abs_start
    .option pop
_abs_start:
    .option norelax
    .cfi_startproc
    .cfi_undefined ra

    # Disable interrupts
    csrw sie, 0
    csrw sip, 0

    # // Set pre-init trap vector
    # "la t0, _pre_init_trap",
    # #[cfg(feature = "s-mode")]
    # "csrw stvec, t0",
    # #[cfg(not(feature = "s-mode"))]
    # "csrw mtvec, t0",
#     // If multi-hart, assert that hart ID is valid
#     #[cfg(not(feature = "single-hart"))]
#     "lui t0, %hi(_max_hart_id)
#     add t0, t0, %lo(_max_hart_id)
#     bgeu t0, a0, 1f
#     la t0, abort // If hart_id > _max_hart_id, jump to abort
#     jr t0
# 1:", // Only valid hart IDs reach this point

    # Initialize GP and SP
    .option push
    .option norelax
    la gp, __global_pointer$
    .option pop

    la t1, __sstack
    andi sp, t1, -16 # align stack to 16-bytes

    # Copy .data from flash to RAM
#     la t0, __sdata
#     la a3, __edata
#     la t1, __sidata
#     bgeu t0, a3, 2f
# 1:
#     ld t2, 0(t1)
#     addi t1, t1, 8
#     sd t2, 0(t0)
#     addi t0, t0, 8
#     bltu t0, a3, 1b

2:  # Zero out .bss
    la t0, __sbss
    la t2, __ebss
    bgeu  t0, t2, 4f
3:
    sd zero, 0(t0)
    addi t0, t0, 8
    bltu t0, t2, 3b

4:  # RAM initialized

    # Initialize FPU
    li t0, 0x4000 # Bit 14 is FS most significant bit
    li t2, 0x2000 # Bit 13 is FS least significant bit
    csrrc x0, sstatus, t0
    csrrs x0, sstatus, t2
    fscsr x0

    # Initialize FP and jump to _main
    mv fp, sp
    la t0, _main
    jr t0
    .cfi_endproc
