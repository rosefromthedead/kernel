.section .interrupt_vectors, "x"

.macro vector name, source, type
.balign 0x80
\name:
    stp x0, x1, [sp, -16]!
    mov x0, #\source
    mov x1, #\type
    bl demux_interrupt
    ldp x0, x1, [sp], #16
    eret
.endm

.global current_el_sp_el0_sync

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
