#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use blog_os::{println, memory::{create_example_mapping, BootInfoFrameAllocator}};
use bootloader::{BootInfo, entry_point};
use x86_64::{VirtAddr, structures::paging::Page};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello world{}", "!");

    blog_os::init();

    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { blog_os::memory::init(physical_memory_offset) };
    let mut frame_allocator = BootInfoFrameAllocator::init(&boot_info.memory_map);

    let page = Page::containing_address(VirtAddr::new(0));
    create_example_mapping(page, &mut mapper, &mut frame_allocator);

    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe {
        page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e);
    }

    /*
     *let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
     *let mapper = unsafe { blog_os::memory::init(physical_memory_offset) };
     *let addrs = [
     *    0xb8000,
     *    0x201008,
     *    0x0100_0020_1a10,
     *    boot_info.physical_memory_offset
     *];
     *for addr in addrs {
     *    let virtual_addr = VirtAddr::new(addr);
     *    //let physical_addr = unsafe {translate_addr(virtual_addr, physical_memory_offset)};
     *    let physical_addr = mapper.translate_addr(virtual_addr);
     *    println!("{:?} -> {:?}", virtual_addr, physical_addr);
     *}
     */

    /*
     *let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
     *let l4_table = unsafe {
     *    active_level_4_table(physical_memory_offset)
     *};
     *for (i, entry) in l4_table.iter().enumerate() {
     *    if !entry.is_unused() {
     *        println!("L4 entry {}: {:#?}", i, entry);
     *        let physical_addr = entry.frame().unwrap().start_address();
     *        let virtual_addr = physical_memory_offset + physical_addr.as_u64();
     *        let l3_table = unsafe {
     *            let l3_table_ptr: *mut PageTable = virtual_addr.as_mut_ptr();
     *            &*l3_table_ptr
     *        };
     *        
     *        for (i, entry) in l3_table.iter().enumerate() {
     *            if !entry.is_unused() {
     *                println!("L3 entry {}: {:#?}", i, entry);
     *            }
     *        }
     *    }
     *}
     */


    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    blog_os::hlt_loop();
}

/*
 *#[no_mangle]
 *extern "C" fn _start() -> ! {
 *
 *}
 */

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    blog_os::hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(_info)
}

#[test_case]
fn simple_unit_test() {
    assert_eq!(1, 1);
}
