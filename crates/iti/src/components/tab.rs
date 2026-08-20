//! Page tabs (Bootstrap nav-tabs).
//!
//! This module provides [`TabPanel`], a component that pairs a horizontal tab
//! bar with a content area. Each tab `T` and its associated pane `P` are stored
//! together in a single [`Vec`] of [`TabPanelEntryKind`] entries, so that
//! [`StepWithMut`] can hand a closure `&mut `[`TabPanelEntry`] with direct
//! `.tab` and `.pane` field access. This lets callers race tab-click events
//! against pane-specific event loops in a single future-race, which is
//! essential for closable detail-panel tabs and other interactive pane content.
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
//!         let on_click = entry.tab.on_click();
//!         let pane = &mut entry.pane;
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
    inner: T,
    is_active: Proxy<bool>,
    id: Id<T>,
}

impl<V: View, T: ViewChild<V>> TabListItem<V, T> {
    /// Create a new tab item with the given [`Id`] and inner content.
    ///
    /// This is called by [`TabPanel::push`] / [`TabPanel::insert`]; you
    /// usually don't construct `TabListItem` values yourself.
    pub fn new(id: Id<T>, inner: T) -> Self {
        let mut is_active = Proxy::new(false);
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
                }
            }
        }

        Self {
            li,
            a,
            on_click,
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
}

/// Event emitted when a tab is clicked in a [`TabPanel`].
///
/// Returned by [`TabPanel::step_mut`] (via [`StepMut`]) and surfaced inside
/// the [`TabPanelEvent::Tabs`] variant when using [`StepWithMut`].
pub enum TabListEvent<V: View, T> {
    /// A tab was clicked.
    ItemClicked {
        /// The [`Id`] of the clicked tab.
        id: Id<T>,
        /// The tab's index among tab items (spacers are not counted).
        index: usize,
        /// The underlying DOM event.
        event: V::Event,
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

/// A flexible spacer element within a [`TabPanel`].
///
/// Spacers are `flex-grow: 1` elements that absorb available space in the tab
/// bar. Insert them before, after, or between tabs to control alignment.
#[derive(ViewChild)]
pub struct TabSpacer<V: View> {
    #[child]
    li: V::Element,
}

impl<V: View> TabSpacer<V> {
    fn new() -> Self {
        rsx! {
            let li = li(class = "nav-tab-spacer") {}
        }
        Self { li }
    }
}

/// A tab + pane pair stored in a [`TabPanel`].
///
/// This is the type the [`StepWithMut`] closure receives a mutable reference
/// to, allowing callers to step both the tab `T` and the pane `P` together in
/// a single race. Destructure `&mut TabPanelEntry` to get simultaneous mutable
/// access to the separate fields:
///
/// ```ignore
/// panel.step_with_mut(|entry| {
///     let on_click = entry.tab.on_click();
///     let pane = &mut entry.pane;
///     async { /* race on_click.next() vs pane.step_mut() */ }.boxed_local()
/// })
/// ```
///
/// The `slot` field is the wrapper `<div>` that holds the pane in the DOM.
/// [`TabPanel::select`] toggles `display: none` on it.
pub struct TabPanelEntry<V: View, T, P> {
    /// The tab list item (the clickable tab header).
    pub tab: TabListItem<V, T>,
    /// The pane content associated with this tab.
    pub pane: P,
    /// Wrapper `div` slot element that holds the pane in the DOM.
    /// Toggled `display: none` to show/hide the pane.
    slot: V::Element,
}

/// An entry in a [`TabPanel`] — either a tab+pane pair or a spacer.
pub enum TabPanelEntryKind<V: View, T, P> {
    /// A tab and its associated pane.
    Item(TabPanelEntry<V, T, P>),
    /// A flexible spacer element (no tab, no pane).
    Spacer(TabSpacer<V>),
}

impl<V: View, T, P> TabPanelEntryKind<V, T, P> {
    /// Get the underlying tab `li` element (for Item) or spacer `li` (for
    /// Spacer), for DOM operations.
    fn element(&self) -> &V::Element {
        match self {
            TabPanelEntryKind::Item(entry) => &entry.tab.li,
            TabPanelEntryKind::Spacer(spacer) => &spacer.li,
        }
    }

    /// Returns `true` if this entry is a spacer.
    fn is_spacer(&self) -> bool {
        matches!(self, TabPanelEntryKind::Spacer(_))
    }

    /// Try to get the entry as a tab+pane pair reference.
    fn as_item(&self) -> Option<&TabPanelEntry<V, T, P>> {
        match self {
            TabPanelEntryKind::Item(entry) => Some(entry),
            TabPanelEntryKind::Spacer(_) => None,
        }
    }

    /// Try to get the entry as a mutable tab+pane pair reference.
    fn as_item_mut(&mut self) -> Option<&mut TabPanelEntry<V, T, P>> {
        match self {
            TabPanelEntryKind::Item(entry) => Some(entry),
            TabPanelEntryKind::Spacer(_) => None,
        }
    }
}

impl<V: View, T: ViewChild<V>, P: ViewChild<V>> ViewChild<V> for TabPanelEntryKind<V, T, P> {
    fn as_append_arg(&self) -> AppendArg<V, impl Iterator<Item = std::borrow::Cow<'_, V::Node>>> {
        match self {
            TabPanelEntryKind::Item(entry) => entry.tab.as_boxed_append_arg(),
            TabPanelEntryKind::Spacer(spacer) => spacer.as_boxed_append_arg(),
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
/// [`TabPanelEntryKind<V, T, P>`], eliminating the cross-collection join that
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
    entries: Vec<TabPanelEntryKind<V, T, P>>,
    id_pool: IdPool<T>,
    default_slot: Option<V::Element>,
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
        }
    }

    /// Return the number of tabs (spacers are not counted).
    pub fn len(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| e.as_item().is_some())
            .count()
    }

    /// Returns `true` if there are no tabs (spacers are not counted).
    pub fn is_empty(&self) -> bool {
        !self.entries.iter().any(|e| e.as_item().is_some())
    }

    /// Return a reference to the [`TabListItem`] at the given tab index.
    ///
    /// The index counts only tab items, not spacers.
    pub fn get(&self, index: usize) -> Option<&TabListItem<V, T>> {
        self.entries
            .iter()
            .filter_map(|e| e.as_item())
            .nth(index)
            .map(|e| &e.tab)
    }

    /// Iterator over all entries, including spacers.
    ///
    /// Entries are yielded in display order. Match on
    /// [`TabPanelEntryKind::Item`] to access the tab + pane pair, or
    /// [`TabPanelEntryKind::Spacer`] for spacers.
    pub fn iter(&self) -> impl Iterator<Item = &TabPanelEntryKind<V, T, P>> {
        self.entries.iter()
    }

    /// Mutable iterator over all entries, including spacers.
    ///
    /// See [`iter`](Self::iter) for ordering and matching details.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut TabPanelEntryKind<V, T, P>> {
        self.entries.iter_mut()
    }

    /// Push a new tab onto the end of the stack.
    ///
    /// Returns the [`Id`] allocated for the new tab.
    pub fn push(&mut self, tab: T, pane: P) -> Id<T> {
        let id = self.id_pool.get_id();
        let tab_item = TabListItem::new(id.clone(), tab);

        let slot = V::Element::new("div");
        slot.set_style("display", "none");
        slot.set_style("flex", "1");
        slot.set_style("min-height", "0");
        slot.append_child(&pane);
        self.content.append_child(&slot);

        let entry = TabPanelEntry {
            tab: tab_item,
            pane,
            slot,
        };
        let kind = TabPanelEntryKind::Item(entry);
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
            TabPanelEntryKind::Item(entry) => {
                self.ul.remove_child(&entry.tab);
                self.content.remove_child(&entry.slot);
                Some((entry.tab.id, entry.tab.inner, entry.pane))
            }
            TabPanelEntryKind::Spacer(_) => unreachable!(),
        }
    }

    /// Insert a new tab at the given tab index and return a unique identifier.
    ///
    /// The index counts only tab items, not spacers. If the index is out of
    /// bounds the tab is appended to the end.
    pub fn insert(&mut self, index: usize, tab: T, pane: P) -> Id<T> {
        let id = self.id_pool.get_id();
        let tab_item = TabListItem::new(id.clone(), tab);

        let slot = V::Element::new("div");
        slot.set_style("display", "none");
        slot.set_style("flex", "1");
        slot.set_style("min-height", "0");
        slot.append_child(&pane);
        self.content.append_child(&slot);

        let entry = TabPanelEntry {
            tab: tab_item,
            pane,
            slot,
        };
        let kind = TabPanelEntryKind::Item(entry);

        let entry_pos = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.as_item().is_some())
            .nth(index)
            .map(|(i, _)| i);

        if let Some(pos) = entry_pos {
            self.ul
                .insert_child_before(&kind, Some(self.entries[pos].element()));
            self.entries.insert(pos, kind);
        } else {
            self.ul.append_child(&kind);
            self.entries.push(kind);
        }
        id
    }

    /// Remove a tab by its [`Id`].
    ///
    /// Returns the [`TabItemRemoval`] for the tab and the associated pane `P`,
    /// or `None` if the tab was not found.
    pub fn remove_by_id(&mut self, id: &Id<T>) -> Option<(TabItemRemoval<T>, P)> {
        let mut found = None;
        let mut tab_index = 0;
        for (entry_i, entry) in self.entries.iter().enumerate() {
            if let Some(item) = entry.as_item() {
                if &item.tab.id == id {
                    found = Some((entry_i, tab_index, *item.tab.is_active));
                    break;
                }
                tab_index += 1;
            }
        }
        let (entry_index, tab_index, was_selected) = found?;
        let kind = self.entries.remove(entry_index);
        match kind {
            TabPanelEntryKind::Item(entry) => {
                self.ul.remove_child(&entry.tab);
                self.content.remove_child(&entry.slot);
                Some((
                    TabItemRemoval {
                        id: entry.tab.id,
                        index: tab_index,
                        item: entry.tab.inner,
                        was_selected,
                    },
                    entry.pane,
                ))
            }
            TabPanelEntryKind::Spacer(_) => unreachable!(),
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

    /// Select the active tab using a tab index (spacers are not counted).
    ///
    /// Returns the [`Id`] of the selected tab, if any.
    ///
    /// Returns `None` if the given `index` was out of bounds.
    pub fn select_by_index(&mut self, index: usize) -> Option<Id<T>> {
        let mut id = None;
        let mut tab_i = 0;
        for entry in self.entries.iter_mut() {
            if let Some(item) = entry.as_item_mut() {
                item.tab.is_active.set(tab_i == index);
                if tab_i == index {
                    if let Some(ds) = &self.default_slot {
                        ds.set_style("display", "none");
                    }
                    item.slot.remove_style("display");
                    id = Some(item.tab.id.clone());
                } else {
                    item.slot.set_style("display", "none");
                }
                tab_i += 1;
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
                    item.slot.remove_style("display");
                    found = true;
                } else {
                    item.slot.set_style("display", "none");
                }
            }
        }
        if found {
            Some(())
        } else {
            None
        }
    }

    /// Return the entry-vec index of a tab identifier.
    ///
    /// This is the raw position in the entries `Vec`, which includes spacers.
    /// If you need the tab-item-only index (spacers not counted), filter
    /// through [`iter`](Self::iter) and enumerate yourself.
    pub fn index_of_tab(&self, tab_id: &Id<T>) -> Option<usize> {
        self.entries
            .iter()
            .enumerate()
            .find_map(|(index, entry)| match entry {
                TabPanelEntryKind::Item(e) if &e.tab.id == tab_id => Some(index),
                _ => None,
            })
    }

    /// Return the `Id<T>` of the tab at the given index, if it's not a spacer.
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
            TabPanelEntryKind::Item(e) if *e.tab.is_active => Some(&e.pane),
            _ => None,
        })
    }

    /// Returns a mutable reference to the active pane, if any.
    ///
    /// "Active" means the tab whose `is_active` is `true`.
    pub fn get_active_pane_mut(&mut self) -> Option<&mut P> {
        self.entries.iter_mut().find_map(|entry| match entry {
            TabPanelEntryKind::Item(e) if *e.tab.is_active => Some(&mut e.pane),
            _ => None,
        })
    }

    /// Push a spacer onto the end of the tab bar.
    pub fn push_spacer(&mut self) {
        let spacer = TabSpacer::new();
        let kind = TabPanelEntryKind::Spacer(spacer);
        self.ul.append_child(&kind);
        self.entries.push(kind);
    }

    /// Insert a spacer before the tab identified by `tab_id`.
    ///
    /// Does nothing if the tab is not found.
    pub fn insert_spacer_before(&mut self, tab_id: &Id<T>) {
        let pos = self.entries.iter().enumerate().find_map(|(i, e)| {
            e.as_item()
                .and_then(|item| (&item.tab.id == tab_id).then_some(i))
        });
        if let Some(pos) = pos {
            let spacer = TabSpacer::new();
            let kind = TabPanelEntryKind::Spacer(spacer);
            self.ul
                .insert_child_before(&kind, Some(self.entries[pos].element()));
            self.entries.insert(pos, kind);
        }
    }

    /// Insert a spacer after the tab identified by `tab_id`.
    ///
    /// Does nothing if the tab is not found.
    pub fn insert_spacer_after(&mut self, tab_id: &Id<T>) {
        let pos = self.entries.iter().enumerate().find_map(|(i, e)| {
            e.as_item()
                .and_then(|item| (&item.tab.id == tab_id).then_some(i))
        });
        if let Some(pos) = pos {
            let spacer = TabSpacer::new();
            let kind = TabPanelEntryKind::Spacer(spacer);
            let insert_pos = pos + 1;
            if let Some(next_entry) = self.entries.get(insert_pos) {
                self.ul
                    .insert_child_before(&kind, Some(next_entry.element()));
                self.entries.insert(insert_pos, kind);
            } else {
                self.ul.append_child(&kind);
                self.entries.push(kind);
            }
        }
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

    fn item_events(&self) -> impl Future<Output = TabListEvent<V, T>> + '_ {
        let mut race = std::future::pending().boxed_local();
        let mut tab_i = 0;
        for entry in self.entries.iter() {
            if let TabPanelEntryKind::Item(item) = entry {
                let index = tab_i;
                let id = item.tab.id.clone();
                let on_click = &item.tab.on_click;
                let click = async move {
                    let event = on_click.next().await;
                    TabListEvent::ItemClicked { id, index, event }
                };
                race = race.or(click).boxed_local();
                tab_i += 1;
            }
        }
        race
    }
}

/// [`StepMut`] for [`TabPanel`]: races all tab click events.
///
/// On a click, auto-selects the clicked tab (updating `is_active` and the
/// visible pane) and returns the [`TabListEvent`]. Use this when you only need
/// tab selection and don't need to drive per-pane event loops. When pane
/// content has its own `step`, use [`StepWithMut`] instead.
impl<V: View, T: ViewChild<V> + 'static, P: ViewChild<V>> StepMut for TabPanel<V, T, P> {
    type Output = TabListEvent<V, T>;
    async fn step_mut(&mut self) -> TabListEvent<V, T> {
        let ev = self.item_events().await;
        match &ev {
            TabListEvent::ItemClicked {
                id,
                index: _,
                event: _,
            } => {
                self.select(id);
            }
        }
        ev
    }
}

/// [`StepWithMut`] for [`TabPanel`]: drives per-pane event loops.
///
/// The closure `f` is called once per [`TabPanelEntry`] (spacers are skipped
/// and produce perpetually-pending futures that never win the race). All
/// returned futures are raced together via [`mogwai::future::race_all`], and
/// the first to resolve wins.
///
/// The closure receives `&mut `[`TabPanelEntry`] with direct `.tab` and
/// `.pane` fields. It is the closure's responsibility to race the tab's click
/// event against the pane's own `step` if both need to be driven. The impl
/// does **not** wrap the result in [`TabPanelEvent::Panes`]; the return type
/// is `Ev` directly, so the caller's `Ev` is typically `TabPanelEvent<V, T,
/// PaneEvent>` to distinguish tab clicks from pane events.
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
                TabPanelEntryKind::Item(item_entry) => f(item_entry),
                TabPanelEntryKind::Spacer(_) => std::future::pending().boxed_local(),
            })
            .collect();

        mogwai::future::race_all(entry_futs).await
    }
}

/// Discriminates tab-click events from pane events.
///
/// When using [`StepWithMut`] on [`TabPanel`], the closure typically races a
/// tab click against the pane's own `step`. The closure returns
/// `TabPanelEvent<V, T, PaneEvent>` so the caller can match:
///
/// ```ignore
/// match panel.step_with_mut(|entry| { ... }).await {
///     TabPanelEvent::Tabs(TabListEvent::ItemClicked { id, .. }) => {
///         panel.select(&id);
///     }
///     TabPanelEvent::Panes(pane_event) => { /* handle pane event */ }
/// }
/// ```
pub enum TabPanelEvent<V: View, T, Ev> {
    /// A tab was clicked.
    Tabs(TabListEvent<V, T>),
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
                    let id = entry.tab.id().clone();
                    let on_click = &entry.tab.on_click;
                    let pane = &mut entry.pane;
                    async {
                        let tab_fut = async {
                            let event = on_click.next().await;
                            TabPanelEvent::Tabs::<V, V::Element, ()>(TabListEvent::ItemClicked {
                                id,
                                index: 0,
                                event,
                            })
                        };
                        let pane_fut = async {
                            pane.step_mut().await;
                            TabPanelEvent::Panes(())
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
                TabPanelEvent::Panes(()) => {}
            }
        }
    }
}
