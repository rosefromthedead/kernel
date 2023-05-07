use core::{
    cell::UnsafeCell,
    mem::ManuallyDrop,
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
};

use alloc::{boxed::Box, collections::BTreeMap, vec::Vec};
use ring_buffer::RingBuffer;

use crate::{
    arch::context::{ActiveContext, SuspendedContext},
    vm::{Mapping, VirtualAddress},
};

// TODO: make it not static mut
pub static mut CONTEXTS: BTreeMap<usize, Pin<Box<Context>>> = BTreeMap::new();
pub static SCHED_QUEUE: RingBuffer<usize, 512> = RingBuffer::new();

pub struct Context {
    pub id: usize,
    /// Be very very careful with this one!
    active: AtomicBool,
    /// Data that can only be mutably borrowed by a CPU if the context is active there
    thread_local: UnsafeCell<ThreadLocal>,
    /// Arch-specific state if the context is suspended, or uninit if it's active
    arch: UnsafeCell<ArchContext>,
}

union ArchContext {
    suspended: ManuallyDrop<SuspendedContext>,
    active: ManuallyDrop<ActiveContext>,
}

#[repr(transparent)]
pub struct ActiveContextHandle(pub *const Context);

impl Context {
    pub fn new(id: usize) -> Pin<Box<Self>> {
        Box::pin(Context {
            id,
            active: AtomicBool::new(false),
            thread_local: UnsafeCell::new(ThreadLocal::new()),
            arch: UnsafeCell::new(ArchContext {
                suspended: ManuallyDrop::new(SuspendedContext::new()),
            }),
        })
    }

    /// # Safety
    /// Must be called when no other context is active on this CPU. This is either at boot time or
    /// in the implementation of [`ActiveContextHandle::switch_to`].
    pub unsafe fn enter(&self) -> ActiveContextHandle {
        loop {
            if self
                .active
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
        let arch = self.arch.get();
        let this = self as *const _;
        unsafe {
            (*arch).active =
                ManuallyDrop::new(ManuallyDrop::into_inner(arch.read().suspended).enter(this));
        }
        ActiveContextHandle(this)
    }

    /// Get a handle to self while setting its arch state to `state`.
    pub unsafe fn get_handle(&self, state: ActiveContext) -> ActiveContextHandle {
        self.arch.get().write(ArchContext {
            active: ManuallyDrop::new(state),
        });
        ActiveContextHandle(self as *const _)
    }
}

impl ActiveContextHandle {
    pub fn context(&self) -> &Context {
        unsafe { &*self.0 }
    }

    /// Get a mutable reference to the arch-specific parts of a context that can only be accessed
    /// when the context is active.
    pub fn arch(&mut self) -> &mut ActiveContext {
        unsafe { &mut (*(*self.0).arch.get()).active }
    }

    pub fn set_entry_point(&mut self, virt: VirtualAddress) {
        self.arch().set_entry_point(virt)
    }

    pub unsafe fn jump_to_userspace(&mut self) -> ! {
        self.arch().jump_to_userspace()
    }

    pub fn switch_to(self, other: &Context) -> Self {
        if core::ptr::eq(self.context(), other) {
            return self;
        }
        unsafe {
            let self_active = ManuallyDrop::into_inner(self.context().arch.get().read().active);
            let activate =
                other
                    .active
                    .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed);
            if activate.is_err() {
                panic!("tried to switch to an active context!");
            }
            let other_suspended = ManuallyDrop::into_inner(other.arch.get().read().suspended);
            let (self_suspended, _) = self_active.suspend();
            self.context().arch.get().write(ArchContext {
                suspended: ManuallyDrop::new(self_suspended),
            });
            // by self existing, active is false, so we don't need compare-exchange
            self.context().active.store(false, Ordering::Release);
            core::mem::forget(self);
            let other_ptr = other as *const _;
            other_suspended.enter(other_ptr);
            ActiveContextHandle(other_ptr)
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        if self.active.load(Ordering::Relaxed) {
            tracing::error!("Context dropped while active! Expect ghosts!");
        }
    }
}

impl Drop for ActiveContextHandle {
    fn drop(&mut self) {
        tracing::error!("Active context handle dropped. Expect issues.");
    }
}

pub struct ThreadLocal {
    text: Vec<Mapping>,
}

impl ThreadLocal {
    fn new() -> Self {
        ThreadLocal { text: Vec::new() }
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
