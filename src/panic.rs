#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

#[panic_handler]
#[no_mangle]
pub extern "C" fn rust_begin_unwind(_: &core::panic::PanicInfo) -> ! {
    loop {
        unsafe { asm!("wfi"); }
    }
}
