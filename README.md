# Overview

This is a [CHIP-8](https://en.wikipedia.org/wiki/CHIP-8) Emulator/Interpreter written in Rust. It's chief purpose is to attain some familiarity with the Rust programming language and the VSCode IDE.

# Usage

```cargo run <ROM File Path>```

# Implementation notes

Although you can look through the `Cargo.toml` file and find out, I think it's helpful to declare that I use the following crates to support this implementation:
- `show_image`
- `rand`

As such, I believe it should compile cleanly on most Linux distributions, but YMMV. I used a Debian distribution as my development environment, so I have not tested this on Windows.
If anyone is interested in doing so, please let me know your findings (I would gladly accept pull requests for updates which would add Windows support if something is missing).

# Testing

There are *a lot* of ROMs available online to test various bits of functionality. I've tested my implementation to work with the following programs without having to mess with the default
instruction processing latency:
- [IBM Logo test](https://github.com/Timendus/chip8-test-suite?tab=readme-ov-file#ibm-logo)
- [Keypad test](https://github.com/Timendus/chip8-test-suite?tab=readme-ov-file#keypad-test)
- [Pong](https://github.com/netpro2k/Chip8/blob/master/games/Pong.ch8)
- [BonCoder Test](https://github.com/cj1128/chip8-emulator/blob/master/rom/BC_test.ch8)
- [Random number test](https://github.com/mattmikolay/chip-8/blob/master/randomnumber/random_number_test.ch8)
- [Tetris - from the CHIP-8 games pack](https://www.zophar.net/pdroms/chip8/chip-8-games-pack.html)

I'll try to keep this list updated as I test more ROMs, but please let me know if you've had a chance to try out a ROM and it works (so I'll add it to this list).

# Pending tasks
- Make instruction latency configurable via commandline parameter.
- Make shift instruction behaviour quirk configurable via commandline parameter.

# Credits

If anyone wants to try their hand at writing a CHIP-8 emulator, I would recommend they read this [excellent post](https://tobiasvl.github.io/blog/write-a-chip-8-emulator/#timers) by Tobias Langhoff.
It summarizes what needs to be implemented without spelling out the specifics of implementing it, and is written in a language-agnostic way, so you can pick your poison of programming language.
