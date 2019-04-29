build_containing_dir := build

debug ?= 0

ifneq (debug, 1)
else ifndef log_level
    log_level := debug
endif

log_level ?= ""

ifeq ($(debug), 1)
    nasm_flags := -f elf64 -F dwarf -g
    build_type := debug
    qemu_flags := -s -m 256M -d int -no-reboot -no-shutdown -monitor stdio -serial file:serial.log
    kernel_cargo_flags := --features $(log_level)
else
    nasm_flags := -f elf64
    kernel_cargo_flags := --release --features $(log_level)
    init_cargo_flags := --release
    build_type := release
    qemu_flags := -m 256M -serial file:serial.log
endif

ifeq ($(wait_for_gdb), 1)
    qemu_flags += -S
endif

out_dir := $(build_containing_dir)/$(build_type)
grub_cfg := cfg/grub.cfg
grub_iso := $(out_dir)/flower.iso

kernel_crate_dir := kernel/
kernel_lib := $(out_dir)/libflower_kernel.a
kernel := $(out_dir)/kernel.elf

init_crate_dir := init/
init_lib := $(out_dir)/libinit.a
init := $(out_dir)/init.elf

default: build

.PHONY: clean run build $(rust_kernel) iso test

test:
	@cd $(rust_crate_dir) && \
        cargo test

build: $(kernel)
iso: $(grub_iso)

# Run with qemu
run: $(grub_iso)
	qemu-system-x86_64 -cdrom $(grub_iso) $(qemu_flags)

$(grub_iso): rm_old $(kernel) $(init) $(grub_cfg)
	@cp $(grub_cfg) $(out_dir)/isofiles/boot/grub/
	@cp $(kernel) $(out_dir)/isofiles/boot/
	@cp $(init) $(out_dir)/isofiles/boot/
	grub-mkrescue -o $(out_dir)/flower.iso $(out_dir)/isofiles

$(kernel): $(kernel_crate_dir)/**/* makedirs
	@$(MAKE) nasm_flags="$(nasm_flags)" build_type="$(build_type)" cargo_flags="$(kernel_cargo_flags)" -s \
	-C $(kernel_crate_dir)

$(init): $(init_crate_dir)/**/* makedirs
	@$(MAKE) nasm_flags="$(nasm_flags)" build_type="$(build_type)" cargo_flags="$(init_cargo_flags)" \
	 -s -C $(init_crate_dir)

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

rm_old:
	@rm -f $(kernel_lib)
	@rm -f $(init_lib)
