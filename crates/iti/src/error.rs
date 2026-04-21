//! Errors thrown by the iti UI.
use snafu::prelude::*;

/// Iti UI errors.
#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("{source}"))]
    Storage { source: crate::storage::Error },
}

impl From<crate::storage::Error> for Error {
    fn from(source: crate::storage::Error) -> Self {
        Error::Storage { source }
    }
}
