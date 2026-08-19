//! A button group component.
//!
//! Groups child elements inside a Bootstrap `btn-group` (or `btn-group-vertical`).
//! Generic over the child type `T`, which is typically [`super::button::Button`]
//! but can be any [`ViewChild`].
//!
//! Supports reactive size and vertical/horizontal orientation.
use std::future::Future;

use mogwai::prelude::*;

use crate::components::button::Button;

/// Size modifier for a [`ButtonGroup`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ButtonGroupSize {
    Small,
    #[default]
    Default,
    Large,
}

impl ButtonGroupSize {
    fn class_suffix(&self) -> &str {
        match self {
            ButtonGroupSize::Small => " btn-group-sm",
            ButtonGroupSize::Default => "",
            ButtonGroupSize::Large => " btn-group-lg",
        }
    }
}

struct ButtonGroupState {
    size: ButtonGroupSize,
    is_vertical: bool,
}

impl ButtonGroupState {
    fn class(&self) -> String {
        let base = if self.is_vertical {
            "btn-group-vertical"
        } else {
            "btn-group"
        };
        format!("{base}{}", self.size.class_suffix())
    }
}

/// Event emitted when a button group item is clicked.
#[derive(Debug)]
pub struct ButtonGroupEvent<V: View> {
    pub index: usize,
    pub event: V::Event,
}

/// A Bootstrap button group that owns its children.
#[derive(ViewChild, ViewProperties)]
pub struct ButtonGroup<V: View> {
    #[child]
    #[properties]
    div: V::Element,
    buttons: Vec<Button<V>>,
    state: Proxy<ButtonGroupState>,
}

impl<V: View> Default for ButtonGroup<V> {
    fn default() -> Self {
        let mut state = Proxy::new(ButtonGroupState {
            size: ButtonGroupSize::Default,
            is_vertical: false,
        });

        rsx! {
            let div = div(
                class = state(s => s.class()),
                role = "group",
            ) {}
        }

        Self {
            div,
            buttons: Vec::new(),
            state,
        }
    }
}

impl<V: View> ButtonGroup<V> {
    /// Returns a reference to the item at the given index.
    pub fn get(&self, index: usize) -> Option<&Button<V>> {
        self.buttons.get(index)
    }

    /// Returns a mutable reference to the item at the given index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Button<V>> {
        self.buttons.get_mut(index)
    }

    /// Returns the number of items in the group.
    pub fn len(&self) -> usize {
        self.buttons.len()
    }

    /// Returns `true` if the group contains no items.
    pub fn is_empty(&self) -> bool {
        self.buttons.is_empty()
    }

    /// Inserts an item at the given index.
    ///
    /// ## Note
    /// If `index` > len, the item will be appended to the end.
    pub fn insert(&mut self, index: usize, item: Button<V>) {
        if let Some(existing) = self.buttons.get(index) {
            self.div.insert_child_before(existing, Some(&item));
            self.buttons.insert(index, item);
        } else {
            self.div.append_child(&item);
            self.buttons.push(item);
        }
    }

    /// Removes the item at the given index and returns the inner child.
    ///
    /// ## Panics
    /// Panics if `index` >= len.
    pub fn remove(&mut self, index: usize) -> Button<V> {
        let b = self.buttons.remove(index);
        self.div.remove_child(&b);
        b
    }

    /// Appends an item to the end of the group.
    pub fn push(&mut self, item: Button<V>) {
        self.div.append_child(&item);
        self.buttons.push(item);
    }

    /// Append many items to the end of the group.
    pub fn extend(&mut self, items: impl IntoIterator<Item = Button<V>>) {
        for item in items.into_iter() {
            self.push(item);
        }
    }

    /// Sets the size modifier for the group.
    pub fn set_size(&mut self, size: ButtonGroupSize) {
        self.state.modify(|s| s.size = size);
    }

    /// Sets whether the group is rendered vertically.
    pub fn set_is_vertical(&mut self, is_vertical: bool) {
        self.state.modify(|s| s.is_vertical = is_vertical);
    }

    fn item_click_events(&self) -> impl Future<Output = ButtonGroupEvent<V>> + '_ {
        use mogwai::future::*;

        let events = self.buttons.iter().enumerate().map(|(index, item)| {
            item.step()
                .map(move |event| ButtonGroupEvent { index, event })
        });
        race_all(events)
    }
}
impl<V: View> Step for ButtonGroup<V> {
    type Output = ButtonGroupEvent<V>;
    async fn step(&self) -> ButtonGroupEvent<V> {
        self.item_click_events().await
    }
}
impl<V: View> ButtonGroup<V> {
    /// Returns an iterator over the items.
    pub fn iter(&self) -> impl Iterator<Item = &Button<V>> {
        self.buttons.iter()
    }

    /// Returns a mutable iterator over the items.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Button<V>> {
        self.buttons.iter_mut()
    }
}

impl<V: View> FromIterator<Button<V>> for ButtonGroup<V> {
    fn from_iter<I: IntoIterator<Item = Button<V>>>(iter: I) -> Self {
        let mut group = ButtonGroup::default();
        for item in iter {
            group.push(item);
        }
        group
    }
}
