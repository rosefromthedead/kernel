use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::{collections::BTreeMap, boxed::Box};

use crate::{arch::{self, context::CpuState}, vm::{Table, TopLevelTable, VirtualAddress, PhysicalAddress}};

// TODO: make it not static mut
static mut CONTEXTS: BTreeMap<usize, ContextEntry> = BTreeMap::new();
pub static CURRENT_CONTEXT: AtomicUsize = AtomicUsize::new(0);

enum ContextEntry {
    /// The context is not currently active on any CPU
    Suspended(Box<SuspendedContext>),
    /// The context is active on some CPU and cannot be accessed
    Active,
}

impl ContextEntry {
    pub fn take(&mut self) -> Option<Box<SuspendedContext>> {
        let entry = core::mem::replace(self, ContextEntry::Active);
        match entry {
            ContextEntry::Suspended(ctx) => Some(ctx),
            _ => None,
        }
    }
}

pub struct SuspendedContext {
    user_state: CpuState,
    table: PhysicalAddress,
}

impl SuspendedContext {
    pub fn new() -> Self {
        let table = crate::memory::FRAME_ALLOCATOR.lock().alloc();
        SuspendedContext {
            user_state: CpuState::new(),
            table,
        }
    }

    pub fn enter(self: Box<Self>) -> ActiveContext {
        // TODO: save old context
        let SuspendedContext { user_state, table } = *self;
        unsafe { arch::vm::switch_table(table) };
        ActiveContext {
            user_state,
        }
    }

    pub unsafe fn exit() {
        // TODO: race condition??
        if CURRENT_CONTEXT.load(Ordering::Relaxed) == 0 {
            crate::arch::platform::shutdown();
        }
    }
}

pub struct ActiveContext {
    pub user_state: CpuState,
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

    pub fn user_state(&self) -> &CpuState {
        &self.user_state
    }

    pub fn jump_to_userspace(&mut self) {
        unsafe { arch::context::jump_to_userspace(self); }
    }
}
