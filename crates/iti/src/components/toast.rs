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
#[derive(ViewChild)]
pub struct Toast<V: View> {
    #[child]
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

    /// Await the next toast event (currently only [`ToastEvent::Closed`]).
    pub async fn step(&self) -> ToastEvent {
        self.close_click.next().await;
        ToastEvent::Closed
    }
}

#[cfg(feature = "library")]
pub mod library {
    use futures_lite::FutureExt;
    use mogwai::future::MogwaiFutureExt;

    use super::*;

    #[derive(ViewChild)]
    pub struct ToastLibraryItem<V: View> {
        #[child]
        pub wrapper: V::Element,
        toast: Toast<V>,
        show_click: V::EventListener,
        toast_count: usize,
    }

    impl<V: View> Default for ToastLibraryItem<V> {
        fn default() -> Self {
            let mut toast = Toast::new(
                "Toast Title",
                "Hello! This is a toast message.",
                Flavor::Primary,
            );
            toast.show();

            rsx! {
                let wrapper = div() {
                    div(class = "mb-3") {
                        button(
                            type = "button",
                            class = "btn btn-sm btn-outline-primary",
                            on:click = show_click,
                        ) {
                            "Show toast"
                        }
                    }
                    div(class = "toast-container position-relative") {
                        {&toast}
                    }
                }
            }

            Self {
                wrapper,
                toast,
                show_click,
                toast_count: 0,
            }
        }
    }

    impl<V: View> ToastLibraryItem<V> {
        pub async fn step(&mut self) {
            match self
                .toast
                .step()
                .map(Ok)
                .or(self.show_click.next().map(Err))
                .await
            {
                Ok(ToastEvent::Closed) => {
                    self.toast.hide();
                }
                Err(_) => {
                    self.toast_count += 1;
                    self.toast.set_body(format!(
                        "Shown {} time{}!",
                        self.toast_count,
                        if self.toast_count == 1 { "" } else { "s" }
                    ));
                    self.toast.show();
                }
            }
        }
    }
}
