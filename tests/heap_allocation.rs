#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use alloc::{boxed::Box, vec::Vec};
use blog_os::{memory, allocator::{self, HEAP_SIZE}};
use bootloader::{entry_point, BootInfo};
use x86_64::VirtAddr;

extern crate alloc;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    blog_os::init();

    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(physical_memory_offset) };
    let mut frame_allocator = memory::BootInfoFrameAllocator::init(&boot_info.memory_map);
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

#[test_case]
fn simple_allocation() {
    let heap_value1 = Box::new(114);
    let heap_value2 = Box::new(514);
    assert_eq!(114, *heap_value1);
    assert_eq!(514, *heap_value2);
}

#[test_case]
fn large_vec() {
    let mut vec: Vec<i32> = Vec::new();
    for i in 0..1000 {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<i32>(), (1000 - 1) * 1000 / 2);
}

#[test_case]
fn many_boxes() {
    let long_lived_value = Box::new(1);
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(i, *x);
    }
    assert_eq!(*long_lived_value, 1);
}
