use linked_list_allocator::Heap;

#[repr(align(4096))]
pub struct Page([u8; 4096]);

// should be enough to set up the frame allocator
static mut KERNEL_HEAP: [Page; 1] = [Page([0; 4096]); 1];

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
static mut EARLY_STACK: Page = Page([0; 4096]);

pub fn init_heap() {
    unsafe {
        let ptr = &mut KERNEL_HEAP as *mut _ as usize;
        // weird initialisation behaviour
        crate::memory::KERNEL_HEAP_ALLOCATOR.force_unlock();
        *crate::memory::KERNEL_HEAP_ALLOCATOR.lock() = Heap::new(ptr, 4096);
    }
}
