use futures_lite::FutureExt;
use mogwai::{future::MogwaiFutureExt, prelude::*};

use crate::components::modal::{Modal, ModalEvent};

/// Platinum kit sandbox for [`Modal`].
#[derive(ViewChild)]
pub struct PlatinumKitModals<V: View> {
    #[child]
    pub wrapper: V::Element,
    modal: Modal<V>,
    open_click: V::EventListener,
}

impl<V: View> Default for PlatinumKitModals<V> {
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
                    class = "btn",
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

impl<V: View> StepMut for PlatinumKitModals<V> {
    type Output = ();
    async fn step_mut(&mut self) {
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
