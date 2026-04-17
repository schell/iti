//! Dithered drop-shadow wrapper.
//!
//! Wraps arbitrary content in a retro-style dithered drop shadow. The shadow
//! is rendered as a CSS `::after` pseudo-element with a repeating checkerboard
//! pattern, offset behind and to the bottom-right of the content.
//!
//! The shadow color, offset distance, and dither density are all configurable.
use mogwai::prelude::*;

/// Dither density — controls the repeating pattern tile size.
///
/// All levels produce a 50%-density checkerboard; larger tiles make
/// the individual dots more visible (coarser stipple).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Dither {
    /// 2×2 pixel checkerboard (fine stipple).
    #[default]
    Fine,
    /// 4×4 pixel checkerboard.
    Medium,
    /// 8×8 pixel checkerboard.
    Coarse,
}

impl Dither {
    fn class_suffix(&self) -> &'static str {
        match self {
            Dither::Fine => "",
            Dither::Medium => " iti-shadow--medium",
            Dither::Coarse => " iti-shadow--coarse",
        }
    }
}

/// A retro-style dithered drop shadow that wraps arbitrary content.
///
/// The shadow is a CSS `::after` pseudo-element using a repeating
/// checkerboard pattern. Configure with [`set_color`](Shadow::set_color),
/// [`set_offset`](Shadow::set_offset), and [`set_dither`](Shadow::set_dither).
#[derive(ViewChild, ViewProperties)]
pub struct Shadow<V: View> {
    #[child]
    #[properties]
    wrapper: V::Element,
    content_el: V::Element,
    content: ProxyChild<V>,
    dither: Proxy<Dither>,
}

impl<V: View> Shadow<V> {
    /// Create a new shadow wrapper with default settings.
    ///
    /// Defaults: black color, 4 px offset, fine (2×2) dither.
    pub fn new() -> Self {
        rsx! {
            let placeholder = span() {}
        }
        let content = ProxyChild::new(&placeholder);

        let mut dither = Proxy::new(Dither::Fine);

        rsx! {
            let wrapper = div(
                class = dither(d => format!("iti-shadow{}", d.class_suffix())),
            ) {
                let content_el = div(style:position = "relative", style:z_index = "1") {
                    {&content}
                }
            }
        }

        Self {
            wrapper,
            content_el,
            content,
            dither,
        }
    }

    /// Replace the wrapped content.
    pub fn set_content(&mut self, child: &impl ViewChild<V>) {
        self.content.replace(&self.content_el, child);
    }

    /// Set the dither dot color (any CSS color value).
    pub fn set_color(&self, color: &str) {
        self.wrapper.set_style("--iti-shadow-color", color);
    }

    /// Set the shadow offset in pixels.
    pub fn set_offset(&self, px: u32) {
        self.wrapper
            .set_style("--iti-shadow-offset", format!("{px}px"));
    }

    /// Set the dither density.
    pub fn set_dither(&mut self, d: Dither) {
        self.dither.set(d);
    }
}

impl<V: View> Default for Shadow<V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "library")]
pub mod library {
    use futures_lite::FutureExt;
    use mogwai::future::MogwaiFutureExt;

    use super::*;
    use crate::components::card::Card;

    const COLORS: [&str; 6] = [
        "black", "#333399", "#CC0000", "#006600", "#996633", "#666666",
    ];

    const DITHERS: [Dither; 3] = [Dither::Fine, Dither::Medium, Dither::Coarse];
    const DITHER_LABELS: [&str; 3] = ["Fine (2px)", "Medium (4px)", "Coarse (8px)"];

    #[derive(ViewChild)]
    pub struct ShadowLibraryItem<V: View> {
        #[child]
        pub wrapper: V::Element,
        shadows: Vec<Shadow<V>>,
        cycle_color_click: V::EventListener,
        cycle_dither_click: V::EventListener,
        inc_offset_click: V::EventListener,
        dec_offset_click: V::EventListener,
        color_label: V::Text,
        dither_label: V::Text,
        offset_label: V::Text,
        color_index: usize,
        dither_index: usize,
        offset: u32,
    }

    impl<V: View> Default for ShadowLibraryItem<V> {
        fn default() -> Self {
            // Build three demo shadows with different content
            let mut shadow1 = Shadow::new();
            let mut card = Card::new();
            rsx! {
                let header = strong() { "Shadowed Card" }
            }
            card.set_header(&header);
            rsx! {
                let body = p(class = "card-text") {
                    "A card wrapped in a dithered drop shadow."
                }
            }
            card.set_body(&body);
            card.hide_footer();
            shadow1.set_content(&card);

            let mut shadow2 = Shadow::new();
            shadow2.set_color("#333399");
            shadow2.set_dither(Dither::Medium);
            rsx! {
                let box2 = div(
                    class = "p-3 bg-white border",
                    style:width = "200px",
                ) {
                    "Medium dither, blue shadow"
                }
            }
            shadow2.set_content(&box2);

            let mut shadow3 = Shadow::new();
            shadow3.set_color("#006600");
            shadow3.set_dither(Dither::Coarse);
            shadow3.set_offset(8);
            rsx! {
                let box3 = div(
                    class = "p-3 bg-white border",
                    style:width = "200px",
                ) {
                    "Coarse dither, green, 8px offset"
                }
            }
            shadow3.set_content(&box3);

            let shadows = vec![shadow1, shadow2, shadow3];

            rsx! {
                let wrapper = div() {
                    div(class = "d-flex gap-4 flex-wrap mb-4") {
                        {&shadows}
                    }
                    div(class = "d-flex align-items-center gap-3 flex-wrap") {
                        div(class = "btn-group") {
                            button(
                                type = "button",
                                class = "btn btn-sm btn-outline-secondary",
                                on:click = cycle_color_click,
                            ) {
                                "Cycle color"
                            }
                            button(
                                type = "button",
                                class = "btn btn-sm btn-outline-secondary",
                                on:click = cycle_dither_click,
                            ) {
                                "Cycle dither"
                            }
                            button(
                                type = "button",
                                class = "btn btn-sm btn-outline-secondary",
                                on:click = dec_offset_click,
                            ) {
                                "Offset \u{2212}"
                            }
                            button(
                                type = "button",
                                class = "btn btn-sm btn-outline-secondary",
                                on:click = inc_offset_click,
                            ) {
                                "Offset +"
                            }
                        }
                        small(class = "text-muted") {
                            "Color: "
                            let color_label = "black"
                            " | Dither: "
                            let dither_label = "Fine (2px)"
                            " | Offset: "
                            let offset_label = "4px"
                        }
                    }
                }
            }

            Self {
                wrapper,
                shadows,
                cycle_color_click,
                cycle_dither_click,
                inc_offset_click,
                dec_offset_click,
                color_label,
                dither_label,
                offset_label,
                color_index: 0,
                dither_index: 0,
                offset: 4,
            }
        }
    }

    impl<V: View> ShadowLibraryItem<V> {
        pub async fn step(&mut self) {
            enum Action {
                CycleColor,
                CycleDither,
                IncOffset,
                DecOffset,
            }

            let ev = self
                .cycle_color_click
                .next()
                .map(|_| Action::CycleColor)
                .or(self.cycle_dither_click.next().map(|_| Action::CycleDither))
                .or(self.inc_offset_click.next().map(|_| Action::IncOffset))
                .or(self.dec_offset_click.next().map(|_| Action::DecOffset))
                .await;

            match ev {
                Action::CycleColor => {
                    self.color_index = (self.color_index + 1) % COLORS.len();
                    let color = COLORS[self.color_index];
                    for s in &self.shadows {
                        s.set_color(color);
                    }
                    self.color_label.set_text(color);
                }
                Action::CycleDither => {
                    self.dither_index = (self.dither_index + 1) % DITHERS.len();
                    let dither = DITHERS[self.dither_index];
                    for s in &mut self.shadows {
                        s.set_dither(dither);
                    }
                    self.dither_label.set_text(DITHER_LABELS[self.dither_index]);
                }
                Action::IncOffset => {
                    self.offset = self.offset.saturating_add(2).min(24);
                    for s in &self.shadows {
                        s.set_offset(self.offset);
                    }
                    self.offset_label.set_text(format!("{}px", self.offset));
                }
                Action::DecOffset => {
                    self.offset = self.offset.saturating_sub(2);
                    for s in &self.shadows {
                        s.set_offset(self.offset);
                    }
                    self.offset_label.set_text(format!("{}px", self.offset));
                }
            }
        }
    }
}
