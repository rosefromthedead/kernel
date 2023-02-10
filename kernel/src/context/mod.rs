use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::{boxed::Box, collections::BTreeMap, vec::Vec};

use crate::{
    arch::context::{ActiveContext, SuspendedContext},
    vm::Mapping,
};

// TODO: make it not static mut
pub static mut CONTEXTS: BTreeMap<usize, ContextState> = BTreeMap::new();
pub static CURRENT_CONTEXT: AtomicUsize = AtomicUsize::new(0);

pub enum ContextState {
    /// The context is not currently active on any CPU
    Suspended(Box<SuspendedContext>, Box<Context>),
    /// The context is active on some CPU and cannot be accessed
    Active,
}

impl ContextState {
    pub fn is_suspended(&self) -> bool {
        matches!(self, ContextState::Suspended(_, _))
    }

    pub fn take(&mut self) -> Option<(Box<SuspendedContext>, Box<Context>)> {
        let entry = core::mem::replace(self, ContextState::Active);
        match entry {
            ContextState::Suspended(suspended, cx) => Some((suspended, cx)),
            _ => None,
        }
    }
}

#[derive(Default)]
pub struct Context {
    text: Vec<Mapping>,
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
pub fn schedule() -> (usize, (Box<SuspendedContext>, Box<Context>)) {
    let contexts = unsafe { &mut CONTEXTS };
    let Some((&id, entry)) =
        contexts.iter_mut().find(|(_i, s)| s.is_suspended()) else { panic!() };
    (id, entry.take().unwrap())
}

pub fn switch(current: ActiveContext, to_id: usize, to: (Box<SuspendedContext>, Box<Context>)) -> ActiveContext {
    tracing::trace!("switching to context {to_id}");
    // safety: go away
    let contexts = unsafe { &mut CONTEXTS };
    let old_id = CURRENT_CONTEXT.load(Ordering::Relaxed);
    let (suspended, cx) = current.suspend();
    *contexts.get_mut(&old_id).unwrap() = ContextState::Suspended(Box::new(suspended), cx);

    let mut active = to.0.enter(to.1);
    // TODO: check ordering
    CURRENT_CONTEXT.store(to_id, Ordering::Relaxed);
    active
}
