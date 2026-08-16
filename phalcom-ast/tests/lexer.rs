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
fn multiline_string_basic_dedent() {
    let src = "\"\"\"\n    first\n        second\n    third\n    \"\"\"";
    let toks = tokens(src);
    assert_eq!(toks, vec![Token::String("first\n    second\nthird".into()), Token::Eof]);
}

#[test]
fn multiline_string_empty_block() {
    let src = "\"\"\"\n\"\"\"";
    let toks = tokens(src);
    assert_eq!(toks, vec![Token::String("".into()), Token::Eof]);
}

#[test]
fn multiline_string_closing_margin_zero() {
    let src = "\"\"\"\nfirst\n  second\n\"\"\"";
    let toks = tokens(src);
    assert_eq!(toks, vec![Token::String("first\n  second".into()), Token::Eof]);
}

#[test]
fn multiline_string_opening_trailing_hspace() {
    let src = "\"\"\"   \t  \n    hello\n    \"\"\"";
    let toks = tokens(src);
    assert_eq!(toks, vec![Token::String("hello".into()), Token::Eof]);
}

#[test]
fn multiline_string_blank_lines() {
    let src = "\"\"\"\n    first\n\n  \n        \n    second\n    \"\"\"";
    let toks = tokens(src);
    assert_eq!(toks, vec![Token::String("first\n\n\n\nsecond".into()), Token::Eof]);
}

#[test]
fn multiline_string_exact_prefix_tabs() {
    let src = "\"\"\"\n\t\tfirst\n\t\t\tsecond\n\t\t\"\"\"";
    let toks = tokens(src);
    assert_eq!(toks, vec![Token::String("first\n\tsecond".into()), Token::Eof]);
}

#[test]
fn multiline_string_newlines_crlf() {
    let src = "\"\"\"\r\n    first\r\n    second\r\n    \"\"\"";
    let toks = tokens(src);
    assert_eq!(toks, vec![Token::String("first\nsecond".into()), Token::Eof]);
}

#[test]
fn multiline_string_escapes() {
    let src = "\"\"\"\n    \\\" \\\\ \\n \\t \\r\n    \"\"\"";
    let toks = tokens(src);
    assert_eq!(toks, vec![Token::String("\" \\ \n \t \r".into()), Token::Eof]);
}

#[test]
fn multiline_string_embedded_triple_quotes() {
    let src = "\"\"\"\n    She wrote \"\"\"hello\"\"\" here\n    \"\"\"";
    let toks = tokens(src);
    assert_eq!(toks, vec![Token::String("She wrote \"\"\"hello\"\"\" here".into()), Token::Eof]);
}

#[test]
fn multiline_string_interpolation_and_source_ranges() {
    let src = "let s = \"\"\"\n    α \\(x + 1) beta\n    \"\"\"";
    let mut lex = Lexer::new(src);
    let mut items = Vec::new();
    while let Some(res) = lex.next() {
        items.push(res.expect("should lex"));
    }
    // Check tokens
    assert_eq!(items[0].1, Token::Let);
    assert_eq!(items[1].1, Token::Identifier("s".into()));
    assert_eq!(items[2].1, Token::Equal);
    match &items[3].1 {
        Token::StringInterp(segments) => {
            assert_eq!(segments.len(), 3);
            assert_eq!(segments[0], phalcom_ast::token::StringSegment::Literal("α ".into()));
            match &segments[1] {
                phalcom_ast::token::StringSegment::Expr { source, range } => {
                    assert_eq!(source, "x + 1");
                    assert_eq!(&src[range.clone()], "x + 1");
                }
                _ => panic!("expected Expr segment"),
            }
            assert_eq!(segments[2], phalcom_ast::token::StringSegment::Literal(" beta".into()));
        }
        other => panic!("expected StringInterp, got {other:?}"),
    }
}

#[test]
fn multiline_string_nested_multiline_in_interpolation() {
    let src = "\"\"\"\n    outer \\(call(\"\"\"\n        nested\n        \"\"\"\n    ))\n    end\n    \"\"\"";
    let toks = tokens(src);
    assert!(matches!(toks[0], Token::StringInterp(_)));
}

#[test]
fn multiline_string_diagnostics() {
    // Bad opening
    let err = Lexer::new("\"\"\"invalid\n\"\"\"").next().unwrap().unwrap_err();
    assert!(matches!(err, phalcom_ast::token::LexicalError::InvalidMultilineStringOpening(_)));

    // Bad indent
    let err = Lexer::new("\"\"\"\n    first\n  second\n    \"\"\"").next().unwrap().unwrap_err();
    assert!(matches!(err, phalcom_ast::token::LexicalError::InvalidMultilineStringIndentation(_)));

    // Unterminated
    let err = Lexer::new("\"\"\"\n    first").next().unwrap().unwrap_err();
    assert!(matches!(err, phalcom_ast::token::LexicalError::UnterminatedMultilineString(_)));

    // Raw CR
    let err = Lexer::new("\"\"\"\n    first\r    second\n    \"\"\"").next().unwrap().unwrap_err();
    assert!(matches!(err, phalcom_ast::token::LexicalError::InvalidMultilineStringLineEnding(_)));
}

#[test]
fn quoted_symbol_does_not_become_multiline() {
    let err = Lexer::new("#\"\"\"\nhello\n\"\"\"").nth(1).unwrap().unwrap_err();
    assert!(matches!(err, phalcom_ast::token::LexicalError::RawNewlineInString(_)));
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
    // The lexer emits the hash prefix and leaves name parsing to the parser.
    insta::assert_debug_snapshot!(tokens("#move #size #_field"));
}

#[test]
fn symbol_selector_literal_with_whitespace() {
    // Selector components retain ordinary token boundaries. Newline
    // suppression after `(` still applies, while parser adjacency remains
    // observable from the component spans.
    insta::assert_debug_snapshot!(tokens("#move(_,to,duration) #move(\n  _,\n  to,\n  duration\n) #size()"));
}

#[test]
fn symbol_operator_selectors() {
    // Operator spellings are ordinary operator tokens after `Token::Hash`.
    insta::assert_debug_snapshot!(tokens("#+ #- #* #/ #% #== #!= #< #<= #> #>="));
}

#[test]
fn symbol_adjacency_required_for_selector_form() {
    // Whitespace adjacency is now visible to the parser because the lexer
    // does not consume the selector body.
    insta::assert_debug_snapshot!(tokens("#move (a, b)"));
}

#[test]
fn symbol_bare_hash_is_standalone_token() {
    // A hash is valid on its own; selector-shape validation belongs to the
    // parser.
    assert_eq!(tokens("# move"), vec![Token::Hash, Token::Identifier("move".into()), Token::Eof]);
}

#[test]
fn selector_spec_components() {
    insta::assert_debug_snapshot!(tokens(
        "#name #name() #name(_) #name(foo) #name... #name(...) #name(_, ..., foo) #name=... #+ #== #{ key: value } #\"quoted\""
    ));
}

#[test]
fn shebang_at_offset_zero_is_skipped() {
    // selectors.md §2 reserved-sigil carve-out: a `#!` shebang line at byte
    // offset 0 is skipped like a comment, up to (not including) its newline.
    insta::assert_debug_snapshot!(tokens("#!/usr/bin/env phalcom\nlet x = 1"));
}

#[test]
fn hash_not_at_offset_zero_is_not_a_shebang() {
    // The shebang carve-out is offset-0-only. Later `#!` is tokenized by the
    // ordinary lexer as `Hash`, `Bang`, and an identifier.
    let items = tokens("let x = 1\n#!oops");
    assert!(items.contains(&Token::Hash));
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

#[test]
fn module_keywords_and_atbang_tokens() {
    assert_eq!(
        tokens("from export expose @!"),
        vec![Token::From, Token::Export, Token::Expose, Token::AtBang, Token::Eof,]
    );
}
