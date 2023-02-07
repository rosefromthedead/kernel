use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::{boxed::Box, collections::BTreeMap};

use crate::{
    arch::{self, context::{SuspendedCpuState, ActiveCpuState}, vm::get_current_user_table},
    vm::{PhysicalAddress, Table, TopLevelTable, VirtualAddress},
};

// TODO: make it not static mut
pub static mut CONTEXTS: BTreeMap<usize, ContextEntry> = BTreeMap::new();
pub static CURRENT_CONTEXT: AtomicUsize = AtomicUsize::new(0);

pub enum ContextEntry {
    /// The context is not currently active on any CPU
    Suspended(Box<SuspendedContext>),
    /// The context is active on some CPU and cannot be accessed
    Active,
}

impl ContextEntry {
    pub fn is_suspended(&self) -> bool {
        matches!(self, ContextEntry::Suspended(_))
    }

    pub fn take(&mut self) -> Option<Box<SuspendedContext>> {
        let entry = core::mem::replace(self, ContextEntry::Active);
        match entry {
            ContextEntry::Suspended(ctx) => Some(ctx),
            _ => None,
        }
    }
}

pub struct SuspendedContext {
    user_state: SuspendedCpuState,
    table: PhysicalAddress,
}

impl SuspendedContext {
    pub fn new() -> Self {
        let table = crate::memory::FRAME_ALLOCATOR.lock().alloc();
        SuspendedContext {
            user_state: SuspendedCpuState::new(),
            table,
        }
    }

    pub fn enter(self: Box<Self>) -> ActiveContext {
        let SuspendedContext { user_state, table } = *self;
        let user_state = user_state.enter();
        unsafe { arch::vm::switch_table(table) };
        ActiveContext { user_state }
    }
}

pub struct ActiveContext {
    pub user_state: ActiveCpuState,
}

impl ActiveContext {
    pub fn init(&mut self) {
        let table_phys = arch::vm::get_current_user_table();
        arch::vm::init_user_table(table_phys);
        let table = self.table();
        let sp = VirtualAddress(0x0000_0000_7FFF_F000);
        table.alloc(sp, 4096).unwrap();
        self.user_state.set_stack_pointer(sp + 4096);
    }

    pub fn table(&mut self) -> &mut TopLevelTable {
        unsafe { &mut *(arch::vm::USER_TABLE.0 as *mut _) }
    }

    pub fn jump_to_userspace(&mut self) -> ! {
        unsafe {
            arch::context::jump_to_userspace(self);
        }
    }

    pub fn suspend(self) -> SuspendedContext {
        SuspendedContext {
            user_state: self.user_state.suspend(),
            table: get_current_user_table(),
        }
    }
}

pub fn exit() -> ! {
    // if CURRENT_CONTEXT.load(Ordering::Relaxed) == 0 {
        // safety: only running on qemu means system is always psci :)
        unsafe { crate::arch::platform::shutdown() };
    // } else {
        // todo!()
    // }
}

/// Selects a context to switch to.
pub fn schedule() -> (usize, Box<SuspendedContext>) {
    let contexts = unsafe { &mut CONTEXTS };
    let Some((&id, entry)) =
        contexts.iter_mut().find(|(_i, c)| c.is_suspended()) else { panic!() };
    (id, entry.take().unwrap())
}

pub fn switch(current: ActiveContext, to_id: usize, to: Box<SuspendedContext>) -> ActiveContext {
    // safety: go away
    let contexts = unsafe { &mut CONTEXTS };
    let old_id = CURRENT_CONTEXT.load(Ordering::Relaxed);
    let suspended = current.suspend();
    *contexts.get_mut(&old_id).unwrap() = ContextEntry::Suspended(Box::new(suspended));

    let mut active = to.enter();
    // TODO: check ordering
    CURRENT_CONTEXT.store(to_id, Ordering::Relaxed);
    active
}
