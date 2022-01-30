use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{registers::control::Cr3, VirtAddr, structures::paging::{PageTable, page_table::FrameError, OffsetPageTable, PhysFrame, PageTableFlags, Mapper, Page, FrameAllocator, Size4KiB}, PhysAddr};

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let l4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(l4_table, physical_memory_offset)
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)
    -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();
    let physical_addr = level_4_table_frame.start_address();
    let virtual_addr = physical_memory_offset + physical_addr.as_u64();
    let page_table_ptr: *mut PageTable = virtual_addr.as_mut_ptr();
    &mut *page_table_ptr
}

pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>
) {
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    let map_to_result = unsafe {
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}

pub unsafe fn translate_addr(
    virtual_addr: VirtAddr, physical_memory_offset: VirtAddr
) -> Option<PhysAddr> {
    translate_addr_inner(virtual_addr, physical_memory_offset)
}

fn translate_addr_inner(
    virtual_addr: VirtAddr, physical_memory_offset: VirtAddr
) -> Option<PhysAddr> {
    let (level_4_table_frame, _) = Cr3::read();
    let table_idxs = [
        virtual_addr.p4_index(),
        virtual_addr.p3_index(),
        virtual_addr.p2_index(),
        virtual_addr.p1_index()
    ];
    let mut frame = level_4_table_frame;
    
    for idx in table_idxs {
        let virtual_addr = physical_memory_offset + frame.start_address().as_u64();
        let page_table_ptr: *const PageTable = virtual_addr.as_ptr();
        let page_table = unsafe {
            &*page_table_ptr
        };
        let entry = &page_table[idx];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
            Err(FrameError::FrameNotPresent) => return None,
        }
    }
    Some(frame.start_address() + u64::from(virtual_addr.page_offset()))
}

pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        None
    }
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize
}

impl BootInfoFrameAllocator {
    pub fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator { memory_map, next: 0 }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addrs = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addrs.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
