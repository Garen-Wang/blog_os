use core::{alloc::{Layout, GlobalAlloc}, ptr::{null_mut, NonNull}, mem::{size_of, align_of}};

use super::bump::Locked;


struct Node {
    next: Option<&'static mut Node>
}

const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

pub struct FixedSizeBlockAllocator {
    heads: [Option<&'static mut Node>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap
}

impl FixedSizeBlockAllocator {
    pub const fn new() -> Self {
        const NONE: Option<&'static mut Node> = None;
        FixedSizeBlockAllocator {
            heads: [NONE; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty()
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback_allocator.init(heap_start, heap_size);
    }

    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => null_mut()
        }
    }
}

fn get_list_index(layout: &Layout) -> Option<usize> {
    let size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&x| x >= size)
}

unsafe impl GlobalAlloc for Locked<FixedSizeBlockAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();
        match get_list_index(&layout) {
            Some(idx) => {
                match allocator.heads[idx].take() {
                    Some(node) => {
                        allocator.heads[idx] = node.next.take();
                        node as *mut Node as *mut u8
                    }
                    None => {
                        let layout = Layout::from_size_align(
                            BLOCK_SIZES[idx], BLOCK_SIZES[idx]
                        ).unwrap();
                        allocator.fallback_alloc(layout)
                    }
                }
            }
            None => {
                allocator.fallback_alloc(layout)
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();
        match get_list_index(&layout) {
            Some(idx) => {
                let new_node = Node {
                    next: allocator.heads[idx].take()
                };
                assert!(size_of::<Node>() <= BLOCK_SIZES[idx]);
                assert!(align_of::<Node>() <= BLOCK_SIZES[idx]);
                let new_node_ptr = ptr as *mut Node;
                new_node_ptr.write(new_node);
                allocator.heads[idx] = Some(&mut *new_node_ptr);
            }
            None => {
                allocator.fallback_allocator.deallocate(
                    NonNull::new(ptr).unwrap(), layout
                );
            }
        }
    }
}
