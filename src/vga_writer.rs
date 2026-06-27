const COL_SIZE:isize = 80;
const ROW_SIZE:isize = 25;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VGAOutColor {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

pub struct VGAWriter {
    pub vga_addr: *mut u8,
    pub line_char_o: isize,//index of currect column
    pub line_o: isize,//index of current row (line)
    pub color: u8,
}

impl VGAWriter {
    ///### create new instance of VGAWriter
    /// 
    /// ### Example:
    /// ```no_run
    /// pub mod vga_writer;
    /// 
    /// let mut vga = vga_writer::VGAWriter::init();
    /// 
    /// vga.print("Hello World");
    /// //Expected output:
    /// //Hello World
    /// ```
    pub fn init() -> VGAWriter {
        VGAWriter {
            vga_addr: 0xb8000 as *mut u8,
            line_char_o: 0,
            line_o: 0,
            color: 0x2f,//Light green
        }
    }

    ///### Set text color
    /// 
    ///### 8 bit input
    /// 
    /// first 4 bit define text color (| Bright | Red | Green | Blue |)
    /// 
    /// second 4 bit define background color (| Bright | Red | Green | Blue |)
    /// 
    /// ### Example:
    /// ```no_run
    /// pub mod vga_writer;
    /// 
    /// let mut vga = vga_writer::VGAWriter::init();
    /// 
    /// //0x2f -> 0010(second 4 is green) 1111(first 4 bit mean white)
    /// vga.set_color(0x2f);//Set background color green, white text
    /// ```
    pub fn set_color_hex(&mut self, color:u8) {
        self.color = color;
    }

    ///### Set text color
    /// ### Example:
    /// ```no_run
    /// pub mod vga_writer;
    /// 
    /// let mut vga = vga_writer::VGAWriter::init();
    /// 
    /// //0x2f -> 0010(second 4 is green) 1111(first 4 bit mean white)
    /// vga.set_color(VGAOutColor::White, VGAOutColor::Green);//Set background color green, white text
    /// ```
    pub fn set_color(&mut self, text_color: VGAOutColor, background_color: VGAOutColor) {
        self.color = ((background_color as u8) << 4) | text_color as u8;

        //how it work:
        //0000(4 bit bg color) << 4 = (4 bit bg color)0000 //Bit shift
        // (4 bit bg color)0000 | 0000(4 bit text color) = (4 bit bg color)(4 bit text color) //OR operation act like merge
    }

    /// ### Break line
    /// 
    /// ### Example:
    /// ```no_run
    /// pub mod vga_writer;
    /// 
    /// let mut vga = vga_writer::VGAWriter::init();
    /// 
    /// vga.print_char("A");
    /// 
    /// vga.new_line();
    /// 
    /// vga.print_char("A");
    /// 
    /// //Expected output:
    /// //A
    /// //A
    /// ```
    pub fn new_line(&mut self) {
        self.line_o += 1;//add new line
    }

    /// ### Print a single character
    /// 
    /// ### Example:
    /// ```no_run
    /// pub mod vga_writer;
    /// 
    /// let mut vga = vga_writer::VGAWriter::init();
    /// 
    /// vga.print_char("A");
    /// //Expected output:
    /// //A
    /// ```
    pub fn print_char(&mut self, c:u8) {
        let offset = (COL_SIZE * self.line_o + self.line_char_o) * 2;//Mapping to memory address offset
        
        unsafe {
            *self.vga_addr.offset(offset) = c;
            *self.vga_addr.offset(offset + 1) = self.color;
        }

        self.line_char_o += 1;
    }

    /// ### Print a complete string
    /// 
    /// ### Example:
    /// ```no_run
    /// pub mod vga_writer;
    /// 
    /// let mut vga = vga_writer::VGAWriter::init();
    /// 
    /// vga.print("Hello World");
    /// //Expected output:
    /// //Hello World
    /// ```
    pub fn print(&mut self, content: &str) {
        for s in content.bytes() {
            match s {
                b'\n' => self.new_line(),
                s => self.print_char(s),
            }
        }
    }

    /// ### Print a complete string
    /// 
    /// ### Example:
    /// ```no_run
    /// pub mod vga_writer;
    /// 
    /// let mut vga = vga_writer::VGAWriter::init();
    /// 
    /// vga.println("Hello");
    /// vga.println("World");
    /// //Expected output:
    /// //Hello 
    /// //World
    /// ```
    pub fn println(&mut self, content: &str) {
        self.print(content);

        self.new_line();
    }
}