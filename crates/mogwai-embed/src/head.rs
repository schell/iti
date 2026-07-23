//! DOM `<head>` injection helpers.
//!
//! Each function appends a single element to `<head>` using the global
//! `window.document.head`. All errors (e.g. no `document`, no `<head>`)
//! are surfaced via `unwrap_throw()` so they show up in the browser
//! console as a clear JavaScript exception rather than a generic panic.

use js_sys::wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{HtmlLinkElement, HtmlScriptElement, HtmlStyleElement};

fn head() -> web_sys::HtmlHeadElement {
    web_sys::window()
        .unwrap_throw()
        .document()
        .unwrap_throw()
        .head()
        .unwrap_throw()
}

/// Append a `<link rel="stylesheet">` element to `<head>`.
pub fn append_link(href: &str) {
    let document = web_sys::window().unwrap_throw().document().unwrap_throw();
    let link = document
        .create_element("link")
        .unwrap_throw()
        .unchecked_into::<HtmlLinkElement>();
    link.set_rel("stylesheet");
    link.set_href(href);
    head().append_child(&link).unwrap_throw();
}

/// Append a `<style>` element with the given CSS text to `<head>`.
pub fn append_style(css: &str) {
    let document = web_sys::window().unwrap_throw().document().unwrap_throw();
    let style = document
        .create_element("style")
        .unwrap_throw()
        .unchecked_into::<HtmlStyleElement>();
    style.set_text_content(Some(css));
    head().append_child(&style).unwrap_throw();
}

/// Append a `<script>` element to `<head>`.
///
/// `script_type` is the value for the `type` attribute (e.g. `"module"`,
/// `"text/javascript"`). Pass an empty string to omit the attribute.
pub fn append_script(src: &str, script_type: &str) {
    let document = web_sys::window().unwrap_throw().document().unwrap_throw();
    let script = document
        .create_element("script")
        .unwrap_throw()
        .unchecked_into::<HtmlScriptElement>();
    script.set_src(src);
    if !script_type.is_empty() {
        script.set_type(script_type);
    }
    head().append_child(&script).unwrap_throw();
}
