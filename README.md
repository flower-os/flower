# FlowerOS [![Gitter](https://badges.gitter.im/flower-os/Lobby.png)](https://gitter.im/flower-os/Lobby)

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

It will currently build on `rustc 1.36.0-nightly (e305df184 2019-04-24)`.

## Building

You can make the iso with `make iso`, and launch qemu and run it with `make run`. To enable debug symbols,
add `debug=1` to the make command. If this does not work, try `make clean`ing and checking you're on the latest 
rust/cargo-xbuild version. If this does still not work, then open an issue.

You can also get builds from [Flower's CI/CD](https://ci.gegy1000.net/job/Flower/).

## Contributing

If you wish to PR something to Flower, thanks so much! Just note to please **pull request into development, not master** 
if you are making a change to the codebase. 

Generally, the workflow for submitting a pull request goes like this:

1. Open an issue that your PR aims to solve and request to be assigned. This is just so we don't have multiple people working on 
the same thing on different branches/forks.
2. Fork flower
3. Create a new branch from `development` (if you're editing code and not e.g the README) which briefly describes the thing you 
are doing, e.g `acpi`.
4. Commit your things
5. Open a pull request. Select base as `development` (again, if you're editing code).
6. Wait for review. Sorry if the reviews are a bit nitpicky -- @gegy1000 and @Restioson (the maintainers) usually write reviews like that. It
does help to keep code quality good though.
7. Debate review comments/implement requested changes.
8. Repeat until everyone is happy with the changes.
9. Your PR should be merged Soon™!

## Code Style

Generally, we try to follow [the rust style guidline](https://github.com/rust-lang-nursery/fmt-rfcs/blob/master/guide/guide.md).
To keep the code consistent, we ask if all contributors could also adhere to these guidelines. Unfortunately, we haven't run 
Clippy or Rustfmt on flower [just yet](https://github.com/Restioson/flower/issues/13), but this is slated to be done just before 
0.2.0. Thus, please refrain from formatting things unrelated to the PR you are working on. This is to avoid merge conflicts.

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
