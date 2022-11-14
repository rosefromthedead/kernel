#![no_main]
#![no_std]

use core::{arch::asm, fmt::Write};

struct Stdout;
impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let s_addr = s.as_ptr() as usize;
        let s_len = s.len();
        unsafe {
            asm!("svc #1", in("x6") s_addr, in("x7") s_len);
        }
        Ok(())
    }
}

#[no_mangle]
fn _start() {
    let _res = writeln!(Stdout, "Hello, world!");
    unsafe { asm!("svc #0", options(noreturn)) }
}

#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
