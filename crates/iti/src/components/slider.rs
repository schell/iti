//! Range slider component.
//!
//! Wraps a native `<input type="range">` with reactive min/max/step/value
//! configuration and a pull-based async event model.
use mogwai::prelude::*;
use mogwai::web::WebElement;

/// Event produced when the user moves the slider.
pub struct SliderEvent<V: View> {
    /// The current slider value.
    pub value: f64,
    /// The raw DOM event.
    pub event: V::Event,
}

/// A range slider (`<input type="range">`).
///
/// Provides a configurable numeric slider with `f64` values and an async
/// [`step`](Slider::step) method that yields [`SliderEvent`]s on user input.
#[derive(ViewChild, ViewProperties)]
pub struct Slider<V: View> {
    #[child]
    #[properties]
    input: V::Element,
    on_input: V::EventListener,
    value: f64,
}

impl<V: View> Slider<V> {
    /// Create a new slider with the given range and initial value.
    ///
    /// # Arguments
    ///
    /// * `min` — minimum value
    /// * `max` — maximum value
    /// * `step` — step increment (use `"any"` semantics by passing a small
    ///   value like `0.01` for near-continuous sliding)
    /// * `value` — initial value (clamped to `[min, max]`)
    pub fn new(min: f64, max: f64, step: f64, value: f64) -> Self {
        let value = value.clamp(min, max);
        let min_s = format_f64(min);
        let max_s = format_f64(max);
        let step_s = format_f64(step);
        let value_s = format_f64(value);

        rsx! {
            let input = input(
                type = "range",
                class = "iti-slider",
                min = min_s,
                max = max_s,
                step = step_s,
                value = value_s,
                on:input = on_input,
            ) {}
        }

        Self {
            input,
            on_input,
            value,
        }
    }

    /// Read the current value.
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Programmatically set the slider value.
    ///
    /// The value is clamped to the current `[min, max]` range.
    pub fn set_value(&mut self, value: f64) {
        self.value = value;
        self.input.set_property("value", format_f64(value));
    }

    /// Set the minimum value.
    pub fn set_min(&self, min: f64) {
        self.input.set_property("min", format_f64(min));
    }

    /// Set the maximum value.
    pub fn set_max(&self, max: f64) {
        self.input.set_property("max", format_f64(max));
    }

    /// Set the step increment.
    pub fn set_step(&self, step: f64) {
        self.input.set_property("step", format_f64(step));
    }

    /// Disable the slider.
    pub fn disable(&self) {
        self.input.set_property("disabled", "");
    }

    /// Enable the slider.
    pub fn enable(&self) {
        self.input.remove_property("disabled");
    }

    /// Await the next user input and return a [`SliderEvent`] with the new
    /// value.
    ///
    /// The internal `value` field is updated before returning.
    pub async fn step(&mut self) -> SliderEvent<V> {
        let event = self.on_input.next().await;
        // Read the current value from the DOM input element.
        let dom_value = self
            .input
            .dyn_el(|el: &web_sys::HtmlInputElement| el.value());
        if let Some(s) = dom_value {
            if let Ok(v) = s.parse::<f64>() {
                self.value = v;
            }
        }
        SliderEvent {
            value: self.value,
            event,
        }
    }
}

impl<V: View> Default for Slider<V> {
    fn default() -> Self {
        Self::new(0.0, 100.0, 1.0, 50.0)
    }
}

/// A slider with evenly-spaced tick marks and optional labels below the track.
///
/// Wraps a [`Slider`] in a container with tick mark elements. The tick marks
/// are purely visual — they don't snap the slider to specific values.
///
/// # Example
///
/// ```ignore
/// let slider = SliderWithTicks::new(
///     0.0, 6.0, 1.0, 3.0,
///     &["01", "02", "03", "04", "05", "06", "07"],
/// );
/// ```
#[derive(ViewChild, ViewProperties)]
pub struct SliderWithTicks<V: View> {
    #[child]
    #[properties]
    wrapper: V::Element,
    slider: Slider<V>,
    #[allow(dead_code)]
    tick_container: V::Element,
}

impl<V: View> SliderWithTicks<V> {
    /// Create a slider with tick marks and optional labels.
    ///
    /// `tick_labels` is a slice of label strings. Each string becomes a
    /// tick mark positioned evenly along the track. Use empty strings
    /// for tick lines without labels.
    pub fn new(min: f64, max: f64, step: f64, value: f64, tick_labels: &[&str]) -> Self {
        let slider = Slider::new(min, max, step, value);
        rsx! {
            let wrapper = div(class = "iti-slider-ticks") {
                {&slider}
                let tick_container = div(class = "iti-slider-tick-marks") {}
            }
        }

        for label in tick_labels {
            rsx! {
                let tick = span(class = "iti-tick") {
                    {V::Text::new(*label)}
                }
            }
            tick_container.append_child(&tick);
        }

        Self {
            wrapper,
            slider,
            tick_container,
        }
    }

    /// Create a slider with a given number of unlabeled tick marks.
    pub fn with_tick_count(min: f64, max: f64, step: f64, value: f64, count: usize) -> Self {
        let labels: Vec<&str> = vec![""; count];
        Self::new(min, max, step, value, &labels)
    }

    /// Access the inner slider.
    pub fn slider(&self) -> &Slider<V> {
        &self.slider
    }

    /// Mutably access the inner slider.
    pub fn slider_mut(&mut self) -> &mut Slider<V> {
        &mut self.slider
    }

    /// Read the current value.
    pub fn value(&self) -> f64 {
        self.slider.value()
    }

    /// Programmatically set the slider value.
    pub fn set_value(&mut self, value: f64) {
        self.slider.set_value(value);
    }

    /// Set the minimum value.
    pub fn set_min(&self, min: f64) {
        self.slider.set_min(min);
    }

    /// Set the maximum value.
    pub fn set_max(&self, max: f64) {
        self.slider.set_max(max);
    }

    /// Set the step increment.
    pub fn set_step(&self, step: f64) {
        self.slider.set_step(step);
    }

    /// Disable the slider.
    pub fn disable(&self) {
        self.slider.disable();
    }

    /// Enable the slider.
    pub fn enable(&self) {
        self.slider.enable();
    }

    /// Await the next user input and return a [`SliderEvent`] with the new value.
    pub async fn step(&mut self) -> SliderEvent<V> {
        self.slider.step().await
    }
}

/// Format an f64 as a compact string, omitting trailing `.0` for integers.
fn format_f64(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

#[cfg(feature = "library")]
pub mod library {
    use futures_lite::FutureExt;
    use mogwai::future::MogwaiFutureExt;

    use super::*;

    #[derive(ViewChild)]
    pub struct SliderLibraryItem<V: View> {
        #[child]
        pub wrapper: V::Element,
        slider_a: Slider<V>,
        slider_b: Slider<V>,
        slider_c: Slider<V>,
        label_a: V::Text,
        label_b: V::Text,
        label_c: V::Text,
        reset_click: V::EventListener,
        toggle_click: V::EventListener,
        disabled: bool,
    }

    impl<V: View> Default for SliderLibraryItem<V> {
        fn default() -> Self {
            let slider_a = Slider::new(0.0, 100.0, 1.0, 50.0);
            let slider_b = Slider::new(0.0, 1.0, 0.01, 0.5);
            let slider_c = Slider::new(-50.0, 50.0, 5.0, 0.0);

            rsx! {
                let wrapper = div() {
                    div(class = "mb-3") {
                        div(class = "mb-2") {
                            label(class = "form-label") {
                                "Integer (0\u{2013}100, step 1): "
                                let label_a = "50"
                            }
                            {&slider_a}
                        }
                        div(class = "mb-2") {
                            label(class = "form-label") {
                                "Float (0.0\u{2013}1.0, step 0.01): "
                                let label_b = "0.5"
                            }
                            {&slider_b}
                        }
                        div(class = "mb-2") {
                            label(class = "form-label") {
                                "Signed (\u{2212}50\u{2013}50, step 5): "
                                let label_c = "0"
                            }
                            {&slider_c}
                        }
                    }
                    div(class = "btn-group") {
                        button(
                            type = "button",
                            class = "btn btn-sm btn-outline-secondary",
                            on:click = reset_click,
                        ) {
                            "Reset"
                        }
                        button(
                            type = "button",
                            class = "btn btn-sm btn-outline-secondary",
                            on:click = toggle_click,
                        ) {
                            "Toggle disabled"
                        }
                    }
                }
            }

            Self {
                wrapper,
                slider_a,
                slider_b,
                slider_c,
                label_a,
                label_b,
                label_c,
                reset_click,
                toggle_click,
                disabled: false,
            }
        }
    }

    impl<V: View> SliderLibraryItem<V> {
        fn format_value(v: f64, decimals: usize) -> String {
            format!("{v:.decimals$}")
        }

        pub async fn step(&mut self) {
            enum Action {
                SliderA(f64),
                SliderB(f64),
                SliderC(f64),
                Reset,
                Toggle,
            }

            let ev = self
                .slider_a
                .step()
                .map(|e| Action::SliderA(e.value))
                .or(self.slider_b.step().map(|e| Action::SliderB(e.value)))
                .or(self.slider_c.step().map(|e| Action::SliderC(e.value)))
                .or(self.reset_click.next().map(|_| Action::Reset))
                .or(self.toggle_click.next().map(|_| Action::Toggle))
                .await;

            match ev {
                Action::SliderA(v) => {
                    self.label_a.set_text(Self::format_value(v, 0));
                }
                Action::SliderB(v) => {
                    self.label_b.set_text(Self::format_value(v, 2));
                }
                Action::SliderC(v) => {
                    self.label_c.set_text(Self::format_value(v, 0));
                }
                Action::Reset => {
                    self.slider_a.set_value(50.0);
                    self.slider_b.set_value(0.5);
                    self.slider_c.set_value(0.0);
                    self.label_a.set_text("50");
                    self.label_b.set_text("0.50");
                    self.label_c.set_text("0");
                }
                Action::Toggle => {
                    self.disabled = !self.disabled;
                    if self.disabled {
                        self.slider_a.disable();
                        self.slider_b.disable();
                        self.slider_c.disable();
                    } else {
                        self.slider_a.enable();
                        self.slider_b.enable();
                        self.slider_c.enable();
                    }
                }
            }
        }
    }
}
