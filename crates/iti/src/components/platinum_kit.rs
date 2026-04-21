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
use crate::components::card::Card;
use crate::components::checkbox::Checkbox;
use crate::components::dropdown::{Dropdown, DropdownEvent};
use crate::components::icon::{Icon, IconGlyph, IconSize};
use crate::components::progress::Progress;
use crate::components::radio::RadioGroup;
use crate::components::select::Select;
use crate::components::shadow::{Dither, Shadow};
use crate::components::slider::SliderWithTicks;
use crate::components::tab::library::TabListLibraryItem;
use crate::components::tab::{TabList, TabListEvent, TabPanel};
use crate::components::title_bar::TitleBar;
use crate::components::Flavor;

#[derive(ViewChild)]
struct ProgressBars<V: View> {
    #[child]
    wrapper: V::Element,
    progress: Progress<V>,
    zero_button: Button<V>,
    percent_text: V::Text,
}

impl<V: View> ProgressBars<V> {
    async fn step(&mut self) {
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

pub enum SectionContent<V: View> {
    Any(V::Element),
    ProgressBars(ProgressBars<V>),
    TabPanel {
        wrapper: V::Element,
        tabs: TabList<V, V::Element>,
        tab_panel: TabPanel<V, V::Element, V::Element>,
    },
}

impl<V: View> ViewChild<V> for SectionContent<V> {
    fn as_append_arg(
        &self,
    ) -> AppendArg<V, impl Iterator<Item = std::borrow::Cow<'_, <V as View>::Node>>> {
        match self {
            SectionContent::Any(el) => el.as_boxed_append_arg(),
            SectionContent::ProgressBars(progress_bars) => progress_bars.as_boxed_append_arg(),
            SectionContent::TabPanel { wrapper, .. } => wrapper.as_boxed_append_arg(),
        }
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
                class = "editorial row",
                style:font_size = "2em",
                style:font_weight = "lighter",
                style:color = crate::color::PURPLE,
                style:cursor = "pointer",
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

    /// Awaits a click and returns the toggled "enabled" state.
    async fn step(&mut self) -> bool {
        let _ev = self.on_click.next().await;

        self.enabled = !self.enabled;

        if self.toggle.is_checked() != self.enabled {
            self.toggle.set_checked(self.enabled);
        }

        self.enabled
    }
}
/// A dashed purple section in the Platinum Kit sandbox.
///
/// Provides a titled container with a purple dashed border and an
/// editorial section heading.
#[derive(ViewChild)]
struct Section<V: View> {
    #[child]
    wrapper: V::Element,
    top: SectionTop<V>,
    content: SectionContent<V>,
    enabled: Proxy<bool>,
}

impl<V: View> Section<V> {
    /// Create a new section with the given title.
    fn new(title: &str, section_content: SectionContent<V>) -> Self {
        let top = SectionTop::new(title);
        let mut enabled = Proxy::new(top.enabled);

        rsx! {
            let wrapper = div(class = "container", style:margin_top = "2em") {
                {&top}
                div(
                    class = "row",
                    style:border = "2px dashed #7B61FF",
                    style:border_radius = "4px",
                    style:padding = "1em",
                    style:display = enabled(is_enabled => if *is_enabled {
                        "block"
                    } else {
                        "none"
                    })
                ) {
                    {&section_content}
                }
            }
        }
        Self {
            wrapper,
            content: section_content,
            enabled,
            top,
        }
    }

    async fn step(&mut self) {
        enum Step<V: View> {
            None,
            Top(bool),
            TabList(TabListEvent<V, V::Element>),
            TabPanel(TabListEvent<V, V::Element>),
        }
        loop {
            let top_toggled = self.top.step().map(Step::Top);
            let content = match &mut self.content {
                SectionContent::ProgressBars(progress_bars) => {
                    progress_bars.step().map(|_| Step::None).boxed_local()
                }
                SectionContent::TabPanel {
                    wrapper: _,
                    tabs,
                    tab_panel,
                } => {
                    let tab_list = tabs.step().map(Step::TabList);
                    let tab_panel = tab_panel.step().map(Step::TabPanel);
                    tab_list.or(tab_panel).boxed_local()
                }
                _ => futures_lite::future::pending().boxed_local(),
            };

            match top_toggled.or(content).await {
                Step::None => {}
                Step::Top(enabled) => {
                    log::info!("section {} toggled: {enabled}", self.top.title);
                    self.top.write_enabled().unwrap_throw();
                    self.enabled.set(enabled);
                }
                Step::TabList(_ev) => {
                    log::info!("tab list stepped");
                }
                Step::TabPanel(_ev) => {
                    log::info!("tab panel stepped");
                }
            }
        }
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
    Section::new("Panels and colors", SectionContent::Any(panels))
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
    Section::new("Buttons", SectionContent::Any(content))
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
    Section::new("Checkboxes & Radios", SectionContent::Any(content))
}

/// Build the "Progress Bars" section.
fn build_progress_bars<V: View>() -> Section<V> {
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

            let zero_button = {{
                let mut b = Button::new("Set to 0%", None);
                b.set_has_icon(false);
                b
            }}
        }
    }

    Section::new(
        "Progress Bars",
        SectionContent::ProgressBars(ProgressBars {
            wrapper,
            progress,
            zero_button,
            percent_text,
        }),
    )
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
    Section::new("Sliders", SectionContent::Any(content))
}

/// Build the "Selects" section with native select dropdowns.
fn build_selects<V: View>() -> Section<V> {
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
    Section::new("Selects", SectionContent::Any(content))
}

/// Build the "Dropdowns" section with button dropdown menus.
fn build_dropdowns<V: View>() -> Section<V> {
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

    Section::new("Dropdowns", SectionContent::Any(content))
}

/// Build the "Text Inputs" section with input variants and textarea.
fn build_text_inputs<V: View>() -> Section<V> {
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
    Section::new("Text Inputs", SectionContent::Any(content))
}

/// Build the "Alerts" section showing all flavor variants.
fn build_alerts<V: View>() -> Section<V> {
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
    rsx! {
        let content = slot() {
            {alert_items}
        }
    }
    Section::new("Alerts", SectionContent::Any(content))
}

/// Build the "Badges" section showing all flavor variants plus pill style.
fn build_badges<V: View>() -> Section<V> {
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
    Section::new("Badges", SectionContent::Any(content))
}

/// Build the "Cards" section with a sample card.
fn build_tabs<V: View>() -> Section<V> {
    rsx! {
        let default_pane = p() {
            "This is the default pane."
            "One must be supplied."
        }
    }
    let mut panel: TabPanel<V, V::Element, V::Element> = TabPanel::new(default_pane);
    panel.set_style("min-width", "500px");
    {
        rsx! {
            let dino_tab = span() {
                "Dinosaurs"
            }
        }
        rsx! {
            let dino_pane = div(class = "row") {
                ul() {
                    li() { "Galimimus" }
                    li() { "Deinonychus" }
                    li() { "Ankylosaurus" }
                    li() { "Barney" }
                }
            }
        }
        rsx! {
            let plant_tab = span() {
                "Plants"
            }
        }
        rsx! {
            let plant_pane = div(class = "row") {
                ul() {
                    li() { "Fern" }
                    li() { "Tree Fern" }
                    li() { "Other Ferns" }
                }
            }
        }
        rsx! {
            let cave_tab = span() {
                "Cave Folks"
            }
        }
        rsx! {
            let cave_pane = div(class = "row") {
                ul() {
                    li() { "Zog" }
                    li() { "Zug" }
                    li() { "Zub" }
                }
            }
        }

        let _ = panel.push(dino_tab, dino_pane);
        let _ = panel.push(plant_tab, plant_pane);
        let _ = panel.push(cave_tab, cave_pane);
    }

    let mut list = TabList::default();
    list.push({
        rsx! {
            let item = span() { "Mammals" }
        }
        item
    });
    list.push({
        rsx! {
            let item = span() { "Birds" }
        }
        item
    });
    list.push({
        rsx! {
            let item = span() { "Rocks" }
        }
        item
    });

    rsx! {
        let wrapper = div(class = "container-fluid") {
            div(class = "row mb-4") {
                {&list}
            }
            div(class = "row") {
                {&panel}
            }
        }
    }
    Section::new(
        "Tabs",
        SectionContent::TabPanel {
            wrapper,
            tabs: list,
            tab_panel: panel,
        },
    )
}

/// Build the "Icons" section with a sampling from each category.
fn build_icons<V: View>() -> Section<V> {
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
    Section::new("Icons", SectionContent::Any(content))
}

/// Build the "Title Bars" section showing various title bar configurations.
fn build_title_bars<V: View>() -> Section<V> {
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
    Section::new("Title Bars", SectionContent::Any(content))
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
    sections: Vec<Section<V>>,
}

impl<V: View> Default for OverhaulLibraryItem<V> {
    fn default() -> Self {
        // Add the section, but don't append it to the DOM.
        // Instead, returns the root element to be appended manually.
        let mut sections = vec![];
        let mut add_section = |section: Section<V>| -> V::Element {
            let root = section.wrapper.clone();
            sections.push(section);
            root
        };

        let header = build_header::<V>();
        let panels = add_section(build_panels_and_colors::<V>());
        let buttons = add_section(build_buttons::<V>());
        let checkboxes = add_section(build_checkboxes_and_radios::<V>());
        let progress = add_section(build_progress_bars::<V>());
        let sliders = add_section(build_sliders::<V>());
        let selects = add_section(build_selects::<V>());
        let dropdowns = add_section(build_dropdowns::<V>());
        let text_inputs = add_section(build_text_inputs::<V>());
        let alerts = add_section(build_alerts::<V>());
        let badges = add_section(build_badges::<V>());
        let tabs = add_section(build_tabs::<V>());
        let icons = add_section(build_icons::<V>());
        let title_bars = add_section(build_title_bars::<V>());

        rsx! {
            let wrapper = div(class = "container") {
                {header}
                div(class = "row") {
                    div(class = "col-auto") {
                        {&panels}
                    }
                    div(class = "col-auto") {
                        {&buttons}
                    }
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
                    div(class = "col-auto") {
                        {&text_inputs}
                    }
                    div(class = "col-auto") {
                        {&alerts}
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
                        {&title_bars}
                    }
                }
            }
        }

        Self { wrapper, sections }
    }
}

impl<V: View> OverhaulLibraryItem<V> {
    pub async fn step(&mut self) {
        let sections = self
            .sections
            .iter_mut()
            .map(|section| section.step())
            .collect::<Vec<_>>();
        mogwai::future::race_all(sections).await
    }
}
