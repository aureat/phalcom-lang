//! Snapshot tests for the parser.
//!
//! These capture the parser's output (a `Program` AST on success, or the
//! `SyntaxError` on failure) as a regression baseline. Both outcomes are
//! legitimate snapshots — the point is to detect *unintended* changes in
//! parsing behavior. To accept intentional changes:
//!
//! ```sh
//! INSTA_UPDATE=always cargo test -p phalcom-ast
//! ```
//!
//! Note on grammar coverage: the grammar accepts a program with *or without* a
//! trailing newline at end-of-input (see the `trailing_newline_*` tests — U0
//! fixed the prior panic on `\n`-terminated files). It does not yet accept
//! `fn`/`if`/`@`-annotations or a bare `class` declaration *without* a trailing
//! newline at top level. Fixtures below reflect that reality — the
//! `*_current_limitation` tests pin behavior we expect to change as the grammar
//! grows, so their snapshots act as executable TODOs.

use phalcom_ast::parse_source;

/// Parse `src` and render the result deterministically for snapshotting.
fn parse(src: &str) -> String {
    match parse_source(src, 0) {
        Ok(program) => format!("{program:#?}"),
        Err(err) => format!("Err: {err:?}"),
    }
}

// --- Error recovery: one bad input yields many diagnostics ---

#[test]
fn recovers_across_multiple_broken_statements() {
    // The hand-written parser synchronises at statement boundaries, so a file
    // with several syntax errors reports each one instead of stopping at the
    // first. Snapshot the recovered messages to pin the recovery behaviour.
    //
    // Each line ends in a token that *can* end a statement (a number or a `)`)
    // so the separating newline survives D3's continuation rule
    // (`lexer::suppresses_following_newline`) and the statements stay distinct;
    // lines ending in an operator would legitimately be joined.
    let result = phalcom_ast::parse("let 9\nreturn )\nlet 9\n", 0);
    let rendered: Vec<String> = result.errors.iter().map(ToString::to_string).collect();
    assert!(
        result.errors.len() >= 3,
        "expected at least three recovered errors, got {rendered:?}"
    );
    insta::assert_debug_snapshot!(rendered);
}

/// Parse `src` and render the *user-facing* `Display` of any error. This pins
/// F9: `SyntaxError`'s `Display` used to be `todo!()`, so any parse error
/// panicked instead of producing a diagnostic.
fn parse_display(src: &str) -> String {
    match parse_source(src, 0) {
        Ok(_) => "Ok".to_string(),
        Err(err) => err.to_string(),
    }
}

// --- Inputs that parse to an AST today ---

#[test]
fn let_binding() {
    insta::assert_snapshot!(parse("let x = 1 + 2"));
}

#[test]
fn assignment() {
    insta::assert_snapshot!(parse("x = 1"));
}

#[test]
fn member_access() {
    insta::assert_snapshot!(parse("x.y"));
}

#[test]
fn unary_expression() {
    insta::assert_snapshot!(parse("!true"));
}

#[test]
fn return_statement() {
    insta::assert_snapshot!(parse("return 1"));
}

#[test]
fn multiple_statements() {
    insta::assert_snapshot!(parse("let x = 1\nlet y = 2"));
}

#[test]
fn multiple_statements_semicolon_separated() {
    // Companion to `multiple_statements`: the corpus also exercises
    // semicolon-separated statements on one line
    // (`tests/lang/lexical/lexical_multi_statement_semicolon.ph`).
    insta::assert_snapshot!(parse("System.print(1); System.print(2)"));
}

#[test]
fn comparison_operators_parse() {
    // Current-behavior snapshot for `< > <= >=` as parsed binary expressions
    // (the lexer-level tokenization is already pinned by
    // `comparison_operators` in `tests/lexer.rs`).
    insta::assert_snapshot!(parse("a < b\na > b\na <= b\na >= b"));
}

// --- Trailing newline / EOF handling (U0 / F10) ---

#[test]
fn trailing_newline_parses() {
    // A single statement followed by a trailing `\n` (the shape of virtually
    // every real `.ph` file) used to panic; it now parses to an AST.
    insta::assert_snapshot!(parse("let x = 1\n"));
}

#[test]
fn trailing_blank_lines_and_whitespace_parse() {
    // Leading/trailing blank lines and trailing spaces around a statement all
    // parse cleanly to the same single-statement program.
    insta::assert_snapshot!(parse("\n\nlet x = 1\n  \n"));
}

#[test]
fn empty_input_parses() {
    insta::assert_snapshot!(parse(""));
}

#[test]
fn whitespace_only_input_parses() {
    insta::assert_snapshot!(parse("\n  \n"));
}

#[test]
fn class_declaration_with_trailing_newline_parses() {
    // Companion to `class_declaration_current_limitation`: with a terminating
    // newline the compound `class` statement parses (the bare, newline-less
    // form remains a separate grammar gap).
    insta::assert_snapshot!(parse("class Point {}\n"));
}

// --- F9: parse errors render via `Display` instead of panicking ---

#[test]
fn error_display_is_not_a_panic() {
    // Exercises `SyntaxError`'s `Display` impl directly: a malformed program
    // must produce a readable one-line diagnostic (with a span), never `todo!()`.
    insta::assert_snapshot!(parse_display("let = "));
}

#[test]
fn error_display_trailing_garbage() {
    insta::assert_snapshot!(parse_display("1 + )"));
}

// --- Documented current limitations (expected to change; snapshots are TODOs) ---

#[test]
fn class_declaration_current_limitation() {
    // Full `class` declarations do not yet parse as a complete program: the
    // class rule wants a trailing newline while the program rule then expects a
    // further statement. When the grammar is fixed, this snapshot will flip to
    // an AST and should be renamed.
    insta::assert_snapshot!(parse("class Point {}"));
}

// --- Genuine syntax errors (pin the error shape) ---

#[test]
fn error_missing_expression() {
    insta::assert_snapshot!(parse("let x = "));
}

#[test]
fn error_unexpected_token() {
    insta::assert_snapshot!(parse("let = )"));
}

// --- Block Parsing Tests (U4) ---

#[test]
fn braced_block_zero_params() {
    insta::assert_snapshot!(parse("{ System.print(\"hi\") }"));
}

#[test]
fn braced_block_params() {
    insta::assert_snapshot!(parse("{ acc, n => acc + n }"));
}

#[test]
fn unbraced_block_single_param() {
    insta::assert_snapshot!(parse("n => n * 2"));
}

#[test]
fn trailing_block_sugar_method_call() {
    insta::assert_snapshot!(parse("numbers.map { n => n * 2 }"));
}

#[test]
fn trailing_block_sugar_chained() {
    insta::assert_snapshot!(parse("numbers.reduce(0) { acc, n => acc + n }"));
}

#[test]
fn postfix_call_desugars_to_call_method() {
    insta::assert_snapshot!(parse("f(1, 2)"));
}

