//! Native select component.
//!
//! Wraps a native HTML `<select>` element styled with Bootstrap's `form-select`
//! class.  Options are managed dynamically and selection changes are delivered
//! through the pull-based [`Select::step`] async method.

use mogwai::prelude::*;
use mogwai::web::WebElement;

use super::Flavor;

/// Event produced when the user changes the selected option.
pub struct SelectEvent<V: View> {
    /// Index of the newly selected option.
    pub index: usize,
    /// The value string of the selected option.
    pub value: String,
    /// The raw DOM event.
    pub event: V::Event,
}

/// A single `<option>` within a [`Select`].
#[derive(ViewChild, ViewProperties)]
struct SelectOption<V: View> {
    #[child]
    #[properties]
    option: V::Element,
    value: String,
}

impl<V: View> SelectOption<V> {
    fn new(label: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        let value = value.as_ref().to_string();
        let text = V::Text::new(&label);
        let value_attr = value.clone();

        rsx! {
            let option = option(value = value_attr) {
                {text}
            }
        }

        Self { option, value }
    }
}

/// A native `<select>` component styled with Bootstrap's `form-select`.
///
/// Wraps a native HTML `<select>` element with dynamically managed `<option>`
/// children and a pull-based async event model.  Use [`Select::push`] to add
/// options and [`Select::step`] to await selection changes.
///
/// # Example
///
/// ```ignore
/// let mut select = Select::<V>::new(Some(Flavor::Primary));
/// select.push("Apple", "apple");
/// select.push("Banana", "banana");
/// select.push("Cherry", "cherry");
/// loop {
///     let ev = select.step().await;
///     log::info!("Selected index {} with value {}", ev.index, ev.value);
/// }
/// ```
#[derive(ViewChild, ViewProperties)]
pub struct Select<V: View> {
    #[child]
    #[properties]
    select: V::Element,
    on_change: V::EventListener,
    options: Vec<SelectOption<V>>,
    flavor: Proxy<Option<Flavor>>,
}

impl<V: View> Select<V> {
    /// Create a new empty select with an optional flavor.
    ///
    /// When a [`Flavor`] is provided the select receives a coloured border
    /// via Bootstrap's `border-{flavor}` utility class.
    pub fn new(flavor: Option<Flavor>) -> Self {
        let mut flavor_proxy = Proxy::new(flavor);

        rsx! {
            let select = select(
                class = flavor_proxy(f => match f {
                    Some(fl) => format!("form-select border-{fl}"),
                    None => "form-select".to_string(),
                }),
                on:change = on_change,
            ) {
                let options = {vec![]}
            }
        }

        Self {
            select,
            on_change,
            options,
            flavor: flavor_proxy,
        }
    }

    /// Add an option with the given display label and form value.
    ///
    /// Returns the index of the newly added option.
    pub fn push(&mut self, label: impl AsRef<str>, value: impl AsRef<str>) -> usize {
        let index = self.options.len();
        let opt = SelectOption::new(label, value);
        self.select.append_child(&opt);
        self.options.push(opt);
        index
    }

    /// Add an option whose value equals its label.
    ///
    /// Convenience wrapper around [`Select::push`].
    pub fn push_label(&mut self, label: impl AsRef<str>) -> usize {
        let s = label.as_ref().to_string();
        self.push(&s, &s)
    }

    /// Remove an option by index.
    ///
    /// ## Panics
    ///
    /// Panics if `index` is out of bounds.
    pub fn remove(&mut self, index: usize) {
        let opt = self.options.remove(index);
        self.select.remove_child(&opt);
    }

    /// Return the number of options.
    pub fn len(&self) -> usize {
        self.options.len()
    }

    /// Return `true` when the select has no options.
    pub fn is_empty(&self) -> bool {
        self.options.is_empty()
    }

    /// Update the visual flavor.
    pub fn set_flavor(&mut self, flavor: Option<Flavor>) {
        self.flavor.set(flavor);
    }

    /// Disable the select element.
    pub fn disable(&self) {
        self.select.set_property("disabled", "");
    }

    /// Enable the select element.
    pub fn enable(&self) {
        self.select.remove_property("disabled");
    }

    /// Read the currently selected index from the DOM.
    ///
    /// Returns [`None`] when nothing is selected (the underlying
    /// `selectedIndex` is `-1`).
    pub fn selected_index(&self) -> Option<usize> {
        let raw: Option<i32> = self
            .select
            .dyn_el(|el: &web_sys::HtmlSelectElement| el.selected_index());
        match raw {
            Some(i) if i >= 0 => Some(i as usize),
            _ => None,
        }
    }

    /// Read the value string of the currently selected option from the DOM.
    ///
    /// Returns [`None`] when the value is empty or the DOM query fails.
    pub fn selected_value(&self) -> Option<String> {
        let raw: Option<String> = self
            .select
            .dyn_el(|el: &web_sys::HtmlSelectElement| el.value());
        match raw {
            Some(s) if !s.is_empty() => Some(s),
            _ => None,
        }
    }

    /// Programmatically select an option by index.
    pub fn set_selected_index(&self, index: usize) {
        self.select
            .set_property("selectedIndex", format!("{index}"));
    }

    /// Await the next selection change.
    ///
    /// Returns a [`SelectEvent`] containing the index, value, and raw DOM
    /// event of the newly selected option.
    pub async fn step(&self) -> SelectEvent<V> {
        let event = self.on_change.next().await;
        let index = self.selected_index().unwrap_or(0);
        let value = self
            .options
            .get(index)
            .map(|o| o.value.clone())
            .unwrap_or_default();
        SelectEvent {
            index,
            value,
            event,
        }
    }
}

impl<V: View> Default for Select<V> {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(feature = "library")]
pub mod library {
    use mogwai::prelude::*;

    use super::*;

    /// Gallery sandbox for the [`Select`] component.
    #[derive(ViewChild)]
    pub struct SelectLibraryItem<V: View> {
        #[child]
        pub wrapper: V::Element,
        select: Select<V>,
        status_text: V::Text,
    }

    impl<V: View> Default for SelectLibraryItem<V> {
        fn default() -> Self {
            let mut select = Select::new(Some(Flavor::Primary));
            select.push("Apple", "apple");
            select.push("Banana", "banana");
            select.push("Cherry", "cherry");
            select.push("Date", "date");

            let status_text = V::Text::new("No selection yet.");

            rsx! {
                let wrapper = div() {
                    div(class = "mb-3") {
                        label(class = "form-label") {
                            "Pick a fruit:"
                        }
                        {&select}
                    }
                    p() {
                        {&status_text}
                    }
                }
            }

            Self {
                wrapper,
                select,
                status_text,
            }
        }
    }

    impl<V: View> SelectLibraryItem<V> {
        pub async fn step(&mut self) {
            let ev = self.select.step().await;
            self.status_text
                .set_text(format!("Selected: {} (index {})", ev.value, ev.index));
        }
    }
}
