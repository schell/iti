//! # iti
//!
//! A small mogwai WASM UI component library with a Mac OS 9 Platinum aesthetic.
//!
//! ## Patterns
//!
//! - All components use `#[derive(ViewChild)]` with `V: View` generics
//! - Reactive state via `Proxy<T>`
//! - Async event loop via `step()` methods
//! - Capabilities traits for abstracting side effects
//! - `#[cfg(feature = "library")]` sandbox modules for isolated development

// Allow proc macros (which generate `::iti::` paths) to reference this crate
// when used from within itself.
extern crate self as iti;

use mogwai::web::prelude::*;
use wasm_bindgen::prelude::*;

pub mod assets;
pub mod color;
pub mod components;
pub mod error;
pub mod form_traits;
pub mod id;
pub mod storage;

pub use iti_derive::Form;

/// Prelude module with common imports for iti users.
pub mod prelude {
    pub use crate::form_traits::{Form, FormComponent, FormError, FormEvent};
    pub use mogwai::prelude::*;
}

#[cfg(feature = "library")]
mod library;

#[cfg(feature = "library")]
#[wasm_bindgen(start)]
pub async fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    fern::Dispatch::new()
        .level(log::LevelFilter::Trace)
        .chain(fern::Output::call(console_log::log))
        .apply()
        .unwrap();

    // Inject color token CSS custom properties before iti.css resolves
    // its var() references. The CDN and embedded paths do this
    // automatically, but the Trunk path loads iti.css via <link> tags
    // so we need to inject the tokens explicitly.
    assets::inject_color_tokens();

    library::main().await;
}
