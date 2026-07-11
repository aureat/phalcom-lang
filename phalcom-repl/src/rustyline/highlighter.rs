// use fancy_regex::Regex;
use once_cell::sync::Lazy;
use regex::Regex;
use rustyline::highlight::{CmdKind, Highlighter, MatchingBracketHighlighter};
use std::cell::RefCell;

// ========== Colors (ANSI) ==========
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";

const BLUE: &str = "\x1b[94m";
const CYAN: &str = "\x1b[96m";
const GREEN: &str = "\x1b[92m";
const MAGENTA: &str = "\x1b[95m";
const YELLOW: &str = "\x1b[93m";
const DIM: &str = "\x1b[2m";
const LIGHT_YELLOW: &str = "\x1b[93m"; // classes
const LIGHT_RED: &str = "\x1b[91m"; // variables
const LIGHT_ORANGE: &str = "\x1b[38;5;215m";

// ========== Regexes ==========
// Order matters: we’ll first carve out strings/comments so other rules don’t color inside them.
static RE_STRING: Lazy<Regex> = Lazy::new(|| {
    // Match " ... " with backslash escapes, multi-line allowed in REPL buffer
    Regex::new(r#""(?:\\.|[^"\\])*""#).unwrap()
});

static RE_LINE_COMMENT: Lazy<Regex> = Lazy::new(|| Regex::new(r"//[^\n]*").unwrap());
static RE_BLOCK_COMMENT: Lazy<Regex> = Lazy::new(|| Regex::new(r"/\*([^*]|\*[^/])*\*/").unwrap());

// Keywords (standalone)
static RE_KEYWORD: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(class|import|for|while|if|else|return|break|continue|let|and|or|not)\b").unwrap());

// Literals
static RE_BOOL_NIL: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(true|false|nil)\b").unwrap());

// Numbers (123, 3.14, .5, 1e10, 2.3e-4)
// static RE_NUMBER: Lazy<Regex> = Lazy::new(|| {
//     Regex::new(
//         r"(?x)
//         (?<![\w_])          # not preceded by ident
//         (?:\d+\.\d*|\.\d+|\d+)(?:[eE][+-]?\d+)?  # float-ish
//         (?![\w_])
//     ",
//     )
//     .unwrap()
// });

// Numbers: match the numeric core; we’ll check boundaries in code
static RE_NUMBER: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?:\d+\.\d*|\.\d+|\d+)(?:[eE][+-]?\d+)?").unwrap());

// Any identifier (for method/class/var passes)
static RE_IDENT: Lazy<Regex> = Lazy::new(|| Regex::new(r"[A-Za-z_][A-Za-z0-9_]*").unwrap());

// Capitalized identifiers (for class-ish)
static RE_CLASS_IDENT: Lazy<Regex> = Lazy::new(|| Regex::new(r"[A-Z][A-Za-z0-9_]*").unwrap());

// Method call names: identifier immediately followed by '('
// static RE_METHOD_CALL: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*(?=\()").unwrap());

// Class-like identifiers: Start with capital, not followed by '(' (avoid double-highlighting methods)
// static RE_CLASS_ID: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b([A-Z][A-Za-z0-9_]*)\b(?!\s*\()").unwrap());

// Lowercase-ish variables (not keywords/bools/nil)
static RE_VAR_ID: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b([a-z_][A-Za-z0-9_]*)\b").unwrap());

#[inline]
fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

#[inline]
fn prev_char(input: &str, idx: usize) -> Option<char> {
    input[..idx].chars().rev().next()
}
#[inline]
fn next_char(input: &str, idx: usize) -> Option<char> {
    input[idx..].chars().next()
}

/// Check "not part of identifier" boundaries for a range.
// fn has_non_ident_boundaries(s: &str, start: usize, end: usize) -> bool {
//     let left_ok = match prev_char(s, start) {
//         None => true,
//         Some(c) => !is_ident_char(c),
//     };
//     let right_ok = match next_char(s, end) {
//         None => true,
//         Some(c) => !is_ident_char(c),
//     };
//     left_ok && right_ok
// }

fn has_non_ident_boundaries(s: &str, start: usize, end: usize) -> bool {
    let left_ok = if start == 0 {
        true
    } else {
        !is_ident_char(s[..start].chars().rev().next().unwrap())
    };
    let right_ok = if end >= s.len() {
        true
    } else {
        !is_ident_char(s[end..].chars().next().unwrap())
    };
    left_ok && right_ok
}

/// After a match, skip spaces and see if the next char is '('
fn followed_by_paren(s: &str, mut end: usize) -> bool {
    // step to byte boundary of next char(s)
    let bytes = s.as_bytes();
    while end < s.len() && bytes[end].is_ascii_whitespace() {
        end += 1;
    }
    matches!(next_char(s, end), Some('('))
}

/// After a match, skip spaces and ensure the next char is NOT '('
fn not_followed_by_paren(s: &str, end: usize) -> bool {
    !followed_by_paren(s, end)
}

pub struct PhalcomHighlighter {
    match_brackets: MatchingBracketHighlighter,
}

impl Default for PhalcomHighlighter {
    fn default() -> Self {
        Self {
            match_brackets: MatchingBracketHighlighter::new(),
        }
    }
}

impl Highlighter for PhalcomHighlighter {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> std::borrow::Cow<'l, str> {
        // 1) Build a mask so we don't recolor inside strings/comments
        let mut mask = vec![false; line.len()];
        for m in RE_STRING.find_iter(line) {
            for i in m.start()..m.end() {
                mask[i] = true;
            }
        }
        for m in RE_LINE_COMMENT.find_iter(line) {
            for i in m.start()..m.end() {
                mask[i] = true;
            }
        }
        for m in RE_BLOCK_COMMENT.find_iter(line) {
            for i in m.start()..m.end() {
                mask[i] = true;
            }
        }

        // Helper to paint matches that are NOT masked
        let mut out = String::with_capacity(line.len() + 32);

        // First pass: actually splice colored segments in one go.
        // We'll collect spans to color (start,end,color,style) then render left→right.
        #[derive(Clone, Copy)]
        struct Span {
            s: usize,
            e: usize,
            color: &'static str,
            bold: bool,
        }

        let mut spans: RefCell<Vec<Span>> = RefCell::new(Vec::new());
        // let mut spans: Vec<Span> = Vec::new();

        // Always color strings/comments first
        for m in RE_STRING.find_iter(line) {
            spans.borrow_mut().push(Span {
                s: m.start(),
                e: m.end(),
                color: GREEN,
                bold: false,
            });
        }
        for m in RE_LINE_COMMENT.find_iter(line) {
            spans.borrow_mut().push(Span {
                s: m.start(),
                e: m.end(),
                color: DIM,
                bold: false,
            });
        }
        for m in RE_BLOCK_COMMENT.find_iter(line) {
            spans.borrow_mut().push(Span {
                s: m.start(),
                e: m.end(),
                color: DIM,
                bold: false,
            });
        }

        // Function to push regex matches if *not* masked (i.e., code regions)
        let mut push_unmasked = |re: &Regex, color: &'static str, bold: bool| {
            for m in re.find_iter(line) {
                // skip any fully masked region (inside string/comment)
                if (m.start()..m.end()).any(|i| mask.get(i).copied().unwrap_or(false)) {
                    continue;
                }
                spans.borrow_mut().push(Span {
                    s: m.start(),
                    e: m.end(),
                    color,
                    bold,
                });
            }
        };

        push_unmasked(&RE_KEYWORD, MAGENTA, true);
        push_unmasked(&RE_BOOL_NIL, LIGHT_ORANGE, false);

        // push_unmasked(&RE_NUMBER, MAGENTA, false);

        for m in RE_NUMBER.find_iter(line) {
            let s = m.start();
            let e = m.end();
            if (s..e).any(|i| mask.get(i).copied().unwrap_or(false)) {
                continue;
            }
            if has_non_ident_boundaries(line, s, e) {
                spans.borrow_mut().push(Span {
                    s,
                    e,
                    color: LIGHT_ORANGE,
                    bold: false,
                });
            }
        }

        // push_unmasked(&RE_METHOD_CALL, CYAN, true); // method names: bold cyan
        for m in RE_IDENT.find_iter(line) {
            let s = m.start();
            let e = m.end();
            if (s..e).any(|i| mask.get(i).copied().unwrap_or(false)) {
                continue;
            }
            if followed_by_paren(line, e) {
                spans.borrow_mut().push(Span { s, e, color: BLUE, bold: true });
            }
            let first = line[s..].chars().next().unwrap();
            if first.is_lowercase() {
                spans.borrow_mut().push(Span {
                    s,
                    e,
                    color: LIGHT_RED,
                    bold: false,
                });
            }
        }

        // push_unmasked(&RE_CLASS_ID, YELLOW, true); // class-like: bold yellow

        // class-like identifiers: Capitalized, and NOT followed by '('
        for m in RE_CLASS_IDENT.find_iter(line) {
            let s = m.start();
            let e = m.end();
            if (s..e).any(|i| mask.get(i).copied().unwrap_or(false)) {
                continue;
            }

            // Don’t recolor if this exact range was already marked as a method name
            if spans.borrow().iter().any(|sp| s < sp.e && sp.s < e) {
                continue;
            }

            if not_followed_by_paren(line, e) {
                spans.borrow_mut().push(Span {
                    s,
                    e,
                    color: LIGHT_YELLOW,
                    bold: true,
                });
            }
        }

        // Variables last; we’ll exclude ones already covered (keywords/bools/classes/methods)
        // We’ll do that by skipping if this range already intersects another span.
        'vars: for m in RE_VAR_ID.find_iter(line) {
            if (m.start()..m.end()).any(|i| mask.get(i).copied().unwrap_or(false)) {
                continue;
            }
            for s in spans.borrow().iter() {
                if m.start() < s.e && s.s < m.end() {
                    continue 'vars;
                } // overlap → already colored
            }
            spans.borrow_mut().push(Span {
                s: m.start(),
                e: m.end(),
                color: RESET,
                bold: false,
            }); // keep default; change to DIM if you want
        }

        // Render: sort spans by start; merge overlaps by priority (earlier spans win).
        spans.borrow_mut().sort_by_key(|sp| (sp.s, usize::MAX - sp.e)); // stable order, longer first when same start

        // Collapse overlaps: keep first span covering each byte
        let mut taken = vec![false; line.len()];
        let mut final_spans: Vec<Span> = Vec::new();

        for sp in spans.borrow().iter() {
            if sp.s >= sp.e || sp.e > line.len() {
                continue;
            }
            if (sp.s..sp.e).any(|i| taken[i]) {
                continue;
            }
            for i in sp.s..sp.e {
                taken[i] = true;
            }
            final_spans.push(*sp);
        }

        // Stitch string together
        let mut cursor = 0;
        for sp in final_spans {
            if cursor < sp.s {
                out.push_str(&line[cursor..sp.s]);
            }
            if sp.bold {
                out.push_str(BOLD);
            }
            out.push_str(sp.color);
            out.push_str(&line[sp.s..sp.e]);
            out.push_str(RESET);
            cursor = sp.e;
        }
        if cursor < line.len() {
            out.push_str(&line[cursor..]);
        }

        std::borrow::Cow::Owned(out)
    }

    fn highlight_char(&self, line: &str, pos: usize, kind: CmdKind) -> bool {
        self.match_brackets.highlight_char(line, pos, kind)
    }
}
