# Oxydizedbrain

Silly Rust learning project, somewhat inspired by my [ashBF project](https://github.com/AsuMagic/AshBF).

This is a simple optimizing interpreter for the Brainf\*\*k esoteric programming language.

## Features

- Recompilation of Brainf\*\*k source code into an optimized IR.
- Safe VM for the generated IR (a faster variant can be used when passing `--allow-unsafe`).
- JIT compilation using [Cranelift](https://github.com/bytecodealliance/wasmtime/tree/master/cranelift) (`--jit`).

## Usage

```bash
cargo run --release -- ./some_program.bf
```