use crate::completer::PhalcomCompleter;
use crate::highlighter::PhalcomHighlighter;
use rustyline::completion::Completer;
use rustyline::highlight::{CmdKind, Highlighter};
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::Helper;

pub struct PhalcomHelper {
    pub completer: PhalcomCompleter,
    pub highlighter: PhalcomHighlighter,
}

impl Helper for PhalcomHelper {}

impl Validator for PhalcomHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        let s = ctx.input();
        if is_incomplete_block(s) {
            Ok(ValidationResult::Incomplete)
        } else {
            Ok(ValidationResult::Valid(None))
        }
    }
}

pub fn is_incomplete_block(s: &str) -> bool {
    // quick & dirty: unmatched braces/quotes or trailing open block
    let mut braces = 0usize;
    let mut in_str = false;
    let mut prev = '\0';
    for c in s.chars() {
        match c {
            '"' if prev != '\\' => in_str = !in_str,
            '{' if !in_str => braces += 1,
            '}' if !in_str && braces > 0 => braces -= 1,
            _ => {}
        }
        prev = c;
    }
    in_str || braces > 0 || s.trim_end().ends_with('{')
}

impl Highlighter for PhalcomHelper {
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> std::borrow::Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize, kind: CmdKind) -> bool {
        self.highlighter.highlight_char(line, pos, kind)
    }
}

impl Hinter for PhalcomHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &rustyline::Context<'_>) -> Option<Self::Hint> {
        self.completer.hint(line, pos, ctx)
    }
}

impl Completer for PhalcomHelper {
    type Candidate = rustyline::completion::Pair;

    fn complete(&self, line: &str, pos: usize, ctx: &rustyline::Context<'_>) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        self.completer.complete(line, pos, ctx)
    }
}
