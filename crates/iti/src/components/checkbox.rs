//! Checkbox component.
//!
//! Wraps a native HTML `<input type="checkbox">` with Platinum styling and a
//! pull-based async event model. Supports standard checkbox and switch styles.

use futures_lite::FutureExt;
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
    on_click: V::EventListener,
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

                let label = label(
                    class = "form-check-label",
                    on:click = on_click,
                ) {
                    {label_text}
                }
            }
        }

        let mut cb = Self {
            wrapper,
            input,
            label,
            on_change,
            on_click,
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

    async fn label_clicked(&self) -> CheckboxEvent<V> {
        let event = self.on_click.next().await;
        let new_checked = self
            .input
            .dyn_el(|input: &HtmlInputElement| {
                if input.disabled() {
                    self.checked
                } else {
                    let next = !self.checked;
                    input.set_checked(next);
                    next
                }
            })
            .unwrap_or(self.checked);
        CheckboxEvent {
            checked: new_checked,
            event,
        }
    }

    async fn changed(&self) -> CheckboxEvent<V> {
        let event = self.on_change.next().await;
        let checked = self
            .input
            .dyn_el(|el: &HtmlInputElement| el.checked())
            .unwrap_or(false);
        CheckboxEvent { checked, event }
    }
}

impl<V: View> StepMut for Checkbox<V> {
    type Output = CheckboxEvent<V>;
    async fn step_mut(&mut self) -> CheckboxEvent<V> {
        let event = self.changed().or(self.label_clicked()).await;
        self.checked = event.checked;
        event
    }
}
