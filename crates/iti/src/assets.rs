//! Static asset helpers for Bootstrap 5, Bootstrap Icons, and Font Awesome 6.
//!
//! Provides three ways for consumers to load the CSS and fonts that iti
//! components depend on:
//!
//! 1. **CDN links** — [`inject_cdn_links`] adds `<link>` tags pointing to
//!    public CDNs. Simplest approach; requires an internet connection.
//!
//! 2. **Fully embedded** — With the `embed-assets` feature enabled,
//!    [`embedded::inject_styles`] injects all CSS and fonts directly from
//!    the WASM binary. No network connection required. Fonts are served
//!    via Blob URLs created at runtime from compiled-in woff2 bytes.
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

/// Fully embedded assets — available when the `embed-assets` feature is
/// enabled.
///
/// All CSS and icon fonts are compiled into the WASM binary. At runtime,
/// fonts are exposed via Blob URLs and the CSS `@font-face` declarations
/// are rewritten to reference those URLs before injection. No network
/// connection is required.
///
/// **Binary size cost:** approximately 720 KB (CSS + woff2 fonts). Only
/// woff2 fonts are included — all WASM-capable browsers support woff2.
/// Font Awesome Brands icons are excluded to save space; only Solid,
/// Regular, and v4-compatibility fonts are embedded.
#[cfg(feature = "embed-assets")]
pub mod embedded {
    use js_sys::wasm_bindgen::UnwrapThrowExt;

    use super::*;

    // ── CSS ──────────────────────────────────────────────────────

    /// System9 (retro) styles.
    pub const SYSTEM9_CSS: &str = include_str!("../../../assets/system9.css");

    /// Bootstrap 5.3.3 minified CSS, embedded at compile time.
    pub const BOOTSTRAP_CSS: &str = include_str!("../../../assets/bootstrap.min.css");

    /// Bootstrap Icons 1.13.1 minified CSS, embedded at compile time.
    ///
    /// The `@font-face` URLs are rewritten at runtime by
    /// [`inject_styles`] to point at Blob URLs.
    const BOOTSTRAP_ICONS_CSS: &str = include_str!("../../../assets/bootstrap-icons.min.css");

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
    const BOOTSTRAP_ICONS_WOFF2: &[u8] =
        include_bytes!("../../../assets/fonts/bootstrap-icons.woff2");

    // -- Fonts (ttf)
    const CHICAGO_TTF: &[u8] = include_bytes!("../../../assets/fonts/ChicagoFLF.ttf");

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

    /// Rewrite Bootstrap Icons CSS to use a Blob URL for the embedded
    /// font.
    ///
    /// Replaces the woff2 path with a Blob URL and strips the woff
    /// fallback.
    fn rewrite_bootstrap_icons_css(css: &str, woff2_url: &str) -> String {
        css.replace(
            "url(\"fonts/bootstrap-icons.woff2?e34853135f9e39acf64315236852cd5a\")",
            &format!("url(\"{woff2_url}\")"),
        )
        .replace(
            ",url(\"fonts/bootstrap-icons.woff?e34853135f9e39acf64315236852cd5a\") format(\"woff\")",
            "",
        )
    }

    /// Rewrite system-9 fonts to use a Blob URL for the embedded font.
    ///
    /// Replaces the ttf path with a Blob URL.
    fn rewrite_system_9_css(css: &str, chicago_url: &str) -> String {
        css.replace(
            "url('fonts/ChicagoFLF.ttf')",
            &format!("url(\"{chicago_url}\")"),
        )
    }

    // ── Public API ──────────────────────────────────────────────

    /// Inject all required styles from the embedded assets.
    ///
    /// Creates four `<style>` elements in `<head>` — no `<link>` tags,
    /// no network requests:
    ///
    /// 1. Bootstrap 5 CSS
    /// 2. Bootstrap Icons CSS (with `@font-face` rewritten to Blob URLs)
    /// 3. Font Awesome 6 CSS (with `@font-face` rewritten to Blob URLs)
    /// 4. iti custom styles
    ///
    /// Font Awesome Brands icons are **not** embedded to save binary
    /// space. Brand icon classes (`.fa-brands`) will render as blank
    /// unless the consumer loads the Brands font separately.
    pub fn inject_styles() {
        // Create Blob URLs for each embedded font
        let fa_solid_url = create_blob_url(FA_SOLID_WOFF2, "font/woff2");
        let fa_regular_url = create_blob_url(FA_REGULAR_WOFF2, "font/woff2");
        let fa_v4compat_url = create_blob_url(FA_V4COMPAT_WOFF2, "font/woff2");
        let bi_url = create_blob_url(BOOTSTRAP_ICONS_WOFF2, "font/woff2");
        let chicago_url = create_blob_url(CHICAGO_TTF, "font/ttf");

        // Rewrite CSS @font-face declarations to use Blob URLs
        let fa_css = rewrite_fontawesome_css(
            FONTAWESOME_CSS,
            &fa_solid_url,
            &fa_regular_url,
            &fa_v4compat_url,
        );
        let bi_css = rewrite_bootstrap_icons_css(BOOTSTRAP_ICONS_CSS, &bi_url);
        let system9 = rewrite_system_9_css(SYSTEM9_CSS, &chicago_url);

        // Inject everything as <style> elements — zero network requests
        append_style(BOOTSTRAP_CSS);
        append_style(&bi_css);
        append_style(&fa_css);
        append_style(&system9);
        append_style(ITI_CSS);
    }
}
