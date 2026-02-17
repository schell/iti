//! Progress bar component.
//!
//! A Bootstrap progress bar with reactive value, flavor, and optional
//! striped/animated styles.
use mogwai::prelude::*;

use super::Flavor;

struct ProgressState {
    value: u8,
    flavor: Flavor,
    striped: bool,
    animated: bool,
}

/// A Bootstrap progress bar.
///
/// The value ranges from 0 to 100. Setting a value outside this range clamps
/// it to the nearest bound.
#[derive(ViewChild)]
pub struct Progress<V: View> {
    #[child]
    wrapper: V::Element,
    #[allow(dead_code)]
    bar: V::Element,
    state: Proxy<ProgressState>,
}

impl<V: View> Progress<V> {
    pub fn new(value: u8, flavor: Flavor) -> Self {
        let clamped = value.min(100);
        let mut state = Proxy::new(ProgressState {
            value: clamped,
            flavor,
            striped: false,
            animated: false,
        });

        rsx! {
            let wrapper = div(
                class = "progress",
                role = "progressbar",
                aria_valuenow = state(s => format!("{}", s.value)),
                aria_valuemin = "0",
                aria_valuemax = "100",
            ) {
                let bar = div(
                    class = state(s => {
                        let striped = if s.striped { " progress-bar-striped" } else { "" };
                        let animated = if s.animated { " progress-bar-animated" } else { "" };
                        format!("progress-bar bg-{}{striped}{animated}", s.flavor)
                    }),
                    style:width = state(s => format!("{}%", s.value)),
                ) {}
            }
        }

        Self {
            wrapper,
            bar,
            state,
        }
    }

    pub fn set_value(&mut self, value: u8) {
        self.state.modify(|s| s.value = value.min(100));
    }

    pub fn set_flavor(&mut self, flavor: Flavor) {
        self.state.modify(|s| s.flavor = flavor);
    }

    pub fn set_striped(&mut self, striped: bool) {
        self.state.modify(|s| s.striped = striped);
    }

    pub fn set_animated(&mut self, animated: bool) {
        self.state.modify(|s| {
            s.animated = animated;
            if animated {
                s.striped = true;
            }
        });
    }
}

#[cfg(feature = "library")]
pub mod library {
    use std::pin::Pin;

    use futures_lite::{FutureExt, Stream, StreamExt};
    use mogwai::future::MogwaiFutureExt;

    use super::*;

    #[derive(ViewChild)]
    pub struct ProgressLibraryItem<V: View> {
        #[child]
        pub wrapper: V::Element,
        progress: Progress<V>,
        value: u8,
        is_striped: bool,
        is_animated: bool,
        inc_click: V::EventListener,
        dec_click: V::EventListener,
        stripe_click: V::EventListener,
        animate_click: V::EventListener,
        timer: Pin<Box<dyn Stream<Item = ()>>>,
    }

    impl<V: View> Default for ProgressLibraryItem<V> {
        fn default() -> Self {
            let progress = Progress::new(25, super::Flavor::Primary);

            rsx! {
                let wrapper = div() {
                    div(class = "mb-3") {
                        {&progress}
                    }
                    div(class = "btn-group") {
                        button(
                            type = "button",
                            class = "btn btn-sm btn-outline-primary",
                            on:click = inc_click,
                        ) {
                            "+10"
                        }
                        button(
                            type = "button",
                            class = "btn btn-sm btn-outline-primary",
                            on:click = dec_click,
                        ) {
                            "-10"
                        }
                        button(
                            type = "button",
                            class = "btn btn-sm btn-outline-secondary",
                            on:click = stripe_click,
                        ) {
                            "Toggle striped"
                        }
                        button(
                            type = "button",
                            class = "btn btn-sm btn-outline-secondary",
                            on:click = animate_click,
                        ) {
                            "Toggle animated"
                        }
                    }
                }
            }

            let timer = Box::pin(futures_lite::stream::unfold((), |()| async {
                mogwai::time::wait_millis(500).await;
                Some(((), ()))
            }));

            Self {
                wrapper,
                progress,
                value: 25,
                is_striped: false,
                is_animated: false,
                inc_click,
                dec_click,
                stripe_click,
                animate_click,
                timer,
            }
        }
    }

    enum ProgressAction {
        Inc,
        Dec,
        Stripe,
        Animate,
        Tick,
    }

    impl<V: View> ProgressLibraryItem<V> {
        pub async fn step(&mut self) {
            let action = self
                .inc_click
                .next()
                .map(|_| ProgressAction::Inc)
                .or(self.dec_click.next().map(|_| ProgressAction::Dec))
                .or(self.stripe_click.next().map(|_| ProgressAction::Stripe))
                .or(self.animate_click.next().map(|_| ProgressAction::Animate))
                .or(self.timer.next().map(|_| ProgressAction::Tick))
                .await;

            match action {
                ProgressAction::Inc => {
                    self.value = self.value.saturating_add(10).min(100);
                    self.progress.set_value(self.value);
                }
                ProgressAction::Dec => {
                    self.value = self.value.saturating_sub(10);
                    self.progress.set_value(self.value);
                }
                ProgressAction::Stripe => {
                    self.is_striped = !self.is_striped;
                    self.progress.set_striped(self.is_striped);
                }
                ProgressAction::Animate => {
                    self.is_animated = !self.is_animated;
                    self.progress.set_animated(self.is_animated);
                }
                ProgressAction::Tick => {
                    // Auto-increment wrapping around
                    self.value = if self.value >= 100 { 0 } else { self.value + 1 };
                    self.progress.set_value(self.value);
                }
            }
        }
    }
}
