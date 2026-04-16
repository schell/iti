//! Platinum Kit — design system sandbox for the Mac OS 9 Platinum overhaul.
//!
//! This module exists purely as a playground for experimenting with the
//! Platinum aesthetic: shadow utilities, font tiers, color palette, and
//! bevel effects. It is gated behind the `library` feature and appears
//! as "Platinum Kit" in the component gallery.

#[cfg(feature = "library")]
pub mod library {
    use mogwai::prelude::*;

    /// Sandbox library item for the Platinum design system overhaul.
    ///
    /// Purely presentational — no interactive events. Add new demo
    /// sections here as the design system evolves.
    #[derive(ViewChild)]
    pub struct OverhaulLibraryItem<V: View> {
        #[child]
        pub wrapper: V::Element,
    }

    impl<V: View> Default for OverhaulLibraryItem<V> {
        fn default() -> Self {
            rsx! {
                let wrapper = div() {
                    div(
                        style:margin_bottom = "3em",
                    ) {
                        h1(
                            class = "editorial",
                            style:font_size = "4em",
                            style:font_weight = "lighter",
                            style:margin_bottom = "0",
                        ) { "Platinum Kit" }
                        p(class = "text-muted") {
                            "Design system sandbox \u{2014} experiment with shadows, \
                             fonts, and colors here."
                        }
                    }

                    span(
                        class = "editorial",
                        style:font_size = "2em",
                        style:font_weight = "lighter",
                        style:color = crate::color::PURPLE,
                    ) {
                        "Panels and colors"
                    }
                    div(
                        class = "d-flex flex-wrap gap-4",
                        style:border = "2px dashed #7B61FF",
                        style:border_radius = "4px",
                        style:padding = "1em"
                    ) {
                        // ── Window shadow + inner stroke ──
                        div(
                            class = "window-shadow inner-stroke bg-gray200",
                            style:padding = "16px",
                            style:width = "260px"
                        ) {
                            p() {
                                strong() { ".window-shadow .inner-stroke .bg-gray200" }
                            }
                            p() {
                                "A container with the Platinum window bevel \
                                 and inner stroke applied. Gray 200 background."
                            }
                        }

                        // ── Window shadow only ──
                        div(
                            class = "window-shadow bg-gray400",
                            style:padding = "16px",
                            style:width = "260px"
                        ) {
                            p() {
                                strong() { ".window-shadow .bg-gray400" }
                            }
                            p() {
                                "Bevel and drop shadow without the inner stroke. Gray 400 background."
                            }
                        }

                        // ── Inner stroke only ──
                        div(
                            class = "inner-stroke bg-gray200",
                            style:padding = "16px",
                            style:width = "260px"
                        ) {
                            p() {
                                strong() { ".inner-stroke .bg-gray200" }
                            }
                            p() {
                                "Just the 1px inner outline, no shadow. Gray 200 background."
                            }
                        }

                        // ── Plain (no effects) ──
                        div(
                            class = "bg-gray200",
                            style:padding = "16px",
                            style:width = "260px",
                        ) {
                            p() {
                                strong() { ".bg-gray200" }
                            }
                            p() {
                                "No shadow or stroke - for comparison. Gray 200 background."
                            }
                        }
                    }
                }
            }

            Self { wrapper }
        }
    }
}
