arch ?= x86_64
MODE ?= debug

# Adjust paths and flags based on the chosen mode
# Default is debug. Run with `MODE=release make` to switch.
remap_path = target/x86_64-unknown-none/$(MODE)/libhlq_os.a

ifeq ($(MODE), release)
    CARGO_FLAGS := --release
    rust_os := target/x86_64-unknown-none/release/libhlq_os.a
else
    CARGO_FLAGS :=
    rust_os := target/x86_64-unknown-none/debug/libhlq_os.a
endif

kernel := build/kernel-$(arch).bin
iso := build/hlq-os-$(arch).iso

linker_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
assembly_source_files := $(wildcard src/arch/$(arch)/*.asm)
assembly_object_files := $(patsubst src/arch/$(arch)/%.asm, \
    build/arch/$(arch)/%.o, $(assembly_source_files))

.PHONY: all clean run run_log iso kernel cargo

all: $(kernel)

clean:
	@rm -rf build
	@cargo clean

run: $(iso)
	@qemu-system-x86_64 -cdrom $(iso) -no-reboot -no-shutdown

run_log: $(iso)
	@qemu-system-x86_64 -cdrom $(iso) -d int -no-reboot -no-shutdown

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub
	@grub-mkrescue -o $(iso) build/isofiles 2> /dev/null
	@rm -rf build/isofiles

# Forces cargo to run, passing the dynamic flags (empty or --release)
cargo:
	@cargo build $(CARGO_FLAGS) --target x86_64-unknown-none

# Links the kernel, pointing to the correct debug or release .a file
$(kernel): cargo $(assembly_object_files) $(linker_script)
	@ld -n -T $(linker_script) -o $(kernel) $(assembly_object_files) $(rust_os)

# compile assembly files
build/arch/$(arch)/%.o: src/arch/$(arch)/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 $< -o $@