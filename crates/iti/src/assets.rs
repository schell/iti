//! Static asset helpers for iti's custom CSS and Font Awesome 6.
//!
//! Provides three ways for consumers to load the CSS and fonts that iti
//! components depend on:
//!
//! 1. **CDN links** — [`inject_cdn_links`] adds a `<link>` tag for Font
//!    Awesome from a public CDN and injects iti's CSS as a `<style>` tag.
//!    Requires an internet connection for the icon font.
//!
//! 2. **Fully embedded** — With the `embed-assets` feature enabled,
//!    [`embedded::inject_styles`] injects all CSS and fonts directly from
//!    the WASM binary. No network connection required. Fonts are served
//!    via Blob URLs created at runtime from compiled-in woff2/ttf bytes.
//!
//! 3. **Manual / Trunk** — Consumers can ignore this module entirely and
//!    wire up assets themselves (e.g. with Trunk `data-trunk` directives
//!    or plain `<link>` tags in their `index.html`).

use js_sys::wasm_bindgen::{JsCast, UnwrapThrowExt};

/// iti's unified stylesheet (always embedded — includes all component styles).
pub const ITI_CSS: &str = include_str!("../../../assets/iti.css");

/// CDN URLs for external dependencies.
///
/// Only Font Awesome is loaded from a CDN. All other styles are provided
/// by `iti.css`.
pub mod cdn {
    /// Font Awesome 6.6.0 Free — all styles (includes `@font-face` for
    /// Solid, Regular, and Brands webfonts).
    pub const FONTAWESOME_CSS: &str =
        "https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.6.0/css/all.min.css";
}

/// Append a `<link rel="stylesheet">` element to `<head>`.
fn append_link(href: &str) {
    let document = web_sys::window().unwrap_throw().document().unwrap_throw();
    let head = document.head().unwrap_throw();
    let link = document
        .create_element("link")
        .unwrap_throw()
        .unchecked_into::<web_sys::HtmlLinkElement>();
    link.set_rel("stylesheet");
    link.set_href(href);
    head.append_child(&link).unwrap_throw();
}

/// Append a `<style>` element with the given CSS text to `<head>`.
fn append_style(css: &str) {
    let document = web_sys::window().unwrap_throw().document().unwrap_throw();
    let head = document.head().unwrap_throw();
    let style = document
        .create_element("style")
        .unwrap_throw()
        .unchecked_into::<web_sys::HtmlStyleElement>();
    style.set_text_content(Some(css));
    head.append_child(&style).unwrap_throw();
}

/// Inject the design token CSS custom properties as a `<style>` element.
///
/// This must be called **before** `iti.css` is loaded (whether via `<link>`
/// or `<style>`) so that the token variables (`--azul`, `--gray300`, etc.)
/// are available when the semantic aliases resolve their `var()` references.
///
/// Called automatically by [`inject_cdn_links`] and
/// [`embedded::inject_styles`]. For the Trunk/manual loading path, call
/// this from your WASM entry point before the stylesheet `<link>` loads.
pub fn inject_color_tokens() {
    append_style(&crate::color::css_tokens());
}

/// Inject all required stylesheets using a CDN `<link>` for Font Awesome.
///
/// Creates three elements in `<head>`:
/// - `<style>` for color token CSS custom properties
/// - `<link>` for Font Awesome 6 CSS (with fonts from CDN)
/// - `<style>` for iti's unified stylesheet
///
/// Requires an internet connection to reach the Font Awesome CDN.
pub fn inject_cdn_links() {
    inject_color_tokens();
    append_link(cdn::FONTAWESOME_CSS);
    append_style(ITI_CSS);
}

/// Fully embedded assets — available when the `embed-assets` feature is
/// enabled.
///
/// All CSS and icon fonts are compiled into the WASM binary. At runtime,
/// fonts are exposed via Blob URLs and the CSS `@font-face` declarations
/// are rewritten to reference those URLs before injection. No network
/// connection is required.
///
/// **Binary size cost:** approximately 1.2 MB (CSS + woff2/ttf fonts).
/// Only woff2 fonts are included for Font Awesome — all WASM-capable
/// browsers support woff2. Font Awesome Brands icons are excluded to
/// save space; only Solid, Regular, and v4-compatibility fonts are
/// embedded.
#[cfg(feature = "embed-assets")]
pub mod embedded {
    use js_sys::wasm_bindgen::UnwrapThrowExt;

    use super::*;

    // ── CSS ──────────────────────────────────────────────────────

    /// Font Awesome 6.6.0 Free minified CSS, embedded at compile time.
    ///
    /// The `@font-face` URLs are rewritten at runtime by
    /// [`inject_styles`] to point at Blob URLs.
    const FONTAWESOME_CSS: &str = include_str!("../../../assets/fontawesome/css/all.min.css");

    // ── Fonts (woff2) ──────────────────────────────────────

    const FA_SOLID_WOFF2: &[u8] =
        include_bytes!("../../../assets/fontawesome/webfonts/fa-solid-900.woff2");
    const FA_REGULAR_WOFF2: &[u8] =
        include_bytes!("../../../assets/fontawesome/webfonts/fa-regular-400.woff2");
    const FA_V4COMPAT_WOFF2: &[u8] =
        include_bytes!("../../../assets/fontawesome/webfonts/fa-v4compatibility.woff2");
    // -- Fonts (ttf)
    const CHICAGO_TTF: &[u8] = include_bytes!("../../../assets/fonts/ChicagoFLF.ttf");
    const GENEVA_TTF: &[u8] = include_bytes!("../../../assets/fonts/Geneva.ttf");
    const GARAMOND_LIGHT_TTF: &[u8] =
        include_bytes!("../../../assets/fonts/AppleGaramond-Light.ttf");
    const GARAMOND_REGULAR_TTF: &[u8] = include_bytes!("../../../assets/fonts/AppleGaramond.ttf");
    const GARAMOND_BOLD_TTF: &[u8] = include_bytes!("../../../assets/fonts/AppleGaramond-Bold.ttf");

    // ── Classic Mac OS Icons (149 PNGs) ──

    // APPLICATIONS
    const APPLICATIONS_ADOBE_ILLUSTRATOR_5_5: &[u8] =
        include_bytes!("../../../assets/icons-classic/applications/Adobe Illustrator 5.5.png");
    const APPLICATIONS_ADOBE_PHOTOSHOP_5_0: &[u8] =
        include_bytes!("../../../assets/icons-classic/applications/Adobe Photoshop 5.0.png");
    const APPLICATIONS_APPLE_FM_RADIO: &[u8] =
        include_bytes!("../../../assets/icons-classic/applications/Apple FM radio.png");
    const APPLICATIONS_APPLE_FILE_SECURITY: &[u8] =
        include_bytes!("../../../assets/icons-classic/applications/Apple File security.png");
    const APPLICATIONS_APPLE_SHARE_PREP: &[u8] =
        include_bytes!("../../../assets/icons-classic/applications/Apple Share Prep.png");
    const APPLICATIONS_APPLE_VERIFIER: &[u8] =
        include_bytes!("../../../assets/icons-classic/applications/Apple verifier.png");
    const APPLICATIONS_CALCULATOR: &[u8] =
        include_bytes!("../../../assets/icons-classic/applications/Calculator.png");
    const APPLICATIONS_DISK_COPY: &[u8] =
        include_bytes!("../../../assets/icons-classic/applications/Disk copy.png");
    const APPLICATIONS_DISK_FIRST_AID: &[u8] =
        include_bytes!("../../../assets/icons-classic/applications/Disk first aid.png");
    const APPLICATIONS_GRAPHING_CALCULATOR: &[u8] =
        include_bytes!("../../../assets/icons-classic/applications/Graphing calculator.png");
    const APPLICATIONS_KEY_CAPS: &[u8] =
        include_bytes!("../../../assets/icons-classic/applications/Key caps.png");
    const APPLICATIONS_MAIL: &[u8] =
        include_bytes!("../../../assets/icons-classic/applications/Mail.png");
    const APPLICATIONS_MICROSOFT_INTERNET_EXPLORER: &[u8] = include_bytes!(
        "../../../assets/icons-classic/applications/Microsoft Internet Explorer.png"
    );
    const APPLICATIONS_NOTEPAD: &[u8] =
        include_bytes!("../../../assets/icons-classic/applications/Notepad.png");
    const APPLICATIONS_QUICKTIME_PLAYER: &[u8] =
        include_bytes!("../../../assets/icons-classic/applications/Quicktime Player.png");
    const APPLICATIONS_SCRAPBOOK: &[u8] =
        include_bytes!("../../../assets/icons-classic/applications/Scrapbook.png");
    const APPLICATIONS_SCRIPT_EDITOR: &[u8] =
        include_bytes!("../../../assets/icons-classic/applications/Script editor.png");
    const APPLICATIONS_SHERLOCK_2_0: &[u8] =
        include_bytes!("../../../assets/icons-classic/applications/Sherlock 2.0.png");
    const APPLICATIONS_STICKIES: &[u8] =
        include_bytes!("../../../assets/icons-classic/applications/Stickies.png");
    const APPLICATIONS_STUFFIT_EXPANDER: &[u8] =
        include_bytes!("../../../assets/icons-classic/applications/Stuffit expander.png");
    const APPLICATIONS_WEBSITE: &[u8] =
        include_bytes!("../../../assets/icons-classic/applications/Website.png");
    const APPLICATIONS_ITUNES: &[u8] =
        include_bytes!("../../../assets/icons-classic/applications/iTunes.png");

    // CONTROL-PANEL
    const CONTROL_PANEL_ADOBE_GAMMA: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Adobe gamma.png");
    const CONTROL_PANEL_APPEARANCE: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Appearance.png");
    const CONTROL_PANEL_APPLE_MENU_OPTIONS: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Apple menu options.png");
    const CONTROL_PANEL_APPLETALK: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Appletalk.png");
    const CONTROL_PANEL_ATM: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Atm.png");
    const CONTROL_PANEL_COLORSYNC: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Colorsync.png");
    const CONTROL_PANEL_CONTROL_STRIP: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Control strip.png");
    const CONTROL_PANEL_DATE_AND_TIME: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Date and time.png");
    const CONTROL_PANEL_DIAL_ASSIST: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Dial assist.png");
    const CONTROL_PANEL_ENERGY_SAVER: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Energy saver.png");
    const CONTROL_PANEL_EXTENSIONS_MANAGER: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Extensions manager.png");
    const CONTROL_PANEL_FILE_EXCHANGE: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/File exchange.png");
    const CONTROL_PANEL_FILE_SHARING: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/File sharing.png");
    const CONTROL_PANEL_GENERAL_CONTROLS: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/General controls.png");
    const CONTROL_PANEL_INTERNET: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Internet.png");
    const CONTROL_PANEL_KEYBOARD: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Keyboard.png");
    const CONTROL_PANEL_KEYCHAIN_ACCESS: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Keychain access.png");
    const CONTROL_PANEL_LAUNCHER: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Launcher.png");
    const CONTROL_PANEL_LOCATION_MANAGER: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Location manager.png");
    const CONTROL_PANEL_MEMORY: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Memory.png");
    const CONTROL_PANEL_MODEM: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Modem.png");
    const CONTROL_PANEL_MONITOR: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Monitor.png");
    const CONTROL_PANEL_MOUSE: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Mouse.png");
    const CONTROL_PANEL_MULTIPLE_USERS: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Multiple users.png");
    const CONTROL_PANEL_NUMBERS: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Numbers.png");
    const CONTROL_PANEL_QUICKTIME_SETTINGS: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Quicktime settings.png");
    const CONTROL_PANEL_REMOTE_ACCESS: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Remote access.png");
    const CONTROL_PANEL_SOFTWARE_UPDATE: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Software update.png");
    const CONTROL_PANEL_SOUND: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Sound.png");
    const CONTROL_PANEL_SPEECH: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Speech.png");
    const CONTROL_PANEL_STARTUP_DISK: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Startup disk.png");
    const CONTROL_PANEL_TCPIP: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/TCPIP.png");
    const CONTROL_PANEL_TEXT: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Text.png");
    const CONTROL_PANEL_WEB_SHARING: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-panel/Web sharing.png");

    // CONTROL-STRIP
    const CONTROL_STRIP_APPLE_LOCATION: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-strip/Apple location.png");
    const CONTROL_STRIP_APPLE_TALK: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-strip/Apple talk.png");
    const CONTROL_STRIP_CD: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-strip/CD.png");
    const CONTROL_STRIP_FILE_SHARING: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-strip/File sharing.png");
    const CONTROL_STRIP_KEYCHAIN_STRIP: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-strip/Keychain strip.png");
    const CONTROL_STRIP_MONITOR_BITDEPTH: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-strip/Monitor bitdepth.png");
    const CONTROL_STRIP_MONITOR_RESOLUTION: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-strip/Monitor resolution.png");
    const CONTROL_STRIP_PRINTER: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-strip/Printer.png");
    const CONTROL_STRIP_REMOTE_ACCESS: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-strip/Remote access.png");
    const CONTROL_STRIP_SOUND_VOLUME: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-strip/Sound volume.png");
    const CONTROL_STRIP_WEB_SHARING: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-strip/Web sharing.png");
    const CONTROL_STRIP_ITUNES: &[u8] =
        include_bytes!("../../../assets/icons-classic/control-strip/iTunes.png");

    // FOLDER
    const FOLDER_APPLE_MENU_ITEM: &[u8] =
        include_bytes!("../../../assets/icons-classic/folder/Apple menu item.png");
    const FOLDER_APPLICATION_SUPPORT: &[u8] =
        include_bytes!("../../../assets/icons-classic/folder/Application Support.png");
    const FOLDER_APPLICATIONS: &[u8] =
        include_bytes!("../../../assets/icons-classic/folder/Applications.png");
    const FOLDER_ASSISTANT: &[u8] =
        include_bytes!("../../../assets/icons-classic/folder/Assistant.png");
    const FOLDER_COLORSYNC_PROFILES: &[u8] =
        include_bytes!("../../../assets/icons-classic/folder/Colorsync profiles.png");
    const FOLDER_CONTEXTUAL_MENU_ITEMS: &[u8] =
        include_bytes!("../../../assets/icons-classic/folder/Contextual menu items.png");
    const FOLDER_CONTROL_PANELS: &[u8] =
        include_bytes!("../../../assets/icons-classic/folder/Control panels.png");
    const FOLDER_CONTROL_STRIP: &[u8] =
        include_bytes!("../../../assets/icons-classic/folder/Control strip.png");
    const FOLDER_DEFAULT: &[u8] =
        include_bytes!("../../../assets/icons-classic/folder/Default.png");
    const FOLDER_EXTENSIONS: &[u8] =
        include_bytes!("../../../assets/icons-classic/folder/Extensions.png");
    const FOLDER_EXTRAS: &[u8] = include_bytes!("../../../assets/icons-classic/folder/Extras.png");
    const FOLDER_FAVORITES: &[u8] =
        include_bytes!("../../../assets/icons-classic/folder/Favorites.png");
    const FOLDER_FONTS: &[u8] = include_bytes!("../../../assets/icons-classic/folder/Fonts.png");
    const FOLDER_HELP: &[u8] = include_bytes!("../../../assets/icons-classic/folder/Help.png");
    const FOLDER_INTERNET_SEARCH_SITES: &[u8] =
        include_bytes!("../../../assets/icons-classic/folder/Internet search sites.png");
    const FOLDER_INTERNET: &[u8] =
        include_bytes!("../../../assets/icons-classic/folder/Internet.png");
    const FOLDER_LANGUAGE_AND_REGION: &[u8] =
        include_bytes!("../../../assets/icons-classic/folder/Language and region.png");
    const FOLDER_PREFERENCES: &[u8] =
        include_bytes!("../../../assets/icons-classic/folder/Preferences.png");
    const FOLDER_RECENT_DOCUMENTS: &[u8] =
        include_bytes!("../../../assets/icons-classic/folder/Recent documents.png");
    const FOLDER_SCRIPTS: &[u8] =
        include_bytes!("../../../assets/icons-classic/folder/Scripts.png");
    const FOLDER_STARTUP_ITEMS: &[u8] =
        include_bytes!("../../../assets/icons-classic/folder/Startup items.png");
    const FOLDER_SYSTEM: &[u8] = include_bytes!("../../../assets/icons-classic/folder/System.png");
    const FOLDER_TEXT_ENCODINGS: &[u8] =
        include_bytes!("../../../assets/icons-classic/folder/Text encodings.png");
    const FOLDER_UTILITIES: &[u8] =
        include_bytes!("../../../assets/icons-classic/folder/Utilities.png");

    // MENU-BAR
    const MENU_BAR_APPLE_LOGO: &[u8] =
        include_bytes!("../../../assets/icons-classic/menu-bar/Apple logo.png");
    const MENU_BAR_APPLE_SYSTEM: &[u8] =
        include_bytes!("../../../assets/icons-classic/menu-bar/Apple system.png");
    const MENU_BAR_CALCULATOR: &[u8] =
        include_bytes!("../../../assets/icons-classic/menu-bar/Calculator.png");
    const MENU_BAR_CHOOSER: &[u8] =
        include_bytes!("../../../assets/icons-classic/menu-bar/Chooser.png");
    const MENU_BAR_CONTROL_PANELS: &[u8] =
        include_bytes!("../../../assets/icons-classic/menu-bar/Control panels.png");
    const MENU_BAR_FAVORITES: &[u8] =
        include_bytes!("../../../assets/icons-classic/menu-bar/Favorites.png");
    const MENU_BAR_FINDER: &[u8] =
        include_bytes!("../../../assets/icons-classic/menu-bar/Finder.png");
    const MENU_BAR_KEY_CAPS: &[u8] =
        include_bytes!("../../../assets/icons-classic/menu-bar/Key caps.png");
    const MENU_BAR_NETWORK_BROWSER: &[u8] =
        include_bytes!("../../../assets/icons-classic/menu-bar/Network browser.png");
    const MENU_BAR_RECENT_APPLICATIONS: &[u8] =
        include_bytes!("../../../assets/icons-classic/menu-bar/Recent applications.png");
    const MENU_BAR_RECENT_DOCUMENTS: &[u8] =
        include_bytes!("../../../assets/icons-classic/menu-bar/Recent documents.png");
    const MENU_BAR_REMOTE_ACCESS: &[u8] =
        include_bytes!("../../../assets/icons-classic/menu-bar/Remote access.png");
    const MENU_BAR_SCRAPBOOK: &[u8] =
        include_bytes!("../../../assets/icons-classic/menu-bar/Scrapbook.png");
    const MENU_BAR_SHERLOCK_2_0: &[u8] =
        include_bytes!("../../../assets/icons-classic/menu-bar/Sherlock 2.0.png");
    const MENU_BAR_STICKIES: &[u8] =
        include_bytes!("../../../assets/icons-classic/menu-bar/Stickies.png");
    const MENU_BAR_SUITCASE: &[u8] =
        include_bytes!("../../../assets/icons-classic/menu-bar/Suitcase.png");

    // SYSTEM
    const SYSTEM_BLANK_FILE: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Blank file.png");
    const SYSTEM_CSW_6000_SERIES: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/CSW 6000 Series.png");
    const SYSTEM_CLIPBOARD: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Clipboard.png");
    const SYSTEM_COLOR_SW_1500: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Color SW 1500.png");
    const SYSTEM_COLOR_SW_2500: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Color SW 2500.png");
    const SYSTEM_COLOR_SW_PRO: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Color SW Pro.png");
    const SYSTEM_COLOR_PROFILE: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Color profile.png");
    const SYSTEM_DRIVE_SETUP: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Drive setup.png");
    const SYSTEM_EDIT: &[u8] = include_bytes!("../../../assets/icons-classic/system/Edit.png");
    const SYSTEM_FLOPPY_DISK: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Floppy disk.png");
    const SYSTEM_FOLDER_PANELS: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Folder panels.png");
    const SYSTEM_FONT_FILE: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Font file.png");
    const SYSTEM_FONT_SUITCASE: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Font suitcase.png");
    const SYSTEM_HARD_DRIVE_SHARED: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Hard drive (Shared).png");
    const SYSTEM_HARD_DRIVE_EXTERNAL: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Hard drive (external).png");
    const SYSTEM_HARD_DRIVE_FINDER: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Hard drive (finder).png");
    const SYSTEM_HARD_DRIVE_SHARED_FINDER: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Hard drive (shared finder).png");
    const SYSTEM_HARD_DRIVE: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Hard drive.png");
    const SYSTEM_HELP: &[u8] = include_bytes!("../../../assets/icons-classic/system/Help.png");
    const SYSTEM_INTERNET_BROWSE: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Internet Browse.png");
    const SYSTEM_INTERNET_SEARCH: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Internet search.png");
    const SYSTEM_INTERNET_SETUP: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Internet setup.png");
    const SYSTEM_LANGUAGE_FILE: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Language file.png");
    const SYSTEM_LOG: &[u8] = include_bytes!("../../../assets/icons-classic/system/Log.png");
    const SYSTEM_MAP_TCP_FILE: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Map tcp file.png");
    const SYSTEM_NOTEPAD_FILE: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Notepad file.png");
    const SYSTEM_PDF: &[u8] = include_bytes!("../../../assets/icons-classic/system/PDF.png");
    const SYSTEM_QUESTION: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Question.png");
    const SYSTEM_QUICKTIME_MOVIE: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Quicktime movie.png");
    const SYSTEM_REGISTER_WITH_APPLE: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Register with Apple.png");
    const SYSTEM_SCREEN: &[u8] = include_bytes!("../../../assets/icons-classic/system/Screen.png");
    const SYSTEM_SETUP_ASSISTANT: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Setup assistant.png");
    const SYSTEM_SOUND_SETTINGS: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Sound settings.png");
    const SYSTEM_SYSTEM_FOLDER: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/System folder.png");
    const SYSTEM_TEACH_TEXT: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Teach text.png");
    const SYSTEM_TEXT_FILE: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Text file.png");
    const SYSTEM_TRASH_EMPTY: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Trash (Empty).png");
    const SYSTEM_TRASH_FULL: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/Trash (Full).png");
    const SYSTEM_URL_ACCESS: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/URL Access.png");
    const SYSTEM_ITUNES_PLAYLIST: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/iTunes playlist.png");
    const SYSTEM_ITUNES_PLUGIN: &[u8] =
        include_bytes!("../../../assets/icons-classic/system/iTunes plugin.png");

    // ── Blob URL helper ─────────────────────────────────────────

    /// Create a `blob:` URL from raw bytes with the given MIME type.
    ///
    /// The resulting URL is valid for the lifetime of the page. It does
    /// not need to be revoked for fonts that live forever.
    fn create_blob_url(bytes: &[u8], mime_type: &str) -> String {
        let uint8_array = js_sys::Uint8Array::new_with_length(bytes.len() as u32);
        uint8_array.copy_from(bytes);

        let parts = js_sys::Array::new();
        parts.push(&uint8_array);

        let options = web_sys::BlobPropertyBag::new();
        options.set_type(mime_type);

        let blob =
            web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &options).unwrap_throw();

        web_sys::Url::create_object_url_with_blob(&blob).unwrap_throw()
    }

    // ── CSS rewriting ───────────────────────────────────────────

    /// Rewrite Font Awesome CSS to use Blob URLs for embedded fonts.
    ///
    /// Replaces woff2 relative paths with Blob URLs and strips the
    /// ttf fallback entries (we only ship woff2).
    fn rewrite_fontawesome_css(
        css: &str,
        solid_url: &str,
        regular_url: &str,
        v4compat_url: &str,
    ) -> String {
        css
            // Replace woff2 paths with Blob URLs
            .replace("../webfonts/fa-solid-900.woff2", solid_url)
            .replace("../webfonts/fa-regular-400.woff2", regular_url)
            .replace("../webfonts/fa-v4compatibility.woff2", v4compat_url)
            // Strip ttf fallbacks (we only embed woff2)
            .replace(
                ",url(../webfonts/fa-solid-900.ttf) format(\"truetype\")",
                "",
            )
            .replace(
                ",url(../webfonts/fa-regular-400.ttf) format(\"truetype\")",
                "",
            )
            .replace(
                ",url(../webfonts/fa-brands-400.ttf) format(\"truetype\")",
                "",
            )
            .replace(
                ",url(../webfonts/fa-v4compatibility.ttf) format(\"truetype\")",
                "",
            )
    }

    /// Rewrite iti CSS to use Blob URLs for the embedded fonts.
    ///
    /// Replaces the ttf paths for Geneva, ChicagoFLF, and Apple Garamond
    /// with Blob URLs.
    fn rewrite_iti_css(
        css: &str,
        chicago_url: &str,
        geneva_url: &str,
        garamond_light_url: &str,
        garamond_regular_url: &str,
        garamond_bold_url: &str,
    ) -> String {
        css.replace(
            "url('fonts/ChicagoFLF.ttf')",
            &format!("url(\"{chicago_url}\")"),
        )
        .replace("url('fonts/Geneva.ttf')", &format!("url(\"{geneva_url}\")"))
        .replace(
            "url('fonts/AppleGaramond-Light.ttf')",
            &format!("url(\"{garamond_light_url}\")"),
        )
        .replace(
            "url('fonts/AppleGaramond.ttf')",
            &format!("url(\"{garamond_regular_url}\")"),
        )
        .replace(
            "url('fonts/AppleGaramond-Bold.ttf')",
            &format!("url(\"{garamond_bold_url}\")"),
        )
    }

    // ── Public API ──────────────────────────────────────────────

    /// Inject all required styles from the embedded assets.
    ///
    /// Creates `<style>` elements in `<head>` — no `<link>` tags,
    /// no network requests:
    ///
    /// 1. Color token CSS custom properties (from `color.rs`)
    /// 2. iti unified CSS (with `@font-face` rewritten to Blob URLs)
    /// 3. Font Awesome 6 CSS (with `@font-face` rewritten to Blob URLs)
    ///
    /// Font Awesome Brands icons are **not** embedded to save binary
    /// space. Brand icon classes (`.fa-brands`) will render as blank
    /// unless the consumer loads the Brands font separately.
    pub fn inject_styles() {
        // Create Blob URLs for each embedded font
        let fa_solid_url = create_blob_url(FA_SOLID_WOFF2, "font/woff2");
        let fa_regular_url = create_blob_url(FA_REGULAR_WOFF2, "font/woff2");
        let fa_v4compat_url = create_blob_url(FA_V4COMPAT_WOFF2, "font/woff2");
        let chicago_url = create_blob_url(CHICAGO_TTF, "font/ttf");
        let geneva_url = create_blob_url(GENEVA_TTF, "font/ttf");
        let garamond_light_url = create_blob_url(GARAMOND_LIGHT_TTF, "font/ttf");
        let garamond_regular_url = create_blob_url(GARAMOND_REGULAR_TTF, "font/ttf");
        let garamond_bold_url = create_blob_url(GARAMOND_BOLD_TTF, "font/ttf");

        // Rewrite CSS @font-face declarations to use Blob URLs
        let fa_css = rewrite_fontawesome_css(
            FONTAWESOME_CSS,
            &fa_solid_url,
            &fa_regular_url,
            &fa_v4compat_url,
        );
        let iti_css = rewrite_iti_css(
            ITI_CSS,
            &chicago_url,
            &geneva_url,
            &garamond_light_url,
            &garamond_regular_url,
            &garamond_bold_url,
        );

        // Inject everything as <style> elements — zero network requests
        inject_color_tokens();
        append_style(&iti_css);
        append_style(&fa_css);
    }

    /// Get a Blob URL for a classic Mac OS icon.
    ///
    /// Uses a lazy static cache to avoid recreating Blob URLs for the same
    /// icon multiple times. Icons are loaded on-demand from the compiled
    /// constants via the embedded icon lookup.
    pub fn blob_url_for_classic_icon(
        glyph: &crate::components::icon_classic::IconClassicGlyph,
    ) -> String {
        use std::collections::HashMap;
        use std::sync::OnceLock;

        static CLASSIC_ICON_CACHE: OnceLock<std::sync::Mutex<HashMap<String, String>>> =
            OnceLock::new();

        let cache = CLASSIC_ICON_CACHE.get_or_init(|| std::sync::Mutex::new(HashMap::new()));
        let mut cache_map = cache.lock().unwrap();

        let key = format!("{}/{}", glyph.category(), glyph.filename());

        // Check if already cached
        if let Some(cached_url) = cache_map.get(&key) {
            return cached_url.clone();
        }

        // Load the icon from compiled constants
        if let Some(bytes) = get_classic_icon_bytes(glyph) {
            let blob_url = create_blob_url(bytes, "image/png");
            cache_map.insert(key, blob_url.clone());
            blob_url
        } else {
            // Fallback (should not happen with correct icon definitions)
            format!("/icons-classic/{}/{}", glyph.category(), glyph.filename())
        }
    }

    /// Get embedded classic icon bytes by glyph.
    fn get_classic_icon_bytes(
        glyph: &crate::components::icon_classic::IconClassicGlyph,
    ) -> Option<&'static [u8]> {
        match glyph {
            crate::components::icon_classic::IconClassicGlyph::Applications(
                crate::components::icon_classic::ApplicationsIcon::AdobeIllustrator55,
            ) => Some(APPLICATIONS_ADOBE_ILLUSTRATOR_5_5),
            crate::components::icon_classic::IconClassicGlyph::Applications(
                crate::components::icon_classic::ApplicationsIcon::AdobePhotoshop50,
            ) => Some(APPLICATIONS_ADOBE_PHOTOSHOP_5_0),
            crate::components::icon_classic::IconClassicGlyph::Applications(
                crate::components::icon_classic::ApplicationsIcon::AppleFmRadio,
            ) => Some(APPLICATIONS_APPLE_FM_RADIO),
            crate::components::icon_classic::IconClassicGlyph::Applications(
                crate::components::icon_classic::ApplicationsIcon::AppleFileecurity,
            ) => Some(APPLICATIONS_APPLE_FILE_SECURITY),
            crate::components::icon_classic::IconClassicGlyph::Applications(
                crate::components::icon_classic::ApplicationsIcon::AppleSharePrep,
            ) => Some(APPLICATIONS_APPLE_SHARE_PREP),
            crate::components::icon_classic::IconClassicGlyph::Applications(
                crate::components::icon_classic::ApplicationsIcon::AppleVerifier,
            ) => Some(APPLICATIONS_APPLE_VERIFIER),
            crate::components::icon_classic::IconClassicGlyph::Applications(
                crate::components::icon_classic::ApplicationsIcon::Calculator,
            ) => Some(APPLICATIONS_CALCULATOR),
            crate::components::icon_classic::IconClassicGlyph::Applications(
                crate::components::icon_classic::ApplicationsIcon::DiskCopy,
            ) => Some(APPLICATIONS_DISK_COPY),
            crate::components::icon_classic::IconClassicGlyph::Applications(
                crate::components::icon_classic::ApplicationsIcon::DiskFirstAid,
            ) => Some(APPLICATIONS_DISK_FIRST_AID),
            crate::components::icon_classic::IconClassicGlyph::Applications(
                crate::components::icon_classic::ApplicationsIcon::GraphingCalculator,
            ) => Some(APPLICATIONS_GRAPHING_CALCULATOR),
            crate::components::icon_classic::IconClassicGlyph::Applications(
                crate::components::icon_classic::ApplicationsIcon::KeyCaps,
            ) => Some(APPLICATIONS_KEY_CAPS),
            crate::components::icon_classic::IconClassicGlyph::Applications(
                crate::components::icon_classic::ApplicationsIcon::Mail,
            ) => Some(APPLICATIONS_MAIL),
            crate::components::icon_classic::IconClassicGlyph::Applications(
                crate::components::icon_classic::ApplicationsIcon::MicrosoftInternetExplorer,
            ) => Some(APPLICATIONS_MICROSOFT_INTERNET_EXPLORER),
            crate::components::icon_classic::IconClassicGlyph::Applications(
                crate::components::icon_classic::ApplicationsIcon::Notepad,
            ) => Some(APPLICATIONS_NOTEPAD),
            crate::components::icon_classic::IconClassicGlyph::Applications(
                crate::components::icon_classic::ApplicationsIcon::QuicktimePlayer,
            ) => Some(APPLICATIONS_QUICKTIME_PLAYER),
            crate::components::icon_classic::IconClassicGlyph::Applications(
                crate::components::icon_classic::ApplicationsIcon::Scrapbook,
            ) => Some(APPLICATIONS_SCRAPBOOK),
            crate::components::icon_classic::IconClassicGlyph::Applications(
                crate::components::icon_classic::ApplicationsIcon::ScriptEditor,
            ) => Some(APPLICATIONS_SCRIPT_EDITOR),
            crate::components::icon_classic::IconClassicGlyph::Applications(
                crate::components::icon_classic::ApplicationsIcon::Sherlock20,
            ) => Some(APPLICATIONS_SHERLOCK_2_0),
            crate::components::icon_classic::IconClassicGlyph::Applications(
                crate::components::icon_classic::ApplicationsIcon::Stickies,
            ) => Some(APPLICATIONS_STICKIES),
            crate::components::icon_classic::IconClassicGlyph::Applications(
                crate::components::icon_classic::ApplicationsIcon::Stuffitexpander,
            ) => Some(APPLICATIONS_STUFFIT_EXPANDER),
            crate::components::icon_classic::IconClassicGlyph::Applications(
                crate::components::icon_classic::ApplicationsIcon::Website,
            ) => Some(APPLICATIONS_WEBSITE),
            crate::components::icon_classic::IconClassicGlyph::Applications(
                crate::components::icon_classic::ApplicationsIcon::Itunes,
            ) => Some(APPLICATIONS_ITUNES),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::AdobeGamma,
            ) => Some(CONTROL_PANEL_ADOBE_GAMMA),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::Appearance,
            ) => Some(CONTROL_PANEL_APPEARANCE),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::AppleMenuOptions,
            ) => Some(CONTROL_PANEL_APPLE_MENU_OPTIONS),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::Appletalk,
            ) => Some(CONTROL_PANEL_APPLETALK),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::Atm,
            ) => Some(CONTROL_PANEL_ATM),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::Colorsync,
            ) => Some(CONTROL_PANEL_COLORSYNC),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::ControlStrip,
            ) => Some(CONTROL_PANEL_CONTROL_STRIP),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::DateAndTime,
            ) => Some(CONTROL_PANEL_DATE_AND_TIME),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::DialAssist,
            ) => Some(CONTROL_PANEL_DIAL_ASSIST),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::EnergySaver,
            ) => Some(CONTROL_PANEL_ENERGY_SAVER),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::ExtensionsManager,
            ) => Some(CONTROL_PANEL_EXTENSIONS_MANAGER),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::FileExchange,
            ) => Some(CONTROL_PANEL_FILE_EXCHANGE),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::FileSharing,
            ) => Some(CONTROL_PANEL_FILE_SHARING),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::GeneralControls,
            ) => Some(CONTROL_PANEL_GENERAL_CONTROLS),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::Internet,
            ) => Some(CONTROL_PANEL_INTERNET),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::Keyboard,
            ) => Some(CONTROL_PANEL_KEYBOARD),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::KeychainAccess,
            ) => Some(CONTROL_PANEL_KEYCHAIN_ACCESS),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::Launcher,
            ) => Some(CONTROL_PANEL_LAUNCHER),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::LocationManager,
            ) => Some(CONTROL_PANEL_LOCATION_MANAGER),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::Memory,
            ) => Some(CONTROL_PANEL_MEMORY),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::Modem,
            ) => Some(CONTROL_PANEL_MODEM),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::Monitor,
            ) => Some(CONTROL_PANEL_MONITOR),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::Mouse,
            ) => Some(CONTROL_PANEL_MOUSE),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::MultipleUsers,
            ) => Some(CONTROL_PANEL_MULTIPLE_USERS),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::Numbers,
            ) => Some(CONTROL_PANEL_NUMBERS),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::QuicktimeSettings,
            ) => Some(CONTROL_PANEL_QUICKTIME_SETTINGS),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::RemoteAccess,
            ) => Some(CONTROL_PANEL_REMOTE_ACCESS),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::SoftwareUpdate,
            ) => Some(CONTROL_PANEL_SOFTWARE_UPDATE),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::Sound,
            ) => Some(CONTROL_PANEL_SOUND),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::Speech,
            ) => Some(CONTROL_PANEL_SPEECH),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::StartupDisk,
            ) => Some(CONTROL_PANEL_STARTUP_DISK),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::Tcpip,
            ) => Some(CONTROL_PANEL_TCPIP),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::Text,
            ) => Some(CONTROL_PANEL_TEXT),
            crate::components::icon_classic::IconClassicGlyph::ControlPanel(
                crate::components::icon_classic::ControlPanelIcon::WebSharing,
            ) => Some(CONTROL_PANEL_WEB_SHARING),
            crate::components::icon_classic::IconClassicGlyph::ControlStrip(
                crate::components::icon_classic::ControlStripIcon::AppleLocation,
            ) => Some(CONTROL_STRIP_APPLE_LOCATION),
            crate::components::icon_classic::IconClassicGlyph::ControlStrip(
                crate::components::icon_classic::ControlStripIcon::AppleTalk,
            ) => Some(CONTROL_STRIP_APPLE_TALK),
            crate::components::icon_classic::IconClassicGlyph::ControlStrip(
                crate::components::icon_classic::ControlStripIcon::Cd,
            ) => Some(CONTROL_STRIP_CD),
            crate::components::icon_classic::IconClassicGlyph::ControlStrip(
                crate::components::icon_classic::ControlStripIcon::FileSharing,
            ) => Some(CONTROL_STRIP_FILE_SHARING),
            crate::components::icon_classic::IconClassicGlyph::ControlStrip(
                crate::components::icon_classic::ControlStripIcon::KeychainStrip,
            ) => Some(CONTROL_STRIP_KEYCHAIN_STRIP),
            crate::components::icon_classic::IconClassicGlyph::ControlStrip(
                crate::components::icon_classic::ControlStripIcon::MonitorBitdepth,
            ) => Some(CONTROL_STRIP_MONITOR_BITDEPTH),
            crate::components::icon_classic::IconClassicGlyph::ControlStrip(
                crate::components::icon_classic::ControlStripIcon::MonitorResolution,
            ) => Some(CONTROL_STRIP_MONITOR_RESOLUTION),
            crate::components::icon_classic::IconClassicGlyph::ControlStrip(
                crate::components::icon_classic::ControlStripIcon::Printer,
            ) => Some(CONTROL_STRIP_PRINTER),
            crate::components::icon_classic::IconClassicGlyph::ControlStrip(
                crate::components::icon_classic::ControlStripIcon::RemoteAccess,
            ) => Some(CONTROL_STRIP_REMOTE_ACCESS),
            crate::components::icon_classic::IconClassicGlyph::ControlStrip(
                crate::components::icon_classic::ControlStripIcon::SoundVolume,
            ) => Some(CONTROL_STRIP_SOUND_VOLUME),
            crate::components::icon_classic::IconClassicGlyph::ControlStrip(
                crate::components::icon_classic::ControlStripIcon::WebSharing,
            ) => Some(CONTROL_STRIP_WEB_SHARING),
            crate::components::icon_classic::IconClassicGlyph::ControlStrip(
                crate::components::icon_classic::ControlStripIcon::Itunes,
            ) => Some(CONTROL_STRIP_ITUNES),
            crate::components::icon_classic::IconClassicGlyph::Folder(
                crate::components::icon_classic::FolderIcon::AppleMenuItem,
            ) => Some(FOLDER_APPLE_MENU_ITEM),
            crate::components::icon_classic::IconClassicGlyph::Folder(
                crate::components::icon_classic::FolderIcon::ApplicationSupport,
            ) => Some(FOLDER_APPLICATION_SUPPORT),
            crate::components::icon_classic::IconClassicGlyph::Folder(
                crate::components::icon_classic::FolderIcon::Applications,
            ) => Some(FOLDER_APPLICATIONS),
            crate::components::icon_classic::IconClassicGlyph::Folder(
                crate::components::icon_classic::FolderIcon::Assistant,
            ) => Some(FOLDER_ASSISTANT),
            crate::components::icon_classic::IconClassicGlyph::Folder(
                crate::components::icon_classic::FolderIcon::ColorSyncprofiles,
            ) => Some(FOLDER_COLORSYNC_PROFILES),
            crate::components::icon_classic::IconClassicGlyph::Folder(
                crate::components::icon_classic::FolderIcon::ContextualMenuItems,
            ) => Some(FOLDER_CONTEXTUAL_MENU_ITEMS),
            crate::components::icon_classic::IconClassicGlyph::Folder(
                crate::components::icon_classic::FolderIcon::ControlPanels,
            ) => Some(FOLDER_CONTROL_PANELS),
            crate::components::icon_classic::IconClassicGlyph::Folder(
                crate::components::icon_classic::FolderIcon::ControlStrip,
            ) => Some(FOLDER_CONTROL_STRIP),
            crate::components::icon_classic::IconClassicGlyph::Folder(
                crate::components::icon_classic::FolderIcon::Default,
            ) => Some(FOLDER_DEFAULT),
            crate::components::icon_classic::IconClassicGlyph::Folder(
                crate::components::icon_classic::FolderIcon::Extensions,
            ) => Some(FOLDER_EXTENSIONS),
            crate::components::icon_classic::IconClassicGlyph::Folder(
                crate::components::icon_classic::FolderIcon::Extras,
            ) => Some(FOLDER_EXTRAS),
            crate::components::icon_classic::IconClassicGlyph::Folder(
                crate::components::icon_classic::FolderIcon::Favorites,
            ) => Some(FOLDER_FAVORITES),
            crate::components::icon_classic::IconClassicGlyph::Folder(
                crate::components::icon_classic::FolderIcon::Fonts,
            ) => Some(FOLDER_FONTS),
            crate::components::icon_classic::IconClassicGlyph::Folder(
                crate::components::icon_classic::FolderIcon::Help,
            ) => Some(FOLDER_HELP),
            crate::components::icon_classic::IconClassicGlyph::Folder(
                crate::components::icon_classic::FolderIcon::InternetSearchSites,
            ) => Some(FOLDER_INTERNET_SEARCH_SITES),
            crate::components::icon_classic::IconClassicGlyph::Folder(
                crate::components::icon_classic::FolderIcon::Internet,
            ) => Some(FOLDER_INTERNET),
            crate::components::icon_classic::IconClassicGlyph::Folder(
                crate::components::icon_classic::FolderIcon::LanguageAndRegion,
            ) => Some(FOLDER_LANGUAGE_AND_REGION),
            crate::components::icon_classic::IconClassicGlyph::Folder(
                crate::components::icon_classic::FolderIcon::Preferences,
            ) => Some(FOLDER_PREFERENCES),
            crate::components::icon_classic::IconClassicGlyph::Folder(
                crate::components::icon_classic::FolderIcon::RecentDocuments,
            ) => Some(FOLDER_RECENT_DOCUMENTS),
            crate::components::icon_classic::IconClassicGlyph::Folder(
                crate::components::icon_classic::FolderIcon::Scripts,
            ) => Some(FOLDER_SCRIPTS),
            crate::components::icon_classic::IconClassicGlyph::Folder(
                crate::components::icon_classic::FolderIcon::StartupItems,
            ) => Some(FOLDER_STARTUP_ITEMS),
            crate::components::icon_classic::IconClassicGlyph::Folder(
                crate::components::icon_classic::FolderIcon::System,
            ) => Some(FOLDER_SYSTEM),
            crate::components::icon_classic::IconClassicGlyph::Folder(
                crate::components::icon_classic::FolderIcon::TextEncodings,
            ) => Some(FOLDER_TEXT_ENCODINGS),
            crate::components::icon_classic::IconClassicGlyph::Folder(
                crate::components::icon_classic::FolderIcon::Utilities,
            ) => Some(FOLDER_UTILITIES),
            crate::components::icon_classic::IconClassicGlyph::MenuBar(
                crate::components::icon_classic::MenuBarIcon::AppleLogo,
            ) => Some(MENU_BAR_APPLE_LOGO),
            crate::components::icon_classic::IconClassicGlyph::MenuBar(
                crate::components::icon_classic::MenuBarIcon::AppleSystem,
            ) => Some(MENU_BAR_APPLE_SYSTEM),
            crate::components::icon_classic::IconClassicGlyph::MenuBar(
                crate::components::icon_classic::MenuBarIcon::Calculator,
            ) => Some(MENU_BAR_CALCULATOR),
            crate::components::icon_classic::IconClassicGlyph::MenuBar(
                crate::components::icon_classic::MenuBarIcon::Chooser,
            ) => Some(MENU_BAR_CHOOSER),
            crate::components::icon_classic::IconClassicGlyph::MenuBar(
                crate::components::icon_classic::MenuBarIcon::ControlPanels,
            ) => Some(MENU_BAR_CONTROL_PANELS),
            crate::components::icon_classic::IconClassicGlyph::MenuBar(
                crate::components::icon_classic::MenuBarIcon::Favorites,
            ) => Some(MENU_BAR_FAVORITES),
            crate::components::icon_classic::IconClassicGlyph::MenuBar(
                crate::components::icon_classic::MenuBarIcon::Finder,
            ) => Some(MENU_BAR_FINDER),
            crate::components::icon_classic::IconClassicGlyph::MenuBar(
                crate::components::icon_classic::MenuBarIcon::KeyCaps,
            ) => Some(MENU_BAR_KEY_CAPS),
            crate::components::icon_classic::IconClassicGlyph::MenuBar(
                crate::components::icon_classic::MenuBarIcon::NetworkBrowser,
            ) => Some(MENU_BAR_NETWORK_BROWSER),
            crate::components::icon_classic::IconClassicGlyph::MenuBar(
                crate::components::icon_classic::MenuBarIcon::RecentApplications,
            ) => Some(MENU_BAR_RECENT_APPLICATIONS),
            crate::components::icon_classic::IconClassicGlyph::MenuBar(
                crate::components::icon_classic::MenuBarIcon::RecentDocuments,
            ) => Some(MENU_BAR_RECENT_DOCUMENTS),
            crate::components::icon_classic::IconClassicGlyph::MenuBar(
                crate::components::icon_classic::MenuBarIcon::RemoteAccess,
            ) => Some(MENU_BAR_REMOTE_ACCESS),
            crate::components::icon_classic::IconClassicGlyph::MenuBar(
                crate::components::icon_classic::MenuBarIcon::Scrapbook,
            ) => Some(MENU_BAR_SCRAPBOOK),
            crate::components::icon_classic::IconClassicGlyph::MenuBar(
                crate::components::icon_classic::MenuBarIcon::Sherlock20,
            ) => Some(MENU_BAR_SHERLOCK_2_0),
            crate::components::icon_classic::IconClassicGlyph::MenuBar(
                crate::components::icon_classic::MenuBarIcon::Stickies,
            ) => Some(MENU_BAR_STICKIES),
            crate::components::icon_classic::IconClassicGlyph::MenuBar(
                crate::components::icon_classic::MenuBarIcon::Suitcase,
            ) => Some(MENU_BAR_SUITCASE),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::BlankFile,
            ) => Some(SYSTEM_BLANK_FILE),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::Csw6000Series,
            ) => Some(SYSTEM_CSW_6000_SERIES),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::Clipboard,
            ) => Some(SYSTEM_CLIPBOARD),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::ColorSw1500,
            ) => Some(SYSTEM_COLOR_SW_1500),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::ColorSw2500,
            ) => Some(SYSTEM_COLOR_SW_2500),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::ColorSwPro,
            ) => Some(SYSTEM_COLOR_SW_PRO),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::ColorProfile,
            ) => Some(SYSTEM_COLOR_PROFILE),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::DriveSetup,
            ) => Some(SYSTEM_DRIVE_SETUP),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::Edit,
            ) => Some(SYSTEM_EDIT),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::FloppyDisk,
            ) => Some(SYSTEM_FLOPPY_DISK),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::FolderPanels,
            ) => Some(SYSTEM_FOLDER_PANELS),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::FontFile,
            ) => Some(SYSTEM_FONT_FILE),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::FontSuitcase,
            ) => Some(SYSTEM_FONT_SUITCASE),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::HardDriveShared,
            ) => Some(SYSTEM_HARD_DRIVE_SHARED),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::HardDriveExternal,
            ) => Some(SYSTEM_HARD_DRIVE_EXTERNAL),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::HardDriveFinder,
            ) => Some(SYSTEM_HARD_DRIVE_FINDER),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::HardDriveSharedFinder,
            ) => Some(SYSTEM_HARD_DRIVE_SHARED_FINDER),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::HardDrive,
            ) => Some(SYSTEM_HARD_DRIVE),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::Help,
            ) => Some(SYSTEM_HELP),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::InternetBrowse,
            ) => Some(SYSTEM_INTERNET_BROWSE),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::InternetSearch,
            ) => Some(SYSTEM_INTERNET_SEARCH),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::InternetSetup,
            ) => Some(SYSTEM_INTERNET_SETUP),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::LanguageFile,
            ) => Some(SYSTEM_LANGUAGE_FILE),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::Log,
            ) => Some(SYSTEM_LOG),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::MapTcpFile,
            ) => Some(SYSTEM_MAP_TCP_FILE),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::NotepadFile,
            ) => Some(SYSTEM_NOTEPAD_FILE),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::Pdf,
            ) => Some(SYSTEM_PDF),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::Question,
            ) => Some(SYSTEM_QUESTION),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::QuicktimeMovie,
            ) => Some(SYSTEM_QUICKTIME_MOVIE),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::RegisterWithApple,
            ) => Some(SYSTEM_REGISTER_WITH_APPLE),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::Screen,
            ) => Some(SYSTEM_SCREEN),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::SetupAssistant,
            ) => Some(SYSTEM_SETUP_ASSISTANT),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::SoundSettings,
            ) => Some(SYSTEM_SOUND_SETTINGS),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::SystemFolder,
            ) => Some(SYSTEM_SYSTEM_FOLDER),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::TeachText,
            ) => Some(SYSTEM_TEACH_TEXT),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::TextFile,
            ) => Some(SYSTEM_TEXT_FILE),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::TrashEmpty,
            ) => Some(SYSTEM_TRASH_EMPTY),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::TrashFull,
            ) => Some(SYSTEM_TRASH_FULL),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::UrlAccess,
            ) => Some(SYSTEM_URL_ACCESS),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::ItunesPlaylist,
            ) => Some(SYSTEM_ITUNES_PLAYLIST),
            crate::components::icon_classic::IconClassicGlyph::System(
                crate::components::icon_classic::SystemIcon::ItunesPlugin,
            ) => Some(SYSTEM_ITUNES_PLUGIN),
        }
    }
}
