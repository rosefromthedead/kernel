use linked_list_allocator::Heap;

use crate::{vm::{PhysicalAddress, Table}, arch::vm::table::{IntermediateLevel, Level1, Level2}};

use super::vm::{table::{IntermediateTable, Level0}, KERNEL_HEAP_START};

#[repr(align(4096))]
pub struct Page([u8; 4096]);

// should be enough to set up the frame allocator
static mut EARLY_HEAP: [Page; 1] = [Page([0; 4096])];

// for mapping the device tree, before we know where free memory is
// annoying syntax
pub static mut SPARE_FRAMES: [Page; 8] = [
    Page([0; 4096]),
    Page([0; 4096]),
    Page([0; 4096]),
    Page([0; 4096]),
    Page([0; 4096]),
    Page([0; 4096]),
    Page([0; 4096]),
    Page([0; 4096]),
];

#[no_mangle]
static mut EARLY_STACK: [Page; 2] = [Page([0; 4096]), Page([0; 4096])];

pub fn init_early_heap(table: &mut IntermediateTable<Level0>) {
    static mut HEAP_MAP_FRAMES: [Page; 3] = [Page([0; 4096]), Page([0; 4096]), Page([0; 4096])];
    unsafe {
        let heap_virt = &mut EARLY_HEAP as *mut _ as usize;
        let heap_phys: usize;
        let map_frames_virt = &mut HEAP_MAP_FRAMES as *mut _ as usize;
        let map_frames_phys: usize;
        core::arch::asm!("
            at s1e1r, {0}
            mrs {1}, PAR_EL1
            at s1e1r, {2}
            mrs {3}, PAR_EL1
        ", in(reg) heap_virt, out(reg) heap_phys,
           in(reg) map_frames_virt, lateout(reg) map_frames_phys);
        let heap_phys = PhysicalAddress(heap_phys);
        let map_frames_phys = PhysicalAddress(map_frames_phys);

        let l0_index = KERNEL_HEAP_START.0 >> Level0::VIRT_SHIFT_AMT & 0x1ff;
        let l1_index = KERNEL_HEAP_START.0 >> Level1::VIRT_SHIFT_AMT & 0x1ff;
        let l2_index = KERNEL_HEAP_START.0 >> Level2::VIRT_SHIFT_AMT & 0x1ff;

        table.insert_raw(map_frames_phys, l0_index);

        let level1 = table.entry_mut(l0_index).get_next_table_mut().unwrap();
        level1.insert_raw(map_frames_phys + 4096, l1_index);
        let level2 = level1.entry_mut(l1_index).get_next_table_mut().unwrap();
        level2.insert_raw(map_frames_phys + 8192, l2_index);
        let level3 = level2.entry_mut(l2_index).get_next_table_mut().unwrap();
        level3.map_to(KERNEL_HEAP_START, heap_phys, 4096);

        // weird initialisation behaviour
        crate::memory::KERNEL_HEAP_ALLOCATOR.force_unlock();
        *crate::memory::KERNEL_HEAP_ALLOCATOR.lock() = Heap::new(KERNEL_HEAP_START.0, 4096);
    }
}

pub fn init_main_heap(table: &mut IntermediateTable<Level0>) {
    table.alloc(KERNEL_HEAP_START + 4096, 1048576 - 4096).unwrap();
    unsafe { crate::memory::KERNEL_HEAP_ALLOCATOR.lock().extend(1048576 - 4096) };
}
