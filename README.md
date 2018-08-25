# FlowerOS

*A small learning OS*

## Setup

You will need:
 - [rustup](https://rustup.rs) and a nightly Rust (if yours doesn't work, then update to latest) build to compile;
 - The `rust-src` component from rustup;
 - cargo-xbuild (`cargo install` it);
 - [nasm](http://www.nasm.us/);
 - ld;
 - [qemu](https://www.qemu.org/) (to run in a virtual machine);
 - X server to run qemu;
 - GNU GRUB (grub-mkrescue);
 - GNU make;

## Building

You can make the iso with `make iso`, and launch qemu and run it with `make run`. To enable debug symbols,
add `debug=1` to the make command. If this does not work, try `make clean`ing and checking you're on the latest 
rust/cargo-xbuild version. If this does still not work, then open an issue.

You can also get builds from [Flower's CI/CD](https://ci.gegy1000.net/job/Flower/).

## Contributing

If you wish to PR something to Flower, thanks so much! Just note to please **pull request into development, not master**.

## Thanks

Much thanks to:
 - [IntermezzOS](https://intermezzos.github.io) and its guide;
 - [Steve Klabnik](https://http://www.steveklabnik.com/) (its creator);
 - [Phil Opp](https://phil-opp.com) and his [blog OS](https://os.phil-opp.com);
 - [Redox](https://github.com/redox-os);
 - the people over on the [Rust discord](https://discord.me/rust-lang), such as:
   - [Toor](https://github.com/too-r),
   - [Rep nop](https://github.com/repnop), 
   - [Evrey](https://github.com/Evrey), 
   - Cult Pony,
   - [Oberien](https://github.com/oberien) 
   - and [Nyrox](https://github.com/Nyrox), 
 - the people on the [OSDev discord](https://discordapp.com/invite/aJPTY8Y) such as:
   - Omarrx024, 
   - [Mintsuki](https://github.com/mintsuki), 
   - [Korona](https://github.com/avdgrinten), 
   - and [Safsom](https://github.com/asfsom);
 - the people part of the [rust-osdev](https://github.com/rust-osdev), such as
   - [Isaac Woods](https://github.com/IsaacWoods);
 - the [OsDev wiki](http://wiki.osdev.org);
 - [Bare Metal Rust](http://www.randomhacks.net/bare-metal-rust/);
 - and [Wikipedia](https://wikipedia.org) (of course!);

This is not, of course, an exhaustive list! 
