//! Syntax highlighting for the Phalcom REPL.
//!
//! [`PhalcomHighlighter`] implements [`reedline::Highlighter`] and applies
//! token-class colours (keywords, strings, numbers, comments, identifiers,
//! brackets) to each line the user types, using a priority mask so that
//! string/comment spans suppress inner matches.

use crate::common::KEYWORDS;
use nu_ansi_term::{Color, Style};
use once_cell::sync::Lazy;
use reedline::{Highlighter, StyledText};
use regex::Regex;
use std::cell::RefCell;

static RE_STRING: Lazy<Regex> = Lazy::new(|| Regex::new(r#""(?:\\.|[^"\\])*""#).unwrap());
static RE_LINE_COMMENT: Lazy<Regex> = Lazy::new(|| Regex::new(r"//[^\n]*").unwrap());
static RE_BLOCK_COMMENT: Lazy<Regex> = Lazy::new(|| Regex::new(r"/\*([^*]|\*[^/])*\*/").unwrap());
static RE_KEYWORD: Lazy<Regex> = Lazy::new(|| {
    let keys = KEYWORDS.iter().map(|kw| format!(r"\b{}\b", regex::escape(kw))).collect::<Vec<_>>().join("|");
    Regex::new(&format!(r"\b(?:{keys})\b")).unwrap()
});
static RE_BOOL_NIL: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b(true|false|nil)\b").unwrap());
static RE_NUMBER: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?:\d+\.\d*|\.\d+|\d+)(?:[eE][+-]?\d+)?").unwrap());
static RE_IDENT: Lazy<Regex> = Lazy::new(|| Regex::new(r"[A-Za-z_][A-Za-z0-9_]*").unwrap());
static RE_CLASS_IDENT: Lazy<Regex> = Lazy::new(|| Regex::new(r"[A-Z][A-Za-z0-9_]*").unwrap());
static RE_BRACKET: Lazy<Regex> = Lazy::new(|| Regex::new(r"[{}()\[\]]").unwrap());

#[inline]
fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}
#[inline]
fn prev_char(s: &str, i: usize) -> Option<char> {
    if i == 0 { None } else { s[..i].chars().next_back() }
}
#[inline]
fn next_char(s: &str, i: usize) -> Option<char> {
    if i >= s.len() { None } else { s[i..].chars().next() }
}

fn has_non_ident_boundaries(s: &str, start: usize, end: usize) -> bool {
    let left_ok = prev_char(s, start).map(|c| !is_ident_char(c)).unwrap_or(true);
    let right_ok = next_char(s, end).map(|c| !is_ident_char(c)).unwrap_or(true);
    left_ok && right_ok
}
fn followed_by_paren(s: &str, mut end: usize) -> bool {
    let bytes = s.as_bytes();
    while end < s.len() && bytes[end].is_ascii_whitespace() {
        end += 1;
    }
    matches!(next_char(s, end), Some('('))
}
fn not_followed_by_paren(s: &str, end: usize) -> bool {
    !followed_by_paren(s, end)
}

#[derive(Clone, Copy)]
struct Span {
    s: usize,
    e: usize,
    /// The style to apply; stored for future use in overlap painting.
    #[allow(dead_code)]
    style: Style,
}

impl Span {
    fn new(s: usize, e: usize, style: Style) -> Self {
        Span { s, e, style }
    }
}


/// Syntax highlighter for Phalcom source input in the REPL.
///
/// Applies distinct colours to keywords, boolean/nil literals, numbers,
/// strings, comments, bracket pairs, method calls, class names, and lowercase
/// variable identifiers.  Colour assignments are applied in priority order
/// via a byte-level mask so that string and comment spans are never
/// over-painted by identifier passes.
pub struct PhalcomHighlighter;

impl Default for PhalcomHighlighter {
    fn default() -> Self {
        Self
    }
}

impl Highlighter for PhalcomHighlighter {
    fn highlight(&self, line: &str, _pos: usize) -> StyledText {
        let spans = RefCell::new(Vec::<Span>::new());

        let mut styled = StyledText::new();
        styled.push((Style::default(), line.to_string()));

        let mut color = |s: usize, e: usize, style: Style| {
            if s < e && e <= line.len() {
                styled.style_range(s, e, style);
            }
            spans.borrow_mut().push(Span::new(s, e, style));
        };

        // keywords (light blue + bold)
        let kw_style = Style::new().fg(Color::Magenta).bold();
        // brighter pastel blue for methods
        let method_style = Style::new().fg(Color::Fixed(75)).bold();
        let str_style = Style::new().fg(Color::Green);
        let num_style = Style::new().fg(Color::Fixed(216));
        let class_style = Style::new().fg(Color::Fixed(222)).bold();
        let var_style = Style::new().fg(Color::Red);
        let bool_style = Style::new().fg(Color::Fixed(216));
        let comment_style = Style::new().dimmed();
        let bracket_style = Style::new().fg(Color::Fixed(223)).bold();

        let mut mask = vec![false; line.len()];

        for m in RE_STRING.find_iter(line) {
            mask[m.start()..m.end()].fill(true);
        }
        for m in RE_LINE_COMMENT.find_iter(line) {
            mask[m.start()..m.end()].fill(true);
        }
        for m in RE_BLOCK_COMMENT.find_iter(line) {
            mask[m.start()..m.end()].fill(true);
        }

        for m in RE_STRING.find_iter(line) {
            color(m.start(), m.end(), str_style);
        }

        for m in RE_LINE_COMMENT.find_iter(line) {
            color(m.start(), m.end(), comment_style);
        }

        for m in RE_BLOCK_COMMENT.find_iter(line) {
            color(m.start(), m.end(), comment_style);
        }

        for m in RE_BRACKET.find_iter(line) {
            color(m.start(), m.end(), bracket_style);
        }

        // keywords, bool/nil, numbers
        for m in RE_KEYWORD.find_iter(line) {
            let (s, e) = (m.start(), m.end());
            if (s..e).any(|i| mask[i]) {
                continue;
            }
            color(s, e, kw_style);
        }

        for m in RE_BOOL_NIL.find_iter(line) {
            let (s, e) = (m.start(), m.end());
            if (s..e).any(|i| mask[i]) {
                continue;
            }
            color(s, e, bool_style);
        }

        for m in RE_NUMBER.find_iter(line) {
            let (s, e) = (m.start(), m.end());
            if (s..e).any(|i| mask[i]) {
                continue;
            }
            if !has_non_ident_boundaries(line, s, e) {
                continue;
            }
            color(s, e, num_style);
        }

        // methods
        for m in RE_IDENT.find_iter(line) {
            let (s, e) = (m.start(), m.end());
            if (s..e).any(|i| mask[i]) {
                continue;
            }
            if followed_by_paren(line, e) {
                color(s, e, method_style);
            }
        }

        // classes (Capitalized, not followed by '(')
        for m in RE_CLASS_IDENT.find_iter(line) {
            let (s, e) = (m.start(), m.end());
            if (s..e).any(|i| mask[i]) {
                continue;
            }
            if spans.borrow().iter().any(|sp| s < sp.e && sp.s < e) {
                continue;
            } // already colored
            if not_followed_by_paren(line, e) {
                color(s, e, class_style);
            }
        }

        // variables (lowercase identifiers not already colored)
        for m in RE_IDENT.find_iter(line) {
            let (s, e) = (m.start(), m.end());
            if (s..e).any(|i| mask[i]) {
                continue;
            }
            if spans.borrow().iter().any(|sp| s < sp.e && sp.s < e) {
                continue;
            }
            if line[s..].chars().next().map(|c| c.is_lowercase()).unwrap_or(false) {
                color(s, e, var_style);
            }
        }

        styled
    }
}
