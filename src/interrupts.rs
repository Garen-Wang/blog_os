use pc_keyboard::{Keyboard, layouts::Us104Key, ScancodeSet1, HandleControl, DecodedKey};
use x86_64::{structures::idt, instructions::port::Port};
use crate::{print, println, gdt};
use lazy_static::lazy_static;
use spin;
use pic8259;

lazy_static! {
    static ref IDT: idt::InterruptDescriptorTable = {
        let mut idt = idt::InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<pic8259::ChainedPics> = spin::Mutex::new(
    unsafe { pic8259::ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) }
);

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: idt::InterruptStackFrame
) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: idt::InterruptStackFrame,
    _error_code: u64
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: idt::InterruptStackFrame
) {
    print!(".");
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: idt::InterruptStackFrame
) {
    lazy_static! {
        static ref KEYBOARD: spin::Mutex<Keyboard<Us104Key, ScancodeSet1>> = 
            spin::Mutex::new(Keyboard::new(Us104Key, ScancodeSet1, 
                HandleControl::Ignore)
        );
    }
    let mut keyboard = KEYBOARD.lock();
    let mut port: Port<u8> = Port::new(0x60);
    let scan_code = unsafe { port.read() };

    /*
     *let key = match scan_code {
     *    x if (2 <= x && x <= 11) => {
     *        Some(if x == 11 { '0' } else { char::from(x - 1 + 0x30) })
     *    },
     *    _ => None
     *};
     *if let Some(key) = key {
     *    print!("{}", key);
     *}
     */

    if let Ok(Some(key_event)) = keyboard.add_byte(scan_code) {
        if let Some(decoded_key) = keyboard.process_keyevent(key_event) {
            match decoded_key {
                DecodedKey::Unicode(ch) => print!("{}", ch),
                DecodedKey::RawKey(key) => print!("{:#?}", key)
            }
        }
    }
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
