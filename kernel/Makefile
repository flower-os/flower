build_containing_dir := ../build

target := x86_64-unknown-flowerkernel-none
out_dir := $(build_containing_dir)/$(build_type)
asm_dir := src/asm
asm_source_files := $(wildcard $(asm_dir)/*.asm)
asm_obj_files := $(patsubst $(asm_dir)/%.asm, $(out_dir)/%.o, $(asm_source_files))
linker_script := linker.ld
kernel_lib := $(out_dir)/libflower_kernel.a
kernel := $(out_dir)/kernel.elf
rustflags := "-C code-model=kernel"

default: $(kernel)

# Compile rust
$(kernel_lib):
	@RUST_TARGET_PATH=$(shell pwd) RUSTFLAGS=$(rustflags) cargo xbuild --target $(target) $(cargo_flags)
	@mv target/$(target)/$(build_type)/libflower_kernel.a $(kernel_lib)

# Link elf
$(kernel): $(asm_obj_files) $(linker_script) $(kernel_lib)
	@ld -n -T $(linker_script) -o $(kernel) $(asm_obj_files) $(kernel_lib) --gc-sections

# Compile asm files
$(out_dir)/%.o: $(asm_dir)/%.asm
	@nasm $(nasm_flags) $< -o $@
