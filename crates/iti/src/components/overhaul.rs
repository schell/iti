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
    use crate::components::slider::SliderWithTicks;
    use crate::components::Flavor;

    // ── Section separator ───────────────────────────────────────────

    /// A dashed purple section in the Platinum Kit sandbox.
    ///
    /// Provides a titled container with a purple dashed border. Build
    /// the section content inside the `rsx!` block using the `content`
    /// element as the parent.
    #[derive(ViewChild)]
    struct Section<V: View> {
        #[child]
        wrapper: V::Element,
    }

    // ── Color swatch helper ─────────────────────────────────────────

    /// Build a single 48×48 color swatch with a label below.
    fn swatch<V: View>(bg_class: &str, label: &str) -> V::Element {
        rsx! {
            let el = div(style:text_align = "center") {
                div(
                    class = bg_class,
                    style:width = "48px",
                    style:height = "48px",
                    style:border = "1px solid var(--charcoal)",
                ) {}
                small() { {V::Text::new(label)} }
            }
        }
        el
    }

    // ── Section builders ────────────────────────────────────────────

    /// Build the "Platinum Kit" header with title and subtitle.
    fn build_header<V: View>() -> V::Element {
        rsx! {
            let header = div(style:margin_bottom = "3em") {
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
        }
        header
    }

    /// Build the "Panels and Colors" section with shadow demos and palette swatches.
    fn build_panels_and_colors<V: View>() -> Section<V> {
        rsx! {
            let wrapper = div(style:margin_top = "2em") {
                span(
                    class = "editorial",
                    style:font_size = "2em",
                    style:font_weight = "lighter",
                    style:color = crate::color::PURPLE,
                ) { "Panels and colors" }
                div(
                    class = "d-flex flex-wrap gap-4",
                    style:border = "2px dashed #7B61FF",
                    style:border_radius = "4px",
                    style:padding = "1em",
                ) {
                    div(
                        class = "window-shadow inner-stroke bg-gray200",
                        style:padding = "16px",
                        style:width = "260px"
                    ) {
                        p() { strong() { ".window-shadow .inner-stroke .bg-gray200" } }
                        p() { "A container with the Platinum window bevel \
                               and inner stroke applied. Gray 200 background." }
                    }
                    div(
                        class = "window-shadow bg-gray400",
                        style:padding = "16px",
                        style:width = "260px"
                    ) {
                        p() { strong() { ".window-shadow .bg-gray400" } }
                        p() { "Bevel and drop shadow without the inner stroke. Gray 400 background." }
                    }
                    div(
                        class = "inner-stroke bg-gray200",
                        style:padding = "16px",
                        style:width = "260px"
                    ) {
                        p() { strong() { ".inner-stroke .bg-gray200" } }
                        p() { "Just the 1px inner outline, no shadow. Gray 200 background." }
                    }
                    div(
                        class = "bg-gray200",
                        style:padding = "16px",
                        style:width = "260px",
                    ) {
                        p() { strong() { ".bg-gray200" } }
                        p() { "No shadow or stroke - for comparison. Gray 200 background." }
                    }
                    div(
                        class = "panel",
                        style:padding = "16px",
                        style:width = "260px",
                    ) {
                        p() { strong() { ".panel" } }
                        p() { "Using the .panel class gets you all of the above" }
                    }
                    div() {
                        div(style:margin_top = "1em") {
                            p() { strong() { "Color Palette" } }
                            div(class = "d-flex flex-wrap gap-2") {
                                {swatch::<V>("bg-black900", "black900")}
                                {swatch::<V>("bg-gray800", "gray800")}
                                {swatch::<V>("bg-gray700", "gray700")}
                                {swatch::<V>("bg-gray600", "gray600")}
                                {swatch::<V>("bg-gray500", "gray500")}
                                {swatch::<V>("bg-gray400", "gray400")}
                                {swatch::<V>("bg-gray300", "gray300")}
                                {swatch::<V>("bg-gray200", "gray200")}
                                {swatch::<V>("bg-white100", "white100")}
                            }
                            div(class = "d-flex flex-wrap gap-2", style:margin_top = "0.5em") {
                                {swatch::<V>("bg-azul", "azul")}
                                {swatch::<V>("bg-lavender", "lavender")}
                                {swatch::<V>("bg-thistle", "thistle")}
                                {swatch::<V>("bg-ice", "ice")}
                                {swatch::<V>("bg-cream", "cream")}
                                {swatch::<V>("bg-success", "success")}
                                {swatch::<V>("bg-danger", "danger")}
                                {swatch::<V>("bg-warning", "warning")}
                                {swatch::<V>("bg-charcoal", "charcoal")}
                            }
                        }
                    }
                }
            }
        }
        Section { wrapper }
    }

    /// Build the "Buttons" section with all button variants.
    fn build_buttons<V: View>() -> Section<V> {
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

        let btn_add = Button::new("Add", None);

        let mut btn_delete = Button::new("Delete", None);
        btn_delete.get_icon_mut().set_glyph(IconGlyph::Trash);

        let mut btn_edit = Button::new("Edit", None);
        btn_edit.get_icon_mut().set_glyph(IconGlyph::Pen);

        let mut btn_search = Button::new("Search", None);
        btn_search
            .get_icon_mut()
            .set_glyph(IconGlyph::MagnifyingGlass);

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

        rsx! {
            let wrapper = div(style:margin_top = "2em") {
                span(
                    class = "editorial",
                    style:font_size = "2em",
                    style:font_weight = "lighter",
                    style:color = crate::color::PURPLE,
                ) { "Buttons" }
                div(
                    style:border = "2px dashed #7B61FF",
                    style:border_radius = "4px",
                    style:padding = "1em",
                ) {
                    div(class = "d-flex flex-wrap gap-4 panel") {
                        div() {
                            p() { strong() { "Standard" } }
                            div(class = "d-flex gap-2 flex-wrap align-items-center") {
                                {&btn_normal}
                                {&btn_disabled}
                            }
                        }
                        div() {
                            p() { strong() { "Primary (Ringed)" } }
                            div(class = "d-flex gap-2 flex-wrap align-items-center") {
                                {&primary_normal}
                                {&primary_disabled}
                            }
                        }
                        div() {
                            p() { strong() { "Flavor Tints" } }
                            div(class = "d-flex gap-2 flex-wrap align-items-center") {
                                {&btn_success}
                                {&btn_danger}
                                {&btn_warning}
                                {&btn_info}
                            }
                        }
                        div() {
                            p() { strong() { "Sizes" } }
                            div(class = "d-flex gap-2 flex-wrap align-items-center") {
                                button(type = "button", class = "btn btn-sm") { "Small (.btn-sm)" }
                                button(type = "button", class = "btn") { "Default" }
                                button(type = "button", class = "btn btn-lg") { "Large (.btn-lg)" }
                            }
                        }
                        div() {
                            p() { strong() { "Primary Sizes" } }
                            div(class = "d-flex gap-2 flex-wrap align-items-center") {
                                span(class = "btn-primary-ring") {
                                    button(type = "button", class = "btn btn-sm") { "Small" }
                                }
                                span(class = "btn-primary-ring") {
                                    button(type = "button", class = "btn") { "Default" }
                                }
                                span(class = "btn-primary-ring") {
                                    button(type = "button", class = "btn btn-lg") { "Large" }
                                }
                            }
                        }
                        div() {
                            p() { strong() { "Icon + Text" } }
                            div(class = "d-flex gap-2 flex-wrap align-items-center") {
                                {&btn_add}
                                {&btn_delete}
                                {&btn_edit}
                                {&btn_search}
                            }
                        }
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
            }
        }

        Section { wrapper }
    }

    /// Build the "Checkboxes & Radios" section.
    fn build_checkboxes_and_radios<V: View>() -> Section<V> {
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
            let wrapper = div(style:margin_top = "2em") {
                span(
                    class = "editorial",
                    style:font_size = "2em",
                    style:font_weight = "lighter",
                    style:color = crate::color::PURPLE,
                ) { "Checkboxes & Radios" }
                div(
                    style:border = "2px dashed #7B61FF",
                    style:border_radius = "4px",
                    style:padding = "1em",
                ) {
                    div(class = "d-flex flex-wrap gap-4 panel") {
                        div() {
                            p() { strong() { "Checkboxes" } }
                            {&cb_default}
                            {&cb_checked}
                            {&cb_disabled}
                            {&cb_disabled_checked}
                        }
                        div() {
                            p() { strong() { "Switches" } }
                            {&cb_switch}
                            {&cb_switch_on}
                        }
                        div() {
                            p() { strong() { "Radio Group" } }
                            {&radio_group}
                        }
                        div() {
                            p() { strong() { "Radio Inline" } }
                            {&radio_inline}
                        }
                    }
                }
            }
        }

        Section { wrapper }
    }

    /// Build the "Progress Bars" section.
    fn build_progress_bars<V: View>() -> Section<V> {
        rsx! {
            let wrapper = div(style:margin_top = "2em") {
                span(
                    class = "editorial",
                    style:font_size = "2em",
                    style:font_weight = "lighter",
                    style:color = crate::color::PURPLE,
                ) { "Progress Bars" }
                div(
                    style:border = "2px dashed #7B61FF",
                    style:border_radius = "4px",
                    style:padding = "1em",
                ) {
                    div(class = "panel", style:padding = "1em") {
                        p() { strong() { "Empty (0%)" } }
                        div(class = "progress mb-3") {}

                        p() { strong() { "25%" } }
                        div(class = "progress mb-3") {
                            div(class = "progress-bar", style:width = "25%") {}
                        }

                        p() { strong() { "50%" } }
                        div(class = "progress mb-3") {
                            div(class = "progress-bar", style:width = "50%") {}
                        }

                        p() { strong() { "75%" } }
                        div(class = "progress mb-3") {
                            div(class = "progress-bar", style:width = "75%") {}
                        }

                        p() { strong() { "100%" } }
                        div(class = "progress") {
                            div(class = "progress-bar", style:width = "100%") {}
                        }
                    }
                }
            }
        }

        Section { wrapper }
    }

    /// Build the "Sliders" section.
    fn build_sliders<V: View>() -> Section<V> {
        let ticked_slider = SliderWithTicks::new(
            0.0,
            6.0,
            1.0,
            3.0,
            &["01", "02", "03", "04", "05", "06", "07"],
        );

        let unlabeled_ticks = SliderWithTicks::with_tick_count(0.0, 100.0, 10.0, 50.0, 11);

        rsx! {
            let wrapper = div(style:margin_top = "2em") {
                span(
                    class = "editorial",
                    style:font_size = "2em",
                    style:font_weight = "lighter",
                    style:color = crate::color::PURPLE,
                ) { "Sliders" }
                div(
                    style:border = "2px dashed #7B61FF",
                    style:border_radius = "4px",
                    style:padding = "1em",
                ) {
                    div(class = "panel", style:padding = "1em") {
                        p() { strong() { "Default" } }
                        input(
                            type = "range",
                            class = "iti-slider mb-3",
                            min = "0", max = "100", value = "50",
                        ) {}

                        p() { strong() { "At minimum" } }
                        input(
                            type = "range",
                            class = "iti-slider mb-3",
                            min = "0", max = "100", value = "0",
                        ) {}

                        p() { strong() { "At maximum" } }
                        input(
                            type = "range",
                            class = "iti-slider mb-3",
                            min = "0", max = "100", value = "100",
                        ) {}

                        p() { strong() { "Disabled" } }
                        input(
                            type = "range",
                            class = "iti-slider mb-3",
                            min = "0", max = "100", value = "30",
                            disabled = "",
                        ) {}

                        p() { strong() { "With Labeled Ticks" } }
                        div(class = "mb-3") {
                            {&ticked_slider}
                        }

                        p() { strong() { "With Unlabeled Ticks" } }
                        {&unlabeled_ticks}
                    }
                }
            }
        }

        Section { wrapper }
    }

    // ── Main component ──────────────────────────────────────────────

    /// Sandbox library item for the Platinum design system overhaul.
    ///
    /// Purely presentational — no interactive events. Each section is
    /// built by a dedicated helper function and collected here.
    #[derive(ViewChild)]
    pub struct OverhaulLibraryItem<V: View> {
        #[child]
        pub wrapper: V::Element,
    }

    impl<V: View> Default for OverhaulLibraryItem<V> {
        fn default() -> Self {
            let header = build_header::<V>();
            let panels = build_panels_and_colors::<V>();
            let buttons = build_buttons::<V>();
            let checkboxes = build_checkboxes_and_radios::<V>();
            let progress = build_progress_bars::<V>();
            let sliders = build_sliders::<V>();

            rsx! {
                let wrapper = div() {
                    {header}
                    {&panels}
                    {&buttons}
                    {&checkboxes}
                    {&progress}
                    {&sliders}
                }
            }

            Self { wrapper }
        }
    }
}
