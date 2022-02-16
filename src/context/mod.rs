use core::{cell::Cell, sync::atomic::{AtomicUsize, Ordering}};

use alloc::collections::BTreeMap;

use crate::{arch::context::CpuState, vm::{Table, TopLevelTable, VirtualAddress}};

// TODO: make it not static mut
static mut CONTEXTS: BTreeMap<usize, Context> = BTreeMap::new();
pub static CURRENT_CONTEXT: AtomicUsize = AtomicUsize::new(0);

pub struct Context {
    pub state: ContextState,
    pub table: &'static mut TopLevelTable,
}

pub enum ContextState {
    Running,
    Suspended(CpuState),
    Invalid,
}

impl Context {
    pub fn new() -> Self {
        let table = TopLevelTable::new_top_level();
        let stack_pointer = VirtualAddress(0x0000_0000_8000_0000);
        table.alloc(stack_pointer, 4096).unwrap();
        Context {
            state: ContextState::Suspended(CpuState::new(VirtualAddress(0), stack_pointer + 4096)),
            table,
        }
    }

    pub fn get_entry_point(&self) -> VirtualAddress {
        match self.state {
            ContextState::Suspended(ref state) => state.get_entry_point(),
            _ => panic!("tried to get entry point on running or invalid process"),
        }
    }

    pub fn set_entry_point(&mut self, virt: VirtualAddress) {
        match self.state {
            ContextState::Suspended(ref mut state) => state.set_entry_point(virt),
            _ => panic!("tried to set entry point on running or invalid process"),
        }
    }

    pub unsafe fn enter(&self) {
        self.table.switch_el0_top_level();
    }

    pub unsafe fn jump_to_userspace(&self) {
        crate::arch::context::jump_to_userspace(&self);
    }

    pub unsafe fn get_current() -> &'static Self {
        CONTEXTS.get(&CURRENT_CONTEXT.load(Ordering::Relaxed)).unwrap()
    }
}
