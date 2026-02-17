//! Font Awesome icon component.
//!
//! Supports Font Awesome 6 Free icon styles (Solid, Regular, Brands) with
//! a comprehensive set of named glyph variants covering common UI needs.
use mogwai::prelude::*;

/// Font Awesome icon style.
///
/// Determines the visual weight and font family used to render the icon.
/// Not all glyphs are available in every style — Solid has the broadest
/// coverage in the Free set, while Regular offers outlined versions of
/// a subset, and Brands is exclusively for logo icons.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum IconStyle {
    /// Filled icons (`fa-solid`, weight 900). Broadest coverage.
    #[default]
    Solid,
    /// Outlined icons (`fa-regular`, weight 400). Subset of Solid.
    Regular,
    /// Brand/logo icons (`fa-brands`, weight 400).
    Brands,
}

impl IconStyle {
    pub fn as_str(&self) -> &str {
        match self {
            IconStyle::Solid => "fa-solid",
            IconStyle::Regular => "fa-regular",
            IconStyle::Brands => "fa-brands",
        }
    }
}

/// Font Awesome icon size classes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
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

    /// All sizes in order from smallest to largest.
    pub const ALL: [IconSize; 7] = [
        IconSize::Xxs,
        IconSize::Xs,
        IconSize::Sm,
        IconSize::Regular,
        IconSize::Large,
        IconSize::Xl,
        IconSize::Xxl,
    ];
}

/// Font Awesome icon glyph identifiers.
///
/// Named variants cover common UI icon needs across navigation, actions,
/// status, content, objects, and people categories. Use [`IconGlyph::Other`]
/// for any glyph not listed here — pass the Font Awesome class name
/// (e.g. `"fa-wand-magic-wand"`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IconGlyph {
    // ── Navigation ──────────────────────────────────────────────
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    Bars,
    ChevronDown,
    ChevronLeft,
    ChevronRight,
    ChevronUp,

    // ── Actions ─────────────────────────────────────────────────
    Check,
    Download,
    Filter,
    Link,
    MagnifyingGlass,
    Minus,
    Pen,
    Plus,
    Share,
    Sort,
    Trash,
    Upload,

    // ── Status / Feedback ───────────────────────────────────────
    Bell,
    CircleCheck,
    CircleExclamation,
    CircleInfo,
    CircleMinus,
    CirclePlus,
    CircleXmark,
    Flag,
    Spinner,
    TriangleExclamation,

    // ── Content ─────────────────────────────────────────────────
    Calendar,
    Clock,
    Envelope,
    File,
    Folder,
    Image,
    Tag,

    // ── Objects / Settings ──────────────────────────────────────
    Bolt,
    Eye,
    EyeSlash,
    Gear,
    Lock,

    // ── People / Social ─────────────────────────────────────────
    Globe,
    Heart,
    Star,
    User,
    Xmark,

    // ── Layout ──────────────────────────────────────────────────
    Grip,
    TableCells,

    /// Arbitrary glyph class not covered by a named variant.
    ///
    /// Pass the Font Awesome class name, e.g. `"fa-wand-magic-wand"`.
    Other(&'static str),
}

impl IconGlyph {
    pub fn as_str(&self) -> &str {
        match self {
            // Navigation
            IconGlyph::ArrowDown => "fa-arrow-down",
            IconGlyph::ArrowLeft => "fa-arrow-left",
            IconGlyph::ArrowRight => "fa-arrow-right",
            IconGlyph::ArrowUp => "fa-arrow-up",
            IconGlyph::Bars => "fa-bars",
            IconGlyph::ChevronDown => "fa-chevron-down",
            IconGlyph::ChevronLeft => "fa-chevron-left",
            IconGlyph::ChevronRight => "fa-chevron-right",
            IconGlyph::ChevronUp => "fa-chevron-up",

            // Actions
            IconGlyph::Check => "fa-check",
            IconGlyph::Download => "fa-download",
            IconGlyph::Filter => "fa-filter",
            IconGlyph::Link => "fa-link",
            IconGlyph::MagnifyingGlass => "fa-magnifying-glass",
            IconGlyph::Minus => "fa-minus",
            IconGlyph::Pen => "fa-pen",
            IconGlyph::Plus => "fa-plus",
            IconGlyph::Share => "fa-share",
            IconGlyph::Sort => "fa-sort",
            IconGlyph::Trash => "fa-trash",
            IconGlyph::Upload => "fa-upload",

            // Status / Feedback
            IconGlyph::Bell => "fa-bell",
            IconGlyph::CircleCheck => "fa-circle-check",
            IconGlyph::CircleExclamation => "fa-circle-exclamation",
            IconGlyph::CircleInfo => "fa-circle-info",
            IconGlyph::CircleMinus => "fa-circle-minus",
            IconGlyph::CirclePlus => "fa-circle-plus",
            IconGlyph::CircleXmark => "fa-circle-xmark",
            IconGlyph::Flag => "fa-flag",
            IconGlyph::Spinner => "fa-spinner",
            IconGlyph::TriangleExclamation => "fa-triangle-exclamation",

            // Content
            IconGlyph::Calendar => "fa-calendar",
            IconGlyph::Clock => "fa-clock",
            IconGlyph::Envelope => "fa-envelope",
            IconGlyph::File => "fa-file",
            IconGlyph::Folder => "fa-folder",
            IconGlyph::Image => "fa-image",
            IconGlyph::Tag => "fa-tag",

            // Objects / Settings
            IconGlyph::Bolt => "fa-bolt",
            IconGlyph::Eye => "fa-eye",
            IconGlyph::EyeSlash => "fa-eye-slash",
            IconGlyph::Gear => "fa-gear",
            IconGlyph::Lock => "fa-lock",

            // People / Social
            IconGlyph::Globe => "fa-globe",
            IconGlyph::Heart => "fa-heart",
            IconGlyph::Star => "fa-star",
            IconGlyph::User => "fa-user",
            IconGlyph::Xmark => "fa-xmark",

            // Layout
            IconGlyph::Grip => "fa-grip",
            IconGlyph::TableCells => "fa-table-cells",

            IconGlyph::Other(s) => s,
        }
    }

    /// Human-readable label for the glyph.
    pub fn label(&self) -> &str {
        match self {
            IconGlyph::ArrowDown => "ArrowDown",
            IconGlyph::ArrowLeft => "ArrowLeft",
            IconGlyph::ArrowRight => "ArrowRight",
            IconGlyph::ArrowUp => "ArrowUp",
            IconGlyph::Bars => "Bars",
            IconGlyph::ChevronDown => "ChevronDown",
            IconGlyph::ChevronLeft => "ChevronLeft",
            IconGlyph::ChevronRight => "ChevronRight",
            IconGlyph::ChevronUp => "ChevronUp",
            IconGlyph::Check => "Check",
            IconGlyph::Download => "Download",
            IconGlyph::Filter => "Filter",
            IconGlyph::Link => "Link",
            IconGlyph::MagnifyingGlass => "MagnifyingGlass",
            IconGlyph::Minus => "Minus",
            IconGlyph::Pen => "Pen",
            IconGlyph::Plus => "Plus",
            IconGlyph::Share => "Share",
            IconGlyph::Sort => "Sort",
            IconGlyph::Trash => "Trash",
            IconGlyph::Upload => "Upload",
            IconGlyph::Bell => "Bell",
            IconGlyph::CircleCheck => "CircleCheck",
            IconGlyph::CircleExclamation => "CircleExclamation",
            IconGlyph::CircleInfo => "CircleInfo",
            IconGlyph::CircleMinus => "CircleMinus",
            IconGlyph::CirclePlus => "CirclePlus",
            IconGlyph::CircleXmark => "CircleXmark",
            IconGlyph::Flag => "Flag",
            IconGlyph::Spinner => "Spinner",
            IconGlyph::TriangleExclamation => "TriangleExclamation",
            IconGlyph::Calendar => "Calendar",
            IconGlyph::Clock => "Clock",
            IconGlyph::Envelope => "Envelope",
            IconGlyph::File => "File",
            IconGlyph::Folder => "Folder",
            IconGlyph::Image => "Image",
            IconGlyph::Tag => "Tag",
            IconGlyph::Bolt => "Bolt",
            IconGlyph::Eye => "Eye",
            IconGlyph::EyeSlash => "EyeSlash",
            IconGlyph::Gear => "Gear",
            IconGlyph::Lock => "Lock",
            IconGlyph::Globe => "Globe",
            IconGlyph::Heart => "Heart",
            IconGlyph::Star => "Star",
            IconGlyph::User => "User",
            IconGlyph::Xmark => "Xmark",
            IconGlyph::Grip => "Grip",
            IconGlyph::TableCells => "TableCells",
            IconGlyph::Other(s) => s,
        }
    }

    /// All named glyphs (excluding [`IconGlyph::Other`]), grouped by category.
    pub const NAVIGATION: [IconGlyph; 9] = [
        IconGlyph::ArrowDown,
        IconGlyph::ArrowLeft,
        IconGlyph::ArrowRight,
        IconGlyph::ArrowUp,
        IconGlyph::Bars,
        IconGlyph::ChevronDown,
        IconGlyph::ChevronLeft,
        IconGlyph::ChevronRight,
        IconGlyph::ChevronUp,
    ];

    pub const ACTIONS: [IconGlyph; 12] = [
        IconGlyph::Check,
        IconGlyph::Download,
        IconGlyph::Filter,
        IconGlyph::Link,
        IconGlyph::MagnifyingGlass,
        IconGlyph::Minus,
        IconGlyph::Pen,
        IconGlyph::Plus,
        IconGlyph::Share,
        IconGlyph::Sort,
        IconGlyph::Trash,
        IconGlyph::Upload,
    ];

    pub const STATUS: [IconGlyph; 10] = [
        IconGlyph::Bell,
        IconGlyph::CircleCheck,
        IconGlyph::CircleExclamation,
        IconGlyph::CircleInfo,
        IconGlyph::CircleMinus,
        IconGlyph::CirclePlus,
        IconGlyph::CircleXmark,
        IconGlyph::Flag,
        IconGlyph::Spinner,
        IconGlyph::TriangleExclamation,
    ];

    pub const CONTENT: [IconGlyph; 7] = [
        IconGlyph::Calendar,
        IconGlyph::Clock,
        IconGlyph::Envelope,
        IconGlyph::File,
        IconGlyph::Folder,
        IconGlyph::Image,
        IconGlyph::Tag,
    ];

    pub const OBJECTS: [IconGlyph; 5] = [
        IconGlyph::Bolt,
        IconGlyph::Eye,
        IconGlyph::EyeSlash,
        IconGlyph::Gear,
        IconGlyph::Lock,
    ];

    pub const PEOPLE: [IconGlyph; 5] = [
        IconGlyph::Globe,
        IconGlyph::Heart,
        IconGlyph::Star,
        IconGlyph::User,
        IconGlyph::Xmark,
    ];

    pub const LAYOUT: [IconGlyph; 2] = [IconGlyph::Grip, IconGlyph::TableCells];
}

struct IconState {
    style: IconStyle,
    glyph: IconGlyph,
    size: IconSize,
    additional_classes: String,
}

/// A Font Awesome icon element.
///
/// Supports setting the glyph, size, style, additional CSS classes, and
/// visibility.
#[derive(ViewChild)]
pub struct Icon<V: View> {
    #[child]
    i: V::Element,
    state: Proxy<IconState>,
}

impl<V: View> Icon<V> {
    /// Create an icon with the given glyph and size, using [`IconStyle::Solid`].
    pub fn new(glyph: IconGlyph, size: IconSize) -> Self {
        Self::with_style(glyph, size, IconStyle::Solid)
    }

    /// Create an icon with explicit glyph, size, and style.
    pub fn with_style(glyph: IconGlyph, size: IconSize, style: IconStyle) -> Self {
        let mut state = Proxy::new(IconState {
            style,
            glyph,
            size,
            additional_classes: Default::default(),
        });

        rsx! {
            let i = i(
                class = state(
                    s => format!(
                        "{} {} {} {}",
                        s.style.as_str(),
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

    pub fn set_style(&mut self, style: IconStyle) {
        self.state.modify(|s| s.style = style);
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
    use futures_lite::FutureExt;
    use mogwai::future::MogwaiFutureExt;
    use mogwai::prelude::*;

    use super::*;

    /// Category with a title and a set of glyphs.
    struct Category {
        title: &'static str,
        glyphs: &'static [IconGlyph],
    }

    const CATEGORIES: &[Category] = &[
        Category {
            title: "Navigation",
            glyphs: &IconGlyph::NAVIGATION,
        },
        Category {
            title: "Actions",
            glyphs: &IconGlyph::ACTIONS,
        },
        Category {
            title: "Status",
            glyphs: &IconGlyph::STATUS,
        },
        Category {
            title: "Content",
            glyphs: &IconGlyph::CONTENT,
        },
        Category {
            title: "Objects",
            glyphs: &IconGlyph::OBJECTS,
        },
        Category {
            title: "People",
            glyphs: &IconGlyph::PEOPLE,
        },
        Category {
            title: "Layout",
            glyphs: &IconGlyph::LAYOUT,
        },
    ];

    #[derive(ViewChild)]
    pub struct IconLibraryItem<V: View> {
        #[child]
        pub wrapper: V::Element,
        icons: Vec<Icon<V>>,
        size_up_click: V::EventListener,
        size_down_click: V::EventListener,
        style_solid_click: V::EventListener,
        style_regular_click: V::EventListener,
        size_index: usize,
        current_style: IconStyle,
    }

    impl<V: View> Default for IconLibraryItem<V> {
        fn default() -> Self {
            let mut icons = Vec::new();

            let category_sections: Vec<V::Element> = CATEGORIES
                .iter()
                .map(|cat| {
                    let icon_cells: Vec<V::Element> = cat
                        .glyphs
                        .iter()
                        .map(|&glyph| {
                            let icon = Icon::new(glyph, IconSize::Large);
                            let label_text = V::Text::new(glyph.label());
                            rsx! {
                                let cell = div(
                                    class = "col text-center mb-3",
                                    style:min_width = "5rem",
                                ) {
                                    div() { {&icon} }
                                    small(class = "text-body-secondary") {
                                        {label_text}
                                    }
                                }
                            }
                            icons.push(icon);
                            cell
                        })
                        .collect();

                    let heading_text = V::Text::new(cat.title);
                    rsx! {
                        let section = div(class = "mb-4") {
                            h6(class = "text-body-secondary") { {heading_text} }
                            div(class = "row row-cols-auto g-2") {
                                {icon_cells}
                            }
                        }
                    }
                    section
                })
                .collect();

            rsx! {
                let wrapper = div() {
                    div(class = "btn-group mb-3 me-2") {
                        button(
                            type = "button",
                            class = "btn btn-sm btn-outline-secondary",
                            on:click = size_down_click,
                        ) {
                            "Size -"
                        }
                        button(
                            type = "button",
                            class = "btn btn-sm btn-outline-secondary",
                            on:click = size_up_click,
                        ) {
                            "Size +"
                        }
                    }
                    div(class = "btn-group mb-3") {
                        button(
                            type = "button",
                            class = "btn btn-sm btn-outline-primary active",
                            on:click = style_solid_click,
                        ) {
                            "Solid"
                        }
                        button(
                            type = "button",
                            class = "btn btn-sm btn-outline-primary",
                            on:click = style_regular_click,
                        ) {
                            "Regular"
                        }
                    }
                    {category_sections}
                }
            }

            Self {
                wrapper,
                icons,
                size_up_click,
                size_down_click,
                style_solid_click,
                style_regular_click,
                size_index: 4, // IconSize::Large
                current_style: IconStyle::Solid,
            }
        }
    }

    enum IconAction {
        SizeUp,
        SizeDown,
        StyleSolid,
        StyleRegular,
    }

    impl<V: View> IconLibraryItem<V> {
        pub async fn step(&mut self) {
            let action = self
                .size_up_click
                .next()
                .map(|_| IconAction::SizeUp)
                .or(self.size_down_click.next().map(|_| IconAction::SizeDown))
                .or(self
                    .style_solid_click
                    .next()
                    .map(|_| IconAction::StyleSolid))
                .or(self
                    .style_regular_click
                    .next()
                    .map(|_| IconAction::StyleRegular))
                .await;

            match action {
                IconAction::SizeUp => {
                    if self.size_index < IconSize::ALL.len() - 1 {
                        self.size_index += 1;
                    }
                    let size = IconSize::ALL[self.size_index];
                    for icon in &mut self.icons {
                        icon.set_size(size);
                    }
                }
                IconAction::SizeDown => {
                    if self.size_index > 0 {
                        self.size_index -= 1;
                    }
                    let size = IconSize::ALL[self.size_index];
                    for icon in &mut self.icons {
                        icon.set_size(size);
                    }
                }
                IconAction::StyleSolid => {
                    self.current_style = IconStyle::Solid;
                    for icon in &mut self.icons {
                        icon.set_style(IconStyle::Solid);
                    }
                }
                IconAction::StyleRegular => {
                    self.current_style = IconStyle::Regular;
                    for icon in &mut self.icons {
                        icon.set_style(IconStyle::Regular);
                    }
                }
            }
        }
    }
}
