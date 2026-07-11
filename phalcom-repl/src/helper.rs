//! [`rustyline`] helper bundle wiring together completion, highlighting, and validation.
//!
//! [`PhalcomHelper`] implements the [`rustyline::Helper`] umbrella trait by
//! composing [`PhalcomCompleter`], [`PhalcomHighlighter`], and the
//! [`Validator`] logic from this module.
//!
//! > **Note:** this module is part of the *experimental* rustyline REPL stack
//! > (`src/rustyline/`).  The active REPL uses `reedline`; see `src/main.rs`.

use crate::completer::PhalcomCompleter;
use crate::highlighter::PhalcomHighlighter;
use rustyline::completion::Completer;
use rustyline::highlight::{CmdKind, Highlighter};
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::Helper;

/// Composite [`rustyline::Helper`] that bundles completion, highlighting, and
/// block-validation for the experimental rustyline REPL backend.
pub struct PhalcomHelper {
    /// The completion and hinting provider.
    pub completer: PhalcomCompleter,
    /// The syntax-highlighting provider.
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

/// Returns `true` when `s` contains an unmatched brace, an open string
/// literal, or ends with a trailing `{`.
///
/// This is a lightweight heuristic used by the validator to determine whether
/// the user needs to enter more text before the block is complete.
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
