//! Platinum Kit — design system sandbox for the Mac OS 9 Platinum overhaul.
//!
//! This module exists purely as a playground for experimenting with the
//! Platinum aesthetic: shadow utilities, font tiers, color palette, and
//! bevel effects. It is gated behind the `library` feature and appears
//! as "Platinum Kit" in the component gallery.
use futures_lite::FutureExt;
use mogwai::future::MogwaiFutureExt;
use mogwai::prelude::*;
use mogwai::web::prelude::wasm_bindgen_futures;

use crate::components::alert::Alert;
use crate::components::badge::Badge;
use crate::components::button::{Button, PrimaryButton};
use crate::components::card::Card;
use crate::components::checkbox::Checkbox;
use crate::components::dropdown::{Dropdown, DropdownEvent};
use crate::components::icon::{Icon, IconGlyph, IconSize};
use crate::components::progress::Progress;
use crate::components::radio::RadioGroup;
use crate::components::select::Select;
use crate::components::shadow::{Dither, Shadow};
use crate::components::slider::SliderWithTicks;
use crate::components::title_bar::TitleBar;
use crate::components::Flavor;

// ── Section separator ───────────────────────────────────────────

/// A dashed purple section in the Platinum Kit sandbox.
///
/// Provides a titled container with a purple dashed border and an
/// editorial section heading. Use [`push`](Section::push) to add
/// content elements inside the dashed border area.
#[derive(ViewChild)]
struct Section<V: View> {
    #[child]
    wrapper: V::Element,
    content: V::Element,
}

impl<V: View> Section<V> {
    /// Create a new section with the given title.
    fn new(title: &str) -> Self {
        let mut enabled = Proxy::new(true);
        rsx! {
            let wrapper = div(class = "container", style:margin_top = "2em") {
                span(
                    class = "editorial row",
                    style:font_size = "2em",
                    style:font_weight = "lighter",
                    style:color = crate::color::PURPLE,
                    style:cursor = "pointer",
                    on:click = on_click
                ) {
                    let toggle = {{
                        let c = Checkbox::new("", *enabled);
                        c.set_style("float", "left");
                        c
                    }}
                    {V::Text::new(title)}
                }
                let content = div(
                    class = "row",
                    style:border = "2px dashed #7B61FF",
                    style:border_radius = "4px",
                    style:padding = "1em",
                    style:display = enabled(is_enabled => if *is_enabled {
                        "block"
                    } else {
                        "none"
                    })
                ) {}
            }
        }
        wasm_bindgen_futures::spawn_local(async move {
            let mut toggle = toggle;
            loop {
                let _ev = on_click.next().await;

                enabled.modify(|is_enabled| {
                    *is_enabled = !*is_enabled;
                });

                if toggle.is_checked() != *enabled {
                    toggle.set_checked(*enabled);
                }

                log::info!("enabled: {}", *enabled);
            }
        });
        Self { wrapper, content }
    }

    /// Append a child element to the section content area.
    fn push(&self, child: &impl ViewChild<V>) {
        self.content.append_child(child);
    }
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
    let section = Section::new("Panels and colors");

    rsx! {
        let panels = div(class = "d-flex flex-wrap gap-4") {
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
    section.push(&panels);
    section
}

/// Build the "Buttons" section with all button variants.
fn build_buttons<V: View>() -> Section<V> {
    let section = Section::new("Buttons");

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
    icon_plus.get_icon_mut().remove_class("me-1");

    let mut icon_trash = Button::new("", None);
    icon_trash.get_icon_mut().set_glyph(IconGlyph::Trash);
    icon_trash.get_icon_mut().remove_class("me-1");

    let mut icon_edit = Button::new("", None);
    icon_edit.get_icon_mut().set_glyph(IconGlyph::Pen);
    icon_edit.get_icon_mut().remove_class("me-1");

    let mut icon_search = Button::new("", None);
    icon_search
        .get_icon_mut()
        .set_glyph(IconGlyph::MagnifyingGlass);
    icon_search.get_icon_mut().remove_class("me-1");

    let icons_square: Vec<Button<V>> = IconGlyph::PEOPLE
        .into_iter()
        .map(|g| {
            let mut icon = Button::new("", None);
            icon.get_icon_mut().set_glyph(g);
            icon.add_class("btn-square");
            icon
        })
        .collect();

    rsx! {
        let content = div(class = "d-flex flex-wrap gap-4 panel") {
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
            div() {
                p() { strong() { "Square Icon Buttons"}}
                div(class = "d-flex gap-2 flex-wrap align-items-center") {
                    {&icons_square}
                }
            }
        }
    }
    section.push(&content);
    section
}

/// Build the "Checkboxes & Radios" section.
fn build_checkboxes_and_radios<V: View>() -> Section<V> {
    let section = Section::new("Checkboxes & Radios");

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
        let content = div(class = "d-flex flex-wrap gap-4 panel") {
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
    section.push(&content);
    section
}

/// Build the "Progress Bars" section.
fn build_progress_bars<V: View>() -> Section<V> {
    let section = Section::new("Progress Bars");

    rsx! {
        let content = div(class = "panel", style:padding = "1em") {
            p() {
                strong() {
                    let percent_text = "0%"
                }
            }

            div(class = "mb-3") {
                let progress = {Progress::new(0)}
            }

            let zero_button = {{
                let mut b = Button::new("Set to 0%", None);
                b.set_has_icon(false);
                b
            }}
        }
    }

    section.push(&content);
    wasm_bindgen_futures::spawn_local(async move {
        let mut progress = progress;
        let zero_button = zero_button;

        loop {
            let hit_zero = zero_button.step().map(Some);
            let tick = async {
                mogwai::time::wait_millis(200).await;
                None
            };
            match hit_zero.or(tick).await {
                Some(_ev) => {
                    progress.set_value(0);
                }
                None => {
                    let current = progress.get_value();
                    progress.set_value(current + 1);
                }
            }
            percent_text.set_text(format!("{}%", progress.get_value()));
        }
    });
    section
}

/// Build the "Sliders" section.
fn build_sliders<V: View>() -> Section<V> {
    let section = Section::new("Sliders");

    let ticked_slider = SliderWithTicks::new(
        0.0,
        6.0,
        1.0,
        3.0,
        &["01", "02", "03", "04", "05", "06", "07"],
    );

    let unlabeled_ticks = SliderWithTicks::with_tick_count(0.0, 100.0, 10.0, 50.0, 11);

    rsx! {
        let content = div(class = "panel", style:padding = "1em") {
            p() { strong() { "Default" } }
            input(
                type = "range",
                class = "iti-slider mb-3",
                min = "0", max = "100", value = "50",
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
    section.push(&content);
    section
}

/// Build the "Selects" section with native select dropdowns.
fn build_selects<V: View>() -> Section<V> {
    let section = Section::new("Selects");

    // Default select
    let mut select_default = Select::new(None);
    select_default.push("Apple", "apple");
    select_default.push("Banana", "banana");
    select_default.push("Cherry", "cherry");

    // Flavored select
    let mut select_primary = Select::new(Some(Flavor::Primary));
    select_primary.push("Option A", "a");
    select_primary.push("Option B", "b");
    select_primary.push("Option C", "c");

    // Disabled select
    let mut select_disabled = Select::new(None);
    select_disabled.push("Can't change", "disabled");
    select_disabled.disable();

    rsx! {
        let content = div(class = "d-flex flex-wrap gap-4 panel") {
            div() {
                p() { strong() { "Default" } }
                {&select_default}
            }
            div() {
                p() { strong() { "With Flavor" } }
                {&select_primary}
            }
            div() {
                p() { strong() { "Disabled" } }
                {&select_disabled}
            }
        }
    }
    section.push(&content);
    section
}

/// Build the "Dropdowns" section with button dropdown menus.
fn build_dropdowns<V: View>() -> Section<V> {
    let section = Section::new("Dropdowns");

    // Interactive dropdown
    let mut dropdown = Dropdown::new("Click me", Flavor::Primary);
    dropdown.push("Action");
    dropdown.push("Another action");
    dropdown.push("Something else");

    rsx! {
        let content = div(class = "d-flex flex-wrap gap-4 panel") {
            // Static open dropdown (visual demo only)
            div() {
                p() { strong() { "Open State (static)" } }
                div(class = "dropdown") {
                    button(type = "button", class = "btn dropdown-toggle") {
                        "Dropdown"
                    }
                    ul(
                        class = "dropdown-menu show",
                        style:position = "static",
                        style:display = "block",
                    ) {
                        li() { a(class = "dropdown-item") { "Action" } }
                        li() { a(class = "dropdown-item") { "Another action" } }
                        li() { a(class = "dropdown-item") { "Something else" } }
                    }
                }
            }
            // Interactive dropdown
            div() {
                p() { strong() { "Interactive" } }
                {&dropdown}
            }
        }
    }

    // Wire up dropdown event handling
    wasm_bindgen_futures::spawn_local(async move {
        let mut dropdown = dropdown;
        loop {
            match dropdown.step().await {
                None => dropdown.toggle(),
                Some(DropdownEvent::ItemClicked { .. }) => dropdown.hide(),
                Some(DropdownEvent::Dismissed) => dropdown.hide(),
            }
        }
    });

    section.push(&content);
    section
}

/// Build the "Text Inputs" section with input variants and textarea.
fn build_text_inputs<V: View>() -> Section<V> {
    let section = Section::new("Text Inputs");

    rsx! {
        let content = div(class = "d-flex flex-wrap gap-4 panel") {
            // Default and placeholder
            div() {
                p() { strong() { "Default" } }
                input(type = "text", class = "form-control", style:width = "200px") {}
            }
            div() {
                p() { strong() { "With Placeholder" } }
                input(
                    type = "text",
                    class = "form-control",
                    style:width = "200px",
                    placeholder = "Enter your name...",
                ) {}
            }

            // Label and help text
            div() {
                p() { strong() { "With Label" } }
                label(class = "form-label") { "Email Address" }
                input(
                    type = "email",
                    class = "form-control",
                    style:width = "200px",
                    placeholder = "user@example.com",
                ) {}
            }
            div() {
                p() { strong() { "With Help Text" } }
                label(class = "form-label") { "Username" }
                input(type = "text", class = "form-control", style:width = "200px") {}
                div(class = "form-text") { "Choose a unique username." }
            }

            // Disabled and read-only
            div() {
                p() { strong() { "Disabled" } }
                input(
                    type = "text",
                    class = "form-control",
                    style:width = "200px",
                    value = "Can't edit this",
                    disabled = "",
                ) {}
            }
            div() {
                p() { strong() { "Read-only" } }
                input(
                    type = "text",
                    class = "form-control",
                    style:width = "200px",
                    value = "Read-only value",
                    readonly = "",
                ) {}
            }

            // Sizes
            div() {
                p() { strong() { "Sizes" } }
                div(class = "d-flex gap-2 align-items-center flex-wrap") {
                    input(
                        type = "text",
                        class = "form-control form-control-sm",
                        style:width = "120px",
                        placeholder = "Small",
                    ) {}
                    input(
                        type = "text",
                        class = "form-control",
                        style:width = "150px",
                        placeholder = "Default",
                    ) {}
                    input(
                        type = "text",
                        class = "form-control form-control-lg",
                        style:width = "180px",
                        placeholder = "Large",
                    ) {}
                }
            }

            // Input types
            div() {
                p() { strong() { "Input Types" } }
                div(class = "d-flex flex-column gap-2") {
                    div() {
                        label(class = "form-label") { "Password" }
                        input(
                            type = "password",
                            class = "form-control",
                            style:width = "200px",
                            value = "secret123",
                        ) {}
                    }
                    div() {
                        label(class = "form-label") { "Number" }
                        input(
                            type = "number",
                            class = "form-control",
                            style:width = "200px",
                            value = "42",
                            min = "0",
                            max = "100",
                        ) {}
                    }
                    div() {
                        label(class = "form-label") { "Search" }
                        input(
                            type = "search",
                            class = "form-control",
                            style:width = "200px",
                            placeholder = "Search...",
                        ) {}
                    }
                }
            }

            // Textarea
            div() {
                p() { strong() { "Textarea" } }
                label(class = "form-label") { "Comments" }
                textarea(
                    class = "form-control",
                    style:width = "300px",
                    rows = "4",
                    placeholder = "Enter your comments here...",
                ) {}
            }
        }
    }
    section.push(&content);
    section
}

/// Build the "Alerts" section showing all flavor variants.
fn build_alerts<V: View>() -> Section<V> {
    let section = Section::new("Alerts");

    const FLAVORS: [Flavor; 8] = [
        Flavor::Primary,
        Flavor::Secondary,
        Flavor::Success,
        Flavor::Danger,
        Flavor::Warning,
        Flavor::Info,
        Flavor::Light,
        Flavor::Dark,
    ];

    let alert_items: Vec<V::Element> = FLAVORS
        .iter()
        .map(|&f| {
            let alert = Alert::new(format!("This is a {f} alert!"), f);
            rsx! {
                let item = div(class = "mb-2") {
                    {&alert}
                }
            }
            item
        })
        .collect();

    section.push(&alert_items);
    section
}

/// Build the "Badges" section showing all flavor variants plus pill style.
fn build_badges<V: View>() -> Section<V> {
    let section = Section::new("Badges");

    const FLAVORS: [Flavor; 8] = [
        Flavor::Primary,
        Flavor::Secondary,
        Flavor::Success,
        Flavor::Danger,
        Flavor::Warning,
        Flavor::Info,
        Flavor::Light,
        Flavor::Dark,
    ];

    let standard_badges: Vec<Badge<V>> = FLAVORS
        .iter()
        .map(|&f| Badge::new(format!("{f}"), f))
        .collect();

    let pill_badges: Vec<Badge<V>> = FLAVORS
        .iter()
        .map(|&f| {
            let mut badge = Badge::new(format!("{f}"), f);
            badge.set_pill(true);
            badge
        })
        .collect();

    rsx! {
        let content = div(class = "panel") {
            div(class = "mb-3") {
                p() { strong() { "Standard" } }
                div(class = "d-flex flex-wrap gap-2") {
                    {&standard_badges}
                }
            }
            div() {
                p() { strong() { "Pill" } }
                div(class = "d-flex flex-wrap gap-2") {
                    {&pill_badges}
                }
            }
        }
    }
    section.push(&content);
    section
}

/// Build the "Cards" section with a sample card.
fn build_cards<V: View>() -> Section<V> {
    let section = Section::new("Cards");

    let mut card = Card::new();
    card.set_header(&"Card Header".into_text::<V>());

    rsx! {
        let body_content = div() {
            h5(class = "card-title") { "Card Title" }
            p(class = "card-text") {
                "Some quick example text to build on the card title and \
                 make up the bulk of the card\u{2019}s content."
            }
        }
    }
    card.set_body(&body_content);

    rsx! {
        let footer_text = small(class = "text-body-secondary") {
            "Last updated 3 mins ago"
        }
    }
    card.set_footer(&footer_text);

    rsx! {
        let content = div(class = "panel", style:max_width = "24rem") {
            {&card}
        }
    }
    section.push(&content);
    section
}

/// Build the "Shadows" section showing the three dither levels.
fn build_shadows<V: View>() -> Section<V> {
    let section = Section::new("Shadows");

    // Fine dither (default)
    let mut shadow_fine = Shadow::new();
    rsx! {
        let box_fine = div(
            class = "p-3 bg-white border",
            style:width = "140px",
            style:text_align = "center",
        ) {
            strong() { "Fine" }
            br() {}
            small(class = "text-muted") { "2px dither" }
        }
    }
    shadow_fine.set_content(&box_fine);

    // Medium dither
    let mut shadow_medium = Shadow::new();
    shadow_medium.set_dither(Dither::Medium);
    shadow_medium.set_color("#333399");
    rsx! {
        let box_medium = div(
            class = "p-3 bg-white border",
            style:width = "140px",
            style:text_align = "center",
        ) {
            strong() { "Medium" }
            br() {}
            small(class = "text-muted") { "4px dither" }
        }
    }
    shadow_medium.set_content(&box_medium);

    // Coarse dither
    let mut shadow_coarse = Shadow::new();
    shadow_coarse.set_dither(Dither::Coarse);
    shadow_coarse.set_color("#006600");
    shadow_coarse.set_offset(8);
    rsx! {
        let box_coarse = div(
            class = "p-3 bg-white border",
            style:width = "140px",
            style:text_align = "center",
        ) {
            strong() { "Coarse" }
            br() {}
            small(class = "text-muted") { "8px dither" }
        }
    }
    shadow_coarse.set_content(&box_coarse);

    rsx! {
        let content = div(
            class = "d-flex flex-wrap gap-4",
            style:background_color = "var(--gray200)",
            style:padding = "1.5em",
        ) {
            {&shadow_fine}
            {&shadow_medium}
            {&shadow_coarse}
        }
    }
    section.push(&content);
    section
}

/// Build the "Icons" section with a sampling from each category.
fn build_icons<V: View>() -> Section<V> {
    let section = Section::new("Icons");

    // Representative sampling: ~3 per category
    const SAMPLE_ICONS: &[(IconGlyph, &str)] = &[
        // Navigation
        (IconGlyph::ArrowLeft, "ArrowLeft"),
        (IconGlyph::ArrowRight, "ArrowRight"),
        (IconGlyph::Bars, "Bars"),
        // Actions
        (IconGlyph::Check, "Check"),
        (IconGlyph::Plus, "Plus"),
        (IconGlyph::Trash, "Trash"),
        // Status
        (IconGlyph::Bell, "Bell"),
        (IconGlyph::CircleCheck, "CircleCheck"),
        (IconGlyph::TriangleExclamation, "Warning"),
        // Content
        (IconGlyph::Calendar, "Calendar"),
        (IconGlyph::Envelope, "Envelope"),
        (IconGlyph::Folder, "Folder"),
        // Objects
        (IconGlyph::Eye, "Eye"),
        (IconGlyph::Gear, "Gear"),
        (IconGlyph::Lock, "Lock"),
        // People
        (IconGlyph::Heart, "Heart"),
        (IconGlyph::Star, "Star"),
        (IconGlyph::User, "User"),
        // Layout
        (IconGlyph::Grip, "Grip"),
        (IconGlyph::TableCells, "TableCells"),
    ];

    let icon_cells: Vec<V::Element> = SAMPLE_ICONS
        .iter()
        .map(|(glyph, label)| {
            let icon = Icon::new(*glyph, IconSize::Large);
            let label_text = V::Text::new(*label);
            rsx! {
                let cell = div(
                    style:text_align = "center",
                    style:min_width = "4.5rem",
                ) {
                    div() { {&icon} }
                    small(class = "text-muted") { {label_text} }
                }
            }
            cell
        })
        .collect();

    rsx! {
        let content = div(class = "panel") {
            div(class = "d-flex flex-wrap gap-3") {
                {icon_cells}
            }
            small(class = "text-muted mt-2", style:display = "block") {
                "Showing 20 of 50 available icons. See IconGlyph for full list."
            }
        }
    }
    section.push(&content);
    section
}

/// Build the "Title Bars" section showing various title bar configurations.
fn build_title_bars<V: View>() -> Section<V> {
    let section = Section::new("Title Bars");

    // Basic title bar (no close button, no icon)
    let title_bar_basic = TitleBar::new("My Window");

    // Title bar with close button
    let mut title_bar_close = TitleBar::new("Closeable Window");
    title_bar_close.set_show_close_button(true);

    // Title bar with icon
    let mut title_bar_icon = TitleBar::new("Document.txt");
    title_bar_icon.set_icon(Some(IconGlyph::File));

    // Title bar with close button and icon
    let mut title_bar_full = TitleBar::new("Finder");
    title_bar_full.set_show_close_button(true);
    title_bar_full.set_icon(Some(IconGlyph::Folder));

    // Title bar with long title (to show ellipsis behavior)
    let mut title_bar_long = TitleBar::new("This Is A Very Long Window Title That Should Truncate");
    title_bar_long.set_show_close_button(true);

    rsx! {
        let content = div(class = "d-flex flex-wrap gap-4") {
            div(class = "window", style:width = "300px") {
                {&title_bar_basic}
                div(class = "container") { "Basic (no close button)" }
            }
            div(class = "window", style:width = "300px") {
                {&title_bar_close}
                div(class = "container") { "With Close Button" }
            }
            div(class = "window", style:width = "300px") {
                {&title_bar_icon}
                div(class = "container") { "With Icon" }
            }
            div(class = "window", style:width = "300px") {
                {&title_bar_full}
                div(class = "container") { "Full (Close + Icon)" }
            }
            div(class = "window", style:width = "300px") {
                {&title_bar_long}
                div(class = "container") { "Long Title (truncation)" }
            }
        }
    }
    section.push(&content);
    section
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
        let selects = build_selects::<V>();
        let dropdowns = build_dropdowns::<V>();
        let text_inputs = build_text_inputs::<V>();
        let alerts = build_alerts::<V>();
        let badges = build_badges::<V>();
        let cards = build_cards::<V>();
        let shadows = build_shadows::<V>();
        let icons = build_icons::<V>();
        let title_bars = build_title_bars::<V>();

        rsx! {
            let wrapper = div(class = "container") {
                {header}
                {&panels}
                {&buttons}
                div(class = "row") {
                    div(class = "col-auto") {
                        {&checkboxes}
                    }
                    div(class = "col-auto") {
                        {&progress}
                    }
                    div(class = "col-auto") {
                        {&sliders}
                    }
                    div(class = "col-auto") {
                        {&selects}
                    }
                    div(class = "col-auto") {
                        {&dropdowns}
                    }
                }
                {&text_inputs}
                div(class = "row") {
                    div(class = "col-auto") {
                        {&alerts}
                    }
                    div(class = "col-auto") {
                        {&badges}
                    }
                    div(class = "col-auto") {
                        {&cards}
                    }
                }
                div(class = "row") {
                    div(class = "col-auto") {
                        {&shadows}
                    }
                    div(class = "col-auto") {
                        {&icons}
                    }
                    div(class = "col-auto") {
                        {&title_bars}
                    }
                }
            }
        }

        Self { wrapper }
    }
}
