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
