#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(lang_items)]
#![feature(naked_functions)]
#![feature(new_uninit)]

extern crate alloc;

mod arch;
#[macro_use]
mod console;
mod memory;
mod panic;
mod vm;

#[no_mangle]
unsafe fn main() {
    arch::init_regs();
    console::virt::virt_putchar(b'a');

    arch::vm::VirtMem::init();
    println!("Hello, universe!");

    loop {
        cortex_a::asm::wfi();
    }
}
