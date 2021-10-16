use crate::{arch::context::CpuState, vm::{TopLevelTable, VirtualAddress}};

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
        Context {
            state: ContextState::Suspended(CpuState::new(VirtualAddress(0), VirtualAddress(0x0000_0000_8000_0000))),
            table: TopLevelTable::new_top_level(),
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
}
