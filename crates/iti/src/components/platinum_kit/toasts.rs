use futures_lite::FutureExt;
use mogwai::{future::MogwaiFutureExt, prelude::*};

use crate::components::{
    toast::{Toast, ToastEvent},
    Flavor,
};

/// Platinum kit sandbox for [`Toast`].
#[derive(ViewChild)]
pub struct PlatinumKitToasts<V: View> {
    #[child]
    pub wrapper: V::Element,
    toast: Toast<V>,
    show_click: V::EventListener,
    toast_count: usize,
}

impl<V: View> Default for PlatinumKitToasts<V> {
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

impl<V: View> StepMut for PlatinumKitToasts<V> {
    type Output = ();
    async fn step_mut(&mut self) {
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
