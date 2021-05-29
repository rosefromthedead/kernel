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
    pub fn new() -> Self {
        let entry_virt = crate::arch::context::very_good_context as usize;
        println!("{:#018X}", entry_virt);
        Context {
            state: ContextState::Suspended(ArchContext::new(VirtualAddress(entry_virt), VirtualAddress(0x0000_0000_8000_0000))),
        }
    }

    pub unsafe fn enter(&self) {
        crate::arch::context::enter_context(self);
    }
}
