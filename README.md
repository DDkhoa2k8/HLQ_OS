# HLQ-OS (x86_64 Bare-Metal Kernel)

A custom 64-bit operating system kernel built using Rust and Assembly.

---

## Prerequisites

This project requires a native **Linux (Ubuntu)** environment or **Windows Subsystem for Linux (WSL)**.

### 1. Install System Dependencies

Install the required compilation tools, assembler, ISO creation utilities, and the QEMU emulator:

```bash
sudo apt update
sudo apt install -y \
    build-essential \
    nasm \
    binutils \
    qemu-system-x86_64 \
    grub-common \
    grub-pc-bin \
    xorriso
```

## How to Build & Run

The build pipeline uses a dynamic `Makefile` that manages both Assembly compilation and Rust compilation under the hood. It supports two modes: **Debug** (for inspecting variables) and **Release** (optimized for low-level execution).

### 1. Default Debug Mode

By default, the Makefile compiles everything in debug mode. This retains all code symbols, making it ideal if you plan to attach a debugger later.

```bash
# Build the kernel and bundle it into a bootable ISO image
make iso

# Build, build the ISO, and immediately boot it up in QEMU
make run

# Boot in QEMU with CPU interrupt logging active (great for catching triple faults)
make run_log
```

### 2. Release mode

Bare-metal environments have tiny initial stacks. To prevent unoptimized code from causing stack overflows or breaking hardware timing loops, switch to release mode by prepending MODE=release

```bash
# Build the optimized release kernel and ISO
MODE=release make iso

# Build and run the optimized version in QEMU
MODE=release make run

# Run the optimized version with CPU interrupt logging active
MODE=release make run_log
```

### 3. Cleaning the Build Environment

```bash
make clean
```
