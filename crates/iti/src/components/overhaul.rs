//! Platinum Kit — design system sandbox for the Mac OS 9 Platinum overhaul.
//!
//! This module exists purely as a playground for experimenting with the
//! Platinum aesthetic: shadow utilities, font tiers, color palette, and
//! bevel effects. It is gated behind the `library` feature and appears
//! as "Platinum Kit" in the component gallery.

#[cfg(feature = "library")]
pub mod library {
    use mogwai::prelude::*;

    use crate::components::button::{Button, PrimaryButton};
    use crate::components::Flavor;

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
            // Create button demos
            let mut btn_normal = Button::new("Button", None);
            btn_normal.set_has_icon(false);

            let mut btn_disabled = Button::new("Disabled", None);
            btn_disabled.set_has_icon(false);
            btn_disabled.disable();

            let mut primary_normal = PrimaryButton::new("OK", None);
            primary_normal.set_has_icon(false);

            let mut primary_disabled = PrimaryButton::new("Disabled", None);
            primary_disabled.set_has_icon(false);
            primary_disabled.disable();

            let mut btn_success = Button::new("Success", Some(Flavor::Success));
            btn_success.set_has_icon(false);

            let mut btn_danger = Button::new("Danger", Some(Flavor::Danger));
            btn_danger.set_has_icon(false);

            let mut btn_warning = Button::new("Warning", Some(Flavor::Warning));
            btn_warning.set_has_icon(false);

            let mut btn_info = Button::new("Info", Some(Flavor::Info));
            btn_info.set_has_icon(false);

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

                    // ── Buttons section ──
                    div(style:margin_top = "2em") {
                        span(
                            class = "editorial",
                            style:font_size = "2em",
                            style:font_weight = "lighter",
                            style:color = crate::color::PURPLE,
                        ) {
                            "Buttons"
                        }
                    }
                    div(
                        class = "d-flex flex-wrap gap-4",
                        style:border = "2px dashed #7B61FF",
                        style:border_radius = "4px",
                        style:padding = "1em"
                    ) {
                        // Standard buttons
                        div() {
                            p() { strong() { "Standard" } }
                            div(class = "d-flex gap-2 flex-wrap align-items-center") {
                                {&btn_normal}
                                {&btn_disabled}
                            }
                        }

                        // Primary (ringed) buttons
                        div() {
                            p() { strong() { "Primary (Ringed)" } }
                            div(class = "d-flex gap-2 flex-wrap align-items-center") {
                                {&primary_normal}
                                {&primary_disabled}
                            }
                        }

                        // Flavor tints
                        div() {
                            p() { strong() { "Flavor Tints" } }
                            div(class = "d-flex gap-2 flex-wrap align-items-center") {
                                {&btn_success}
                                {&btn_danger}
                                {&btn_warning}
                                {&btn_info}
                            }
                        }

                        // Sizes
                        div() {
                            p() { strong() { "Sizes" } }
                            div(class = "d-flex gap-2 flex-wrap align-items-center") {
                                button(type = "button", class = "btn btn-sm") {
                                    "Small (.btn-sm)"
                                }
                                button(type = "button", class = "btn") {
                                    "Default"
                                }
                                button(type = "button", class = "btn btn-lg") {
                                    "Large (.btn-lg)"
                                }
                            }
                        }

                        // Sizes with primary ring
                        div() {
                            p() { strong() { "Primary Sizes" } }
                            div(class = "d-flex gap-2 flex-wrap align-items-center") {
                                span(class = "btn-primary-ring") {
                                    button(type = "button", class = "btn btn-sm") {
                                        "Small"
                                    }
                                }
                                span(class = "btn-primary-ring") {
                                    button(type = "button", class = "btn") {
                                        "Default"
                                    }
                                }
                                span(class = "btn-primary-ring") {
                                    button(type = "button", class = "btn btn-lg") {
                                        "Large"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            Self { wrapper }
        }
    }
}
