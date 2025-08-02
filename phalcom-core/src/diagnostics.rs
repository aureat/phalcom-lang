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
    pub source: Arc<String>,
}

// use std::ops::Range;

pub fn print_line_information(source: &str, range: Range<usize>) {
    let mut line_start = 0;
    let mut line_number = 1;

    // Identify which line the range starts in
    for (idx, line) in source.lines().enumerate() {
        let line_end = line_start + line.len();
        if range.start >= line_start && range.start <= line_end {
            line_number = idx + 1;
            break;
        }
        line_start = line_end + 1; // +1 for '\n'
    }

    let lines: Vec<&str> = source.lines().collect();
    let current = line_number - 1;

    let col_start = range.start - lines[..current].iter().map(|l| l.len() + 1).sum::<usize>();
    let col_end = range.end - lines[..current].iter().map(|l| l.len() + 1).sum::<usize>();

    ceprintln!("   <s,r!>--></> Error at {}:{}", line_number, col_start);
    ceprintln!("    <s,r!>|</>");

    if current > 0 {
        ceprintln!("<s,r!>{:>3} |</> {}", current, lines[current - 1].trim_end());
    }

    ceprintln!("<s,r!>{:>3} |</> {}", line_number, lines[current].trim_end());

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

/// Pretty-print a *runtime* error with Python-style stack trace.
/// `stack` must be ordered **caller → callee** (older frames first).
pub fn print_rt(msg: &str, stack: &[SourceLoc]) {
    ceprintln!("<s,r!>Traceback (most recent call last):");
    ceprintln!("    <s><r!>=</r!> {msg}");

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
    print_line_information(&loc.source, loc.span.start..loc.span.end);
}

// /// Fetch `n`-th (1-based) line from `src`.
// fn line<'a>(src: &'a str, n: u32) -> Option<&'a str> {
//     src.lines().nth((n - 1) as usize)
// }
