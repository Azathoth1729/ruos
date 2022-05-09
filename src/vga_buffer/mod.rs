//! The [VGA Text Mode](https://www.wikiwand.com/en/VGA-compatible_text_mode) is a simple way to print text to the screen.
//!
//! To print a character to the screen in VGA text mode, one has to write it to the text buffer of the VGA hardware.
//!
//! The VGA text buffer is a two-dimensional array with typically 25 rows and 80 columns, which is directly rendered to the screen.
//! Each array entry describes a single screen character through the following format:
//!
//! | Bit(s)   |         Value        |
//! | :---     |         :----        |     
//! | 0-7      |   ASCII code point   |
//! | 8-11	   |   Foreground color   |
//! | 12-14		 |   Background color   |
//! | 15  		 |   Blink              |

use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

use color::{Color, ColorCode};
use core::fmt;

mod color;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

/// A char printed on screen with certain color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

/// VGA Text Buffer
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// Text Writer
pub struct Writer {
    /// Current position in the last row
    column_position: usize,
    /// The current colorcode(foreground and background colors)
    color_code: ColorCode,
    /// VGA Text Buffer
    buffer: &'static mut Buffer,
}

impl Writer {
    /// Write a btye to buffer
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                self.buffer.chars[BUFFER_HEIGHT - 1][self.column_position].write(ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                });

                self.column_position += 1;
            }
        }
    }

    /// Write a string to buffer
    pub fn write_string(&mut self, s: &str) {
        for btye in s.bytes() {
            match btye {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(btye),
                _ => self.write_byte(0xfe),
            }
        }
    }

    /// Shift line up and reset column_position to zero
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let char = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(char);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    /// Clear the character at cartain row
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[allow(dead_code)]
#[doc(hidden)]
/// Test function
pub fn test_print() {
    use core::fmt::Write;

    WRITER.lock().write_byte(b'H');
    WRITER.lock().write_string("ello");
    WRITER.lock().write_string("Wörld!\n");
    write!(WRITER.lock(), "The numbers are {} and {}", 42, 1.0 / 3.0).unwrap();
}
