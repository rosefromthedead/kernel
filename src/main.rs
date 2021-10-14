#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(asm)]
#![feature(const_fn_trait_bound)]
#![feature(lang_items)]
#![feature(maybe_uninit_extra)]
#![feature(naked_functions)]
#![feature(never_type)]
#![feature(new_uninit)]
#![feature(panic_info_message)]
#![feature(ptr_as_uninit)]

extern crate alloc;

mod arch;
#[macro_use]
mod console;
mod context;
mod memory;
mod panic;
mod tracing;
mod vm;

pub fn main() {

}
