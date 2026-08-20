//! Reusable UI components with a Mac OS 9 Platinum aesthetic.

use mogwai::prelude::View;

pub mod alert;
pub mod badge;
pub mod button;
pub mod button_group;
pub mod card;
pub mod checkbox;
pub mod dropdown;
pub mod form_group;
pub mod icon;
pub mod icon_classic;
pub mod list;
pub mod modal;
pub mod pane;
#[cfg(feature = "library")]
pub mod platinum_kit;
pub mod progress;
pub mod radio;
pub mod section;
pub mod select;
pub mod shadow;
pub mod slider;
pub mod tab;
pub mod table;
pub mod text_input;
pub mod textarea;
pub mod title_bar;
pub mod widget;

// Re-export form-related components for use by the Form derive macro.
pub use checkbox::Checkbox;
pub use form_group::{FormGroup, LabelPlacement};
pub use text_input::{TextInput, TextInputType};
pub use textarea::Textarea;

/// Contextual color variant.
///
/// Maps to contextual class suffixes used across components (e.g.
/// `flavor-primary`, `alert-danger`, `list-group-item-success`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Flavor {
    #[default]
    Primary,
    Secondary,
    Success,
    Danger,
    Warning,
    Info,
    Light,
    Dark,
    Link,
}

impl std::fmt::Display for Flavor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.class_name())
    }
}

impl Flavor {
    pub fn class_name(&self) -> &str {
        match self {
            Flavor::Primary => "primary",
            Flavor::Secondary => "secondary",
            Flavor::Success => "success",
            Flavor::Danger => "danger",
            Flavor::Warning => "warning",
            Flavor::Info => "info",
            Flavor::Light => "light",
            Flavor::Dark => "dark",
            Flavor::Link => "link",
        }
    }
}

/// Trait for components that support HTML5 constraint validation.
///
/// Implemented by form input components ([`TextInput`],
/// [`Textarea`]) so that [`FormGroup`]
/// can query validation state and associate labels, help text, and error
/// messages via ARIA attributes.
pub trait Validatable<V: View> {
    /// Check if the input's current value is valid.
    ///
    /// Uses the browser's native HTML5 constraint validation API.
    fn is_valid(&self) -> bool;

    /// Get the current validation error message, if any.
    ///
    /// Returns `None` if the input is valid, or `Some(message)` containing the
    /// browser's native validation message.
    fn validation_message(&self) -> Option<String>;

    /// Check if validation has been triggered (i.e., user has interacted with the field).
    ///
    /// Typically set to `true` after the first blur event. Used to determine whether
    /// validation feedback should be displayed.
    fn validation_attempted(&self) -> bool;

    /// Set the input element's `id` attribute.
    ///
    /// Used by [`FormGroup`] to associate labels with inputs
    /// via the `for` attribute.
    fn set_id(&self, id: impl AsRef<str>);

    /// Set the input's `aria-describedby` attribute.
    ///
    /// Used by [`FormGroup`] to associate error messages
    /// and help text with the input for screen readers.
    fn set_aria_describedby(&self, ids: impl AsRef<str>);
}
