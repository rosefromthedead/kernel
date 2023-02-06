// PSCI based platform control

/// system must be a psci system else UB
pub unsafe fn shutdown() -> ! {
    let x: usize = 0x84000008;
    unsafe {
        core::arch::asm!("hvc #0", in("x0") x, options(noreturn));
    }
}
