//! Window / page panes.
//!
//! A type for storing panes of HTML and their logic.
//!
//! A "pane" is a spot of HTML suitable for storing in a collection, where only
//! one item in that collection is visible at a time.
//!
//! Think of the content represented by a tab.
use mogwai::prelude::*;

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

/// Static panes container.
///
/// Stores panes as concrete values. Visibility is controlled by the
/// [`PaneMode`] chosen at construction time:
///
/// * [`PaneMode::Replace`] (the default, via [`Panes::new`]) — swaps DOM nodes
///   with [`ProxyChild::replace`].
/// * [`PaneMode::Retain`] (via [`Panes::new_retained`]) — keeps every pane in
///   the DOM inside a wrapper `div` and toggles `display: none`.
#[derive(ViewChild)]
pub struct Panes<V: View, T> {
    #[child]
    wrapper: V::Element,
    mode: PaneMode,
    index: Option<usize>,
    child: ProxyChild<V>,
    slots: Vec<V::Element>,
    default_slot: Option<V::Element>,
    default_pane: T,
    panes: Vec<T>,
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
            index: None,
            child,
            slots: vec![],
            default_slot: None,
            default_pane: pane,
            panes: vec![],
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
            index: None,
            child,
            slots: vec![],
            default_slot: Some(default_slot),
            default_pane: pane,
            panes: vec![],
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
    pub fn add_pane(&mut self, pane: T) {
        if self.mode == PaneMode::Retain {
            let slot = V::Element::new("div");
            slot.set_style("display", "none");
            slot.set_style("flex", "1");
            slot.set_style("min-height", "0");
            slot.append_child(&pane);
            self.wrapper.append_child(&slot);
            self.slots.push(slot);
        }
        self.panes.push(pane);
    }

    /// Show the pane at `index`, hiding the previously active pane.
    ///
    /// In [`PaneMode::Replace`], the selected pane's DOM nodes replace the
    /// current content via [`ProxyChild::replace`].
    ///
    /// In [`PaneMode::Retain`], the previously active slot gets
    /// `display: none` and the newly active slot has that style removed.
    pub fn select(&mut self, index: usize) {
        if Some(index) != self.index {
            match self.mode {
                PaneMode::Replace => {
                    if let Some(pane) = self.panes.get(index) {
                        self.index = Some(index);
                        self.child.replace(&self.wrapper, pane);
                    }
                }
                PaneMode::Retain => {
                    if index < self.panes.len() {
                        // Hide the currently active slot.
                        self.active_slot().set_style("display", "none");

                        // Show the newly selected slot.
                        self.slots[index].remove_style("display");
                        self.index = Some(index);
                    }
                }
            }
        }
    }

    /// Returns a reference to the currently visible pane.
    pub fn get_pane(&self) -> &T {
        match self.index {
            Some(n) => self.panes.get(n).unwrap_or(&self.default_pane),
            None => &self.default_pane,
        }
    }

    /// Returns a mutable reference to the currently visible pane.
    pub fn get_pane_mut(&mut self) -> &mut T {
        match self.index {
            Some(n) => {
                if let Some(pane) = self.panes.get_mut(n) {
                    pane
                } else {
                    &mut self.default_pane
                }
            }
            None => &mut self.default_pane,
        }
    }

    /// Returns a reference to the pane at `index`, if it exists.
    pub fn get_pane_at(&self, index: usize) -> Option<&T> {
        self.panes.get(index)
    }

    /// Returns a mutable reference to the pane at `index`, if it exists.
    pub fn get_pane_at_mut(&mut self, index: usize) -> Option<&mut T> {
        self.panes.get_mut(index)
    }

    /// Returns the slot element that is currently visible.
    ///
    /// In [`PaneMode::Retain`] this is the wrapper `div` for the active pane
    /// (or the default slot if no pane has been selected). In
    /// [`PaneMode::Replace`] this method is not called.
    fn active_slot(&self) -> &V::Element {
        match self.index {
            Some(n) => &self.slots[n],
            None => self
                .default_slot
                .as_ref()
                .expect("Retain mode has a default slot"),
        }
    }
}

#[cfg(feature = "library")]
pub mod library {
    //! Storybook sandbox for [`Panes`] in [`PaneMode::Retain`] mode.

    use futures_lite::FutureExt;
    use mogwai::prelude::*;

    use crate::components::tab::{TabList, TabListEvent};

    use super::Panes;

    /// Library item demonstrating retained panes.
    ///
    /// Three tabs with scrollable content and a live timer prove that both
    /// scroll position and async state survive tab switches.
    #[derive(ViewChild)]
    pub struct PaneRetainLibraryItem<V: View> {
        #[child]
        div: V::Element,
        list: TabList<V, V::Element>,
        panes: Panes<V, V::Element>,
        timer_text: V::Text,
        seconds: u32,
    }

    impl<V: View> Default for PaneRetainLibraryItem<V> {
        fn default() -> Self {
            rsx! {
                let div = div() {
                    let list = {TabList::default()}
                    let pane_wrapper = div() {}
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
                list,
                panes,
                timer_text,
                seconds: 0,
            };

            item.list.push({
                rsx! { let s = span() { "Scrollable A" } }
                s
            });
            item.panes.add_pane(pane_a);

            item.list.push({
                rsx! { let s = span() { "Scrollable B" } }
                s
            });
            item.panes.add_pane(pane_b);

            item.list.push({
                rsx! { let s = span() { "Timer" } }
                s
            });
            item.panes.add_pane(pane_timer);

            // Show the first pane by default.
            item.select(0);

            item
        }
    }

    impl<V: View> PaneRetainLibraryItem<V> {
        fn select(&mut self, index: usize) {
            self.list.select(index);
            self.panes.select(index);
        }

        pub async fn step(&mut self) {
            let timer_fut = async {
                mogwai::time::wait_millis(1000).await;
                None::<TabListEvent<V>>
            };
            let list_fut = async {
                let event = self.list.step().await;
                Some(event)
            };
            match timer_fut.or(list_fut).await {
                Some(TabListEvent::ItemClicked { index, event: _ }) => {
                    self.select(index);
                }
                None => {
                    self.seconds += 1;
                    self.timer_text
                        .set_text(format!("{} seconds elapsed", self.seconds));
                }
            }
        }
    }
}

/// Factory-based panes container.
///
/// Stores pane *factories* (`Box<dyn FnMut() -> T>`). Each time a pane is
/// selected it is re-created from the factory, ensuring a fresh state.
#[derive(ViewChild)]
pub struct RestartPanes<V: View, T> {
    #[child]
    wrapper: V::Element,
    index: Option<usize>,
    child: ProxyChild<V>,
    pane: T,
    panes: Vec<Box<dyn FnMut() -> T>>,
}

impl<V: View, T: ViewChild<V>> RestartPanes<V, T> {
    pub fn new(wrapper: V::Element, default_pane: T) -> Self {
        let child = ProxyChild::new(&default_pane);
        wrapper.append_child(&child);
        Self {
            wrapper,
            index: None,
            child,
            pane: default_pane,
            panes: vec![],
        }
    }

    pub fn add_pane(&mut self, create: impl FnMut() -> T + 'static) {
        self.panes.push(Box::new(create));
        if self.panes.len() == 1 {
            log::info!("selecting tab 0");
            self.select(0);
        }
    }

    pub fn select(&mut self, index: usize) {
        if Some(index) != self.index {
            if let Some(f) = self.panes.get_mut(index) {
                let pane = f();
                self.pane = pane;
                self.child.replace(&self.wrapper, &self.pane);
                self.index = Some(index);
            }
        }
    }

    pub fn get_pane(&self) -> &T {
        &self.pane
    }

    pub fn get_pane_mut(&mut self) -> &mut T {
        &mut self.pane
    }
}
