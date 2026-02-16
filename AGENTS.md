# AGENTS.md — Coding Agent Guidelines for iti

## Project Overview

`iti` is a Rust/WebAssembly UI component library built on [mogwai](https://crates.io/crates/mogwai) v0.7
and Bootstrap 5. It compiles to `wasm32-unknown-unknown` and runs in the browser.

The workspace lives at the repo root (`Cargo.toml`) with a single crate at `crates/iti/`.
The crate produces both an `rlib` (for use as a library dependency) and a `cdylib` (for WASM).

A `"library"` feature (enabled by default) adds a built-in component gallery (storybook)
defined in `src/library.rs` with sandbox modules gated by `#[cfg(feature = "library")]` inside
individual component files.

## Build / Lint / Test Commands

### Prerequisites

- Rust toolchain (edition 2021)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev/): `cargo install trunk`

### Build

```bash
cargo check -p iti                   # type-check (fast feedback loop)
trunk build                          # WASM dev build (from crates/iti/)
trunk build --release                # WASM release build
trunk serve                          # dev server with live-reload (from crates/iti/)
cargo build                          # host-target build (useful for IDE checks)
```

### Lint

```bash
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check                    # verify formatting
cargo fmt                            # apply formatting
```

There is no `rustfmt.toml` or `clippy.toml` — default settings apply.

### Test

There are currently no tests. If you add tests:

```bash
cargo test                           # run all tests
cargo test -p iti                    # run tests in the iti crate
cargo test test_name                 # run a single test by name
cargo test test_name -- --nocapture  # run a single test with stdout
```

## Code Style Guidelines

### Formatting

- Use default `rustfmt` settings (no config file).
- 4-space indentation, no tabs.
- Run `cargo fmt` before committing.

### Imports

Imports follow a consistent three-group pattern separated by blank lines:

1. **Standard library** — `use std::...`
2. **External crates** — `use futures_lite::...`, `use mogwai::prelude::*`, etc.
3. **Local crate** — `use crate::...` or `use super::...`

Mogwai is imported via its prelude: `use mogwai::prelude::*`. In the WASM entry
point (`lib.rs`), use `use mogwai::web::prelude::*` for web-specific types.

Use glob imports only for preludes (`mogwai::prelude::*`) and parent `library`
modules (`use super::*`). Prefer explicit imports elsewhere.

### Naming Conventions

- **Types**: `PascalCase` — `Alert`, `Button`, `TabList`, `ListEvent`, `IconGlyph`
- **Functions/methods**: `snake_case` — `set_flavor`, `get_pane_mut`, `step`
- **Feature flags**: lowercase kebab — `"library"`
- **Enum variants**: `PascalCase` — `Flavor::Primary`, `IconGlyph::CirclePlus`
- Accessor pairs: `get_foo()` / `get_foo_mut()`, `inner()` / `inner_mut()`
- Boolean setters: `set_is_visible(bool)`, `set_is_active(bool)`
- State mutators: `set_text()`, `set_flavor()`, `set_size()`

### Type Patterns

All UI components follow this structure:

1. **`#[derive(ViewChild)]`** on the struct.
2. **Generic over `V: View`** — the mogwai view abstraction.
3. **`#[child]` field** — the root DOM element (`V::Element`).
4. **`Proxy<T>`** — reactive state cells for values that update the DOM.
5. **`V::EventListener`** — for DOM event streams (clicks, etc.).
6. **`new()` constructor** — uses `rsx! { ... }` macro for declarative DOM.
7. **`async fn step(&self/&mut self)`** — async event loop; awaits the next
   event and mutates state. Called in a `loop { component.step().await }` pattern.

When a component needs a complex internal state, wrap it in a private struct
(e.g., `ItemState`, `IconState`) behind a `Proxy<T>`.

### Component Sandbox Modules

Each component that has an interactive demo defines a `pub mod library` gated
by `#[cfg(feature = "library")]` at the bottom of its file. These modules
contain a `*LibraryItem<V: View>` struct with `Default` impl and `step()` method.

### Error Handling

- Use `snafu` for typed errors and the `whatever_context` pattern for ad-hoc
  error messages (see `storage.rs`).
- In WASM entry points, use `.unwrap_throw()` (from `wasm-bindgen`) instead of
  `.unwrap()` for better browser error messages.
- Return `Result<T, snafu::Whatever>` for functions that can fail with
  heterogeneous errors.
- Reserve `.unwrap()` for cases where failure is a programming bug (e.g.,
  `fern::Dispatch::apply().unwrap()` during initialization).

### Async Patterns

- Async event loops use `step()` methods called inside `loop { }` blocks.
- Use `futures_lite::FutureExt::or()` to race multiple futures (e.g., a pane
  future vs. a list click future).
- Use `futures_lite::stream::unfold` for creating stateful event streams.
- Use `mogwai::future::race_all` when racing a collection of homogeneous futures.
- Pin boxed streams as `Pin<Box<dyn Stream<Item = T>>>` when storing them in structs.
- Use `.boxed_local()` for locally-pinned futures (WASM is single-threaded).

### DOM Construction

- Use the `rsx! { }` macro for all DOM construction.
- Inside `rsx!`, bind variables with `let name = element(...) { children }`.
- Reactive attributes use `Proxy` bindings:
  `class = proxy_var(value => format!("class-{value}"))`.
- Event listeners: `on:click = variable_name`.
- Embed child components with `{&component}` or `{component}`.

### Documentation

- Every module has a `//!` doc comment at the top describing its purpose.
- Every public struct has a `///` doc comment.
- Method-level docs are used for non-obvious behavior (e.g., panic conditions,
  edge cases like index-out-of-bounds).
- Use `## Panics` and `## Note` sections in doc comments where appropriate.

### Dependencies

- All dependency versions are declared in the workspace root `Cargo.toml` under
  `[workspace.dependencies]` and referenced as `dep.workspace = true` in crate manifests.
- When adding a new dependency, add it to the workspace first, then reference it
  from the crate.
- `web-sys` features are declared centrally in the workspace. Add new features
  there when needed.

### Feature Flags

- `"library"` (default): enables the component gallery and per-component sandbox
  modules. Disable with `default-features = false` when using iti as a dependency.
- Gate all gallery-only code with `#[cfg(feature = "library")]`.

### Files and Structure

New components go in `crates/iti/src/components/` as a new file, re-exported
from `mod.rs`. If the component needs a gallery demo, add a
`#[cfg(feature = "library")] pub mod library` section and register it in
`library.rs`.
