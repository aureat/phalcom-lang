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

use phalcom_ast::{
    ast::{ClassMember, Expr, Statement},
    parse_source,
};

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
    assert!(result.errors.len() >= 3, "expected at least three recovered errors, got {rendered:?}");
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
fn class_inherits_with_is_compiles() {
    let result = parse_source("class Child is Parent {}\n", 0);
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn class_inherits_with_extends_produces_syntax_error() {
    let result = parse_source("class Child extends Parent {}\n", 0);
    assert!(result.is_err(), "class with extends must produce a syntax error");
}

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
    insta::assert_snapshot!(parse("not true"));
}

#[test]
fn return_statement() {
    insta::assert_snapshot!(parse("return 1"));
}

#[test]
fn interpolated_string_parses() {
    let result = parse_source("return \"\\(value)\"", 0);
    assert!(result.is_ok(), "{result:?}");
}

#[test]
fn symbol_name_literal_parses() {
    // selectors.md §2: `#move` parses to `Expr::Symbol(SymbolLiteralKind::Name)`.
    insta::assert_snapshot!(parse("let s = #move"));
}

#[test]
fn symbol_selector_literal_parses() {
    // `#move(_,to,duration)` parses to `Expr::Symbol(SymbolLiteralKind::Selector)`
    // carrying the ordered labels.
    insta::assert_snapshot!(parse("let s = #move(_,to,duration)"));
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

#[test]
fn class_keyword_send_in_method_body_targets_self_class() {
    let program = parse_source("class Counter {\n  bump() {\n    class.bump()\n  }\n}\n", 0).expect("method body containing `class.bump()` should parse");

    let Statement::Class(class) = &program.statements[0] else {
        panic!("expected class declaration");
    };
    let ClassMember::Method(method) = &class.members[0] else {
        panic!("expected method declaration");
    };
    let Statement::Expr {
        expr: Expr::MethodCall(send), ..
    } = &method.body[0]
    else {
        panic!("expected `class.bump()` send");
    };

    assert_eq!(send.method, "bump");
    let Expr::GetProperty(class_property) = &send.object else {
        panic!("expected send receiver to be `self.class`");
    };
    assert_eq!(class_property.property, "class");
    assert!(matches!(class_property.object, Expr::SelfVar { .. }));
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
fn postfix_call_preserves_unqualified_call() {
    insta::assert_snapshot!(parse("f(1, 2)"));
}

// --- Iteration surface (U-ITER, ADR-0035 §2/§3) ---

#[test]
fn for_loop_over_list() {
    insta::assert_snapshot!(parse("for (x in xs) { System.print(x) }"));
}

#[test]
fn for_loop_binding_and_iter_expression() {
    insta::assert_snapshot!(parse("for (item in makeList(3)) { total = total + item }"));
}

#[test]
fn break_statement_parses() {
    insta::assert_snapshot!(parse("for (x in xs) { break }"));
}

#[test]
fn continue_statement_parses() {
    insta::assert_snapshot!(parse("for (x in xs) { continue }"));
}

#[test]
fn for_missing_in_is_error() {
    insta::assert_snapshot!(parse("for (x xs) { }"));
}

// --- `@` attributes (U-ANNOT-CONTRACTS, annotations-legality-grammar.md) ---

#[test]
fn requires_attribute_attaches_to_following_method() {
    // `@requires(...)` binds to the method immediately following it —
    // `MethodDef::attributes` carries the parsed `Attribute`, not yet
    // expanded (expansion is `phalcom-core`'s job, not the parser's).
    insta::assert_snapshot!(parse("class Point {\n  @requires(x > 0)\n  set(x) {\n    self.x = x\n  }\n}\n"));
}

#[test]
fn ensures_attribute_on_getter() {
    insta::assert_snapshot!(parse("class Point {\n  @ensures(__result >= 0)\n  magnitude() {\n    return 1\n  }\n}\n"));
}

#[test]
fn bare_attribute_with_no_args_parses() {
    insta::assert_snapshot!(parse("class Point {\n  @pure\n  x() {\n    return 1\n  }\n}\n"));
}

#[test]
fn multiple_attributes_attach_to_same_member_in_order() {
    insta::assert_snapshot!(parse(
        "class Point {\n  @requires(x > 0)\n  @requires(x < 100)\n  set(x) {\n    self.x = x\n  }\n}\n"
    ));
}

#[test]
fn standalone_invariant_diverts_to_class_invariants() {
    // DEC-ANNOT-B: `@invariant` alone has no following member — it lands in
    // `ClassDef::invariants`, not in any member's `attributes`.
    insta::assert_snapshot!(parse("class Point {\n  @invariant(x >= 0)\n\n  get() {\n    return x\n  }\n}\n"));
}

#[test]
fn dangling_attribute_at_end_of_class_is_error() {
    // `@requires(...)` with nothing following it before `}` is `attr.dangling`.
    insta::assert_snapshot!(parse("class Point {\n  @requires(x > 0)\n}\n"));
}

// --- Declaration Grammar & Parameter Parser Verification ---

#[test]
fn parse_param_underscore_label_and_shorthand() {
    // 1. Position-only underscore value parameter
    let res = parse_source("class Foo {\n  bar(_ val) {}\n}\n", 0);
    assert!(res.is_ok(), "failed to parse `_ val`: {:?}", res.err());

    // 2. Labeled parameter `label value`
    let res = parse_source("class Foo {\n  move(to dest, duration d) {}\n}\n", 0);
    assert!(res.is_ok(), "failed to parse `to dest`: {:?}", res.err());

    // 3. Labeled shorthand `label` (external name == local name)
    let res = parse_source("class Foo {\n  move(to, duration) {}\n}\n", 0);
    assert!(res.is_ok(), "failed to parse label shorthand: {:?}", res.err());
}

#[test]
fn parse_setter_name_put_value() {
    let res = parse_source("class Foo {\n  width=(put val) {\n    self.width = val\n  }\n}\n", 0);
    assert!(res.is_ok(), "failed to parse `width=(put val)`: {:?}", res.err());
}

#[test]
fn parse_subscript_getter_and_setter() {
    // Getter: [_ index]
    let res = parse_source("class Foo {\n  [_ index] {\n    return self.items[index]\n  }\n}\n", 0);
    assert!(res.is_ok(), "failed to parse subscript getter `[_ index]`: {:?}", res.err());

    // Setter: [_ index]=(put value)
    let res = parse_source("class Foo {\n  [_ index]=(put val) {\n    self.items[index] = val\n  }\n}\n", 0);
    assert!(res.is_ok(), "failed to parse subscript setter `[_ index]=(put val)`: {:?}", res.err());
}

#[test]
fn parse_rejects_legacy_label_colon_syntax() {
    let err = parse_display("class Foo {\n  move(to: dest) {}\n}\n");
    assert!(
        err.contains("parameter declaration labels no longer use `:`"),
        "unexpected error message: {err}"
    );
}

#[test]
fn parse_rejects_positional_after_labeled_parameter() {
    let err = parse_display("class Foo {\n  move(to dest, _ other) {}\n}\n");
    assert!(
        err.contains("positional parameters must precede labeled parameters"),
        "unexpected error message: {err}"
    );
}

#[test]
fn parse_reserved_word_as_external_label() {
    let res = parse_source("class Map {\n  insert(_ value, for key) {}\n}\n", 0);
    assert!(res.is_ok(), "failed to parse reserved external label `for key`: {:?}", res.err());
}

#[test]
fn parse_rejects_field_spelling_as_method_name() {
    let err = parse_display("class Foo {\n  _helper(_ value) { value }\n}\n");
    assert!(err.contains("method name"), "unexpected error message: {err}");
}
