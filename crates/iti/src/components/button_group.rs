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
#[derive(ViewChild)]
pub struct ButtonGroup<V: View> {
    #[child]
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

    /// Awaits the next click on any child and returns a [`ButtonGroupEvent`]
    /// indicating which item was clicked.
    pub async fn step(&self) -> ButtonGroupEvent<V> {
        self.item_click_events().await
    }

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

#[cfg(feature = "library")]
pub mod library {
    use futures_lite::FutureExt;

    use super::*;
    use crate::components::{button::Button, icon::IconGlyph, Flavor};

    const SIZES: [ButtonGroupSize; 3] = [
        ButtonGroupSize::Small,
        ButtonGroupSize::Default,
        ButtonGroupSize::Large,
    ];

    const SIZE_LABELS: [&str; 3] = ["Small", "Default", "Large"];

    /// Library sandbox for [`ButtonGroup`].
    #[derive(ViewChild)]
    pub struct ButtonGroupLibraryItem<V: View> {
        #[child]
        pub wrapper: V::Element,
        subject_group: ButtonGroup<V>,
        controls_group: ButtonGroup<V>,
        status_text: V::Text,
        size_index: usize,
        is_vertical: bool,
        count: usize,
    }

    impl<V: View> Default for ButtonGroupLibraryItem<V> {
        fn default() -> Self {
            let flavors = [Flavor::Primary, Flavor::Success, Flavor::Danger];
            let labels = ["Alpha", "Beta", "Gamma"];
            let glyphs = [IconGlyph::Plus, IconGlyph::Check, IconGlyph::Xmark];

            let mut subject_group: ButtonGroup<V> = ButtonGroup::default();
            for i in 0..3 {
                let mut btn = Button::new(labels[i], Some(flavors[i]));
                btn.get_icon_mut().set_glyph(glyphs[i]);
                subject_group.push(btn);
            }

            let mut controls_group: ButtonGroup<V> = ButtonGroup::default();
            controls_group.extend([
                Button::new("Add button", None),
                Button::new("Remove last", None),
                Button::new("Cycle size", None),
                Button::new("Toggle vertical", None),
            ]);

            let status_text = V::Text::new("Click a button in the group");

            rsx! {
                let wrapper = div() {
                    div(class = "mb-3") {
                        {&subject_group}
                    }
                    div(class = "mb-3") {
                        p(class = "text-muted") {
                            {&status_text}
                        }
                    }
                    {&controls_group}
                }
            }

            Self {
                wrapper,
                subject_group,
                controls_group,
                status_text,
                size_index: 1,
                is_vertical: false,
                count: 3,
            }
        }
    }

    impl<V: View> ButtonGroupLibraryItem<V> {
        pub async fn step(&mut self) {
            log::info!("waiting on button group step");
            // Race the two button groups
            enum Group<V: View> {
                Control(ButtonGroupEvent<V>),
                Subject(ButtonGroupEvent<V>),
            }
            let control = async { Group::Control(self.controls_group.step().await) };
            let subject = async { Group::Subject(self.subject_group.step().await) };
            let event = control.or(subject).await;

            match event {
                Group::Subject(ev) => {
                    self.status_text
                        .set_text(format!("Clicked button at index {}", ev.index));
                }
                Group::Control(ev) => match ev.index {
                    0 => {
                        self.count += 1;
                        let flavors = [
                            Flavor::Primary,
                            Flavor::Secondary,
                            Flavor::Success,
                            Flavor::Danger,
                            Flavor::Warning,
                            Flavor::Info,
                        ];
                        let flavor = flavors[self.count % flavors.len()];
                        let mut btn = Button::new(format!("Button {}", self.count), Some(flavor));
                        btn.get_icon_mut().set_glyph(IconGlyph::Plus);
                        self.subject_group.push(btn);
                        self.status_text.set_text(format!(
                            "Added button {} (total: {})",
                            self.count,
                            self.subject_group.len()
                        ));
                    }
                    1 => {
                        if !self.subject_group.is_empty() {
                            let _removed = self.subject_group.remove(self.subject_group.len() - 1);
                            self.status_text.set_text(format!(
                                "Removed last button (total: {})",
                                self.subject_group.len()
                            ));
                        } else {
                            self.status_text.set_text("No buttons to remove");
                        }
                    }
                    2 => {
                        self.size_index = (self.size_index + 1) % SIZES.len();
                        self.subject_group.set_size(SIZES[self.size_index]);
                        self.status_text
                            .set_text(format!("Size: {}", SIZE_LABELS[self.size_index]));
                    }
                    3 => {
                        self.is_vertical = !self.is_vertical;
                        self.subject_group.set_is_vertical(self.is_vertical);
                        self.status_text.set_text(if self.is_vertical {
                            "Orientation: vertical"
                        } else {
                            "Orientation: horizontal"
                        });
                    }
                    _ => unreachable!(),
                },
            }
        }
    }
}
