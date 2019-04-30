build_containing_dir := ../build

target := x86_64-unknown-flower-none
out_dir := $(build_containing_dir)/$(build_type)
asm_dir := src/asm
asm_source_files := $(wildcard $(asm_dir)/*.asm)
asm_obj_files := $(patsubst $(asm_dir)/%.asm, $(out_dir)/%.o, $(asm_source_files))
linker_script := linker.ld
init_lib := $(out_dir)/libinit.a
init := $(out_dir)/init.elf

default: $(init)

# Compile rust
$(init_lib):
	@RUST_TARGET_PATH=$(shell pwd) cargo xbuild --target $(target) $(cargo_flags)
	@mv target/$(target)/$(build_type)/libinit.a $(init_lib)

# Link elf
$(init): $(asm_obj_files) $(linker_script) $(init_lib)
	@ld -n -T $(linker_script) -o $(init) $(asm_obj_files) $(init_lib) --gc-sections

# Compile asm files
$(out_dir)/%.o: $(asm_dir)/%.asm
	@nasm $(nasm_flags) $< -o $@
