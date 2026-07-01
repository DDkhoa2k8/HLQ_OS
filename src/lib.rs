#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::arch::*;
pub mod vga_writer;
pub mod interrupts;

/// This is OS entry point. 
/// `extern "C"` forces the compiler to use the standard C calling convention, 
/// and `#[no_mangle]` keeps the name exactly `rust_main` so assembly can find it.
#[unsafe(no_mangle)]
pub extern "C" fn rust_main(_mbi_ptr: usize) -> ! {
    vga_writer::GL_VGA_WT_REF.set_color(vga_writer::VGAOutColor::Green, vga_writer::VGAOutColor::Black);
        
    vga_writer::GL_VGA_WT_REF.line_o = 2;//start from line no.3 to avoid overlap with 2 previous line of the text print by boot.asm and long_mode.asm

    //Break point exception
    unsafe {
        interrupts::init_idt();

        asm!("int3");
    }

    //Test Divide by 0 exception
    // unsafe {
    //     asm!("mov rax, 100");//move 100 to rax register
    //     asm!("mov rbx, 0");//move 0 to rbx register
    //     asm!("div rbx");//divide rax by rbx => 100 / 0 => divide by 0
    // }
    
    let pi = 3.14;

    vga_println!("hello world! using vga buffer pi = {}", pi);

    loop {}
}

/// A panic handler is mandatory when working with #![no_std]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}