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

use crate::context::{Context, CONTEXTS};

extern crate alloc;

mod arch;
#[macro_use]
mod console;
mod context;
mod elf;
mod fmt;
mod memory;
mod panic;
mod syscall;
mod tracing;
mod vm;

pub fn main(arch: arch::Arch) {
    let context = unsafe { &mut CONTEXTS }
        .entry(0)
        .or_insert(Context::new(0))
        .as_ref();
    let mut active = unsafe { context.enter() };
    active.arch().init();
    elf::load_elf(arch.initrd, &mut active).unwrap();

    unsafe { active.jump_to_userspace() };

    panic!("end of main");
}
