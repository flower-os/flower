debug ?= 0
asm_dir := src/boot/

ifeq ($(debug), 1)
    nasm_flags = -f elf64 -F dwarf -g
    out_dir = build/debug
    qemu_flags = -S
else
    nasm_flags = -f elf64
    out_dir = build/release
endif
    
linker_script := src/linker.ld
grub_cfg := src/grub.cfg

asm_source_files := $(wildcard $(asm_dir)/*.asm)
asm_obj_files = $(patsubst $(asm_dir)/%.asm,  $(out_dir)/%.o, $(asm_source_files))

kernel = $(out_dir)/kernel.bin
iso = $(out_dir)/flower.iso

default: $(iso)

.PHONY: clean run debug
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

# Compile kernel.bin
$(out_dir)/kernel.bin: $(asm_obj_files) $(linker_script)
	@ld -n -T $(linker_script) -o $(kernel) $(asm_obj_files)
    
# Compile asm files
$(out_dir)/%.o: $(asm_dir)/%.asm makedirs
	@nasm $(nasm_flags) $< -o $@
