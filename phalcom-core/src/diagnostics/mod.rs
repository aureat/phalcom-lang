//! Diagnostic rendering: line/column arithmetic, caret snippets, and the traceback renderer
//! (PDR-0014, IS §1–§5).
//!
//! - [`style`] — [`style::Role`], [`style::ColorMode`], [`style::GlyphSet`], [`style::Styler`]:
//!   the only place SGR (ANSI escape) bytes are produced (IS §3.1).
//! - [`caret`] — [`caret::Snippet`]/[`caret::Label`]: the column-arithmetic caret/snippet
//!   renderer (IS §3.4).
//! - [`traceback`] — [`traceback::render_traceback`]: the human and JSON traceback renderers;
//!   replaces the legacy `print_rt`/`print_frame` functions (plan.md T4).
//! - This module itself keeps [`line_col`], [`print_parse`]/[`print_compile`] (restyled, IS §1)
//!   and the legacy [`SOURCE_MAP`] until plan.md §7 deletes it.

pub mod caret;
pub mod style;
pub mod suggest;
pub mod traceback;

use phalcom_common::range::SourceRange;
use std::ops::Range;
use std::sync::{Arc, OnceLock};
use style::{ColorMode, RenderConfig, Role, Styler};

/// The process-wide [`RenderConfig`], installed once at CLI startup
/// (`bin/phalcom/main.rs::main`) from the `--color`/`--plain` flags.
///
/// **Interim bridge, not the final architecture.** IS §3.2's target design threads a
/// `RenderConfig` explicitly through every renderer, with no global mutable state — a REPL or
/// embedder builds its own against whatever stream it writes to. Getting there for
/// [`print_compile`]/[`print_parse`]/[`print_rt`] means giving them a `&RenderConfig`
/// parameter, which ripples into `vm/dispatch.rs` and `interpret.rs` (their callers) —
/// outside this unit's write-set (plan.md T2; those files are T4/T5's). This `OnceLock` lets
/// `--color`/`--plain` actually take effect today without touching those files: it is set once
/// and read-only afterward (the same one-shot-init shape `main.rs` already uses for its
/// `tracing_subscriber` registry). [`install_render_config`] is a no-op after the first
/// successful call; [`active_render_config`] falls back to a fresh
/// [`RenderConfig::from_env`] for any caller that runs before installation (tests, tools that
/// link `phalcom-core` without going through the CLI).
static RENDER_CONFIG: OnceLock<RenderConfig> = OnceLock::new();

/// Installs the process-wide [`RenderConfig`], once. See the module-private `RENDER_CONFIG`
/// static's docs (above, in source) for why this exists and what supersedes it.
pub fn install_render_config(config: RenderConfig) {
    let _ = RENDER_CONFIG.set(config);
}

/// Returns the installed [`RenderConfig`], or a fresh [`RenderConfig::from_env`] default if
/// [`install_render_config`] has not run yet.
pub fn active_render_config() -> RenderConfig {
    match RENDER_CONFIG.get() {
        Some(config) => *config,
        None => RenderConfig::from_env(ColorMode::Auto, false),
    }
}

/// A pointer into a specific module’s source.
#[derive(Clone, Debug)]
pub struct SourceLoc {
    /// The module the failing frame belongs to.
    pub module_name: String,
    /// The method (or `<main>`/block) the failing frame belongs to.
    pub method_name: String,
    /// The byte span within [`Self::source`] the frame's instruction pointer maps to.
    pub span: SourceRange,
    /// The text [`Self::span`] indexes into, resolved through the frame's
    /// [`Chunk::source_id`](crate::chunk::Chunk::source_id).
    ///
    /// `None` when the frame's chunk carries an id the module never recorded —
    /// a synthesized chunk on a source-less module. Such a frame still appears
    /// in the traceback, just without a code excerpt (U-REPL §D2).
    pub source: Option<Arc<String>>,
}

/// Resolves a byte `offset` into `source` to a 1-based `(line, column)` pair.
///
/// Extracted from the line/column arithmetic [`print_line_information`]
/// already performed inline (U-CLASSCLOSE §3 option A), so a compile error
/// that has no renderer to hang a caret span on can still put a real
/// `line:col` into its message text — e.g. `class 'Point' is already defined
/// in this module (first declared at 3:1)`.
///
/// Clamps to `(last_line, 1)` for an `offset` past every line the source
/// actually has (e.g. a synthesized end-of-file span); never panics.
pub fn line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line_start = 0;
    for (idx, line) in source.lines().enumerate() {
        let line_end = line_start + line.len();
        if offset >= line_start && offset <= line_end {
            return (idx + 1, offset - line_start + 1);
        }
        line_start = line_end + 1; // +1 for '\n'
    }
    (source.lines().count().max(1), 1)
}

/// Prints a byte `range` in `source` as a numbered source excerpt with a caret underline.
///
/// Mechanically migrated onto [`Styler`] (PDR-0014): identical text and layout to the
/// `color_print`/`ceprintln!` version this replaces, only the emission path changed. The box
/// look and column-exact caret arithmetic (IS §3.4) are [`caret::Snippet`]'s job — this
/// function is the legacy renderer plan.md T4 replaces outright.
pub fn print_line_information(source: &str, range: Range<usize>) {
    let styler = Styler::new(&active_render_config());
    let (line_number, col_start_1based) = line_col(source, range.start);
    let col_start = col_start_1based - 1;

    let lines: Vec<&str> = source.lines().collect();
    let current = line_number - 1;

    let col_end = range.end - lines[..current].iter().map(|l| l.len() + 1).sum::<usize>();

    eprintln!(
        "   {} {}",
        styler.paint(Role::Rail, "-->"),
        styler.paint(Role::Location, &format!("Error at {line_number}:{col_start}"))
    );
    eprintln!("    {}", styler.paint(Role::Rail, "|"));

    if current > 0 {
        eprintln!(
            "{} {}",
            styler.paint(Role::LineNumber, &format!("{current:>3} |")),
            lines[current - 1].trim_end()
        );
    }

    eprintln!(
        "{} {}",
        styler.paint(Role::LineNumber, &format!("{line_number:>3} |")),
        styler.paint(Role::Source, lines[current].trim_end())
    );

    let indent = " ".repeat(col_start);
    let carets = "^".repeat((col_end - col_start).max(1));
    eprintln!("    {} {}{}", styler.paint(Role::Rail, "|"), indent, styler.paint(Role::SpanPrimary, &carets));

    if current + 1 < lines.len() {
        eprintln!(
            "{} {}",
            styler.paint(Role::LineNumber, &format!("{:>3} |", line_number + 1)),
            lines[current + 1].trim_end()
        );
    }

    eprintln!("    {}", styler.paint(Role::Rail, "|"));
}

/// Pretty-prints a parse error given only a byte range into the source string.
pub fn print_parse(source: &str, path: Option<&str>, msg: &str, range: Range<usize>) {
    let config = active_render_config();
    let styler = Styler::new(&config);
    eprintln!(
        "{}{} {}",
        styler.paint(Role::SeverityError, "error"),
        styler.paint(Role::SeverityError, ":"),
        msg
    );

    let label = caret::Label {
        span: SourceRange::from(range),
        text: msg,
        kind: caret::LabelKind::Primary,
    };
    let snippet = match path {
        Some(p) => caret::Snippet::with_file(p.to_string()),
        None => caret::Snippet::new(),
    };
    let rendered = snippet.render(source, &[label], &config);
    eprint!("{rendered}");
}

/// Pretty-prints a *compile* error.
///
/// Span-less by design for now: most [`CompilerError`](crate::compiler::lib::CompilerError)
/// variants carry no [`SourceRange`], and the
/// few that do are not threaded together with the source text needed to render
/// an excerpt. Rendering the message alone is what
/// [PDR-0008](../../docs/decisions/0008-cell-boundary-diagnostics-and-state-hygiene.md) §1
/// requires; adding spans is a separate change that has to plumb the source
/// through first.
pub fn print_compile(msg: &str) {
    let styler = Styler::new(&active_render_config());
    eprintln!(
        "{}{} {}",
        styler.paint(Role::SeverityError, "error"),
        styler.paint(Role::SeverityError, ":"),
        msg
    );
}

// `print_rt` and `print_frame` removed in plan.md T4.
// Their replacement is [`traceback::render_traceback`].
