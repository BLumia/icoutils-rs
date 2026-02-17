# icoutils-rs

> [!NOTE]
> This project is not affiliated with, endorsed by, supported by, or in any way associated with the original [`icoutils`](https://www.nongnu.org/icoutils/) project.

This project provides a drop-in replacement of `icotool` from [`icoutils`](https://www.nongnu.org/icoutils/), implemented in Rust.

## Why?

- `icoutils` is a great tool, but it's based on autotools build system, which is hard to compile on Windows.
- Extra CMake Modules' [`ECMAddAppIcon`](https://api.kde.org/ecm/module/ECMAddAppIcon.html) requires `icotool` to generate icon files on Windows.
- I previously said "Why nobody RIIR `icoutils`" as a RIIR meme joke.

## Installation

### Via `cargo-binstall` (suggested)

If you have [`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall) installed, you can use it to install this program.

```bash
cargo binstall icoutils-rs
```

### Via `cargo install`

```bash
cargo install icoutils-rs
```

## Roadmap

- [x] `icotool --help`
- [x] `icotool --list`
- [x] `icotool --create`
- [x] `icotool --extract`
- [ ] `wrestool --list`
- [ ] `wrestool --extract`

## LICENSE

icoutils-rs as a whole is licensed under MIT license. Individual files may have a different, but compatible license.
