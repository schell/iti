//! Page tabs (Bootstrap nav-tabs).
//!
//! This module provides [`TabPanel`], a component that pairs a horizontal tab
//! bar with a content area. Each tab `T` and its associated pane `P` are stored
//! together in a single [`Vec`] of [`TabOrSpacer`] entries, so that
//! [`StepWithMut`] can hand a closure `&mut `[`TabPanelEntry`] with direct
//! [`TabPanelEntry::tab`] and [`TabPanelEntry::pane`] access. This lets callers
//! race tab-click events against pane-specific event loops in a single
//! future-race, which is essential for closable detail-panel tabs and other
//! interactive pane content.
//!
//! Spacers ([`TabSpacer`]) can be interleaved with tabs to control alignment
//! via [`TabPanel::set_alignment`] / [`TabAlignment`].
//!
//! # Typical usage
//!
//! ```ignore
//! let mut panel = TabPanel::<V, V::Element, MyPane>::new(default_pane);
//! let id = panel.push(tab_label, my_pane);
//! panel.select(&id);
//! // In the event loop:
//! let ev = panel
//!     .step_with_mut(|entry| {
//!         let on_click = entry.tab().on_click();
//!         let pane = entry.pane_mut();
//!         async {
//!             // Race the tab click against the pane's step.
//!             let click = async { on_click.next().await; TabPanelEvent::Tabs(...) };
//!             let pane_ev = async { pane.step_mut().await; TabPanelEvent::Panes(...) };
//!             click.or(pane_ev).await
//!         }
//!         .boxed_local()
//!     })
//!     .await;
//! ```
//!
//! When all you need is tab selection (no pane stepping), use [`StepMut`]
//! instead, which auto-selects the clicked tab and returns a [`TabListEvent`].
use std::{future::Future, pin::Pin};

use futures_lite::FutureExt;
use mogwai::prelude::*;

use crate::id::{Id, IdPool};

/// A single tab within a [`TabPanel`].
///
/// Generic over the view type `V` and the tab's inner content type `T`. The
/// `is_active` flag is a `Proxy<bool>` that reactively toggles the
/// `nav-link active` CSS class on the underlying `<a>` element.
///
/// Each tab may optionally show a close button on the right side of the tab
/// label. The close button uses the `title-bar-close` CSS class (the same
/// Platinum close-box used by [`crate::components::title_bar::TitleBar`]).
/// When clicked, it emits [`TabListEvent::CloseClicked`] from the owning
/// [`TabPanel`].
///
/// Constructed internally by [`TabPanel::push`] / [`TabPanel::insert`]; users
/// rarely call [`TabListItem::new`] directly.
#[derive(ViewChild, ViewProperties)]
pub struct TabListItem<V: View, T> {
    #[child]
    #[properties]
    li: V::Element,
    #[allow(dead_code)]
    a: V::Element,
    on_click: V::EventListener,
    close_click: V::EventListener,
    close_visible: Proxy<bool>,
    closable: bool,
    inner: T,
    is_active: Proxy<bool>,
    id: Id<T>,
}

impl<V: View, T: ViewChild<V>> TabListItem<V, T> {
    /// Create a new tab item with the given [`Id`] and inner content.
    ///
    /// The close button is hidden by default. Use
    /// [`TabListItem::set_closable`] to show it.
    ///
    /// This is called by [`TabPanel::push`] / [`TabPanel::insert`]; you
    /// usually don't construct `TabListItem` values yourself.
    pub fn new(id: Id<T>, inner: T) -> Self {
        let mut is_active = Proxy::new(false);
        let mut close_visible = Proxy::new(false);
        rsx! {
            let li = li(class = "nav-item", style:cursor = "pointer") {
                let a = a(
                    class = is_active(active => if *active {
                        "nav-link active"
                    } else {
                        "nav-link"
                    }),
                    on:click = on_click,
                ) {
                    {&inner}
                    button(
                        type = "button",
                        class = "title-bar-close nav-tab-close",
                        style:display = close_visible(v => if *v {
                            "inline-block"
                        } else {
                            "none"
                        }),
                        on:click = close_click,
                    ) {}
                }
            }
        }

        Self {
            li,
            a,
            on_click,
            close_click,
            close_visible,
            closable: false,
            inner,
            is_active,
            id,
        }
    }

    /// Returns a reference to the tab's inner content.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Returns a mutable reference to the tab's inner content.
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Get a reference to this item's [`Id`].
    pub fn id(&self) -> &Id<T> {
        &self.id
    }

    /// Whether this tab is currently the active (selected) tab.
    pub fn is_active(&self) -> bool {
        *self.is_active
    }

    /// Get a reference to this tab's click event listener.
    pub fn on_click(&self) -> &V::EventListener {
        &self.on_click
    }

    /// Get a reference to this tab's close-button click event listener.
    pub fn close_listener(&self) -> &V::EventListener {
        &self.close_click
    }

    /// Show or hide this tab's close button.
    pub fn set_closable(&mut self, closable: bool) {
        self.closable = closable;
        self.close_visible.set(closable);
    }

    /// Whether this tab's close button is visible.
    pub fn is_closable(&self) -> bool {
        self.closable
    }

    /// Borrow the click listener, close listener, and inner content
    /// simultaneously.
    ///
    /// Returns shared references to the tab's `on_click` and `close_click`
    /// listeners and a mutable reference to the inner content `T`, allowing
    /// all three to be used in the same scope (e.g. racing a tab-click future
    /// and a close-click future against a pane-step future in a
    /// `step_with_mut` closure).
    pub fn split_borrows(&mut self) -> (&V::EventListener, &V::EventListener, &mut T) {
        (&self.on_click, &self.close_click, &mut self.inner)
    }
}

/// Event emitted by a [`TabPanel`].
///
/// Returned by [`TabPanel::step_mut`] (via [`StepMut`]) and surfaced inside
/// the [`TabPanelEvent::Tabs`] variant when using [`StepWithMut`].
pub enum TabListEvent<V: View, T, P> {
    /// A tab was clicked.
    ItemClicked {
        /// The [`Id`] of the clicked tab.
        id: Id<T>,
        /// The entry index (includes spacers).
        index: usize,
        /// The underlying DOM event.
        event: V::Event,
    },
    /// A tab's close button was clicked. The tab has already been removed
    /// from the panel; the inner content `T` and pane `P` are returned for
    /// cleanup.
    CloseClicked {
        /// The [`Id`] of the closed tab.
        id: Id<T>,
        /// The entry index before removal.
        index: usize,
        /// The tab's inner content.
        item: T,
        /// The tab's associated pane.
        pane: P,
    },
}

/// Result of removing an item from a [`TabPanel`].
pub struct TabItemRemoval<T> {
    /// [`Id`] of the item removed.
    pub id: Id<T>,
    /// The index of the item before it was removed.
    pub index: usize,
    /// The item that was removed.
    pub item: T,
    /// Whether or not the tab was active when it was removed.
    pub was_selected: bool,
}

/// Result of removing an entry (tab or spacer) from a [`TabPanel`].
pub enum RemovedEntry<T, P> {
    /// A tab was removed, returning the tab removal info and its pane.
    Tab(TabItemRemoval<T>, P),
    /// A spacer was removed.
    Spacer,
}

/// A flexible spacer element within a [`TabPanel`].
///
/// Spacers are `flex-grow: 1` elements that absorb available space in the tab
/// bar. Insert them before, after, or between tabs to control alignment.
///
/// Each spacer has a unique [`Id<T>`] (allocated from the same pool as tabs)
/// so it can be individually identified and removed via [`TabPanel::remove_by_id`].
#[derive(ViewChild)]
pub struct TabSpacer<V: View, T> {
    #[child]
    li: V::Element,
    id: Id<T>,
}

impl<V: View, T> TabSpacer<V, T> {
    fn new(id: Id<T>) -> Self {
        rsx! {
            let li = li(class = "nav-tab-spacer") {}
        }
        Self { li, id }
    }

    /// Get a reference to this spacer's [`Id`].
    pub fn id(&self) -> &Id<T> {
        &self.id
    }
}

/// A pane content element and its wrapper slot `<div>`.
///
/// The slot is the DOM element appended to [`TabPanel`]'s content area.
/// Show/hide is controlled by toggling `display: none` on the slot via
/// [`TabbedPane::show`] / [`TabbedPane::hide`].
pub struct TabbedPane<V: View, P> {
    /// The pane content.
    pane: P,
    /// Wrapper `<div>` holding the pane in the DOM.
    /// Toggled `display: none` to show/hide the pane.
    slot: V::Element,
}

impl<V: View, P: ViewChild<V>> TabbedPane<V, P> {
    /// Create a new `TabbedPane` wrapping the given pane content.
    ///
    /// The slot `<div>` is created with `display: none` (hidden), `flex: 1`,
    /// and `min-height: 0` so it fills the content area when shown.
    pub fn new(pane: P) -> Self {
        let slot = V::Element::new("div");
        slot.set_style("display", "none");
        slot.set_style("flex", "1");
        slot.set_style("min-height", "0");
        slot.append_child(&pane);
        Self { pane, slot }
    }

    /// Returns a reference to the pane content.
    pub fn pane(&self) -> &P {
        &self.pane
    }

    /// Returns a mutable reference to the pane content.
    pub fn pane_mut(&mut self) -> &mut P {
        &mut self.pane
    }

    /// Returns a reference to the wrapper slot element (for DOM operations
    /// like `parent.remove_child(&slot)`).
    pub fn slot(&self) -> &V::Element {
        &self.slot
    }

    /// Show this pane (removes `display: none` from the slot).
    pub fn show(&self) {
        self.slot.remove_style("display");
    }

    /// Hide this pane (sets `display: none` on the slot).
    pub fn hide(&self) {
        self.slot.set_style("display", "none");
    }

    /// Consume this `TabbedPane`, returning the pane content and slot element.
    pub fn into_parts(self) -> (P, V::Element) {
        (self.pane, self.slot)
    }
}

/// A tab + pane pair stored in a [`TabPanel`].
///
/// This is the type the [`StepWithMut`] closure receives a mutable reference
/// to, allowing callers to step both the tab `T` and the pane `P` together in
/// a single race. Use [`TabPanelEntry::tab`] / [`TabPanelEntry::tab_mut`] and
/// [`TabPanelEntry::pane`] / [`TabPanelEntry::pane_mut`] to access the
/// components.
///
/// ```ignore
/// panel.step_with_mut(|entry| {
///     let on_click = entry.tab().on_click();
///     let pane = entry.pane_mut();
///     async { /* race on_click.next() vs pane.step_mut() */ }.boxed_local()
/// })
/// ```
pub struct TabPanelEntry<V: View, T, P> {
    tab: TabListItem<V, T>,
    pane: TabbedPane<V, P>,
}

impl<V: View, T, P: ViewChild<V>> TabPanelEntry<V, T, P> {
    /// Returns a reference to the tab list item (the clickable tab header).
    pub fn tab(&self) -> &TabListItem<V, T> {
        &self.tab
    }

    /// Returns a mutable reference to the tab list item.
    pub fn tab_mut(&mut self) -> &mut TabListItem<V, T> {
        &mut self.tab
    }

    /// Returns a reference to the pane content.
    pub fn pane(&self) -> &P {
        self.pane.pane()
    }

    /// Returns a mutable reference to the pane content.
    pub fn pane_mut(&mut self) -> &mut P {
        self.pane.pane_mut()
    }

    /// Borrow the tab and pane simultaneously.
    ///
    /// Returns a shared reference to the tab and a mutable reference to the
    /// pane, allowing both to be used in the same scope (e.g. racing a
    /// tab-click future against a pane-step future in a `step_with_mut`
    /// closure).
    pub fn split_tab_pane(&mut self) -> (&TabListItem<V, T>, &mut P) {
        (&self.tab, self.pane.pane_mut())
    }
}

/// An entry in a [`TabPanel`] — either a tab+pane pair or a spacer.
pub enum TabOrSpacer<V: View, T, P> {
    /// A tab and its associated pane.
    Item(TabPanelEntry<V, T, P>),
    /// A flexible spacer element (no tab, no pane).
    Spacer(TabSpacer<V, T>),
}

impl<V: View, T, P> TabOrSpacer<V, T, P> {
    /// Get the underlying tab `li` element (for Item) or spacer `li` (for
    /// Spacer), for DOM operations.
    fn element(&self) -> &V::Element {
        match self {
            TabOrSpacer::Item(entry) => &entry.tab.li,
            TabOrSpacer::Spacer(spacer) => &spacer.li,
        }
    }

    /// Returns `true` if this entry is a spacer.
    pub fn is_spacer(&self) -> bool {
        matches!(self, TabOrSpacer::Spacer(_))
    }

    /// Try to get the entry as a tab+pane pair reference.
    pub fn as_item(&self) -> Option<&TabPanelEntry<V, T, P>> {
        match self {
            TabOrSpacer::Item(entry) => Some(entry),
            TabOrSpacer::Spacer(_) => None,
        }
    }

    /// Try to get the entry as a mutable tab+pane pair reference.
    pub fn as_item_mut(&mut self) -> Option<&mut TabPanelEntry<V, T, P>> {
        match self {
            TabOrSpacer::Item(entry) => Some(entry),
            TabOrSpacer::Spacer(_) => None,
        }
    }
}

impl<V: View, T: ViewChild<V>, P: ViewChild<V>> ViewChild<V> for TabOrSpacer<V, T, P> {
    fn as_append_arg(&self) -> AppendArg<V, impl Iterator<Item = std::borrow::Cow<'_, V::Node>>> {
        match self {
            TabOrSpacer::Item(entry) => entry.tab.as_boxed_append_arg(),
            TabOrSpacer::Spacer(spacer) => spacer.as_boxed_append_arg(),
        }
    }
}

/// Alignment of tabs within a [`TabPanel`].
///
/// Controls the placement of spacers around the tab items.
pub enum TabAlignment {
    /// Tabs align to the start (left). A spacer is placed after all tabs.
    Start,
    /// Tabs are centered. Spacers are placed before and after all tabs.
    Center,
    /// Tabs align to the end (right). A spacer is placed before all tabs.
    End,
}

/// A panel topped with a tab list.
///
/// Stores tabs and panes together in a single [`Vec`] of
/// [`TabOrSpacer<V, T, P>`], eliminating the cross-collection join that
/// separate `TabList` + `Panes` + `tabs_to_panes` collections would require.
/// This allows [`StepWithMut`] to pass `&mut `[`TabPanelEntry<V, T, P>`] to a
/// closure, giving simultaneous mutable access to both the tab `T` and the
/// pane `P`.
///
/// # DOM structure
///
/// ```text
/// div.tab-panel
/// ├── ul.nav.nav-tabs      (tab bar)
/// │   ├── li.nav-item      (tab 0)
/// │   ├── li.nav-tab-spacer (optional spacer)
/// │   └── li.nav-item      (tab 1)
/// └── div.container-fluid   (content area)
///     ├── div               (default slot, shown when no tab is active)
///     ├── div               (pane 0 slot, display:none when inactive)
///     └── div               (pane 1 slot, display:none when inactive)
/// ```
///
/// Tab selection toggles `is_active` on the [`TabListItem`] (which reactively
/// sets the `active` CSS class) and toggles `display: none` on the slot `div`
/// to show/hide the corresponding pane.
///
/// # Stepping
///
/// Two step implementations are provided:
///
/// - [`StepMut`]: Races all tab click events. On a click, auto-selects the
///   clicked tab and returns a [`TabListEvent`]. Use this when you only need
///   tab selection and don't need to drive per-pane event loops.
/// - [`StepWithMut<TabPanelEntry<V, T, P>>`]: Calls the closure once per
///   `TabPanelEntry` (spacers are skipped), racing all returned futures. The
///   closure receives `&mut TabPanelEntry` and is responsible for racing the
///   tab click against the pane's own event loop. Use this when pane content
///   has its own `step` to drive.
///
/// [`StepWithMut<TabPanelEntry<V, T, P>>`]: StepWithMut
#[derive(ViewChild, ViewProperties)]
pub struct TabPanel<V: View, T, P> {
    #[child]
    #[properties]
    window: V::Element,
    ul: V::Element,
    content: V::Element,
    entries: Vec<TabOrSpacer<V, T, P>>,
    id_pool: IdPool<T>,
    default_slot: Option<V::Element>,
    default_closable: bool,
}

impl<V: View, T: ViewChild<V>, P: ViewChild<V>> TabPanel<V, T, P> {
    /// Create a new `TabPanel` with the default pane.
    ///
    /// The default pane is shown when no tabs are present or when all tabs have
    /// been removed. Once tabs are added, the default pane is hidden and the
    /// first tab's pane is shown instead.
    pub fn new(default_pane: P) -> Self {
        rsx! {
            let window = div(class = "tab-panel") {
                let ul = ul(class = "nav nav-tabs") {}
                let content = div(class = "container-fluid") { }
            }
        }
        let default_slot = V::Element::new("div");
        default_slot.append_child(&default_pane);
        content.set_style("display", "flex");
        content.set_style("flex-direction", "column");
        content.append_child(&default_slot);

        Self {
            window,
            ul,
            content,
            entries: vec![],
            id_pool: Default::default(),
            default_slot: Some(default_slot),
            default_closable: false,
        }
    }

    /// Return the number of entries (tabs and spacers).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if there are no entries (tabs or spacers).
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Return a reference to the [`TabListItem`] at the given entry index.
    ///
    /// Returns `None` if the index is out of bounds or points to a spacer.
    pub fn get(&self, index: usize) -> Option<&TabListItem<V, T>> {
        self.entries
            .get(index)
            .and_then(|e| e.as_item())
            .map(|e| &e.tab)
    }

    /// Iterator over all entries, including spacers.
    ///
    /// Entries are yielded in display order. Match on
    /// [`TabOrSpacer::Item`] to access the tab + pane pair, or
    /// [`TabOrSpacer::Spacer`] for spacers.
    pub fn iter(&self) -> impl Iterator<Item = &TabOrSpacer<V, T, P>> {
        self.entries.iter()
    }

    /// Mutable iterator over all entries, including spacers.
    ///
    /// See [`iter`](Self::iter) for ordering and matching details.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut TabOrSpacer<V, T, P>> {
        self.entries.iter_mut()
    }

    /// Push a new tab onto the end of the stack.
    ///
    /// Returns the [`Id`] allocated for the new tab.
    pub fn push(&mut self, tab: T, pane: P) -> Id<T> {
        let id = self.id_pool.get_id();
        let mut tab_item = TabListItem::new(id.clone(), tab);
        tab_item.set_closable(self.default_closable);

        let pane = TabbedPane::new(pane);
        self.content.append_child(pane.slot());

        let entry = TabPanelEntry {
            tab: tab_item,
            pane,
        };
        let kind = TabOrSpacer::Item(entry);
        self.ul.append_child(&kind);
        self.entries.push(kind);

        if self.len() == 1 {
            self.select_by_index(0);
        }
        id
    }

    /// Pop the last tab off the end of the list.
    ///
    /// Spacers at the end are skipped — this removes the last actual tab.
    /// Returns the tab's [`Id`], the tab inner `T`, and the pane `P`.
    pub fn pop(&mut self) -> Option<(Id<T>, T, P)> {
        let pos = self
            .entries
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, e)| e.as_item().map(|_| i))?;
        let kind = self.entries.remove(pos);
        match kind {
            TabOrSpacer::Item(entry) => {
                self.ul.remove_child(&entry.tab);
                let (pane, slot) = entry.pane.into_parts();
                self.content.remove_child(&slot);
                Some((entry.tab.id, entry.tab.inner, pane))
            }
            TabOrSpacer::Spacer(_) => unreachable!(),
        }
    }

    /// Insert a new tab at the given entry index and return a unique identifier.
    ///
    /// The index is a raw entry position (includes spacers). If the index is
    /// out of bounds the tab is appended to the end.
    pub fn insert(&mut self, index: usize, tab: T, pane: P) -> Id<T> {
        let id = self.id_pool.get_id();
        let mut tab_item = TabListItem::new(id.clone(), tab);
        tab_item.set_closable(self.default_closable);

        let pane = TabbedPane::new(pane);
        self.content.append_child(pane.slot());

        let entry = TabPanelEntry {
            tab: tab_item,
            pane,
        };
        let kind = TabOrSpacer::Item(entry);

        if index < self.entries.len() {
            self.ul
                .insert_child_before(&kind, Some(self.entries[index].element()));
            self.entries.insert(index, kind);
        } else {
            self.ul.append_child(&kind);
            self.entries.push(kind);
        }
        id
    }

    /// Remove a tab or spacer by its [`Id`].
    ///
    /// Returns [`RemovedEntry::Tab`] with the [`TabItemRemoval`] and associated
    /// pane, or [`RemovedEntry::Spacer`] if a spacer was removed, or `None` if
    /// no entry with the given [`Id`] was found.
    ///
    /// If the removed tab was the active tab, the nearest surviving neighbor is
    /// selected automatically (previous tab if available, otherwise the next
    /// tab). If no tabs remain, the default slot is shown.
    pub fn remove_by_id(&mut self, id: &Id<T>) -> Option<RemovedEntry<T, P>> {
        enum Found {
            Tab {
                entry_index: usize,
                was_selected: bool,
            },
            Spacer {
                entry_index: usize,
            },
        }

        let mut found = None;
        for (entry_i, entry) in self.entries.iter().enumerate() {
            match entry {
                TabOrSpacer::Item(item) if &item.tab.id == id => {
                    found = Some(Found::Tab {
                        entry_index: entry_i,
                        was_selected: *item.tab.is_active,
                    });
                    break;
                }
                TabOrSpacer::Spacer(s) if &s.id == id => {
                    found = Some(Found::Spacer {
                        entry_index: entry_i,
                    });
                    break;
                }
                _ => {}
            }
        }

        let found = found?;
        match found {
            Found::Tab {
                entry_index,
                was_selected,
            } => {
                let kind = self.entries.remove(entry_index);
                match kind {
                    TabOrSpacer::Item(entry) => {
                        self.ul.remove_child(&entry.tab);
                        let (pane, slot) = entry.pane.into_parts();
                        self.content.remove_child(&slot);

                        if was_selected {
                            let any_tabs_left = self.entries.iter().any(|e| e.as_item().is_some());
                            if !any_tabs_left {
                                if let Some(ds) = &self.default_slot {
                                    ds.remove_style("display");
                                }
                            } else {
                                self.select_nearest_tab(entry_index);
                            }
                        }

                        Some(RemovedEntry::Tab(
                            TabItemRemoval {
                                id: entry.tab.id,
                                index: entry_index,
                                item: entry.tab.inner,
                                was_selected,
                            },
                            pane,
                        ))
                    }
                    TabOrSpacer::Spacer(_) => unreachable!(),
                }
            }
            Found::Spacer { entry_index } => {
                let kind = self.entries.remove(entry_index);
                match kind {
                    TabOrSpacer::Spacer(spacer) => {
                        self.ul.remove_child(&spacer);
                        Some(RemovedEntry::Spacer)
                    }
                    TabOrSpacer::Item(_) => unreachable!(),
                }
            }
        }
    }

    /// Select the nearest surviving tab to `entry_index` (scanning backward
    /// first, then forward). Does nothing if no tabs remain.
    fn select_nearest_tab(&mut self, entry_index: usize) {
        for i in (0..entry_index).rev() {
            if self.entries.get(i).is_some_and(|e| e.as_item().is_some()) {
                self.select_by_index(i);
                return;
            }
        }
        for i in entry_index..self.entries.len() {
            if self.entries.get(i).is_some_and(|e| e.as_item().is_some()) {
                self.select_by_index(i);
                return;
            }
        }
    }

    /// Deselect all tabs (sets `is_active` to false on every tab).
    pub fn deselect_all(&mut self) {
        for entry in self.entries.iter_mut() {
            if let Some(item) = entry.as_item_mut() {
                item.tab.is_active.set(false);
            }
        }
    }

    /// Select the active tab using an entry index (includes spacers).
    ///
    /// Returns the [`Id`] of the selected tab, if any.
    ///
    /// Returns `None` if the given `index` was out of bounds or points to a
    /// spacer. In that case no selection state is changed.
    pub fn select_by_index(&mut self, index: usize) -> Option<Id<T>> {
        let target_is_tab = self
            .entries
            .get(index)
            .is_some_and(|e| e.as_item().is_some());
        if !target_is_tab {
            return None;
        }
        let mut id = None;
        for (i, entry) in self.entries.iter_mut().enumerate() {
            if let Some(item) = entry.as_item_mut() {
                item.tab.is_active.set(i == index);
                if i == index {
                    if let Some(ds) = &self.default_slot {
                        ds.set_style("display", "none");
                    }
                    item.pane.show();
                    id = Some(item.tab.id.clone());
                } else {
                    item.pane.hide();
                }
            }
        }
        id
    }

    /// Select the active tab using an [`Id`].
    ///
    /// Returns `Some(())` when the tab exists and was selected, otherwise `None`.
    pub fn select(&mut self, tab_id: &Id<T>) -> Option<()> {
        let mut found = false;
        for entry in self.entries.iter_mut() {
            if let Some(item) = entry.as_item_mut() {
                let is_match = &item.tab.id == tab_id;
                item.tab.is_active.set(is_match);
                if is_match {
                    if let Some(ds) = &self.default_slot {
                        ds.set_style("display", "none");
                    }
                    item.pane.show();
                    found = true;
                } else {
                    item.pane.hide();
                }
            }
        }
        if found {
            Some(())
        } else {
            None
        }
    }

    /// Return the entry index of a tab identifier.
    ///
    /// This is the position in the entries `Vec`, which includes spacers.
    pub fn index_of_tab(&self, tab_id: &Id<T>) -> Option<usize> {
        self.entries
            .iter()
            .enumerate()
            .find_map(|(index, entry)| match entry {
                TabOrSpacer::Item(e) if &e.tab.id == tab_id => Some(index),
                _ => None,
            })
    }

    /// Return the `Id<T>` of the tab at the given entry index, if it's not a spacer.
    pub fn id_of_tab(&self, index: usize) -> Option<Id<T>> {
        let entry = self.entries.get(index)?;
        let item = entry.as_item()?;
        Some(item.tab.id.clone())
    }

    /// Returns a reference to the active pane, if any.
    ///
    /// "Active" means the tab whose `is_active` is `true`.
    pub fn get_active_pane(&self) -> Option<&P> {
        self.entries.iter().find_map(|entry| match entry {
            TabOrSpacer::Item(e) if *e.tab.is_active => Some(e.pane.pane()),
            _ => None,
        })
    }

    /// Returns a mutable reference to the active pane, if any.
    ///
    /// "Active" means the tab whose `is_active` is `true`.
    pub fn get_active_pane_mut(&mut self) -> Option<&mut P> {
        self.entries.iter_mut().find_map(|entry| match entry {
            TabOrSpacer::Item(e) if *e.tab.is_active => Some(e.pane.pane_mut()),
            _ => None,
        })
    }

    /// Push a spacer onto the end of the tab bar.
    ///
    /// Returns the [`Id`] allocated for the new spacer.
    pub fn push_spacer(&mut self) -> Id<T> {
        let id = self.id_pool.get_id();
        let spacer = TabSpacer::new(id.clone());
        let kind = TabOrSpacer::Spacer(spacer);
        self.ul.append_child(&kind);
        self.entries.push(kind);
        id
    }

    /// Insert a spacer before the tab identified by `tab_id`.
    ///
    /// Returns the [`Id`] of the inserted spacer, or `None` if the tab was not
    /// found.
    pub fn insert_spacer_before(&mut self, tab_id: &Id<T>) -> Option<Id<T>> {
        let pos = self.entries.iter().enumerate().find_map(|(i, e)| {
            e.as_item()
                .and_then(|item| (&item.tab.id == tab_id).then_some(i))
        });
        let pos = pos?;
        let id = self.id_pool.get_id();
        let spacer = TabSpacer::new(id.clone());
        let kind = TabOrSpacer::Spacer(spacer);
        self.ul
            .insert_child_before(&kind, Some(self.entries[pos].element()));
        self.entries.insert(pos, kind);
        Some(id)
    }

    /// Insert a spacer after the tab identified by `tab_id`.
    ///
    /// Returns the [`Id`] of the inserted spacer, or `None` if the tab was not
    /// found.
    pub fn insert_spacer_after(&mut self, tab_id: &Id<T>) -> Option<Id<T>> {
        let pos = self.entries.iter().enumerate().find_map(|(i, e)| {
            e.as_item()
                .and_then(|item| (&item.tab.id == tab_id).then_some(i))
        });
        let pos = pos?;
        let id = self.id_pool.get_id();
        let spacer = TabSpacer::new(id.clone());
        let kind = TabOrSpacer::Spacer(spacer);
        let insert_pos = pos + 1;
        if let Some(next_entry) = self.entries.get(insert_pos) {
            self.ul
                .insert_child_before(&kind, Some(next_entry.element()));
            self.entries.insert(insert_pos, kind);
        } else {
            self.ul.append_child(&kind);
            self.entries.push(kind);
        }
        Some(id)
    }

    /// Remove all spacers from the tab bar.
    pub fn remove_all_spacers(&mut self) {
        self.entries.retain(|entry| {
            if entry.is_spacer() {
                self.ul.remove_child(entry);
                false
            } else {
                true
            }
        });
    }

    /// Set the alignment of tabs within the panel.
    ///
    /// This inserts or removes spacers to achieve the desired alignment.
    pub fn set_alignment(&mut self, alignment: TabAlignment) {
        self.remove_all_spacers();
        let first_id = self
            .entries
            .iter()
            .filter_map(|e| e.as_item())
            .next()
            .map(|item| item.tab.id.clone());
        match alignment {
            TabAlignment::Start => {
                self.push_spacer();
            }
            TabAlignment::Center => {
                if let Some(id) = &first_id {
                    self.insert_spacer_before(id);
                }
                self.push_spacer();
            }
            TabAlignment::End => {
                if let Some(id) = &first_id {
                    self.insert_spacer_before(id);
                }
            }
        }
    }

    /// Set whether new tabs should show a close button by default.
    ///
    /// Also updates all existing tabs to match.
    pub fn set_default_closable(&mut self, closable: bool) {
        self.default_closable = closable;
        for entry in self.entries.iter_mut() {
            if let Some(item) = entry.as_item_mut() {
                item.tab.set_closable(closable);
            }
        }
    }

    /// Set whether a specific tab should show a close button.
    ///
    /// Does nothing if the tab is not found.
    pub fn set_tab_closable(&mut self, id: &Id<T>, closable: bool) {
        for entry in self.entries.iter_mut() {
            if let Some(item) = entry.as_item_mut() {
                if &item.tab.id == id {
                    item.tab.set_closable(closable);
                    return;
                }
            }
        }
    }

    fn item_events(&self) -> impl Future<Output = ItemEvent<V, T>> + '_ {
        let mut race = std::future::pending().boxed_local();
        for (index, entry) in self.entries.iter().enumerate() {
            if let TabOrSpacer::Item(item) = entry {
                let click_id = item.tab.id.clone();
                let close_id = item.tab.id.clone();
                let on_click = &item.tab.on_click;
                let close_click = &item.tab.close_click;
                let click = async move {
                    let event = on_click.next().await;
                    ItemEvent::Click {
                        id: click_id,
                        index,
                        event,
                    }
                };
                let close = async move {
                    close_click.next().await;
                    ItemEvent::Close { id: close_id }
                };
                race = race.or(close).or(click).boxed_local();
            }
        }
        race
    }
}

/// Internal intermediate event type. `item_events` can't move `T` or `P`
/// out of entries, so it produces this lightweight signal. `StepMut` then
/// does the actual removal and constructs the public [`TabListEvent`].
enum ItemEvent<V: View, T> {
    Click {
        id: Id<T>,
        index: usize,
        event: V::Event,
    },
    Close {
        id: Id<T>,
    },
}

/// [`StepMut`] for [`TabPanel`]: races all tab click and close-click events.
///
/// On a tab click, auto-selects the clicked tab. On a close click, auto-removes
/// the tab (reselecting the nearest neighbor) and returns the inner content `T`
/// and pane `P` for cleanup. Use [`StepWithMut`] when pane content has its own
/// `step` to drive.
impl<V: View, T: ViewChild<V> + 'static, P: ViewChild<V> + 'static> StepMut for TabPanel<V, T, P> {
    type Output = TabListEvent<V, T, P>;
    async fn step_mut(&mut self) -> TabListEvent<V, T, P> {
        let ev = self.item_events().await;
        match ev {
            ItemEvent::Click { id, index, event } => {
                self.select(&id);
                TabListEvent::ItemClicked { id, index, event }
            }
            ItemEvent::Close { id } => {
                let (removal, pane) = match self.remove_by_id(&id) {
                    Some(RemovedEntry::Tab(removal, pane)) => (removal, pane),
                    _ => unreachable!("close click on non-existent tab"),
                };
                TabListEvent::CloseClicked {
                    id: removal.id,
                    index: removal.index,
                    item: removal.item,
                    pane,
                }
            }
        }
    }
}

/// [`StepWithMut`] for [`TabPanel`]: drives per-pane event loops.
///
/// The closure `f` is called once per [`TabPanelEntry`] (spacers are skipped
/// and produce perpetually-pending futures that never win the race). All
/// returned futures are raced together via [`mogwai::future::race_all`], and
/// the first to resolve wins.
///
/// The closure receives `&mut `[`TabPanelEntry`] with [`TabPanelEntry::tab`]
/// and [`TabPanelEntry::pane`] accessors. It is the closure's responsibility
/// to race the tab's click event against the pane's own `step` if both need
/// to be driven. The impl does **not** wrap the result in
/// [`TabPanelEvent::Panes`]; the return type is `Ev` directly, so the
/// caller's `Ev` is typically `TabPanelEvent<V, T, PaneEvent>` to
/// distinguish tab clicks from pane events.
impl<V: View, T: ViewChild<V> + 'static, P: ViewChild<V>> StepWithMut<TabPanelEntry<V, T, P>>
    for TabPanel<V, T, P>
{
    type Output<Ev: 'static> = Ev;
    async fn step_with_mut<Ev>(
        &mut self,
        f: impl for<'a> FnMut(&'a mut TabPanelEntry<V, T, P>) -> Pin<Box<dyn Future<Output = Ev> + 'a>>,
    ) -> Ev
    where
        Ev: 'static,
    {
        let mut f = f;
        // Race all entry closures together. Spacers produce pending futures
        // that never resolve.
        let entry_futs: Vec<Pin<Box<dyn Future<Output = Ev> + '_>>> = self
            .entries
            .iter_mut()
            .map(|entry| match entry {
                TabOrSpacer::Item(item_entry) => f(item_entry),
                TabOrSpacer::Spacer(_) => std::future::pending().boxed_local(),
            })
            .collect();

        mogwai::future::race_all(entry_futs).await
    }
}

/// Discriminates tab-click events from pane events.
///
/// When using [`StepWithMut`] on [`TabPanel`], the closure typically races a
/// tab click against the pane's own `step`. The closure returns
/// `TabPanelEvent<V, T, P, PaneEvent>` so the caller can match:
///
/// ```ignore
/// match panel.step_with_mut(|entry| { ... }).await {
///     TabPanelEvent::Tabs(TabListEvent::ItemClicked { id, .. }) => {
///         panel.select(&id);
///     }
///     TabPanelEvent::Panes(pane_event) => { /* handle pane event */ }
/// }
/// ```
pub enum TabPanelEvent<V: View, T, P, Ev> {
    /// A tab event (click or close).
    Tabs(TabListEvent<V, T, P>),
    /// A pane produced an event.
    Panes(Ev),
}

/// Component gallery sandbox for [`TabPanel`].
///
/// Demonstrates three tabs with live timer widgets that keep running even when
/// their pane is hidden.
#[cfg(feature = "library")]
pub mod library {

    use crate::components::widget::Widget;

    use super::*;

    /// Gallery item showcasing a [`TabPanel`] with three animated panes.
    #[derive(ViewChild)]
    pub struct TabListLibraryItem<V: View> {
        #[child]
        pub div: V::Element,
        panel: TabPanel<V, V::Element, Widget<V, ()>>,
    }

    impl<V: View> Default for TabListLibraryItem<V> {
        fn default() -> Self {
            rsx! {
                let html = div() {}
            }
            let default_widget = Widget::new(html, futures_lite::stream::pending());
            let mut panel = TabPanel::new(default_widget);

            // Pane 0
            rsx! {
                let wrapper = div(class = "container") {
                    div(class = "row") {
                        h1() { "Pane 0" }
                        p() { "Contains nothing of importance." }
                        p() { let count_text = "0 seconds" }
                        p() { let loop_text = "0 loops" }
                    }
                }
            }
            let w0 = Widget::new(
                wrapper,
                futures_lite::stream::unfold(
                    (count_text, loop_text, 0.0f32, 0u32),
                    |(count_text, loop_text, mut count, mut loops)| async move {
                        let elapsed = mogwai::time::wait_millis(1000).await as f32;
                        count += elapsed as f32 / 1000.0;
                        loops += 1;
                        count_text.set_text(format!("{count} seconds, {loops} loops"));
                        loop_text.set_text(format!("{loops} loops have run"));
                        Some(((), (count_text, loop_text, count, loops)))
                    },
                ),
            );
            rsx! {
                let tab0 = span() { "Tab Zero" }
            }
            panel.push(tab0, w0);

            // Pane 1
            rsx! {
                let html = div(class = "container") {
                    div(class = "row") {
                        h1() { "Pane One" }
                        p() {
                            "Also contains nothing of importance."
                            br{}
                            let count_text = "waiting..."
                        }
                    }
                }
            }
            let w1 = Widget::new(
                html,
                futures_lite::stream::unfold(
                    (count_text, 0f32, 0u32),
                    |(count_text, mut count, mut loops)| async move {
                        let elapsed = mogwai::time::wait_millis(1000).await as f32;
                        count += elapsed as f32 / 1000.0;
                        loops += 1;
                        count_text.set_text(format!("{count} seconds, {loops} loops"));
                        Some(((), (count_text, count, loops)))
                    },
                ),
            );
            rsx! {
                let tab1 = span() { "Tab 1" }
            }
            panel.push(tab1, w1);

            // Pane 2
            rsx! {
                let html = div(class = "container") {
                    div(class = "row") {
                        h1() { "Last Pane" }
                        p() { "Super important stuff here, y'all." }
                    }
                }
            }
            let w2 = Widget::new(html, futures_lite::stream::pending());
            rsx! {
                let tab2 = span() { "Tabbity Too" }
            }
            panel.push(tab2, w2);

            rsx! {
                let div = div() {
                    {&panel}
                }
            }

            Self { div, panel }
        }
    }

    impl<V: View> TabListLibraryItem<V> {
        /// Select the tab at the given index.
        pub fn select(&mut self, index: usize) {
            log::info!("selecting pane {index}");
            if let Some(id) = self.panel.id_of_tab(index) {
                self.panel.select(&id);
            }
        }
    }

    impl<V: View> StepMut for TabListLibraryItem<V> {
        type Output = ();
        async fn step_mut(&mut self) {
            // The closure receives &mut TabPanelEntry and races the tab's
            // click event against the pane's step. On a tab click, we select
            // that tab.
            let ev = self
                .panel
                .step_with_mut(|entry| {
                    let (tab, pane) = entry.split_tab_pane();
                    let id = tab.id().clone();
                    let on_click = tab.on_click();
                    async {
                        let tab_fut = async {
                            let event = on_click.next().await;
                            TabPanelEvent::<V, V::Element, Widget<V, ()>, ()>::Tabs(
                                TabListEvent::ItemClicked {
                                    id,
                                    index: 0,
                                    event,
                                },
                            )
                        };
                        let pane_fut = async {
                            pane.step_mut().await;
                            TabPanelEvent::<V, V::Element, Widget<V, ()>, ()>::Panes(())
                        };
                        tab_fut.or(pane_fut).await
                    }
                    .boxed_local()
                })
                .await;
            match ev {
                TabPanelEvent::Tabs(TabListEvent::ItemClicked {
                    id,
                    index: _,
                    event: _,
                }) => {
                    self.panel.select(&id);
                }
                TabPanelEvent::Tabs(TabListEvent::CloseClicked { .. }) => {
                    // Close handled by StepMut internally; nothing to do.
                }
                TabPanelEvent::Panes(()) => {}
            }
        }
    }
}
