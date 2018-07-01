build_containing_dir := build
debug ?= 0

ifneq ($(debug), 1)
else ifndef log_level
    log_level := debug
endif

ifndef log_level
    log_level := ""
endif

ifeq ($(debug), 1)
    nasm_flags := -f elf64 -F dwarf -g
    build_type := debug
    qemu_flags := -s -S
    cargo_flags := --features $(log_level)
else
    nasm_flags := -f elf64
    cargo_flags := --release --features $(log_level)
    build_type := release
endif

linker_script := cfg/linker.ld
grub_cfg := cfg/grub.cfg
out_dir = $(build_containing_dir)/$(build_type)
asm_dir := kernel/src/asm
rust_crate_dir := kernel
rust_kernel := $(out_dir)/libflower_kernel.a
target := x86_64-unknown-flower-none
asm_source_files := $(wildcard $(asm_dir)/*.asm)
asm_obj_files = $(patsubst $(asm_dir)/%.asm,  $(out_dir)/%.o, $(asm_source_files))

kernel = $(out_dir)/kernel.elf
grub_iso = $(out_dir)/flower.iso

default: build

.PHONY: clean run build $(rust_kernel) iso test
$(grub_iso): $(kernel) $(grub_cfg)
	@cp $(grub_cfg) $(out_dir)/isofiles/boot/grub/
	@cp $(kernel) $(out_dir)/isofiles/boot/
	@grub-mkrescue -o $(out_dir)/flower.iso $(out_dir)/isofiles

test:
	cd $(rust_crate_dir) && \
        cargo test

build: $(kernel)
iso: $(grub_iso)

# Run with qemu
run: $(grub_iso)
	@qemu-system-x86_64 -cdrom $(grub_iso) $(qemu_flags)

# Clean build dir
clean:
	@rm -rf build
	@cd $(rust_crate_dir) && \
	  RUST_TARGET_PATH=$(shell pwd)/$(rust_crate_dir) cargo clean

# Make build directories
makedirs:
	@mkdir -p $(out_dir)
	@mkdir -p $(out_dir)/isofiles
	@mkdir -p $(out_dir)/isofiles/boot/grub

# Compile rust
$(rust_kernel): $(rust_crate_dir)/**/*
	@cd $(rust_crate_dir) && \
      RUST_TARGET_PATH=$(shell pwd)/$(rust_crate_dir) cargo xbuild --target $(target) $(cargo_flags)
	@mv $(rust_crate_dir)/target/$(target)/$(build_type)/libflower_kernel.a $(rust_kernel)

# Compile kernel.elf
$(kernel): $(asm_obj_files) $(linker_script) $(rust_kernel)
	@ld -n -T $(linker_script) -o $(kernel) $(asm_obj_files) $(rust_kernel) --gc-sections
    
# Compile asm files
$(out_dir)/%.o: $(asm_dir)/%.asm makedirs
	@nasm $(nasm_flags) $< -o $@
