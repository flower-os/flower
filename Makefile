default: iso

.PHONY: clean run

# Intended-to-be-run configurations

## Normal run

iso: build src/grub.cfg
	cp src/grub.cfg build/release/isofiles/boot/grub/
	cp build/release/kernel.bin build/release/isofiles/boot/
	grub-mkrescue -o build/release/flower.iso build/release/isofiles
    
build: makedirs multiboot_header.o boot.o src/linker.ld
	ld -n -o build/release/kernel.bin -T src/linker.ld build/release/boot/multiboot_header.o build/release/boot/boot.o

run: iso
	qemu-system-x86_64 -cdrom build/release/flower.iso

## Debug run

debug-iso: debug-build src/grub.cfg
	cp src/grub.cfg build/debug/isofiles/boot/grub/
	cp build/debug/kernel.bin build/debug/isofiles/boot/
	grub-mkrescue -o build/debug/flower.iso build/debug/isofiles

debug-build: debug-makedirs debug-multiboot_header.o debug-boot.o src/linker.ld
	ld -n -o build/debug/kernel.bin -T src/linker.ld build/debug/boot/multiboot_header.o build/debug/boot/boot.o

debug: debug-iso
	qemu-system-x86_64 -cdrom build/debug/flower.iso -s

debug-wait: debug-iso
	qemu-system-x86_64 -cdrom build/debug/flower.iso -s -S

# Util configurations

## General
clean:
	rm -rf build
	rm -rf isofiles
	rm -rf debug-isofiles

## Normal files
makedirs:
	mkdir -p build/release/isofiles
	mkdir -p build/release/isofiles/boot/grub
	mkdir -p build/release
	mkdir -p build/release/boot 

## Debug
debug-makedirs:
	mkdir -p build/debug/isofiles
	mkdir -p build/debug/isofiles/boot/grub
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
