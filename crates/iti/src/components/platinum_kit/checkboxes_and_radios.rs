use futures_lite::FutureExt;
use mogwai::{future::MogwaiFutureExt, prelude::*};

use crate::components::{checkbox::*, radio::*};

#[derive(ViewChild)]
pub struct PlatinumKitCheckboxesAndRadios<V: View> {
    #[child]
    container: V::Element,
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
            group1,
            group2,
            log_text,
        }
    }
}

impl<V: View> StepMut for PlatinumKitCheckboxesAndRadios<V> {
    type Output = ();

    async fn step_mut(&mut self) {
        let future1 = self.group1.step_mut().map(|e| ("size", e));
        let future2 = self.group2.step_mut().map(|e| ("color", e));
        let (group_name, event) = future1.or(future2).await;

        self.log_text.set_text(format!(
            "{}: Selected '{}' (index {})",
            group_name, event.value, event.index
        ));
    }
}
