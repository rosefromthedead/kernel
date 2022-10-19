// TODO: add PAR_EL1 reg

#[derive(Debug, PartialEq, Eq)]
pub enum ExceptionClass {
    Unknown,
    SimdOrFpTrapped,
    IllegalExecutionState,
    SvcAa64,
    InstructionAbortLowerEl,
    InstructionAbortSameEl,
    PcAlignment,
    DataAbortLowerEl,
    DataAbortSameEl,
    SpAlignment,
    SError,
    BreakpointLowerEl,
    BreakpointSameEl,
    SoftwareStepLowerEl,
    SoftwareStepSameEl,
    WatchpointLowerEl,
    WatchpointSameEl,
}

#[derive(Debug)]
pub struct ExceptionSyndrome {
    /// if false, 16-bit instruction faulted, otherwise 32-bit instruction or N/A
    pub instr_len: bool,
    pub cause: ExceptionClass,
    pub iss: u32,
}

impl ExceptionSyndrome {
    pub fn get() -> Self {
        let esr: u64;
        unsafe { asm!("mrs {0}, ESR_EL1", out(reg) esr) };
        let instr_len = ((esr >> 25) & 1) == 1;
        let class_bits = esr >> 26 & 0b0011_1111;
        let cause = match class_bits {
            0b000111 => ExceptionClass::SimdOrFpTrapped,
            0b001110 => ExceptionClass::IllegalExecutionState,
            0b010101 => ExceptionClass::SvcAa64,
            0b100000 => ExceptionClass::InstructionAbortLowerEl,
            0b100001 => ExceptionClass::InstructionAbortSameEl,
            0b100010 => ExceptionClass::PcAlignment,
            0b100100 => ExceptionClass::DataAbortLowerEl,
            0b100101 => ExceptionClass::DataAbortSameEl,
            0b100110 => ExceptionClass::SpAlignment,
            0b101111 => ExceptionClass::SError,
            0b110000 => ExceptionClass::BreakpointLowerEl,
            0b110001 => ExceptionClass::BreakpointSameEl,
            0b110010 => ExceptionClass::SoftwareStepLowerEl,
            0b110011 => ExceptionClass::SoftwareStepSameEl,
            0b110100 => ExceptionClass::WatchpointLowerEl,
            0b110101 => ExceptionClass::WatchpointSameEl,
            _ => ExceptionClass::Unknown,
        };
        let iss = (esr & 0x00FF_FFFF) as u32;

        ExceptionSyndrome {
            instr_len,
            cause,
            iss,
        }
    }
}
