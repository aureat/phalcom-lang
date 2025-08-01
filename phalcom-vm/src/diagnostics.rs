use crate::interner::Symbol;
use crate::module::ModuleId;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::ops::Range;
use std::sync::{Arc, RwLock};

lazy_static! {
    pub static ref SOURCE_MAP: RwLock<HashMap<Symbol, Arc<String>>> = RwLock::new(HashMap::new());
}

/// One-based line/column.
// #[derive(Clone, Copy, Debug, Eq, PartialEq)]
// pub struct Position {
//     pub line: u32,
//     pub column: u32,
// }

/// Inclusive start, exclusive end (like Rust ranges).
// #[derive(Clone, Copy, Debug, Eq, PartialEq)]
// pub struct Span {
//     pub start: Position,
//     pub end: Position,
// }

pub type Span = Range<usize>;

/// A pointer into a specific module’s source.
#[derive(Clone, Debug)]
pub struct SourceLoc {
    pub module_id: ModuleId,
    pub span: Span,
}
//
// /// Pretty-print a *parse / compile* time error (single location, no stack).
// pub fn print_parse(msg: &str, loc: &SourceLoc) {
//     if let Some(entry) = SOURCE_MAP.read().unwrap().get(&loc.module_id) {
//         if let Some(line) = line(&entry.code, loc.span.start.line) {
//             // Header (similar to Rust compiler style)
//             eprintln!("--> {}:{}:{}", entry.name, loc.span.start.line, loc.span.start.column);
//             // Gutter + source line
//             eprintln!(" |");
//             eprintln!("{:>4} | {}", loc.span.start.line, line.trim_end());
//
//             // Caret underline ^^^^
//             let indent = " ".repeat((loc.span.start.column - 1) as usize);
//             let carets = "^".repeat(((loc.span.end.column - loc.span.start.column).max(1)) as usize);
//             eprintln!(" |   {}{}", indent, carets);
//
//             // Final summary
//             eprintln!("\nSyntaxError: {msg}");
//             return;
//         }
//     }
//     // Fallback if we have no source
//     eprintln!("SyntaxError: {msg} (at unknown location)");
// }
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
