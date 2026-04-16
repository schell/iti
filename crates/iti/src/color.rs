//! Design token color palette.
//!
//! All color hex values for the iti design system are defined here as Rust
//! constants. This module is the **single source of truth** for the palette.
//!
//! The corresponding CSS custom properties (e.g. `--azul`, `--gray300`) and
//! background color utility classes (e.g. `.bg-azul`, `.bg-gray300`) are
//! generated at runtime by [`css_tokens`] and prepended to the stylesheet
//! by [`crate::assets`]. Semantic aliases that reference these tokens via
//! `var()` remain in `iti.css`.
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

/// Darker purple-violet.
pub const PURPLE: &str = "#7B61FF";

/// Warm pale yellow — complement to lavender, warning tint.
pub const CREAM: &str = "#FFFFCC";

/// Light purple/violet — primary button tint, thumb highlight.
pub const THISTLE: &str = "#E6CCFF";

/// Light sky blue — info flavor, analogous to lavender.
pub const ICE: &str = "#CCE6FF";

// ── UI Chrome ───────────────────────────────────────────────────────

/// Near-black — button borders, text, pressed shadows.
pub const CHARCOAL: &str = "#262626";

/// Muted gray — disabled border and text.
pub const DISABLED_GRAY: &str = "#9D9D9D";

/// Primary button ring — lighter gradient band.
pub const RING_LIGHT: &str = "#B7B7B7";

/// Primary button ring — mid gradient band.
pub const RING_MID: &str = "#D0D0D0";

/// Primary button ring — shadow edges and corners.
pub const RING_SHADOW: &str = "#848484";

// ── Semantic flavor colors (not derived from var()) ─────────────────

/// Green — success indicators.
pub const SUCCESS: &str = "#339933";

/// Red — danger/error indicators.
pub const DANGER: &str = "#CC3333";

/// Amber — warning indicators.
pub const WARNING: &str = "#CC9933";

// ── Generated CSS ───────────────────────────────────────────────────

/// Generate a CSS string containing:
///
/// 1. A `:root` block with all design token custom properties.
/// 2. Background color utility classes (`.bg-black900`, `.bg-azul`, etc.)
///    for each token.
///
/// This string is prepended to the stylesheet by [`crate::assets`].
/// Semantic aliases that use `var()` or `color-mix()` are defined
/// separately in `iti.css`.
pub fn css_tokens() -> String {
    format!(
        "\
:root {{
\t/* ── Figma Design Tokens (generated from color.rs) ───── */

\t/* Primary */
\t--black900:  {BLACK900};
\t--white100:  {WHITE100};

\t/* Grays */
\t--gray200:   {GRAY200};
\t--gray300:   {GRAY300};
\t--gray400:   {GRAY400};
\t--gray500:   {GRAY500};
\t--gray600:   {GRAY600};
\t--gray700:   {GRAY700};
\t--gray800:   {GRAY800};

\t/* Colors */
\t--azul:      {AZUL};
\t--lavender:  {LAVENDER};
\t--purple:    {PURPLE};
\t--cream:     {CREAM};
\t--thistle:   {THISTLE};
\t--ice:       {ICE};

	/* UI Chrome */
	--charcoal:      {CHARCOAL};
	--disabled-gray: {DISABLED_GRAY};
	--ring-light:    {RING_LIGHT};
	--ring-mid:      {RING_MID};
	--ring-shadow:   {RING_SHADOW};

	/* Flavors */
	--iti-success: {SUCCESS};
	--iti-danger:  {DANGER};
	--iti-warning: {WARNING};
}}

/* ── Background color utilities (generated from color.rs) ───── */

.bg-black900  {{ background-color: {BLACK900} !important; }}
.bg-white100  {{ background-color: {WHITE100} !important; }}
.bg-gray200   {{ background-color: {GRAY200} !important; }}
.bg-gray300   {{ background-color: {GRAY300} !important; }}
.bg-gray400   {{ background-color: {GRAY400} !important; }}
.bg-gray500   {{ background-color: {GRAY500} !important; }}
.bg-gray600   {{ background-color: {GRAY600} !important; }}
.bg-gray700   {{ background-color: {GRAY700} !important; }}
.bg-gray800   {{ background-color: {GRAY800} !important; }}
.bg-charcoal  {{ background-color: {CHARCOAL} !important; }}
.bg-disabled  {{ background-color: {DISABLED_GRAY} !important; }}
.bg-azul      {{ background-color: {AZUL} !important; }}
.bg-lavender  {{ background-color: {LAVENDER} !important; }}
.bg-cream     {{ background-color: {CREAM} !important; }}
.bg-thistle   {{ background-color: {THISTLE} !important; }}
.bg-ice       {{ background-color: {ICE} !important; }}
.bg-success   {{ background-color: {SUCCESS} !important; }}
.bg-danger    {{ background-color: {DANGER} !important; }}
.bg-warning   {{ background-color: {WARNING} !important; }}
"
    )
}
