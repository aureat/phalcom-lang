//! Diagnostic rendering substrate: roles, glyphs, styles, caret snippets, and report formatting.

pub mod labels;
pub mod report;
pub mod snippet;
pub mod style;

pub use labels::{LabelLine, layout_labels};
pub use report::{ReportNote, Severity, SourceSnippet, format_diagnostic};
pub use snippet::{Label, LabelKind, Snippet, col_of, display_width, expand_tabs, line_col_1based, locate_line};
pub use style::{AnsiColor, ColorMode, GlyphSet, Glyphs, RenderConfig, Role, Styler, Weight};
