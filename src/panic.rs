#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

#[panic_handler]
#[no_mangle]
pub extern "C" fn rust_begin_unwind(info: &core::panic::PanicInfo) -> ! {
    match info.location() {
        Some(loc) => tracing::error!(
            file=loc.file(),
            line=loc.line(),
            col=loc.column(),
            "panic encountered",
        ),
        None => tracing::error!("panic encountered"),
    }
    if let Some(message) = info.message() {
        println!("panic message: {:#?}", message);
    }

    loop {
        unsafe { asm!("wfi"); }
    }
}
