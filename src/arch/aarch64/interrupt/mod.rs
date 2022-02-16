use cortex_a::interfaces::Writeable;
use tracing::{info, info_span};

// the first one in the table == base address of vector table
extern "C" {
    fn current_el_sp_el0_sync();
}

pub fn init_interrupts() {
    use cortex_a::registers::VBAR_EL1;
    VBAR_EL1.set(current_el_sp_el0_sync as *const fn() as u64);
}

// allow dead code because rustc thinks we never construct these variants; we construct them in
// assembly in interrupts.S
// same for InterruptType and InterruptArgs
#[allow(dead_code)]
#[repr(C)]
#[derive(Debug)]
enum InterruptSource {
    CurrentElSpEl0 = 0,
    CurrentElSpElx = 1,
    LowerElAa64 = 2,
    LowerElAa32 = 3,
}

#[allow(dead_code)]
#[repr(C)]
#[derive(Debug)]
enum InterruptType {
    Synchronous = 0,
    Irq = 1,
    Fiq = 2,
    SError = 3,
}

#[no_mangle]
extern "C" fn demux_interrupt(
    source: InterruptSource,
    ty: InterruptType,
    a: u64,
    b: u64,
    c: u64,
    d: u64,
    e: u64,
    f: u64,
) {
    let link: u64;
    unsafe { asm!("
        mrs {0}, ELR_EL1
    ", out(reg) link) };
    let syndrome = super::regs::ExceptionSyndrome::get();
    let span = info_span!("interrupt handler", src=?source, ?ty, a, b, c, d, e, f, ?syndrome, link);
    let _guard = span.enter();
    info!(target: "interrupt handler", "hello from interrupt handler");
    match a {
        0 => {
            // print
            let message_bytes = unsafe { core::slice::from_raw_parts(e as *const _, f as usize) };
            let message: &str = core::str::from_utf8(message_bytes).unwrap();
            crate::interrupt::print(message);
        },
        _ => panic!("invalid interrupt number {}", a),
    }
}
