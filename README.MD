# Witnesscalc Adapter

This project provides a Rust adapter for compiling and linking [Circom circuit witness generator C++ sources](https://docs.circom.io/getting-started/computing-the-witness/#computing-the-witness-with-c) into a native library for target platforms (e.g., mobile devices). It includes macros and functions to facilitate the integration of witness generation into Rust codebases.

## Usage

In `build.rs`, add the following code to compile the witness generator C++ sources into a native library and link to it:

```rust
witnesscalc_adapter::build_and_link("../path to directory containing your C++ sources");
```

In your main code, use the `witness` macro to generate a witness for a given input:

```rust
witnesscalc_adapter::witness!(<circuit name>);
```

# Acknowledgements

Inspired by https://github.com/chancehudson/rust-witness. This adapter relies on the [0xPolygonID/witnesscalc library fork](https://github.com/zkmopro/witnesscalc).
