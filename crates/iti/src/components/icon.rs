//! Font Awesome icon component.
use mogwai::prelude::*;

/// Font Awesome icon size classes.
#[derive(Clone, Copy, Default)]
pub enum IconSize {
    Xxs,
    Xs,
    Sm,
    #[default]
    Regular,
    Large,
    Xl,
    Xxl,
}

impl IconSize {
    pub fn as_str(&self) -> &str {
        match self {
            IconSize::Xxs => "fa-2xs",
            IconSize::Xs => "fa-xs",
            IconSize::Sm => "fa-sm",
            IconSize::Regular => "",
            IconSize::Large => "fa-lg",
            IconSize::Xl => "fa-xl",
            IconSize::Xxl => "fa-2xl",
        }
    }
}

/// Font Awesome icon glyph identifiers.
#[derive(Clone, Copy)]
pub enum IconGlyph {
    CirclePlus,
    CircleXmark,
    Plus,
    Xmark,
    Other(&'static str),
}

impl IconGlyph {
    pub fn as_str(&self) -> &str {
        match self {
            IconGlyph::CircleXmark => "fa-circle-xmark",
            IconGlyph::CirclePlus => "fa-circle-plus",
            IconGlyph::Plus => "fa-plus",
            IconGlyph::Xmark => "fa-xmark",
            IconGlyph::Other(s) => s,
        }
    }
}

struct IconState {
    glyph: IconGlyph,
    size: IconSize,
    additional_classes: String,
}

/// A Font Awesome icon element.
///
/// Supports setting the glyph, size, additional CSS classes, and visibility.
#[derive(ViewChild)]
pub struct Icon<V: View> {
    #[child]
    i: V::Element,
    state: Proxy<IconState>,
}

impl<V: View> Icon<V> {
    pub fn new(glyph: IconGlyph, size: IconSize) -> Self {
        let mut state = Proxy::new(IconState {
            glyph,
            size,
            additional_classes: Default::default(),
        });

        rsx! {
            let i = i(
                class = state(
                    s => format!(
                        "fa-solid {} {} {}",
                        s.glyph.as_str(),
                        s.size.as_str(),
                        s.additional_classes.as_str()
                    )
                ),
            ) {}
        }

        Self { i, state }
    }

    pub fn set_glyph(&mut self, glyph: IconGlyph) {
        self.state.modify(|s| s.glyph = glyph);
    }

    pub fn set_size(&mut self, size: IconSize) {
        self.state.modify(|s| s.size = size);
    }

    pub fn set_additional_classes(&mut self, classes: impl AsRef<str>) {
        self.state
            .modify(|s| s.additional_classes = classes.as_ref().to_string());
    }

    pub fn set_is_visible(&self, is_visible: bool) {
        if is_visible {
            self.i.remove_style("display");
        } else {
            self.i.set_style("display", "none");
        }
    }
}

#[cfg(feature = "library")]
pub mod library {
    use mogwai::prelude::*;

    use super::*;

    #[derive(ViewChild)]
    pub struct IconLibraryItem<V: View> {
        #[child]
        wrapper: V::Element,
    }

    impl<V: View> Default for IconLibraryItem<V> {
        fn default() -> Self {
            let glyphs = [
                IconGlyph::CirclePlus,
                IconGlyph::CircleXmark,
                IconGlyph::Plus,
                IconGlyph::Xmark,
            ]
            .map(|glyph| {
                rsx! {
                    let i = div(class = "col") {
                        {Icon::new(glyph, IconSize::Large)}
                    }
                }
                i
            })
            .to_vec();

            rsx! {
                let wrapper = div(class = "row") {
                    {glyphs}
                }
            }

            IconLibraryItem { wrapper }
        }
    }
}
