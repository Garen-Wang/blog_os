use core::{alloc::GlobalAlloc, ptr::null_mut};
use spin;

use super::align_up;

pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    n_allocations: usize
}

pub struct Locked<T> {
    inner: spin::Mutex<T>
}

impl<T> Locked<T> {
    pub const fn new(inner: T) -> Self {
        Locked { inner: spin::Mutex::new(inner) }
    }
    
    pub fn lock(&self) -> spin::MutexGuard<T> {
        self.inner.lock()
    }
}

impl BumpAllocator {
    pub const fn new() -> Self {
        BumpAllocator { heap_start: 0, heap_end: 0, next: 0, n_allocations: 0 }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut allocator = self.lock();
        let alloc_start = align_up(allocator.next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(alloc_end) => alloc_end,
            None => return null_mut()
        };
        if alloc_end > allocator.heap_end {
            null_mut()
        } else {
            allocator.next = alloc_end;
            allocator.n_allocations += 1;
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        let mut allocator = self.lock();
        allocator.n_allocations -= 1;
        if allocator.n_allocations == 0 {
            allocator.next = allocator.heap_start;
        }
    }
}
