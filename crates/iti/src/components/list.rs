//! Item lists.
//!
//! Includes list items and lists.
use std::future::Future;

use mogwai::prelude::*;

use super::Flavor;

struct ItemState {
    flavor: Option<Flavor>,
    is_active: bool,
}

impl ItemState {
    fn class(&self) -> String {
        let list_group = if let Some(flav) = self.flavor.as_ref() {
            format!("list-group-item-{flav}")
        } else {
            "list-group-item".to_string()
        };
        let active = if self.is_active { " active" } else { "" };
        format!("{list_group}{active}")
    }
}

/// A single item within a [`List`].
#[derive(ViewChild)]
pub struct ListItem<V: View, T> {
    #[child]
    li: V::Element,
    item: T,
    on_click: V::EventListener,
    state: Proxy<ItemState>,
}

impl<V: View, T: ViewChild<V>> ListItem<V, T> {
    pub fn new(item: T) -> Self {
        let mut state = Proxy::new(ItemState {
            flavor: None,
            is_active: false,
        });

        rsx! {
            let li = li(
                class = state(s => s.class()),
                on:click = on_click
            ) {
                {&item}
            }
        }

        ListItem {
            li,
            item,
            on_click,
            state,
        }
    }

    pub fn set_flavor(&mut self, flavor: Option<super::Flavor>) {
        self.state.modify(|s| s.flavor = flavor);
    }

    pub fn set_is_active(&mut self, is_active: bool) {
        self.state.modify(|s| s.is_active = is_active);
    }

    pub fn inner(&self) -> &T {
        &self.item
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.item
    }
}

/// Event emitted when a list item is clicked.
#[derive(Debug)]
pub struct ListEvent<V: View> {
    pub index: usize,
    pub event: V::Event,
}

/// A Bootstrap list-group with clickable items.
#[derive(ViewChild)]
pub struct List<V: View, T> {
    #[child]
    ul: V::Element,
    items: Vec<ListItem<V, T>>,
}

impl<V: View, T> Default for List<V, T> {
    fn default() -> Self {
        rsx! {
            let ul = ul(class = "list-group") {
                let items = {vec![]}
            }
        }

        List { ul, items }
    }
}

impl<V: View, A: ViewChild<V>> FromIterator<A> for List<V, A> {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        let mut list = List::default();
        for item in iter.into_iter() {
            list.push(item);
        }
        list
    }
}

impl<V: View, T: ViewChild<V>> List<V, T> {
    pub fn get(&self, index: usize) -> Option<&ListItem<V, T>> {
        self.items.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut ListItem<V, T>> {
        self.items.get_mut(index)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Inserts the item at the given index.
    ///
    /// ## Note
    /// If `index` > len, the item will simply be appended to the end of the list.
    pub fn insert(&mut self, index: usize, item: T) {
        let item = ListItem::new(item);
        if let Some(previous_item) = self.items.get(index) {
            self.ul.insert_child_before(previous_item, Some(&item));
            self.items.insert(index, item);
        } else {
            self.ul.append_child(&item);
            self.items.push(item);
        }
    }

    /// Removes the item at the given index.
    ///
    /// ## Panics
    /// Panics if `index` > len.
    pub fn remove(&mut self, index: usize) -> T {
        let t = self.items.remove(index);
        self.ul.remove_child(&t);
        t.item
    }

    pub fn push(&mut self, item: T) {
        let item = ListItem::new(item);
        self.ul.append_child(&item);
        self.items.push(item);
    }

    fn item_click_events(&self) -> impl Future<Output = ListEvent<V>> + '_ {
        use mogwai::future::*;

        let events = self.items.iter().enumerate().map(|(index, item)| {
            item.on_click
                .next()
                .map(move |event| ListEvent { index, event })
        });
        race_all(events)
    }

    pub async fn step(&self) -> ListEvent<V> {
        self.item_click_events().await
    }

    pub fn iter(&self) -> impl Iterator<Item = &ListItem<V, T>> {
        self.items.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut ListItem<V, T>> {
        self.items.iter_mut()
    }
}
