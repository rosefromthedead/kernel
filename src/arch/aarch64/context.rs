use crate::{vm::VirtualAddress, context::ActiveContext};

pub struct CpuState {
    registers: Registers,
}

#[repr(C)]
struct Registers {
    x: [u64; 31],
    sp: u64,

    elr: u64,
    spsr: u64,
}

impl CpuState {
    pub fn new(entry_point: VirtualAddress, stack: VirtualAddress) -> Self {
        CpuState {
            registers: Registers {
                x: [0; 31],
                sp: stack.0 as u64,
                elr: entry_point.0 as u64,
                // TODO: Very dangerous and bad please review
                spsr: 0,
            }
        }
    }

    pub fn get_entry_point(&self) -> VirtualAddress {
        VirtualAddress(self.registers.elr as usize)
    }

    pub fn set_entry_point(&mut self, virt: VirtualAddress) {
        self.registers.elr = virt.0 as u64;
    }
}

pub unsafe fn jump_to_userspace(ctx: &ActiveContext) -> ! {
    let registers = &ctx.user_state.registers as *const _;
    asm!("
        ldr x0, [x30, #248]
        ldr x1, [x30, #256]
        ldr x2, [x30, #264]
        msr SP_EL0, x0
        msr ELR_EL1, x1
        msr SPSR_EL1, x2

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
    ", in("x30") registers);
    unreachable!();
}

#[no_mangle]
#[naked]
pub unsafe extern "C" fn very_good_context() {
    asm!("
        mov x2, #42
        svc #0
        b .
    ", options(noreturn));
}
