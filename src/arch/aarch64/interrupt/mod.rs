use cortex_a::interfaces::Writeable;

use crate::println;

// the first one in the table == base address of vector table
extern "C" {
    fn current_el_sp_el0_sync();
}

pub fn init_interrupts() {
    use cortex_a::registers::VBAR_EL1;
    VBAR_EL1.set(current_el_sp_el0_sync as *const fn() as u64);
}

#[repr(C)]
#[derive(Debug)]
enum InterruptSource {
    CurrentElSpEl0 = 0,
    CurrentElSpElx = 1,
    LowerElAa64 = 2,
    LowerElAa32 = 3,
}

#[repr(C)]
#[derive(Debug)]
enum InterruptType {
    Synchronous = 0,
    Irq = 1,
    Fiq = 2,
    SError = 3,
}

#[no_mangle]
extern "C" fn demux_interrupt(source: InterruptSource, ty: InterruptType) {
    println!("hello from interrupt handler");
    println!("source: {:?}, type: {:?}", source, ty);
    loop {}
}
