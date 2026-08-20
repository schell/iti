//! Text input component.
//!
//! Wraps a native HTML `<input>` element with support for various text-based input types
//! (text, email, password, tel, url, search). Provides HTML5 validation, reactive state,
//! and a pull-based async event model.

use futures_lite::FutureExt;
use mogwai::future::MogwaiFutureExt;
use mogwai::prelude::*;
use mogwai::web::WebElement;
use web_sys::HtmlInputElement;

use super::Validatable;

/// Event produced by TextInput.
pub enum TextInputEvent<V: View> {
    /// User typed or changed the value.
    Input {
        /// The current input value.
        value: String,
        /// The raw DOM event.
        event: V::Event,
    },
    /// User left the field (triggers validation).
    Blur {
        /// The raw DOM event.
        event: V::Event,
    },
}

/// Input type for text-based inputs.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum TextInputType {
    /// Standard text input.
    #[default]
    Text,
    /// Email address input with browser validation.
    Email,
    /// Password input (masked characters).
    Password,
    /// Telephone number input.
    Tel,
    /// URL input with browser validation.
    Url,
    /// Search input (may include browser-specific styling).
    Search,
}

impl TextInputType {
    fn as_str(&self) -> &str {
        match self {
            TextInputType::Text => "text",
            TextInputType::Email => "email",
            TextInputType::Password => "password",
            TextInputType::Tel => "tel",
            TextInputType::Url => "url",
            TextInputType::Search => "search",
        }
    }
}

/// A text input component with validation support.
///
/// Wraps a native HTML `<input>` element styled with the `form-control` class.
/// Supports HTML5 validation constraints and provides a pull-based async event model
/// that yields both input changes and blur events.
#[derive(ViewChild)]
pub struct TextInput<V: View> {
    #[child]
    input: V::Element,
    on_input: V::EventListener,
    on_blur: V::EventListener,
    current_value: String,
    validation_attempted: bool,
    additional_classes: Proxy<String>,
}

impl<V: View> TextInput<V> {
    /// Create a new text input with the given type and initial value.
    pub fn new(input_type: TextInputType, value: impl AsRef<str>) -> Self {
        let current_value = value.as_ref().to_string();
        let validation_attempted = false;
        let mut additional_classes = Proxy::new(String::new());

        let type_attr = input_type.as_str();
        let value_attr = value.as_ref().to_string();

        rsx! {
            let input = input(
                type = type_attr,
                class = additional_classes(classes => {
                    if classes.is_empty() {
                        "form-control".to_string()
                    } else {
                        format!("form-control {}", classes)
                    }
                }),
                value = value_attr,
                on:input = on_input,
                on:blur = on_blur,
            ) {}
        }

        Self {
            input,
            on_input,
            on_blur,
            current_value,
            validation_attempted,
            additional_classes,
        }
    }

    /// Read the current input value.
    pub fn value(&self) -> String {
        self.current_value.clone()
    }

    /// Programmatically set the input value.
    pub fn set_value(&mut self, value: impl AsRef<str>) {
        let val = value.as_ref().to_string();
        self.current_value = val.clone();
        self.input.set_property("value", val);
    }

    /// Set the placeholder text.
    pub fn set_placeholder(&self, text: impl AsRef<str>) {
        self.input.set_property("placeholder", text.as_ref());
    }

    /// Set whether the input is required.
    pub fn set_required(&self, required: bool) {
        if required {
            self.input.set_property("required", "");
        } else {
            self.input.remove_property("required");
        }
    }

    /// Set a regex pattern for validation.
    pub fn set_pattern(&self, pattern: impl AsRef<str>) {
        self.input.set_property("pattern", pattern.as_ref());
    }

    /// Set the minimum length for the input value.
    pub fn set_min_length(&self, len: u32) {
        self.input.set_property("minLength", len.to_string());
    }

    /// Set the maximum length for the input value.
    pub fn set_max_length(&self, len: u32) {
        self.input.set_property("maxLength", len.to_string());
    }

    /// Set additional CSS classes to append to the input.
    pub fn set_additional_classes(&mut self, classes: impl AsRef<str>) {
        self.additional_classes.set(classes.as_ref().to_string());
    }

    /// Disable the input.
    pub fn disable(&self) {
        self.input.set_property("disabled", "");
    }

    /// Enable the input.
    pub fn enable(&self) {
        self.input.remove_property("disabled");
    }

    /// Set whether the input is read-only.
    pub fn set_readonly(&self, readonly: bool) {
        if readonly {
            self.input.set_property("readonly", "");
        } else {
            self.input.remove_property("readonly");
        }
    }

    /// Update the input's CSS classes based on validation state.
    fn update_validation_classes(&self) {
        if self.validation_attempted {
            if self.is_valid() {
                self.input.dyn_el(|el: &web_sys::HtmlInputElement| {
                    use wasm_bindgen::JsCast;
                    let element: &web_sys::Element = el.unchecked_ref();
                    let list = element.class_list();
                    let _ = list.remove_1("is-invalid");
                    let _ = list.add_1("is-valid");
                });
            } else {
                self.input.dyn_el(|el: &web_sys::HtmlInputElement| {
                    use wasm_bindgen::JsCast;
                    let element: &web_sys::Element = el.unchecked_ref();
                    let list = element.class_list();
                    let _ = list.remove_1("is-valid");
                    let _ = list.add_1("is-invalid");
                });
            }
        }
    }
}

impl<V: View> Validatable<V> for TextInput<V> {
    fn is_valid(&self) -> bool {
        self.input
            .dyn_el(|el: &HtmlInputElement| el.validity().valid())
            .unwrap_or(true)
    }

    fn validation_message(&self) -> Option<String> {
        self.input
            .dyn_el(|el: &HtmlInputElement| {
                el.validation_message().ok().filter(|msg| !msg.is_empty())
            })
            .flatten()
    }

    fn validation_attempted(&self) -> bool {
        self.validation_attempted
    }

    fn set_id(&self, id: impl AsRef<str>) {
        self.input.set_property("id", id.as_ref());
    }

    fn set_aria_describedby(&self, ids: impl AsRef<str>) {
        self.input.dyn_el(|el: &web_sys::HtmlInputElement| {
            let _ = el.set_attribute("aria-describedby", ids.as_ref());
        });
    }
}

impl<V: View> StepMut for TextInput<V> {
    type Output = TextInputEvent<V>;

    async fn step_mut(&mut self) -> TextInputEvent<V> {
        let input_future = self.on_input.next().map(|event| {
            let value: String = self
                .input
                .dyn_el(|el: &HtmlInputElement| el.value())
                .unwrap_or_default();

            (TextInputEvent::Input { value, event }, false)
        });

        let blur_future = self
            .on_blur
            .next()
            .map(|event| (TextInputEvent::Blur { event }, true));

        let (result, is_blur) = input_future.or(blur_future).await;

        if let TextInputEvent::Input { ref value, .. } = result {
            self.current_value = value.clone();
        }

        if is_blur {
            self.validation_attempted = true;
            self.update_validation_classes();
        }

        result
    }
}
