use core::fmt::{Result, Write};

pub mod virt;

pub static mut WRITER: Writer = Writer(virt::virt_putchar);

pub struct Writer(pub fn(u8));

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> Result {
        for byte in s.bytes() {
            (self.0)(byte);
        }
        Ok(())
    }
}

pub fn get_writer() -> &'static mut Writer {
    unsafe { &mut WRITER }
}

// Thanks, Redox!
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        let _ = write!($crate::console::get_writer(), $($arg)*);
    });
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(concat!($fmt, "\n"), $($arg)*));
}
