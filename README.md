# iti

A small [mogwai](https://crates.io/crates/mogwai) WASM UI component library built on Bootstrap 5.

## Prerequisites

- [Rust](https://rustup.rs/)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev/): `cargo install trunk`

## Usage

Add `iti` as a dependency with the gallery feature disabled:

```toml
[dependencies]
iti = { git = "https://github.com/schell/iti", default-features = false }
```

## Component Gallery

To run the built-in component gallery locally:

```sh
trunk serve
```

from `crates/iti/`.

## Documentation

```sh
cargo doc --open
```

## License

MIT OR Apache-2.0
