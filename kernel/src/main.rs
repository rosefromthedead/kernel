#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(asm)]
#![feature(const_btree_new)]
#![feature(const_fn_trait_bound)]
#![feature(lang_items)]
#![feature(maybe_uninit_extra)]
#![feature(naked_functions)]
#![feature(never_type)]
#![feature(new_uninit)]
#![feature(panic_info_message)]
#![feature(ptr_as_uninit)]

use alloc::boxed::Box;
use ::tracing::info_span;

extern crate alloc;

mod arch;
#[macro_use]
mod console;
mod context;
mod elf;
mod interrupt;
mod memory;
mod panic;
mod tracing;
mod vm;

pub fn main(arch: arch::Arch) {
    let span = info_span!("kernel main");
    let _guard = span.enter();

    let init_ctx = Box::new(context::SuspendedContext::new());
    let mut active_ctx = init_ctx.enter();
    active_ctx.init();

    elf::load_elf(arch.initrd, &mut active_ctx).unwrap();

    unsafe { active_ctx.jump_to_userspace(); }

    panic!("end of main");
}
