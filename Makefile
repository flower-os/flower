default: build iso

multiboot_header.o: src/boot
	nasm -f elf64 src/boot/multiboot_header.s -o build/boot/multiboot_header.o

makedirs:
	mkdir -p build
	mkdir -p build/boot

.PHONY: clean run

clean:
	rm -r build

run: iso
	qemu-system-x86_64 -cdrom build/flower.iso

debug: iso
	qemu-system-x86_64 -cdrom build/flower.iso -s

boot.o: src/boot
	nasm -f elf64 src/boot/boot.s -o build/boot/boot.o

build: makedirs multiboot_header.o boot.o src/linker.ld
	ld -n -o build/kernel.bin -T src/linker.ld build/boot/multiboot_header.o build/boot/boot.o

iso: build src/grub.cfg
	mkdir -p isofiles
	mkdir -p isofiles/boot/grub
	cp src/grub.cfg isofiles/boot/grub/
	cp build/kernel.bin isofiles/boot/
	grub-mkrescue -o build/flower.iso isofiles
