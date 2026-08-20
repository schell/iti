//! Traits connecting form data structs to their generated UI components.
//!
//! When a struct is annotated with `#[derive(iti::Form)]`, the derive macro
//! generates a companion `*Component<V>` type and implements both [`Form`] and
//! [`FormComponent`] to link them:
//!
//! ```ignore
//! #[derive(iti::Form)]
//! struct LoginForm {
//!     #[form(input_type = "email", required)]
//!     email: String,
//!
//!     #[form(input_type = "password", required)]
//!     password: String,
//!
//!     #[form(label = "Remember me")]
//!     remember_me: bool,
//! }
//! ```
//!
//! The macro generates `LoginFormComponent<V: View>` and wires up:
//!
//! - `impl Form for LoginForm` — associates the data struct with its component.
//! - `impl<V: View> FormComponent<V> for LoginFormComponent<V>` — provides
//!   [`FormComponent::try_value`] for collecting the data and
//!   [`StepMut`](mogwai::step::StepMut) for the pull-based event loop.
//!
//! ## Driving a form
//!
//! The generated component implements `StepMut<Output = FormEvent>`, so the
//! caller drives it in a `loop { component.step_mut().await }` pattern. Each
//! call processes one field interaction (input change or blur) and returns a
//! [`FormEvent`] describing what happened.
//!
//! ## Collecting form data
//!
//! Call [`FormComponent::try_value`] at any time (e.g. on a submit button
//! click) to collect the current field values into the original struct. It
//! returns `Ok(Self::Data)` when all required fields pass validation, or
//! `Err(Vec<FormError>)` listing every failure.

use std::fmt;

use mogwai::{step::StepMut, view::View};

/// Errors that can occur when collecting or validating form data.
///
/// Each variant identifies the failing field by name so the caller can
/// display targeted feedback.
#[derive(Debug, Clone)]
pub enum FormError {
    /// A required field was left empty.
    RequiredFieldEmpty { field: String },

    /// A field failed browser-native HTML5 constraint validation
    /// (e.g. invalid email format, below `min_length`, bad pattern).
    ValidationFailed { field: String, message: String },

    /// A field value could not be parsed into the target type.
    ParseError { field: String, message: String },

    /// Any other form-related error not covered by the variants above.
    Other(String),
}

impl fmt::Display for FormError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormError::RequiredFieldEmpty { field } => write!(f, "Required field empty: {}", field),
            FormError::ValidationFailed { field, message } => {
                write!(f, "Validation failed for {}: {}", field, message)
            }
            FormError::ParseError { field, message } => {
                write!(f, "Failed to parse {}: {}", field, message)
            }
            FormError::Other(msg) => write!(f, "Form error: {}", msg),
        }
    }
}

impl std::error::Error for FormError {}

/// A typed value extracted from a form field.
///
/// Produced by [`FormEvent::FieldChanged`] so the caller can match on the
/// value kind without downcasting.
#[derive(Debug, Clone)]
pub enum FormValue {
    /// A text-based field value (text input, email, password, textarea, etc.).
    String(String),

    /// A boolean field value (checkbox).
    Bool(bool),
}

/// Events emitted by a form component's [`StepMut`] event loop.
///
/// Each call to `step_mut()` processes one field interaction and returns
/// the corresponding event. The caller can inspect these to update UI,
/// log activity, or decide when to call [`FormComponent::try_value`].
#[derive(Debug, Clone)]
pub enum FormEvent {
    /// A field's value changed (user typed, toggled a checkbox, etc.).
    ///
    /// Includes the field name, the new typed value, and whether the
    /// field currently passes browser-native validation.
    FieldChanged {
        field: String,
        value: FormValue,
        valid: bool,
    },

    /// A field lost focus, which triggers validation feedback display.
    FieldBlur { field: String },
}

/// Marks a struct as form data that has an associated generated component.
///
/// Implemented automatically by `#[derive(iti::Form)]`. The associated type
/// [`Component`](Self::Component) is the generated `*Component<V>` type that
/// renders the form and drives its event loop.
///
/// The `Form` → `FormComponent` link is bidirectional: `Form::Component`
/// points to the component, and `FormComponent::Data` points back to the
/// data struct. The type system enforces that the two agree.
pub trait Form {
    /// The generated component type that renders and drives this form.
    type Component<V: View>: FormComponent<V>;
}

/// The API for a generated form component.
///
/// Implemented automatically by `#[derive(iti::Form)]` on the generated
/// `*Component<V>` type. Combines:
///
/// - [`StepMut`] with `Output = FormEvent` — pull-based event loop that
///   processes one field interaction per `step_mut()` call.
/// - [`try_value`](Self::try_value) — collects all current field values
///   into the original struct type, or returns all validation errors.
///
/// The associated type [`Data`](Self::Data) is the original struct that
/// `#[derive(Form)]` was applied to.
pub trait FormComponent<V: View>: StepMut<Output = FormEvent> {
    /// The form data struct this component renders.
    type Data: Form<Component<V> = Self>;

    /// Collect the current field values into the form data struct.
    ///
    /// Returns `Ok(Self::Data)` when all required fields pass validation.
    /// Returns `Err(Vec<FormError>)` listing **every** validation failure
    /// (not just the first) so the caller can display all errors at once.
    ///
    /// Safe to call at any time — it does not consume or reset the form.
    fn try_value(&self) -> Result<Self::Data, Vec<FormError>>;
}
