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

#[cfg(feature = "library")]
pub mod library {
    use futures_lite::FutureExt;
    use mogwai::future::MogwaiFutureExt;

    use super::*;

    const FLAVORS: [Flavor; 9] = [
        Flavor::Primary,
        Flavor::Secondary,
        Flavor::Success,
        Flavor::Danger,
        Flavor::Warning,
        Flavor::Info,
        Flavor::Light,
        Flavor::Dark,
        Flavor::Link,
    ];

    #[derive(ViewChild)]
    pub struct AlertLibraryItem<V: View> {
        #[child]
        pub wrapper: V::Element,
        alert: Alert<V>,
        cycle_click: V::EventListener,
        toggle_click: V::EventListener,
        flavor_index: usize,
        visible: bool,
    }

    impl<V: View> Default for AlertLibraryItem<V> {
        fn default() -> Self {
            let alert = Alert::new("This is a Bootstrap alert!", Flavor::Primary);

            rsx! {
                let wrapper = div() {
                    div(class = "mb-3") {
                        {&alert}
                    }
                    div(class = "btn-group") {
                        button(
                            type = "button",
                            class = "btn btn-sm btn-outline-primary",
                            on:click = cycle_click,
                        ) {
                            "Cycle flavor"
                        }
                        button(
                            type = "button",
                            class = "btn btn-sm btn-outline-secondary",
                            on:click = toggle_click,
                        ) {
                            "Toggle visibility"
                        }
                    }
                }
            }

            Self {
                wrapper,
                alert,
                cycle_click,
                toggle_click,
                flavor_index: 0,
                visible: true,
            }
        }
    }

    impl<V: View> AlertLibraryItem<V> {
        pub async fn step(&mut self) {
            match self
                .cycle_click
                .next()
                .map(Ok)
                .or(self.toggle_click.next().map(Err))
                .await
            {
                Ok(_) => {
                    self.flavor_index = (self.flavor_index + 1) % FLAVORS.len();
                    let flavor = FLAVORS[self.flavor_index];
                    self.alert.set_flavor(flavor);
                    self.alert.set_text(format!("This is a {flavor} alert!"));
                }
                Err(_) => {
                    self.visible = !self.visible;
                    self.alert.set_is_visible(self.visible);
                }
            }
        }
    }
}
