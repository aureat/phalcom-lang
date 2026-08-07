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
    Lexer::new(src).map(|spanned| spanned.expect("fixture should lex without error").1).collect()
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
    insta::assert_debug_snapshot!(tokens("class Point {\n  static origin { self }\n}"));
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
    insta::assert_debug_snapshot!(tokens("let x = 1 /* block\n comment */ let y = 2"));
}

#[test]
fn string_interpolation_splits_into_segments() {
    // D4: `\(expr)` interpolation lexes to a single `Token::StringInterp` with
    // ordered literal/expression segments (ADR-0022).
    insta::assert_debug_snapshot!(tokens("\"hi \\(name), you are \\(age)\""));
}

#[test]
fn string_without_interpolation_stays_plain() {
    // D4 guard: a string with no `\(` still lexes to `Token::String`, and a
    // `\\(` escape is a literal `\(` (not an interpolation).
    insta::assert_debug_snapshot!(tokens("\"plain \\\\(not interp)\""));
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

#[test]
fn symbol_name_literal() {
    // selectors.md §2: a bare `#name` is a `Token::NameSymbol`, not tied to
    // any selector shape.
    insta::assert_debug_snapshot!(tokens("#move #size #_field"));
}

#[test]
fn symbol_selector_literal_with_whitespace() {
    // Whitespace inside the parens is free and does not affect the parsed
    // labels (selectors.md §2); `_` is a positional slot, a bare identifier
    // is a label.
    insta::assert_debug_snapshot!(tokens("#move(_,to,duration) #move(\n  _,\n  to,\n  duration\n) #size()"));
}

#[test]
fn symbol_operator_selectors() {
    // Bare operator symbols (`#+`, `#==`, ...) always lex as a one-argument
    // `Token::SelectorSymbol` — every operator method definition takes
    // exactly one parameter.
    insta::assert_debug_snapshot!(tokens("#+ #- #* #/ #% #== #!= #< #<= #> #>="));
}

#[test]
fn symbol_adjacency_required_for_selector_form() {
    // selectors.md §2 ASI-hazard guard: `(` must be immediately adjacent to
    // the name to form a selector symbol. `#move (a, b)` lexes as the name
    // symbol `#move` followed by a separate `(` token, not one greedy
    // selector symbol.
    insta::assert_debug_snapshot!(tokens("#move (a, b)"));
}

#[test]
fn symbol_bare_hash_is_not_a_symbol() {
    // `# move` (whitespace between `#` and the name) is not a symbol at all —
    // the lone `#` fails to lex.
    let items: Vec<_> = Lexer::new("# move").collect();
    assert!(matches!(items[0], Err(phalcom_ast::token::LexicalError::InvalidToken(ref span)) if span.start == 0 && span.end == 1));
}

#[test]
fn shebang_at_offset_zero_is_skipped() {
    // selectors.md §2 reserved-sigil carve-out: a `#!` shebang line at byte
    // offset 0 is skipped like a comment, up to (not including) its newline.
    insta::assert_debug_snapshot!(tokens("#!/usr/bin/env phalcom\nlet x = 1"));
}

#[test]
fn hash_not_at_offset_zero_is_a_symbol_not_a_shebang() {
    // The shebang carve-out is offset-0-only: a `#!` appearing later in the
    // source is not special-cased (`!` is not a valid symbol-name start, so
    // this still fails to lex as a symbol — proving no shebang skip fired).
    let items: Vec<_> = Lexer::new("let x = 1\n#!oops").collect();
    assert!(items.iter().any(std::result::Result::is_err));
}

#[test]
fn pdr0026_radix_and_float_literals() {
    assert_eq!(
        tokens("0b1010"),
        vec![
            Token::Int {
                digits: "1010".into(),
                radix: 2
            },
            Token::Eof
        ]
    );
    assert_eq!(
        tokens("0o755"),
        vec![
            Token::Int {
                digits: "755".into(),
                radix: 8
            },
            Token::Eof
        ]
    );
    assert_eq!(
        tokens("0xFF"),
        vec![
            Token::Int {
                digits: "ff".into(),
                radix: 16
            },
            Token::Eof
        ]
    );
    assert_eq!(
        tokens("1_000"),
        vec![
            Token::Int {
                digits: "1000".into(),
                radix: 10
            },
            Token::Eof
        ]
    );
    assert_eq!(tokens(".5"), vec![Token::Float(0.5), Token::Eof]);
    assert_eq!(tokens("6e2"), vec![Token::Float(600.0), Token::Eof]);
}

#[test]
fn double_asterisk_and_int_div_tokens() {
    assert_eq!(
        tokens("2 ** 3 ~/ 4"),
        vec![
            Token::Int { digits: "2".into(), radix: 10 },
            Token::DoubleAsterisk,
            Token::Int { digits: "3".into(), radix: 10 },
            Token::SlashTilde,
            Token::Int { digits: "4".into(), radix: 10 },
            Token::Eof
        ]
    );
}

#[test]
fn pdr0026_malformed_literals_atomic_error() {
    let inputs = vec!["0x_G", "1__0", "0123", "5.", "1e"];
    for inp in inputs {
        let items: Vec<_> = Lexer::new(inp).collect();
        assert!(
            matches!(items[0], Err(phalcom_ast::token::LexicalError::NumericLiteral(_))),
            "Expected NumericLiteral error for input: {inp}"
        );
    }
}
