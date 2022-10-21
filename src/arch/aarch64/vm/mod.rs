use crate::vm::{PhysicalAddress, VirtualAddress};
use table::{IntermediateTable, Level0, Level1, Level2};

pub type TopLevelTable = IntermediateTable<Level0>;

/// # Safety
/// Good luck
pub unsafe fn switch_table(phys: PhysicalAddress) {
    asm!("
        msr TTBR0_EL1, {0}
        tlbi vmalle1
        isb
    ", in(reg) phys.0);
}

pub(super) mod table;
mod fmt;

pub(super) const KERNEL_OFFSET: usize = 0xFFFF_0000_0000_0000;
pub(super) const KERNEL_LOAD_PHYS: PhysicalAddress = PhysicalAddress(0x4020_0000);
pub(super) const KERNEL_HEAP_START: VirtualAddress = VirtualAddress(0xFFFF_1000_8000_0000);
pub const USER_TABLE: VirtualAddress = VirtualAddress(0x0000_FFFF_FFFF_F000);

#[no_mangle]
pub static mut KERNEL_TABLE: IntermediateTable<Level0> = IntermediateTable::new();
#[no_mangle]
pub static mut KERNEL_REMAP_L1: IntermediateTable<Level1> = IntermediateTable::new();
#[no_mangle]
pub static mut KERNEL_REMAP_L2: IntermediateTable<Level2> = IntermediateTable::new();
#[no_mangle]
pub static mut DIRECT_MAP: IntermediateTable<Level1> = IntermediateTable::new();
#[no_mangle]
pub static mut KERNEL_IDENTITY_L0: IntermediateTable<Level0> = IntermediateTable::new();
#[no_mangle]
pub static mut KERNEL_IDENTITY_L1: IntermediateTable<Level1> = IntermediateTable::new();
