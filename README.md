# iti

A small [mogwai](https://crates.io/crates/mogwai) WASM UI component library
with a Mac OS 9 Platinum aesthetic.

iti aims for a faithful classic-Mac look — raised/pressed bevels, period-correct
typography (Chicago FLF, Geneva, Apple Garamond), and a fixed Platinum gray
palette — wrapped in a modern Rust async API.

## Features

- 20+ components themed in Platinum: forms, containers, display, composition primitives
- Reactive state via mogwai's `Proxy<T>` cells
- Async event loop via `step()` — pull-based, no callbacks or channels
- Built-in component gallery for live exploration
- Optional fully-embedded mode — single WASM binary deploy with no network requests

## Component Gallery

To run the built-in component gallery locally:

```sh
trunk serve
```

from `crates/iti/`. Browse every component with interactive controls.

## Quick Example

```rust
use iti::components::button::Button;
use iti::components::Flavor;
use mogwai::web::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub async fn main() {
    iti::assets::inject_cdn_links();

    let button = Button::<Web>::new("Click me", Some(Flavor::Primary));
    mogwai::web::body().append_child(&button);

    loop {
        let _ev = button.step().await;
        log::info!("clicked!");
    }
}
```

## Installation

### Prerequisites

- [Rust](https://rustup.rs/)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- [Trunk](https://trunkrs.dev/): `cargo install trunk`

### As a dependency

```toml
[dependencies]
iti = { git = "https://github.com/schell/iti", default-features = false }
```

The `library` feature (which adds the gallery) is on by default. Disable
it with `default-features = false` when consuming iti as a dependency.

### Loading CSS and fonts

iti components depend on a single unified stylesheet (`iti.css`) plus
[Font Awesome 6](https://fontawesome.com/) for icon glyphs. The
`iti::assets` module provides three ways to load them.

**Option A — CDN (simplest):**

```rust
iti::assets::inject_cdn_links();
```

Injects color tokens and `iti.css` as `<style>` tags, plus a `<link>`
to Font Awesome 6 from a public CDN. Requires an internet connection
for the icon font.

**Option B — Fully embedded (offline-capable):**

```toml
[dependencies]
iti = { git = "https://github.com/schell/iti", default-features = false, features = ["embed-assets"] }
```

```rust
iti::assets::embedded::inject_styles();
```

Compiles all CSS, Font Awesome icon webfonts (woff2), and Mac OS 9
typography fonts (Chicago FLF, Geneva, and Apple Garamond TTF) into
the WASM binary (~1.2 MB total). At runtime, fonts are exposed via
Blob URLs created from the embedded bytes. No network connection
required.

Font Awesome Brands icons are not embedded to save space; only Solid,
Regular, and v4-compatibility fonts are included.

**Option C — Manual / Trunk:**

Ignore the `assets` module and wire up assets yourself. With Trunk:

```html
<link data-trunk rel="copy-file" href="path/to/iti.css" />
<link data-trunk rel="copy-dir" href="path/to/fontawesome" />
<link rel="stylesheet" href="iti.css" />
<link rel="stylesheet" href="fontawesome/css/all.min.css" />
```

When loading CSS via `<link>` rather than the helpers above, you must
also call `iti::assets::inject_color_tokens()` from your WASM entry
point so the design-token CSS custom properties resolve. This is what
the built-in component gallery uses.

## Components

| Category    | Components                                                                            |
| ----------- | ------------------------------------------------------------------------------------- |
| Forms       | `Button`, `ButtonGroup`, `Checkbox`, `Dropdown`, `Radio`, `Select`, `Slider`          |
| Containers  | `Card`, `List`, `Modal`, `Pane`, `Tab` (`TabList` / `TabPanel`), `Table`, `TitleBar`  |
| Display     | `Alert`, `Badge`, `Icon`, `Progress`, `Toast`                                         |
| Composition | `Shadow`, `Widget`                                                                    |

Each lives in `crates/iti/src/components/` and includes a `library`
module with a sandbox demo (visible in the gallery). See the
component docs (`cargo doc --open`) for full APIs.

## Architecture & Conventions

### Component structure

Every component is a struct that derives `ViewChild`, is generic over
`V: View` (mogwai's view abstraction), and holds a `#[child]` root DOM
element. Reactive state lives in `Proxy<T>` cells that automatically
update the DOM when mutated. DOM trees are built declaratively with
the `rsx!{}` macro.

```rust
#[derive(ViewChild)]
pub struct Alert<V: View> {
    #[child]
    div: V::Element,          // root DOM element
    text: V::Text,            // reactive text node
    flavor: Proxy<Flavor>,    // reactive state cell
}
```

### The `step()` convention

Components that produce user-initiated events expose a single async
method:

```rust
pub async fn step(&self) -> SomeEvent { ... }
```

Callers drive the event loop themselves:

```rust
loop {
    component.step().await;
}
```

Key points:

- **Pull-based, not push-based.** There are no callbacks, no channels,
  and no `Stream` types in public APIs. `step()` *is* the stream — one
  event at a time.
- **Always named `step`.** Every async method in the library uses this
  name.
- **Receiver type varies.** Most components take `&self` (mogwai's
  event listeners use interior mutability). Components that manage
  complex internal state — like `Widget` (which wraps an arbitrary
  `Stream`) and `Table` (which manages column resize state) — take
  `&mut self`.
- **Not every component has one.** Purely presentational components
  like `Alert`, `Badge`, and `Progress` expose only synchronous
  setters. Only components with user-initiated events (clicks,
  selections) have `step()`.

Compose multiple event sources by racing `step()` calls with
`futures_lite::FutureExt::or()` (heterogeneous) or
`mogwai::future::race_all` (homogeneous):

```rust
let pane_event = pane.step().map(Ok);
let list_event = list.step().map(Err);
match pane_event.or(list_event).await { ... }
```

### Generic vs. concrete children

Some container components are generic over their child type
(`List<V, T>` accepts any `T: ViewChild<V>`), while others own a
specific child type (`ButtonGroup<V>` owns `Button<V>` directly). The
choice depends on how specialised the container needs to be.

### Component gallery

Each component may define a `#[cfg(feature = "library")] pub mod library`
sandbox module at the bottom of its file. These modules contain a
`*LibraryItem<V>` struct with a `Default` impl and `step()` method,
registered in `library.rs`. Run `trunk serve` from `crates/iti/` to
browse the gallery.

## Documentation

```sh
cargo doc --open
```

## Development

```sh
cargo check -p iti                                        # type-check (fast)
trunk build                                               # WASM dev build (from crates/iti/)
trunk build --release                                     # WASM release build
cargo clippy --all-targets --all-features -- -D warnings  # lint
cargo fmt                                                 # format
```

## License

MIT OR Apache-2.0
