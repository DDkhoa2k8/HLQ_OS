global long_mode_start
extern rust_main

section .text
bits 64
long_mode_start:
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    ; print `LONG BOOT` to screen
    ;'LONG'
    mov rax, 0x2f472f4e2f4f2f4c
    mov qword [0xb80a0], rax
    ;Space
    mov qword [0x80a8], 0x2f20
    ;'BOOT'
    mov rax, 0x2f542f4f2f4f2f42
    mov qword [0xb80aa], rax

    call rust_main
    hlt