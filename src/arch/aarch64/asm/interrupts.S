.section .interrupt_vectors, "x"

.global current_el_sp_el0_sync

current_el_sp_el0_sync:
    mov x0, #0
    mov x1, #0
    bl demux_interrupt
    b .
.balign 0x80
current_el_sp_el0_irq:
    mov x0, #0
    mov x1, #1
    bl demux_interrupt
    b .
.balign 0x80
current_el_sp_el0_fiq:
    mov x0, #0
    mov x1, #2
    bl demux_interrupt
    b .
.balign 0x80
current_el_sp_el0_serror:
    mov x0, #0
    mov x1, #3
    bl demux_interrupt
    b .
.balign 0x80
current_el_sp_elx_sync:
    mov x0, #1
    mov x1, #0
    bl demux_interrupt
    b .
.balign 0x80
current_el_sp_elx_irq:
    mov x0, #1
    mov x1, #1
    bl demux_interrupt
    b .
.balign 0x80
current_el_sp_elx_fiq:
    mov x0, #1
    mov x1, #2
    bl demux_interrupt
    b .
.balign 0x80
current_el_sp_elx_serror:
    mov x0, #1
    mov x1, #3
    bl demux_interrupt
    b .
.balign 0x80
lower_el_aa64_sync:
    mov x0, #2
    mov x1, #0
    bl demux_interrupt
    b .
.balign 0x80
lower_el_aa64_irq:
    mov x0, #2
    mov x1, #1
    bl demux_interrupt
    b .
.balign 0x80
lower_el_aa64_fiq:
    mov x0, #2
    mov x1, #2
    bl demux_interrupt
    b .
.balign 0x80
lower_el_aa64_serror:
    mov x0, #2
    mov x1, #3
    bl demux_interrupt
    b .
.balign 0x80
lower_el_aa32_sync:
    mov x0, #3
    mov x1, #0
    bl demux_interrupt
    b .
.balign 0x80
lower_el_aa32_irq:
    mov x0, #3
    mov x1, #1
    bl demux_interrupt
    b .
.balign 0x80
lower_el_aa32_fiq:
    mov x0, #3
    mov x1, #2
    bl demux_interrupt
    b .
.balign 0x80
lower_el_aa32_serror:
    mov x0, #3
    mov x1, #3
    bl demux_interrupt
    b .
