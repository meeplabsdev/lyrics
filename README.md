# Lyrics

https://github.com/user-attachments/assets/32908b58-635a-4530-9cda-c166dd8e1e28

# How

This is a project I started on week 3 of [Terra](https://terra.hackclub.com/). It is just a lyrics viewer / music controller program written in rust using the eframe/egui ecosystem. Currently the UI includes:
- Song name
- Artist name
- Release year
- Song cover thumbnail
- Track scrub/seek
- Play/Pause
- Prev/Next
- Shuffle/Repeat
- Dynamic background ported from kawarp

It is missing:
- Volume
- Lyrics
- Queue/Playlist

# Demo

I can only build the project for linux at this point in time, so you will only find releases for linux, but it should run perfectly fine on windows or mac aswell, or even webasm with some effort put in. To build it from source is extremely simple, since all you need to do is enter the directory and run `cargo run` or `cargo build` (and obviously you need rust/cargo installed).

When running the release, if you have an error like "cannot execute: required file not found", it is because the compiled program relies on some system libraries that are not present on your system. In this case, you must build it from scratch rather than using the release (see below).

# Building

If you want to test it on windows, (I cannot confirm if this works on not), or if the release does not work for you, you must build it yourself. You must have rust/cargo installed, and run `cargo run` or `cargo build` inside the project directory (the one with `Cargo.toml`). The output will be `target/debug/lyrics(.exe)`. Optionally you can build with `--release` for an optimised build, which will be output to `target/release/lyrics(.exe)`. There are no external dependencies not handled by cargo.

- `git clone https://github.com/meeplabsdev/lyrics.git`
- `cd lyrics`
- `cargo run --release`

# Nix

For running on nixos, I have written a flake. It can be used like `nix run github:meeplabsdev/lyrics`
