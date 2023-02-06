#![feature(naked_functions)]
#![no_main]
#![no_std]

use core::{arch::asm, fmt::Write};

#[naked]
extern "C" fn print(s: *const u8, len: usize) {
    unsafe {
        asm!("svc #1; ret", options(noreturn));
    }
}

struct Stdout;
impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        print(s.as_ptr(), s.len());
        Ok(())
    }
}

#[no_mangle]
fn _start() {
    let _ = writeln!(Stdout, "Hello, world!");
    let _ = writeln!(Stdout, "I'm having a great time in userspace");
    unsafe { asm!("svc #0", options(noreturn)) }
}

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    unsafe { asm!("svc #0", options(noreturn)) }
}
