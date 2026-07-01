use core::ptr::addr_of;
use crate::vga_println;
use core::arch::{asm,global_asm};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ExceptionStackFrame {
    // Pushed by our assembly stub
    pub r15: u64, pub r14: u64, pub r13: u64, pub r12: u64,
    pub r11: u64, pub r10: u64, pub r9:  u64, pub r8:  u64,
    pub rbp: u64, pub rdi: u64, pub rsi: u64, pub rdx: u64,
    pub rcx: u64, pub rbx: u64, pub rax: u64,
    
    // Pushed automatically or manually handled for error codes
    pub error_code: u64,
    
    // Automatically pushed by x86_64 CPU hardware
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}


global_asm!(
    r#"
    .macro exception_stub_no_error name, handler
    .global \name
    \name:
        push 0                    # Dummy error code
        push rax; push rbx; push rcx; push rdx; push rsi; push rdi; push rbp
        push r8;  push r9;  push r10; push r11; push r12; push r13; push r14; push r15
        mov rdi, rsp              # Pass pointer to stack frame as 1st argument (RDI)
        call \handler
        pop r15;  pop r14;  pop r13;  pop r12;  pop r11;  pop r10;  pop r9;  pop r8
        pop rbp;  pop rdi;  pop rsi;  pop rdx;  pop rcx;  pop rbx;  pop rax
        add rsp, 8                # Clean up dummy error code
        iretq                     # Return from interrupt
    .endm

    .macro exception_stub_with_error name, handler
    .global \name
    \name:
        # Error code is already pushed by CPU here
        push rax; push rbx; push rcx; push rdx; push rsi; push rdi; push rbp
        push r8;  push r9;  push r10; push r11; push r12; push r13; push r14; push r15
        mov rdi, rsp              # Pass pointer to stack frame as 1st argument (RDI)
        call \handler
        pop r15;  pop r14;  pop r13;  pop r12;  pop r11;  pop r10;  pop r9;  pop r8
        pop rbp;  pop rdi;  pop rsi;  pop rdx;  pop rcx;  pop rbx;  pop rax
        add rsp, 8                # Clean up CPU error code
        iretq                     # Return from interrupt
    .endm

    # Link the raw ASM stubs to our clean Rust handlers
    exception_stub_no_error   asm_handler_de, rust_handler_de
    exception_stub_no_error   asm_handler_bp, rust_handler_bp
    exception_stub_with_error asm_handler_gp, rust_handler_gp
    exception_stub_with_error asm_handler_pf, rust_handler_pf
    "#
);

// Declare the assembly stubs so we can pass them to our IDT
unsafe extern "C" {
    fn asm_handler_de();
    fn asm_handler_bp();
    fn asm_handler_gp();
    fn asm_handler_pf();
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_handler_de(frame: &ExceptionStackFrame) {
    vga_println!("--- EXCEPTION: DIVIDE BY ZERO (#DE) ---");
    vga_println!("Instruction Pointer (RIP): 0x{}", frame.rip);
    vga_println!("Stack Pointer (RSP):       0x{}", frame.rsp);
    vga_println!("RAX: 0x{} | RBX: 0x{}", frame.rax, frame.rbx);
    loop {} // Diverge since we can't safely resume a divide-by-zero easily
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_handler_bp(frame: &ExceptionStackFrame) {
    vga_println!("--- BREAKPOINT (#BP) ---");
    vga_println!("Resuming execution after RIP: 0x{}", frame.rip);
    // Breakpoints are traps, so we can return normally! The stub handles `iretq`.
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_handler_gp(frame: &ExceptionStackFrame) {
    vga_println!("--- GENERAL PROTECTION FAULT (#GP) ---");
    vga_println!("Error Code:                0x{}", frame.error_code);
    vga_println!("Failing Instruction (RIP): 0x{}", frame.rip);
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_handler_pf(frame: &ExceptionStackFrame) {
    let cr2: u64;
    unsafe {
        core::arch::asm!("mov {}, cr2", out(reg) cr2);
    }
    vga_println!("--- PAGE FAULT (#PF) ---");
    vga_println!("Faulting Memory Address:   0x{}", cr2); // CR2 contains the exact address that triggered the fault
    vga_println!("Error Code Bits:           0x{}", frame.error_code);
    vga_println!("Failing Instruction (RIP): 0x{}", frame.rip);
    loop {}
}

// ── Gate type + attribute byte ──────────────────────────────────────────────
const GATE_PRESENT:    u8 = 1 << 7;   // P bit
const GATE_DPL0:       u8 = 0 << 5;   // ring 0
const GATE_INTERRUPT:  u8 = 0xE;      // 64-bit interrupt gate
const GATE_TRAP:       u8 = 0xF;      // 64-bit trap gate

const KERNEL_CS: u16 = 0x08;          // GDT code segment selector

// ── 16-byte IDT entry ────────────────────────────────────────────────────────
#[derive(Clone, Copy)]
#[repr(C, packed)]
struct IdtEntry {
    offset_low:  u16,   // handler bits [15:0]
    selector:    u16,   // code segment selector
    ist:         u8,    // bits[2:0] = IST index, rest = 0
    type_attr:   u8,    // P | DPL | gate type
    offset_mid:  u16,   // handler bits [31:16]
    offset_high: u32,   // handler bits [63:32]
    _reserved:   u32,
}

impl IdtEntry {
    const fn missing() -> Self {
        IdtEntry {
            offset_low:  0,
            selector:    0,
            ist:         0,
            type_attr:   0,   // P=0 → not present
            offset_mid:  0,
            offset_high: 0,
            _reserved:   0,
        }
    }

    fn new(handler: unsafe extern "C" fn(), gate: u8) -> Self {
        let addr = handler as usize;
        IdtEntry {
            offset_low:  (addr & 0xFFFF) as u16,
            selector:    KERNEL_CS,
            ist:         0,
            type_attr:   GATE_PRESENT | GATE_DPL0 | gate,
            offset_mid:  ((addr >> 16) & 0xFFFF) as u16,
            offset_high: ((addr >> 32) & 0xFFFF_FFFF) as u32,
            _reserved:   0,
        }
    }

    fn new_trap(handler: unsafe extern "C" fn()) -> Self {
        Self::new(handler, GATE_TRAP)
    }

    fn new_interrupt(handler: unsafe extern "C" fn()) -> Self {
        Self::new(handler, GATE_INTERRUPT)
    }
}

// ── The table itself: 256 entries ────────────────────────────────────────────
#[repr(C, align(16))]
struct Idt([IdtEntry; 256]);

static mut IDT: Idt = Idt([IdtEntry::missing(); 256]);

// ── IDTR (what lidt actually receives) ───────────────────────────────────────
#[repr(C, packed)]
struct IdtDescriptor {
    limit: u16,
    base:  u64,
}

// ── Raw exception stubs (must be `extern "C"`, no Rust ABI mangling) ─────────
// unsafe extern "C" fn handler_de()  {  
//     vga_println!("#DE divide-by-zero    "); 
// }

// unsafe extern "C" fn handler_db()  {  
//     vga_println!("#DB debug             "); 
// }

// unsafe extern "C" fn handler_nmi() {  
//     vga_println!("NMI                   "); 
// }

// unsafe extern "C" fn handler_of()  {  
//     vga_println!("#OF overflow (trap)   "); 
// }

// unsafe extern "C" fn handler_gp()  {  
//     vga_println!("#GP general protection"); 
// }

// unsafe extern "C" fn handler_pf()  {  
//     vga_println!("#PF page fault        "); 
// }

// unsafe extern "C" fn handler_bp()  {
//     vga_println!("#BP break point");
// }

// ── Install entries and load ──────────────────────────────────────────────────
pub unsafe fn init_idt() {
    // CPU exceptions (vectors 0–31)
    unsafe {
        IDT.0[0]  = IdtEntry::new_interrupt(asm_handler_de); 
        IDT.0[3]  = IdtEntry::new_trap(asm_handler_bp);      
        IDT.0[13] = IdtEntry::new_interrupt(asm_handler_gp); 
        IDT.0[14] = IdtEntry::new_interrupt(asm_handler_pf);
    }

    let descriptor = IdtDescriptor {
        limit: (core::mem::size_of::<Idt>() - 1) as u16,
        base: unsafe {
                addr_of!(IDT.0) as u64
            },
        };

    unsafe {
        core::arch::asm!(
            "lidt [{}]",
            in(reg) &descriptor,
            options(readonly, nostack, preserves_flags)
        );
    }
}

pub unsafe fn trigger_breakpoint() {
    unsafe {
        asm!("int3");
    }
}

pub unsafe fn trigger_pagefault() {
    unsafe {
        asm!("mov eax, [0xffffffffffffffff]");
    }
}

pub unsafe fn trigger_de() {
    unsafe {
        asm!("mov rax, 100");//move 100 to rax register
        asm!("mov rbx, 0");//move 0 to rbx register
        asm!("div rbx");//divide rax by rbx => 100 / 0 => divide by 0
    }
}