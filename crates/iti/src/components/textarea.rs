//! Textarea component.
//!
//! Wraps a native HTML `<textarea>` element with HTML5 validation, reactive state,
//! and a pull-based async event model.

use futures_lite::FutureExt;
use mogwai::future::MogwaiFutureExt;
use mogwai::prelude::*;
use mogwai::web::WebElement;
use web_sys::HtmlTextAreaElement;

use super::Validatable;

/// Event produced by Textarea.
pub enum TextareaEvent<V: View> {
    /// User typed or changed the value.
    Input {
        /// The current textarea value.
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

/// A multi-line text input component with validation support.
#[derive(ViewChild, ViewProperties)]
pub struct Textarea<V: View> {
    #[child]
    #[properties]
    textarea: V::Element,
    on_input: V::EventListener,
    on_blur: V::EventListener,
    current_value: String,
    validation_attempted: bool,
    additional_classes: Proxy<String>,
}

impl<V: View> Textarea<V> {
    /// Create a new textarea with the given initial value.
    pub fn new(value: impl AsRef<str>) -> Self {
        let current_value = value.as_ref().to_string();
        let validation_attempted = false;
        let mut additional_classes = Proxy::new(String::new());

        let value_text = V::Text::new(value.as_ref());

        rsx! {
            let textarea = textarea(
                class = additional_classes(classes => {
                    if classes.is_empty() {
                        "form-control".to_string()
                    } else {
                        format!("form-control {}", classes)
                    }
                }),
                on:input = on_input,
                on:blur = on_blur,
            ) {
                {value_text}
            }
        }

        Self {
            textarea,
            on_input,
            on_blur,
            current_value,
            validation_attempted,
            additional_classes,
        }
    }

    /// Read the current textarea value.
    pub fn value(&self) -> String {
        self.current_value.clone()
    }

    /// Programmatically set the textarea value.
    pub fn set_value(&mut self, value: impl AsRef<str>) {
        let val = value.as_ref().to_string();
        self.current_value = val.clone();
        self.textarea.dyn_el(|el: &HtmlTextAreaElement| {
            el.set_value(&val);
        });
    }

    /// Set the placeholder text.
    pub fn set_placeholder(&self, text: impl AsRef<str>) {
        self.textarea.set_property("placeholder", text.as_ref());
    }

    /// Set the number of visible text rows.
    pub fn set_rows(&self, rows: u32) {
        self.textarea.set_property("rows", rows.to_string());
    }

    /// Set the number of visible text columns.
    pub fn set_cols(&self, cols: u32) {
        self.textarea.set_property("cols", cols.to_string());
    }

    /// Set whether the textarea is required.
    pub fn set_required(&self, required: bool) {
        if required {
            self.textarea.set_property("required", "");
        } else {
            self.textarea.remove_property("required");
        }
    }

    /// Set the minimum length for the textarea value.
    pub fn set_min_length(&self, len: u32) {
        self.textarea.set_property("minLength", len.to_string());
    }

    /// Set the maximum length for the textarea value.
    pub fn set_max_length(&self, len: u32) {
        self.textarea.set_property("maxLength", len.to_string());
    }

    /// Set additional CSS classes to append to the textarea.
    pub fn set_additional_classes(&mut self, classes: impl AsRef<str>) {
        self.additional_classes.set(classes.as_ref().to_string());
    }

    /// Disable the textarea.
    pub fn disable(&self) {
        self.textarea.set_property("disabled", "");
    }

    /// Enable the textarea.
    pub fn enable(&self) {
        self.textarea.remove_property("disabled");
    }

    /// Set whether the textarea is read-only.
    pub fn set_readonly(&self, readonly: bool) {
        if readonly {
            self.textarea.set_property("readonly", "");
        } else {
            self.textarea.remove_property("readonly");
        }
    }

    /// Update the textarea's CSS classes based on validation state.
    fn update_validation_classes(&self) {
        if self.validation_attempted {
            if self.is_valid() {
                self.textarea.dyn_el(|el: &web_sys::HtmlTextAreaElement| {
                    use wasm_bindgen::JsCast;
                    let element: &web_sys::Element = el.unchecked_ref();
                    let list = element.class_list();
                    let _ = list.remove_1("is-invalid");
                    let _ = list.add_1("is-valid");
                });
            } else {
                self.textarea.dyn_el(|el: &web_sys::HtmlTextAreaElement| {
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

impl<V: View> Validatable<V> for Textarea<V> {
    fn is_valid(&self) -> bool {
        self.textarea
            .dyn_el(|el: &HtmlTextAreaElement| el.validity().valid())
            .unwrap_or(true)
    }

    fn validation_message(&self) -> Option<String> {
        self.textarea
            .dyn_el(|el: &HtmlTextAreaElement| {
                el.validation_message().ok().filter(|msg| !msg.is_empty())
            })
            .flatten()
    }

    fn validation_attempted(&self) -> bool {
        self.validation_attempted
    }

    fn set_id(&self, id: impl AsRef<str>) {
        self.textarea.set_property("id", id.as_ref());
    }

    fn set_aria_describedby(&self, ids: impl AsRef<str>) {
        self.textarea.dyn_el(|el: &web_sys::HtmlTextAreaElement| {
            let _ = el.set_attribute("aria-describedby", ids.as_ref());
        });
    }
}

impl<V: View> StepMut for Textarea<V> {
    type Output = TextareaEvent<V>;

    async fn step_mut(&mut self) -> TextareaEvent<V> {
        let input_future = self.on_input.next().map(|event| {
            let value: String = self
                .textarea
                .dyn_el(|el: &HtmlTextAreaElement| el.value())
                .unwrap_or_default();

            (TextareaEvent::Input { value, event }, false)
        });

        let blur_future = self
            .on_blur
            .next()
            .map(|event| (TextareaEvent::Blur { event }, true));

        let (result, is_blur) = input_future.or(blur_future).await;

        if let TextareaEvent::Input { ref value, .. } = result {
            self.current_value = value.clone();
        }

        if is_blur {
            self.validation_attempted = true;
            self.update_validation_classes();
        }

        result
    }
}
