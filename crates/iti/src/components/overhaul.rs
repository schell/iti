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
    use crate::components::checkbox::Checkbox;
    use crate::components::icon::IconGlyph;
    use crate::components::radio::RadioGroup;
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

            // Icon + Text buttons
            let btn_add = Button::new("Add", None);
            // Plus icon is the default — no need to change glyph

            let mut btn_delete = Button::new("Delete", None);
            btn_delete.get_icon_mut().set_glyph(IconGlyph::Trash);

            let mut btn_edit = Button::new("Edit", None);
            btn_edit.get_icon_mut().set_glyph(IconGlyph::Pen);

            let mut btn_search = Button::new("Search", None);
            btn_search
                .get_icon_mut()
                .set_glyph(IconGlyph::MagnifyingGlass);

            // Icon-only buttons
            let mut icon_plus = Button::new("", None);
            icon_plus.get_icon_mut().set_additional_classes("");

            let mut icon_trash = Button::new("", None);
            icon_trash.get_icon_mut().set_glyph(IconGlyph::Trash);
            icon_trash.get_icon_mut().set_additional_classes("");

            let mut icon_edit = Button::new("", None);
            icon_edit.get_icon_mut().set_glyph(IconGlyph::Pen);
            icon_edit.get_icon_mut().set_additional_classes("");

            let mut icon_search = Button::new("", None);
            icon_search
                .get_icon_mut()
                .set_glyph(IconGlyph::MagnifyingGlass);
            icon_search.get_icon_mut().set_additional_classes("");

            // Checkbox demos
            let cb_default = Checkbox::new("Unchecked", false);
            let cb_checked = Checkbox::new("Checked", true);

            let cb_disabled = Checkbox::new("Disabled", false);
            cb_disabled.disable();

            let cb_disabled_checked = Checkbox::new("Disabled checked", true);
            cb_disabled_checked.disable();

            let mut cb_switch = Checkbox::new("Switch off", false);
            cb_switch.set_switch_style(true);

            let mut cb_switch_on = Checkbox::new("Switch on", true);
            cb_switch_on.set_switch_style(true);

            // Radio demo
            let mut radio_group = RadioGroup::new("platinum-demo");
            radio_group.push("Option A", "a");
            radio_group.push("Option B", "b");
            radio_group.push("Option C", "c");

            let mut radio_inline = RadioGroup::new("platinum-inline");
            radio_inline.push("Small", "sm");
            radio_inline.push("Medium", "md");
            radio_inline.push("Large", "lg");
            radio_inline.set_inline(true);

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
                        // Panel - bg-gray200, window shadow, inset stroke, padding
                        div(
                            class = "panel",
                            style:padding = "16px",
                            style:width = "260px",
                        ) {
                            p() {
                                strong() { ".panel" }
                            }
                            p() {
                                "Using the .panel class gets you all of the above"
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
                        class = "gap-4",
                        style:border = "2px dashed #7B61FF",
                        style:border_radius = "4px",
                        style:padding = "1em"
                    ) {
                        div(
                            class = "d-flex flex-wrap gap-4 panel",
                            style = ""
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

                            // Icon + Text
                            div() {
                                p() { strong() { "Icon + Text" } }
                                div(class = "d-flex gap-2 flex-wrap align-items-center") {
                                    {&btn_add}
                                    {&btn_delete}
                                    {&btn_edit}
                                    {&btn_search}
                                }
                            }

                            // Icon Only
                            div() {
                                p() { strong() { "Icon Only" } }
                                div(class = "d-flex gap-2 flex-wrap align-items-center") {
                                    {&icon_plus}
                                    {&icon_trash}
                                    {&icon_edit}
                                    {&icon_search}
                                }
                            }
                        }
                    }

                    // ── Checkboxes & Radios section ──
                    div(style:margin_top = "2em") {
                        span(
                            class = "editorial",
                            style:font_size = "2em",
                            style:font_weight = "lighter",
                            style:color = crate::color::PURPLE,
                        ) {
                            "Checkboxes & Radios"
                        }
                    }
                    div(
                        class = "gap-4",
                        style:border = "2px dashed #7B61FF",
                        style:border_radius = "4px",
                        style:padding = "1em"
                    ) {
                        div(
                            class = "d-flex flex-wrap gap-4 panel",
                        ) {
                            // Checkboxes
                            div() {
                                p() { strong() { "Checkboxes" } }
                                {&cb_default}
                                {&cb_checked}
                                {&cb_disabled}
                                {&cb_disabled_checked}
                            }

                            // Switches
                            div() {
                                p() { strong() { "Switches" } }
                                {&cb_switch}
                                {&cb_switch_on}
                            }

                            // Radio (vertical)
                            div() {
                                p() { strong() { "Radio Group" } }
                                {&radio_group}
                            }

                            // Radio (inline)
                            div() {
                                p() { strong() { "Radio Inline" } }
                                {&radio_inline}
                            }
                        }
                    }

                    // ── Progress Bars section ──
                    div(style:margin_top = "2em") {
                        span(
                            class = "editorial",
                            style:font_size = "2em",
                            style:font_weight = "lighter",
                            style:color = crate::color::PURPLE,
                        ) {
                            "Progress Bars"
                        }
                    }
                    div(
                        class = "gap-4",
                        style:border = "2px dashed #7B61FF",
                        style:border_radius = "4px",
                        style:padding = "1em"
                    ) {
                        div(class = "panel", style:padding = "1em") {
                            p() { strong() { "Empty (0%)" } }
                            div(class = "progress mb-3") {}

                            p() { strong() { "25%" } }
                            div(class = "progress mb-3") {
                                div(
                                    class = "progress-bar",
                                    style:width = "25%",
                                ) {}
                            }

                            p() { strong() { "50%" } }
                            div(class = "progress mb-3") {
                                div(
                                    class = "progress-bar",
                                    style:width = "50%",
                                ) {}
                            }

                            p() { strong() { "75%" } }
                            div(class = "progress mb-3") {
                                div(
                                    class = "progress-bar",
                                    style:width = "75%",
                                ) {}
                            }

                            p() { strong() { "100%" } }
                            div(class = "progress") {
                                div(
                                    class = "progress-bar",
                                    style:width = "100%",
                                ) {}
                            }
                        }
                    }
                }
            }

            Self { wrapper }
        }
    }
}
