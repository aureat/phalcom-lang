//! Semantic-role styling substrate for Phalcom diagnostics.
//!
//! [PDR-0014] rules that the diagnostic renderer is built in-house.
//! All SGR (ANSI escape) emission lives in [`Styler::paint`].

use std::borrow::Cow;
use std::io::IsTerminal;

/// One of the seven colors this renderer is allowed to emit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnsiColor {
    /// The terminal's own default foreground (SGR `39`) — adapts to the user's theme.
    Default,
    /// ANSI red (SGR `31`) — `severity.error`, `span.primary`.
    Red,
    /// ANSI yellow (SGR `33`) — `severity.warn`.
    Yellow,
    /// ANSI blue (SGR `34`) — `location`, `span.secondary`.
    Blue,
    /// ANSI magenta (SGR `35`) — `chain` (fiber/cause boundary links).
    Magenta,
    /// ANSI cyan (SGR `36`) — `severity.help`.
    Cyan,
}

impl AnsiColor {
    /// Returns this color's SGR foreground code (`30`–`37`/`39` range).
    fn sgr_code(self) -> u8 {
        match self {
            AnsiColor::Default => 39,
            AnsiColor::Red => 31,
            AnsiColor::Yellow => 33,
            AnsiColor::Blue => 34,
            AnsiColor::Magenta => 35,
            AnsiColor::Cyan => 36,
        }
    }
}

/// The SGR text attributes a [`Role`] may compose, independent of color.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Weight {
    /// SGR `1` — used for severity markers, primary spans, and identifiers.
    pub bold: bool,
    /// SGR `2` — used for rails, gutters, and line numbers, which should recede visually.
    pub dim: bool,
    /// SGR `3` — used only in combination with `dim`, for elided-content notices.
    pub italic: bool,
}

impl Weight {
    /// No attributes — plain text at the role's color (or the terminal default).
    pub const NORMAL: Weight = Weight {
        bold: false,
        dim: false,
        italic: false,
    };
    /// Bold only.
    pub const BOLD: Weight = Weight {
        bold: true,
        dim: false,
        italic: false,
    };
    /// Dim only.
    pub const DIM: Weight = Weight {
        bold: false,
        dim: true,
        italic: false,
    };
    /// Dim and italic together — reserved for the `elision` role.
    pub const DIM_ITALIC: Weight = Weight {
        bold: false,
        dim: true,
        italic: true,
    };
}

/// The closed set of semantic roles a diagnostic surface may paint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// `error:` prefixes and the `×` marker. Bold red.
    SeverityError,
    /// `warning:` prefixes. Bold yellow.
    SeverityWarn,
    /// `help:`/`note:` prefixes. Bold cyan.
    SeverityHelp,
    /// `shop.ph:3:48`-shaped locations, including frame `file:line`. Blue.
    Location,
    /// Frame names, selectors, and class names (e.g. `Cart.total`, `'negatd'`). Bold default.
    Identifier,
    /// Box-drawing rails and gutters (`│ ╭ ╰ ·`). Dim default.
    Rail,
    /// The ` 3 │` gutter line number. Dim default.
    LineNumber,
    /// The echoed source line itself. Plain default.
    Source,
    /// The underline beneath the failing span. Bold red.
    SpanPrimary,
    /// The underline beneath a second, supporting span. Blue.
    SpanSecondary,
    /// Text hanging off a caret (`╰── Number has no method 'negatd'`).
    Label,
    /// `[2 core frames elided — pass --trace-core to expand]`. Dim italic default.
    Elision,
    /// `⤷ raised inside fiber #3, spawned at job.ph:1` fiber/cause boundary links. Magenta.
    Chain,
}

impl Role {
    /// This role's fixed color.
    fn color(self) -> AnsiColor {
        match self {
            Role::SeverityError | Role::SpanPrimary => AnsiColor::Red,
            Role::SeverityWarn => AnsiColor::Yellow,
            Role::SeverityHelp => AnsiColor::Cyan,
            Role::Location | Role::SpanSecondary => AnsiColor::Blue,
            Role::Chain => AnsiColor::Magenta,
            Role::Identifier | Role::Rail | Role::LineNumber | Role::Source | Role::Label | Role::Elision => AnsiColor::Default,
        }
    }

    /// This role's fixed weight.
    fn weight(self) -> Weight {
        match self {
            Role::SeverityError | Role::SeverityWarn | Role::SeverityHelp | Role::Identifier | Role::SpanPrimary => Weight::BOLD,
            Role::Rail | Role::LineNumber => Weight::DIM,
            Role::Elision => Weight::DIM_ITALIC,
            Role::Location | Role::Source | Role::SpanSecondary | Role::Label | Role::Chain => Weight::NORMAL,
        }
    }
}

/// How color resolves for a render: explicit override, or auto-detected.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum ColorMode {
    /// Color iff stderr is a TTY and `NO_COLOR` is unset (the default).
    Auto,
    /// Always emit color, regardless of `NO_COLOR` or TTY state.
    Always,
    /// Never emit color.
    Never,
}

/// Which glyph repertoire a render uses for box-drawing and markers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlyphSet {
    /// `╭ │ · ╰ ─ ┬ × ⤷` — the default, including when output is piped.
    Unicode,
    /// `+- | : \`-- x ->` — the ASCII fallback, selected only by `--plain`.
    Ascii,
}

/// The concrete glyphs one [`GlyphSet`] resolves to, used by the caret renderer.
#[derive(Clone, Copy, Debug)]
pub struct Glyphs {
    /// The vertical rail character (`│` / `|`).
    pub rail: &'static str,
    /// The leader dot on an annotation line (`·` / `:`).
    pub dot: &'static str,
    /// The top-left corner opening a snippet block (`╭─` / `+-`).
    pub top_left: &'static str,
    /// The bottom-left corner closing a snippet block (`╰────` / `` `---- ``).
    pub bottom_left: &'static str,
    /// The single-width fill character an underline is built from (`─` / `-`).
    pub underline: &'static str,
    /// The branch-tee character where a label's pointer drops from the underline (`┬` / `+`).
    pub branch_tee: &'static str,
    /// The corner-and-lead-in before a label's text (`╰── ` / `` `-- ``).
    pub branch_corner: &'static str,
    /// The elision marker used when a source line is width-windowed (`…` / `...`).
    pub ellipsis: &'static str,
    /// The severity marker preceding a diagnostic's headline message (`×` / `x`).
    pub error_marker: &'static str,
    /// The chain-link arrow preceding a fiber/cause boundary annotation (`⤷` / `->`).
    pub chain_arrow: &'static str,
}

impl GlyphSet {
    /// Resolves this glyph set to its concrete character sequences.
    #[must_use]
    pub fn glyphs(self) -> Glyphs {
        match self {
            GlyphSet::Unicode => Glyphs {
                rail: "│",
                dot: "·",
                top_left: "╭─",
                bottom_left: "╰────",
                underline: "─",
                branch_tee: "┬",
                branch_corner: "╰── ",
                ellipsis: "…",
                error_marker: "×",
                chain_arrow: "⤷",
            },
            GlyphSet::Ascii => Glyphs {
                rail: "|",
                dot: ":",
                top_left: "+-",
                bottom_left: "`----",
                underline: "-",
                branch_tee: "+",
                branch_corner: "`-- ",
                ellipsis: "...",
                error_marker: "x",
                chain_arrow: "->",
            },
        }
    }
}

/// Resolved rendering configuration for one diagnostic surface.
#[derive(Clone, Copy, Debug)]
pub struct RenderConfig {
    /// The resolved color decision — always [`ColorMode::Always`] or [`ColorMode::Never`].
    pub color: ColorMode,
    /// The glyph repertoire to render box-drawing and markers with.
    pub glyphs: GlyphSet,
    /// The rendering width budget in display columns. Falls back to 80 when the real terminal width is unknown.
    pub width: u16,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self::resolve(ColorMode::Never, true, true, false)
    }
}

impl RenderConfig {
    pub const DEFAULT_WIDTH: u16 = 80;

    #[must_use]
    pub fn resolve(cli_color: ColorMode, plain: bool, no_color_env: bool, stderr_is_tty: bool) -> RenderConfig {
        let glyphs = if plain { GlyphSet::Ascii } else { GlyphSet::Unicode };
        let color = match cli_color {
            ColorMode::Always => ColorMode::Always,
            ColorMode::Never => ColorMode::Never,
            ColorMode::Auto => {
                if plain || no_color_env || !stderr_is_tty {
                    ColorMode::Never
                } else {
                    ColorMode::Always
                }
            }
        };
        RenderConfig {
            color,
            glyphs,
            width: Self::DEFAULT_WIDTH,
        }
    }

    #[must_use]
    pub fn from_env(cli_color: ColorMode, plain: bool) -> RenderConfig {
        let no_color_env = std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
        let stderr_is_tty = std::io::stderr().is_terminal();
        Self::resolve(cli_color, plain, no_color_env, stderr_is_tty)
    }
}

/// The single place SGR (ANSI escape) bytes are produced.
#[derive(Clone, Copy, Debug)]
pub struct Styler {
    enabled: bool,
}

impl Styler {
    #[must_use]
    pub fn new(config: &RenderConfig) -> Styler {
        Styler {
            enabled: matches!(config.color, ColorMode::Always),
        }
    }

    #[must_use]
    pub fn with_color(enabled: bool) -> Styler {
        Styler { enabled }
    }

    #[must_use]
    pub fn paint<'a>(&self, role: Role, text: &'a str) -> Cow<'a, str> {
        if !self.enabled || text.is_empty() {
            return Cow::Borrowed(text);
        }
        let weight = role.weight();
        let mut codes = String::new();
        if weight.bold {
            codes.push_str("1;");
        }
        if weight.dim {
            codes.push_str("2;");
        }
        if weight.italic {
            codes.push_str("3;");
        }
        codes.push_str(&role.color().sgr_code().to_string());
        Cow::Owned(format!("\x1b[{codes}m{text}\x1b[0m"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_never_returns_input_unchanged() {
        let styler = Styler::with_color(false);
        for role in [
            Role::SeverityError,
            Role::SeverityWarn,
            Role::SeverityHelp,
            Role::Location,
            Role::Identifier,
            Role::Rail,
            Role::LineNumber,
            Role::Source,
            Role::SpanPrimary,
            Role::SpanSecondary,
            Role::Label,
            Role::Elision,
            Role::Chain,
        ] {
            let text = "does not understand 'negatd'";
            assert_eq!(styler.paint(role, text), text);
        }
    }

    #[test]
    fn color_always_emits_sgr_and_reset() {
        let styler = Styler::with_color(true);
        let painted = styler.paint(Role::SeverityError, "boom");
        assert!(painted.starts_with("\x1b["));
        assert!(painted.ends_with("\x1b[0m"));
        assert!(painted.contains("boom"));
        assert!(painted.contains("31"));
        assert!(painted.contains('1'));
    }

    #[test]
    fn empty_text_never_gains_escapes() {
        let styler = Styler::with_color(true);
        assert_eq!(styler.paint(Role::SeverityError, ""), "");
    }

    #[test]
    fn explicit_always_beats_no_color_and_non_tty() {
        let config = RenderConfig::resolve(ColorMode::Always, false, true, false);
        assert_eq!(config.color, ColorMode::Always);
    }

    #[test]
    fn no_color_env_beats_tty_auto_detection() {
        let config = RenderConfig::resolve(ColorMode::Auto, false, true, true);
        assert_eq!(config.color, ColorMode::Never);
    }

    #[test]
    fn auto_on_tty_without_no_color_resolves_always() {
        let config = RenderConfig::resolve(ColorMode::Auto, false, false, true);
        assert_eq!(config.color, ColorMode::Always);
    }

    #[test]
    fn plain_forces_ascii_and_color_never() {
        let config = RenderConfig::resolve(ColorMode::Auto, true, false, true);
        assert_eq!(config.color, ColorMode::Never);
        assert_eq!(config.glyphs, GlyphSet::Ascii);
    }

    #[test]
    fn explicit_always_survives_plain() {
        let config = RenderConfig::resolve(ColorMode::Always, true, false, false);
        assert_eq!(config.color, ColorMode::Always);
        assert_eq!(config.glyphs, GlyphSet::Ascii);
    }
}
