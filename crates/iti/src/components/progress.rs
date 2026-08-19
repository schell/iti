//! Progress bar component.
//!
//! A Bootstrap progress bar with reactive value, flavor, and optional
//! striped/animated styles.
use mogwai::prelude::*;

struct ProgressState {
    value: u8,
    striped: bool,
    animated: bool,
}

/// A Bootstrap progress bar.
///
/// The value ranges from 0 to 100. Setting a value outside this range clamps
/// it to the nearest bound.
#[derive(ViewChild, ViewProperties)]
pub struct Progress<V: View> {
    #[child]
    #[properties]
    wrapper: V::Element,
    #[allow(dead_code)]
    bar: V::Element,
    state: Proxy<ProgressState>,
}

impl<V: View> Progress<V> {
    pub fn new(value: u8) -> Self {
        let clamped = value.min(100);
        let mut state = Proxy::new(ProgressState {
            value: clamped,
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
                    class = "progress-bar",
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

    pub fn get_value(&self) -> u8 {
        self.state.value
    }

    pub fn set_value(&mut self, value: u8) {
        self.state.modify(|s| s.value = value.min(100));
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
