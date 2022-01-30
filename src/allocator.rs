pub mod bump;
pub mod linked_list;
pub mod fixed_size_block;

use core::{alloc::{GlobalAlloc, Layout}, ptr::null_mut};
//use linked_list_allocator::LockedHeap;
use x86_64::{VirtAddr, structures::paging::{Page, Size4KiB, FrameAllocator, mapper::MapToError, PageTableFlags, Mapper}};

use self::{bump::Locked, fixed_size_block::FixedSizeBlockAllocator};

#[global_allocator]
//static ALLOCATOR: Dummy = Dummy;
//static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());
//static ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(FixedSizeBlockAllocator::new());

pub struct Dummy;

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        panic!("dealloc should never be called");
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error: {:#?}", layout)
}

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024;

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let heap_start = VirtAddr::new(HEAP_START as u64);
    let heap_end = heap_start + HEAP_SIZE - 1u64;
    let heap_start_page: Page<Size4KiB> = Page::containing_address(heap_start);
    let heap_end_page: Page<Size4KiB> = Page::containing_address(heap_end);
    let heap_page_range = Page::range_inclusive(heap_start_page, heap_end_page);
    for page in heap_page_range {
        let frame = frame_allocator.allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }
    Ok(())
}

fn align_up(addr: usize, align: usize) -> usize {
    /*
     *let remainder = addr % align;
     *if remainder == 0 { addr } else { addr - remainder + align }
     */
    (addr + align - 1) & !(align - 1)
}
