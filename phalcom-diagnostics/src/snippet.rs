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
