# Witnesscalc Adapter

[![Crates.io](https://img.shields.io/crates/v/witnesscalc-adapter?label=witnesscalc-adapter)](https://crates.io/crates/witnesscalc-adapter)

This project provides a Rust adapter for compiling and linking [Circom circuit witness generator C++ sources](https://docs.circom.io/getting-started/computing-the-witness/#computing-the-witness-with-c) into a native library for target platforms (e.g., mobile devices). It includes macros and functions to facilitate the integration of witness generation into Rust codebases.

## Usage

Include the crate in your `Cargo.toml`:

```toml
[dependencies]
witnesscalc-adapter = "0.1"

[build-dependencies]
witnesscalc-adapter = "0.1"
```

In `build.rs`, add the following code to compile the witness generator C++ sources (`<circuit name>.cpp` and `<circuit name>.dat`) into a native library and link to it:

```rust
witnesscalc_adapter::build_and_link("../path to directory containing your C++ sources");
// e.g. witnesscalc_adapter::build_and_link("../testdata");
// The directory should contain the following files:
// - <circuit name>.cpp
// - <circuit name>.dat
```

In your main code, use the `witness` macro to generate a witness for a given input:

```rust
witnesscalc_adapter::witness!(<circuit name>);
// e.g. witnesscalc_adapter::witness!(multiplier2);
```

Calculate the witness by using the `<circuit name>_witness` function.

```rust
let wtns = <circuit name>_witness(inputs_json_string)
// e.g. let wtns = multiplier2_witness("{\"a\": [\"2\"], \"b\": [\"3\"]}")
```

It will generate a `wtns` bytes array like the output of [witnesscalc](https://github.com/0xPolygonID/witnesscalc) or [snarkjs](https://github.com/iden3/snarkjs).

Convert the `wtns` to bigints by using the `parse_witness_to_bigints` function.

```rust
let witness = parse_witness_to_bigints(&wtns).unwrap();
```

## Supported platforms

### Linux

-   x86_64 linux
-   arm64 linux

### MacOS

-   aarch64-apple-darwin
-   x86_64-apple-darwin

### iOS

-   aarch64-apple-ios
-   aarch64-apple-ios-sim
-   x86_64-apple-ios

### Android

-   aarch64-linux-android
-   x86_64-linux-android

## Community

-   Website: [zkmopro.com](https://zkmopro.com)
-   X account: <a href="https://twitter.com/zkmopro"><img src="https://img.shields.io/twitter/follow/zkmopro?style=flat-square&logo=x&label=zkmopro"></a>
-   Telegram group: <a href="https://t.me/zkmopro"><img src="https://img.shields.io/badge/telegram-@zkmopro-blue.svg?style=flat-square&logo=telegram"></a>

# Acknowledgements

-   Inspired by https://github.com/chancehudson/rust-witness. This adapter relies on the [0xPolygonID/witnesscalc library fork](https://github.com/zkmopro/witnesscalc).
-   The project is sponsored by [PSE](https://pse.dev/).
