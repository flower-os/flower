build_containing_dir := build
debug ?= 0

ifeq ($(debug), 1)
    nasm_flags := -f elf64 -F dwarf -g
    build_type := debug
    qemu_flags := -s -S
else
    nasm_flags := -f elf64
    xargo_flags := --release
    build_type := release
endif
    
linker_script := cfg/linker.ld
grub_cfg := cfg/grub.cfg
out_dir = $(build_containing_dir)/$(build_type)
asm_dir := kernel/src/asm
rust_crate_dir := kernel
rust_kernel := $(out_dir)/libflower_kernel.a
asm_source_files := $(wildcard $(asm_dir)/*.asm)
asm_obj_files = $(patsubst $(asm_dir)/%.asm,  $(out_dir)/%.o, $(asm_source_files))

kernel = $(out_dir)/kernel.bin
iso = $(out_dir)/flower.iso

default: $(iso)

.PHONY: clean run $(rust_kernel)
$(iso): $(kernel) $(grub_cfg)
	@cp $(grub_cfg) $(out_dir)/isofiles/boot/grub/
	@cp $(kernel) $(out_dir)/isofiles/boot/
	@grub-mkrescue -o $(out_dir)/flower.iso $(out_dir)/isofiles

run: $(iso)
	@qemu-system-x86_64 -cdrom $(iso) $(qemu_flags)

# Clean build dir
clean:
	@rm -rf build

# Make build directories
makedirs:
	@mkdir -p $(out_dir)
	@mkdir -p $(out_dir)/isofiles
	@mkdir -p $(out_dir)/isofiles/boot/grub

# Compile rust
$(rust_kernel): $(rust_crate_dir)/Cargo.toml
	@cd $(rust_crate_dir) && \
      xargo build --target x86_64-unknown-flower-gnu $(xargo_flags)
	@mv $(rust_crate_dir)/target/x86_64-unknown-flower-gnu/$(build_type)/libflower_kernel.a $(rust_kernel)

# Compile kernel.bin
$(out_dir)/kernel.bin: $(asm_obj_files) $(linker_script) $(rust_kernel)
	@ld -n -T $(linker_script) -o $(kernel) $(asm_obj_files) $(rust_kernel) --gc-sections
    
# Compile asm files
$(out_dir)/%.o: $(asm_dir)/%.asm makedirs
	@nasm $(nasm_flags) $< -o $@
