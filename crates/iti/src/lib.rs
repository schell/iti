//! # iti
//!
//! A small mogwai WASM UI component library built on Bootstrap 5.
//!
//! ## Components
//!
//! - [`components::alert::Alert`] -- Bootstrap alert with reactive flavor and text
//! - [`components::button::Button`] -- Button with icon, spinner, enable/disable
//! - [`components::icon::Icon`] -- Font Awesome icon with glyph/size/classes
//! - [`components::list::List`] -- Generic clickable list (Bootstrap list-group)
//! - [`components::pane::Panes`] -- Static tab content container
//! - [`components::pane::RestartPanes`] -- Factory-based tab content container
//! - [`components::tab::TabList`] -- Bootstrap nav-tabs
//! - [`components::widget::Widget`] -- Generic element + event stream container
//!
//! ## Patterns
//!
//! - All components use `#[derive(ViewChild)]` with `V: View` generics
//! - Reactive state via `Proxy<T>`
//! - Async event loop via `step()` methods
//! - Capabilities traits for abstracting side effects
//! - `#[cfg(feature = "library")]` sandbox modules for isolated development

use mogwai::web::prelude::*;
use wasm_bindgen::prelude::*;

pub mod components;
pub mod storage;

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

    library::main().await;
}
