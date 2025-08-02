use crate::interner::Symbol;
use crate::module::ModuleId;
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
    pub module_id: ModuleId,
    pub span: SourceRange,
}

/// Pretty-prints a parse error given only a byte range into the source string.
pub fn print_parse(source: &str, msg: &str, range: Range<usize>) {
    if range.start >= source.len() || range.end > source.len() || range.start >= range.end {
        eprintln!("SyntaxError: {msg}");
        return;
    }

    // let line_start = source[..range.start].rfind('\n').map_or(0, |i| i + 1);
    // let line_end = source[range.end..].find('\n').map_or(source.len(), |i| range.end + i);

    let line_start = source[..range.start].rfind('\n').map_or(0, |i| i + 1);
    let line_end = source[range.start..].find('\n').map_or(source.len(), |i| range.start + i);

    let line_str = &source[line_start..line_end];

    // 2. Compute line number (1-based) by counting newlines before the line
    let line_number = source[..line_start].chars().filter(|&c| c == '\n').count() + 1;

    // 3. Byte offset of range relative to line start
    let col_start = range.start - line_start;
    let col_end = range.end - line_start;

    // 4. Print
    ceprintln!("   <s,r!>--></> Error at {line_number}:{col_start}");
    ceprintln!("    <s,r!>|</>");
    ceprintln!("<s,r!>{:>3} |</> {}", line_number, line_str.trim_end());

    let indent = " ".repeat(col_start);
    let carets = "^".repeat((col_end - col_start).max(1));
    ceprintln!("    <s,r!>|</> {}<s,y>{}</>", indent, carets);
    ceprintln!("    <s,r!>|</>");
    ceprintln!("    <s><r!>=</r!> {msg}");
}

//
// /// Pretty-print a *runtime* error with Python-style stack trace.
// /// `stack` must be ordered **caller → callee** (older frames first).
// pub fn print_rt(msg: &str, loc: &SourceLoc, stack: &[SourceLoc]) {
//     eprintln!("Traceback (most recent call last):");
//
//     // We want *oldest* first, so iterate the slice as-is.
//     for frame in stack {
//         print_frame(frame);
//     }
//     // The site that actually threw (`loc`)
//     print_frame(loc);
//
//     eprintln!("RuntimeError: {msg}");
// }
//
// /// Print one “File "...", line X” entry plus its source line.
// fn print_frame(loc: &SourceLoc) {
//     if let Some(entry) = SOURCE_MAP.read().unwrap().get(&loc.module_id) {
//         if let Some(line) = line(&entry.code, loc.span.start.line) {
//             eprintln!("  File \"{}\", line {}", entry.name, loc.span.start.line);
//             eprintln!("    {}", line.trim_end());
//         }
//     }
// }
//
// /// Fetch `n`-th (1-based) line from `src`.
// fn line<'a>(src: &'a str, n: u32) -> Option<&'a str> {
//     src.lines().nth((n - 1) as usize)
// }
