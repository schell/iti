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

### Loading CSS and fonts

iti components depend on Bootstrap 5, Bootstrap Icons, and Font Awesome 6.
The `iti::assets` module provides helpers to load these stylesheets.

**Option A — CDN (simplest):**

```rust
iti::assets::inject_cdn_links();
```

This adds `<link>` tags for all three stylesheets (served from jsdelivr and
cdnjs) plus iti's own small custom stylesheet. Requires an internet connection.

**Option B — Embedded Bootstrap + CDN icon fonts:**

```toml
[dependencies]
iti = { git = "https://github.com/schell/iti", default-features = false, features = ["embed-assets"] }
```

```rust
iti::assets::embedded::inject_styles();
```

Bootstrap CSS (~227 KB) is compiled into the WASM binary and injected as a
`<style>` element — no network fetch. Icon font stylesheets (Bootstrap Icons,
Font Awesome) are still loaded from CDN because their CSS contains relative
`@font-face` URLs that only resolve when served from the CDN origin.

**Option C — Manual / Trunk:**

Ignore the `assets` module and wire up stylesheets yourself. For example, with
Trunk you can reference the vendored files in iti's `assets/` directory:

```html
<link data-trunk rel="css" href="path/to/bootstrap.min.css" />
<link data-trunk rel="css" href="path/to/bootstrap-icons.min.css" />
<link data-trunk rel="copy-dir" href="path/to/fontawesome" />
<link rel="stylesheet" href="fontawesome/css/all.min.css" />
```

This is what the built-in component gallery uses.

## Component Gallery

To run the built-in component gallery locally:

```sh
trunk serve
```

from `crates/iti/`.

## Architecture & Conventions

### Component structure

Every component is a struct that derives `ViewChild`, is generic over `V: View`
(mogwai's view abstraction), and holds a `#[child]` root DOM element. Reactive
state lives in `Proxy<T>` cells that automatically update the DOM when mutated.
DOM trees are built declaratively with the `rsx!{}` macro.

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

Components that produce user-initiated events expose a single async method:

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

- **Pull-based, not push-based.** There are no callbacks, no channels, and no
  `Stream` types in public APIs. `step()` *is* the stream — one event at a time.
- **Always named `step`.** Every async method in the library uses this name.
- **Takes `&self`**, not `&mut self`, thanks to mogwai's interior mutability on
  event listeners. (The sole exception is `Widget`, which wraps an arbitrary
  `Stream` internally.)
- **Not every component has one.** Purely presentational components like `Alert`,
  `Badge`, and `Progress` expose only synchronous setters. Only components with
  user-initiated events (clicks, selections) have `step()`.

Compose multiple event sources by racing `step()` calls with
`futures_lite::FutureExt::or()`:

```rust
let pane_event = pane.step().map(Ok);
let list_event = list.step().map(Err);
match pane_event.or(list_event).await { ... }
```

### Event return types

| Pattern | Example | Used by |
|---|---|---|
| Raw DOM event | `V::Event` | `Button` |
| Single-variant enum | `ModalEvent::Closed` | `Modal`, `Toast` |
| Index + event struct | `ListEvent { index, event }` | `List`, `ButtonGroup`, `TabList` |
| Optional event | `Option<DropdownEvent>` | `Dropdown` (None = toggle) |

### Generic vs. concrete children

Some container components are generic over their child type (`List<V, T>` accepts
any `T: ViewChild<V>`), while others own a specific child type (`ButtonGroup<V>`
owns `Button<V>` directly). The choice depends on how specialised the container
needs to be.

### Component gallery

Each component may define a `#[cfg(feature = "library")] pub mod library` sandbox
module at the bottom of its file. These modules contain a `*LibraryItem<V>`
struct with a `Default` impl and `step()` method, registered in `library.rs`.
Run `trunk serve` from `crates/iti/` to browse the gallery.

## Documentation

```sh
cargo doc --open
```

## License

MIT OR Apache-2.0
