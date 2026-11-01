.option norvc
.altmacro
.set REG_SIZE, 8   # Register size (in bytes)

.set PC, 0
.set RA, 1
.set SP, 2
.set GP, 3
.set TP, 4
.set T0, 5
.set T1, 6
.set T2, 7
.set S0, 8
.set S1, 9
.set A0, 10
.set A1, 11
.set A2, 12
.set A3, 13
.set A4, 14
.set A5, 15
.set A6, 16
.set A7, 17
.set S2, 18
.set S3, 19
.set S4, 20
.set S5, 21
.set S6, 22
.set S7, 23
.set S8, 24
.set S9, 25
.set S10, 26
.set S11, 27
.set T3, 28
.set T4, 29
.set T5, 30
.set T6, 31
.set SSTATUS, 32
.set FRAME_SIZE, 34 # Stack pointer must be aligned by 16 bytes 

.set SSTATUS_SPIE, 0x20
.set SSTATUS_SPP, 0x100
.set SSTATUS_MASK, SSTATUS_SPIE | SSTATUS_SPP

# Use macros for saving and restoring multiple registers
.macro save_gp i, basereg=sp
	sd	x\i, ((\i)*REG_SIZE)(\basereg)
.endm
.macro load_gp i, basereg=sp
	ld	x\i, ((\i)*REG_SIZE)(\basereg)
.endm

.section .text
.global _start_trap
.align 4

# kernel_sp is the pointer to the stack used by kernel threads and interrupts handlers
# TrapFrame is always saved on kernel stack

# kernel_tp is the value always stored in tp register while working in kernel 
# kernel_tp contains the address of the current thread
# When switching threads, scheduler among other things disables interrupts and changes tp

# Userspace programs need their own sp and tp. During that kernel_sp is stored in sscratch register
# kernel_tp is stored in TrapFrame.tp field on kernel stack. Since the kernel thread can change its stack,
# and the only way to enter it is through this trap handler, it's save to store something on this stack.

_start_trap:

# If we came from kernel, SPP == 1, sscratch == 0 and sp == kernel_sp, tp == current_thread_ptr
# If we came from userspace, SPP == 0, sscratch == kernel_sp, sp and tp need to be saved, kernel_tp is in TrapFrame.tp
    csrrw	sp, sscratch, sp
    bnez sp, 1f # If we came from userspace, now sscratch == user_sp, sp == kernel_sp. If from kernel, sp == 0
    csrrw	sp, sscratch, sp # Only if came from kernel. Again, sscratch == 0 and sp == kernel_sp

1:	
    addi sp, sp, - FRAME_SIZE * REG_SIZE

    save_gp %RA
    save_gp %GP
    save_gp %T0
    save_gp %T1
    save_gp %T2
    save_gp %S0
    save_gp %S1
    save_gp %A0
    save_gp %A1
    save_gp %A2
    save_gp %A3
    save_gp %A4
    save_gp %A5
    save_gp %A6
    save_gp %A7
    save_gp %S2
    save_gp %S3
    save_gp %S4
    save_gp %S5
    save_gp %S6
    save_gp %S7
    save_gp %S8
    save_gp %S9
    save_gp %S10
    save_gp %S11
    save_gp %T3
    save_gp %T4
    save_gp %T5
    save_gp %T6

    csrr t0, sepc
    sd t0, PC*REG_SIZE(sp)

    csrr t0, sscratch
    sd t0, SP*REG_SIZE(sp)
    csrw sscratch, zero
    
    csrr t0, sstatus
    sd t0, SSTATUS*REG_SIZE(sp) # save sstatus and check SPP
    andi t0, t0, SSTATUS_SPP
    bnez t0, 1f
    
# If came from userspace, need to swap user tp (tp register) and current thread pointer (TrapFrame.tp)
    ld t1, TP*REG_SIZE(sp)
    save_gp %TP
    mv tp, t1

1:
    mv   a0, sp
	jal  ra, _handle_trap_rust # returns need_reschedule

    beqz a0, 1f
    jal  ra, _reschedule_rust
1:


# Initialize context.ra for a new thread to this
.global _initial_return_trap
_initial_return_trap:

    ld t0, PC*REG_SIZE(sp)
    csrw sepc, t0
    
    csrr t0, sstatus # restore sstatus and check SPP
    ld t1, SSTATUS*REG_SIZE(sp)
    # li t2, ~SSTATUS_MASK
    andi t0, t0, ~SSTATUS_MASK
    andi t1, t1, SSTATUS_MASK
    or t0, t0, t1
    csrw sstatus, t0
    andi t0, t0, SSTATUS_SPP
    bnez t0, 1f

# If returning to userspace, need to swap user tp (TrapFrame.tp) and current thread pointer (tp register)
    mv t1, tp
    load_gp %TP
    sd t1, TP*REG_SIZE(sp)

1:
    load_gp %RA
    load_gp %GP
    load_gp %T1
    load_gp %T2
    load_gp %S0
    load_gp %S1
    load_gp %A0
    load_gp %A1
    load_gp %A2
    load_gp %A3
    load_gp %A4
    load_gp %A5
    load_gp %A6
    load_gp %A7
    load_gp %S2
    load_gp %S3
    load_gp %S4
    load_gp %S5
    load_gp %S6
    load_gp %S7
    load_gp %S8
    load_gp %S9
    load_gp %S10
    load_gp %S11
    load_gp %T3
    load_gp %T4
    load_gp %T5
    load_gp %T6

	addi sp, sp, FRAME_SIZE * REG_SIZE

    bnez t0, 1f // t0 still holds SPP after previous check

# Returning to user. Restoring user_sp
    ld t0, (-FRAME_SIZE + T0)*REG_SIZE(sp)
    csrw sscratch, sp
    ld sp, (-FRAME_SIZE + SP)*REG_SIZE(sp)
    sret

# Returning to kernel
1:
    ld t0, (-FRAME_SIZE + T0)*REG_SIZE(sp)
    sret
