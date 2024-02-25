use core::arch::asm;
use core::mem::MaybeUninit;

use crate::vm::{PhysicalAddress, Table, VirtualAddress};
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

pub fn init_user_table(phys: PhysicalAddress) {
    unsafe {
        KERNEL_TABLE.map_to(USER_TABLE_SCRATCH, phys, 4096).unwrap();
        let new_table_uninit =
            unsafe { &mut *(USER_TABLE_SCRATCH.0 as *mut MaybeUninit<TopLevelTable>) };
        let init: &mut TopLevelTable = Table::clear(new_table_uninit);
        // recursive mapping!
        unsafe {
            init.insert_raw(get_current_user_table(), 511).unwrap();
            asm!(
                "
                tlbi vmalle1
                isb
            "
            );
        }
        KERNEL_TABLE.unmap(USER_TABLE_SCRATCH, 4096);
    }
}

pub fn get_current_user_table() -> PhysicalAddress {
    let table_phys: usize;
    unsafe {
        asm!("
            mrs {0}, TTBR0_EL1
        ", out(reg) table_phys);
    }
    PhysicalAddress(table_phys & 0x0000_FFFF_FFFF_FFFE)
}

mod fmt;
pub(super) mod table;

pub(super) const KERNEL_OFFSET: usize = 0xFFFF_0000_0000_0000;
pub(super) const KERNEL_LOAD_PHYS: PhysicalAddress = PhysicalAddress(0x4020_0000);
pub(super) const KERNEL_HEAP_START: VirtualAddress = VirtualAddress(0xFFFF_1000_8000_0000);
pub const USER_TABLE: VirtualAddress = VirtualAddress(0x0000_FFFF_FFFF_F000);
const USER_TABLE_SCRATCH: VirtualAddress = VirtualAddress(0xFFFF_0000_1000_0000);

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
