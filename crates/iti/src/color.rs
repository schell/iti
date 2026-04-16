//! Design token color palette.
//!
//! All color hex values for the iti design system are defined here as Rust
//! constants. This module is the **single source of truth** for the palette.
//!
//! The corresponding CSS custom properties (e.g. `--azul`, `--gray300`) are
//! generated at compile time by [`CSS_TOKENS`] and prepended to the
//! stylesheet at runtime by [`crate::assets`]. Semantic aliases that
//! reference these tokens via `var()` remain in `iti.css`.
//!
//! ## Naming
//!
//! Constants follow the Figma token names in UPPER_SNAKE_CASE. The CSS
//! custom property names are the kebab-case equivalents (e.g.
//! `BLACK900` → `--black900`).

// ── Primary ─────────────────────────────────────────────────────────

/// Black — primary text, borders, icons.
pub const BLACK900: &str = "#000000";

/// White — bevel highlights, selection text on dark backgrounds.
pub const WHITE100: &str = "#FFFFFF";

// ── Grays ───────────────────────────────────────────────────────────

/// Lightest gray — card bodies, alert backgrounds, input fields.
pub const GRAY200: &str = "#EEEEEE";

/// Light gray — body background (Platinum base).
pub const GRAY300: &str = "#DDDDDD";

/// Medium gray — card headers, button faces, pinstripe bars.
pub const GRAY400: &str = "#CCCCCC";

/// Dark-medium gray — active/pressed button states.
pub const GRAY500: &str = "#BBBBBB";

/// Mid gray — secondary color, muted elements.
pub const GRAY600: &str = "#999999";

/// Dark gray — bevel shadow edges.
pub const GRAY700: &str = "#808080";

/// Darkest gray — muted text, dark flavor.
pub const GRAY800: &str = "#666666";

// ── Colors ──────────────────────────────────────────────────────────

/// Deep indigo-blue — primary accent, selection highlight.
pub const AZUL: &str = "#333399";

/// Soft purple-blue — hover/highlight, slider thumbs, checkboxes.
pub const LAVENDER: &str = "#CCCCFF";

/// Warm pale yellow — complement to lavender, warning tint.
pub const CREAM: &str = "#FFFFCC";

/// Light purple/violet — primary button tint, thumb highlight.
pub const THISTLE: &str = "#E6CCFF";

/// Light sky blue — info flavor, analogous to lavender.
pub const ICE: &str = "#CCE6FF";

// ── Semantic flavor colors (not derived from var()) ─────────────────

/// Green — success indicators.
pub const SUCCESS: &str = "#339933";

/// Red — danger/error indicators.
pub const DANGER: &str = "#CC3333";

/// Amber — warning indicators.
pub const WARNING: &str = "#CC9933";

// ── Generated CSS ───────────────────────────────────────────────────

/// CSS `:root` block containing all design token custom properties.
///
/// This string is generated at compile time from the constants above
/// and prepended to the stylesheet by [`crate::assets`]. Semantic
/// aliases that use `var()` or `color-mix()` are defined separately
/// in `iti.css`.
pub const CSS_TOKENS: &str = concat!(
    ":root {\n",
    "\t/* ── Figma Design Tokens (generated from color.rs) ───── */\n",
    "\n",
    "\t/* Primary */\n",
    "\t--black900:  ",
    "#000000",
    ";\n",
    "\t--white100:  ",
    "#FFFFFF",
    ";\n",
    "\n",
    "\t/* Grays */\n",
    "\t--gray200:   ",
    "#EEEEEE",
    ";\n",
    "\t--gray300:   ",
    "#DDDDDD",
    ";\n",
    "\t--gray400:   ",
    "#CCCCCC",
    ";\n",
    "\t--gray500:   ",
    "#BBBBBB",
    ";\n",
    "\t--gray600:   ",
    "#999999",
    ";\n",
    "\t--gray700:   ",
    "#808080",
    ";\n",
    "\t--gray800:   ",
    "#666666",
    ";\n",
    "\n",
    "\t/* Colors */\n",
    "\t--azul:      ",
    "#333399",
    ";\n",
    "\t--lavender:  ",
    "#CCCCFF",
    ";\n",
    "\t--cream:     ",
    "#FFFFCC",
    ";\n",
    "\t--thistle:   ",
    "#E6CCFF",
    ";\n",
    "\t--ice:       ",
    "#CCE6FF",
    ";\n",
    "\n",
    "\t/* Flavors */\n",
    "\t--iti-success: ",
    "#339933",
    ";\n",
    "\t--iti-danger:  ",
    "#CC3333",
    ";\n",
    "\t--iti-warning: ",
    "#CC9933",
    ";\n",
    "}\n",
);
