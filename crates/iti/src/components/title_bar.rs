//! Mac OS 9 Platinum-style window title bar.
//!
//! A title bar component with optional close button and icon, featuring the
//! distinctive aluminum pinstripe pattern on the sides.
use mogwai::prelude::*;

use crate::components::icon::{Icon, IconGlyph, IconSize};

/// Event emitted by a [`TitleBar`].
pub enum TitleBarEvent {
    /// The close button was clicked.
    CloseClicked,
}

/// A Mac OS 9 Platinum-style window title bar.
///
/// Features an optional close button on the left, a centered title, and
/// aluminum pinstripe strips on either side of the title. The height is
/// 19px to match the classic Platinum aesthetic.
///
/// # Example
///
/// ```ignore
/// let mut title_bar = TitleBar::new("My Window");
/// title_bar.set_show_close_button(true);
///
/// loop {
///     match title_bar.step().await {
///         TitleBarEvent::CloseClicked => {
///             // handle close
///         }
///     }
/// }
/// ```
#[derive(ViewChild, ViewProperties)]
pub struct TitleBar<V: View> {
    #[child]
    #[properties]
    wrapper: V::Element,
    title: V::Text,
    icon: Icon<V>,
    icon_wrapper: V::Element,
    has_icon: bool,
    close_visible: Proxy<bool>,
    close_click: V::EventListener,
}

impl<V: View> TitleBar<V> {
    /// Create a new title bar with the given title.
    ///
    /// The close button is hidden by default. Use
    /// [`set_show_close_button`](TitleBar::set_show_close_button) to show it.
    pub fn new(title: impl AsRef<str>) -> Self {
        let icon = Icon::new(IconGlyph::File, IconSize::Regular);
        let mut close_visible = Proxy::new(false);

        rsx! {
            let wrapper = div(class = "title-bar") {
                button(
                    type = "button",
                    class = "title-bar-close",
                    style:display = close_visible(v => if *v { "block" } else { "none" }),
                    on:click = close_click,
                ) {}
                div(class = "title-bar-aluminum title-bar-aluminum-left") {}
                let icon_wrapper = span(class = "title-bar-icon") {
                    {&icon}
                }
                span(class = "title-bar-title") {
                    let title_text = ""
                }
                div(class = "title-bar-aluminum title-bar-aluminum-right") {}
            }
        }

        title_text.set_text(title);

        // Icon is hidden by default
        icon_wrapper.remove_child(&icon);

        Self {
            wrapper,
            title: title_text,
            icon,
            icon_wrapper,
            has_icon: false,
            close_visible,
            close_click,
        }
    }

    /// Set the title text.
    pub fn set_title(&self, title: impl AsRef<str>) {
        self.title.set_text(title);
    }

    /// Show or hide the close button.
    pub fn set_show_close_button(&mut self, show: bool) {
        self.close_visible.set(show);
        if show {
            self.wrapper.add_class("has-close-button");
        } else {
            self.wrapper.remove_class("has-close-button");
        }
    }

    /// Returns `true` if the close button is visible.
    pub fn is_close_button_visible(&self) -> bool {
        *self.close_visible
    }

    /// Set the icon to display next to the title.
    ///
    /// Pass `Some(glyph)` to show an icon, or `None` to hide it.
    pub fn set_icon(&mut self, glyph: Option<IconGlyph>) {
        match glyph {
            Some(g) => {
                self.icon.set_glyph(g);
                if !self.has_icon {
                    self.icon_wrapper.append_child(&self.icon);
                    self.has_icon = true;
                }
            }
            None => {
                if self.has_icon {
                    self.icon_wrapper.remove_child(&self.icon);
                    self.has_icon = false;
                }
            }
        }
    }

    /// Access the icon component.
    pub fn get_icon(&self) -> &Icon<V> {
        &self.icon
    }

    /// Mutably access the icon component.
    pub fn get_icon_mut(&mut self) -> &mut Icon<V> {
        &mut self.icon
    }

    /// Await the next title bar event.
    ///
    /// Currently only emits [`TitleBarEvent::CloseClicked`] when the close
    /// button is clicked. If the close button is hidden, this will wait
    /// indefinitely.
    pub async fn step(&self) -> TitleBarEvent {
        self.close_click.next().await;
        TitleBarEvent::CloseClicked
    }
}
