//! Page tabs (Bootstrap nav-tabs).
use std::future::Future;

use futures_lite::FutureExt;
use mogwai::prelude::*;

use crate::id::{Id, IdPool};

/// A single tab within a [`TabList`].
#[derive(ViewChild)]
pub struct TabListItem<V: View, T> {
    #[child]
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
}

/// Event emitted by a [`TabList`].
pub enum TabListEvent<V: View, T> {
    ItemClicked {
        id: Id<T>,
        index: usize,
        event: V::Event,
    },
}

/// A Bootstrap nav-tabs component.
#[derive(ViewChild)]
pub struct TabList<V: View, T> {
    #[child]
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

    /// Remove a tab by its index.
    ///
    /// ## Panics
    /// Panics if there is no tab item at the given index.
    pub fn remove_by_index(&mut self, index: usize) -> (Id<T>, T) {
        let item = self.items.remove(index);
        self.ul.remove_child(&item);
        item.a.remove_child(&item.inner);
        (item.id, item.inner)
    }

    /// Remove a tab by its [`Id`].
    pub fn remove_by_id(&mut self, id: &Id<T>) -> Option<T> {
        let mut found = None;
        for (i, item) in self.items.iter().enumerate() {
            if &item.id == id {
                found = Some(i);
                break;
            }
        }
        found.map(|i| self.remove_by_index(i).1)
    }

    pub fn deselect_all(&mut self) {
        for item in self.items.iter_mut() {
            item.is_active.set(false);
        }
    }

    /// Select the active tab using an index.
    pub fn select_by_index(&mut self, index: usize) {
        self.deselect_all();
        if let Some(item) = self.items.get_mut(index) {
            item.is_active.set(true);
        }
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
            };

            item.list.push(Self::new_html_for_tab("Tab Zero"));
            item.panes.add_pane(|| {
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

            item.list.push(Self::new_html_for_tab("Tab 1"));
            item.panes.add_pane(|| {
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

            item.list.push(Self::new_html_for_tab("Tabbity Too"));
            item.panes.add_pane(|| {
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
            self.panes.select(index);
        }

        pub async fn step(&mut self) {
            let pane_fut = async {
                self.panes.get_pane_mut().step().await;
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
