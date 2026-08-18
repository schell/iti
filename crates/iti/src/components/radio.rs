//! Radio button group component.
//!
//! Manages a group of mutually-exclusive radio buttons with Platinum styling and a
//! pull-based async event model.

use std::sync::atomic::{AtomicU32, Ordering};

use futures_lite::FutureExt;
use mogwai::prelude::*;
use mogwai::web::WebElement;

/// Generate a unique name for radio button groups.
fn generate_radio_name() -> String {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("radio-group-{}", id)
}

/// Event produced when a radio button is selected.
pub struct RadioEvent<V: View> {
    /// Index of the selected radio button.
    pub index: usize,
    /// The value string of the selected radio button.
    pub value: String,
    /// The raw DOM event.
    pub event: V::Event,
}

/// A single radio button within a [`RadioGroup`].
#[derive(ViewChild, ViewProperties)]
struct RadioOption<V: View> {
    #[child]
    #[properties]
    wrapper: V::Element,
    input: V::Element,
    #[allow(dead_code)]
    label: V::Element,
    value: String,
    on_change: V::EventListener,
    on_click: V::EventListener,
    index: usize,
}

impl<V: View> RadioOption<V> {
    fn new(
        name: impl AsRef<str>,
        label: impl AsRef<str>,
        value: impl AsRef<str>,
        index: usize,
    ) -> Self {
        let value = value.as_ref().to_string();
        let label_text = V::Text::new(label);
        let name_attr = name.as_ref().to_string();

        rsx! {
            let wrapper = div(class = "form-check") {
                let input = input(
                    type = "radio",
                    class = "form-check-input",
                    name = name_attr,
                    on:change = on_change,
                ) {}

                let label = label(
                    class = "form-check-label",
                    on:click = on_click
                ) {
                    {label_text}
                }
            }
        }

        Self {
            wrapper,
            input,
            label,
            value,
            on_change,
            on_click,
            index,
        }
    }

    fn set_inline(&self, inline: bool) {
        if inline {
            self.wrapper
                .set_property("class", "form-check form-check-inline");
        } else {
            self.wrapper.set_property("class", "form-check");
        }
    }

    async fn click(&self) -> RadioEvent<V> {
        let ev = self.on_click.next().await;
        self.input.dyn_el::<web_sys::HtmlInputElement, _>(|input| {
            input.set_checked(true);
        });
        RadioEvent {
            index: self.index,
            value: self.value.clone(),
            event: ev,
        }
    }

    async fn changed(&self) -> RadioEvent<V> {
        let ev = self.on_change.next().await;
        RadioEvent {
            index: self.index,
            value: self.value.clone(),
            event: ev,
        }
    }
}

/// State for a radio group.
struct RadioGroupState {
    inline: bool,
}

impl RadioGroupState {
    fn wrapper_class(&self) -> &str {
        if self.inline {
            "radio-group d-flex flex-wrap gap-2"
        } else {
            "radio-group"
        }
    }
}

/// A group of mutually-exclusive radio buttons.
///
/// Wraps multiple radio inputs styled with Platinum `form-check` classes.
/// All radio buttons in the group share a unique `name` attribute to ensure
/// mutual exclusivity.
///
/// # Example
///
/// ```ignore
/// let mut group = RadioGroup::<V>::new("color");
/// group.push("Red", "red");
/// group.push("Green", "green");
/// group.push("Blue", "blue");
/// loop {
///     let event = group.step().await;
///     log::info!("Selected: {} (index {})", event.value, event.index);
/// }
/// ```
#[derive(ViewChild, ViewProperties)]
pub struct RadioGroup<V: View> {
    #[child]
    #[properties]
    wrapper: V::Element,
    options: Vec<RadioOption<V>>,
    name: String,
    selected_index: Option<usize>,
    state: Proxy<RadioGroupState>,
    inline: bool,
}

impl<V: View> RadioGroup<V> {
    /// Create a new radio group with the given name.
    ///
    /// If an empty name is provided, a unique name is generated automatically.
    ///
    /// # Arguments
    ///
    /// * `name` — the `name` attribute shared by all radio buttons in this group
    pub fn new(name: impl AsRef<str>) -> Self {
        let name = if name.as_ref().is_empty() {
            generate_radio_name()
        } else {
            name.as_ref().to_string()
        };

        let selected_index = None;
        let inline = false;
        let mut state = Proxy::new(RadioGroupState { inline: false });

        rsx! {
            let wrapper = div(
                class = state(s => s.wrapper_class()),
                role = "radiogroup",
            ) {}
        }

        Self {
            wrapper,
            options: Vec::new(),
            name,
            selected_index,
            state,
            inline,
        }
    }

    /// Add a radio button with the given label and value.
    ///
    /// Returns the index of the newly added option.
    pub fn push(&mut self, label: impl AsRef<str>, value: impl AsRef<str>) -> usize {
        let index = self.options.len();
        let option = RadioOption::new(&self.name, label, value, index);

        // Apply current inline state
        if self.inline {
            option.set_inline(true);
        }

        self.wrapper.append_child(&option);
        self.options.push(option);
        index
    }

    /// Enable or disable inline layout.
    ///
    /// When enabled, radio buttons are displayed horizontally. When disabled,
    /// they are stacked vertically (default).
    pub fn set_inline(&mut self, inline: bool) {
        self.inline = inline;
        self.state.modify(|s| s.inline = inline);

        // Update all existing options
        for option in &self.options {
            option.set_inline(inline);
        }
    }

    /// Get the index of the currently selected radio button.
    ///
    /// Returns `None` if no radio button is selected.
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    /// Get the value of the currently selected radio button.
    ///
    /// Returns `None` if no radio button is selected or the index is out of bounds.
    pub fn selected_value(&self) -> Option<String> {
        let index = self.selected_index?;
        self.options.get(index).map(|opt| opt.value.clone())
    }

    /// Programmatically select a radio button by index.
    ///
    /// ## Panics
    ///
    /// Panics if `index` is out of bounds.
    pub fn set_selected(&mut self, index: usize) {
        assert!(index < self.options.len(), "Radio index out of bounds");

        // Uncheck all, check the selected one
        for (i, option) in self.options.iter().enumerate() {
            if i == index {
                option.input.set_property("checked", "true");
            } else {
                option.input.remove_property("checked");
            }
        }

        self.selected_index = Some(index);
    }

    /// Disable all radio buttons in the group.
    pub fn disable(&self) {
        for option in &self.options {
            option.input.set_property("disabled", "");
        }
    }

    /// Enable all radio buttons in the group.
    pub fn enable(&self) {
        for option in &self.options {
            option.input.remove_property("disabled");
        }
    }

    /// Return the number of radio buttons in the group.
    pub fn len(&self) -> usize {
        self.options.len()
    }

    /// Return `true` if the group contains no radio buttons.
    pub fn is_empty(&self) -> bool {
        self.options.is_empty()
    }

    async fn label_clicked(&self) -> RadioEvent<V> {
        let events = self.options.iter().map(|option| option.click());
        mogwai::future::race_all(events).await
    }

    async fn radio_changed(&self) -> RadioEvent<V> {
        let events = self.options.iter().map(|option| option.changed());
        mogwai::future::race_all(events).await
    }
}

impl<V: View> StepMut for RadioGroup<V> {
    type Output = RadioEvent<V>;
    async fn step_mut(&mut self) -> RadioEvent<V> {
        let event = self.radio_changed().or(self.label_clicked()).await;
        self.selected_index = Some(event.index);
        event
    }
}
