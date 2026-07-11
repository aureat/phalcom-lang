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
fn punctuation_and_ranges() {
    insta::assert_debug_snapshot!(tokens("a.b :: c .. d ... e -> f => g"));
}
