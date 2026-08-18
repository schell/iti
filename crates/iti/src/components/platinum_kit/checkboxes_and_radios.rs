use std::future::Future;
use std::pin::Pin;

use mogwai::{future::MogwaiFutureExt, prelude::*};

use crate::components::{checkbox::*, radio::*};

enum DemoEvent<V: View> {
    Checkbox(&'static str, CheckboxEvent<V>),
    Radio(&'static str, RadioEvent<V>),
}

#[derive(ViewChild)]
pub struct PlatinumKitCheckboxesAndRadios<V: View> {
    #[child]
    container: V::Element,
    cb_default: Checkbox<V>,
    cb_checked: Checkbox<V>,
    cb_disabled: Checkbox<V>,
    cb_disabled_checked: Checkbox<V>,
    cb_switch: Checkbox<V>,
    cb_switch_on: Checkbox<V>,
    group1: RadioGroup<V>,
    group2: RadioGroup<V>,
    log_text: V::Text,
}

impl<V: View> Default for PlatinumKitCheckboxesAndRadios<V> {
    fn default() -> Self {
        let cb_default = Checkbox::new("Unchecked", false);
        let cb_checked = Checkbox::new("Checked", true);

        let cb_disabled = Checkbox::new("Disabled", false);
        cb_disabled.disable();

        let cb_disabled_checked = Checkbox::new("Disabled checked", true);
        cb_disabled_checked.disable();

        let mut cb_switch = Checkbox::new("Switch off", false);
        cb_switch.set_switch_style(true);

        let mut cb_switch_on = Checkbox::new("Switch on", true);
        cb_switch_on.set_switch_style(true);

        let mut group1 = RadioGroup::new("size");
        group1.push("Small", "sm");
        group1.push("Medium", "md");
        group1.push("Large", "lg");

        let mut group2 = RadioGroup::new("color");
        group2.push("Red", "red");
        group2.push("Green", "green");
        group2.push("Blue", "blue");
        group2.push("Yellow", "yellow");
        group2.set_inline(true);

        rsx! {
            let container = div(class = "d-flex flex-wrap gap-4 panel") {
                div() {
                    p() { strong() { "Checkboxes" } }
                    {&cb_default}
                    {&cb_checked}
                    {&cb_disabled}
                    {&cb_disabled_checked}
                }
                div() {
                    p() { strong() { "Switches" } }
                    {&cb_switch}
                    {&cb_switch_on}
                }
                div() {
                    p() { strong() { "Radio Group" } }
                    {&group1}
                }
                div() {
                    p() { strong() { "Radio Inline" } }
                    {&group2}
                }
                div(class = "alert alert-light mt-4") {
                    strong() { "Event:" }
                    pre() {
                        let log_text = "Waiting for next event"
                    }
                }
            }
        }

        Self {
            container,
            cb_default,
            cb_checked,
            cb_disabled,
            cb_disabled_checked,
            cb_switch,
            cb_switch_on,
            group1,
            group2,
            log_text,
        }
    }
}

impl<V: View> StepMut for PlatinumKitCheckboxesAndRadios<V> {
    type Output = ();

    async fn step_mut(&mut self) {
        let futs: Vec<Pin<Box<dyn Future<Output = DemoEvent<V>> + '_>>> = vec![
            Box::pin(
                self.cb_default
                    .step_mut()
                    .map(|e| DemoEvent::Checkbox("default", e)),
            ),
            Box::pin(
                self.cb_checked
                    .step_mut()
                    .map(|e| DemoEvent::Checkbox("pre-checked", e)),
            ),
            Box::pin(
                self.cb_disabled
                    .step_mut()
                    .map(|e| DemoEvent::Checkbox("disabled", e)),
            ),
            Box::pin(
                self.cb_disabled_checked
                    .step_mut()
                    .map(|e| DemoEvent::Checkbox("disabled checked", e)),
            ),
            Box::pin(
                self.cb_switch
                    .step_mut()
                    .map(|e| DemoEvent::Checkbox("switch off", e)),
            ),
            Box::pin(
                self.cb_switch_on
                    .step_mut()
                    .map(|e| DemoEvent::Checkbox("switch on", e)),
            ),
            Box::pin(self.group1.step_mut().map(|e| DemoEvent::Radio("size", e))),
            Box::pin(self.group2.step_mut().map(|e| DemoEvent::Radio("color", e))),
        ];

        let event = mogwai::future::race_all(futs).await;

        let msg = match event {
            DemoEvent::Checkbox(name, e) => format!(
                "{}: {} (checked: {})",
                name,
                if e.checked { "checked" } else { "unchecked" },
                e.checked
            ),
            DemoEvent::Radio(name, e) => {
                format!("{}: Selected '{}' (index {})", name, e.value, e.index)
            }
        };
        self.log_text.set_text(msg);
    }
}
