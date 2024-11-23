use core::{alloc, arch::asm, cmp, error::Error, ptr::write_volatile};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGrey = 7,
    DarkGrey = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    LightBrown = 14,
    White = 15,
}

impl Color {
    pub fn to_foreground(self) -> u8 {
        self as u8
    }

    pub fn to_background(self) -> u8 {
        (self as u8) << 4
    }
}

pub const VGA_WIDTH: u8 = 80;
pub const VGA_HEIGHT: u8 = 25;

#[cfg(test)]
static mut VGA_BUFFER_ADDR: [u16; VGA_WIDTH as usize * VGA_HEIGHT as usize] = [0; VGA_WIDTH as usize * VGA_HEIGHT as usize];

#[cfg(not(test))]
const VGA_BUFFER_ADDR: *mut u16 = 0xB8000 as *mut u16;

#[cfg(test)]
fn get_vga_buffer_ptr() -> *mut u16 {
    unsafe { VGA_BUFFER_ADDR.as_mut_ptr() }
}

pub struct OutOfBoundsError;

#[derive(Clone, Copy)]
pub struct Vga {
    color: u8,
    x: u8,
    y: u8,
    cursor: Cursor,
}

/// Abstraction for the ugliness behind updating the cursor.
///
/// https://wiki.osdev.org/Text_Mode_Cursor
#[derive(Clone, Copy)]
pub struct Cursor {}

impl Cursor {
    /// Updates the text-mode cursor to position `x, y` in the VGA buffer.
    /// ## SAFETY
    /// This writes to kernel-managed memory, running this in a non-bare-metal environment
    /// will result in invalid memory access.
    pub unsafe fn update(&self, x: usize, y: usize) {
        let out_of_bounds: bool = !(0..VGA_HEIGHT).contains(&(y as u8)) || !(0..VGA_WIDTH).contains(&(x as u8));
        if out_of_bounds {
            return;
        }

        let pos = y * VGA_WIDTH as usize + x;

        asm!(
            "mov dx, 0x3D4",
            "mov al, 0x0F",
            "out dx, al",
            "inc dx",
            "mov al, {low}",
            "out dx, al",
            "mov dx, 0x3D4",
            "mov al, 0x0E",
            "out dx, al",
            "inc dx",
            "mov al, {high}",
            "out dx, al",
            low = in(reg_byte) (pos & 0xFF) as u8,
            high = in(reg_byte) ((pos >> 8) & 0xFF) as u8,
            out("dx") _,
            out("al") _,
        );
    }
}

impl Default for Vga {
    fn default() -> Self {
        Self::new()
    }
}

impl Vga {
    pub fn new() -> Self {
        let mut t = Vga {
            color: 0,
            x: 0,
            y: 0,
            cursor: Cursor {},
        };

        t.set_foreground_color(Color::White);
        t.set_background_color(Color::Black);

        #[cfg(not(test))]
        unsafe {
            t.cursor.update(0, 0);
        }

        t
    }

    fn get_buffer_addr(&self) -> *mut u16 {
        #[cfg(test)]
        unsafe {
            get_vga_buffer_ptr()
        }

        #[cfg(not(test))]
        {
            VGA_BUFFER_ADDR
        }
    }

    pub fn write_char(&mut self, c: u8) {
        let _ = self.write_char_at(self.y, self.x, c);
        self.inc_cursor();
    }

    pub fn delete_char(&mut self) {
        self.dec_cursor();
        self.write_char_at(self.y, self.x, 0);
    }

    fn write_char_at(&self, row: u8, column: u8, character: u8) -> Result<(), OutOfBoundsError> {
        if row >= VGA_HEIGHT || column >= VGA_WIDTH {
            return Err(OutOfBoundsError);
        }
        let entry: u16 = (character as u16) | (self.color as u16) << 8;
        let index: isize = row as isize * VGA_WIDTH as isize + column as isize;

        // Safety:
        // The if statement above ensures that the index never goes out of the dedicated memory for the chars on screen.
        unsafe {
            write_volatile(self.get_buffer_addr().offset(index), entry);
        }
        Ok(())
    }

    pub fn write_u8_arr(&mut self, s: &[u8]) {
        for c in s.iter() {
            if *c == 0 {
                return;
            }
            self.write_char(*c);
        }
    }

    pub fn clear_screen(&mut self) {
        for row in 0..VGA_HEIGHT {
            for col in 0..VGA_WIDTH {
                self.write_char_at(row, col, 0);
            }
        }
    }

    fn dec_cursor(&mut self) {
        let on_first_col = self.x == 0;
        let on_first_row = self.y == 0;

        if on_first_col && on_first_row {
            return;
        } else if on_first_col {
            self.y = cmp::max(self.y - 1, 0);
            self.x = self.get_row_pos_for_col(self.y as usize) as u8;
        } else {
            self.x -= 1;
        }

        #[cfg(not(test))]
        unsafe {
            self.cursor.update(self.x as usize, self.y as usize);
        }
    }

    fn inc_cursor(&mut self) {
        self.x += 1;
        if self.x >= VGA_WIDTH {
            self.x = 0;
            self.y += 1;
        }
        if self.y >= VGA_HEIGHT {
            self.y = 0;
        }

        #[cfg(not(test))]
        unsafe {
            self.cursor.update(self.x as usize, self.y as usize);
        }
    }

    /// Gets the position of the last written character for `row`, to ensure the cursor returns
    /// to the correct position when backspacing at `col == 0`.
    fn get_row_pos_for_col(&self, row: usize) -> usize {
        if !(0..VGA_HEIGHT).contains(&(row as u8)) {
            return 0;
        }

        let mut pos: isize = row as isize * VGA_WIDTH as isize + (VGA_WIDTH as isize - 1);

        // Safety:
        // The above check ensures we stay within the bounds of the VGA buffer row-wise. Then,
        // the loop condition ensures we never read outside the bounds of the current row.
        unsafe {
            while pos >= row as isize * VGA_WIDTH as isize {
                if (*self.get_buffer_addr().offset(pos) & 0xFF) != 0 {
                    return pos as usize % VGA_WIDTH as usize;
                }
                pos -= 1;
            }
        }

        0
    }

    pub fn new_line(&mut self) {
        let current_row = self.y;
        while current_row == self.y {
            self.inc_cursor();
        }

        #[cfg(not(test))]
        unsafe {
            self.cursor.update(self.x as usize, self.y as usize);
        }
    }

    pub fn set_foreground_color(&mut self, foreground: Color) {
        self.color &= 0xF0;
        self.color |= foreground.to_foreground();
    }

    pub fn set_background_color(&mut self, background: Color) {
        self.color &= 0x0F;
        self.color |= background.to_background();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new_vga() {
        let v = Vga::new();

        assert!(v.x == 0);
        assert!(v.y == 0);

        let expected_color = Color::Black.to_background() | Color::White.to_foreground();
        assert!(v.color == expected_color);
    }
}
