//! Static asset helpers for iti's custom CSS and Font Awesome 6.
//!
//! Provides three ways for consumers to load the CSS and fonts that iti
//! components depend on:
//!
//! 1. **CDN links** — [`inject_cdn_links`] adds a `<link>` tag for Font
//!    Awesome from a public CDN and injects iti's CSS as a `<style>` tag.
//!    Requires an internet connection for the icon font.
//!
//! 2. **Fully embedded** — With the `embed-assets` feature enabled,
//!    [`embedded::inject_styles`] injects all CSS and fonts directly from
//!    the WASM binary. No network connection required. Fonts are served
//!    via Blob URLs created at runtime from compiled-in woff2/ttf bytes.
//!
//! 3. **Manual / Trunk** — Consumers can ignore this module entirely and
//!    wire up assets themselves (e.g. with Trunk `data-trunk` directives
//!    or plain `<link>` tags in their `index.html`).

use js_sys::wasm_bindgen::{JsCast, UnwrapThrowExt};

/// iti's unified stylesheet (always embedded — includes all component styles).
pub const ITI_CSS: &str = include_str!("../../../assets/iti.css");

/// CDN URLs for external dependencies.
///
/// Only Font Awesome is loaded from a CDN. All other styles are provided
/// by `iti.css`.
pub mod cdn {
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

/// Inject the design token CSS custom properties as a `<style>` element.
///
/// This must be called **before** `iti.css` is loaded (whether via `<link>`
/// or `<style>`) so that the token variables (`--azul`, `--gray300`, etc.)
/// are available when the semantic aliases resolve their `var()` references.
///
/// Called automatically by [`inject_cdn_links`] and
/// [`embedded::inject_styles`]. For the Trunk/manual loading path, call
/// this from your WASM entry point before the stylesheet `<link>` loads.
pub fn inject_color_tokens() {
    append_style(crate::color::CSS_TOKENS);
}

/// Inject all required stylesheets using a CDN `<link>` for Font Awesome.
///
/// Creates three elements in `<head>`:
/// - `<style>` for color token CSS custom properties
/// - `<link>` for Font Awesome 6 CSS (with fonts from CDN)
/// - `<style>` for iti's unified stylesheet
///
/// Requires an internet connection to reach the Font Awesome CDN.
pub fn inject_cdn_links() {
    inject_color_tokens();
    append_link(cdn::FONTAWESOME_CSS);
    append_style(ITI_CSS);
}

/// Fully embedded assets — available when the `embed-assets` feature is
/// enabled.
///
/// All CSS and icon fonts are compiled into the WASM binary. At runtime,
/// fonts are exposed via Blob URLs and the CSS `@font-face` declarations
/// are rewritten to reference those URLs before injection. No network
/// connection is required.
///
/// **Binary size cost:** approximately 1.2 MB (CSS + woff2/ttf fonts).
/// Only woff2 fonts are included for Font Awesome — all WASM-capable
/// browsers support woff2. Font Awesome Brands icons are excluded to
/// save space; only Solid, Regular, and v4-compatibility fonts are
/// embedded.
#[cfg(feature = "embed-assets")]
pub mod embedded {
    use js_sys::wasm_bindgen::UnwrapThrowExt;

    use super::*;

    // ── CSS ──────────────────────────────────────────────────────

    /// Font Awesome 6.6.0 Free minified CSS, embedded at compile time.
    ///
    /// The `@font-face` URLs are rewritten at runtime by
    /// [`inject_styles`] to point at Blob URLs.
    const FONTAWESOME_CSS: &str = include_str!("../../../assets/fontawesome/css/all.min.css");

    // ── Fonts (woff2) ──────────────────────────────────────

    const FA_SOLID_WOFF2: &[u8] =
        include_bytes!("../../../assets/fontawesome/webfonts/fa-solid-900.woff2");
    const FA_REGULAR_WOFF2: &[u8] =
        include_bytes!("../../../assets/fontawesome/webfonts/fa-regular-400.woff2");
    const FA_V4COMPAT_WOFF2: &[u8] =
        include_bytes!("../../../assets/fontawesome/webfonts/fa-v4compatibility.woff2");
    // -- Fonts (ttf)
    const CHICAGO_TTF: &[u8] = include_bytes!("../../../assets/fonts/ChicagoFLF.ttf");
    const GENEVA_TTF: &[u8] = include_bytes!("../../../assets/fonts/Geneva.ttf");
    const GARAMOND_LIGHT_TTF: &[u8] =
        include_bytes!("../../../assets/fonts/AppleGaramond-Light.ttf");
    const GARAMOND_REGULAR_TTF: &[u8] = include_bytes!("../../../assets/fonts/AppleGaramond.ttf");
    const GARAMOND_BOLD_TTF: &[u8] = include_bytes!("../../../assets/fonts/AppleGaramond-Bold.ttf");
    // ── Blob URL helper ─────────────────────────────────────────

    /// Create a `blob:` URL from raw bytes with the given MIME type.
    ///
    /// The resulting URL is valid for the lifetime of the page. It does
    /// not need to be revoked for fonts that live forever.
    fn create_blob_url(bytes: &[u8], mime_type: &str) -> String {
        let uint8_array = js_sys::Uint8Array::new_with_length(bytes.len() as u32);
        uint8_array.copy_from(bytes);

        let parts = js_sys::Array::new();
        parts.push(&uint8_array);

        let options = web_sys::BlobPropertyBag::new();
        options.set_type(mime_type);

        let blob =
            web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &options).unwrap_throw();

        web_sys::Url::create_object_url_with_blob(&blob).unwrap_throw()
    }

    // ── CSS rewriting ───────────────────────────────────────────

    /// Rewrite Font Awesome CSS to use Blob URLs for embedded fonts.
    ///
    /// Replaces woff2 relative paths with Blob URLs and strips the
    /// ttf fallback entries (we only ship woff2).
    fn rewrite_fontawesome_css(
        css: &str,
        solid_url: &str,
        regular_url: &str,
        v4compat_url: &str,
    ) -> String {
        css
            // Replace woff2 paths with Blob URLs
            .replace("../webfonts/fa-solid-900.woff2", solid_url)
            .replace("../webfonts/fa-regular-400.woff2", regular_url)
            .replace("../webfonts/fa-v4compatibility.woff2", v4compat_url)
            // Strip ttf fallbacks (we only embed woff2)
            .replace(
                ",url(../webfonts/fa-solid-900.ttf) format(\"truetype\")",
                "",
            )
            .replace(
                ",url(../webfonts/fa-regular-400.ttf) format(\"truetype\")",
                "",
            )
            .replace(
                ",url(../webfonts/fa-brands-400.ttf) format(\"truetype\")",
                "",
            )
            .replace(
                ",url(../webfonts/fa-v4compatibility.ttf) format(\"truetype\")",
                "",
            )
    }

    /// Rewrite iti CSS to use Blob URLs for the embedded fonts.
    ///
    /// Replaces the ttf paths for Geneva, ChicagoFLF, and Apple Garamond
    /// with Blob URLs.
    fn rewrite_iti_css(
        css: &str,
        chicago_url: &str,
        geneva_url: &str,
        garamond_light_url: &str,
        garamond_regular_url: &str,
        garamond_bold_url: &str,
    ) -> String {
        css.replace(
            "url('fonts/ChicagoFLF.ttf')",
            &format!("url(\"{chicago_url}\")"),
        )
        .replace("url('fonts/Geneva.ttf')", &format!("url(\"{geneva_url}\")"))
        .replace(
            "url('fonts/AppleGaramond-Light.ttf')",
            &format!("url(\"{garamond_light_url}\")"),
        )
        .replace(
            "url('fonts/AppleGaramond.ttf')",
            &format!("url(\"{garamond_regular_url}\")"),
        )
        .replace(
            "url('fonts/AppleGaramond-Bold.ttf')",
            &format!("url(\"{garamond_bold_url}\")"),
        )
    }

    // ── Public API ──────────────────────────────────────────────

    /// Inject all required styles from the embedded assets.
    ///
    /// Creates `<style>` elements in `<head>` — no `<link>` tags,
    /// no network requests:
    ///
    /// 1. Color token CSS custom properties (from `color.rs`)
    /// 2. iti unified CSS (with `@font-face` rewritten to Blob URLs)
    /// 3. Font Awesome 6 CSS (with `@font-face` rewritten to Blob URLs)
    ///
    /// Font Awesome Brands icons are **not** embedded to save binary
    /// space. Brand icon classes (`.fa-brands`) will render as blank
    /// unless the consumer loads the Brands font separately.
    pub fn inject_styles() {
        // Create Blob URLs for each embedded font
        let fa_solid_url = create_blob_url(FA_SOLID_WOFF2, "font/woff2");
        let fa_regular_url = create_blob_url(FA_REGULAR_WOFF2, "font/woff2");
        let fa_v4compat_url = create_blob_url(FA_V4COMPAT_WOFF2, "font/woff2");
        let chicago_url = create_blob_url(CHICAGO_TTF, "font/ttf");
        let geneva_url = create_blob_url(GENEVA_TTF, "font/ttf");
        let garamond_light_url = create_blob_url(GARAMOND_LIGHT_TTF, "font/ttf");
        let garamond_regular_url = create_blob_url(GARAMOND_REGULAR_TTF, "font/ttf");
        let garamond_bold_url = create_blob_url(GARAMOND_BOLD_TTF, "font/ttf");

        // Rewrite CSS @font-face declarations to use Blob URLs
        let fa_css = rewrite_fontawesome_css(
            FONTAWESOME_CSS,
            &fa_solid_url,
            &fa_regular_url,
            &fa_v4compat_url,
        );
        let iti_css = rewrite_iti_css(
            ITI_CSS,
            &chicago_url,
            &geneva_url,
            &garamond_light_url,
            &garamond_regular_url,
            &garamond_bold_url,
        );

        // Inject everything as <style> elements — zero network requests
        inject_color_tokens();
        append_style(&iti_css);
        append_style(&fa_css);
    }
}
