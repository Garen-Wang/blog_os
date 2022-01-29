#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use blog_os::println;
use x86_64::registers::control::Cr3;

#[no_mangle]
extern "C" fn _start() -> ! {
    println!("Hello world{}", "!");

    blog_os::init();

    /*
     *unsafe {
     *    let x = *(0x205354 as *mut u32);
     *    println!("read worked: {}", x);
     *    *(0x205354 as *mut u32) = 42;
     *    println!("write worked: {}", *(0x205354 as *mut u32));
     *}
     */
    let (level_4_page_table, _) = Cr3::read();
    println!("Level 4 page table at :{:#?}", level_4_page_table.start_address());

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    blog_os::hlt_loop();
}

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
