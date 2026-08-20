//! Button components.
//!
//! Provides [`Button`] (the standard Platinum button) and [`PrimaryButton`]
//! (a default-action button wrapped in the distinctive Mac OS 9 outer ring).
//!
//! Buttons may have an icon, a progress spinner, and a reactive text/flavor.
//! Use `step()` to await the next click event.
use std::future::Future;

use mogwai::prelude::*;

use crate::components::{
    icon::{Icon, IconGlyph, IconSize},
    Flavor,
};

/// A Platinum-styled button with icon, spinner, and reactive text/flavor.
#[derive(ViewChild, ViewProperties)]
pub struct Button<V: View> {
    #[child]
    #[properties]
    button: V::Element,
    icon: Icon<V>,
    flavor: Proxy<Option<Flavor>>,
    text: Proxy<String>,
    on_click: V::EventListener,
    spinner: V::Element,
    spinner_attached: bool,
    icon_wrapper: V::Element,
    has_icon: bool,
}

impl<V: View> Button<V> {
    pub fn new(text: impl AsRef<str>, flavor: Option<Flavor>) -> Self {
        let mut flavor = Proxy::new(flavor);
        let mut text = Proxy::new(text.as_ref().to_string());
        let icon = {
            let i = Icon::new(IconGlyph::Plus, IconSize::Regular);
            i.add_class("me-1");
            i
        };
        rsx! {
            let button = button(
                type = "button",
                class = flavor(
                    maybe_flav => {
                        match maybe_flav {
                            Some(Flavor::Link) => "btn btn-link".to_string(),
                            Some(flav) => format!("btn flavor-{flav}"),
                            None => "btn".to_string(),
                        }
                    }
                ),
                style:cursor = "pointer",
                on:click = on_click,
            ) {
                let icon_wrapper = span() { }
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
            flavor,
            text,
            on_click,
            spinner,
            spinner_attached: false,
            icon,
            icon_wrapper,
            has_icon: false,
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

    /// Show or hide the icon, reclaiming the layout space.
    pub fn set_has_icon(&mut self, has_icon: bool) {
        self.has_icon = has_icon;
        if self.has_icon {
            self.icon_wrapper.append_child(&self.icon);
        } else {
            self.icon_wrapper.remove_child(&self.icon);
        }
    }
}

impl<V: View> Step for Button<V> {
    type Output = V::Event;
    fn step(&self) -> impl Future<Output = V::Event> {
        self.on_click.next()
    }
}

/// A primary (default action) button with the Mac OS 9 outer ring.
///
/// Wraps a standard [`Button`] in a frame element that provides the
/// distinctive double-border ring used for the default action in dialogs.
///
/// All mutating methods delegate to the inner [`Button`]. Access the inner
/// button directly via [`button()`](PrimaryButton::button) or
/// [`button_mut()`](PrimaryButton::button_mut) for full API access.
#[derive(ViewChild, ViewProperties)]
pub struct PrimaryButton<V: View> {
    #[child]
    #[properties]
    frame: V::Element,
    button: Button<V>,
}

impl<V: View> PrimaryButton<V> {
    pub fn new(text: impl AsRef<str>, flavor: Option<Flavor>) -> Self {
        let button = Button::new(text, flavor);
        rsx! {
            let frame = span(class = "btn-primary-ring") {
                {&button}
            }
        }
        Self { frame, button }
    }

    /// Access the inner button.
    pub fn button(&self) -> &Button<V> {
        &self.button
    }

    /// Mutably access the inner button.
    pub fn button_mut(&mut self) -> &mut Button<V> {
        &mut self.button
    }

    pub fn set_text(&mut self, text: impl AsRef<str>) {
        self.button.set_text(text);
    }

    pub fn set_flavor(&mut self, flavor: Option<Flavor>) {
        self.button.set_flavor(flavor);
    }

    pub fn enable(&self) {
        self.button.enable();
    }

    pub fn disable(&self) {
        self.button.disable();
    }

    pub fn start_spinner(&mut self) {
        self.button.start_spinner();
    }

    pub fn stop_spinner(&mut self) {
        self.button.stop_spinner();
    }

    pub fn set_has_icon(&mut self, has_icon: bool) {
        self.button.set_has_icon(has_icon);
    }

    pub fn get_icon(&self) -> &Icon<V> {
        self.button.get_icon()
    }

    pub fn get_icon_mut(&mut self) -> &mut Icon<V> {
        self.button.get_icon_mut()
    }
}

impl<V: View> Step for PrimaryButton<V> {
    type Output = V::Event;
    fn step(&self) -> impl Future<Output = V::Event> {
        self.button.step()
    }
}
