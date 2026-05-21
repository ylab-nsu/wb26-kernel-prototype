.option norvc
.altmacro
.set NUM_GP_REGS, 32  # Number of registers per context
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
_start_trap:
    # addi sp, sp, - NUM_GP_REGS * REG_SIZE
    csrrw	sp, sscratch, sp

    bnez sp, 1f
    csrrw	sp, sscratch, sp
1:	

    save_gp %RA
    save_gp %GP
    save_gp %TP
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

	csrr t0, sscratch
    sd t0, SP*REG_SIZE(sp)
    csrr t0, sepc
    sd t0, PC*REG_SIZE(sp)

    mv   a0, sp
	jal  ra, _handle_trap_rust # returns need_reschedule

    beqz a0, 1f
    # mv   a0, sp
    jal  ra, _reschedule_rust # returns new sp
    mv sp, a0
1:


    ld t0, PC*REG_SIZE(sp)
    csrw sepc, t0
    ld t0, SP*REG_SIZE(sp)
    csrw sscratch, t0

    load_gp %RA
    load_gp %GP
    load_gp %TP
    load_gp %T0
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


    csrrw	sp, sscratch, sp

    bnez sp, 1f
    csrrw	sp, sscratch, sp
1:	

    sret

	# .set	i, 1
	# .rept	31
	# 	load_gp %i
	# 	.set	i, i+1
	# .endr

	# addi sp, sp, NUM_GP_REGS * REG_SIZE

	# sret
