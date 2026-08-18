//! Alert messages.
use mogwai::prelude::*;

use super::Flavor;

/// A div-based alert message.
///
/// Its text is settable.
/// Its flavor is settable.
/// It can be hidden and revealed.
#[derive(ViewChild, ViewProperties)]
pub struct Alert<V: View> {
    #[child]
    #[properties]
    div: V::Element,
    text: V::Text,
    flavor: Proxy<Flavor>,
}

impl<V: View> Alert<V> {
    pub fn new(initial_text: impl AsRef<str>, flavor: Flavor) -> Self {
        let mut flavor = Proxy::new(flavor);

        rsx! {
            let div = div(
                class = "alert",
                role = "alert",
            ) {
                span(class = flavor(flav => format!("alert-{flav}"))) {
                    let text = ""
                }
            }
        }

        text.set_text(initial_text);

        Self { div, text, flavor }
    }

    pub fn set_text(&self, text: impl AsRef<str>) {
        self.text.set_text(text);
    }

    pub fn set_flavor(&mut self, flavor: Flavor) {
        self.flavor.set(flavor);
    }

    pub fn set_is_visible(&self, is_visible: bool) {
        if is_visible {
            self.div.remove_style("visibility");
        } else {
            self.div.set_style("visibility", "hidden");
        }
    }

    /// Stretch the alert horizontally into the panel's padding.
    pub fn set_flush_x(&self) {
        self.add_class("alert-flush-x");
    }

    /// Extend the alert up into the panel's top padding.
    pub fn set_flush_top(&self) {
        self.add_class("alert-flush-top");
    }

    /// Extend the alert down into the panel's bottom padding.
    pub fn set_flush_bottom(&self) {
        self.add_class("alert-flush-bottom");
    }
}
