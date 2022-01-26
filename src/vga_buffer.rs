use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[allow(dead_code)]
enum Color {
    Black = 0,
    Blue,
    Green,
    Cyan,
    Red,
    Magenta,
    Brown,
    LightGray,
    DarkGray,
    LightBlue,
    LightGreen,
    LightCyan,
    LightRed,
    Pink,
    Yellow,
    White,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(fg: Color, bg: Color) -> Self {
        Self(((bg as u8) << 4) | (fg as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ch: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_pos: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_pos >= BUFFER_WIDTH {
                    self.new_line();
                }
                let r = BUFFER_HEIGHT - 1;
                let c = self.column_pos;
                self.buffer.chars[r][c].write(ScreenChar {
                    ch: byte,
                    color_code: self.color_code,
                });
                self.column_pos += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        for r in 1..BUFFER_HEIGHT {
            for c in 0..BUFFER_WIDTH {
                let ch = self.buffer.chars[r][c].read();
                self.buffer.chars[r - 1][c].write(ch);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_pos = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let whitespace = ScreenChar {
            ch: b' ',
            color_code: self.color_code,
        };
        for i in 0..BUFFER_WIDTH {
            self.buffer.chars[row][i].write(whitespace);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/*
 *pub fn print() {
 *    use core::fmt::Write;
 *    let mut writer = Writer {
 *        column_pos: 0,
 *        color_code: ColorCode::new(Color::Yellow, Color::Black),
 *        buffer: unsafe {&mut *(0xb8000 as *mut Buffer)}
 *    };
 *
 *    writer.write_byte(b'H');
 *    writer.write_string("ello ");
 *    // writer.write_string("wörld!");
 *    write!(writer, "The number are {} and {}", 42, 1.0 / 3.0).unwrap();
 *}
 */

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_pos: 0,
        color_code: ColorCode::new(Color::Cyan, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) }
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[test_case]
fn test_println_output() {
    let s = "Some test string that fits on a single line";
    println!("{}", s);
    for (i, ch) in s.chars().enumerate() {
        let temp = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(temp.ch), ch);
    }
}
