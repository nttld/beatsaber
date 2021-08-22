# beatsaber

For langjam

## Documentation

See [docs](docs) for more info.

### Prerequisites

* A compatible C compiler
* LLVM 12
* Rust

### Getting Started

See [getting-started](docs/getting_started.md)

### Building the compiler

```bash
# Compile to ./target/release/bsc
cargo build --release
# Or alternatively install to ~/.cargo/bin
cargo install --path .
```

### Compiling the example

```bash
# Compile the beatsaber program
bsc examples/beatsaber.beatsaber -o beatsaber
# Run the result
./beatsaber
```
