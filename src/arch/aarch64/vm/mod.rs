use crate::vm::PhysicalAddress;
use table::{IntermediateTable, Level0, Level1, Level2};

pub(super) mod table;

pub(super) const KERNEL_OFFSET: usize = 0xFFFF_0000_0000_0000;
pub(super) const KERNEL_LOAD_PHYS: PhysicalAddress = PhysicalAddress(0x4020_0000);

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
