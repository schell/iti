//! Reusable UI components built on mogwai and Bootstrap 5.

pub mod alert;
pub mod badge;
pub mod button;
pub mod button_group;
pub mod card;
pub mod dropdown;
pub mod icon;
pub mod list;
pub mod modal;
pub mod pane;
pub mod progress;
pub mod tab;
pub mod toast;
pub mod widget;

/// Bootstrap 5 color variant.
///
/// Maps to Bootstrap's contextual class suffixes (e.g. `btn-primary`,
/// `alert-danger`, `list-group-item-success`).
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
