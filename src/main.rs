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

use ::tracing::info_span;

extern crate alloc;

mod arch;
#[macro_use]
mod console;
mod context;
mod elf;
mod memory;
mod panic;
mod tracing;
mod vm;

pub fn main(arch: arch::Arch) {
    let span = info_span!("kernel main");
    let _guard = span.enter();

    let mut init_ctx = context::Context::new();
    unsafe { init_ctx.enter(); }
    elf::load_elf(arch.initrd, &mut init_ctx).unwrap();
    unsafe { init_ctx.jump_to_userspace(); }

    panic!("end of main");
}
