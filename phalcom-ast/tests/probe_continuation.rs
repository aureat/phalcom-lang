//! U-REPL §D7 — truncated input is classified apart from wrong input.
//!
//! A REPL must decide, per line, whether to evaluate or to keep reading. That
//! decision needs the parser to distinguish *ran out of input* from *found the
//! wrong token*. Before §D7 it could not: every truncation surfaced as
//! `UnrecognizedToken { token: "", .. }`, EOF being modelled as a zero-length
//! token, so the only available check was sniffing for an empty token string —
//! a load-bearing implementation detail rather than a named signal.
//!
//! EOF is now routed to [`SyntaxErrorKind::UnrecognizedEof`], and this file
//! asserts the three-way classification §D7 specifies. It began life as a probe
//! that printed a table and asserted nothing; it is a guard now.
//!
//! The REPL half of §D7 (the reedline `Validator`, trailing `\`, blank-line
//! submit) consumes this signal and is specified separately.

use phalcom_ast::error::SyntaxErrorKind;
use phalcom_ast::parser::parse;

/// How a REPL should treat a given input.
#[derive(Debug, PartialEq, Eq)]
enum Verdict {
    /// Parsed with no errors — evaluate it.
    Complete,
    /// Ran out of input — keep reading.
    Incomplete,
    /// Genuinely malformed — report it now; more input will not help.
    Error,
}

/// Classifies `src` the way the REPL's validator will.
///
/// `Incomplete` means that the input can potentially become valid solely by
/// appending more source text. Fatal diagnostics take precedence over secondary
/// EOF diagnostics produced during parser recovery.
///
/// An ordinary string that reaches EOF before its closing quote is incomplete:
/// appending `"` can repair it. Once a physical newline occurs inside the
/// string, however, the source is irrecoverably malformed and is an error.
fn classify(src: &str) -> Verdict {
    let parsed = parse(src, 0);
    if parsed.errors.is_empty() {
        return Verdict::Complete;
    }
    // Fatal lexical errors take precedence over secondary parser EOF errors.
    if parsed.errors.iter().any(|error| {
        matches!(
            error.kind,
            SyntaxErrorKind::RawNewlineInString
                | SyntaxErrorKind::InvalidMultilineStringOpening
                | SyntaxErrorKind::InvalidMultilineStringIndentation
                | SyntaxErrorKind::InvalidMultilineStringLineEnding
        )
    }) {
        return Verdict::Error;
    }
    // Secondary parser EOF errors
    if parsed.errors.iter().any(|error| matches!(error.kind, SyntaxErrorKind::UnrecognizedEof { .. })) {
        return Verdict::Incomplete;
    }
    // Error otherwise as well
    Verdict::Error
}

#[test]
fn classifies_truncated_wrong_and_finished_input() {
    let cases = [
        // Complete.
        ("let x = 1", Verdict::Complete),
        ("1 + 1", Verdict::Complete),
        ("", Verdict::Complete),
        ("let s = \"abcdef\"", Verdict::Complete),
        // Incomplete — the parser reached EOF still wanting more.
        ("class Foo {", Verdict::Incomplete),
        ("class Foo {\n  bar() { 1 }", Verdict::Incomplete),
        ("let x = 1 +", Verdict::Incomplete),
        ("let x =", Verdict::Incomplete),
        ("foo(1,", Verdict::Incomplete),
        ("[1, 2,", Verdict::Incomplete),
        ("if (x) {", Verdict::Incomplete),
        // String literal incomplete
        ("let s = \"abc", Verdict::Incomplete),
        // LF
        ("let s = \"abc\ndef", Verdict::Error),
        ("let s = \"abc\ndef\"", Verdict::Error),
        // CRLF
        ("let s = \"abc\r\ndef", Verdict::Error),
        ("let s = \"abc\r\ndef\"", Verdict::Error),
        // CR
        ("let s = \"abc\rdef", Verdict::Error),
        ("let s = \"abc\rdef\"", Verdict::Error),
        // Multiline string incomplete / complete / fatal
        ("let s = \"\"\"", Verdict::Incomplete),
        ("let s = \"\"\"   ", Verdict::Incomplete),
        ("let s = \"\"\"\n    hello", Verdict::Incomplete),
        ("let s = \"\"\"\n    hello\n", Verdict::Incomplete),
        ("let s = \"\"\"\n    hello\n    \"\"\"", Verdict::Complete),
        ("let s = \"\"\"same line\n    \"\"\"", Verdict::Error),
        ("let s = \"\"\"\n    first\n  second\n    \"\"\"", Verdict::Error),
        // Error — a real token in the wrong place. More input cannot fix these,
        // so a REPL must not sit waiting for it.
        ("let x = )", Verdict::Error),
        ("1 +* 2", Verdict::Error),
    ];

    for (src, want) in cases {
        assert_eq!(classify(src), want, "misclassified {src:?}");
    }
}

/// The EOF error names what was expected, which is the readability half of the
/// change: `Expected "}"` alone never said the input had simply run out.
#[test]
fn eof_error_reports_end_of_file_and_what_was_expected() {
    let parsed = parse("class Foo {", 0);
    let err = parsed.errors.first().expect("truncated class body is an error");

    match &err.kind {
        SyntaxErrorKind::UnrecognizedEof { expected } => {
            assert!(
                expected.iter().any(|e| e.contains('}')),
                "expected set should still name the missing brace, got {expected:?}"
            );
        }
        other => panic!("truncation must be UnrecognizedEof, got {other:?}"),
    }

    let rendered = err.kind.to_string();
    assert!(
        rendered.contains("Unexpected end of file"),
        "message should say the input ended, got {rendered:?}"
    );
}

/// A wrong token keeps carrying its text — `UnrecognizedEof` must not swallow
/// the ordinary case.
#[test]
fn wrong_token_still_reports_the_offending_text() {
    let parsed = parse("let x = )", 0);
    let err = parsed.errors.first().expect("`)` is not an expression");

    match &err.kind {
        SyntaxErrorKind::UnrecognizedToken { token, .. } => {
            assert_eq!(token, ")", "the offending token's text must survive");
        }
        other => panic!("a real token in the wrong place stays UnrecognizedToken, got {other:?}"),
    }
}
