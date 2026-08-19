use futures_lite::FutureExt;
use mogwai::prelude::*;

use crate::components::{button::Button, button_group::*, icon::IconGlyph, Flavor};

const SIZES: [ButtonGroupSize; 3] = [
    ButtonGroupSize::Small,
    ButtonGroupSize::Default,
    ButtonGroupSize::Large,
];

const SIZE_LABELS: [&str; 3] = ["Small", "Default", "Large"];

/// Platinum kit sandbox for [`ButtonGroup`].
#[derive(ViewChild)]
pub struct PlatinumKitButtonGroups<V: View> {
    #[child]
    pub wrapper: V::Element,
    subject_group: ButtonGroup<V>,
    controls_group: ButtonGroup<V>,
    status_text: V::Text,
    size_index: usize,
    is_vertical: bool,
    count: usize,
}

impl<V: View> Default for PlatinumKitButtonGroups<V> {
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

impl<V: View> StepMut for PlatinumKitButtonGroups<V> {
    type Output = ();
    async fn step_mut(&mut self) {
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
