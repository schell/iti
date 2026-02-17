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

    use crate::components::{
        button::Button,
        button_group::{ButtonGroup, ButtonGroupEvent},
    };

    use super::*;

    #[derive(ViewChild)]
    pub struct ProgressLibraryItem<V: View> {
        #[child]
        pub wrapper: V::Element,
        progress: Progress<V>,
        control_group: ButtonGroup<V>,
        value: u8,
        is_striped: bool,
        is_animated: bool,
        timer: Pin<Box<dyn Stream<Item = ()>>>,
    }

    impl<V: View> Default for ProgressLibraryItem<V> {
        fn default() -> Self {
            let progress = Progress::new(25, super::Flavor::Primary);
            let mut control_group = ButtonGroup::<V>::default();
            control_group.extend([
                Button::new("+10", Some(Flavor::Primary)),
                Button::new("-10", Some(Flavor::Primary)),
                Button::new("Toggle striped", Some(Flavor::Secondary)),
                Button::new("Toggle animated", Some(Flavor::Secondary)),
            ]);
            for button in control_group.iter_mut() {
                button.set_has_icon(false);
            }

            rsx! {
                let wrapper = div() {
                    div(class = "mb-3") {
                        {&progress}
                    }
                    {&control_group}
                }
            }

            let timer = Box::pin(futures_lite::stream::unfold((), |()| async {
                mogwai::time::wait_millis(500).await;
                Some(((), ()))
            }));

            Self {
                wrapper,
                progress,
                control_group,
                value: 25,
                is_striped: false,
                is_animated: false,
                timer,
            }
        }
    }

    impl<V: View> ProgressLibraryItem<V> {
        pub async fn step(&mut self) {
            #[derive(Debug)]
            enum Action {
                Control(usize),
                Tick,
            }
            let control = async {
                let ButtonGroupEvent { index, event: _ } = self.control_group.step().await;
                Action::Control(index)
            };
            let tick = async {
                self.timer.next().await;
                Action::Tick
            };
            let event = control.or(tick).await;
            log::info!("event: {event:#?}");

            match event {
                Action::Control(0) => {
                    self.value = self.value.saturating_add(10).min(100);
                    self.progress.set_value(self.value);
                }
                Action::Control(1) => {
                    self.value = self.value.saturating_sub(10);
                    self.progress.set_value(self.value);
                }
                Action::Control(2) => {
                    self.is_striped = !self.is_striped;
                    self.progress.set_striped(self.is_striped);
                }
                Action::Control(3) => {
                    self.is_animated = !self.is_animated;
                    self.progress.set_animated(self.is_animated);
                }
                Action::Control(_) => unreachable!(),
                Action::Tick => {
                    // Auto-increment wrapping around
                    self.value = if self.value >= 100 { 0 } else { self.value + 1 };
                    self.progress.set_value(self.value);
                }
            }
        }
    }
}
