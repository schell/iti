//! Badge component.
//!
//! A small label for counts, tags, and status indicators.
use mogwai::prelude::*;

use super::Flavor;

struct BadgeState {
    flavor: Flavor,
    pill: bool,
}

/// A Bootstrap badge (`<span class="badge">`).
///
/// Supports reactive text, flavor, and an optional pill (rounded) style.
#[derive(ViewChild)]
pub struct Badge<V: View> {
    #[child]
    span: V::Element,
    text: V::Text,
    state: Proxy<BadgeState>,
}

impl<V: View> Badge<V> {
    pub fn new(initial_text: impl AsRef<str>, flavor: Flavor) -> Self {
        let mut state = Proxy::new(BadgeState {
            flavor,
            pill: false,
        });

        rsx! {
            let span = span(
                class = state(s => {
                    let pill = if s.pill { " rounded-pill" } else { "" };
                    format!("badge text-bg-{}{pill}", s.flavor)
                }),
            ) {
                let text = ""
            }
        }

        text.set_text(initial_text);

        Self { span, text, state }
    }

    pub fn set_text(&self, text: impl AsRef<str>) {
        self.text.set_text(text);
    }

    pub fn set_flavor(&mut self, flavor: Flavor) {
        self.state.modify(|s| s.flavor = flavor);
    }

    pub fn set_pill(&mut self, pill: bool) {
        self.state.modify(|s| s.pill = pill);
    }
}

#[cfg(feature = "library")]
pub mod library {
    use futures_lite::FutureExt;
    use mogwai::future::MogwaiFutureExt;

    use super::*;

    #[derive(ViewChild)]
    pub struct BadgeLibraryItem<V: View> {
        #[child]
        pub wrapper: V::Element,
        badges: Vec<Badge<V>>,
        cycle_click: V::EventListener,
        pill_click: V::EventListener,
        flavor_index: usize,
        is_pill: bool,
    }

    const FLAVORS: [Flavor; 8] = [
        Flavor::Primary,
        Flavor::Secondary,
        Flavor::Success,
        Flavor::Danger,
        Flavor::Warning,
        Flavor::Info,
        Flavor::Light,
        Flavor::Dark,
    ];

    impl<V: View> Default for BadgeLibraryItem<V> {
        fn default() -> Self {
            let badges: Vec<Badge<V>> = FLAVORS
                .iter()
                .map(|&f| Badge::new(format!("{f}"), f))
                .collect();

            rsx! {
                let wrapper = div() {
                    div(class = "mb-3") {
                        {&badges}
                    }
                    div(class = "btn-group") {
                        button(
                            type = "button",
                            class = "btn btn-sm btn-outline-secondary",
                            on:click = cycle_click,
                        ) {
                            "Cycle flavors"
                        }
                        button(
                            type = "button",
                            class = "btn btn-sm btn-outline-secondary",
                            on:click = pill_click,
                        ) {
                            "Toggle pill"
                        }
                    }
                }
            }

            Self {
                wrapper,
                badges,
                cycle_click,
                pill_click,
                flavor_index: 0,
                is_pill: false,
            }
        }
    }

    impl<V: View> BadgeLibraryItem<V> {
        pub async fn step(&mut self) {
            match self
                .cycle_click
                .next()
                .map(Ok)
                .or(self.pill_click.next().map(Err))
                .await
            {
                Ok(_) => {
                    self.flavor_index = (self.flavor_index + 1) % FLAVORS.len();
                    for badge in &mut self.badges {
                        badge.set_flavor(FLAVORS[self.flavor_index]);
                    }
                }
                Err(_) => {
                    self.is_pill = !self.is_pill;
                    for badge in &mut self.badges {
                        badge.set_pill(self.is_pill);
                    }
                }
            }
        }
    }
}
