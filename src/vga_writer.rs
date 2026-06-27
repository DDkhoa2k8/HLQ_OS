const COL_SIZE:u8 = 80;
const ROW_SIZE:u8 = 25;

pub struct VGAWriter {
    pub vga_addr: *mut u8,
    pub line_char_o: u8,//number of char per line
    pub line_o: u8,//number of line
    pub color: u8,
}

impl VGAWriter {
    pub fn init() -> VGAWriter {
        VGAWriter {
            vga_addr: 0x8000 as *mut u8,
            line_char_o: 0,
            line_o: 0,
            color: 0x002f,//Light green
        }
    }

    pub fn set_color(&mut self, color:u8) {
        self.color = color;
    }

    pub fn print_char(&mut self, c:u8) {
        self.line_char_o += 1;

        let offset = COL_SIZE * self.line_o * 2 + self.line_char_o;

        unsafe {
            *self.vga_addr.offset(offset.into()) = c;
            *self.vga_addr.offset((offset + 1).into()) = self.color;
        }
    }

    pub fn print(&mut self, content: &str) {
        for s in content.bytes() {
            self.print_char(s);
        }
    }
}