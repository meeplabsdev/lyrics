# Lyrics

https://github.com/user-attachments/assets/6b72b98e-3c21-455d-ad0a-c51c5292b15a

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

Prebuilt binaries can be found for linux (both x86 and arm) and for windows (x86 only) under the releases. On linux, you may have to run `chmod +x lyrics*` to make it executable, but then it can be executed without any other special requirements (provided you are on a supported desktop environment).

# Building

If the release does not work for you or you are on an unsupported platform, you must build it yourself. You must have rust/cargo installed, and run `cargo run` or `cargo build` inside the project directory (the one with `Cargo.toml`). The output will be `target/debug/lyrics(.exe)`. Optionally you can build with `--release` for an optimised build, which will be output to `target/release/lyrics(.exe)`. There are no external dependencies that are not handled by cargo.

- `git clone https://github.com/meeplabsdev/lyrics.git`
- `cd lyrics`
- `cargo run --release`

# Nix

For running on nixos, I have written a flake. It can be used like `nix run github:meeplabsdev/lyrics`, which will automatically set up the correct environment (but you still need to be on a supported desktop environment).
