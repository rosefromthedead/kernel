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

use alloc::boxed::Box;

use crate::context::ContextState;

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
    let contexts = unsafe { &mut context::CONTEXTS };
    let mut suspended = Box::new(arch::context::SuspendedContext::new());
    let cx = Box::default();
    let mut active = suspended.enter(cx);
    contexts.insert(0, ContextState::Active);
    active.init();
    elf::load_elf(arch.initrd, &mut active).unwrap();

    unsafe { active.jump_to_userspace() };

    panic!("end of main");
}
