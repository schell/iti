//! Browser localStorage abstraction.

use snafu::{prelude::*, OptionExt, ResultExt};

/// All storage errors.
#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("DOM API unavailable"))]
    Dom,
    #[snafu(display("No local storage"))]
    Storage,
    #[snafu(display("Could not search"))]
    Search,
    #[snafu(display("Error parsing storage value for key:'{key}' value:'{value}': {source}"))]
    Parse {
        key: String,
        value: String,
        source: serde_json::Error,
    },
    #[snafu(display("Error serializing storage value for key:'{key}': {source}"))]
    Serialize {
        key: String,
        source: serde_json::Error,
    },
    #[snafu(display("Cannot store value for key:'{key}'"))]
    Store { key: String },
}

fn get_storage() -> Result<web_sys::Storage, Error> {
    let window = web_sys::window().context(DomSnafu)?;
    window
        .local_storage()
        .ok()
        .context(StorageSnafu)?
        .context(StorageSnafu)
}

/// Retrieve a JSON-deserialized value from localStorage.
pub fn get_item<T: serde::de::DeserializeOwned>(key: impl AsRef<str>) -> Result<Option<T>, Error> {
    let storage = get_storage()?;
    if let Some(string) = storage.get_item(key.as_ref()).ok().context(SearchSnafu)? {
        serde_json::from_str(&string).with_context(|_| ParseSnafu {
            key: key.as_ref().to_string(),
            value: string,
        })
    } else {
        Ok(None)
    }
}

/// Serialize a value to JSON and store it in localStorage.
pub fn set_item(key: impl AsRef<str>, value: &impl serde::Serialize) -> Result<(), Error> {
    let storage = get_storage()?;
    let value = serde_json::to_string(value).with_context(|_| SerializeSnafu {
        key: key.as_ref().to_string(),
    })?;
    storage
        .set_item(key.as_ref(), &value)
        .ok()
        .with_context(|| StoreSnafu {
            key: key.as_ref().to_string(),
        })
}
