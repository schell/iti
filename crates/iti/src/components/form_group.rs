//! Form group wrapper component.
//!
//! Provides a form group layout that wraps a validatable input component
//! with a label, help text, and automatic validation feedback display. Handles ARIA
//! associations for accessibility.

use std::sync::atomic::{AtomicU32, Ordering};

use mogwai::prelude::*;

use super::Validatable;

/// Generate a unique ID for form elements.
fn generate_id(prefix: &str) -> String {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{}", prefix, id)
}

/// Where the label appears relative to the input within a [`FormGroup`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum LabelPlacement {
    /// Label stacked above the input (default, conventional form layout).
    #[default]
    Above,
    /// Label stacked below the input.
    Below,
    /// Label and input on the same horizontal row.
    Inline,
    /// Floating label that overlays the input (Bootstrap `form-floating` style).
    Floating,
}

impl LabelPlacement {
    fn wrapper_class(&self) -> &'static str {
        match self {
            LabelPlacement::Above => "mb-3",
            LabelPlacement::Below => "d-flex flex-column-reverse mb-3",
            LabelPlacement::Inline => "d-flex align-items-center gap-2 mb-3",
            LabelPlacement::Floating => "form-floating mb-3",
        }
    }

    fn label_class(&self) -> &'static str {
        match self {
            LabelPlacement::Inline => "form-label flex-shrink-0 mb-0",
            LabelPlacement::Floating => "form-label",
            _ => "form-label",
        }
    }

    fn label_style(&self) -> &'static str {
        match self {
            LabelPlacement::Inline => "min-width: 8em",
            _ => "",
        }
    }
}

/// A form group wrapper that provides label, help text, and validation feedback.
///
/// Wraps any component implementing [`Validatable`] with a form group structure,
/// including proper ARIA associations for accessibility. Automatically displays validation
/// errors when the child input has been validated.
///
/// Use [`set_label_placement`](Self::set_label_placement) to control whether the
/// label appears above (default), below, inline with, or floating over the input.
///
/// # Example
///
/// ```ignore
/// let mut input = TextInput::<V>::new(TextInputType::Email, "");
/// input.set_required(true);
/// let group = FormGroup::new("Email Address", input);
/// group.set_help_text("We'll never share your email.");
/// group.set_label_placement(LabelPlacement::Inline);
/// ```
#[derive(ViewChild)]
pub struct FormGroup<V: View, C: ViewChild<V> + Validatable<V>> {
    #[child]
    wrapper: V::Element,
    child: C,
    placement: Proxy<LabelPlacement>,
    required_indicator: Proxy<bool>,
    help_text: Proxy<String>,
    error_text: Proxy<String>,
    error_visible: Proxy<bool>,
}

impl<V: View, C> FormGroup<V, C>
where
    C: ViewChild<V> + Validatable<V>,
{
    /// Create a new form group with a label and child input.
    ///
    /// The default label placement is [`LabelPlacement::Above`].
    ///
    /// # Arguments
    ///
    /// * `label` — the label text for the input
    /// * `child` — the validatable input component to wrap
    pub fn new(label: impl AsRef<str>, child: C) -> Self {
        let mut placement = Proxy::new(LabelPlacement::Above);
        let mut required_indicator = Proxy::new(false);
        let mut help_text = Proxy::new(String::new());
        let mut error_text = Proxy::new(String::new());
        let mut error_visible = Proxy::new(false);

        let input_id = generate_id("input");
        let help_id = generate_id("help");
        let error_id = generate_id("error");

        // Set the child's ID and aria-describedby attributes
        child.set_id(&input_id);
        child.set_aria_describedby(format!("{help_id} {error_id}"));

        rsx! {
            let wrapper = div(
                class = placement(p => p.wrapper_class())
            ) {
                let label_elem = label(
                    class = placement(p => p.label_class()),
                    style = placement(p => p.label_style()),
                    r#for = input_id
                ) {
                    {label.into_text::<V>()}
                    span(
                        class = "text-danger ms-1",
                        style:display = required_indicator(req => if *req { "inline" } else { "none" })
                    ) { "*" }
                }

                {&child}

                let help_text_elem = div(
                    id = help_id,
                    class = "form-text"
                ) {
                    {help_text(s => s.clone())}
                }

                let error_elem = div(
                    id = error_id,
                    class = "invalid-feedback",
                    style:display = error_visible(visible => if *visible { "block" } else { "none" })
                ) {
                    {error_text(s => s.clone())}
                }
            }
        }

        Self {
            wrapper,
            child,
            placement,
            required_indicator,
            help_text,
            error_text,
            error_visible,
        }
    }

    /// Set the help text (small text displayed below the input).
    ///
    /// Help text provides additional context or instructions for the user.
    pub fn set_help_text(&mut self, text: impl AsRef<str>) {
        self.help_text.set(text.as_ref().into());
    }

    /// Set where the label appears relative to the input.
    ///
    /// See [`LabelPlacement`] for the available options.
    pub fn set_label_placement(&mut self, placement: LabelPlacement) {
        self.placement.set(placement);
    }

    /// Show or hide the required indicator (asterisk) next to the label.
    pub fn set_required_indicator(&mut self, show: bool) {
        self.required_indicator.set(show);
    }

    /// Update the validation display based on the child input's current state.
    ///
    /// Call this after the child input emits a blur event to update the
    /// error message display.
    pub fn update_validation(&mut self) {
        if self.child.validation_attempted() {
            if let Some(msg) = self.child.validation_message() {
                self.error_text.set(msg);
                self.error_visible.set(true);
            } else {
                self.error_text.set(String::new());
                self.error_visible.set(false);
            }
        } else {
            self.error_visible.set(false);
        }
    }

    /// Get a reference to the child input component.
    pub fn child(&self) -> &C {
        &self.child
    }

    /// Get a mutable reference to the child input component.
    pub fn child_mut(&mut self) -> &mut C {
        &mut self.child
    }
}
