//! A button component.
//!
//! May have an icon.
//! The text can be set.
//! The button can be enabled/disabled.
//! It has a progress spinner.
//! It has a flavor.
//! It has an "onclick" event stream.
use mogwai::prelude::*;

use crate::components::{
    icon::{Icon, IconGlyph, IconSize},
    Flavor,
};

/// A Bootstrap-styled button with icon, spinner, and reactive text/flavor.
#[derive(ViewChild)]
pub struct Button<V: View> {
    #[child]
    button: V::Element,
    icon: Icon<V>,
    flavor: Proxy<Option<Flavor>>,
    text: Proxy<String>,
    on_click: V::EventListener,
    spinner: V::Element,
    spinner_attached: bool,
}

impl<V: View> Button<V> {
    pub fn new(text: impl AsRef<str>, flavor: Option<Flavor>) -> Self {
        let mut flavor = Proxy::new(flavor);
        let mut text = Proxy::new(text.as_ref().to_string());
        let icon = {
            let mut i = Icon::new(IconGlyph::Plus, IconSize::Regular);
            i.set_additional_classes("me-1");
            i
        };
        rsx! {
            let button = button(
                type = "button",
                class = flavor(
                    maybe_flav => {
                        let class = format!("btn btn-{}", maybe_flav.unwrap_or(Flavor::Secondary));
                        class
                    }
                ),
                style:cursor = "pointer",
                on:click = on_click,
            ) {
                span() {
                    {&icon}
                }
                span() {
                    {text(t => t)}
                }
            }
        }

        rsx! {
            let spinner = span(
                class="spinner-border spinner-border-sm ms-1",
                role="status",
                aria_hidden="true"
            ) {}
        }

        Button {
            button,
            icon,
            flavor,
            text,
            on_click,
            spinner,
            spinner_attached: false,
        }
    }

    pub fn get_icon(&self) -> &Icon<V> {
        &self.icon
    }

    pub fn get_icon_mut(&mut self) -> &mut Icon<V> {
        &mut self.icon
    }

    pub fn enable(&self) {
        self.button.remove_property("disabled");
    }

    pub fn disable(&self) {
        self.button.set_property("disabled", "");
    }

    pub fn start_spinner(&mut self) {
        if !self.spinner_attached {
            self.button.append_child(&self.spinner);
            self.spinner_attached = true;
        }
    }

    pub fn stop_spinner(&mut self) {
        if self.spinner_attached {
            self.button.remove_child(&self.spinner);
            self.spinner_attached = false;
        }
    }

    pub fn set_text(&mut self, text: impl AsRef<str>) {
        self.text.set(text.as_ref().into());
    }

    pub fn set_flavor(&mut self, flavor: Option<Flavor>) {
        self.flavor.set(flavor);
    }

    pub async fn step(&self) -> V::Event {
        self.on_click.next().await
    }
}

#[cfg(feature = "library")]
pub mod library {
    use std::pin::Pin;

    use futures_lite::{FutureExt, Stream};
    use mogwai::future::MogwaiFutureExt;

    use super::*;

    #[derive(ViewChild)]
    pub struct ButtonLibraryItem<V: View> {
        #[child]
        pub wrapper: V::Element,
        clicks: usize,
        button: Button<V>,
        flavor_changes: Pin<Box<dyn Stream<Item = Flavor>>>,
    }

    impl<V: View> Default for ButtonLibraryItem<V> {
        fn default() -> Self {
            rsx! {
                let wrapper = fieldset() {
                    div(class = "row") {
                        div() {
                            let button = {Button::new("0 clicks", Some(Flavor::Primary))}
                        }
                    }
                    div(class = "row") {
                        ul() {
                            li() {
                                a(
                                    href = "#",
                                    on:click = click_primary
                                ){
                                    "Change to primary"
                                }
                            }
                            li() {
                                a(
                                    href = "#",
                                    on:click = click_warning
                                ) {
                                    "Change to warning"
                                }
                            }
                        }
                    }
                }
            }

            let flavor_changes = Box::pin(futures_lite::stream::unfold(
                (click_primary, click_warning),
                |(prim, warn)| async move {
                    let flav = prim
                        .next()
                        .map(|_| Flavor::Primary)
                        .or(warn.next().map(|_| Flavor::Warning))
                        .await;
                    Some((flav, (prim, warn)))
                },
            ));
            Self {
                wrapper,
                clicks: 0,
                button,
                flavor_changes,
            }
        }
    }

    impl<V: View> ButtonLibraryItem<V> {
        pub async fn step(&mut self) {
            use futures_lite::StreamExt;
            match self
                .button
                .step()
                .map(Ok)
                .or(self.flavor_changes.next().map(Err))
                .await
            {
                Ok(_event) => {
                    log::debug!("got click");
                    self.clicks += 1;
                    self.button.set_text(if self.clicks == 1 {
                        "1 click".into()
                    } else {
                        format!("{} clicks", self.clicks)
                    });
                }
                Err(Some(flav)) => {
                    self.button.set_flavor(Some(flav));
                }
                _ => unreachable!("blarg!"),
            }
        }
    }
}
