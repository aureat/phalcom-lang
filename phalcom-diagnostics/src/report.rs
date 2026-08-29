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
