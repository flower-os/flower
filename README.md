# FlowerOS

*A small learning OS*

## Setup

You will need:
 - [rustup](https://rustup.rs) and a nightly Rust build to compile;
 - The `rust-src` component from rustup;
 - [Xargo](https://github.com/japaric/xargo);
 - [nasm](http://www.nasm.us/);
 - ld;
 - [qemu](https://www.qemu.org/) (to run in a virtual machine);
 - X server to run qemu;
 - GNU GRUB (grub-mkrescue);
 - GNU make;

## Building

You can make the iso with `make iso`, and launch qemu and run it with `make run`. To enable debug symbols,
add `debug=1` to the make command.

## Thanks

Much thanks to:
 - [IntermezzOS](https://intermezzos.github.io) and its guide;
 - [Steve Klabnik](https://http://www.steveklabnik.com/) (its creator);
 - [Phil Opp](https://phil-opp.com) and his [blog OS](https://os.phil-opp.com);
 - the people over on the [Rust discord](https://discord.me/rust-lang), such as:
   - toor,
   - rep nop,
   - and nyrox;
 - the [OsDev wiki](http://wiki.osdev.org)
 - [Wikipedia](https://wikipedia.org) (of course!)
