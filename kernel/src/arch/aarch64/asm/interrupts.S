.section .interrupt_vectors, "x"

.global user_return
.global current_el_sp_el0_sync

.macro vector name, source, type
.balign 0x80
\name:
    sub sp, sp, #8
    str x30, [sp, -8]!
    stp x28, x29, [sp, -16]!
    stp x26, x27, [sp, -16]!
    stp x24, x25, [sp, -16]!
    stp x22, x23, [sp, -16]!
    stp x20, x21, [sp, -16]!
    stp x18, x19, [sp, -16]!
    stp x16, x17, [sp, -16]!
    stp x14, x15, [sp, -16]!
    stp x12, x13, [sp, -16]!
    stp x10, x11, [sp, -16]!
    stp x8, x9, [sp, -16]!
    stp x6, x7, [sp, -16]!
    stp x4, x5, [sp, -16]!
    stp x2, x3, [sp, -16]!
    stp x0, x1, [sp, -16]!

    mov x0, sp
    mov x1, #\source
    mov x2, #\type
    adr x30, user_return
    b demux_interrupt
.endm

vector current_el_sp_el0_sync, 0, 0
vector current_el_sp_el0_irq, 0, 1
vector current_el_sp_el0_fiq, 0, 2
vector current_el_sp_el0_serror, 0, 3
vector current_el_sp_elx_sync, 1, 0
vector current_el_sp_elx_irq, 1, 1
vector current_el_sp_elx_fiq, 1, 2
vector current_el_sp_elx_serror, 1, 3
vector lower_el_aa64_sync, 2, 0
vector lower_el_aa64_irq, 2, 1
vector lower_el_aa64_fiq, 2, 2
vector lower_el_aa64_serror, 2, 3
vector lower_el_aa32_sync, 3, 0
vector lower_el_aa32_irq, 3, 1
vector lower_el_aa32_fiq, 3, 2
vector lower_el_aa32_serror, 3, 3

.balign 0x80

user_return:
    ldp x0, x1, [sp], #16
    ldp x2, x3, [sp], #16
    ldp x4, x5, [sp], #16
    ldp x6, x7, [sp], #16
    ldp x8, x9, [sp], #16
    ldp x10, x11, [sp], #16
    ldp x12, x13, [sp], #16
    ldp x14, x15, [sp], #16
    ldp x16, x17, [sp], #16
    ldp x18, x19, [sp], #16
    ldp x20, x21, [sp], #16
    ldp x22, x23, [sp], #16
    ldp x24, x25, [sp], #16
    ldp x26, x27, [sp], #16
    ldp x28, x29, [sp], #16
    ldr x30, [sp], #8
    add sp, sp, #8
    eret
