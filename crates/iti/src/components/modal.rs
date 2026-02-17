//! Modal dialog component.
//!
//! A Bootstrap modal with title, body slot, and close handling.  The backdrop
//! and visibility are managed in pure Rust — no Bootstrap JS required.
use mogwai::prelude::*;

/// Event emitted by a [`Modal`].
pub enum ModalEvent {
    /// The modal was closed (via close button or backdrop click).
    Closed,
}

/// A Bootstrap modal dialog.
///
/// The modal consists of a semi-transparent backdrop and the dialog itself.
/// Call [`Modal::show`] and [`Modal::hide`] to toggle visibility, and
/// [`Modal::step`] to await close events.
#[derive(ViewChild)]
pub struct Modal<V: View> {
    #[child]
    wrapper: V::Element,
    title: V::Text,
    body: V::Element,
    body_child: ProxyChild<V>,
    close_click: V::EventListener,
    backdrop_click: V::EventListener,
    visible: Proxy<bool>,
}

impl<V: View> Modal<V> {
    pub fn new(title: impl AsRef<str>) -> Self {
        let mut visible = Proxy::new(false);

        rsx! {
            let wrapper = div() {
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
                    div(class = "modal-dialog") {
                        div(class = "modal-content") {
                            div(class = "modal-header") {
                                h5(class = "modal-title") {
                                    let title_text = ""
                                }
                                button(
                                    type = "button",
                                    class = "btn-close",
                                    aria_label = "Close",
                                    on:click = close_click,
                                ) {}
                            }
                            let body = div(class = "modal-body") {}
                        }
                    }
                }
            }
        }

        title_text.set_text(title);

        let body_child = ProxyChild::new(&{
            rsx! {
                let placeholder = span() {}
            }
            placeholder
        });
        body.append_child(&body_child);

        Self {
            wrapper,
            title: title_text,
            body,
            body_child,
            close_click,
            backdrop_click,
            visible,
        }
    }

    pub fn set_title(&self, title: impl AsRef<str>) {
        self.title.set_text(title);
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

    /// Await the next modal event (close button or backdrop click).
    pub async fn step(&self) -> ModalEvent {
        use futures_lite::FutureExt;

        self.close_click.next().or(self.backdrop_click.next()).await;
        ModalEvent::Closed
    }
}

#[cfg(feature = "library")]
pub mod library {
    use futures_lite::FutureExt;
    use mogwai::future::MogwaiFutureExt;
    use mogwai::prelude::*;

    use super::*;

    #[derive(ViewChild)]
    pub struct ModalLibraryItem<V: View> {
        #[child]
        pub wrapper: V::Element,
        modal: Modal<V>,
        open_click: V::EventListener,
    }

    impl<V: View> Default for ModalLibraryItem<V> {
        fn default() -> Self {
            let mut modal = Modal::new("Example Modal");

            rsx! {
                let body_content = div() {
                    p() { "This is the modal body. It can contain any content." }
                    p() { "Click the backdrop or the close button to dismiss." }
                }
            }
            modal.set_body(&body_content);

            rsx! {
                let wrapper = div() {
                    button(
                        type = "button",
                        class = "btn btn-primary",
                        on:click = open_click,
                    ) {
                        "Open modal"
                    }
                    {&modal}
                }
            }

            Self {
                wrapper,
                modal,
                open_click,
            }
        }
    }

    impl<V: View> ModalLibraryItem<V> {
        pub async fn step(&mut self) {
            match self
                .open_click
                .next()
                .map(Ok)
                .or(self.modal.step().map(Err))
                .await
            {
                Ok(_) => {
                    self.modal.show();
                }
                Err(ModalEvent::Closed) => {
                    self.modal.hide();
                }
            }
        }
    }
}
