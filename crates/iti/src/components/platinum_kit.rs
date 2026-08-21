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
use wasm_bindgen::UnwrapThrowExt;

use crate::components::alert::Alert;
use crate::components::badge::Badge;
use crate::components::button::{Button, PrimaryButton};
use crate::components::checkbox::Checkbox;
use crate::components::dropdown::{Dropdown, DropdownEvent};
use crate::components::form_group::FormGroup;
use crate::components::icon::{Icon, IconGlyph, IconSize};
use crate::components::icon_classic::{
    ApplicationsIcon, ControlPanelIcon, ControlStripIcon, FolderIcon, IconClassic,
    IconClassicGlyph, MenuBarIcon, SystemIcon,
};
use crate::components::progress::Progress;
use crate::components::section::{Section, SectionEntry, SectionStyle, StaticContent};
use crate::components::select::Select;
use crate::components::slider::SliderWithTicks;
use crate::components::tab::{
    EmptySpacer, TabAlignment, TabListItemEvent, TabPanel, TabPanelEntryEvent, TabPanelEvent,
};
use crate::components::table::library::TableLibraryItem;
use crate::components::text_input::{TextInput, TextInputType};
use crate::components::textarea::Textarea;
use crate::components::title_bar::TitleBar;
use crate::components::Flavor;
use crate::form_traits::FormComponent;
use crate::Form;

pub mod button_groups;
pub mod checkboxes_and_radios;
pub mod lists;
pub mod modals;
pub mod panes;

#[derive(ViewChild)]
pub struct ProgressBars<V: View> {
    #[child]
    wrapper: V::Element,
    progress: Progress<V>,
    zero_button: Button<V>,
    percent_text: V::Text,
}

impl<V: View> StepMut for ProgressBars<V> {
    type Output = ();
    async fn step_mut(&mut self) {
        loop {
            let hit_zero = self.zero_button.step().map(Some);
            let tick = async {
                mogwai::time::wait_millis(200).await;
                None
            };
            match hit_zero.or(tick).await {
                Some(_ev) => {
                    self.progress.set_value(0);
                }
                None => {
                    let current = self.progress.get_value();
                    self.progress.set_value(current + 1);
                }
            }
            self.percent_text
                .set_text(format!("{}%", self.progress.get_value()));
        }
    }
}

#[derive(ViewChild)]
pub struct IconClassicLibraryItem<V: View> {
    #[child]
    container: V::Element,
}

impl<V: View> Default for IconClassicLibraryItem<V> {
    fn default() -> Self {
        fn make_icons<V: View>(
            title: &str,
            glyphs: impl IntoIterator<Item = IconClassicGlyph>,
        ) -> V::Element {
            rsx! {
                let wrapper = div(class = "panel mb-4") {
                    p(class = "mb-2") {
                        {title.into_text::<V>()}
                    }
                    div(class = "row") {
                        {{
                            glyphs
                                .into_iter()
                                .map(|icon| {
                                    rsx! {
                                        let wrapper = div(class = "col-auto mb-3") {
                                            {IconClassic::<V>::new(icon)}
                                        }
                                    }
                                    wrapper
                                })
                                .collect::<Vec<_>>()
                        }}
                    }
                }
            }
            wrapper
        }
        rsx! {
            let container = slot() {
                {make_icons::<V>("System Icons (41)", SystemIcon::ALL.map(IconClassicGlyph::System))}
                {make_icons::<V>("Application Icons (22)", ApplicationsIcon::ALL.map(IconClassicGlyph::Applications))}
                {make_icons::<V>("Control Panel Icons (34)", ControlPanelIcon::ALL.map(IconClassicGlyph::ControlPanel))}
                {make_icons::<V>("Control Strip Icons (12)", ControlStripIcon::ALL.map(IconClassicGlyph::ControlStrip))}
                {make_icons::<V>("Folder Icons (24)", FolderIcon::ALL.map(IconClassicGlyph::Folder))}
                {make_icons::<V>("Menu Bar Icons (16)", MenuBarIcon::ALL.map(IconClassicGlyph::MenuBar))}
            }
        }

        Self { container }
    }
}

impl<V: View> StepMut for IconClassicLibraryItem<V> {
    type Output = ();
    async fn step_mut(&mut self) {
        // Await on pending to keep the event loop running
        futures_lite::future::pending().await
    }
}

#[derive(ViewChild)]
struct SectionTop<V: View> {
    #[child]
    wrapper: V::Element,
    title: String,
    enabled: bool,
    on_click: V::EventListener,
    toggle: Checkbox<V>,
}

impl<V: View> SectionTop<V> {
    fn format_enabled_key(title: &str) -> String {
        let title = title.replace(" ", "-").to_lowercase();
        format!("section-{title}-enabled")
    }

    fn read_enabled(title: &str) -> Result<bool, crate::error::Error> {
        let key = Self::format_enabled_key(title);
        let maybe_bool: Option<bool> = crate::storage::get_item(&key)?;
        Ok(maybe_bool.unwrap_or_else(|| {
            log::info!("{key} was not stored, defaulting");
            true
        }))
    }

    fn write_enabled(&self) -> Result<(), crate::error::Error> {
        let key = Self::format_enabled_key(&self.title);
        let enabled = self.enabled;
        log::info!("writing {key}: {enabled}");
        crate::storage::set_item(key, &enabled)?;
        Ok(())
    }

    fn new(title: &str) -> Self {
        let enabled = Self::read_enabled(title).unwrap_throw();

        rsx! {
            let wrapper = span(
                class = "section-top-title",
                on:click = on_click
            ) {
                let toggle = {{
                    let c = Checkbox::new("", enabled);
                    c.set_style("float", "left");
                    c
                }}
                {V::Text::new(title)}
            }
        }

        Self {
            wrapper,
            title: title.to_string(),
            enabled,
            on_click,
            toggle,
        }
    }
}

impl<V: View> StepMut for SectionTop<V> {
    type Output = bool;
    async fn step_mut(&mut self) -> bool {
        let _ev = self.on_click.next().await;

        self.enabled = !self.enabled;

        if self.toggle.is_checked() != self.enabled {
            self.toggle.set_checked(self.enabled);
        }

        log::info!("section {} toggled: {}", self.title, self.enabled);
        let _ = self.write_enabled();

        self.enabled
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
fn build_panels_and_colors<V: View>() -> Section<V, SectionTop<V>, StaticContent<V>> {
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
                    p() { strong() { "Bevels" } }
                    p() {
                        "Border-color and box-shadow bevels driven by the \
                         --iti-bevel-* tokens. Outer bevels raise (light \
                         top-left); inner bevels press (light bottom-right)."
                    }
                    div(class = "d-flex flex-wrap gap-2") {
                        div(
                            class = "bevel-outer bg-gray200",
                            style:width = "120px",
                            style:height = "48px",
                            style:display = "flex",
                            style:align_items = "center",
                            style:justify_content = "center",
                        ) { small() { ".bevel-outer" } }
                        div(
                            class = "bevel-inner bg-gray200",
                            style:width = "120px",
                            style:height = "48px",
                            style:display = "flex",
                            style:align_items = "center",
                            style:justify_content = "center",
                        ) { small() { ".bevel-inner" } }
                        div(
                            class = "bevel-outer-shadow bg-gray200",
                            style:width = "120px",
                            style:height = "48px",
                            style:display = "flex",
                            style:align_items = "center",
                            style:justify_content = "center",
                        ) { small() { ".bevel-outer-shadow" } }
                        div(
                            class = "bevel-inner-shadow bg-gray200",
                            style:width = "120px",
                            style:height = "48px",
                            style:display = "flex",
                            style:align_items = "center",
                            style:justify_content = "center",
                        ) { small() { ".bevel-inner-shadow" } }
                        div(
                            class = "bevel-outer-shadow-sm bg-gray200",
                            style:width = "120px",
                            style:height = "48px",
                            style:display = "flex",
                            style:align_items = "center",
                            style:justify_content = "center",
                        ) { small() { ".bevel-outer-shadow-sm" } }
                        div(
                            class = "bevel-inner-shadow-sm bg-gray200",
                            style:width = "120px",
                            style:height = "48px",
                            style:display = "flex",
                            style:align_items = "center",
                            style:justify_content = "center",
                        ) { small() { ".bevel-inner-shadow-sm" } }
                    }
                }
                div(style:margin_top = "1em") {
                    p() { strong() { "Color Palette" } }
                    div() { "Panels and framing:" }
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
                    div(){"Soothing flavors:"}
                    div(class = "d-flex flex-wrap gap-2") {
                        {swatch::<V>("bg-azul", "azul")}
                        {swatch::<V>("bg-lavender", "lavender")}
                        {swatch::<V>("bg-thistle", "thistle")}
                        {swatch::<V>("bg-ice", "ice")}
                        {swatch::<V>("bg-cream", "cream")}
                        {swatch::<V>("bg-charcoal", "charcoal")}
                    }
                    div(){ "Attention getting:" }
                    div(class = "d-flex flex-wrap gap-2", style:margin_top = "0.5em") {
                        {swatch::<V>("bg-success", "success")}
                        {swatch::<V>("bg-danger", "danger")}
                        {swatch::<V>("bg-warning", "warning")}
                    }
                }
            }
        }
    }
    Section::new(
        SectionStyle::Titled,
        crate::color::PURPLE,
        SectionTop::new("Panels and colors"),
        StaticContent::new(panels),
    )
}

/// Build the "Buttons" section with all button variants.
fn build_buttons<V: View>() -> Section<V, SectionTop<V>, StaticContent<V>> {
    let btn_normal = Button::new("Button", None);

    let btn_disabled = Button::new("Disabled", None);
    btn_disabled.disable();

    let primary_normal = PrimaryButton::new("OK", None);

    let primary_disabled = PrimaryButton::new("Disabled", None);
    primary_disabled.disable();

    let btn_success = Button::new("Success", Some(Flavor::Success));
    let btn_danger = Button::new("Danger", Some(Flavor::Danger));
    let btn_warning = Button::new("Warning", Some(Flavor::Warning));
    let btn_info = Button::new("Info", Some(Flavor::Info));

    let mut btn_add = Button::new("Add", None);
    btn_add.set_has_icon(true);

    let mut btn_delete = Button::new("Delete", None);
    btn_delete.set_has_icon(true);
    btn_delete.get_icon_mut().set_glyph(IconGlyph::Trash);

    let mut btn_edit = Button::new("Edit", None);
    btn_edit.set_has_icon(true);
    btn_edit.get_icon_mut().set_glyph(IconGlyph::Pen);

    let mut btn_search = Button::new("Search", None);
    btn_search.set_has_icon(true);
    btn_search
        .get_icon_mut()
        .set_glyph(IconGlyph::MagnifyingGlass);

    let mut icon_plus = Button::new("", None);
    icon_plus.set_has_icon(true);
    icon_plus.get_icon_mut().remove_class("me-1");

    let mut icon_trash = Button::new("", None);
    icon_trash.set_has_icon(true);
    icon_trash.get_icon_mut().set_glyph(IconGlyph::Trash);
    icon_trash.get_icon_mut().remove_class("me-1");

    let mut icon_edit = Button::new("", None);
    icon_edit.set_has_icon(true);
    icon_edit.get_icon_mut().set_glyph(IconGlyph::Pen);
    icon_edit.get_icon_mut().remove_class("me-1");

    let mut icon_search = Button::new("", None);
    icon_search.set_has_icon(true);
    icon_search
        .get_icon_mut()
        .set_glyph(IconGlyph::MagnifyingGlass);
    icon_search.get_icon_mut().remove_class("me-1");

    let icons_square: Vec<Button<V>> = IconGlyph::PEOPLE
        .into_iter()
        .map(|g| {
            let mut icon = Button::new("", None);
            icon.set_has_icon(true);
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
    Section::new(
        SectionStyle::Titled,
        crate::color::PURPLE,
        SectionTop::new("Buttons"),
        StaticContent::new(content),
    )
}

/// Build the "Checkboxes & Radios" section.
fn build_checkboxes_and_radios<V: View>(
) -> Section<V, SectionTop<V>, checkboxes_and_radios::PlatinumKitCheckboxesAndRadios<V>> {
    Section::new(
        SectionStyle::Titled,
        crate::color::PURPLE,
        SectionTop::new("Checkboxes & Radios"),
        Default::default(),
    )
}

/// Build the "Progress Bars" section.
fn build_progress_bars<V: View>() -> Section<V, SectionTop<V>, ProgressBars<V>> {
    rsx! {
        let wrapper = div(class = "panel", style:padding = "1em") {
            p() {
                strong() {
                    let percent_text = "0%"
                }
            }

            div(class = "mb-3") {
                let progress = {Progress::new(0)}
            }

            let zero_button = {{ Button::new("Set to 0%", None) }}
        }
    }

    Section::new(
        SectionStyle::Titled,
        crate::color::PURPLE,
        SectionTop::new("Progress Bars"),
        ProgressBars {
            wrapper,
            progress,
            zero_button,
            percent_text,
        },
    )
}

/// Build the "Sliders" section.
fn build_sliders<V: View>() -> Section<V, SectionTop<V>, StaticContent<V>> {
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
    Section::new(
        SectionStyle::Titled,
        crate::color::PURPLE,
        SectionTop::new("Sliders"),
        StaticContent::new(content),
    )
}

/// Build the "Selects" section with native select dropdowns.
fn build_selects<V: View>() -> Section<V, SectionTop<V>, StaticContent<V>> {
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
    Section::new(
        SectionStyle::Titled,
        crate::color::PURPLE,
        SectionTop::new("Selects"),
        StaticContent::new(content),
    )
}

/// Build the "Dropdowns" section with button dropdown menus.
fn build_dropdowns<V: View>() -> Section<V, SectionTop<V>, StaticContent<V>> {
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

    Section::new(
        SectionStyle::Titled,
        crate::color::PURPLE,
        SectionTop::new("Dropdowns"),
        StaticContent::new(content),
    )
}

/// Build the "Text Inputs" section using TextInput and Textarea components.
fn build_text_inputs<V: View>() -> Section<V, SectionTop<V>, StaticContent<V>> {
    // Default text input
    let default_input = TextInput::<V>::new(TextInputType::Text, "");
    default_input.set_placeholder("Enter text");

    // With placeholder
    let placeholder_input = TextInput::<V>::new(TextInputType::Text, "");
    placeholder_input.set_placeholder("Enter your name...");

    // Email (required, with validation)
    let email_input = TextInput::<V>::new(TextInputType::Email, "");
    email_input.set_placeholder("user@example.com");
    email_input.set_required(true);

    // With help text — wrap in a FormGroup
    let username_input = TextInput::<V>::new(TextInputType::Text, "");
    let mut username_group = FormGroup::new("Username", username_input);
    username_group.set_help_text("Choose a unique username.");

    // Disabled
    let disabled_input = TextInput::<V>::new(TextInputType::Text, "Can't edit this");
    disabled_input.disable();

    // Read-only
    let readonly_input = TextInput::<V>::new(TextInputType::Text, "Read-only value");
    readonly_input.set_readonly(true);

    // Sizes
    let mut small_input = TextInput::<V>::new(TextInputType::Text, "");
    small_input.set_placeholder("Small");
    small_input.set_additional_classes("form-control-sm");

    let default_size_input = TextInput::<V>::new(TextInputType::Text, "");
    default_size_input.set_placeholder("Default");

    let mut large_input = TextInput::<V>::new(TextInputType::Text, "");
    large_input.set_placeholder("Large");
    large_input.set_additional_classes("form-control-lg");

    // Input types
    let password_input = TextInput::<V>::new(TextInputType::Password, "secret123");
    let search_input = TextInput::<V>::new(TextInputType::Search, "");
    search_input.set_placeholder("Search...");

    // Textarea
    let comments = Textarea::<V>::new("");
    comments.set_placeholder("Enter your comments here...");
    comments.set_rows(4);
    let comments_group = FormGroup::new("Comments", comments);

    rsx! {
        let content = div(class = "d-flex flex-wrap gap-4 panel") {
            div() {
                p() { strong() { "Default" } }
                {&default_input}
            }
            div() {
                p() { strong() { "With Placeholder" } }
                {&placeholder_input}
            }

            div() {
                p() { strong() { "Email (Required)" } }
                {&email_input}
            }
            div() {
                p() { strong() { "With Help Text" } }
                {&username_group}
            }

            div() {
                p() { strong() { "Disabled" } }
                {&disabled_input}
            }
            div() {
                p() { strong() { "Read-only" } }
                {&readonly_input}
            }

            div() {
                p() { strong() { "Sizes" } }
                div(class = "d-flex gap-2 align-items-center flex-wrap") {
                    {&small_input}
                    {&default_size_input}
                    {&large_input}
                }
            }

            div() {
                p() { strong() { "Input Types" } }
                div(class = "d-flex flex-column gap-2") {
                    div() {
                        label(class = "form-label") { "Password" }
                        {&password_input}
                    }
                    div() {
                        label(class = "form-label") { "Search" }
                        {&search_input}
                    }
                }
            }

            div() {
                p() { strong() { "Textarea" } }
                {&comments_group}
            }
        }
    }

    // Drive interactive inputs so validation feedback works on blur.
    wasm_bindgen_futures::spawn_local(async move {
        use futures_lite::FutureExt;
        use mogwai::future::MogwaiFutureExt;

        let mut email_input = email_input;
        let mut username_group = username_group;
        let mut comments_group = comments_group;

        loop {
            let email_fut = email_input.step_mut().map(|_| 0u8);
            let username_fut = username_group.child_mut().step_mut().map(|_| 1u8);
            let comments_fut = comments_group.child_mut().step_mut().map(|_| 2u8);

            email_fut.or(username_fut.or(comments_fut)).await;

            username_group.update_validation();
            comments_group.update_validation();
        }
    });

    Section::new(
        SectionStyle::Titled,
        crate::color::PURPLE,
        SectionTop::new("Text Inputs"),
        StaticContent::new(content),
    )
}

/// Login form struct — the derive macro generates `LoginFormComponent<V>`.
#[allow(dead_code)]
#[derive(Debug, Form)]
pub struct LoginForm {
    #[form(input_type = "email")]
    #[form(required)]
    #[form(placeholder = "user@example.com")]
    #[form(help = "We never share your email.")]
    email: String,

    #[form(input_type = "password")]
    #[form(required)]
    #[form(placeholder = "At least 8 characters")]
    #[form(min_length = 8)]
    password: String,

    #[form(label = "Remember me")]
    remember_me: bool,
}

/// Contact form struct — the derive macro generates `ContactFormComponent<V>`.
#[allow(dead_code)]
#[derive(Debug, Form)]
pub struct ContactForm {
    #[form(required)]
    #[form(placeholder = "Jane Doe")]
    #[form(label_placement = "inline")]
    full_name: String,

    #[form(input_type = "url")]
    #[form(placeholder = "https://example.com")]
    #[form(label_placement = "inline")]
    website: String,

    #[form(input_type = "textarea")]
    #[form(placeholder = "Tell us what you think...")]
    comments: String,
}

/// Build the "Forms" section demonstrating the Form derive macro.
fn build_forms<V: View>() -> Section<V, SectionTop<V>, StaticContent<V>> {
    let login_form = LoginFormComponent::<V>::default();
    let contact_form = ContactFormComponent::<V>::default();
    let login_submit = Button::<V>::new("Sign In", None);
    let contact_submit = Button::<V>::new("Send", None);

    rsx! {
        let content = div(class = "d-flex flex-wrap gap-4") {
            // Explainer (bare on the lavender background, no panel)
            div(class = "mb-2") {
                p() {
                    strong() { "Your form is a Rust struct." }
                }
                p() {
                    "With iti, you define a plain struct, annotate it with "
                    code() { "#[derive(Form)]" }
                    ", and get a fully-functional form component at compile time, "
                    "no hand-written HTML required. Each field becomes a labeled "
                    "input with validation, help text, and ARIA wiring for free."
                }
                p() {
                    strong() { "Type-safe by construction." }
                    " Field types drive input kinds: "
                    code() { "String" }
                    " becomes a text input, "
                    code() { "bool" }
                    " becomes a checkbox, and "
                    code() { "#[form(input_type = \"email\")]" }
                    " opts into browser-native validation. The compiler catches "
                    "mismatches before they reach the browser."
                }
                p() {
                    strong() { "Validation is real." }
                    " FormGroup wraps every field with HTML5 constraint validation "
                    "(required, min length, pattern, email format) and surfaces "
                    "browser-native error messages on blur, all driven by a "
                    "pull-based async event loop. No callbacks, no channels, "
                    "no hidden state."
                }
                p() {
                    strong() { "Accessible out of the box." }
                    " Labels, help text, and error messages are linked to their "
                    "inputs via "
                    code() { "for" }
                    " and "
                    code() { "aria-describedby" }
                    " automatically. Screen readers get the full picture."
                }
            }

            // Login form: struct definition + rendered form side by side
            div(class = "d-flex flex-wrap gap-3") {
                pre(
                    class = "panel",
                    style:max_width = "380px",
                    style:overflow_x = "auto",
                    style:font_size = "13px",
                    style:margin = "0",
                ) {
                    r#"#[derive(Form)]
struct LoginForm {
    #[form(
        input_type = "email",
        required,
        placeholder = "user@example.com",
        help = "We'll never share your email."
    )]
    email: String,

    #[form(
        input_type = "password",
        required,
        placeholder = "At least 8 characters",
        min_length = 8
    )]
    password: String,

    #[form(label = "Remember me")]
    remember_me: bool,
}"#
                }

                div(class = "panel", style:max_width = "360px") {
                    h4(class = "mb-3") { "Login Form" }
                    {&login_form}
                    {&login_submit}
                }
            }

            // Contact form: struct definition + rendered form side by side
            div(class = "d-flex flex-wrap gap-3") {
                pre(
                    class = "panel",
                    style:max_width = "380px",
                    style:overflow_x = "auto",
                    style:font_size = "13px",
                    style:margin = "0",
                ) {
                    r#"#[derive(Form)]
struct ContactForm {
    #[form(
        required,
        placeholder = "Jane Doe",
        label_placement = "inline"
    )]
    full_name: String,

    #[form(
        input_type = "url",
        placeholder = "https://example.com",
        label_placement = "inline"
    )]
    website: String,

    #[form(
        input_type = "textarea",
        placeholder = "Tell us what you think..."
    )]
    comments: String,
}"#
                }

                div(class = "panel", style:max_width = "360px") {
                    h4(class = "mb-3") { "Contact Form" }
                    {&contact_form}
                    {&contact_submit}
                }
            }
        }
    }

    // Drive the form event loops. Each form races field events against its
    // submit button click. On submit, call value() to get the constructed
    // struct (or a FormError if validation fails).
    wasm_bindgen_futures::spawn_local(async move {
        use mogwai::step::{Step, StepMut};

        let mut login_form = login_form;
        let login_submit = login_submit;
        let mut contact_form = contact_form;
        let contact_submit = contact_submit;

        loop {
            use mogwai::future::MogwaiFutureExt;

            enum FormsAction {
                LoginField,
                LoginSubmit,
                ContactField,
                ContactSubmit,
            }

            let login_field = login_form
                .step_mut()
                .map(|_| FormsAction::LoginField)
                .boxed_local();
            let login_btn = login_submit
                .step()
                .map(|_| FormsAction::LoginSubmit)
                .boxed_local();
            let contact_field = contact_form
                .step_mut()
                .map(|_| FormsAction::ContactField)
                .boxed_local();
            let contact_btn = contact_submit
                .step()
                .map(|_| FormsAction::ContactSubmit)
                .boxed_local();

            let event =
                mogwai::future::race_all([login_field, login_btn, contact_field, contact_btn])
                    .await;

            match event {
                FormsAction::LoginField => {}
                FormsAction::LoginSubmit => match login_form.try_value() {
                    Ok(data) => log::info!("Login form submitted: {data:?}"),
                    Err(errors) => log::info!("Login form invalid: {errors:?}"),
                },
                FormsAction::ContactField => {}
                FormsAction::ContactSubmit => match contact_form.try_value() {
                    Ok(data) => log::info!("Contact form submitted: {data:?}"),
                    Err(errors) => log::info!("Contact form invalid: {errors:?}"),
                },
            }
        }
    });

    Section::new(
        SectionStyle::Titled,
        crate::color::PURPLE,
        SectionTop::new("Forms"),
        StaticContent::new(content),
    )
}

/// Build the "Alerts" section showing all flavor variants.
fn build_alerts<V: View>() -> Section<V, SectionTop<V>, StaticContent<V>> {
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

    let alert_items: Vec<_> = FLAVORS
        .iter()
        .map(|&f| Alert::new(format!("This is a {f} alert!"), f))
        .collect();
    rsx! {
        let content = div(class = "panel") {
            {alert_items}
        }
    }
    Section::new(
        SectionStyle::Titled,
        crate::color::PURPLE,
        SectionTop::new("Alerts"),
        StaticContent::new(content),
    )
}

/// Build the "Flush Alerts" section showing all flavor variants flush with the
/// panel's edges.
fn build_flush_alerts<V: View>() -> Section<V, SectionTop<V>, StaticContent<V>> {
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

    let flush_items: Vec<_> = FLAVORS
        .iter()
        .enumerate()
        .map(|(i, &f)| {
            let alert = Alert::new(format!("This is a {f} alert!"), f);
            alert.set_flush_x();
            if i == 0 {
                alert.set_flush_top();
            }
            if i == FLAVORS.len() - 1 {
                alert.set_flush_bottom();
            }
            alert
        })
        .collect();
    rsx! {
        let content = div(class = "panel") {
            {flush_items}
        }
    }
    Section::new(
        SectionStyle::Titled,
        crate::color::PURPLE,
        SectionTop::new("Flush Alerts"),
        StaticContent::new(content),
    )
}

/// Build the "Badges" section showing all flavor variants plus pill style.
fn build_badges<V: View>() -> Section<V, SectionTop<V>, StaticContent<V>> {
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
    Section::new(
        SectionStyle::Titled,
        crate::color::PURPLE,
        SectionTop::new("Badges"),
        StaticContent::new(content),
    )
}

/// Helper to build a simple [`TabPanel`] with the given tab/pane pairs.
fn make_tab_panel<V: View>(items: &[(&str, &[&str])]) -> TabPanel<V> {
    rsx! {
        let default_pane = p() { "Empty." }
    }
    let mut panel: TabPanel<V> = TabPanel::new(default_pane);
    for (tab_label, pane_items) in items {
        rsx! {
            let tab = span() { {(*tab_label).into_text::<V>()} }
        }
        rsx! {
            let pane = div(class = "row", style:padding = "0 1em") {
                let list = ul() {}
            }
        }
        for pane_item in *pane_items {
            rsx! {
                let item = li() { {(*pane_item).into_text::<V>()} }
            }
            list.append_child(&item);
        }
        let _ = panel.push(tab, pane);
    }
    panel
}

/// A tab panel whose panes are themselves tab panels.
type NestedTabPanel<V> = TabPanel<V, TabPanel<V>>;

/// Tabs section content for the platinum kit.
#[derive(ViewChild)]
pub struct PlatinumKitTabs<V: View> {
    #[child]
    wrapper: V::Element,
    /// Outer panel whose tabs select which inner panel is visible.
    /// Each pane is an inner `TabPanel` demo.
    outer_panel: NestedTabPanel<V>,
}

impl<V: View> StepMut for PlatinumKitTabs<V> {
    type Output = ();
    async fn step_mut(&mut self) {
        let ev = self
            .outer_panel
            .step_with_mut(|entry| match entry {
                crate::components::tab::TabOrSpacer::Item(item) => item
                    .step_with_mut(|inner| inner.step_mut().boxed_local())
                    .map(OuterEv::OuterClick)
                    .boxed_local(),
                crate::components::tab::TabOrSpacer::Spacer(_) => {
                    std::future::pending().boxed_local()
                }
            })
            .await;
        if let OuterEv::OuterClick(TabPanelEntryEvent::Tab(TabListItemEvent::Click(data))) = ev {
            self.outer_panel.select(&data.id);
        }
    }
}

type InnerPanelEvent<V> = TabPanelEvent<V, <V as View>::Element, <V as View>::Element, EmptySpacer>;

enum OuterEv<V: View> {
    OuterClick(TabPanelEntryEvent<V, <V as View>::Element, EmptySpacer, InnerPanelEvent<V>>),
}

/// Build the "Tabs" section with multiple tab panel alignment demos.
fn build_tabs<V: View>() -> Section<V, SectionTop<V>, PlatinumKitTabs<V>> {
    // ── TabPanel: no alignment (tabs fill naturally) ──
    let panel_default = make_tab_panel::<V>(&[
        (
            "Dinosaurs",
            &["Galimimus", "Deinonychus", "Ankylosaurus", "Barney"],
        ),
        ("Plants", &["Fern", "Tree Fern", "Other Ferns"]),
        ("Cave Folks", &["Zog", "Zug", "Zub"]),
    ]);
    panel_default.set_style("min-width", "500px");

    // ── TabPanel: Start alignment (tabs left, spacer right) ──
    let mut panel_start = make_tab_panel::<V>(&[
        ("Alpha", &["First item", "Second item"]),
        ("Beta", &["Another item"]),
    ]);
    panel_start.set_style("min-width", "500px");
    panel_start.set_alignment(TabAlignment::Start);

    // ── TabPanel: Center alignment ──
    let mut panel_center = make_tab_panel::<V>(&[
        ("Left", &["Port side"]),
        ("Middle", &["Amidships"]),
        ("Right", &["Starboard side"]),
    ]);
    panel_center.set_style("min-width", "500px");
    panel_center.set_alignment(TabAlignment::Center);

    // ── TabPanel: End alignment (spacer left, tabs right) ──
    let mut panel_end = make_tab_panel::<V>(&[
        ("Settings", &["Volume", "Brightness"]),
        ("About", &["Version 1.0"]),
    ]);
    panel_end.set_style("min-width", "500px");
    panel_end.set_alignment(TabAlignment::End);

    // ── TabPanel: Split groups (two left, spacer, two right) ──
    rsx! { let split_default = p() { "Select a tab." } }
    let mut panel_split: TabPanel<V> = TabPanel::new(split_default);
    panel_split.set_style("min-width", "500px");
    {
        rsx! { let tab = span() { "File" } }
        rsx! { let pane = div(class = "row", style:padding = "0 1em") { ul() { li() { "New" } li() { "Open" } li() { "Save" } } } }
        let _ = panel_split.push(tab, pane);

        rsx! { let tab = span() { "Edit" } }
        rsx! { let pane = div(class = "row", style:padding = "0 1em") { ul() { li() { "Cut" } li() { "Copy" } li() { "Paste" } } } }
        let edit_id = panel_split.push(tab, pane);

        rsx! { let tab = span() { "View" } }
        rsx! { let pane = div(class = "row", style:padding = "0 1em") { ul() { li() { "Zoom In" } li() { "Zoom Out" } } } }
        let _ = panel_split.push(tab, pane);

        rsx! { let tab = span() { "Help" } }
        rsx! { let pane = div(class = "row", style:padding = "0 1em") { ul() { li() { "About" } li() { "Docs" } } } }
        let _ = panel_split.push(tab, pane);

        // Insert a spacer between "Edit" and "View".
        panel_split.insert_spacer_after(&edit_id, crate::components::tab::EmptySpacer);
    }

    // ── Outer TabPanel: each pane is an inner TabPanel demo ──
    rsx! { let outer_default_pane = p() { "Select a tab." } }
    let outer_default = TabPanel::<V>::new(outer_default_pane);
    let mut outer_panel: NestedTabPanel<V> = TabPanel::new(outer_default);

    rsx! { let tab = span() { "No alignment" } }
    outer_panel.push(tab, panel_default);

    rsx! { let tab = span() { "Start alignment" } }
    outer_panel.push(tab, panel_start);

    rsx! { let tab = span() { "Center alignment" } }
    outer_panel.push(tab, panel_center);

    rsx! { let tab = span() { "End alignment" } }
    outer_panel.push(tab, panel_end);

    rsx! { let tab = span() { "Split groups" } }
    outer_panel.push(tab, panel_split);

    rsx! {
        let wrapper = div(class = "container-fluid") {
            {&outer_panel}
        }
    }

    Section::new(
        SectionStyle::Titled,
        crate::color::PURPLE,
        SectionTop::new("Tabs"),
        PlatinumKitTabs {
            wrapper,
            outer_panel,
        },
    )
}

/// Build the "Icons" section with a sampling from each category.
fn build_icons<V: View>() -> Section<V, SectionTop<V>, StaticContent<V>> {
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
    Section::new(
        SectionStyle::Titled,
        crate::color::PURPLE,
        SectionTop::new("FontAwesome Icons"),
        StaticContent::new(content),
    )
}

/// Build the "Title Bars" section showing various title bar configurations.
fn build_title_bars<V: View>() -> Section<V, SectionTop<V>, StaticContent<V>> {
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
    Section::new(
        SectionStyle::Titled,
        crate::color::PURPLE,
        SectionTop::new("Title Bars"),
        StaticContent::new(content),
    )
}

/// Build the "Classic Icons" section showing the System classic Mac OS icons.
fn build_icon_classic<V: View>() -> Section<V, SectionTop<V>, IconClassicLibraryItem<V>> {
    let icon_library = IconClassicLibraryItem::default();
    Section::new(
        SectionStyle::Titled,
        crate::color::PURPLE,
        SectionTop::new("Classic Icons"),
        icon_library,
    )
}

/// Build the "Tables" section showing the Platinum folder list style table.
fn build_tables<V: View>() -> Section<V, SectionTop<V>, TableLibraryItem<V>> {
    let table_library = TableLibraryItem::default();
    Section::new(
        SectionStyle::Titled,
        crate::color::PURPLE,
        SectionTop::new("Tables"),
        table_library,
    )
}

fn build_button_groups<V: View>(
) -> Section<V, SectionTop<V>, button_groups::PlatinumKitButtonGroups<V>> {
    Section::new(
        SectionStyle::Titled,
        crate::color::PURPLE,
        SectionTop::new("Button Groups"),
        Default::default(),
    )
}

fn build_lists<V: View>() -> Section<V, SectionTop<V>, lists::PlatinumKitLists<V>> {
    Section::new(
        SectionStyle::Titled,
        crate::color::PURPLE,
        SectionTop::new("Lists"),
        Default::default(),
    )
}

fn build_modals<V: View>() -> Section<V, SectionTop<V>, modals::PlatinumKitModals<V>> {
    Section::new(
        SectionStyle::Titled,
        crate::color::PURPLE,
        SectionTop::new("Modals"),
        Default::default(),
    )
}

fn build_panes<V: View>() -> Section<V, SectionTop<V>, panes::PlatinumKitPanes<V>> {
    Section::new(
        SectionStyle::Titled,
        crate::color::PURPLE,
        SectionTop::new("Panes"),
        Default::default(),
    )
}

// ── Main component ──────────────────────────────────────────────

/// Sandbox library item for the Platinum design system overhaul.
///
/// Each section is built by a dedicated helper function and collected here.
/// Sections are type-erased via [`SectionEntry`] so heterogeneous section
/// types can be stored in a single `Vec`.
#[derive(ViewChild)]
pub struct OverhaulLibraryItem<V: View> {
    #[child]
    pub wrapper: V::Element,
    sections: Vec<Box<dyn SectionEntry<V>>>,
}

/// Build an explainer panel for the section-style demos.
fn explainer_panel<V: View>(text: &str) -> V::Element {
    rsx! {
        let el = div(class = "panel") {
            p() { {V::Text::new(text)} }
        }
    }
    el
}

impl<V: View> Default for OverhaulLibraryItem<V> {
    fn default() -> Self {
        let mut sections: Vec<Box<dyn SectionEntry<V>>> = vec![];
        let mut add_section = |section: Box<dyn SectionEntry<V>>| -> V::Element {
            section.element().set_style("max-width", "1140px");
            let root = section.element().clone();
            sections.push(section);
            root
        };

        // ── Section style demos ──
        rsx! {
            let fieldset = fieldset(
                class = "section-fieldset",
            ) {
                legend(class = "section-legend") {
                    "Fieldset"
                }
                div(
                    class = "section-body"
                ) {
                    "This is a fieldset. It has a soothing dashed border and an inset title."
                    "Create this with the rsx! macro."
                }
            }
        }
        fieldset.set_style("--section-color", crate::color::AZUL);

        let titled_demo = add_section(Box::new(Section::new(
            SectionStyle::Titled,
            crate::color::PURPLE,
            SectionTop::new("Titled Section"),
            StaticContent::new(explainer_panel::<V>(
                "This is a titled section. The legend sits above the dashed border.",
            )),
        )));
        let fieldset_demo = add_section(Box::new(Section::new(
            SectionStyle::Fieldset,
            crate::color::PURPLE,
            SectionTop::new("Fieldset Section"),
            StaticContent::new(explainer_panel::<V>(
                "This is a fieldset section. The legend sits embedded in the top border.",
            )),
        )));

        let header = build_header::<V>();
        let panels = add_section(Box::new(build_panels_and_colors::<V>()));
        let buttons = add_section(Box::new(build_buttons::<V>()));
        let checkboxes = add_section(Box::new(build_checkboxes_and_radios::<V>()));
        let progress = add_section(Box::new(build_progress_bars::<V>()));
        let sliders = add_section(Box::new(build_sliders::<V>()));
        let selects = add_section(Box::new(build_selects::<V>()));
        let dropdowns = add_section(Box::new(build_dropdowns::<V>()));
        let text_inputs = add_section(Box::new(build_text_inputs::<V>()));
        let forms = add_section(Box::new(build_forms::<V>()));
        let alerts = add_section(Box::new(build_alerts::<V>()));
        let flush_alerts = add_section(Box::new(build_flush_alerts::<V>()));
        let badges = add_section(Box::new(build_badges::<V>()));
        let tabs = add_section(Box::new(build_tabs::<V>()));
        let icons = add_section(Box::new(build_icons::<V>()));
        let icon_classics = add_section(Box::new(build_icon_classic::<V>()));
        let title_bars = add_section(Box::new(build_title_bars::<V>()));
        let tables = add_section(Box::new(build_tables::<V>()));
        let button_groups = add_section(Box::new(build_button_groups::<V>()));
        let lists = add_section(Box::new(build_lists::<V>()));
        let modals = add_section(Box::new(build_modals::<V>()));
        let panes = add_section(Box::new(build_panes::<V>()));

        rsx! {
            let wrapper = div(class = "container") {
                {header}
                {fieldset}
                div(class = "row") {
                    div(class = "col-auto") {
                        {&titled_demo}
                    }
                    div(class = "col-auto") {
                        {&fieldset_demo}
                    }
                    div(class = "col-auto") {
                        {&panels}
                    }
                    div(class = "col-auto") {
                        {&buttons}
                    }
                    div(class = "col") {
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
                    div(class = "col-auto") {
                        {&text_inputs}
                    }
                    div(class = "col-auto") {
                        {&forms}
                    }
                    div(class = "col-auto") {
                        {&alerts}
                    }
                    div(class = "col-auto") {
                        {&flush_alerts}
                    }
                    div(class = "col-auto") {
                        {&badges}
                    }
                    div(class = "col-auto") {
                        {&tabs}
                    }
                    div(class = "col-auto") {
                        {&icons}
                    }
                    div(class = "col-auto") {
                        {&icon_classics}
                    }
                    div(class = "col-auto") {
                        {&title_bars}
                    }
                    div(class = "col-auto") {
                        {&tables}
                    }
                    div(class = "col-auto") {
                        {&button_groups}
                    }
                    div(class = "col-auto") {
                        {&lists}
                    }
                    div(class = "col-auto") {
                        {&modals}
                    }
                    div(class = "col-auto") {
                        {&panes}
                    }
                }
            }
        }

        Self { wrapper, sections }
    }
}

impl<V: View> StepMut for OverhaulLibraryItem<V> {
    type Output = ();
    async fn step_mut(&mut self) {
        let sections = self
            .sections
            .iter_mut()
            .map(|section| section.step())
            .collect::<Vec<_>>();
        mogwai::future::race_all(sections).await
    }
}
