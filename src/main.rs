#![no_std]
#![no_main]

mod vga_buffer;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    loop {}
}

#[no_mangle]
extern "C" fn _start() -> ! {
    // vga_buffer::print();

    // use core::fmt::Write;
    // vga_buffer::WRITER.lock().write_str("Hello again").unwrap();
    // write!(vga_buffer::WRITER.lock(), ", some numbers: {}, {}", 42, 1.0 / 3.0).unwrap();

    // println!("Hello world{}", "!");
    // loop {}

    panic!("Some panic message...");
}
