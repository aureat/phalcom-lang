# Phalcom Diagnostics source snapshot

> Source: Local repository snapshot at `phalcom-diagnostics/`
> Collected: 2026-09-07
> Published: Unknown

## `phalcom-diagnostics/Cargo.toml`

~~~~toml
[package]
name = "phalcom-diagnostics"
version = "0.1.0"
edition = "2024"

[dependencies]
phalcom-common = { path = "../phalcom-common" }
unicode-width = { workspace = true }
clap = { version = "4.5.41", features = ["derive"] }
~~~~

## `phalcom-diagnostics/src/lib.rs`

~~~~rust
//! Diagnostic rendering substrate: roles, glyphs, styles, caret snippets, and report formatting.

pub mod labels;
pub mod report;
pub mod snippet;
pub mod style;

pub use labels::{LabelLine, layout_labels};
pub use report::{ReportNote, ReportSection, ReportSectionKind, Severity, SourceSnippet, format_diagnostic};
pub use snippet::{Label, LabelKind, Snippet, col_of, display_width, expand_tabs, line_col_1based, locate_line};
pub use style::{AnsiColor, ColorMode, GlyphSet, Glyphs, RenderConfig, Role, Styler, Weight};
~~~~

## `phalcom-diagnostics/src/labels.rs`

~~~~rust
//! Layout and column calculation for diagnostic labels.

use crate::snippet::Label;

/// A formatted label line representation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LabelLine {
    pub col: usize,
    pub len: usize,
    pub text: String,
    pub is_primary: bool,
}

/// Computes label layouts for a source line.
pub fn layout_labels(_source: &str, labels: &[Label<'_>]) -> Vec<LabelLine> {
    labels
        .iter()
        .map(|l| LabelLine {
            col: l.span.start,
            len: l.span.end.saturating_sub(l.span.start).max(1),
            text: l.text.to_string(),
            is_primary: matches!(l.kind, crate::snippet::LabelKind::Primary),
        })
        .collect()
}
~~~~

## `phalcom-diagnostics/src/report.rs`

~~~~rust
//! Diagnostic report representation and formatting helpers.

use super::snippet::{Label, Snippet};
use super::style::{RenderConfig, Role, Styler};

/// Severity level for a diagnostic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Help,
    Note,
    Information,
    Hint,
}

impl Severity {
    pub fn role(self) -> Role {
        match self {
            Severity::Error => Role::SeverityError,
            Severity::Warning => Role::SeverityWarn,
            Severity::Help | Severity::Note | Severity::Information | Severity::Hint => Role::SeverityHelp,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Help => "help",
            Severity::Note => "note",
            Severity::Information => "info",
            Severity::Hint => "hint",
        }
    }
}

/// A structured report note or help line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReportNote {
    pub message: String,
    pub is_help: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReportSectionKind {
    Explanation,
    Guidance,
    Context,
    Trace,
}

impl ReportSectionKind {
    fn heading(self) -> &'static str {
        match self {
            Self::Explanation => "explanation",
            Self::Guidance => "guidance",
            Self::Context => "context",
            Self::Trace => "type trace",
        }
    }

    fn heading_role(self) -> Role {
        match self {
            Self::Explanation => Role::Identifier,
            Self::Guidance => Role::SeverityHelp,
            Self::Context => Role::Chain,
            Self::Trace => Role::Rail,
        }
    }

    fn body_role(self) -> Role {
        match self {
            Self::Trace => Role::Rail,
            Self::Explanation | Self::Guidance | Self::Context => Role::Source,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReportSection {
    pub kind: ReportSectionKind,
    pub lines: Vec<String>,
}

/// A single source snippet entry for rendering.
#[derive(Clone, Debug)]
pub struct SourceSnippet<'a> {
    pub file: Option<String>,
    pub source: &'a str,
    pub labels: Vec<Label<'a>>,
}

/// Formats a complete diagnostic with title, snippets, notes, help items, and
/// protocol-neutral rich sections. This renderer performs no semantic work.
pub fn format_diagnostic<'a>(
    code: Option<&str>,
    severity: Severity,
    title: &str,
    snippets: &[SourceSnippet<'a>],
    notes: &[ReportNote],
    sections: &[ReportSection],
    config: &RenderConfig,
) -> String {
    let styler = Styler::new(config);
    let mut out = String::new();

    let sev_str = severity.as_str();
    out.push_str(&styler.paint(severity.role(), sev_str));
    if let Some(c) = code {
        let code_bracket = format!("[{c}]");
        out.push_str(&styler.paint(severity.role(), &code_bracket));
    }
    out.push_str(&styler.paint(severity.role(), ": "));
    out.push_str(title);
    out.push('\n');

    for snip in snippets {
        let snippet_renderer = match &snip.file {
            Some(f) => Snippet::with_file(f),
            None => Snippet::new(),
        };
        out.push_str(&snippet_renderer.render(snip.source, &snip.labels, config));
    }

    for section in sections.iter().filter(|section| !section.lines.is_empty()) {
        out.push_str("  ");
        out.push_str(&styler.paint(section.kind.heading_role(), section.kind.heading()));
        out.push_str(&styler.paint(section.kind.heading_role(), ":"));
        out.push('\n');
        for line in &section.lines {
            out.push_str("    ");
            out.push_str(&styler.paint(section.kind.body_role(), line));
            out.push('\n');
        }
    }

    for note in notes {
        let prefix = if note.is_help { "help" } else { "note" };
        let role = Role::SeverityHelp;
        out.push_str("  ");
        out.push_str(&styler.paint(role, prefix));
        out.push_str(&styler.paint(role, ": "));
        out.push_str(&note.message);
        out.push('\n');
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::{ColorMode, GlyphSet};

    fn config(color: ColorMode, glyphs: GlyphSet) -> RenderConfig {
        RenderConfig { color, glyphs, width: 80 }
    }

    #[test]
    fn no_color_sections_are_stable_and_unboxed() {
        let rendered = format_diagnostic(
            Some("type.binding.initializer_mismatch"),
            Severity::Error,
            "initializer conflicts with declared type",
            &[],
            &[],
            &[
                ReportSection {
                    kind: ReportSectionKind::Explanation,
                    lines: vec!["the constructor returns `Self`".into(), "here `Self` resolves to `CellNum`".into()],
                },
                ReportSection {
                    kind: ReportSectionKind::Guidance,
                    lines: vec!["`result` can be declared as `CellNum`".into()],
                },
            ],
            &config(ColorMode::Never, GlyphSet::Unicode),
        );
        assert!(rendered.starts_with("error[type.binding.initializer_mismatch]: initializer conflicts with declared type\n"));
        assert!(rendered.contains("  explanation:\n    the constructor returns `Self`\n"));
        assert!(rendered.contains("  guidance:\n    `result` can be declared as `CellNum`\n"));
        assert!(!rendered.contains("\x1b["));
    }

    #[test]
    fn forced_color_styles_section_headings() {
        let rendered = format_diagnostic(
            None,
            Severity::Error,
            "bad type",
            &[],
            &[],
            &[ReportSection {
                kind: ReportSectionKind::Context,
                lines: vec!["tooling observes `User`".into()],
            }],
            &config(ColorMode::Always, GlyphSet::Unicode),
        );
        assert!(rendered.contains("\x1b["));
    }

    #[test]
    fn trace_body_uses_dim_style_when_colored() {
        let rendered = format_diagnostic(
            None,
            Severity::Error,
            "bad type",
            &[],
            &[],
            &[ReportSection {
                kind: ReportSectionKind::Trace,
                lines: vec!["[e1] relation — refuted".into()],
            }],
            &config(ColorMode::Always, GlyphSet::Ascii),
        );
        assert!(rendered.contains("[e1] relation"));
        assert!(rendered.contains("\x1b[2;39m"));
    }
}
~~~~

## `phalcom-diagnostics/src/snippet.rs`

~~~~rust
//! The caret/snippet renderer — column arithmetic for pointing at a source span.

use super::style::{RenderConfig, Role, Styler};
use phalcom_common::range::SourceRange;
use unicode_width::UnicodeWidthChar;

/// Whether a [`Label`] marks the failing span or a supporting one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabelKind {
    /// The span the diagnostic is actually about — underlined in [`Role::SpanPrimary`].
    Primary,
    /// A second, supporting span — underlined in [`Role::SpanSecondary`].
    Secondary,
}

/// One labeled span to render on a [`Snippet`].
#[derive(Clone, Debug)]
pub struct Label<'a> {
    /// The byte span into the snippet's source this label points at.
    pub span: SourceRange,
    /// The text hanging off this label's pointer.
    pub text: &'a str,
    /// Whether this is the diagnostic's primary or a secondary span.
    pub kind: LabelKind,
}

/// A renderable source excerpt: one or more labeled spans, boxed in `╭─ │ · ╰──`.
#[derive(Clone, Debug, Default)]
pub struct Snippet {
    /// The file name shown in the header (`╭─[shop.ph:3:48]`).
    pub file: Option<String>,
}

impl Snippet {
    #[must_use]
    pub fn new() -> Snippet {
        Snippet { file: None }
    }

    #[must_use]
    pub fn with_file(file: impl Into<String>) -> Snippet {
        Snippet { file: Some(file.into()) }
    }

    #[must_use]
    pub fn render(&self, source: &str, labels: &[Label<'_>], config: &RenderConfig) -> String {
        if labels.is_empty() {
            return String::new();
        }
        let styler = Styler::new(config);
        let glyphs = config.glyphs.glyphs();

        let mut ordered: Vec<&Label<'_>> = labels.iter().collect();
        ordered.sort_by_key(|l| (l.span.start, matches!(l.kind, LabelKind::Secondary)));

        let mut out = String::new();
        let (head_line, head_col) = line_col_1based(source, ordered[0].span.start);
        let gutter_width = ordered
            .iter()
            .map(|l| line_col_1based(source, l.span.start).0)
            .max()
            .unwrap_or(head_line)
            .to_string()
            .len()
            .max(2);

        let rail = styler.paint(Role::Rail, glyphs.top_left);
        if let Some(file) = &self.file {
            let location_text = format!("[{file}:{head_line}:{head_col}]");
            let location = styler.paint(Role::Location, &location_text);
            out.push_str(&" ".repeat(gutter_width + 1));
            out.push_str(&rail);
            out.push_str(&location);
            out.push('\n');
        } else {
            out.push_str(&" ".repeat(gutter_width + 1));
            out.push_str(&rail);
            out.push('\n');
        }

        let mut lines_seen: Vec<usize> = Vec::new();
        for label in &ordered {
            let (line_no, _) = line_col_1based(source, label.span.start);
            if !lines_seen.contains(&line_no) {
                lines_seen.push(line_no);
            }
        }

        for line_no in lines_seen {
            let group: Vec<&&Label<'_>> = ordered.iter().filter(|l| line_col_1based(source, l.span.start).0 == line_no).collect();
            render_line_block(&mut out, source, &group, config, &styler, &glyphs, gutter_width);
        }

        out.push_str(&" ".repeat(gutter_width + 1));
        out.push_str(&styler.paint(Role::Rail, glyphs.bottom_left));
        out.push('\n');
        out
    }
}

fn render_line_block(
    out: &mut String,
    source: &str,
    group: &[&&Label<'_>],
    config: &RenderConfig,
    styler: &Styler,
    glyphs: &super::style::Glyphs,
    gutter_width: usize,
) {
    let primary = group.iter().find(|l| matches!(l.kind, LabelKind::Primary)).copied().unwrap_or(group[0]);
    let (line_no, line_start, line_text) = locate_line(source, primary.span.start);

    let expanded = expand_tabs(line_text);
    let total_width = display_width(&expanded);

    let (win_start_col, win_text, left_trim, right_trim) = window(
        &expanded,
        total_width,
        col_of(line_text, byte_in_line(primary, line_start, line_text)),
        config.width,
        glyphs,
    );

    out.push_str(&styler.paint(Role::LineNumber, &format!("{line_no:>gutter_width$}")));
    out.push(' ');
    out.push_str(&styler.paint(Role::Rail, glyphs.rail));
    out.push(' ');
    out.push_str(&styler.paint(Role::Source, &win_text));
    out.push('\n');

    for label in group {
        let role = match label.kind {
            LabelKind::Primary => Role::SpanPrimary,
            LabelKind::Secondary => Role::SpanSecondary,
        };
        let multiline = source[label.span.start..label.span.end.min(source.len())].contains('\n');
        let start_byte = byte_in_line(label, line_start, line_text);
        let end_byte = if multiline {
            line_text.len()
        } else {
            (label.span.end.saturating_sub(line_start)).min(line_text.len())
        };
        let start_col = col_of(line_text, start_byte);
        let end_col = col_of(line_text, end_byte).max(start_col + 1);

        let win_display_width = display_width(&win_text);
        let shift = |c: usize| -> usize {
            let left_pad = usize::from(left_trim);
            (c.saturating_sub(win_start_col) + left_pad).min(win_display_width)
        };
        let ustart = shift(start_col);
        let uend = shift(end_col).max(ustart + 1);

        out.push_str(&" ".repeat(gutter_width));
        out.push(' ');
        out.push_str(&styler.paint(Role::Rail, glyphs.dot));
        out.push(' ');
        out.push_str(&" ".repeat(ustart));
        let underline: String = glyphs.underline.repeat(uend - ustart);
        out.push_str(&styler.paint(role, &underline));
        out.push('\n');

        out.push_str(&" ".repeat(gutter_width));
        out.push(' ');
        out.push_str(&styler.paint(Role::Rail, glyphs.dot));
        out.push(' ');
        out.push_str(&" ".repeat(ustart));
        let mut text = label.text.to_string();
        if multiline {
            let spans = source[label.span.start..label.span.end.min(source.len())].matches('\n').count() + 1;
            text.push_str(&format!(" (spans {spans} lines)"));
        }
        out.push_str(&styler.paint(Role::Rail, glyphs.branch_corner));
        out.push_str(&styler.paint(role, &text));
        out.push('\n');
        let _ = right_trim;
    }
}

fn byte_in_line(label: &Label<'_>, line_start: usize, line_text: &str) -> usize {
    label.span.start.saturating_sub(line_start).min(line_text.len())
}

pub fn line_col_1based(source: &str, offset: usize) -> (usize, usize) {
    let (line_no, line_start, line_text) = locate_line(source, offset);
    let byte_col = offset.saturating_sub(line_start).min(line_text.len());
    (line_no, col_of(line_text, byte_col) + 1)
}

pub fn locate_line(source: &str, offset: usize) -> (usize, usize, &str) {
    let mut start = 0usize;
    let lines: Vec<&str> = source.split('\n').collect();
    let last = lines.len().saturating_sub(1);
    for (idx, line) in lines.iter().enumerate() {
        let end = start + line.len();
        if offset <= end || idx == last {
            return (idx + 1, start, line);
        }
        start = end + 1;
    }
    (1, 0, "")
}

pub fn expand_tabs(line: &str) -> String {
    let mut out = String::new();
    let mut col = 0usize;
    for ch in line.chars() {
        if ch == '\t' {
            let next = col + (4 - col % 4);
            out.push_str(&" ".repeat(next - col));
            col = next;
        } else {
            out.push(ch);
            col += UnicodeWidthChar::width(ch).unwrap_or(0);
        }
    }
    out
}

pub fn display_width(expanded: &str) -> usize {
    expanded.chars().map(|c| UnicodeWidthChar::width(c).unwrap_or(0)).sum()
}

pub fn col_of(line: &str, byte_offset: usize) -> usize {
    let mut cut = byte_offset.min(line.len());
    while cut > 0 && !line.is_char_boundary(cut) {
        cut -= 1;
    }
    display_width(&expand_tabs(&line[..cut]))
}

fn window(expanded: &str, total_width: usize, focus_col: usize, width: u16, glyphs: &super::style::Glyphs) -> (usize, String, bool, bool) {
    let width = width.max(10) as usize;
    if total_width <= width {
        return (0, expanded.to_string(), false, false);
    }

    let ellipsis_cols = glyphs.ellipsis.chars().count();
    let budget = width.saturating_sub(2 * ellipsis_cols).max(1);
    let half = budget / 2;
    let mut win_start = focus_col.saturating_sub(half);
    if win_start + budget > total_width {
        win_start = total_width.saturating_sub(budget);
    }
    let win_end = (win_start + budget).min(total_width);

    let chars: Vec<char> = expanded.chars().collect();
    let mut col = 0usize;
    let mut start_idx = chars.len();
    let mut end_idx = chars.len();
    for (idx, ch) in chars.iter().enumerate() {
        if col >= win_start && start_idx == chars.len() {
            start_idx = idx;
        }
        if col >= win_end {
            end_idx = idx;
            break;
        }
        col += UnicodeWidthChar::width(*ch).unwrap_or(0);
    }

    let left_trim = win_start > 0;
    let right_trim = win_end < total_width;
    let mut text = String::new();
    if left_trim {
        text.push_str(glyphs.ellipsis);
    }
    text.push_str(&chars[start_idx..end_idx].iter().collect::<String>());
    if right_trim {
        text.push_str(glyphs.ellipsis);
    }
    let effective_start = if left_trim { win_start.saturating_sub(ellipsis_cols) } else { win_start };
    (effective_start, text, left_trim, right_trim)
}

#[cfg(test)]
mod tests {
    use super::super::style::GlyphSet;
    use super::*;
    use phalcom_common::range::SourceRange;

    fn cfg(color: bool, glyphs: GlyphSet, width: u16) -> RenderConfig {
        RenderConfig {
            color: if color {
                super::super::style::ColorMode::Always
            } else {
                super::super::style::ColorMode::Never
            },
            glyphs,
            width,
        }
    }

    fn range(start: usize, end: usize) -> SourceRange {
        SourceRange { start, end }
    }

    #[test]
    fn tab_expansion_aligns_underline_to_next_multiple_of_four() {
        let source = "\tx = 1";
        let labels = [Label {
            span: range(1, 2),
            text: "here",
            kind: LabelKind::Primary,
        }];
        let out = Snippet::new().render(source, &labels, &cfg(false, GlyphSet::Unicode, 80));
        let underline_line = out.lines().find(|l| l.contains('─') && l.contains('·')).expect("underline row");
        let after_dot = underline_line.split('·').nth(1).expect("rail dot marker");
        let indent = after_dot.chars().skip(1).take_while(|c| *c == ' ').count();
        assert_eq!(indent, 4);
    }

    #[test]
    fn cjk_span_underlines_full_display_width() {
        let source = "let x = 你好";
        let start = source.find("你好").unwrap();
        let end = source.len();
        let labels = [Label {
            span: range(start, end),
            text: "wide",
            kind: LabelKind::Primary,
        }];
        let out = Snippet::new().render(source, &labels, &cfg(false, GlyphSet::Unicode, 80));
        let underline_line = out.lines().find(|l| l.contains('─') && l.contains('·')).unwrap();
        let underline_cols = underline_line.chars().filter(|c| *c == '─').count();
        assert_eq!(underline_cols, 4);
    }

    #[test]
    fn combining_marks_are_zero_width() {
        let source = "cafe\u{0301} ok";
        let start = source.find("e\u{0301}").unwrap();
        let end = start + "e\u{0301}".len();
        let labels = [Label {
            span: range(start, end),
            text: "accent",
            kind: LabelKind::Primary,
        }];
        let out = Snippet::new().render(source, &labels, &cfg(false, GlyphSet::Unicode, 80));
        let underline_line = out.lines().find(|l| l.contains('─') && !l.contains('╭') && !l.contains('╰')).unwrap();
        let underline_cols = underline_line.chars().filter(|c| *c == '─').count();
        assert_eq!(underline_cols, 1);
    }

    #[test]
    fn window_elision_trims_long_lines_around_the_primary_span() {
        let long_prefix = "x".repeat(100);
        let source = format!("{long_prefix}TARGET{}", "y".repeat(100));
        let start = long_prefix.len();
        let end = start + "TARGET".len();
        let labels = [Label {
            span: range(start, end),
            text: "here",
            kind: LabelKind::Primary,
        }];
        let out = Snippet::new().render(&source, &labels, &cfg(false, GlyphSet::Unicode, 40));
        let source_line = out.lines().find(|l| l.contains("TARGET")).expect("windowed source row");
        assert!(source_line.contains('…'));
        assert!(source_line.len() < source.len());
    }

    #[test]
    fn two_label_layout_renders_primary_and_secondary() {
        let source = "{ unterminated";
        let opener = Label {
            span: range(0, 1),
            text: "opened here",
            kind: LabelKind::Secondary,
        };
        let eof = Label {
            span: range(source.len(), source.len()),
            text: "expected '}'",
            kind: LabelKind::Primary,
        };
        let out = Snippet::new().render(source, &[opener, eof], &cfg(false, GlyphSet::Unicode, 80));
        assert!(out.contains("opened here"));
        assert!(out.contains("expected '}'"));
    }

    #[test]
    fn ascii_glyph_set_uses_ascii_only_box_drawing() {
        let source = "1 + negatd";
        let labels = [Label {
            span: range(4, 10),
            text: "no such method",
            kind: LabelKind::Primary,
        }];
        let out = Snippet::new().render(source, &labels, &cfg(false, GlyphSet::Ascii, 80));
        assert!(!out.chars().any(|c| c as u32 > 0x7f));
        assert!(out.contains('|'));
        assert!(out.contains('-'));
    }

    #[test]
    fn strip_sgr_invariance() {
        let source = "1 + negatd";
        let labels = [Label {
            span: range(4, 10),
            text: "no such method",
            kind: LabelKind::Primary,
        }];
        let styled = Snippet::new().render(source, &labels, &cfg(true, GlyphSet::Unicode, 80));
        let plain = Snippet::new().render(source, &labels, &cfg(false, GlyphSet::Unicode, 80));
        assert_eq!(strip_sgr(&styled), plain);
    }

    fn strip_sgr(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\x1b' && chars.peek() == Some(&'[') {
                chars.next();
                for c in chars.by_ref() {
                    if c == 'm' {
                        break;
                    }
                }
                continue;
            }
            out.push(c);
        }
        out
    }
}
~~~~

## `phalcom-diagnostics/src/style.rs`

~~~~rust
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
~~~~


