//! Page tabs (Bootstrap nav-tabs).
//!
//! This module provides [`TabPanel`], a component that pairs a horizontal tab
//! bar with a content area. Each tab `T` and its associated pane `P` are stored
//! together in a single [`Vec`] of [`TabOrSpacer`] entries, so that
//! [`StepWithMut`] can hand a closure `&mut `[`TabOrSpacer`] with direct
//! access to the [`TabListItem`] and [`TabbedPane`]. This lets callers
//! race tab-click events against pane-specific event loops in a single
//! future-race, which is essential for closable detail-panel tabs and other
//! interactive pane content.
//!
//! # Type containment
//!
//! ```text
//! TabPanel<V, P, T, S>
//! └── Vec<TabOrSpacer<V, P, T, S>>
//!     ├── Item ── TabPanelEntry<V, P, T>
//!     │            ├── TabListItem<V, T, S>  (the clickable tab header)
//!     │            │             └── T      (tab label content)
//!     │            └── TabbedPane<V, P>      (pane wrapper + slot div)
//!     │                          └── P      (pane content)
//!     └── Spacer ── TabSpacer<V, T, S>      (flex spacer, optional inner S)
//! ```
//!
//! Spacers ([`TabSpacer`]) can be interleaved with tabs to control alignment
//! via [`TabPanel::set_alignment`] / [`TabAlignment`].
//!
//! # Typical usage
//!
//! ```ignore
//! let mut panel = TabPanel::<V, MyPane>::new(default_pane);
//! let id = panel.push(tab_label, my_pane);
//! panel.select(&id);
//! // In the event loop:
//! let ev = panel
//!     .step_with_mut(|entry| {
//!         match entry {
//!             TabOrSpacer::Item(item) => {
//!                 item.step_with_mut(|pane| pane.step_mut().boxed_local())
//!                     .map(TabPanelEvent::Item)
//!                     .boxed_local()
//!             }
//!             TabOrSpacer::Spacer(spacer) => {
//!                 spacer.step_with(|s| /* user domain */)
//!                     .map(TabPanelEvent::User)
//!                     .boxed_local()
//!             }
//!         }
//!     })
//!     .await;
//! ```
//!
//! When all you need is tab selection (no pane stepping), use [`StepMut`]
//! instead, which auto-selects the clicked tab and returns a [`TabPanelEvent`].
use std::{future::Future, pin::Pin};

use futures_lite::FutureExt;
use mogwai::{prelude::*, step::StepWithMut};

use crate::id::{Id, IdPool};

mod entry;
mod item;

pub use entry::{TabOrSpacer, TabPanelEntry, TabPanelEntryEvent, TabSpacer, TabbedPane};
pub use item::{EmptySpacer, TabListItem, TabListItemEvent, TabListItemEventData};

/// Event emitted by a [`TabPanel`].
///
/// - [`ItemClicked`](Self::ItemClicked): a tab was clicked. `StepMut`
///   auto-selects the tab; `Step` does not.
/// - [`ItemCloseClicked`](Self::ItemCloseClicked): the close button was
///   clicked. Raw report, no removal. Only `Step` produces this.
/// - [`ItemClosed`](Self::ItemClosed): the tab was removed. `StepMut`
///   auto-removes and moves out `T`/`P`. Only `StepMut` produces this.
/// - [`User`](Self::User): a user-domain event from the closure passed to
///   [`StepWith`] / [`StepWithMut`]. Unreachable when using [`Step`] /
///   [`StepMut`].
pub enum TabPanelEvent<V: View, P, T = <V as View>::Element, S = EmptySpacer, Ev = ()> {
    /// A tab was clicked. `StepMut` auto-selects; `Step` does not.
    ItemClicked {
        id: Id<(T, S)>,
        index: usize,
        event: V::Event,
    },
    /// The close button was clicked. Raw report, no removal.
    /// Only `Step` produces this.
    ItemCloseClicked {
        id: Id<(T, S)>,
        index: usize,
        event: V::Event,
    },
    /// The tab was removed. `StepMut` auto-removes and moves out `T`/`P`.
    /// Only `StepMut` produces this.
    ItemClosed {
        id: Id<(T, S)>,
        index: usize,
        item: T,
        pane: P,
    },
    /// A user-domain event from the closure passed to `StepWith` /
    /// `StepWithMut`. Unreachable when using `Step` / `StepMut`.
    User(Ev),
}

/// Result of removing an item from a [`TabPanel`].
pub struct TabItemRemoval<T, S> {
    /// [`Id`] of the item removed.
    pub id: Id<(T, S)>,
    /// The index of the item before it was removed.
    pub index: usize,
    /// The item that was removed.
    pub item: T,
    /// Whether or not the tab was active when it was removed.
    pub was_selected: bool,
}

/// Result of removing an entry (tab or spacer) from a [`TabPanel`].
pub enum RemovedEntry<T, P, S> {
    /// A tab was removed, returning the tab removal info and its pane.
    Tab(TabItemRemoval<T, S>, P),
    /// A spacer was removed.
    Spacer,
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
/// [`TabOrSpacer<V, P, T, S>`], eliminating the cross-collection join that
/// separate `TabList` + `Panes` + `tabs_to_panes` collections would require.
/// This allows [`StepWithMut`] to pass `&mut `[`TabOrSpacer<V, P, T, S>`] to a
/// closure, giving simultaneous mutable access to both the tab `T` and the
/// pane `P`.
///
/// # Type parameters
///
/// - `V`: the mogwai view abstraction.
/// - `P`: the pane content type. Defaults to `V::Element`.
/// - `T`: the tab's inner content type. Defaults to `V::Element`.
/// - `S`: the spacer's inner content type. Defaults to `()`.
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
/// - [`Step`]: Races all tab click/close events (via [`TabListItem::step`]).
///   Reports events without side effects. The `User` variant is unreachable.
/// - [`StepMut`]: Same as [`Step`], but auto-selects on click and auto-removes
///   on close (moving out `T`/`P`).
/// - [`StepWithMut<TabOrSpacer<V, P, T, S>>`]: Calls the closure once per
///   entry (including spacers), racing all returned futures. The closure
///   receives `&mut TabOrSpacer` and delegates to each child's own step impls.
///   The return type is `Ev` directly (wrapped in [`TabPanelEvent::User`]).
///
/// [`StepWithMut<TabOrSpacer<V, P, T, S>>`]: StepWithMut
#[derive(ViewChild, ViewProperties)]
pub struct TabPanel<V: View, P = <V as View>::Element, T = <V as View>::Element, S = EmptySpacer> {
    #[child]
    #[properties]
    window: V::Element,
    ul: V::Element,
    content: V::Element,
    entries: Vec<TabOrSpacer<V, P, T, S>>,
    id_pool: IdPool<(T, S)>,
    default_slot: Option<V::Element>,
    default_closable: bool,
}

impl<V: View, P: ViewChild<V>, T: ViewChild<V> + 'static, S: ViewChild<V> + 'static>
    TabPanel<V, P, T, S>
{
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
    pub fn get(&self, index: usize) -> Option<&TabListItem<V, T, S>> {
        self.entries
            .get(index)
            .and_then(|e| e.as_item())
            .map(|e| e.tab())
    }

    /// Iterator over all entries, including spacers.
    ///
    /// Entries are yielded in display order. Match on
    /// [`TabOrSpacer::Item`] to access the tab + pane pair, or
    /// [`TabOrSpacer::Spacer`] for spacers.
    pub fn iter(&self) -> impl Iterator<Item = &TabOrSpacer<V, P, T, S>> {
        self.entries.iter()
    }

    /// Mutable iterator over all entries, including spacers.
    ///
    /// See [`iter`](Self::iter) for ordering and matching details.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut TabOrSpacer<V, P, T, S>> {
        self.entries.iter_mut()
    }

    /// Push a new tab onto the end of the stack.
    ///
    /// Returns the [`Id`] allocated for the new tab.
    pub fn push(&mut self, tab: T, pane: P) -> Id<(T, S)> {
        let id = self.id_pool.get_id();
        let index = self.entries.len();
        let mut tab_item = TabListItem::new(id.clone(), index, tab);
        tab_item.set_closable(self.default_closable);

        let pane = TabbedPane::new(pane);
        self.content.append_child(pane.slot());

        let entry = TabPanelEntry::new(tab_item, pane);
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
    #[allow(clippy::type_complexity)]
    pub fn pop(&mut self) -> Option<(Id<(T, S)>, T, P)> {
        let pos = self
            .entries
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, e)| e.as_item().map(|_| i))?;
        let kind = self.entries.remove(pos);
        match kind {
            TabOrSpacer::Item(entry) => {
                let (tab, pane) = entry.into_parts();
                self.ul.remove_child(&tab);
                let (pane, slot) = pane.into_parts();
                self.content.remove_child(&slot);
                let (id, inner) = tab.into_parts();
                self.reindex_entries();
                Some((id, inner, pane))
            }
            TabOrSpacer::Spacer(_) => unreachable!(),
        }
    }

    /// Insert a new tab at the given entry index and return a unique identifier.
    ///
    /// The index is a raw entry position (includes spacers). If the index is
    /// out of bounds the tab is appended to the end.
    pub fn insert(&mut self, index: usize, tab: T, pane: P) -> Id<(T, S)> {
        let id = self.id_pool.get_id();
        let mut tab_item = TabListItem::new(id.clone(), index, tab);
        tab_item.set_closable(self.default_closable);

        let pane = TabbedPane::new(pane);
        self.content.append_child(pane.slot());

        let entry = TabPanelEntry::new(tab_item, pane);
        let kind = TabOrSpacer::Item(entry);

        if index < self.entries.len() {
            self.ul
                .insert_child_before(&kind, Some(self.entries[index].element()));
            self.entries.insert(index, kind);
        } else {
            self.ul.append_child(&kind);
            self.entries.push(kind);
        }
        self.reindex_entries();
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
    pub fn remove_by_id(&mut self, id: &Id<(T, S)>) -> Option<RemovedEntry<T, P, S>> {
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
                TabOrSpacer::Item(item) if item.tab().id() == id => {
                    found = Some(Found::Tab {
                        entry_index: entry_i,
                        was_selected: item.tab().is_active(),
                    });
                    break;
                }
                TabOrSpacer::Spacer(s) if s.id() == id => {
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
                        let (tab, pane) = entry.into_parts();
                        self.ul.remove_child(&tab);
                        let (pane, slot) = pane.into_parts();
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

                        let (id, inner) = tab.into_parts();
                        self.reindex_entries();
                        Some(RemovedEntry::Tab(
                            TabItemRemoval {
                                id,
                                index: entry_index,
                                item: inner,
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
                        self.reindex_entries();
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
                item.tab_mut().set_is_active(false);
            }
        }
    }

    /// Select the active tab using an entry index (includes spacers).
    ///
    /// Returns the [`Id`] of the selected tab, if any.
    ///
    /// Returns `None` if the given `index` was out of bounds or points to a
    /// spacer. In that case no selection state is changed.
    pub fn select_by_index(&mut self, index: usize) -> Option<Id<(T, S)>> {
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
                item.tab_mut().set_is_active(i == index);
                if i == index {
                    if let Some(ds) = &self.default_slot {
                        ds.set_style("display", "none");
                    }
                    item.show_pane();
                    id = Some(item.tab().id().clone());
                } else {
                    item.hide_pane();
                }
            }
        }
        id
    }

    /// Select the active tab using an [`Id`].
    ///
    /// Returns `Some(())` when the tab exists and was selected, otherwise `None`.
    pub fn select(&mut self, tab_id: &Id<(T, S)>) -> Option<()> {
        let mut found = false;
        for entry in self.entries.iter_mut() {
            if let Some(item) = entry.as_item_mut() {
                let is_match = item.tab().id() == tab_id;
                item.tab_mut().set_is_active(is_match);
                if is_match {
                    if let Some(ds) = &self.default_slot {
                        ds.set_style("display", "none");
                    }
                    item.show_pane();
                    found = true;
                } else {
                    item.hide_pane();
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
    pub fn index_of_tab(&self, tab_id: &Id<(T, S)>) -> Option<usize> {
        self.entries
            .iter()
            .enumerate()
            .find_map(|(index, entry)| match entry {
                TabOrSpacer::Item(e) if e.tab().id() == tab_id => Some(index),
                _ => None,
            })
    }

    /// Return the [`Id`] of the tab at the given entry index, if it's not a spacer.
    pub fn id_of_tab(&self, index: usize) -> Option<Id<(T, S)>> {
        let entry = self.entries.get(index)?;
        let item = entry.as_item()?;
        Some(item.tab().id().clone())
    }

    /// Returns a reference to the active pane, if any.
    ///
    /// "Active" means the tab whose `is_active` is `true`.
    pub fn get_active_pane(&self) -> Option<&P> {
        self.entries.iter().find_map(|entry| match entry {
            TabOrSpacer::Item(e) if e.tab().is_active() => Some(e.pane()),
            _ => None,
        })
    }

    /// Returns a mutable reference to the active pane, if any.
    ///
    /// "Active" means the tab whose `is_active` is `true`.
    pub fn get_active_pane_mut(&mut self) -> Option<&mut P> {
        self.entries.iter_mut().find_map(|entry| match entry {
            TabOrSpacer::Item(e) if e.tab().is_active() => Some(e.pane_mut()),
            _ => None,
        })
    }

    /// Push a spacer onto the end of the tab bar.
    ///
    /// Returns the [`Id`] allocated for the new spacer.
    pub fn push_spacer(&mut self, inner: S) -> Id<(T, S)> {
        let id = self.id_pool.get_id();
        let index = self.entries.len();
        let spacer = TabSpacer::new(id.clone(), index, inner);
        let kind = TabOrSpacer::Spacer(spacer);
        self.ul.append_child(&kind);
        self.entries.push(kind);
        id
    }

    /// Insert a spacer before the tab identified by `tab_id`.
    ///
    /// Returns the [`Id`] of the inserted spacer, or `None` if the tab was not
    /// found.
    pub fn insert_spacer_before(&mut self, tab_id: &Id<(T, S)>, inner: S) -> Option<Id<(T, S)>> {
        let pos = self.entries.iter().enumerate().find_map(|(i, e)| {
            e.as_item()
                .and_then(|item| (item.tab().id() == tab_id).then_some(i))
        });
        let pos = pos?;
        let id = self.id_pool.get_id();
        let spacer = TabSpacer::new(id.clone(), pos, inner);
        let kind = TabOrSpacer::Spacer(spacer);
        self.ul
            .insert_child_before(&kind, Some(self.entries[pos].element()));
        self.entries.insert(pos, kind);
        self.reindex_entries();
        Some(id)
    }

    /// Insert a spacer after the tab identified by `tab_id`.
    ///
    /// Returns the [`Id`] of the inserted spacer, or `None` if the tab was not
    /// found.
    pub fn insert_spacer_after(&mut self, tab_id: &Id<(T, S)>, inner: S) -> Option<Id<(T, S)>> {
        let pos = self.entries.iter().enumerate().find_map(|(i, e)| {
            e.as_item()
                .and_then(|item| (item.tab().id() == tab_id).then_some(i))
        });
        let pos = pos?;
        let id = self.id_pool.get_id();
        let insert_pos = pos + 1;
        let spacer = TabSpacer::new(id.clone(), insert_pos, inner);
        let kind = TabOrSpacer::Spacer(spacer);
        if let Some(next_entry) = self.entries.get(insert_pos) {
            self.ul
                .insert_child_before(&kind, Some(next_entry.element()));
            self.entries.insert(insert_pos, kind);
        } else {
            self.ul.append_child(&kind);
            self.entries.push(kind);
        }
        self.reindex_entries();
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
        self.reindex_entries();
    }

    /// Set the alignment of tabs within the panel.
    ///
    /// This inserts or removes spacers to achieve the desired alignment.
    /// Only available when the spacer type `S` is [`EmptySpacer`] (the
    /// default), since alignment spacers have no content.
    pub fn set_alignment(&mut self, alignment: TabAlignment)
    where
        S: Default,
    {
        self.remove_all_spacers();
        let first_id = self
            .entries
            .iter()
            .filter_map(|e| e.as_item())
            .next()
            .map(|item| item.tab().id().clone());
        match alignment {
            TabAlignment::Start => {
                self.push_spacer(S::default());
            }
            TabAlignment::Center => {
                if let Some(id) = &first_id {
                    self.insert_spacer_before(id, S::default());
                }
                self.push_spacer(S::default());
            }
            TabAlignment::End => {
                if let Some(id) = &first_id {
                    self.insert_spacer_before(id, S::default());
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
                item.tab_mut().set_closable(closable);
            }
        }
    }

    /// Set whether a specific tab should show a close button.
    ///
    /// Does nothing if the tab is not found.
    pub fn set_tab_closable(&mut self, id: &Id<(T, S)>, closable: bool) {
        for entry in self.entries.iter_mut() {
            if let Some(item) = entry.as_item_mut() {
                if item.tab().id() == id {
                    item.tab_mut().set_closable(closable);
                    return;
                }
            }
        }
    }

    /// Update the cached `index` field on all entries to match their `Vec`
    /// position. Call after any mutation that shifts entry positions.
    fn reindex_entries(&mut self) {
        for (i, entry) in self.entries.iter_mut().enumerate() {
            match entry {
                TabOrSpacer::Item(item) => item.tab_mut().set_index(i),
                TabOrSpacer::Spacer(s) => s.set_index(i),
            }
        }
    }

    /// Race all tab items' [`Step`] futures (click + close), returning the first
    /// to resolve. Spacers produce perpetually-pending futures.
    fn item_events(&self) -> impl Future<Output = TabListItemEvent<V, T, S>> + '_ {
        let mut race = std::future::pending().boxed_local();
        for entry in self.entries.iter() {
            if let TabOrSpacer::Item(item) = entry {
                race = race.or(item.tab().step()).boxed_local();
            }
        }
        race
    }
}

/// [`Step`] for [`TabPanel`]: races all tab click/close events (via
/// [`TabListItem::step`]). Reports events without side effects — does not
/// auto-select or auto-remove. The [`User`](TabPanelEvent::User) variant is
/// unreachable.
impl<V: View, P: ViewChild<V> + 'static, T: ViewChild<V> + 'static, S: ViewChild<V> + 'static> Step
    for TabPanel<V, P, T, S>
{
    type Output = TabPanelEvent<V, P, T, S>;

    async fn step(&self) -> Self::Output {
        let ev = self.item_events().await;
        match ev {
            TabListItemEvent::Click(data) => TabPanelEvent::ItemCloseClicked {
                id: data.id,
                index: data.index,
                event: data.event,
            },
            TabListItemEvent::Close(data) => TabPanelEvent::ItemCloseClicked {
                id: data.id,
                index: data.index,
                event: data.event,
            },
            TabListItemEvent::User(()) => {
                unreachable!("item_events only returns click or close variants")
            }
        }
    }
}

/// [`StepMut`] for [`TabPanel`]: races all tab click/close events (via
/// [`TabListItem::step`]). On a click, auto-selects the clicked tab. On a
/// close, auto-removes the tab (reselecting the nearest neighbor) and returns
/// the inner content `T` and pane `P` for cleanup. The
/// [`User`](TabPanelEvent::User) variant is unreachable.
impl<V: View, P: ViewChild<V> + 'static, T: ViewChild<V> + 'static, S: ViewChild<V> + 'static>
    StepMut for TabPanel<V, P, T, S>
{
    type Output = TabPanelEvent<V, P, T, S>;

    async fn step_mut(&mut self) -> TabPanelEvent<V, P, T, S> {
        let ev = self.item_events().await;
        match ev {
            TabListItemEvent::Click(data) => {
                self.select(&data.id);
                TabPanelEvent::ItemClicked {
                    id: data.id,
                    index: data.index,
                    event: data.event,
                }
            }
            TabListItemEvent::Close(data) => {
                let (removal, pane) = match self.remove_by_id(&data.id) {
                    Some(RemovedEntry::Tab(removal, pane)) => (removal, pane),
                    _ => unreachable!("close click on non-existent tab"),
                };
                TabPanelEvent::ItemClosed {
                    id: removal.id,
                    index: removal.index,
                    item: removal.item,
                    pane,
                }
            }
            TabListItemEvent::User(()) => {
                unreachable!("item_events only returns click or close variants")
            }
        }
    }
}

/// [`StepWithMut<TabOrSpacer<V, P, T, S>>`] for [`TabPanel`]: calls the closure
/// once per entry (including spacers), racing all returned futures via
/// [`mogwai::future::race_all`]. The closure receives `&mut TabOrSpacer` and
/// delegates to each child's own step impls. The return type is `Ev` directly
/// (typically wrapped in [`TabPanelEvent::User`]).
impl<V: View, P, T: ViewChild<V> + 'static, S: ViewChild<V> + 'static>
    StepWithMut<TabOrSpacer<V, P, T, S>> for TabPanel<V, P, T, S>
{
    type Output<Ev: 'static> = Ev;

    async fn step_with_mut<Ev>(
        &mut self,
        f: impl for<'a> FnMut(&'a mut TabOrSpacer<V, P, T, S>) -> Pin<Box<dyn Future<Output = Ev> + 'a>>,
    ) -> Ev
    where
        Ev: 'static,
    {
        let entry_futs: Vec<Pin<Box<dyn Future<Output = Ev> + '_>>> =
            self.entries.iter_mut().map(f).collect();

        mogwai::future::race_all(entry_futs).await
    }
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
        panel: TabPanel<V, Widget<V, ()>>,
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
            let ev = self.panel.step_mut().await;
            match ev {
                TabPanelEvent::ItemClicked { id, .. } => {
                    self.panel.select(&id);
                }
                TabPanelEvent::ItemClosed { .. } => {}
                TabPanelEvent::ItemCloseClicked { .. } => {}
                TabPanelEvent::User(()) => {}
            }
        }
    }
}
