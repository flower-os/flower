default: iso

.PHONY: clean run

# Intended-to-be-run configurations

## Normal run

iso: build src/grub.cfg
	mkdir -p isofiles
	mkdir -p isofiles/boot/grub
	cp src/grub.cfg isofiles/boot/grub/
	cp build/kernel.bin isofiles/boot/
	grub-mkrescue -o build/release/flower.iso isofiles
    
build: makedirs multiboot_header.o boot.o src/linker.ld
	ld -n -o build/release/kernel.bin -T src/linker.ld build/release/boot/multiboot_header.o build/release/boot/boot.o

run: iso
	qemu-system-x86_64 -cdrom build/flower.iso

## Debug run

debug-iso: debug-build src/grub.cfg
	mkdir -p debug-isofiles
	mkdir -p debug-isofiles/boot/grub
	cp src/grub.cfg debug-isofiles/boot/grub/
	cp build/kernel.bin debug-isofiles/boot/
	grub-mkrescue -o build/debug/flower.iso isofiles

debug-build: debug-makedirs debug-multiboot_header.o debug-boot.o src/linker.ld
	ld -n -o build/debug/kernel.bin -T src/linker.ld build/boot/multiboot_header.o build/boot/boot.o

debug: iso
	qemu-system-x86_64 -cdrom build/flower.iso -s

debug-wait: iso
	qemu-system-x86_64 -cdrom build/flower.iso -s -S

# Util configurations

## General
clean:
	rm -r build

## Normal files
makedirs:
	mkdir -p build/release
	mkdir -p build/release/boot 

## Debug
debug-makedirs:
	mkdir -p build/debug
	mkdir -p build/debug/boot 

# File configurations

## Normal files

multiboot_header.o: src/boot/multiboot_header.s
	nasm -f elf64 src/boot/multiboot_header.s -o build/release/boot/multiboot_header.o

boot.o: src/boot/boot.s
	nasm -f elf64 src/boot/boot.s -o build/release/boot/boot.o

## Files with debug symbols

debug-multiboot_header.o: src/boot/multiboot_header.s
	nasm -f elf64 -F dwarf -g src/boot/multiboot_header.s -o build/debug/boot/multiboot_header.o

debug-boot.o: src/boot/boot.s
	nasm -f elf64 -F dwarf -g src/boot/boot.s -o build/debug/boot/boot.o  
