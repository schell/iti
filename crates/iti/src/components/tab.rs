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

/// A nav-tabs component.
#[derive(ViewChild, ViewProperties)]
pub struct TabList<V: View, T> {
    #[child]
    #[properties]
    ul: V::Element,
    items: Vec<TabListItem<V, T>>,
    id_pool: IdPool<T>,
}

impl<V: View, T: ViewChild<V>> Default for TabList<V, T> {
    fn default() -> Self {
        rsx! {
            let ul = ul(class = "nav nav-tabs") {
                let items = {vec![]}
            }
        }
        Self {
            ul,
            items,
            id_pool: Default::default(),
        }
    }
}

impl<V: View, T: ViewChild<V>> TabList<V, T> {
    /// Return the number of tabs.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Return a reference to the [`TabListItem`].
    pub fn get(&self, index: usize) -> Option<&TabListItem<V, T>> {
        self.items.get(index)
    }

    /// Iterator over all tab items.
    pub fn iter(&self) -> impl Iterator<Item = &TabListItem<V, T>> {
        self.items.iter()
    }

    /// Push a new tab and return a unique identifier for that tab.
    pub fn push(&mut self, item: T) -> Id<T> {
        let id = self.id_pool.get_id();
        let item = TabListItem::new(id.clone(), item);
        self.ul.append_child(&item);
        self.items.push(item);
        if self.items.len() == 1 {
            self.select_by_index(0);
        }
        id
    }

    /// Pop the last tab off the end of the list of tabs.
    pub fn pop(&mut self) -> Option<(Id<T>, T)> {
        let item = self.items.pop()?;
        self.ul.remove_child(&item);
        item.a.remove_child(&item.inner);
        Some((item.id, item.inner))
    }

    /// Insert a new tab at the given index and return a unique identifier for that tab.
    pub fn insert(&mut self, index: usize, item: T) -> Id<T> {
        let id = self.id_pool.get_id();
        let item = TabListItem::new(id.clone(), item);
        self.ul.append_child(&item);
        self.items.insert(index, item);
        id
    }

    /// Remove a tab by its [`Id`].
    pub fn remove_by_id(&mut self, id: &Id<T>) -> Option<TabItemRemoval<T>> {
        let mut found = None;
        for (i, item) in self.items.iter().enumerate() {
            if &item.id == id {
                found = Some((i, *item.is_active));
                break;
            }
        }
        let (index, was_selected) = found?;
        // Does not panic as we already know it exists
        let item = self.items.remove(index);
        self.ul.remove_child(&item);
        item.a.remove_child(&item.inner);
        Some(TabItemRemoval {
            id: item.id,
            index,
            item: item.inner,
            was_selected,
        })
    }

    pub fn deselect_all(&mut self) {
        for item in self.items.iter_mut() {
            item.is_active.set(false);
        }
    }

    /// Select the active tab using an index.
    ///
    /// Returns the [`Id`] of the select tab, if any.
    ///
    /// Returns `None` if the given `index` was out of bounds.
    pub fn select_by_index(&mut self, index: usize) -> Option<Id<T>> {
        let mut id = None;
        for (i, item) in self.items.iter_mut().enumerate() {
            item.is_active.set(i == index);
            if i == index {
                id = Some(item.id.clone());
            }
        }
        id
    }

    /// Select the active tab using an [`Id`].
    pub fn select_by_id(&mut self, id: &Id<T>) {
        for item in self.items.iter_mut() {
            item.is_active.set(&item.id == id);
        }
    }

    fn item_events(&self) -> impl Future<Output = TabListEvent<V, T>> + '_ {
        let mut race = std::future::pending().boxed_local();
        for (index, item) in self.items.iter().enumerate() {
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
