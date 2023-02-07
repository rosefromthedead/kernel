use core::{arch::asm, fmt::Display};

use crate::{context::ActiveContext, vm::VirtualAddress};

pub struct SuspendedCpuState {
    registers: Registers,
    sp: VirtualAddress,

    elr: VirtualAddress,
    spsr: u64,
}

pub struct ActiveCpuState {
    registers: Registers,
    // sp, elr, spsr are stored in their registers upon context entry
}

#[repr(C)]
pub(super) struct Registers {
    pub x: [usize; 31],
}

impl SuspendedCpuState {
    pub fn new() -> Self {
        SuspendedCpuState {
            registers: Registers { x: [0; 31] },
            sp: VirtualAddress(0),
            elr: VirtualAddress(0),
            // TODO: Very dangerous and bad please review
            spsr: 0,
        }
    }

    pub fn enter(self) -> ActiveCpuState {
        let SuspendedCpuState { registers, sp, elr, spsr } = self;
        unsafe {
            asm!("msr SP_EL0, {0}", in(reg) sp.0, options(nomem, nostack, preserves_flags));
            asm!("msr ELR_EL1, {0}", in(reg) elr.0, options(nomem, nostack, preserves_flags));
            asm!("msr SPSR_EL1, {0}", in(reg) spsr, options(nomem, nostack, preserves_flags));
        }
        ActiveCpuState { registers }
    }
}

impl ActiveCpuState {
    pub fn set_entry_point(&mut self, virt: VirtualAddress) {
        unsafe { asm!("msr ELR_EL1, {0}", in(reg) virt.0, options(nomem, nostack, preserves_flags)) }
    }

    pub fn set_stack_pointer(&mut self, virt: VirtualAddress) {
        unsafe { asm!("msr SP_EL0, {0}", in(reg) virt.0, options(nomem, nostack, preserves_flags)) }
    }
}

pub unsafe fn jump_to_userspace(ctx: &ActiveContext) -> ! {
    let registers = &ctx.user_state as *const _;
    asm!("
        adrp x0, EARLY_STACK
        add x0, x0, #0x2000
        mov sp, x0

        ldr x0, [x30, #0]
        ldr x1, [x30, #8]
        ldr x2, [x30, #16]
        ldr x3, [x30, #24]
        ldr x4, [x30, #32]
        ldr x5, [x30, #40]
        ldr x6, [x30, #48]
        ldr x7, [x30, #56]
        ldr x8, [x30, #64]
        ldr x9, [x30, #72]
        ldr x10, [x30, #80]
        ldr x11, [x30, #88]
        ldr x12, [x30, #96]
        ldr x13, [x30, #104]
        ldr x14, [x30, #112]
        ldr x15, [x30, #120]
        ldr x16, [x30, #128]
        ldr x17, [x30, #136]
        ldr x18, [x30, #144]
        ldr x19, [x30, #152]
        ldr x20, [x30, #160]
        ldr x21, [x30, #168]
        ldr x22, [x30, #176]
        ldr x23, [x30, #184]
        ldr x24, [x30, #192]
        ldr x25, [x30, #200]
        ldr x26, [x30, #208]
        ldr x27, [x30, #216]
        ldr x28, [x30, #224]
        ldr x29, [x30, #232]
        ldr x30, [x30, #240]

        eret
    ", in("x30") registers, options(noreturn));
}

impl Display for Registers {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for (i, reg) in self.x.iter().enumerate() {
            if i < 10 {
                write!(f, " ")?;
            }
            write!(f, "x{i}: {:#018x} ", reg)?;
            match i % 4 {
                3 => writeln!(f)?,
                _ => {}
            }
        }
        Ok(())
    }
}
