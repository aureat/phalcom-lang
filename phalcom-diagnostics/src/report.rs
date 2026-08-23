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
}

impl Severity {
    pub fn role(self) -> Role {
        match self {
            Severity::Error => Role::SeverityError,
            Severity::Warning => Role::SeverityWarn,
            Severity::Help | Severity::Note => Role::SeverityHelp,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Help => "help",
            Severity::Note => "note",
        }
    }
}

/// A structured report note or help line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReportNote {
    pub message: String,
    pub is_help: bool,
}

/// A single source snippet entry for rendering.
#[derive(Clone, Debug)]
pub struct SourceSnippet<'a> {
    pub file: Option<String>,
    pub source: &'a str,
    pub labels: Vec<Label<'a>>,
}

/// Formats a complete diagnostic with title, snippets, notes, and help items.
pub fn format_diagnostic<'a>(
    code: Option<&str>,
    severity: Severity,
    title: &str,
    snippets: &[SourceSnippet<'a>],
    notes: &[ReportNote],
    config: &RenderConfig,
) -> String {
    let styler = Styler::new(config);
    let _glyphs = config.glyphs.glyphs();
    let mut out = String::new();

    // Headline: error[code]: title OR error: title
    let sev_str = severity.as_str();
    let sev_painted = styler.paint(severity.role(), sev_str);
    out.push_str(&sev_painted);
    if let Some(c) = code {
        let code_bracket = format!("[{c}]");
        out.push_str(&styler.paint(severity.role(), &code_bracket));
    }
    out.push_str(&styler.paint(severity.role(), ": "));
    out.push_str(title);
    out.push('\n');

    // Render each snippet
    for snip in snippets {
        let snippet_renderer = match &snip.file {
            Some(f) => Snippet::with_file(f),
            None => Snippet::new(),
        };
        let rendered = snippet_renderer.render(snip.source, &snip.labels, config);
        out.push_str(&rendered);
    }

    // Notes and helps
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
