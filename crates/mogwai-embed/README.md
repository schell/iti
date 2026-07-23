# mogwai-embed

Helpers for embedding static assets (CSS, fonts, images, scripts) into a
[mogwai](https://crates.io/crates/mogwai)-based WASM binary, with no
network requests at runtime.

## Why

Most crates that wrap a JS/CSS frontend (a stylesheet plus an icon font,
say) want to compile those bytes into the WASM binary and expose them to
the DOM via `blob:` URLs. The technique is small but fiddly: you need
`<link>` / `<style>` / `<script>` injection into `<head>`, and you need
to wrap `include_bytes!` data in a `Blob` to get a URL. This crate
packages those pieces so each downstream crate does not have to
re-implement them.

Targets `wasm32-unknown-unknown` only.

## Layers

- `head::append_link`, `head::append_style`, `head::append_script` —
  raw DOM injection into `<head>`.
- `blob::create_blob_url` — one-off Blob URL creation from raw bytes.
- `blob::AssetRegistry` — memoised, keyed Blob URL caching across many
  lookups (e.g. one Blob URL per icon glyph).

## Example

```rust,ignore
use mogwai_embed::{append_link, append_style, create_blob_url};

const MY_CSS: &str = include_str!("../assets/my.css");
const LOGO_PNG: &[u8] = include_bytes!("../assets/logo.png");

// Inject a stylesheet (no network).
append_style(MY_CSS);

// Or a <link> with a runtime Blob URL.
let logo_url = create_blob_url(LOGO_PNG, "image/png");
append_link(&logo_url);
```

## Memoised lookups

For callers that need many Blob URLs of the same shape (one per icon
glyph, for example), `AssetRegistry` collapses repeated lookups to a
single Blob URL per key:

```rust,ignore
use std::sync::OnceLock;
use mogwai_embed::blob::AssetRegistry;

static ICON_REGISTRY: OnceLock<AssetRegistry<&'static str>> = OnceLock::new();

fn icon_url(name: &'static str, bytes: &'static [u8]) -> String {
    ICON_REGISTRY
        .get_or_init(AssetRegistry::new)
        .get_or_insert(name, bytes, "image/png")
}
```

## License

Same as the parent workspace.
