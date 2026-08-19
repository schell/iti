use futures_lite::FutureExt;
use mogwai::{future::MogwaiFutureExt, prelude::*};

use crate::components::list::{List, ListEvent};

enum ListAction<V: View> {
    ItemClicked(ListEvent<V>),
    Add,
    Remove,
}

/// Platinum kit sandbox for [`List`].
#[derive(ViewChild)]
pub struct PlatinumKitLists<V: View> {
    #[child]
    pub wrapper: V::Element,
    list: List<V, V::Element>,
    add_click: V::EventListener,
    remove_click: V::EventListener,
    selected: Option<usize>,
    count: usize,
}

impl<V: View> Default for PlatinumKitLists<V> {
    fn default() -> Self {
        let mut list = List::default();
        for label in ["Apple", "Banana", "Cherry"] {
            let text = V::Text::new(label);
            rsx! {
                let el = span() { {text} }
            }
            list.push(el);
        }

        rsx! {
            let wrapper = div() {
                div(class = "mb-3") {
                    {&list}
                }
                div(class = "btn-group") {
                    button(
                        type = "button",
                        class = "btn btn-sm btn-outline-success",
                        on:click = add_click,
                    ) {
                        "Add item"
                    }
                    button(
                        type = "button",
                        class = "btn btn-sm btn-outline-danger",
                        on:click = remove_click,
                    ) {
                        "Remove selected"
                    }
                }
            }
        }

        Self {
            wrapper,
            list,
            add_click,
            remove_click,
            selected: None,
            count: 3,
        }
    }
}

impl<V: View> StepMut for PlatinumKitLists<V> {
    type Output = ();
    async fn step_mut(&mut self) {
        let action = self
            .list
            .step()
            .map(ListAction::ItemClicked)
            .or(self.add_click.next().map(|_| ListAction::Add))
            .or(self.remove_click.next().map(|_| ListAction::Remove))
            .await;

        match action {
            ListAction::ItemClicked(ListEvent { index, .. }) => {
                // Deselect previous
                if let Some(prev) = self.selected {
                    if let Some(item) = self.list.get_mut(prev) {
                        item.set_is_active(false);
                    }
                }
                // Select new
                if let Some(item) = self.list.get_mut(index) {
                    item.set_is_active(true);
                }
                self.selected = Some(index);
            }
            ListAction::Add => {
                self.count += 1;
                let text = V::Text::new(format!("Item {}", self.count));
                rsx! {
                    let el = span() { {text} }
                }
                self.list.push(el);
            }
            ListAction::Remove => {
                if let Some(index) = self.selected.take() {
                    if index < self.list.len() {
                        self.list.remove(index);
                    }
                }
            }
        }
    }
}
