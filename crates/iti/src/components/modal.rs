//! Modal dialog component.
//!
//! A Platinum-styled modal dialog built from a `.window` container and a
//! [`TitleBar`].  The backdrop and visibility are managed in pure Rust — no
//! Bootstrap JS required.  Pressing Escape while the modal is visible will
//! also close it.
//!
//! By default the modal is anchored to the viewport (covering the entire
//! screen).  Use [`Modal::set_anchored`] to anchor it to a parent element
//! instead — the parent must have `position: relative` (or any non-`static`
//! positioning) so the modal's absolute positioning resolves against it.
use mogwai::prelude::*;
use wasm_bindgen::JsCast;

use crate::components::icon::IconGlyph;
use crate::components::title_bar::TitleBar;

/// Event emitted by a [`Modal`].
pub enum ModalEvent {
    /// The modal was closed (via close button, backdrop click, or Escape key).
    Closed,
}

/// A Platinum-styled modal dialog.
///
/// The modal consists of a semi-transparent backdrop and a `.window` dialog
/// containing a [`TitleBar`] (with a close button) and a body slot.  Call
/// [`Modal::show`] and [`Modal::hide`] to toggle visibility, and
/// [`Modal::step`] to await close events.
///
/// # Anchoring
///
/// By default the modal covers the entire viewport.  When anchored (see
/// [`Modal::set_anchored`]) it covers only its nearest positioned ancestor,
/// allowing it to be scoped to a sub-region of the page.
///
/// # Example
///
/// ```ignore
/// let mut modal = Modal::new("My Dialog");
/// modal.set_body(&content);
///
/// loop {
///     match modal.step().await {
///         ModalEvent::Closed => {
///             modal.hide();
///         }
///     }
/// }
/// ```
#[derive(ViewChild, ViewProperties)]
pub struct Modal<V: View> {
    #[child]
    #[properties]
    wrapper: V::Element,
    title_bar: TitleBar<V>,
    body: V::Element,
    body_child: ProxyChild<V>,
    backdrop_click: V::EventListener,
    keydown: V::EventListener,
    visible: Proxy<bool>,
    anchored: Proxy<bool>,
}

impl<V: View> Modal<V> {
    pub fn new(title: impl AsRef<str>) -> Self {
        let mut visible = Proxy::new(false);
        let mut anchored = Proxy::new(false);

        let mut title_bar = TitleBar::new(title);
        title_bar.set_show_close_button(true);

        rsx! {
            let wrapper = div(
                class = anchored(a => if *a {
                    "modal-root modal-root-anchored"
                } else {
                    "modal-root"
                }),
                document:keydown = keydown,
            ) {
                div(
                    class = visible(v => if *v {
                        "modal-backdrop fade show"
                    } else {
                        "modal-backdrop fade"
                    }),
                    style:display = visible(v => if *v { "block" } else { "none" }),
                    on:click = backdrop_click,
                ) {}
                div(
                    class = visible(v => if *v {
                        "modal fade show"
                    } else {
                        "modal fade"
                    }),
                    tabindex = "-1",
                    style:display = visible(v => if *v { "block" } else { "none" }),
                ) {
                    div(class = "window") {
                        {&title_bar}
                        let body = div(class = "container") {}
                    }
                }
            }
        }

        let body_child = ProxyChild::new(&{
            rsx! {
                let placeholder = span() {}
            }
            placeholder
        });
        body.append_child(&body_child);

        Self {
            wrapper,
            title_bar,
            body,
            body_child,
            backdrop_click,
            keydown,
            visible,
            anchored,
        }
    }

    pub fn set_title(&self, title: impl AsRef<str>) {
        self.title_bar.set_title(title);
    }

    /// Set the icon displayed next to the title.
    ///
    /// Pass `Some(glyph)` to show an icon, or `None` to hide it.
    pub fn set_icon(&mut self, glyph: Option<IconGlyph>) {
        self.title_bar.set_icon(glyph);
    }

    /// Replace the modal body content.
    pub fn set_body(&mut self, content: &impl ViewChild<V>) {
        self.body_child.replace(&self.body, content);
    }

    /// Show the modal and its backdrop.
    pub fn show(&mut self) {
        self.visible.set(true);
    }

    /// Hide the modal and its backdrop.
    pub fn hide(&mut self) {
        self.visible.set(false);
    }

    /// Returns `true` if the modal is currently visible.
    pub fn is_visible(&self) -> bool {
        *self.visible
    }

    /// Anchor the modal to its nearest positioned ancestor instead of the
    /// viewport.
    ///
    /// When anchored, the modal's backdrop and dialog use `position: absolute`
    /// so they cover only the parent element (which must have a non-`static`
    /// `position`, e.g. `relative`).  This is useful for modals that should
    /// only block interaction within a sub-region of the page.
    pub fn set_anchored(&mut self, anchored: bool) {
        self.anchored.set(anchored);
    }

    /// Returns `true` if the modal is anchored to its parent element.
    pub fn is_anchored(&self) -> bool {
        *self.anchored
    }
}

impl<V: View> Step for Modal<V> {
    type Output = ModalEvent;
    async fn step(&self) -> ModalEvent {
        use futures_lite::FutureExt;
        use mogwai::future::MogwaiFutureExt;

        let close_or_backdrop = self
            .title_bar
            .step()
            .map(|_| ())
            .or(self.backdrop_click.next().map(|_| ()));
        let escape = async {
            loop {
                let ev = self.keydown.next().await;
                let is_escape = ev.when_event::<mogwai::web::Web, _>(|e: &web_sys::Event| {
                    e.dyn_ref::<web_sys::KeyboardEvent>()
                        .is_some_and(|ke| ke.key() == "Escape")
                });
                if is_escape == Some(true) {
                    return;
                }
            }
        };
        close_or_backdrop.or(escape).await;
        ModalEvent::Closed
    }
}
