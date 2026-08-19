use futures_lite::FutureExt;
use mogwai::{future::MogwaiFutureExt, prelude::*};

use crate::components::modal::Modal;

/// Platinum kit sandbox for [`Modal`].
#[derive(ViewChild)]
pub struct PlatinumKitModals<V: View> {
    #[child]
    pub wrapper: V::Element,
    modal: Modal<V>,
    open_click: V::EventListener,
    anchored_modal: Modal<V>,
    anchored_open_click: V::EventListener,
}

impl<V: View> Default for PlatinumKitModals<V> {
    fn default() -> Self {
        let mut modal = Modal::new("Example Modal");

        rsx! {
            let body_content = div() {
                p() { "This is the modal body. It can contain any content." }
                p() { "Click the backdrop, the close button, or press Escape to dismiss." }
            }
        }
        modal.set_body(&body_content);

        let mut anchored_modal = Modal::new("Anchored Modal");
        anchored_modal.set_anchored(true);

        rsx! {
            let anchored_body = div() {
                p() { "This modal is anchored to the bordered box below." }
                p() { "It only covers its parent container, not the whole page." }
            }
        }
        anchored_modal.set_body(&anchored_body);

        rsx! {
            let wrapper = div() {
                button(
                    type = "button",
                    class = "btn",
                    on:click = open_click,
                ) {
                    "Open viewport modal"
                }
                div(
                    class = "d-flex flex-column gap-2",
                    style = "position: relative; height: 200px; outline: 1px solid var(--black900); padding: 1em; margin-top: 1em;",
                ) {
                    p() { "Parent container (position: relative)" }
                    button(
                        type = "button",
                        class = "btn",
                        on:click = anchored_open_click,
                    ) {
                        "Open anchored modal"
                    }
                    {&anchored_modal}
                }
                {&modal}
            }
        }

        Self {
            wrapper,
            modal,
            open_click,
            anchored_modal,
            anchored_open_click,
        }
    }
}

impl<V: View> StepMut for PlatinumKitModals<V> {
    type Output = ();
    async fn step_mut(&mut self) {
        let open = self.open_click.next().map(|_| 0);
        let anchored_open = self.anchored_open_click.next().map(|_| 1);
        let modal_event = self.modal.step().map(|_| 2);
        let anchored_event = self.anchored_modal.step().map(|_| 3);

        match open
            .or(anchored_open)
            .or(modal_event)
            .or(anchored_event)
            .await
        {
            0 => {
                self.modal.show();
            }
            1 => {
                self.anchored_modal.show();
            }
            2 => {
                self.modal.hide();
            }
            3 => {
                self.anchored_modal.hide();
            }
            _ => {}
        }
    }
}
