use crate::{arch::context::ArchContext, vm::VirtualAddress};

pub struct Context {
    pub state: ContextState,
}

pub enum ContextState {
    Running,
    Suspended(ArchContext),
    Invalid,
}

impl Context {
    pub fn new(entry_virt: VirtualAddress) -> Self {
        println!("{:#018X}", entry_virt.0);
        Context {
            state: ContextState::Suspended(ArchContext::new(entry_virt, VirtualAddress(0x0000_0000_8000_0000))),
        }
    }

    pub unsafe fn enter(&self) {
        crate::arch::context::enter_context(self);
    }
}
