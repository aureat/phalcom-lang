//! Snapshot tests for the lexer.
//!
//! Each test tokenizes a source string and snapshots the resulting token
//! stream. To review/accept changes after intentionally altering the lexer:
//!
//! ```sh
//! cargo insta review          # interactive, needs `cargo install cargo-insta`
//! INSTA_UPDATE=always cargo test -p phalcom-ast   # accept all
//! ```

use phalcom_ast::lexer::Lexer;
use phalcom_ast::token::Token;

/// Tokenize `src`, returning the tokens (without spans). Panics if the input
/// does not lex cleanly — these fixtures are all valid Phalcom.
fn tokens(src: &str) -> Vec<Token> {
    Lexer::new(src)
        .map(|spanned| spanned.expect("fixture should lex without error").1)
        .collect()
}

#[test]
fn let_binding() {
    insta::assert_debug_snapshot!(tokens("let x = 42"));
}

#[test]
fn arithmetic_and_compound_assignment() {
    insta::assert_debug_snapshot!(tokens("total += a * b - c / 2"));
}

#[test]
fn comparison_operators() {
    insta::assert_debug_snapshot!(tokens("a == b != c <= d >= e < f > g"));
}

#[test]
fn function_definition() {
    insta::assert_debug_snapshot!(tokens("fn add(a, b) { return a + b }"));
}

#[test]
fn class_with_static_method() {
    insta::assert_debug_snapshot!(tokens(
        "class Point {\n  static origin { self }\n}"
    ));
}

#[test]
fn string_and_number_literals() {
    insta::assert_debug_snapshot!(tokens(r#"let s = "hello" let n = 3.14"#));
}

#[test]
fn keywords_and_operators() {
    insta::assert_debug_snapshot!(tokens("if a and b or not c { self } else { super }"));
}

#[test]
fn line_comment_is_skipped() {
    insta::assert_debug_snapshot!(tokens("let x = 1 // trailing comment\nlet y = 2"));
}

#[test]
fn block_comment_is_skipped() {
    // `/* … */` block comments are trivia (D1). A newline *inside* the block
    // comment is consumed with it and never leaks a `Token::Newline`.
    insta::assert_debug_snapshot!(tokens(
        "let x = 1 /* block\n comment */ let y = 2"
    ));
}

#[test]
fn newline_after_operator_is_suppressed() {
    // D3: a `\n` after `+` (which cannot end a statement) is swallowed, so
    // `a +\nb` lexes as one expression with no interior `Token::Newline`.
    insta::assert_debug_snapshot!(tokens("a +\nb"));
}

#[test]
fn newline_after_value_is_preserved() {
    // D3 guard: `a\nb` keeps its `Token::Newline` — an identifier can end a
    // statement, so the newline still terminates it.
    insta::assert_debug_snapshot!(tokens("a\nb"));
}

#[test]
fn numeric_digit_separators() {
    // `_` separators (D2) are stripped before decoding: `1_000_000` reads as
    // `1000000.0`, and `_` works on both sides of the decimal point.
    insta::assert_debug_snapshot!(tokens("1_000_000 1_000.500_5"));
}

#[test]
fn punctuation_and_ranges() {
    insta::assert_debug_snapshot!(tokens("a.b :: c .. d ... e -> f => g"));
}

#[test]
fn modulo_operator() {
    // `%` / `%=` (Token::Percent / Token::PercentEqual): current-behavior
    // snapshot for the modulo operator the arithmetic corpus exercises
    // (`tests/lang/arithmetic/arithmetic_modulo.ph`: `10 % 3`).
    insta::assert_debug_snapshot!(tokens("10 % 3 total %= 2"));
}
