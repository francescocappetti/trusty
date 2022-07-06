# trusty
A dead simple text editor completely written in Rust.

## Installation

### Binary
> NOTE: currently the executable is only available for Linux.

1. Go to the [release](https://github.com/kappq/trusty/releases) tab and download the latest executable.
2. Move into the directory where you downloaded the file.
3. Make the file executable with `chmod +x trusty`.
4. Copy the executable to `/usr/local/bin`.
5. You can run the editor by typing `trusty`.

### Manual
> NOTE: this is installtion method requires the [Rust programming language](https://www.rust-lang.org/tools/install) to be installed.

Clone the repository:
```
git clone https://github.com/kappq/trusty.git
```
Move into the new directory:
```
cd trusty
```
Build the program for release:
```
cargo build --release
```
You can find the final executable in `./target/release`. Copy it in a place where you can easily access it (like in `/usr/local/bin`).

## Usage
Run the editor with `trusty <filename>` if the file name isn't specified, the editor will create an unnamed file.

## Keybindings
- `CTRL-Q` = quit
- `CTRL-S` = save
- `CTRL-F` = find
