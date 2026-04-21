//! Page tabs (Bootstrap nav-tabs).
use std::{collections::HashMap, future::Future};

use futures_lite::FutureExt;
use mogwai::prelude::*;

use crate::{
    components::pane::Panes,
    id::{Id, IdPool},
};

/// A single tab within a [`TabList`].
#[derive(ViewChild, ViewProperties)]
pub struct TabListItem<V: View, T> {
    #[child]
    #[properties]
    li: V::Element,
    a: V::Element,
    on_click: V::EventListener,
    inner: T,
    is_active: Proxy<bool>,
    id: Id<T>,
}

impl<V: View, T: ViewChild<V>> TabListItem<V, T> {
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

    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Get a reference to this item's [`Id`].
    pub fn id(&self) -> &Id<T> {
        &self.id
    }
}

/// Event emitted by a [`TabList`].
pub enum TabListEvent<V: View, T> {
    ItemClicked {
        id: Id<T>,
        index: usize,
        event: V::Event,
    },
}

/// Result of removing an item from the [`TabList`].
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

/// A flexible spacer element within a [`TabList`].
///
/// Spacers are `flex-grow: 1` elements that absorb available space in the tab
/// bar. Insert them before, after, or between tabs to control alignment.
#[derive(ViewChild)]
struct TabSpacer<V: View> {
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

/// An entry in the [`TabList`] — either a tab item or a spacer.
enum TabEntry<V: View, T> {
    Item(TabListItem<V, T>),
    Spacer(TabSpacer<V>),
}

impl<V: View, T: ViewChild<V>> TabEntry<V, T> {
    /// Get the underlying element for DOM operations.
    fn element(&self) -> &V::Element {
        match self {
            TabEntry::Item(item) => &item.li,
            TabEntry::Spacer(spacer) => &spacer.li,
        }
    }

    /// Try to get the entry as a tab item reference.
    fn as_item(&self) -> Option<&TabListItem<V, T>> {
        match self {
            TabEntry::Item(item) => Some(item),
            TabEntry::Spacer(_) => None,
        }
    }

    /// Try to get the entry as a mutable tab item reference.
    fn as_item_mut(&mut self) -> Option<&mut TabListItem<V, T>> {
        match self {
            TabEntry::Item(item) => Some(item),
            TabEntry::Spacer(_) => None,
        }
    }

    /// Returns `true` if this entry is a spacer.
    fn is_spacer(&self) -> bool {
        matches!(self, TabEntry::Spacer(_))
    }
}

impl<V: View, T: ViewChild<V>> ViewChild<V> for TabEntry<V, T> {
    fn as_append_arg(
        &self,
    ) -> AppendArg<V, impl Iterator<Item = std::borrow::Cow<'_, V::Node>>> {
        match self {
            TabEntry::Item(item) => item.as_boxed_append_arg(),
            TabEntry::Spacer(spacer) => spacer.as_boxed_append_arg(),
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

/// A nav-tabs component.
#[derive(ViewChild, ViewProperties)]
pub struct TabList<V: View, T> {
    #[child]
    #[properties]
    ul: V::Element,
    entries: Vec<TabEntry<V, T>>,
    id_pool: IdPool<T>,
}

impl<V: View, T: ViewChild<V>> Default for TabList<V, T> {
    fn default() -> Self {
        rsx! {
            let ul = ul(class = "nav nav-tabs") {}
        }
        Self {
            ul,
            entries: vec![],
            id_pool: Default::default(),
        }
    }
}

impl<V: View, T: ViewChild<V>> TabList<V, T> {
    /// Return the number of tabs (spacers are not counted).
    pub fn len(&self) -> usize {
        self.entries.iter().filter(|e| e.as_item().is_some()).count()
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
    }

    /// Iterator over all tab items (spacers are skipped).
    pub fn iter(&self) -> impl Iterator<Item = &TabListItem<V, T>> {
        self.entries.iter().filter_map(|e| e.as_item())
    }

    /// Push a new tab and return a unique identifier for that tab.
    pub fn push(&mut self, item: T) -> Id<T> {
        let id = self.id_pool.get_id();
        let item = TabListItem::new(id.clone(), item);
        let entry = TabEntry::Item(item);
        self.ul.append_child(&entry);
        self.entries.push(entry);
        if self.len() == 1 {
            self.select_by_index(0);
        }
        id
    }

    /// Pop the last tab off the end of the list.
    ///
    /// Spacers at the end are skipped — this removes the last actual tab.
    pub fn pop(&mut self) -> Option<(Id<T>, T)> {
        let pos = self
            .entries
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, e)| e.as_item().map(|_| i))?;
        let entry = self.entries.remove(pos);
        match entry {
            TabEntry::Item(item) => {
                self.ul.remove_child(&item);
                item.a.remove_child(&item.inner);
                Some((item.id, item.inner))
            }
            TabEntry::Spacer(_) => unreachable!(),
        }
    }

    /// Insert a new tab at the given tab index and return a unique identifier.
    ///
    /// The index counts only tab items, not spacers. If the index is out of
    /// bounds the tab is appended to the end.
    pub fn insert(&mut self, index: usize, item: T) -> Id<T> {
        let id = self.id_pool.get_id();
        let item = TabListItem::new(id.clone(), item);
        let entry = TabEntry::Item(item);

        // Find the entry-vec position of the Nth tab item.
        let entry_pos = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.as_item().is_some())
            .nth(index)
            .map(|(i, _)| i);

        if let Some(pos) = entry_pos {
            self.ul
                .insert_child_before(&entry, Some(self.entries[pos].element()));
            self.entries.insert(pos, entry);
        } else {
            self.ul.append_child(&entry);
            self.entries.push(entry);
        }
        id
    }

    /// Remove a tab by its [`Id`].
    pub fn remove_by_id(&mut self, id: &Id<T>) -> Option<TabItemRemoval<T>> {
        let mut found = None;
        let mut tab_index = 0;
        for (entry_i, entry) in self.entries.iter().enumerate() {
            if let Some(item) = entry.as_item() {
                if &item.id == id {
                    found = Some((entry_i, tab_index, *item.is_active));
                    break;
                }
                tab_index += 1;
            }
        }
        let (entry_index, tab_index, was_selected) = found?;
        let entry = self.entries.remove(entry_index);
        match entry {
            TabEntry::Item(item) => {
                self.ul.remove_child(&item);
                item.a.remove_child(&item.inner);
                Some(TabItemRemoval {
                    id: item.id,
                    index: tab_index,
                    item: item.inner,
                    was_selected,
                })
            }
            TabEntry::Spacer(_) => unreachable!(),
        }
    }

    /// Deselect all tabs.
    pub fn deselect_all(&mut self) {
        for entry in self.entries.iter_mut() {
            if let Some(item) = entry.as_item_mut() {
                item.is_active.set(false);
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
                item.is_active.set(tab_i == index);
                if tab_i == index {
                    id = Some(item.id.clone());
                }
                tab_i += 1;
            }
        }
        id
    }

    /// Select the active tab using an [`Id`].
    pub fn select_by_id(&mut self, id: &Id<T>) {
        for entry in self.entries.iter_mut() {
            if let Some(item) = entry.as_item_mut() {
                item.is_active.set(&item.id == id);
            }
        }
    }

    /// Push a spacer onto the end of the tab bar.
    pub fn push_spacer(&mut self) {
        let spacer = TabSpacer::new();
        let entry = TabEntry::Spacer(spacer);
        self.ul.append_child(&entry);
        self.entries.push(entry);
    }

    /// Insert a spacer before the tab identified by `tab_id`.
    ///
    /// Does nothing if the tab is not found.
    pub fn insert_spacer_before(&mut self, tab_id: &Id<T>) {
        let pos = self.entries.iter().enumerate().find_map(|(i, e)| {
            e.as_item().and_then(|item| (&item.id == tab_id).then_some(i))
        });
        if let Some(pos) = pos {
            let spacer = TabSpacer::new();
            let entry = TabEntry::Spacer(spacer);
            self.ul
                .insert_child_before(&entry, Some(self.entries[pos].element()));
            self.entries.insert(pos, entry);
        }
    }

    /// Insert a spacer after the tab identified by `tab_id`.
    ///
    /// Does nothing if the tab is not found.
    pub fn insert_spacer_after(&mut self, tab_id: &Id<T>) {
        let pos = self.entries.iter().enumerate().find_map(|(i, e)| {
            e.as_item().and_then(|item| (&item.id == tab_id).then_some(i))
        });
        if let Some(pos) = pos {
            let spacer = TabSpacer::new();
            let entry = TabEntry::Spacer(spacer);
            let insert_pos = pos + 1;
            if let Some(next_entry) = self.entries.get(insert_pos) {
                self.ul
                    .insert_child_before(&entry, Some(next_entry.element()));
                self.entries.insert(insert_pos, entry);
            } else {
                self.ul.append_child(&entry);
                self.entries.push(entry);
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

    fn item_events(&self) -> impl Future<Output = TabListEvent<V, T>> + '_ {
        let mut race = std::future::pending().boxed_local();
        for (index, item) in self.iter().enumerate() {
            let click = async move {
                let event = item.on_click.next().await;
                TabListEvent::ItemClicked {
                    id: item.id.clone(),
                    index,
                    event,
                }
            };
            race = race.or(click).boxed_local();
        }
        race
    }

    pub async fn step(&self) -> TabListEvent<V, T> {
        self.item_events().await
    }
}

/// A panel topped with a tab list.
#[derive(ViewChild, ViewProperties)]
pub struct TabPanel<V: View, T, P> {
    #[child]
    #[properties]
    window: V::Element,

    tabs: TabList<V, T>,
    panes: Panes<V, P>,

    tabs_to_panes: HashMap<Id<T>, Id<P>>,
}

impl<V: View, T: ViewChild<V>, P: ViewChild<V>> TabPanel<V, T, P> {
    /// Create a new `TabPanel` with the default pane.
    pub fn new(default_pane: P) -> Self {
        rsx! {
            let window = div(class = "tab-panel") {
                let tabs = {TabList::<V, T>::default()}
                let content = div(class = "container-fluid") { }
            }
        }
        let panes = Panes::new_retained(content, default_pane);

        Self {
            window,
            tabs,
            panes,
            tabs_to_panes: Default::default(),
        }
    }

    /// Push a new tab onto the end of the stack.
    pub fn push(&mut self, tab: T, pane: P) -> Id<T> {
        let tid = self.tabs.push(tab);
        self.tabs.select_by_id(&tid);
        let pid = self.panes.add_pane(pane);
        self.panes.select(&pid);
        self.tabs_to_panes.insert(tid.clone(), pid);
        tid
    }

    /// Set the alignment of tabs within the panel.
    ///
    /// This inserts or removes spacers to achieve the desired alignment.
    pub fn set_alignment(&mut self, alignment: TabAlignment) {
        self.tabs.remove_all_spacers();
        let first_id = self.tabs.iter().next().map(|item| item.id.clone());
        match alignment {
            TabAlignment::Start => {
                self.tabs.push_spacer();
            }
            TabAlignment::Center => {
                if let Some(id) = &first_id {
                    self.tabs.insert_spacer_before(id);
                }
                self.tabs.push_spacer();
            }
            TabAlignment::End => {
                if let Some(id) = &first_id {
                    self.tabs.insert_spacer_before(id);
                }
            }
        }
    }

    /// Select a tab.
    ///
    /// Returns `Some(())` when the tab exists and was selected, otherwise `None`.
    pub fn select(&mut self, tab_id: &Id<T>) -> Option<()> {
        let pane_id = self.tabs_to_panes.get(tab_id)?;
        self.tabs.select_by_id(tab_id);
        self.panes.select(pane_id).then_some(())
    }

    /// Returns a reference to the active pane, if any.
    pub fn get_active_pane(&self) -> Option<&P> {
        self.tabs.iter().find_map(|tab| {
            tab.is_active.then_some(())?;
            let pane_id = self.tabs_to_panes.get(&tab.id)?;
            self.panes.get_pane(pane_id)
        })
    }

    /// Returns a reference to the active pane, if any.
    pub fn get_active_pane_mut(&mut self) -> Option<&mut P> {
        let pane_id = self
            .tabs
            .iter()
            .find_map(|tab| {
                tab.is_active.then_some(())?;
                self.tabs_to_panes.get(&tab.id)
            })?
            .clone();
        self.panes.get_pane_mut(&pane_id)
    }

    /// Step the panel.
    pub async fn step(&mut self) -> TabListEvent<V, T> {
        let ev = self.tabs.step().await;
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

#[cfg(feature = "library")]
pub mod library {

    use crate::components::{pane::RestartPanes, widget::Widget};

    use super::*;

    #[derive(ViewChild)]
    pub struct TabListLibraryItem<V: View> {
        #[child]
        pub div: V::Element,
        list: TabList<V, V::Element>,
        panes: RestartPanes<V, Widget<V, ()>>,
        pane_ids: Vec<crate::id::Id<Widget<V, ()>>>,
    }

    impl<V: View> Default for TabListLibraryItem<V> {
        fn default() -> Self {
            rsx! {
                let div = div() {
                    let list = {TabList::default()}
                    let pane = div() {}
                }
            }
            let mut item = Self {
                div,
                list,
                panes: RestartPanes::new(pane, {
                    rsx! {
                        let html = div() {}
                    }
                    Widget::new(html, futures_lite::stream::pending())
                }),
                pane_ids: vec![],
            };

            item.list.push(Self::new_html_for_tab("Tab Zero"));
            let id_0 = item.panes.add_pane(|| {
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
                Widget::new(
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
                )
            });
            item.pane_ids.push(id_0);

            item.list.push(Self::new_html_for_tab("Tab 1"));
            let id_1 = item.panes.add_pane(|| {
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

                Widget::new(
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
                )
            });
            item.pane_ids.push(id_1);

            item.list.push(Self::new_html_for_tab("Tabbity Too"));
            let id_2 = item.panes.add_pane(|| {
                rsx! {
                    let html = div(class = "container") {
                        div(class = "row") {
                            h1() { "Last Pane" }
                            p() { "Super important stuff here, y'all." }
                        }
                    }
                }
                Widget::new(html, futures_lite::stream::pending())
            });
            item.pane_ids.push(id_2);

            item
        }
    }

    impl<V: View> TabListLibraryItem<V> {
        fn new_html_for_tab(title: impl AsRef<str>) -> V::Element {
            rsx! {
                let html = span() {
                    {title.into_text::<V>()}
                }
            }

            html
        }

        pub fn select(&mut self, index: usize) {
            log::info!("selecting pane {index}");
            self.list.select_by_index(index);
            if let Some(id) = self.pane_ids.get(index) {
                let _ = self.panes.select(id);
            }
        }

        pub async fn step(&mut self) {
            let pane_fut = async {
                self.panes.current_pane_mut().step().await;
                None::<TabListEvent<V, _>>
            };
            let list_fut = async {
                let event = self.list.step().await;
                Some(event)
            };
            if let Some(TabListEvent::ItemClicked {
                id: _,
                index,
                event: _,
            }) = pane_fut.or(list_fut).await
            {
                self.select(index);
            }
        }
    }
}
