//! Blob URL creation and caching.
//!
//! `create_blob_url` builds a `blob:` URL from raw bytes. This is the
//! runtime counterpart to `include_bytes!`/`include_str!` — a way to
//! expose compile-time-embedded assets to the DOM without a network
//! request.
//!
//! [`AssetRegistry`] is a thin memoising wrapper for callers that need
//! many lookups of the same shape (e.g. one Blob URL per icon glyph).
//! Each unique key produces exactly one Blob URL for the lifetime of
//! the page; subsequent lookups return the cached URL.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Mutex;

use js_sys::wasm_bindgen::UnwrapThrowExt;

/// Create a `blob:` URL from raw bytes with the given MIME type.
///
/// The resulting URL is valid for the lifetime of the page. It does
/// not need to be revoked for assets that live forever (e.g. fonts
/// referenced from `@font-face`).
pub fn create_blob_url(bytes: &[u8], mime_type: &str) -> String {
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

/// A memoising registry that maps caller-defined keys to Blob URLs.
///
/// Each unique key produces exactly one Blob URL for the lifetime of
/// the page. Useful for repeating asset lookups (e.g. one per icon
/// glyph, one per sort-arrow SVG).
///
/// The registry is internally a `Mutex<HashMap>`. Calls are write-once,
/// read-many in practice: a given key is set on first lookup and read
/// from then on.
pub struct AssetRegistry<K>
where
    K: Eq + Hash,
{
    map: Mutex<HashMap<K, String>>,
}

impl<K> AssetRegistry<K>
where
    K: Eq + Hash,
{
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
        }
    }

    /// Get-or-create a Blob URL for `bytes` keyed by `key`.
    ///
    /// If the key is already present, the cached URL is returned
    /// without inspecting `bytes` or `mime_type` — so make sure the
    /// same key always maps to the same bytes.
    pub fn get_or_insert(&self, key: K, bytes: &[u8], mime_type: &str) -> String
    where
        K: Clone,
    {
        let mut map = self.map.lock().unwrap();
        if let Some(url) = map.get(&key) {
            return url.clone();
        }
        let url = create_blob_url(bytes, mime_type);
        map.insert(key, url.clone());
        url
    }

    /// Check whether a key has been registered already.
    pub fn contains(&self, key: &K) -> bool {
        self.map.lock().unwrap().contains_key(key)
    }

    /// Number of distinct Blob URLs created so far.
    pub fn len(&self) -> usize {
        self.map.lock().unwrap().len()
    }

    /// `true` if no Blob URLs have been created yet.
    pub fn is_empty(&self) -> bool {
        self.map.lock().unwrap().is_empty()
    }
}

impl<K> Default for AssetRegistry<K>
where
    K: Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}
