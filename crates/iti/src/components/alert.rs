//! Alert messages.
use mogwai::prelude::*;

use super::Flavor;

/// A div-based alert message.
///
/// Its text is settable.
/// Its flavor is settable.
/// It can be hidden and revealed.
#[derive(ViewChild)]
pub struct Alert<V: View> {
    #[child]
    div: V::Element,
    text: V::Text,
    flavor: Proxy<Flavor>,
}

impl<V: View> Alert<V> {
    pub fn new(initial_text: impl AsRef<str>, flavor: Flavor) -> Self {
        let mut flavor = Proxy::new(flavor);

        rsx! {
            let div = div(
                class = flavor(flav => format!("alert alert-{flav}")),
                role = "alert",
            ) {
                let text = ""
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
}
