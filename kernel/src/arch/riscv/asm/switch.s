.option norvc
.altmacro
.set NUM_GP_REGS, 32  # Number of registers per context
.set REG_SIZE, 8   # Register size (in bytes)


# Thread context
.set CONTEXT_RA, 0*REG_SIZE
.set CONTEXT_SP, 1*REG_SIZE
.set CONTEXT_S0, 2*REG_SIZE
.set CONTEXT_S1, 3*REG_SIZE
.set CONTEXT_S2, 4*REG_SIZE
.set CONTEXT_S3, 5*REG_SIZE
.set CONTEXT_S4, 6*REG_SIZE
.set CONTEXT_S5, 7*REG_SIZE
.set CONTEXT_S6, 8*REG_SIZE
.set CONTEXT_S7, 9*REG_SIZE
.set CONTEXT_S8, 10*REG_SIZE
.set CONTEXT_S9, 11*REG_SIZE
.set CONTEXT_S10, 12*REG_SIZE
.set CONTEXT_S11, 13*REG_SIZE
.set CONTEXT_SIZE, 14*REG_SIZE

.section .text
# switch_thread(old_context, new_context)
.global _switch_thread
.align 4
_switch_thread:
    sd ra, CONTEXT_RA(a0)
    sd sp, CONTEXT_SP(a0)
    sd s0, CONTEXT_S0(a0)
    sd s1, CONTEXT_S1(a0)
    sd s2, CONTEXT_S2(a0)
    sd s3, CONTEXT_S3(a0)
    sd s4, CONTEXT_S4(a0)
    sd s5, CONTEXT_S5(a0)
    sd s6, CONTEXT_S6(a0)
    sd s7, CONTEXT_S7(a0)
    sd s8, CONTEXT_S8(a0)
    sd s9, CONTEXT_S9(a0)
    sd s10, CONTEXT_S10(a0)
    sd s11, CONTEXT_S11(a0)

    ld ra, CONTEXT_RA(a1)
    ld sp, CONTEXT_SP(a1)
    ld s0, CONTEXT_S0(a1)
    ld s1, CONTEXT_S1(a1)
    ld s2, CONTEXT_S2(a1)
    ld s3, CONTEXT_S3(a1)
    ld s4, CONTEXT_S4(a1)
    ld s5, CONTEXT_S5(a1)
    ld s6, CONTEXT_S6(a1)
    ld s7, CONTEXT_S7(a1)
    ld s8, CONTEXT_S8(a1)
    ld s9, CONTEXT_S9(a1)
    ld s10, CONTEXT_S10(a1)
    ld s11, CONTEXT_S11(a1)
    ret
