//! Toast notification component.
//!
//! A Bootstrap toast with reactive title, body, and flavor.  Show/hide is
//! managed in pure Rust via a reactive `Proxy<bool>` — no Bootstrap JS required.
use mogwai::prelude::*;

use super::Flavor;

/// Event emitted by a [`Toast`].
pub enum ToastEvent {
    /// The close button was clicked.
    Closed,
}

struct ToastState {
    flavor: Flavor,
    visible: bool,
}

/// A Bootstrap toast notification.
///
/// Call [`Toast::show`] to make it visible and [`Toast::step`] to await user
/// interaction. The toast does **not** auto-dismiss; the caller is responsible
/// for racing a timer against `step()` if auto-dismiss is desired.
#[derive(ViewChild, ViewProperties)]
pub struct Toast<V: View> {
    #[child]
    #[properties]
    div: V::Element,
    title: V::Text,
    body: V::Text,
    state: Proxy<ToastState>,
    close_click: V::EventListener,
}

impl<V: View> Toast<V> {
    pub fn new(title: impl AsRef<str>, body: impl AsRef<str>, flavor: Flavor) -> Self {
        let mut state = Proxy::new(ToastState {
            flavor,
            visible: false,
        });

        rsx! {
            let div = div(
                class = state(s => {
                    if s.visible {
                        "toast show".to_string()
                    } else {
                        "toast".to_string()
                    }
                }),
                role = "alert",
                aria_live = "assertive",
                aria_atomic = "true",
            ) {
                div(
                    class = state(s => format!(
                        "toast-header text-bg-{}", s.flavor
                    )),
                ) {
                    strong(class = "me-auto") {
                        let title_text = ""
                    }
                    button(
                        type = "button",
                        class = "btn-close",
                        aria_label = "Close",
                        on:click = close_click,
                    ) {}
                }
                div(class = "toast-body") {
                    let body_text = ""
                }
            }
        }

        title_text.set_text(title);
        body_text.set_text(body);

        Self {
            div,
            title: title_text,
            body: body_text,
            state,
            close_click,
        }
    }

    pub fn set_title(&self, title: impl AsRef<str>) {
        self.title.set_text(title);
    }

    pub fn set_body(&self, body: impl AsRef<str>) {
        self.body.set_text(body);
    }

    pub fn set_flavor(&mut self, flavor: Flavor) {
        self.state.modify(|s| s.flavor = flavor);
    }

    /// Make the toast visible.
    pub fn show(&mut self) {
        self.state.modify(|s| s.visible = true);
    }

    /// Hide the toast.
    pub fn hide(&mut self) {
        self.state.modify(|s| s.visible = false);
    }
}

impl<V: View> Step for Toast<V> {
    type Output = ToastEvent;
    async fn step(&self) -> ToastEvent {
        self.close_click.next().await;
        ToastEvent::Closed
    }
}
