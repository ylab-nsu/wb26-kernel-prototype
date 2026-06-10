.option norvc

.set PTE_SIZE, 8
.set PAGE_TABLE_ENTRIES, 512
.set PAGE_TABLE_SIZE, PAGE_TABLE_ENTRIES * PTE_SIZE

.macro PTE_SET_CENTER n, addr, value
    sd \value, ((\n - (PAGE_TABLE_ENTRIES / 2)) * PTE_SIZE)(\addr)
.endm

.macro PTE_VALUE reg, addr, flags
    li \reg, ((\addr >> 12) << 10) | \flags
.endm

.section .trampoline.text, "ax"
.align 4
.global __riscv_init_mmu
__riscv_init_mmu:
    la t0, __root_mmu_table_center
    PTE_VALUE t1, 0x80000000, 0b1111
    PTE_SET_CENTER 2, t0, t1
    PTE_SET_CENTER 510, t0, t1

    PTE_SET_CENTER 1, t0, t1


    li t0, 2 << 62 
    # li t0, 0x8000000000000000
    la t1, __root_mmu_table
    srli t1, t1, 12
    or t0, t0, t1

    csrw satp, t0
    sfence.vma

    #la t0, __riscv_start
    #jalr x0, 0(t0)

    #li t0, 0xffffffff82002000
    #jalr x0, 0(t0)

    la t0, 1f
    ld t0, 0(t0)
    jalr x0, 0(t0)
1:
    .dword __riscv_start

.section .trampoline.root_table, "a"
__root_mmu_table:
    .space PAGE_TABLE_SIZE / 2
__root_mmu_table_center:
    .space PAGE_TABLE_SIZE / 2
