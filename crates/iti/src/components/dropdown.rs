//! Dropdown component.
//!
//! A Bootstrap dropdown button with a menu of clickable items.  Open/close and
//! click-outside-to-dismiss are managed in pure Rust — no Bootstrap JS required.
use mogwai::prelude::*;
use wasm_bindgen::JsCast;

use super::Flavor;

/// Event emitted by a [`Dropdown`].
pub enum DropdownEvent<V: View> {
    /// A menu item was clicked.
    ItemClicked { index: usize, event: V::Event },
    /// The dropdown was dismissed (Escape key or click outside).
    Dismissed,
}

/// A single item within a [`Dropdown`] menu.
#[derive(ViewChild)]
pub struct DropdownItem<V: View> {
    #[child]
    li: V::Element,
    on_click: V::EventListener,
}

impl<V: View> DropdownItem<V> {
    fn new(label: impl AsRef<str>) -> Self {
        let text = V::Text::new(label);
        rsx! {
            let li = li() {
                a(
                    class = "dropdown-item",
                    href = "#",
                    on:click = on_click,
                ) {
                    {text}
                }
            }
        }

        Self { li, on_click }
    }
}

/// A Bootstrap dropdown button with a menu.
///
/// Toggle the menu by calling [`Dropdown::toggle`] in response to
/// [`Dropdown::step`] returning [`None`].
#[derive(ViewChild)]
pub struct Dropdown<V: View> {
    #[child]
    wrapper: V::Element,
    menu: V::Element,
    toggle_click: V::EventListener,
    backdrop_click: V::EventListener,
    keydown: V::EventListener,
    items: Vec<DropdownItem<V>>,
    open: Proxy<bool>,
    is_open: bool,
    flavor: Proxy<Flavor>,
}

impl<V: View> Dropdown<V> {
    pub fn new(label: impl AsRef<str>, flavor: Flavor) -> Self {
        let mut flavor_proxy = Proxy::new(flavor);
        let mut open = Proxy::new(false);
        let label_text = V::Text::new(label);

        rsx! {
            let wrapper = div(
                class = "dropdown",
                document:keydown = keydown,
            ) {
                button(
                    class = flavor_proxy(
                        f => format!("btn btn-{f} dropdown-toggle")
                    ),
                    type = "button",
                    on:click = toggle_click,
                ) {
                    {label_text}
                }
                div(
                    style:position = "fixed",
                    style:inset = "0",
                    style:z_index = "1000",
                    style:display = open(
                        is_open => if *is_open { "block" } else { "none" }
                    ),
                    on:click = backdrop_click,
                ) {}
                let menu = ul(
                    class = open(is_open => if *is_open {
                        "dropdown-menu show"
                    } else {
                        "dropdown-menu"
                    }),
                    style:z_index = "1001",
                ) {
                    let items = {vec![]}
                }
            }
        }

        Self {
            wrapper,
            menu,
            toggle_click,
            backdrop_click,
            keydown,
            items,
            open,
            is_open: false,
            flavor: flavor_proxy,
        }
    }

    /// Add a menu item and return its index.
    pub fn push(&mut self, label: impl AsRef<str>) -> usize {
        let index = self.items.len();
        let item = DropdownItem::new(label);
        self.menu.append_child(&item);
        self.items.push(item);
        index
    }

    /// Remove a menu item by index.
    ///
    /// ## Panics
    /// Panics if `index` >= len.
    pub fn remove(&mut self, index: usize) {
        let item = self.items.remove(index);
        self.menu.remove_child(&item);
    }

    pub fn set_flavor(&mut self, flavor: Flavor) {
        self.flavor.set(flavor);
    }

    /// Show the dropdown menu.
    pub fn show(&mut self) {
        self.is_open = true;
        self.open.set(true);
    }

    /// Hide the dropdown menu.
    pub fn hide(&mut self) {
        self.is_open = false;
        self.open.set(false);
    }

    /// Toggle the dropdown menu.
    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
        self.open.set(self.is_open);
    }

    fn item_click_events(&self) -> impl std::future::Future<Output = DropdownEvent<V>> + '_ {
        use mogwai::future::*;

        let events = self.items.iter().enumerate().map(|(index, item)| {
            item.on_click
                .next()
                .map(move |event| DropdownEvent::ItemClicked { index, event })
        });
        race_all(events)
    }

    /// Await the next dropdown interaction.
    ///
    /// Returns [`None`] when the toggle button was clicked (caller should call
    /// [`Dropdown::toggle`]), [`Some(ItemClicked)`] when a menu item was
    /// clicked, or [`Some(Dismissed)`] when the dropdown was dismissed via
    /// Escape or a click outside.
    ///
    /// Escape and click-outside events are only returned when the dropdown is
    /// open; when closed they are silently ignored.
    pub async fn step(&self) -> Option<DropdownEvent<V>> {
        use futures_lite::FutureExt;
        use mogwai::future::MogwaiFutureExt;

        loop {
            let escape = async {
                loop {
                    let ev = self.keydown.next().await;
                    let is_escape = ev.when_event::<mogwai::web::Web, _>(|e: &web_sys::Event| {
                        e.dyn_ref::<web_sys::KeyboardEvent>()
                            .is_some_and(|ke| ke.key() == "Escape")
                    });
                    if is_escape == Some(true) {
                        return;
                    }
                }
            };

            let result = self
                .toggle_click
                .next()
                .map(|_| None)
                .or(self.item_click_events().map(Some))
                .or(self
                    .backdrop_click
                    .next()
                    .map(|_| Some(DropdownEvent::Dismissed)))
                .or(escape.map(|_| Some(DropdownEvent::Dismissed)))
                .await;

            // Only return Dismissed when the dropdown is actually open;
            // otherwise loop back and wait for a meaningful event.
            match &result {
                Some(DropdownEvent::Dismissed) if !self.is_open => continue,
                _ => return result,
            }
        }
    }
}

#[cfg(feature = "library")]
pub mod library {
    use mogwai::prelude::*;

    use super::*;

    #[derive(ViewChild)]
    pub struct DropdownLibraryItem<V: View> {
        #[child]
        pub wrapper: V::Element,
        dropdown: Dropdown<V>,
        status_text: V::Text,
    }

    impl<V: View> Default for DropdownLibraryItem<V> {
        fn default() -> Self {
            let mut dropdown = Dropdown::new("Select an item", Flavor::Primary);
            dropdown.push("Action");
            dropdown.push("Another action");
            dropdown.push("Something else");

            let status_text = V::Text::new("No item selected yet.");

            rsx! {
                let wrapper = div() {
                    div(class = "mb-3") {
                        {&dropdown}
                    }
                    p() {
                        {&status_text}
                    }
                }
            }

            Self {
                wrapper,
                dropdown,
                status_text,
            }
        }
    }

    impl<V: View> DropdownLibraryItem<V> {
        pub async fn step(&mut self) {
            match self.dropdown.step().await {
                None => {
                    self.dropdown.toggle();
                }
                Some(DropdownEvent::ItemClicked { index, .. }) => {
                    self.dropdown.hide();
                    let labels = ["Action", "Another action", "Something else"];
                    let label = labels.get(index).unwrap_or(&"Unknown");
                    self.status_text.set_text(format!("Selected: {label}"));
                }
                Some(DropdownEvent::Dismissed) => {
                    self.dropdown.hide();
                }
            }
        }
    }
}
