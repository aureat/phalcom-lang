use crate::interner::Symbol;
use color_print::ceprintln;
use lazy_static::lazy_static;
use phalcom_common::range::SourceRange;
use std::collections::HashMap;
use std::ops::Range;
use std::sync::{Arc, RwLock};

lazy_static! {
    pub static ref SOURCE_MAP: RwLock<HashMap<Symbol, Arc<String>>> = RwLock::new(HashMap::new());
}

/// A pointer into a specific module’s source.
#[derive(Clone, Debug)]
pub struct SourceLoc {
    pub module_name: String,
    pub method_name: String,
    pub span: SourceRange,
    /// The text [`Self::span`] indexes into, resolved through the frame's
    /// [`Chunk::source_id`](crate::chunk::Chunk::source_id).
    ///
    /// `None` when the frame's chunk carries an id the module never recorded —
    /// a synthesized chunk on a source-less module. Such a frame still appears
    /// in the traceback, just without a code excerpt (U-REPL §D2).
    pub source: Option<Arc<String>>,
}

// use std::ops::Range;

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

pub fn print_line_information(source: &str, range: Range<usize>) {
    let (line_number, col_start_1based) = line_col(source, range.start);
    let col_start = col_start_1based - 1;

    let lines: Vec<&str> = source.lines().collect();
    let current = line_number - 1;

    let col_end = range.end - lines[..current].iter().map(|l| l.len() + 1).sum::<usize>();

    ceprintln!("   <s,r!>--></> Error at {}:{}", line_number, col_start);
    ceprintln!("    <s,r!>|</>");

    if current > 0 {
        ceprintln!("<s,r!>{:>3} |</> {}", current, lines[current - 1].trim_end());
    }

    ceprintln!("<s,r!>{:>3} |</> <s>{}", line_number, lines[current].trim_end());

    let indent = " ".repeat(col_start);
    let carets = "^".repeat((col_end - col_start).max(1));
    ceprintln!("    <s,r!>|</> {}<s,y>{}</>", indent, carets);

    if current + 1 < lines.len() {
        ceprintln!("<s,r!>{:>3} |</> {}", line_number + 1, lines[current + 1].trim_end());
    }

    ceprintln!("    <s,r!>|</>");
}

/// Pretty-prints a parse error given only a byte range into the source string.
pub fn print_parse(source: &str, msg: &str, range: Range<usize>) {
    if range.start >= source.len() || range.end > source.len() || range.start >= range.end {
        ceprintln!("   <s,r!>|</> Syntax error at file");
        ceprintln!("    <s><r!>=</r!> {msg}");
        return;
    }

    print_line_information(source, range);
    ceprintln!("    <s><r!>=</r!> {msg}");
}

/// Pretty-prints a *compile* error.
///
/// Span-less by design for now: most [`CompilerError`](crate::compiler::lib::CompilerError)
/// variants carry no [`SourceRange`](phalcom_common::range::SourceRange), and the
/// few that do are not threaded together with the source text needed to render
/// an excerpt. Rendering the message alone is what
/// [PDR-0008](../../docs/decisions/0008-cell-boundary-diagnostics-and-state-hygiene.md) §1
/// requires; adding spans is a separate change that has to plumb the source
/// through first.
pub fn print_compile(msg: &str) {
    ceprintln!("   <s,r!>|</> Compile error");
    ceprintln!("    <s><r!>=</r!> {msg}");
}

/// Pretty-print a *runtime* error with Python-style stack trace.
/// `stack` must be ordered **caller → callee** (older frames first).
pub fn print_rt(msg: &str, stack: &[SourceLoc]) {
    ceprintln!("<s,r!>Traceback (most recent call last):");
    ceprintln!("    <s,r!>|</>");
    ceprintln!("    <s><r!>=</r!> {msg}");
    ceprintln!("    <s,r!>|</>");

    for frame in stack {
        print_frame(frame);
    }

    // print_frame(loc);
}

/// Print one “File "...", line X” entry plus its source line.
fn print_frame(loc: &SourceLoc) {
    // if let Some(entry) = SOURCE_MAP.read().unwrap().get(&loc.module_id) {
    //     if let Some(line) = line(&entry.code, loc.span.start.line) {
    //         eprintln!("  File \"{}\", line {}", entry.name, loc.span.start.line);
    //         eprintln!("    {}", line.trim_end());
    //     }
    // }
    // A frame whose source could not be resolved still belongs in the
    // traceback — it just cannot show the offending line.
    if let Some(source) = &loc.source {
        print_line_information(source, loc.span.start..loc.span.end);
    }
}

// /// Fetch `n`-th (1-based) line from `src`.
// fn line<'a>(src: &'a str, n: u32) -> Option<&'a str> {
//     src.lines().nth((n - 1) as usize)
// }
