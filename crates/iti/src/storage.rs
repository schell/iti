//! Browser localStorage abstraction.

use snafu::{OptionExt, ResultExt};

/// Retrieve a JSON-deserialized value from localStorage.
pub fn get_item<T: serde::de::DeserializeOwned>(
    key: impl AsRef<str>,
) -> Result<Option<T>, snafu::Whatever> {
    let storage = mogwai::web::window()
        .local_storage()
        .ok()
        .whatever_context("no local storage")?
        .whatever_context("local storage null")?;
    if let Some(string) = storage
        .get_item(key.as_ref())
        .ok()
        .whatever_context("could not search for item")?
    {
        serde_json::from_str(&string).whatever_context("could not deserialize")
    } else {
        Ok(None)
    }
}

/// Serialize a value to JSON and store it in localStorage.
pub fn set_item(
    key: impl AsRef<str>,
    value: &impl serde::Serialize,
) -> Result<(), snafu::Whatever> {
    let storage = mogwai::web::window()
        .local_storage()
        .ok()
        .whatever_context("no local storage")?
        .whatever_context("local storage null")?;
    let value = serde_json::to_string(value).whatever_context("could not serialize")?;
    storage
        .set_item(key.as_ref(), &value)
        .ok()
        .whatever_context("could not store item")
}
