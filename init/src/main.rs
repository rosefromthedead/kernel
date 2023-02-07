#![feature(naked_functions)]
#![no_main]
#![no_std]

use core::{arch::asm, fmt::Write};

#[naked]
extern "C" fn sys_print(s: *const u8, len: usize) {
    unsafe {
        asm!("svc #1; ret", options(noreturn));
    }
}

#[naked]
extern "C" fn sys_yield() {
    unsafe {
        asm!("svc #2; ret", options(noreturn));
    }
}

struct Stdout;
impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        sys_print(s.as_ptr(), s.len());
        Ok(())
    }
}

#[no_mangle]
fn _start() {
    let _ = writeln!(Stdout, "Hello, world!");
    sys_yield();
    let _ = writeln!(Stdout, "I'm having a great time in userspace");
    sys_yield();
    unsafe { asm!("svc #0", options(noreturn)) }
}

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    unsafe { asm!("svc #0", options(noreturn)) }
}
