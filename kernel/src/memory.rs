use alloc::vec::Vec;
use crate::{arch::FRAME_SIZE, vm::PhysicalAddress};
use linked_list_allocator::LockedHeap;

#[global_allocator]
pub static KERNEL_HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub static FRAME_ALLOCATOR: spin::Mutex<FrameAllocator> = spin::Mutex::new(FrameAllocator::empty());

#[alloc_error_handler]
pub fn alloc_error_handler(_: core::alloc::Layout) -> ! {
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

pub struct FrameAllocator {
    holes: Vec<Hole>,
}

struct Hole {
    start: usize,
    size: usize,
}

pub struct Chunk {
    pub phys: PhysicalAddress,
    pub size: usize,
}

impl FrameAllocator {
    pub const fn empty() -> Self {
        Self {
            holes: Vec::new(),
        }
    }

    pub fn insert_hole(&mut self, start: PhysicalAddress, size: usize) {
        let start = start.0 / FRAME_SIZE;
        let size = size / FRAME_SIZE;
        
        // The holes list is sorted; find where the new one should go
        let mut idx = 0;
        while let Some(hole) = self.holes.get(idx) {
            if hole.start > start {
                break;
            }
            idx += 1;
        }

        // Check if the new hole begins exactly at the end of the previous one, and therefore can be
        // squished into it. Note that there may not be a previous hole
        if idx != 0 {
            let prev_hole = &mut self.holes[idx - 1];
            // Assert that the holes do not overlap
            assert!(prev_hole.start + prev_hole.size <= start);
            if prev_hole.start + prev_hole.size == start {
                prev_hole.size += size;
                return;
            }
        }

        // Check if the new hole ends exactly at the start of the next hole, etc etc. Note again
        // that there may not be a next hole
        if let Some(next_hole) = self.holes.get_mut(idx) {
            // More overlap checks
            assert!(start + size <= next_hole.start);
            if start + size == next_hole.start {
                next_hole.start -= size;
            }
        }

        // Otherwise we just insert a new hole
        self.holes.insert(idx, Hole { start, size });
    }

    pub fn alloc(&mut self) -> PhysicalAddress {
        let phys = PhysicalAddress(self.holes[0].start * FRAME_SIZE);
        self.holes[0].start += 1;
        self.holes[0].size -= 1;
        if self.holes[0].size == 0 {
            self.holes.remove(0);
        }
        phys
    }

    pub fn alloc_range(&mut self, size: usize) -> Chunk {
        let size = (size + FRAME_SIZE - 1) / FRAME_SIZE;
        let mut phys = None;
        let mut out_size = 0;
        for hole in self.holes.iter_mut() {
            if hole.size >= size {
                phys = Some(hole.start * FRAME_SIZE);
                out_size = size * FRAME_SIZE;
                hole.start += size;
                hole.size -= size;
            }
        }

        if phys == None {
            phys = Some(self.holes[0].start * FRAME_SIZE);
            out_size = self.holes[0].size * FRAME_SIZE;
            self.holes.remove(0);
        } else if self.holes[0].size == 0 {
            // clean up after the loop; can't remove element when we're iterating
            self.holes.remove(0);
        }

        Chunk {
            phys: PhysicalAddress(phys.unwrap()),
            size: out_size,
        }
    }

    pub fn dealloc(&mut self, phys: PhysicalAddress) {
        self.insert_hole(phys, 1);
    }
}


/*
pub struct IntrusiveLinkedListAllocator {
    head: Option<NonNull<Hole>>,
}

pub struct Hole {
    size: usize,
    next: Option<NonNull<Hole>>,
}

impl IntrusiveLinkedListAllocator {
    pub const fn new() -> Self {
        Self {
            head: None,
        }
    }

    pub unsafe fn insert_hole(&mut self, address: usize, size: usize) {
        assert_ne!(address, 0);
        assert!(size > core::mem::size_of::<Hole>());
        let mut new_hole_ptr = NonNull::new(address as *mut Hole).unwrap();
        match self.head {
            Some(head) => {
                // If the new hole is before the current head, the new hole becomes the head
                if address < head.as_ptr() as usize {
                    assert!(address + size <= head.as_ptr() as usize);
                    new_hole_ptr.as_uninit_mut().write(Hole {
                        size,
                        next: Some(head),
                    });
                    self.head = Some(new_hole_ptr);
                    return;
                }

                // Find a hole which starts after the new one, and insert the new one before it
                let mut current = head;
                while let Some(next) = current.as_ref().next {
                    let next_addr = next.as_ptr() as usize;
                    if next_addr > address {
                        assert!(current.as_ptr() as usize + current.as_ref().size <= address);
                        assert!(address + size <= next_addr);
                        new_hole_ptr.as_uninit_mut().write(Hole {
                            size,
                            next: Some(next),
                        });
                        current.as_mut().next = Some(new_hole_ptr);
                        return;
                    }
                    current = next;
                }

                // The loop has finished; the new hole is past the end of all the existing ones
                assert!(current.as_ptr() as usize + current.as_ref().size <= address);
                new_hole_ptr.as_uninit_mut().write(Hole {
                    size,
                    next: None,
                });
                current.as_mut().next = Some(new_hole_ptr);
            },
            None => {
                new_hole_ptr.as_uninit_mut().write(Hole {
                    size,
                    next: None,
                });
                self.head = Some(new_hole_ptr);
            },
        }
    }

    pub fn alloc(&mut self, size: usize, align: usize) -> Option<NonNull<()>> {
        let mut current = self.head.unwrap();
        while let Some(mut next) = unsafe { current.as_ref() }.next {
            let next_addr = next.as_ptr() as usize;
            let aligned = (next_addr + align - 1) / align * align;
            let difference = aligned - next_addr;
            if unsafe { next.as_ref() }.size - difference >= size {
                unsafe {
                    let old_size = next.as_ref().size;
                    let old_next = next.as_ref().next;
                    let mut new_hole_ptr = NonNull::new((aligned + size) as *mut Hole).unwrap();
                    new_hole_ptr.as_uninit_mut().write(Hole {
                        size: old_size - difference - size,
                        next: old_next,
                    });
                    // Determine whether to keep the hole `next` or remove it entirely
                    // Note that if `0 < difference < size_of::<Hole>()` then we are losing memory
                    // to the void, but it's only a couple of bytes so I don't care
                    if difference > core::mem::size_of::<Hole>() {
                        next.as_mut().size = difference;
                        next.as_mut().next = Some(new_hole_ptr);
                    } else {
                        current.as_mut().next = Some(new_hole_ptr);
                    }
                    return Some(NonNull::new(aligned as *mut _).unwrap());
                }
            }
            break;
        }
        None
    }
}
*/
