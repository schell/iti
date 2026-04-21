//! Checkbox component.
//!
//! Wraps a native HTML `<input type="checkbox">` with Platinum styling and a
//! pull-based async event model. Supports standard checkbox and switch styles.

use mogwai::prelude::*;
use mogwai::web::WebElement;
use web_sys::HtmlInputElement;

/// Event produced when the checkbox is toggled.
pub struct CheckboxEvent<V: View> {
    /// Whether the checkbox is now checked.
    pub checked: bool,
    /// The raw DOM event.
    pub event: V::Event,
}

/// A checkbox component with optional switch styling.
#[derive(ViewChild, ViewProperties)]
pub struct Checkbox<V: View> {
    #[child]
    #[properties]
    wrapper: V::Element,
    input: V::Element,
    #[allow(dead_code)]
    label: V::Element,
    on_change: V::EventListener,
    checked: bool,
    is_switch: Proxy<bool>,
}

impl<V: View> Checkbox<V> {
    /// Create a new checkbox with the given label and initial checked state.
    pub fn new(label: impl AsRef<str>, checked: bool) -> Self {
        let mut is_switch = Proxy::new(false);
        let label_text = V::Text::new(label);

        rsx! {
            let wrapper = div(
                class = is_switch(sw => if *sw { "form-check form-switch" } else { "form-check" })
            ) {
                let input = input(
                    type = "checkbox",
                    class = "form-check-input",
                    on:change = on_change,
                ) {}

                let label = label(class = "form-check-label") {
                    {label_text}
                }
            }
        }

        let mut cb = Self {
            wrapper,
            input,
            label,
            on_change,
            checked,
            is_switch,
        };
        cb.set_checked(checked);
        cb
    }

    /// Check if the checkbox is currently checked.
    pub fn is_checked(&self) -> bool {
        self.checked
    }

    /// Programmatically set the checked state.
    pub fn set_checked(&mut self, checked: bool) {
        self.checked = checked;
        self.input.dyn_el(|input: &web_sys::HtmlInputElement| {
            input.set_checked(checked);
        });
        // if checked {
        //     self.input.set_property("checked", "");
        // } else {
        //     self.input.remove_property("checked");
        // }
    }

    /// Toggle the checked state.
    pub fn toggle(&mut self) {
        self.set_checked(!self.is_checked());
    }

    /// Enable or disable switch styling.
    pub fn set_switch_style(&mut self, is_switch: bool) {
        self.is_switch.set(is_switch);
    }

    /// Disable the checkbox.
    pub fn disable(&self) {
        self.input.set_property("disabled", "");
    }

    /// Enable the checkbox.
    pub fn enable(&self) {
        self.input.remove_property("disabled");
    }

    /// Wait for the next change event.
    pub async fn step(&mut self) -> CheckboxEvent<V> {
        let event = self.on_change.next().await;

        let checked = self
            .input
            .dyn_el(|el: &HtmlInputElement| el.checked())
            .unwrap_or(false);

        self.checked = checked;

        CheckboxEvent { checked, event }
    }
}

#[cfg(feature = "library")]
pub mod library {
    use super::*;

    #[derive(ViewChild)]
    pub struct CheckboxLibraryItem<V: View> {
        #[child]
        container: V::Element,
        checkbox1: Checkbox<V>,
        checkbox2: Checkbox<V>,
        checkbox3: Checkbox<V>,
        log: V::Element,
    }

    impl<V: View> Default for CheckboxLibraryItem<V> {
        fn default() -> Self {
            let checkbox1 = Checkbox::new("Default checkbox", false);
            let checkbox2 = Checkbox::new("Checked by default", true);

            let mut checkbox3 = Checkbox::new("Switch style", false);
            checkbox3.set_switch_style(true);

            let log_text = V::Text::new("");

            rsx! {
                let container = div() {
                    h2() { "Checkbox" }
                    p() { "Checkbox and switch components with Platinum styling." }

                    h4(class = "mt-4") { "Standard Checkboxes" }
                    {&checkbox1}
                    {&checkbox2}

                    h4(class = "mt-4") { "Switch Style" }
                    {&checkbox3}

                    div(class = "alert alert-light mt-4") {
                        strong() { "Event Log:" }
                        let log = pre(class = "mb-0 mt-2") {
                            {log_text}
                        }
                    }
                }
            }

            Self {
                container,
                checkbox1,
                checkbox2,
                checkbox3,
                log,
            }
        }
    }

    impl<V: View> CheckboxLibraryItem<V> {
        pub async fn step(&mut self) {
            use futures_lite::FutureExt;
            use mogwai::future::MogwaiFutureExt;

            let future1 = self.checkbox1.step().map(|e| ("default", e));
            let future2 = self.checkbox2.step().map(|e| ("pre-checked", e));
            let future3 = self.checkbox3.step().map(|e| ("switch", e));

            let (name, event) = future1.or(future2.or(future3)).await;

            let msg = format!(
                "{}: {} (checked: {})",
                name,
                if event.checked {
                    "checked"
                } else {
                    "unchecked"
                },
                event.checked
            );

            let current_text = self
                .log
                .dyn_el(|el: &web_sys::Element| el.text_content())
                .flatten()
                .unwrap_or_default();

            let new_text = if current_text.is_empty() {
                msg
            } else {
                format!("{}\n{}", current_text, msg)
            };

            self.log
                .dyn_el(|el: &web_sys::Element| el.set_text_content(Some(&new_text)));
        }
    }
}
