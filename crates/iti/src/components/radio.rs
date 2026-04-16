//! Radio button group component.
//!
//! Manages a group of mutually-exclusive radio buttons with Platinum styling and a
//! pull-based async event model.

use std::future::Future;
use std::sync::atomic::{AtomicU32, Ordering};

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
#[derive(ViewChild)]
struct RadioOption<V: View> {
    #[child]
    wrapper: V::Element,
    input: V::Element,
    #[allow(dead_code)]
    label: V::Element,
    value: String,
    on_change: V::EventListener,
}

impl<V: View> RadioOption<V> {
    fn new(name: impl AsRef<str>, label: impl AsRef<str>, value: impl AsRef<str>) -> Self {
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

                let label = label(class = "form-check-label") {
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
}

/// State for a radio group.
struct RadioGroupState {
    inline: bool,
}

impl RadioGroupState {
    fn wrapper_class(&self) -> &str {
        if self.inline {
            "d-flex flex-wrap gap-2"
        } else {
            ""
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
#[derive(ViewChild)]
pub struct RadioGroup<V: View> {
    #[child]
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
        let option = RadioOption::new(&self.name, label, value);

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

    fn radio_change_events(&self) -> impl Future<Output = RadioEvent<V>> + '_ {
        use mogwai::future::*;

        let events = self.options.iter().enumerate().map(|(index, option)| {
            option.on_change.next().map(move |event| {
                let value = option.value.clone();
                RadioEvent {
                    index,
                    value,
                    event,
                }
            })
        });
        race_all(events)
    }

    /// Wait for the next radio button selection.
    ///
    /// Returns a [`RadioEvent`] when the user selects a radio button.
    pub async fn step(&mut self) -> RadioEvent<V> {
        let event = self.radio_change_events().await;
        self.selected_index = Some(event.index);
        event
    }
}

#[cfg(feature = "library")]
pub mod library {
    use super::*;

    #[derive(ViewChild)]
    pub struct RadioLibraryItem<V: View> {
        #[child]
        container: V::Element,
        group1: RadioGroup<V>,
        group2: RadioGroup<V>,
        log: V::Element,
    }

    impl<V: View> Default for RadioLibraryItem<V> {
        fn default() -> Self {
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

            let log_text = V::Text::new("");

            rsx! {
                let container = div() {
                    h2() { "Radio Group" }
                    p() { "Mutually-exclusive radio button groups with Platinum styling." }

                    h4(class = "mt-4") { "Vertical Layout (Default)" }
                    div(class = "mb-3") {
                        label(class = "form-label") { "Select Size" }
                        {&group1}
                    }

                    h4(class = "mt-4") { "Inline Layout" }
                    div(class = "mb-3") {
                        label(class = "form-label") { "Select Color" }
                        {&group2}
                    }

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
                group1,
                group2,
                log,
            }
        }
    }

    impl<V: View> RadioLibraryItem<V> {
        pub async fn step(&mut self) {
            use futures_lite::FutureExt;
            use mogwai::future::MogwaiFutureExt;

            let future1 = self.group1.step().map(|e| ("size", e));
            let future2 = self.group2.step().map(|e| ("color", e));

            let (group_name, event) = future1.or(future2).await;

            let msg = format!(
                "{}: Selected '{}' (index {})",
                group_name, event.value, event.index
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
