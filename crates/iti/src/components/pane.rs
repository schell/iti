//! Window / page panes.
//!
//! A type for storing panes of HTML and their logic.
//!
//! A "pane" is a spot of HTML suitable for storing in a collection, where only
//! one item in that collection is visible at a time.
//!
//! Think of the content represented by a tab.
use std::collections::HashMap;

use mogwai::prelude::*;

use crate::id::{Id, IdPool};

/// Controls how [`Panes`] shows and hides pane content.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaneMode {
    /// Swap DOM nodes via [`ProxyChild::replace`] (default).
    ///
    /// The previously visible pane's DOM subtree is removed from the document,
    /// so transient state (scroll position, iframe content, form input, etc.)
    /// is lost on every switch.
    #[default]
    Replace,
    /// Keep all panes in the DOM, toggling `display: none` on wrapper `div`
    /// slots.
    ///
    /// Every pane is appended to the container once (inside its own wrapper
    /// `div`) and never removed. Switching panes only changes which slot is
    /// visible, preserving scroll position, iframe state, and any other
    /// transient DOM state.
    Retain,
}

/// Result of removing an item from [`Panes`].
pub struct PaneItemRemoval<T> {
    /// [`Id`] of the item removed.
    pub id: Id<T>,
    /// The item that was removed.
    pub item: T,
    /// Whether or not the pane was active when it was removed.
    pub was_selected: bool,
}

/// Static panes container.
///
/// Stores panes as concrete values. Visibility is controlled by the
/// [`PaneMode`] chosen at construction time:
///
/// * [`PaneMode::Replace`] (the default, via [`Panes::new`]) — swaps DOM nodes
///   with [`ProxyChild::replace`].
/// * [`PaneMode::Retain`] (via [`Panes::new_retained`]) — keeps every pane in
///   the DOM inside a wrapper `div` and toggles `display: none`.
#[derive(ViewChild, ViewProperties)]
pub struct Panes<V: View, T> {
    #[child]
    #[properties]
    wrapper: V::Element,
    mode: PaneMode,
    id_pool: IdPool<T>,
    current_id: Option<Id<T>>,
    child: ProxyChild<V>,
    slots: HashMap<Id<T>, V::Element>,
    default_slot: Option<V::Element>,
    default_pane: T,
    panes: HashMap<Id<T>, T>,
}

impl<V: View, T: ViewChild<V>> Panes<V, T> {
    /// Create a new panes container using [`PaneMode::Replace`].
    ///
    /// The given `pane` is shown as the default content. When [`select`] is
    /// called the default content is replaced with the selected pane's DOM
    /// nodes via [`ProxyChild::replace`].
    ///
    /// [`select`]: Panes::select
    pub fn new(wrapper: V::Element, pane: T) -> Self {
        let child = ProxyChild::new(&pane);
        wrapper.append_child(&child);
        Self {
            wrapper,
            mode: PaneMode::Replace,
            id_pool: IdPool::default(),
            current_id: None,
            child,
            slots: HashMap::new(),
            default_slot: None,
            default_pane: pane,
            panes: HashMap::new(),
        }
    }

    /// Create a new panes container using [`PaneMode::Retain`].
    ///
    /// Every pane (including the default) is wrapped in a `div` slot element
    /// and appended to the container once. Switching panes toggles
    /// `display: none` on the slot wrappers so that DOM state (scroll position,
    /// iframe content, etc.) is preserved across switches.
    pub fn new_retained(wrapper: V::Element, pane: T) -> Self {
        let default_slot = V::Element::new("div");
        default_slot.append_child(&pane);
        wrapper.set_style("display", "flex");
        wrapper.set_style("flex-direction", "column");
        wrapper.append_child(&default_slot);

        // ProxyChild is unused in Retain mode but we need a value for the
        // field. Create it from an empty text node so it holds no meaningful
        // DOM state.
        let placeholder = V::Text::new("");
        let child = ProxyChild::new(&placeholder);

        Self {
            wrapper,
            mode: PaneMode::Retain,
            id_pool: IdPool::default(),
            current_id: None,
            child,
            slots: HashMap::new(),
            default_slot: Some(default_slot),
            default_pane: pane,
            panes: HashMap::new(),
        }
    }

    /// Returns the [`PaneMode`] this container was created with.
    pub fn mode(&self) -> PaneMode {
        self.mode
    }

    /// Add a pane to the container.
    ///
    /// In [`PaneMode::Retain`], the pane is immediately appended to the DOM
    /// inside a hidden wrapper `div`.
    ///
    /// Returns the [`Id`] allocated for this pane, which can be used with
    /// [`select`] to show or access this pane later.
    ///
    /// [`select`]: Panes::select
    pub fn add_pane(&mut self, pane: T) -> Id<T> {
        let id = self.id_pool.get_id();
        if self.mode == PaneMode::Retain {
            let slot = V::Element::new("div");
            slot.set_style("display", "none");
            slot.set_style("flex", "1");
            slot.set_style("min-height", "0");
            slot.append_child(&pane);
            self.wrapper.append_child(&slot);
            self.slots.insert(id.clone(), slot);
        }
        self.panes.insert(id.clone(), pane);
        id
    }

    /// Show the pane with the given `id`, hiding the previously active pane.
    ///
    /// In [`PaneMode::Replace`], the selected pane's DOM nodes replace the
    /// current content via [`ProxyChild::replace`].
    ///
    /// In [`PaneMode::Retain`], the previously active slot gets
    /// `display: none` and the newly active slot has that style removed.
    ///
    /// Returns `true` if the pane was found and selection changed, `false` otherwise.
    pub fn select(&mut self, id: &Id<T>) -> bool {
        if Some(id) != self.current_id.as_ref() {
            match self.mode {
                PaneMode::Replace => {
                    if let Some(pane) = self.panes.get(id) {
                        self.current_id = Some(id.clone());
                        self.child.replace(&self.wrapper, pane);
                        return true;
                    }
                }
                PaneMode::Retain => {
                    if self.panes.contains_key(id) {
                        // Hide the currently active slot.
                        if let Some(old_id) = &self.current_id {
                            if let Some(slot) = self.slots.get(old_id) {
                                slot.set_style("display", "none");
                            }
                        } else if let Some(default_slot) = &self.default_slot {
                            default_slot.set_style("display", "none");
                        }

                        // Show the newly selected slot.
                        if let Some(slot) = self.slots.get(id) {
                            slot.remove_style("display");
                        }
                        self.current_id = Some(id.clone());
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Returns a reference to the currently visible pane.
    pub fn current_pane(&self) -> Option<&T> {
        match &self.current_id {
            Some(id) => self.panes.get(id).or(Some(&self.default_pane)),
            None => Some(&self.default_pane),
        }
    }

    /// Returns a mutable reference to the currently visible pane.
    pub fn current_pane_mut(&mut self) -> Option<&mut T> {
        match &self.current_id {
            Some(id) => {
                if self.panes.contains_key(id) {
                    Some(self.panes.get_mut(id).unwrap())
                } else {
                    Some(&mut self.default_pane)
                }
            }
            None => Some(&mut self.default_pane),
        }
    }

    /// Returns a reference to the pane with the given `id`, if it exists.
    pub fn get_pane(&self, id: &Id<T>) -> Option<&T> {
        self.panes.get(id)
    }

    /// Returns a mutable reference to the pane with the given `id`, if it exists.
    pub fn get_pane_mut(&mut self, id: &Id<T>) -> Option<&mut T> {
        self.panes.get_mut(id)
    }

    /// Remove the pane with the given [`Id`], if any.
    pub fn remove_by_id(&mut self, id: &Id<T>) -> Option<PaneItemRemoval<T>> {
        let pane = self.panes.remove(id)?;
        let was_selected = self.current_id.as_ref() == Some(id);
        Some(PaneItemRemoval {
            id: id.clone(),
            item: pane,
            was_selected,
        })
    }

    /// Returns an iterator over all panes.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.panes.values()
    }

    /// Returns a mutable iterator over all panes.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.panes.values_mut()
    }
}

#[cfg(feature = "library")]
pub mod library {
    //! Storybook sandbox for [`Panes`] in [`PaneMode::Retain`] mode.

    use std::collections::HashMap;

    use futures_lite::FutureExt;
    use mogwai::{prelude::*, web::WebElement};

    use crate::{
        components::{
            button::Button,
            icon::{Icon, IconGlyph, IconSize, IconStyle},
            tab::{TabItemRemoval, TabList, TabListEvent},
        },
        id::Id,
    };

    use super::Panes;

    /// Library item demonstrating retained panes.
    ///
    /// Three tabs with scrollable content and a live timer prove that both
    /// scroll position and async state survive tab switches.
    #[derive(ViewChild)]
    pub struct PaneRetainLibraryItem<V: View> {
        #[child]
        div: V::Element,
        tabs: TabList<V, V::Element>,
        panes: Panes<V, V::Element>,
        tab_ids_to_pane_ids: HashMap<Id<V::Element>, Id<V::Element>>,
        new_item_input: V::Element,
        new_item_button: Button<V>,
        close_icons: Vec<(Id<V::Element>, Icon<V>)>,
        timer_text: V::Text,
        seconds: u32,
    }

    impl<V: View> Default for PaneRetainLibraryItem<V> {
        fn default() -> Self {
            let new_item_button = {
                let mut b = Button::new("", None);
                b.set_has_icon(true);
                b.get_icon_mut()
                    .set_glyph(crate::components::icon::IconGlyph::Plus);
                b
            };
            rsx! {
                let div = div() {
                    let list = {TabList::default()}
                    let pane_wrapper = div() {}

                    // TODO: use forms here when they are ready
                    div(class = "row container-fluid border-top") {
                        fieldset() {
                            legend() {
                                "Add a new pane"
                            }
                            let new_item_input = input(type = "text") {}
                            {&new_item_button}
                        }
                    }
                }
            }

            // -- Scrollable A ------------------------------------------------
            rsx! {
                let pane_a = div(
                    style:overflow_y = "auto",
                    style:max_height = "200px",
                    style:border = "1px solid #dee2e6",
                    style:padding = "1rem",
                    style:margin_top = "0.5rem",
                ) {
                    h5() { "Pane A" }
                    p(class = "text-muted") {
                        "Scroll down, switch tabs, then come back."
                        br{}
                        "Your scroll position will be preserved."
                    }
                }
            }
            for i in 1..=20 {
                let text = V::Text::new(format!("A — paragraph {i} of 20."));
                rsx! { let p = p() { {text} } }
                pane_a.append_child(&p);
            }

            // -- Scrollable B ------------------------------------------------
            rsx! {
                let pane_b = div(
                    style:overflow_y = "auto",
                    style:max_height = "200px",
                    style:border = "1px solid #dee2e6",
                    style:padding = "1rem",
                    style:margin_top = "0.5rem",
                ) {
                    h5() { "Pane B" }
                    p(class = "text-muted") {
                        "This is a different pane with its own scroll state."
                    }
                }
            }
            for i in 1..=20 {
                let text = V::Text::new(format!("B — paragraph {i} of 20."));
                rsx! { let p = p() { {text} } }
                pane_b.append_child(&p);
            }

            // -- Timer -------------------------------------------------------
            let timer_text = V::Text::new("0 seconds elapsed");
            rsx! {
                let pane_timer = div(
                    style:overflow_y = "auto",
                    style:max_height = "200px",
                    style:border = "1px solid #dee2e6",
                    style:padding = "1rem",
                    style:margin_top = "0.5rem",
                ) {
                    h5() { "Timer Pane" }
                    p(class = "text-muted") {
                        "This timer keeps running even when this tab is hidden."
                        br{}
                        "Scroll down, switch away, then come back."
                    }
                    p(class = "fw-bold") { {&timer_text} }
                }
            }
            for i in 1..=15 {
                let text = V::Text::new(format!("Timer — filler paragraph {i} of 15."));
                rsx! { let p = p() { {text} } }
                pane_timer.append_child(&p);
            }

            // -- Assemble ----------------------------------------------------
            rsx! {
                let default_pane = p(class = "text-muted mt-2") {
                    "Select a tab above."
                }
            }

            let panes = Panes::new_retained(pane_wrapper, default_pane);

            let mut item = Self {
                div,
                tabs: list,
                panes,
                timer_text,
                seconds: 0,
                new_item_input,
                new_item_button,
                close_icons: vec![],
                tab_ids_to_pane_ids: Default::default(),
            };

            let (tab_a_id, _) = item.add(
                {
                    rsx! { let s = span() { "Scrollable A" } }
                    s
                },
                pane_a,
            );

            let _ = item.add(
                {
                    rsx! { let s = span() { "Scrollable B" } }
                    s
                },
                pane_b,
            );

            let _ = item.add(
                {
                    rsx! { let s = span() { "Timer" } }
                    s
                },
                pane_timer,
            );

            // Show the first pane by default.
            item.select(&tab_a_id);

            item
        }
    }

    impl<V: View> PaneRetainLibraryItem<V> {
        fn add(
            &mut self,
            tab_item: V::Element,
            pane_item: V::Element,
        ) -> (Id<V::Element>, Id<V::Element>) {
            let tab_id = self.tabs.push(tab_item);
            let pane_id = self.panes.add_pane(pane_item);
            self.tab_ids_to_pane_ids
                .insert(tab_id.clone(), pane_id.clone());
            (tab_id, pane_id)
        }

        fn select(&mut self, id: &Id<V::Element>) {
            self.tabs.select_by_id(id);
            if let Some(id) = self.tab_ids_to_pane_ids.get(id) {
                let _ = self.panes.select(id);
            }
        }

        pub async fn step(&mut self) {
            enum Ev<V: View, T> {
                Timer,
                Tab(TabListEvent<V, T>),
                NewItem(String),
                Remove(Id<V::Element>),
            }
            let timer_fut = async {
                mogwai::time::wait_millis(1000).await;
                Ev::Timer
            };
            let list_fut = async {
                let event = self.tabs.step().await;
                Ev::Tab(event)
            };
            let new_tab_fut = async {
                let _event = self.new_item_button.step().await;
                let s = self
                    .new_item_input
                    .dyn_el(|el: &web_sys::HtmlInputElement| el.value())
                    .unwrap();
                Ev::NewItem(s)
            };
            let closes = self
                .close_icons
                .iter()
                .map(|(id, icon)| async {
                    let _ = icon.listen("click").next().await;
                    Ev::Remove::<V, V::Element>(id.clone())
                })
                .collect::<Vec<_>>();
            let close_tab_fut = mogwai::future::race_all(closes);
            let result = timer_fut
                .or(list_fut)
                .or(new_tab_fut)
                .or(close_tab_fut)
                .await;
            match result {
                Ev::Tab(TabListEvent::ItemClicked {
                    id,
                    index: _,
                    event: _,
                }) => {
                    self.select(&id);
                }
                Ev::Timer => {
                    self.seconds += 1;
                    self.timer_text
                        .set_text(format!("{} seconds elapsed", self.seconds));
                }
                Ev::NewItem(s) => {
                    let close_icon =
                        Icon::with_style(IconGlyph::Xmark, IconSize::Regular, IconStyle::Solid);
                    rsx! {
                        let item = div() {
                            {&close_icon}
                            {format!("Tab {}", self.close_icons.len()).into_text::<V>()}
                        }
                    }
                    rsx! {
                        let pane = div() {
                            span() {
                                {s.into_text::<V>()}
                            }
                        }
                    }
                    let (tab_id, pane_id) = self.add(item, pane);
                    self.close_icons.push((tab_id.clone(), close_icon));

                    self.tabs.select_by_id(&tab_id);
                    let _ = self.panes.select(&pane_id);
                }
                Ev::Remove(id) => {
                    if let Some(TabItemRemoval {
                        id: _,
                        index,
                        item: _,
                        was_selected: true,
                    }) = self.tabs.remove_by_id(&id)
                    {
                        if let Some(pane_id) = self.tab_ids_to_pane_ids.remove(&id) {
                            let _ = self.panes.remove_by_id(&pane_id);
                        }

                        let selected_index = index.min(self.tab_ids_to_pane_ids.len() - 1);
                        let id = self.tabs.get(selected_index).unwrap().id().clone();
                        self.select(&id);
                    }
                }
            }
        }
    }
}

/// Factory-based panes container.
///
/// Stores pane *factories* (`Box<dyn FnMut() -> T>`). Each time a pane is
/// selected it is re-created from the factory, ensuring a fresh state.
#[derive(ViewChild, ViewProperties)]
pub struct RestartPanes<V: View, T> {
    #[child]
    #[properties]
    wrapper: V::Element,
    id_pool: IdPool<T>,
    current_id: Option<Id<T>>,
    child: ProxyChild<V>,
    pane: T,
    panes: HashMap<Id<T>, Box<dyn FnMut() -> T>>,
}

impl<V: View, T: ViewChild<V>> RestartPanes<V, T> {
    /// Create a new factory-based panes container.
    ///
    /// The given `default_pane` is shown initially. Use [`add_pane`] to add
    /// pane factories that will be recreated each time they are selected.
    ///
    /// [`add_pane`]: RestartPanes::add_pane
    pub fn new(wrapper: V::Element, default_pane: T) -> Self {
        let child = ProxyChild::new(&default_pane);
        wrapper.append_child(&child);
        Self {
            wrapper,
            id_pool: IdPool::default(),
            current_id: None,
            child,
            pane: default_pane,
            panes: HashMap::new(),
        }
    }

    /// Add a pane factory to the container.
    ///
    /// The factory is a closure that creates a new pane each time this pane is
    /// selected. If this is the first pane added, it is automatically selected.
    ///
    /// Returns the [`Id`] allocated for this pane factory, which can be used
    /// with [`select`] to show this pane.
    ///
    /// [`select`]: RestartPanes::select
    pub fn add_pane(&mut self, create: impl FnMut() -> T + 'static) -> Id<T> {
        let id = self.id_pool.get_id();
        let was_empty = self.panes.is_empty();
        self.panes.insert(id.clone(), Box::new(create));
        if was_empty {
            log::info!("selecting first pane");
            let _ = self.select(&id);
        }
        id
    }

    /// Show the pane with the given `id`, hiding the previously active pane.
    ///
    /// The pane is recreated fresh from its factory. Returns `true` if the
    /// pane was found and selected, `false` otherwise.
    pub fn select(&mut self, id: &Id<T>) -> bool {
        if Some(id) != self.current_id.as_ref() {
            if let Some(f) = self.panes.get_mut(id) {
                let pane = f();
                self.pane = pane;
                self.child.replace(&self.wrapper, &self.pane);
                self.current_id = Some(id.clone());
                return true;
            }
        }
        false
    }

    /// Returns a reference to the currently displayed pane.
    pub fn current_pane(&self) -> &T {
        &self.pane
    }

    /// Returns a mutable reference to the currently displayed pane.
    pub fn current_pane_mut(&mut self) -> &mut T {
        &mut self.pane
    }
}
