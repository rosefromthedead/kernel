.section .interrupt_vectors, "x"

.global current_el_sp_el0_sync

current_el_sp_el0_sync:
    nop
.balign 0x80
current_el_sp_el0_irq:
    nop
.balign 0x80
current_el_sp_el0_fiq:
    nop
.balign 0x80
current_el_sp_el0_serror:
    nop
.balign 0x80
current_el_sp_elx_sync:
    mov x0, #1
    mov x1, #0
    b demux_interrupt
.balign 0x80
current_el_sp_elx_irq:
    nop
.balign 0x80
current_el_sp_elx_fiq:
    nop
.balign 0x80
current_el_sp_elx_serror:
    nop
.balign 0x80
lower_el_aa64_sync:
    nop
.balign 0x80
lower_el_aa64_irq:
    nop
.balign 0x80
lower_el_aa64_fiq:
    nop
.balign 0x80
lower_el_aa64_serror:
    nop
