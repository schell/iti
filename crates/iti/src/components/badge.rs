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
#[derive(ViewChild, ViewProperties)]
pub struct Badge<V: View> {
    #[child]
    #[properties]
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
