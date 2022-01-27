#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use blog_os::println;

#[no_mangle]
extern "C" fn _start() -> ! {
    println!("Hello world{}", "!");

    blog_os::init();

    /*
     *unsafe {
     *    *(0xdeadbeef as *mut u64) = 42;
     *}
     */
    
    #[allow(unconditional_recursion)]
    fn stack_overflow() {
        stack_overflow();
    }

    stack_overflow();

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
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
