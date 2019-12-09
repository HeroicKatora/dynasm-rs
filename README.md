# A Dynamic assembler written in Rust for Rust.

The purpose of this tool is to ease the creation of programs that require run-time assembling.

This was forked from the `dynasm` crate, to remove its heavy integration into
proc-macro for basic parsing and code generation. The product is a library 

## Features

- Fully integrated in the rust toolchain, no other tools necessary.
- The assembler library works on the stable Rust compiler.
- The software form of assembly can be converted into a series of `Vec.push` and `Vec.extend`.
- Errors are almost all diagnosed at compile time in a clear fashion.

## Documentation

WIP

## Architecture support

- Supports the x64/x86 instruction sets in long and protected mode with every AMD/Intel/VIA extension except for AVX-512.
- NOT YET: Supports the aarch64 instruction set up to ARMv8.4 except for SVE instructions again. The development of this assembler backend has been generously sponsored by the awesome folks at [Wasmer](https://github.com/wasmerio/wasmer)!

## Example

```rust
WIP
```

## Background

This project is forked from [Dynasm](https://github.com/CensoredUsername/dynasm-rs)

## Sponsorship

None. Please sponsor by contributing code. For example, re-integrating the
`dynasm` plugin frontend on this backend?

## License

Mozilla Public License, v. 2.0, see LICENSE

Copyright 2016 CensoredUsername, HeroicKatora

## Guaranteed to be working compiler versions

Stable
