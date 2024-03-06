# Getting Started

Running Zenix on your own machine is dead simple. These instructions work for both Windows and Linux, and possibly Mac-OS.

## Prerequisites

In order to build the project, you'll need to install Rust Nightly ([rust-lang.org](https://www.rust-lang.org/tools/install)). After installing `rustup` you can install nightly by executing `rustup install nightly`.

That's it! You can now build the project. However, you'll most likely want to run Zenix inside an virtual environment. To do this, make sure you have the `qemu-system-x86_64` binary placed in a directory from your `$PATH` variable.

### Arch

```bash
pacman -S qemu-system-x86_64 qemu-ui-gtk
```

## Debian

```bash
apt install qemu-system-x86_64 qemu-system-gui
```

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run --release
```
