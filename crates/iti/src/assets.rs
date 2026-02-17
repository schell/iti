//! Static asset helpers for Bootstrap 5, Bootstrap Icons, and Font Awesome 6.
//!
//! Provides three ways for consumers to load the CSS and fonts that iti
//! components depend on:
//!
//! 1. **CDN links** — [`inject_cdn_links`] adds `<link>` tags pointing to
//!    public CDNs. Simplest approach; requires an internet connection.
//!
//! 2. **Embedded CSS** — With the `embed-assets` feature enabled,
//!    [`embedded::inject_styles`] injects Bootstrap's CSS directly as a
//!    `<style>` element (no network fetch for Bootstrap). Icon font
//!    stylesheets are still loaded from CDN because their CSS references
//!    font files via relative URLs that only resolve correctly when served
//!    from the CDN origin.
//!
//! 3. **Manual / Trunk** — Consumers can ignore this module entirely and
//!    wire up assets themselves (e.g. with Trunk `data-trunk` directives
//!    or plain `<link>` tags in their `index.html`).

use js_sys::wasm_bindgen::{JsCast, UnwrapThrowExt};

/// Custom iti styles (always embedded — only a few bytes).
pub const ITI_CSS: &str = include_str!("../../../assets/style.css");

/// CDN URLs for the stylesheets iti depends on.
///
/// These point to the same versions that are vendored in the `assets/`
/// directory of the iti repository.
pub mod cdn {
    /// Bootstrap 5.3.3 minified CSS.
    pub const BOOTSTRAP_CSS: &str =
        "https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css";

    /// Bootstrap Icons 1.13.1 CSS (includes `@font-face` for icon fonts).
    pub const BOOTSTRAP_ICONS_CSS: &str =
        "https://cdn.jsdelivr.net/npm/bootstrap-icons@1.13.1/font/bootstrap-icons.min.css";

    /// Font Awesome 6.6.0 Free — all styles (includes `@font-face` for
    /// Solid, Regular, and Brands webfonts).
    pub const FONTAWESOME_CSS: &str =
        "https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.6.0/css/all.min.css";
}

/// Append a `<link rel="stylesheet">` element to `<head>`.
fn append_link(href: &str) {
    let document = web_sys::window().unwrap_throw().document().unwrap_throw();
    let head = document.head().unwrap_throw();
    let link = document
        .create_element("link")
        .unwrap_throw()
        .unchecked_into::<web_sys::HtmlLinkElement>();
    link.set_rel("stylesheet");
    link.set_href(href);
    head.append_child(&link).unwrap_throw();
}

/// Append a `<style>` element with the given CSS text to `<head>`.
fn append_style(css: &str) {
    let document = web_sys::window().unwrap_throw().document().unwrap_throw();
    let head = document.head().unwrap_throw();
    let style = document
        .create_element("style")
        .unwrap_throw()
        .unchecked_into::<web_sys::HtmlStyleElement>();
    style.set_text_content(Some(css));
    head.append_child(&style).unwrap_throw();
}

/// Inject all required stylesheets as CDN `<link>` tags.
///
/// Creates four elements in `<head>`:
/// - `<link>` for Bootstrap 5 CSS
/// - `<link>` for Bootstrap Icons CSS (with fonts)
/// - `<link>` for Font Awesome 6 CSS (with fonts)
/// - `<style>` for iti's own custom styles
///
/// This is the simplest setup — one function call and you're done.
/// Requires an internet connection to reach the CDNs.
pub fn inject_cdn_links() {
    append_link(cdn::BOOTSTRAP_CSS);
    append_link(cdn::BOOTSTRAP_ICONS_CSS);
    append_link(cdn::FONTAWESOME_CSS);
    append_style(ITI_CSS);
}

/// Embedded assets — available when the `embed-assets` feature is enabled.
///
/// Bootstrap CSS is compiled into the binary and injected as a `<style>`
/// element, avoiding a network round-trip for the largest stylesheet.
/// Icon font stylesheets (Bootstrap Icons, Font Awesome) are still loaded
/// from CDN because their CSS contains relative `@font-face url()`
/// references that only resolve when served from the original CDN origin.
#[cfg(feature = "embed-assets")]
pub mod embedded {
    use super::*;

    /// Bootstrap 5.3.3 minified CSS, embedded at compile time.
    pub const BOOTSTRAP_CSS: &str = include_str!("../../../assets/bootstrap.min.css");

    /// Inject all required styles with Bootstrap CSS embedded.
    ///
    /// Creates four elements in `<head>`:
    /// - `<style>` containing the full Bootstrap 5 CSS (no network fetch)
    /// - `<link>` for Bootstrap Icons CSS from CDN (needs fonts)
    /// - `<link>` for Font Awesome 6 CSS from CDN (needs fonts)
    /// - `<style>` for iti's own custom styles
    pub fn inject_styles() {
        append_style(BOOTSTRAP_CSS);
        append_link(cdn::BOOTSTRAP_ICONS_CSS);
        append_link(cdn::FONTAWESOME_CSS);
        append_style(ITI_CSS);
    }
}
