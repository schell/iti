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
                    div() {
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
                    div(class = "d-flex flex-wrap gap-4") {
                        // ── Window shadow + inner stroke ──
                        div(
                            class = "window-shadow inner-stroke",
                            style:background_color = "var(--iti-bg-medium)",
                            style:padding = "16px",
                            style:width = "260px"
                        ) {
                            p() {
                                strong() { ".window-shadow .inner-stroke" }
                            }
                            p() {
                                "A container with the Platinum window bevel \
                                 and inner stroke applied."
                            }
                        }

                        // ── Window shadow only ──
                        div(
                            class = "window-shadow",
                            style:background_color = "var(--iti-bg-medium)",
                            style:padding = "16px",
                            style:width = "260px"
                        ) {
                            p() {
                                strong() { ".window-shadow" }
                            }
                            p() {
                                "Bevel and drop shadow without the inner stroke."
                            }
                        }

                        // ── Inner stroke only ──
                        div(
                            class = "inner-stroke",
                            style:background_color = "var(--iti-bg-light)",
                            style:padding = "16px",
                            style:width = "260px"
                        ) {
                            p() {
                                strong() { ".inner-stroke" }
                            }
                            p() {
                                "Just the 1px inner outline, no shadow."
                            }
                        }

                        // ── Plain (no effects) ──
                        div(
                            style:background_color = "var(--iti-bg-light)",
                            style:padding = "16px",
                            style:width = "260px",
                            style:border = "1px solid var(--iti-border-dark)"
                        ) {
                            p() {
                                strong() { "Plain" }
                            }
                            p() {
                                "No shadow or stroke \u{2014} for comparison."
                            }
                        }
                    }
                }
            }

            Self { wrapper }
        }
    }
}
