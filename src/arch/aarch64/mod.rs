pub mod vm;

#[link_section = ".early_init"]
#[no_mangle]
#[naked]
unsafe fn early_init() {
    asm!("
        adrp {0}, EARLY_STACK
        add {0}, {0}, #0x1000
        mov sp, {0}

        b main
    ", out(reg) _);
}

pub unsafe fn init_regs() {
    use cortex_a::regs::*;
    // Don't trap on FP/SIMD register access
    CPACR_EL1.write(CPACR_EL1::TTA::None + CPACR_EL1::FPEN::None + CPACR_EL1::ZEN::None);
}
