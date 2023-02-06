#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(lang_items)]
#![feature(naked_functions)]
#![feature(never_type)]
#![feature(new_uninit)]
#![feature(panic_info_message)]
#![feature(ptr_as_uninit)]
#![feature(pointer_is_aligned)]

use ::tracing::info_span;
use alloc::boxed::Box;

extern crate alloc;

mod arch;
#[macro_use]
mod console;
mod context;
mod elf;
mod memory;
mod panic;
mod syscall;
mod tracing;
mod vm;

pub fn main(arch: arch::Arch) {
    let init_ctx = Box::new(context::SuspendedContext::new());
    let mut active_ctx = init_ctx.enter();
    active_ctx.init();

    elf::load_elf(arch.initrd, &mut active_ctx).unwrap();

    active_ctx.jump_to_userspace();

    panic!("end of main");
}
