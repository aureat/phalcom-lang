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
    ast::{ClassMember, Expr, RecordLiteralEntry, SelectorSpecSyntax, Statement, SymbolLiteralKind},
    parse_source,
};

fn source_slice(source: &str, range: phalcom_common::range::SourceRange) -> &str {
    &source[range.start..range.end]
}

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
fn parser_retains_exact_written_member_target_ranges() {
    let source = "receiver.toString()\n";
    let program = parse_source(source, 0).expect("member call parses");
    let Statement::Expr {
        expr: Expr::MethodCall(call), ..
    } = &program.statements[0]
    else {
        panic!("expected method call");
    };
    assert_eq!(source_slice(source, call.method_range.expect("written method target")), "toString");

    let source = "receiver.rate\n";
    let program = parse_source(source, 0).expect("getter parses");
    let Statement::Expr {
        expr: Expr::GetProperty(get), ..
    } = &program.statements[0]
    else {
        panic!("expected getter access");
    };
    assert_eq!(source_slice(source, get.property_range.expect("written getter target")), "rate");

    let source = "receiver.rate = next\n";
    let program = parse_source(source, 0).expect("setter parses");
    let Statement::Expr {
        expr: Expr::SetProperty(set), ..
    } = &program.statements[0]
    else {
        panic!("expected setter access");
    };
    assert_eq!(source_slice(source, set.property_range.expect("written setter target")), "rate");
}

#[test]
fn parser_retains_exact_written_operator_and_subscript_ranges() {
    let source = "left + right\n";
    let program = parse_source(source, 0).expect("binary expression parses");
    let Statement::Expr {
        expr: Expr::Binary(binary), ..
    } = &program.statements[0]
    else {
        panic!("expected binary expression");
    };
    assert_eq!(source_slice(source, binary.op_range.expect("written operator")), "+");

    let source = "not value\n";
    let program = parse_source(source, 0).expect("unary expression parses");
    let Statement::Expr { expr: Expr::Unary(unary), .. } = &program.statements[0] else {
        panic!("expected unary expression");
    };
    assert_eq!(source_slice(source, unary.op_range.expect("written unary operator")), "not");

    let source = "items[index]\n";
    let program = parse_source(source, 0).expect("subscript read parses");
    let Statement::Expr { expr: Expr::Index(index), .. } = &program.statements[0] else {
        panic!("expected subscript read");
    };
    assert_eq!(source_slice(source, index.selector_range.expect("written subscript")), "[index]");

    let source = "items[index] = next\n";
    let program = parse_source(source, 0).expect("subscript write parses");
    let Statement::Expr {
        expr: Expr::SetIndex(index), ..
    } = &program.statements[0]
    else {
        panic!("expected subscript write");
    };
    assert_eq!(source_slice(source, index.selector_range.expect("written subscript")), "[index]");
}

#[test]
fn parser_retains_exact_written_binding_and_parameter_ranges() {
    let source = "for (item in items) { item }\n";
    let program = parse_source(source, 0).expect("for statement parses");
    let Statement::For(for_statement) = &program.statements[0] else {
        panic!("expected for statement");
    };
    assert_eq!(source_slice(source, for_statement.binding_range), "item");

    let source = "let mapper = |value| value\n";
    let program = parse_source(source, 0).expect("closure parses");
    let Statement::Let(binding) = &program.statements[0] else {
        panic!("expected let binding");
    };
    let Expr::Block(block) = binding.value.as_ref().expect("closure value") else {
        panic!("expected closure block");
    };
    assert_eq!(source_slice(source, block.params.fixed[0].range), "value");

    let source = "class Sample {\n  method(to value) { value }\n}\n";
    let program = parse_source(source, 0).expect("class parses");
    let Statement::Class(class) = &program.statements[0] else {
        panic!("expected class");
    };
    let ClassMember::Method(method) = &class.members[0] else {
        panic!("expected method");
    };
    assert_eq!(source_slice(source, method.params[0].name_range), "value");
    assert_eq!(source_slice(source, method.params[0].label_range.expect("external label")), "to");
}

#[test]
fn parser_retains_exact_written_declaration_and_method_reference_ranges() {
    let source = "class Sample {\n  const _field\n}\n";
    let program = parse_source(source, 0).expect("field class parses");
    let Statement::Class(class) = &program.statements[0] else {
        panic!("expected class");
    };
    let ClassMember::Field(field) = &class.members[0] else {
        panic!("expected field");
    };
    assert_eq!(source_slice(source, field.name_range), "_field");

    let source = "receiver::method\n";
    let program = parse_source(source, 0).expect("method reference parses");
    let Statement::Expr {
        expr: Expr::MethodRef(reference),
        ..
    } = &program.statements[0]
    else {
        panic!("expected method reference");
    };
    assert_eq!(source_slice(source, reference.selector_range.expect("written method reference")), "method");

    let source = "@sealed\nclass Shape {\n  @variant Circle(radius:)\n}\n";
    let program = parse_source(source, 0).expect("variant class parses");
    let Statement::Class(class) = &program.statements[0] else {
        panic!("expected class");
    };
    let ClassMember::Variant(variant) = &class.members[0] else {
        panic!("expected variant");
    };
    assert_eq!(source_slice(source, variant.name_range), "Circle");
}

#[test]
fn selector_spec_owns_method_parens_and_preserves_pattern_components() {
    let source = "receiver::method()\n";
    let program = parse_source(source, 0).expect("exact selector method parses");
    let Statement::Expr {
        expr: Expr::MethodRef(reference),
        ..
    } = &program.statements[0]
    else {
        panic!("expected MethodRef");
    };
    assert!(matches!(reference.spec, SelectorSpecSyntax::Exact(ref exact) if exact.base == "method" && exact.slots.is_empty()));
    assert_eq!(source_slice(source, reference.selector_range.expect("selector range")), "method()");

    let source = "receiver::method(_, ..., tail)\n";
    let program = parse_source(source, 0).expect("pattern selector parses");
    let Statement::Expr {
        expr: Expr::MethodRef(reference),
        ..
    } = &program.statements[0]
    else {
        panic!("expected MethodRef");
    };
    let SelectorSpecSyntax::Pattern(pattern) = &reference.spec else {
        panic!("expected patterned selector");
    };
    assert_eq!(pattern.prefix.len(), 1);
    assert_eq!(pattern.suffix.len(), 1);
    assert_eq!(source_slice(source, pattern.gap_range), "...");
    assert_eq!(source_slice(source, pattern.suffix[0].range), "tail");
}

#[test]
fn first_class_selector_pattern_is_distinct_from_exact_symbol() {
    let source = "let pattern = #name(...)\n";
    let program = parse_source(source, 0).expect("selector pattern parses");
    let Statement::Let(binding) = &program.statements[0] else {
        panic!("expected binding");
    };
    let Some(Expr::Symbol(symbol)) = &binding.value else {
        panic!("expected symbol expression");
    };
    assert!(matches!(symbol.kind, SymbolLiteralKind::Pattern(_)));
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
fn record_literals_accept_trailing_commas() {
    let cases = [
        ("#{ a: 1, }", false),
        ("#{ **source, }", true),
        ("#{\n  a: 1,\n}", false),
        ("#{\n  **source,\n}", true),
    ];

    for (source, is_expansion) in cases {
        let program = parse_source(source, 0).unwrap_or_else(|err| panic!("{source:?} should parse: {err:?}"));
        let Statement::Expr {
            expr: Expr::RecordLiteral(record),
            ..
        } = &program.statements[0]
        else {
            panic!("{source:?} should produce a Record literal expression");
        };
        assert_eq!(record.entries.len(), 1, "{source:?} should keep one entry");
        assert_eq!(
            matches!(&record.entries[0], RecordLiteralEntry::Expansion { .. }),
            is_expansion,
            "{source:?} parsed wrong entry kind",
        );
    }
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

// --- Retired closure syntax diagnostics ---

#[test]
fn bare_brace_closure_is_rejected() {
    assert!(parse_display("{ System.print(\"hi\") }").starts_with("bare brace block literals were removed; write `|| { ... }` for a closure"));
}

#[test]
fn braced_parameter_closure_is_rejected() {
    assert!(parse_source("{ x => x + 1 }", 0).is_err());
}

#[test]
fn arrow_closure_is_rejected() {
    assert!(parse_source("x => x + 1", 0).is_err());
}

#[test]
fn trailing_brace_closure_is_rejected() {
    assert!(parse_source("numbers.map { n => n * 2 }", 0).is_err());
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
fn parse_reserved_word_as_external_label_in_definition_and_call() {
    let res = parse_source("class Map {\n  insert(_ value, for key) {}\n}\n", 0);
    assert!(res.is_ok(), "failed to parse reserved external label `for key`: {:?}", res.err());

    let res = parse_source("Map.new().insert(1, for: 2)", 0);
    assert!(res.is_ok(), "failed to parse reserved call label `for:`: {:?}", res.err());
}

#[test]
fn parse_rejects_field_spelling_as_method_name() {
    let err = parse_display("class Foo {\n  _helper(_ value) { value }\n}\n");
    assert!(err.contains("method name"), "unexpected error message: {err}");
}
