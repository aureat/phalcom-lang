//! Semantic-role styling substrate for Phalcom diagnostics.
//!
//! [PDR-0014](../../../../docs/decisions/0014-diagnostics-renderer-is-in-house.md) rules that
//! the diagnostic renderer is built in-house rather than adopting `miette`. This module is the
//! concentration point that ruling requires: **all SGR (ANSI escape) emission lives in
//! [`Styler::paint`]** — every other renderer (traceback, compile/parse diagnostics, disasm,
//! trace logs) reaches the terminal only through a [`Role`], never through a raw escape byte.
//! See `docs/spec/traceback/implementation-spec.md` §3.1–§3.3 and `docs/spec/traceback/color.md`
//! for the normative rules this module encodes.
//!
//! Colors attach to **roles**, not literal elements (`color.md` §2): a role gets one appearance
//! everywhere it appears, which is what makes tracebacks, compile errors, disassembly and trace
//! logs read as one program instead of a pile of ad hoc call sites.

use std::borrow::Cow;
use std::io::IsTerminal;

/// One of the seven colors this renderer is allowed to emit.
///
/// Deliberately a closed Rust enum rather than a `u8` palette index: the type itself makes
/// 256-color and truecolor escapes unrepresentable, and pure black/white are omitted outright
/// (`color.md` §3 — both are unreadable against one of the two common terminal backgrounds).
/// "Default" is the color that adapts to the user's theme; prefer it whenever a role does not
/// need a color, since a theme's own default reads correctly against any background.
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
///
/// Only bold, dim, and italic are representable (`color.md` §3: "bold is a real signal here and
/// should be spent sparingly"). No underline, no blink, no reverse-video — those are not part of
/// the palette this renderer speaks.
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
///
/// This is exactly the 13 roles enumerated in `docs/spec/traceback/color.md` §2. Adding a
/// role is a spec change, not a call-site convenience — the point of a closed enum is that
/// every surface using color is drawing from the same, reviewed palette.
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
    /// The echoed source line itself. Plain default — never syntax-highlighted
    /// (`color.md` §2: highlighting the whole line competes with the span underline).
    Source,
    /// The underline beneath the failing span. Bold red.
    SpanPrimary,
    /// The underline beneath a second, supporting span. Blue.
    SpanSecondary,
    /// Text hanging off a caret (`╰── Number has no method 'negatd'`).
    ///
    /// Has no fixed color of its own — `color.md` §2 defines it as "matches its span": a
    /// label attached to a primary span paints with [`Role::SpanPrimary`]'s color, one
    /// attached to a secondary span paints with [`Role::SpanSecondary`]'s. Callers (the caret
    /// renderer) select the underlying span role directly rather than painting `Role::Label`
    /// itself; the variant exists to keep this enum's membership equal to color.md's 13 roles
    /// and to document the rule at its point of definition.
    Label,
    /// `[2 core frames elided — pass --trace-core to expand]`. Dim italic default.
    Elision,
    /// `⤷ raised inside fiber #3, spawned at job.ph:1` fiber/cause boundary links. Magenta.
    Chain,
}

impl Role {
    /// This role's fixed color, per `color.md` §2's table.
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

    /// This role's fixed weight, per `color.md` §2's table.
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
///
/// Mirrors `--color=auto|always|never` (IS §3.2) directly, so it doubles as the `clap`
/// [`clap::ValueEnum`] type for that flag. By the time a [`RenderConfig`] is built via
/// [`RenderConfig::resolve`]/[`RenderConfig::from_env`], the stored value is always
/// [`ColorMode::Always`] or [`ColorMode::Never`] — `Auto` only appears as CLI input, never as
/// a resolved decision (IS §3.2: "Detection happens once at CLI startup").
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
///
/// Orthogonal to [`ColorMode`] (IS §3.3): `--plain` sets both `Ascii` and
/// [`ColorMode::Never`] as one umbrella flag, but `--color=always --plain` is legal and
/// yields ASCII glyphs *with* color.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlyphSet {
    /// `╭ │ · ╰ ─ ┬ × ⤷` — the default, including when output is piped (IS §3.3: ASCII is an
    /// explicit opt-in, not a TTY-detection consequence).
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

/// Resolved rendering configuration for one diagnostic surface (IS §3.2–§3.3).
///
/// Built once — [`RenderConfig::from_env`] at CLI startup — and passed down explicitly rather
/// than read from global mutable state; a REPL or embedder builds its own against whatever
/// stream it actually writes to (IS §3.2). See [`ColorMode`] for why `color` is never `Auto`
/// once a `RenderConfig` exists.
#[derive(Clone, Copy, Debug)]
pub struct RenderConfig {
    /// The resolved color decision — always [`ColorMode::Always`] or [`ColorMode::Never`].
    pub color: ColorMode,
    /// The glyph repertoire to render box-drawing and markers with.
    pub glyphs: GlyphSet,
    /// The rendering width budget in display columns, used by the caret renderer's width
    /// windowing (IS §3.4). Falls back to 80 when the real terminal width is unknown.
    pub width: u16,
}

impl RenderConfig {
    /// The width fallback used whenever the real terminal width cannot be determined — off a
    /// TTY, or on a TTY when this build has no column-count query wired up (see the caret
    /// module's [module docs](super::caret) for the tracked follow-up).
    pub const DEFAULT_WIDTH: u16 = 80;

    /// Resolves a [`RenderConfig`] from already-collected inputs — the pure, unit-testable
    /// core of [`RenderConfig::from_env`] (IS §3.2, §3.3).
    ///
    /// Precedence (IS §3.2): explicit `--color=always|never` beats `NO_COLOR` beats
    /// TTY auto-detection; `plain` forces [`GlyphSet::Ascii`] unconditionally and additionally
    /// forces [`ColorMode::Never`] **only** when `cli_color` is [`ColorMode::Auto`] — an
    /// explicit `--color=always --plain` still yields color (IS §3.3).
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

    /// Resolves a [`RenderConfig`] against the real process environment: `NO_COLOR` (IS §3.2 —
    /// any non-empty value counts) and whether stderr is a TTY, via
    /// [`std::io::IsTerminal`] — **not** the `atty` crate, per IS §3.2.
    #[must_use]
    pub fn from_env(cli_color: ColorMode, plain: bool) -> RenderConfig {
        let no_color_env = std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
        let stderr_is_tty = std::io::stderr().is_terminal();
        Self::resolve(cli_color, plain, no_color_env, stderr_is_tty)
    }
}

/// The single place SGR (ANSI escape) bytes are produced (PDR-0014).
///
/// A `Styler` is a thin, `Copy`-cheap wrapper around a color on/off decision. Every other
/// diagnostics module reaches the terminal through [`Styler::paint`] — raw escape bytes or
/// markup (`color_print`-style tags) outside this module are a review-blockable defect once
/// this unit lands (PDR-0014 consequences).
#[derive(Clone, Copy, Debug)]
pub struct Styler {
    enabled: bool,
}

impl Styler {
    /// Builds a `Styler` from a resolved [`RenderConfig`]. Color is enabled iff
    /// `config.color` is [`ColorMode::Always`]; [`ColorMode::Auto`] is treated as "off" here
    /// since a `RenderConfig` is documented to never carry it after resolution — this is a
    /// defensive default, not a second resolution path.
    #[must_use]
    pub fn new(config: &RenderConfig) -> Styler {
        Styler {
            enabled: matches!(config.color, ColorMode::Always),
        }
    }

    /// Builds a `Styler` with color forced on or off directly, bypassing [`RenderConfig`].
    /// Used by surfaces that must guarantee unstyled output regardless of configuration — the
    /// JSON trace stream (IS §3.2: "`--trace-format=json` → the JSON stream is ALWAYS
    /// unstyled") is the motivating case.
    #[must_use]
    pub fn with_color(enabled: bool) -> Styler {
        Styler { enabled }
    }

    /// Paints `text` with `role`'s color and weight.
    ///
    /// Returns `text` completely unchanged (`Cow::Borrowed`) when color is off — the
    /// [`ColorMode::Never`] contract every distinction in a rendered diagnostic must survive
    /// (IS §3.1; `color.md` §1's "color is emphasis, never information").
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

    /// IS §3.1: `ColorMode::Never` must return the input completely unchanged.
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
        assert!(painted.contains("31")); // red
        assert!(painted.contains('1')); // bold
    }

    #[test]
    fn empty_text_never_gains_escapes() {
        let styler = Styler::with_color(true);
        assert_eq!(styler.paint(Role::SeverityError, ""), "");
    }

    /// Explicit `--color=always` wins over `NO_COLOR` and a non-TTY stderr (IS §3.2 precedence).
    #[test]
    fn explicit_always_beats_no_color_and_non_tty() {
        let config = RenderConfig::resolve(ColorMode::Always, false, true, false);
        assert_eq!(config.color, ColorMode::Always);
    }

    /// `NO_COLOR` wins over TTY auto-detection when `--color` is left at `auto`.
    #[test]
    fn no_color_env_beats_tty_auto_detection() {
        let config = RenderConfig::resolve(ColorMode::Auto, false, true, true);
        assert_eq!(config.color, ColorMode::Never);
    }

    /// `auto` with no `NO_COLOR` and a real TTY resolves to color on.
    #[test]
    fn auto_on_tty_without_no_color_resolves_always() {
        let config = RenderConfig::resolve(ColorMode::Auto, false, false, true);
        assert_eq!(config.color, ColorMode::Always);
    }

    /// `--plain` forces ASCII glyphs and (absent an explicit `--color=always`) color off.
    #[test]
    fn plain_forces_ascii_and_color_never() {
        let config = RenderConfig::resolve(ColorMode::Auto, true, false, true);
        assert_eq!(config.color, ColorMode::Never);
        assert_eq!(config.glyphs, GlyphSet::Ascii);
    }

    /// `--color=always --plain` is the one legal combination that keeps color on with ASCII
    /// glyphs (IS §3.3): the two axes stay separable.
    #[test]
    fn explicit_always_survives_plain() {
        let config = RenderConfig::resolve(ColorMode::Always, true, false, false);
        assert_eq!(config.color, ColorMode::Always);
        assert_eq!(config.glyphs, GlyphSet::Ascii);
    }
}
