//! # mogwai-embed
//!
//! Helpers for embedding static assets (CSS, fonts, images, scripts) into
//! a [mogwai](https://crates.io/crates/mogwai)-based WASM binary, with no
//! network requests at runtime.
//!
//! Most crates that wrap a JS/CSS frontend (e.g. a Bootstrap-style stylesheet
//! plus an icon font) want to compile those bytes into the WASM binary and
//! expose them to the DOM via `blob:` URLs. This crate packages the small
//! pieces of that technique — `<head>` injection and Blob URL creation — so
//! each downstream crate does not have to re-implement them.
//!
//! Three layers of API:
//!
//! - [`head::append_link`], [`head::append_style`], [`head::append_script`]
//!   for raw DOM injection into `<head>`.
//! - [`blob::create_blob_url`] for one-off Blob URL creation.
//! - [`blob::AssetRegistry`] for memoised, keyed Blob URL caching across
//!   many lookups (e.g. one Blob URL per icon glyph).
//!
//! ## Example
//!
//! ```ignore
//! use mogwai_embed::{append_link, append_style, create_blob_url};
//!
//! const MY_CSS: &str = include_str!("../assets/my.css");
//! const LOGO_PNG: &[u8] = include_bytes!("../assets/logo.png");
//!
//! // Inject a stylesheet (no network).
//! append_style(MY_CSS);
//!
//! // Or a <link> with a runtime Blob URL.
//! let logo_url = create_blob_url(LOGO_PNG, "image/png");
//! append_link(&logo_url);
//! ```
//!
//! This crate targets `wasm32-unknown-unknown` only.

pub mod blob;
pub mod head;
