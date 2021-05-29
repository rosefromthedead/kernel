static mut VECTOR_TABLE: [u8; 2048] = [0; 2048];

pub fn init_interrupts() {
    use cortex_a::regs::{VBAR_EL1, RegisterReadWrite};
    VBAR_EL1.set(unsafe { &VECTOR_TABLE } as *const _ as u64);
}
