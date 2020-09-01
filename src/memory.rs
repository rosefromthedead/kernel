use buddy_system_allocator::{LockedFrameAllocator, LockedHeap};
use spin::Once;

#[repr(align(4096))]
struct Page([u8; 4096]);

// shh
static mut EARLY_HEAP: [Page; 8] = [
    Page([0; 4096]),
    Page([0; 4096]),
    Page([0; 4096]),
    Page([0; 4096]),
    Page([0; 4096]),
    Page([0; 4096]),
    Page([0; 4096]),
    Page([0; 4096]),
];

// SHH
#[no_mangle]
static mut EARLY_STACK: Page = Page([0; 4096]);

pub fn init_early_heap() {
    unsafe {
        let ptr = &mut EARLY_HEAP as *mut _ as usize;
        ALLOCATOR.lock().add_to_heap(ptr, ptr + 4096 * 8);
    }
}

#[global_allocator]
pub static mut ALLOCATOR: LockedHeap = LockedHeap::new();

#[alloc_error_handler]
pub fn alloc_error_handler(_: core::alloc::Layout) -> ! {
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

pub static FRAME_ALLOCATOR: Once<LockedFrameAllocator> = Once::new();
