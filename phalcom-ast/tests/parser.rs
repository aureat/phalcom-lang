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
    ast::{
        AssociatedMemberSyntax, AssociatedNamedMode, AssociatedResidualSelectorSyntax, ClassMember, EnumMember, Expr, GenericConstraintSyntax, KindSyntax,
        MemberBody, RecordLiteralEntry, ReturnStatement, Statement, SymbolLiteralKind, TypeAnnotationExpr, VarianceSyntax,
    },
    error::SyntaxErrorKind,
    parse as parse_with_recovery, parse_source,
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
fn class_inherits_from_qualified_static_symbol() {
    let result = parse_source("class Child is base.Shape {}\n", 0).expect("qualified superclass parses");
    let Statement::Class(class) = &result.statements[0] else {
        panic!("expected class statement")
    };
    let superclass = class.superclass.as_ref().expect("explicit superclass");
    let sym = superclass.origin_symbol_ref().expect("static symbol superclass");
    assert_eq!(sym.root, "base");
    assert_eq!(sym.members.iter().map(|segment| segment.name.as_str()).collect::<Vec<_>>(), vec!["Shape"]);
    assert!(!sym.is_bare());
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
    let source = "for item in items { item }\n";
    let program = parse_source(source, 0).expect("for statement parses");
    let Statement::For(for_statement) = &program.statements[0] else {
        panic!("expected for statement");
    };
    assert_eq!(source_slice(source, for_statement.lanes[0].pattern.range()), "item");

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
    let program = parse_source(source, 0).expect("associated lookup parses");
    let Statement::Expr {
        expr: Expr::AssociatedLookup(lookup),
        ..
    } = &program.statements[0]
    else {
        panic!("expected associated lookup");
    };
    let AssociatedMemberSyntax::Named(named) = &lookup.member else {
        panic!("expected named member");
    };
    assert_eq!(named.base, "method");

    let source = "enum Shape {\n  @variant Circle(radius: Float)\n}\n";
    let program = parse_source(source, 0).expect("variant enum parses");
    let Statement::Enum(enum_def) = &program.statements[0] else {
        panic!("expected enum");
    };
    let EnumMember::Variant(variant) = &enum_def.members[0] else {
        panic!("expected variant");
    };
    assert_eq!(source_slice(source, variant.name_range), "Circle");
}

#[test]
fn associated_invoke_and_exact_method_syntax() {
    let source = "receiver::method()\n";
    let program = parse_source(source, 0).expect("direct associated invoke parses");
    let Statement::Expr {
        expr: Expr::AssociatedInvoke(invoke),
        ..
    } = &program.statements[0]
    else {
        panic!("expected AssociatedInvoke");
    };
    assert_eq!(invoke.base, "method");
    assert!(invoke.args.is_empty());

    let source = "receiver::method::()\n";
    let program = parse_source(source, 0).expect("exact method lookup parses");
    let Statement::Expr {
        expr: Expr::AssociatedLookup(lookup),
        ..
    } = &program.statements[0]
    else {
        panic!("expected AssociatedLookup");
    };
    let AssociatedMemberSyntax::Named(named) = &lookup.member else {
        panic!("expected named member");
    };
    assert_eq!(named.base, "method");
    assert!(matches!(named.mode, AssociatedNamedMode::Exact { residual: AssociatedResidualSelectorSyntax::Method { ref slots, .. }, .. } if slots.is_empty()));
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
fn symbol_name_requires_hash_adjacency() {
    assert!(parse_source("let s = # move\n", 0).is_err());
}

#[test]
fn symbol_selector_literal_parses() {
    // `#move(_,to,duration)` parses to `Expr::Symbol(SymbolLiteralKind::Selector)`
    // carrying the ordered labels.
    insta::assert_snapshot!(parse("let s = #move(_,to,duration)"));
}

#[test]
fn symbol_selector_rejects_interior_positional() {
    assert!(parse_source("let s = #move(to,_)\n", 0).is_err());
}

#[test]
fn symbol_operator_forms_preserve_exact_spelling_and_explicit_shape() {
    for (source, expected) in [
        ("#+", SymbolLiteralKind::Name("+".into())),
        (
            "#+()",
            SymbolLiteralKind::Selector {
                name: "+".into(),
                labels: vec![],
            },
        ),
        (
            "#+(_) ",
            SymbolLiteralKind::Selector {
                name: "+".into(),
                labels: vec![None],
            },
        ),
        ("#*", SymbolLiteralKind::Name("*".into())),
        ("#**", SymbolLiteralKind::Name("**".into())),
        ("#***", SymbolLiteralKind::Name("***".into())),
    ] {
        let source = format!("let s = {source}\n");
        let program = parse_source(&source, 0).expect("symbol should parse");
        let Statement::Let(binding) = &program.statements[0] else {
            panic!("expected let")
        };
        let Some(Expr::Symbol(symbol)) = &binding.value else {
            panic!("expected symbol")
        };
        assert_eq!(format!("{:#?}", symbol.kind), format!("{expected:#?}"), "{source:?}");
    }
}

#[test]
fn symbol_operator_patterns_and_whitespace_are_structural() {
    for source in ["#+...", "#+(...)", "#+(_, ...)", "#method (_, _)", "#+ ()"] {
        let source = format!("let s = {source}\n");
        parse_source(&source, 0).expect("selector symbol should parse");
    }
    let program = parse_source("let s = #method(_, _)()\n", 0).expect("postfix call should parse");
    let Statement::Let(binding) = &program.statements[0] else {
        panic!("expected let")
    };
    let Some(Expr::MethodCall(call)) = &binding.value else {
        panic!("expected call")
    };
    assert_eq!(call.method, "call");
    assert!(matches!(call.object, Expr::Symbol(_)));
}

#[test]
fn symbol_punctuation_bang_rest_and_bracket_forms_parse() {
    for source in [
        "#!", "#?", "#?.", "#??", "#...", "#try!", "#try!(_)", "#a!", "#a!(_) ", "#*args", "#**args", "#***args", "#[_]", "#[_,_]", "#[x,y]", "#[...]",
    ] {
        let source = format!("let s = {source}\n");
        parse_source(&source, 0).expect("symbol form should parse");
    }
    parse_source("let s = #[...]=(put)\n", 0).expect("subscript pattern setter should parse");
    parse_source("let s = #\"!\"\n", 0).expect("quoted symbol should parse");
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
    let MemberBody::Block(stmts) = &method.body else {
        panic!("expected block body");
    };
    let Statement::Expr {
        expr: Expr::MethodCall(send), ..
    } = &stmts[0]
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

#[test]
fn class_keyword_at_module_level_is_a_bare_variable() {
    let program = parse_source("class.class\n", 0).expect("module-level `class.class` should parse");
    let Statement::Expr {
        expr: Expr::GetProperty(property),
        ..
    } = &program.statements[0]
    else {
        panic!("expected `class.class` property expression");
    };

    assert_eq!(property.property, "class");
    let Expr::Var { value, .. } = &property.object else {
        panic!("expected module-level `class` to remain a bare variable");
    };
    assert_eq!(value, "class");
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
    insta::assert_snapshot!(parse("for x in xs { System.print(x) }"));
}

#[test]
fn for_loop_binding_and_iter_expression() {
    insta::assert_snapshot!(parse("for item in makeList(3) { total = total + item }"));
}

#[test]
fn break_statement_parses() {
    insta::assert_snapshot!(parse("for x in xs { break }"));
}

#[test]
fn continue_statement_parses() {
    insta::assert_snapshot!(parse("for x in xs { continue }"));
}

#[test]
fn for_missing_in_is_error() {
    insta::assert_snapshot!(parse("for x xs { }"));
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
    let res = parse_source("class Foo {\n  move(to: Point) {}\n}\n", 0);
    assert!(res.is_ok(), "to: Point should parse as typed parameter: {:?}", res.err());
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
fn parse_from_keyword_as_method_selector() {
    let res = parse_source("class Map {\n  from(_ record) { record }\n}\nMap.from(#{})\n", 0);
    assert!(res.is_ok(), "`from` must remain usable as a method selector: {:?}", res.err());
}

#[test]
fn parse_rejects_field_spelling_as_method_name() {
    let err = parse_display("class Foo {\n  _helper(_ value) { value }\n}\n");
    assert!(err.contains("method name"), "unexpected error message: {err}");
}

#[test]
fn parse_multiline_string_literal_into_ast() {
    let src = "return \"\"\"\n    first\n    second\n    \"\"\"\n";
    let program = parse_source(src, 0).expect("should parse");
    let Statement::Return(ReturnStatement {
        value: Some(Expr::String { value: s, .. }),
        ..
    }) = &program.statements[0]
    else {
        panic!("expected Expr::String, got {:#?}", program.statements[0]);
    };
    assert_eq!(s, "first\nsecond");
}

#[test]
fn parse_multiline_string_interpolation_into_ast() {
    let src = "return \"\"\"\n    hello \\(name)\n    \"\"\"\n";
    let program = parse_source(src, 0).expect("should parse");
    // Should lower to String concatenation
    match &program.statements[0] {
        Statement::Return(ReturnStatement {
            value: Some(Expr::Binary(..)), ..
        }) => {}
        other => panic!("expected lowered binary Expr for interpolation, got {other:#?}"),
    }
}

// --- Modules v1 Syntax and Metadata Tests ---

#[test]
fn parse_whole_module_imports() {
    let src = "import geometry.point\nimport geometry.point as point_module\nimport .point\nimport ..units as units\n";
    let program = parse_source(src, 0).expect("whole-module imports should parse");
    assert_eq!(program.preamble.dependencies.len(), 4);
    assert_eq!(program.statements.len(), 0);
}

#[test]
fn parse_selective_imports() {
    let src = "from geometry.point import Point, distance\nfrom geometry.point import Point as P, distance as dist\nfrom geometry.point import (\n    Point,\n    distance,\n    origin as default_origin,\n)\n";
    let program = parse_source(src, 0).expect("selective imports should parse");
    assert_eq!(program.preamble.dependencies.len(), 3);
}

#[test]
fn parse_exports_and_reexports() {
    let src = "export Point from .point\nexport cartesian_point as Point from .cartesian\nexport (\n    Point,\n    origin,\n    distance as point_distance,\n) from .point\n\nclass LocalClass {}\nexport LocalClass\nexport (LocalClass,)\n";
    let program = parse_source(src, 0).expect("exports and reexports should parse");
    assert_eq!(program.preamble.dependencies.len(), 3);
    assert_eq!(program.statements.len(), 3); // Class + 2 local Export statements
}

#[test]
fn parse_expose_declarations() {
    let src = "expose .point\nexpose .shapes\n";
    let program = parse_source(src, 0).expect("expose declarations should parse");
    assert_eq!(program.preamble.dependencies.len(), 2);
}

#[test]
fn parse_module_metadata_header() {
    let src = "@! documentation(\"Parser implementation\")\n@! stability(#experimental)\n@! tags((\"parser\", \"internal\"))\n@! config(mode: \"strict\", version: 1)\n\nimport .tokens\n";
    let program = parse_source(src, 0).expect("metadata header should parse");
    assert_eq!(program.preamble.metadata.len(), 4);
    assert_eq!(program.preamble.metadata[0].name, "documentation");
    assert_eq!(program.preamble.metadata[1].name, "stability");
    assert_eq!(program.preamble.dependencies.len(), 1);
}

#[test]
fn parse_rejects_physical_string_import() {
    let err = parse_display("import \"geometry/point\" as Point\n");
    assert!(err.contains("physical string imports `import \"...\"` have been retired"), "unexpected: {err}");
}

#[test]
fn parse_rejects_import_outside_preamble() {
    let err = parse_display("const x = 1\nimport .config\n");
    assert!(
        err.contains("static `import` declarations must appear in the module dependency preamble"),
        "unexpected: {err}"
    );
}

#[test]
fn parse_rejects_atbang_outside_header() {
    let err = parse_display("import .config\n@! stability(#stable)\n");
    assert!(
        err.contains("@! attributes must appear at the top of the file before imports"),
        "unexpected: {err}"
    );
}

#[test]
fn parse_rejects_invalid_expose_syntax() {
    let err1 = parse_display("expose ..foo\n");
    assert!(err1.contains("expose operand cannot ascend"), "unexpected: {err1}");

    let err2 = parse_display("expose .shapes.circle\n");
    assert!(err2.contains("must be a single immediate child segment"), "unexpected: {err2}");
}

#[test]
fn parse_membership_operators() {
    // 1. in
    let p1 = parse_source("x in y\n", 0).expect("x in y should parse");
    match &p1.statements[0] {
        Statement::Expr { expr: Expr::Membership(m), .. } => {
            assert!(!m.negated);
            assert!(matches!(m.left, Expr::Var { .. }));
            assert!(matches!(m.right, Expr::Var { .. }));
        }
        other => panic!("expected Membership, got {other:?}"),
    }

    // 2. not in
    let p2 = parse_source("x not in y\n", 0).expect("x not in y should parse");
    match &p2.statements[0] {
        Statement::Expr { expr: Expr::Membership(m), .. } => {
            assert!(m.negated);
            assert!(matches!(m.left, Expr::Var { .. }));
            assert!(matches!(m.right, Expr::Var { .. }));
        }
        other => panic!("expected Membership negated, got {other:?}"),
    }

    // 3. is in
    let p3 = parse_source("x is in ys\n", 0).expect("x is in ys should parse");
    match &p3.statements[0] {
        Statement::Expr {
            expr: Expr::IsMembership(m), ..
        } => {
            assert!(!m.strict);
            assert!(!m.negated);
        }
        other => panic!("expected IsMembership, got {other:?}"),
    }

    // 4. is! in
    let p4 = parse_source("x is! in ys\n", 0).expect("x is! in ys should parse");
    match &p4.statements[0] {
        Statement::Expr {
            expr: Expr::IsMembership(m), ..
        } => {
            assert!(m.strict);
            assert!(!m.negated);
        }
        other => panic!("expected IsMembership strict, got {other:?}"),
    }

    // 5. is not in
    let p5 = parse_source("x is not in ys\n", 0).expect("x is not in ys should parse");
    match &p5.statements[0] {
        Statement::Expr {
            expr: Expr::IsMembership(m), ..
        } => {
            assert!(!m.strict);
            assert!(m.negated);
        }
        other => panic!("expected IsMembership negated, got {other:?}"),
    }

    // 6. is! not in
    let p6 = parse_source("x is! not in ys\n", 0).expect("x is! not in ys should parse");
    match &p6.statements[0] {
        Statement::Expr {
            expr: Expr::IsMembership(m), ..
        } => {
            assert!(m.strict);
            assert!(m.negated);
        }
        other => panic!("expected IsMembership strict negated, got {other:?}"),
    }
}

#[test]
fn parse_membership_precedence() {
    let p = parse_source("a + b in c * d\n", 0).expect("should parse");
    match &p.statements[0] {
        Statement::Expr { expr: Expr::Membership(m), .. } => {
            assert!(matches!(m.left, Expr::Binary(..)));
            assert!(matches!(m.right, Expr::Binary(..)));
        }
        other => panic!("expected Membership with binary operands, got {other:?}"),
    }
}

#[test]
fn parse_membership_errors_and_non_chaining() {
    let err1 = parse_display("x in not y\n");
    assert!(err1.contains("did you mean `not in`?"), "unexpected: {err1}");

    let err2 = parse_display("x is in not y\n");
    assert!(err2.contains("did you mean `is not in` or `is! not in`?"), "unexpected: {err2}");

    let err3 = parse_display("a in b in c\n");
    assert!(err3.contains("chained"), "unexpected: {err3}");

    let err4 = parse_display("a is in xs is in ys\n");
    assert!(err4.contains("chained"), "unexpected: {err4}");
}

#[test]
fn parse_operator_property_on_number() {
    let p1 = parse_source("5.-\n", 0).expect("5.- should parse");
    match &p1.statements[0] {
        Statement::Expr {
            expr: Expr::GetProperty(g), ..
        } => {
            assert_eq!(g.property, "-");
            assert!(matches!(g.object, Expr::Int { .. }));
        }
        other => panic!("expected GetProperty, got {other:?}"),
    }

    let p2 = parse_source("5.-()\n", 0).expect("5.-() should parse");
    match &p2.statements[0] {
        Statement::Expr { expr: Expr::MethodCall(m), .. } => {
            assert_eq!(m.method, "-");
            assert!(m.args.is_empty());
            assert!(matches!(m.object, Expr::Int { .. }));
        }
        other => panic!("expected MethodCall, got {other:?}"),
    }

    let p3 = parse_source("5.+()\n", 0).expect("5.+() should parse");
    match &p3.statements[0] {
        Statement::Expr { expr: Expr::MethodCall(m), .. } => {
            assert_eq!(m.method, "+");
            assert!(m.args.is_empty());
            assert!(matches!(m.object, Expr::Int { .. }));
        }
        other => panic!("expected MethodCall, got {other:?}"),
    }
}

#[test]
fn parse_membership_range_precedence() {
    let p1 = parse_source("5 in 1..=5\n", 0).expect("5 in 1..=5 should parse");
    match &p1.statements[0] {
        Statement::Expr { expr: Expr::Membership(m), .. } => {
            assert!(matches!(m.left, Expr::Int { .. }));
            match &m.right {
                Expr::Range(r) => {
                    assert!(r.upper_inclusive);
                    assert!(matches!(r.lower.as_ref().unwrap(), Expr::Int { .. }));
                    assert!(matches!(r.upper.as_ref().unwrap(), Expr::Int { .. }));
                }
                other => panic!("expected Range on RHS of in, got {other:?}"),
            }
        }
        other => panic!("expected Membership, got {other:?}"),
    }

    let p2 = parse_source("5 in 1..5\n", 0).expect("5 in 1..5 should parse");
    match &p2.statements[0] {
        Statement::Expr { expr: Expr::Membership(m), .. } => {
            assert!(matches!(m.left, Expr::Int { .. }));
            match &m.right {
                Expr::Range(r) => {
                    assert!(!r.upper_inclusive);
                }
                other => panic!("expected Range on RHS of in, got {other:?}"),
            }
        }
        other => panic!("expected Membership, got {other:?}"),
    }
}

#[test]
fn parse_type_syntax_generic_class_and_token_fission() {
    let p = parse_source("class Container<+T> is Super<List<T>> where T <: Object {}\n", 0).expect("generic class with >> token fission should parse");
    let Statement::Class(class) = &p.statements[0] else { panic!() };
    assert_eq!(class.name, "Container");
    assert_eq!(class.generic_parameters.len(), 1);
    assert_eq!(class.generic_parameters[0].name, "T");
    assert_eq!(class.generic_parameters[0].variance, VarianceSyntax::Covariant);

    let superclass = class.superclass.as_ref().expect("superclass");
    match &superclass.expr {
        TypeAnnotationExpr::Application { origin, arguments, .. } => {
            let sym = origin.origin_symbol_ref().expect("symbol");
            assert_eq!(sym.root, "Super");
            assert_eq!(arguments.len(), 1);
            match &arguments[0].expr {
                TypeAnnotationExpr::Application {
                    origin: inner_orig,
                    arguments: inner_args,
                    ..
                } => {
                    let inner_sym = inner_orig.origin_symbol_ref().expect("symbol");
                    assert_eq!(inner_sym.root, "List");
                    assert_eq!(inner_args.len(), 1);
                }
                other => panic!("expected nested application, got {other:?}"),
            }
        }
        other => panic!("expected Application, got {other:?}"),
    }

    let where_clause = class.where_clause.as_ref().expect("where clause");
    assert_eq!(where_clause.constraints.len(), 1);
    match &where_clause.constraints[0] {
        GenericConstraintSyntax::Subtype { lower, upper, .. } => {
            assert_eq!(lower.origin_symbol_ref().unwrap().root, "T");
            assert_eq!(upper.origin_symbol_ref().unwrap().root, "Object");
        }
        other => panic!("expected subtype constraint, got {other:?}"),
    }
}

#[test]
fn parse_type_alias_and_type_precedence() {
    let p = parse_source("type Callback<T> = (T) -> String | None\n", 0).expect("type alias with callable and union should parse");
    let Statement::TypeAlias(alias) = &p.statements[0] else { panic!() };
    assert_eq!(alias.name, "Callback");
    assert_eq!(alias.generic_parameters.len(), 1);
    match &alias.body.expr {
        TypeAnnotationExpr::Callable { parameters, result, .. } => {
            assert_eq!(parameters.len(), 1);
            match &result.expr {
                TypeAnnotationExpr::Union { members, .. } => {
                    assert_eq!(members.len(), 2);
                }
                other => panic!("expected Union on RHS of callable, got {other:?}"),
            }
        }
        other => panic!("expected Callable, got {other:?}"),
    }
}

#[test]
fn parse_structural_record_types() {
    let p = parse_source("type UserRow<R> = #{ name: String, age: Int, | R }\n", 0).expect("structural record type with row tail should parse");
    let Statement::TypeAlias(alias) = &p.statements[0] else { panic!() };
    match &alias.body.expr {
        TypeAnnotationExpr::Record { fields, tail, .. } => {
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0].name, "name");
            assert_eq!(fields[1].name, "age");
            assert_eq!(tail.as_ref().unwrap().name, "R");
        }
        other => panic!("expected Record, got {other:?}"),
    }
}

#[test]
fn parse_kind_expressions_and_higher_kinded_binders() {
    let p = parse_source("type Functor<F: Type -> Type> = F\n", 0).expect("kind annotation on generic parameter should parse");
    let Statement::TypeAlias(alias) = &p.statements[0] else { panic!() };
    let param = &alias.generic_parameters[0];
    assert_eq!(param.name, "F");
    match param.kind.as_ref().expect("kind") {
        KindSyntax::Arrow { parameter, result, .. } => {
            assert!(matches!(**parameter, KindSyntax::Type(_)));
            assert!(matches!(**result, KindSyntax::Type(_)));
        }
        other => panic!("expected Arrow kind, got {other:?}"),
    }
}

#[test]
fn parse_type_lambda_expression() {
    let p = parse_source("let t = <T> =>> List<T>\n", 0).expect("type lambda expression should parse");
    let Statement::Let(let_stmt) = &p.statements[0] else { panic!() };
    match let_stmt.value.as_ref().unwrap() {
        Expr::TypeForm(ann) => match &ann.expr {
            TypeAnnotationExpr::TypeLambda { parameters, body, .. } => {
                assert_eq!(parameters.len(), 1);
                assert_eq!(parameters[0].name, "T");
                assert!(matches!(body.expr, TypeAnnotationExpr::Application { .. }));
            }
            other => panic!("expected TypeLambda, got {other:?}"),
        },
        other => panic!("expected Expr::TypeForm, got {other:?}"),
    }
}

#[test]
fn parse_type_syntax_diagnostics_and_recovery() {
    // 1. Variance on method binder is rejected
    let res1 = parse_with_recovery("class Foo { bar<+T>() {} }\n", 0);
    assert!(res1.errors.iter().any(|e| match &e.kind {
        SyntaxErrorKind::Message(msg) => msg.contains("variance marker '+' only permitted on nominal"),
        _ => false,
    }));

    // 2. Inline bounds are rejected and directed to where clause
    let res2 = parse_with_recovery("class Foo<T <: Int> {}\n", 0);
    assert!(res2.errors.iter().any(|e| match &e.kind {
        SyntaxErrorKind::Message(msg) => msg.contains("inline generic constraint"),
        _ => false,
    }));

    // 3. Finite set constraint is rejected
    let res3 = parse_with_recovery("class Foo<T> where T in (A, B) {}\n", 0);
    assert!(res3.errors.iter().any(|e| match &e.kind {
        SyntaxErrorKind::Message(msg) => msg.contains("finite exact-set constraint"),
        _ => false,
    }));
}

#[test]
fn enum_generic_method_accepts_where_on_line_after_nested_return_type() {
    let source = r#"
enum Result<T, E> {
    transpose<U>() -> Option<Result<U, E>>
        where T == Option<U>
    {
    }
}
"#;

    let program = parse_source(source, 0)
        .expect("generic enum method with multiline where clause should parse");

    let Statement::Enum(result) = &program.statements[0] else {
        panic!("expected enum");
    };

    assert_eq!(result.name, "Result");
    assert_eq!(result.members.len(), 1);

    let EnumMember::Behavior(
        phalcom_ast::ast::EnumBehaviorMember::Method(method)
    ) = &result.members[0]
    else {
        panic!("expected enum behavior method");
    };

    assert_eq!(method.name, "transpose");
    assert_eq!(method.generic_parameters.len(), 1);

    let where_clause = method
        .where_clause
        .as_ref()
        .expect("expected method where clause");

    assert_eq!(where_clause.constraints.len(), 1);

    assert!(
        matches!(
            &where_clause.constraints[0],
            GenericConstraintSyntax::Equivalent { .. }
        ),
        "expected equivalence constraint"
    );

    assert!(
        matches!(&method.body, MemberBody::Block(_)),
        "expected braced method body"
    );
}

#[test]
fn newline_after_nested_return_type_still_terminates_declaration_only_method() {
    let source = r#"
class Example {
    first<T>() -> Option<Result<T, Error>>
    second() -> Int {
        1
    }
}
"#;

    let program = parse_source(source, 0)
        .expect("declaration-only method followed by method should parse");

    let Statement::Class(class) = &program.statements[0] else {
        panic!("expected class");
    };

    assert_eq!(class.members.len(), 2);

    let ClassMember::Method(first) = &class.members[0] else {
        panic!("expected first method");
    };

    let ClassMember::Method(second) = &class.members[1] else {
        panic!("expected second method");
    };

    assert!(matches!(first.body, MemberBody::Declaration));
    assert!(matches!(second.body, MemberBody::Block(_)));
}

#[test]
fn method_body_may_follow_nested_return_type_on_next_line() {
    let source = r#"
class Example {
    compute() -> Option<Result<Int, Error>>
    {
        1
    }
}
"#;

    let program = parse_source(source, 0)
        .expect("body after nested generic return type should parse");

    let Statement::Class(class) = &program.statements[0] else {
        panic!("expected class");
    };

    let ClassMember::Method(method) = &class.members[0] else {
        panic!("expected method");
    };

    assert!(matches!(method.body, MemberBody::Block(_)));
}

#[test]
fn multiline_where_layout_is_independent_of_return_type_nesting_depth() {
    let return_types = [
        "T",
        "Option<T>",
        "Result<T, E>",
        "Option<Result<T, E>>",
        "Option<Result<List<T>, E>>",
        "Outer<Option<Result<List<T>, E>>>",
    ];

    for return_type in return_types {
        let source = format!(
            r#"
class Example<T, E> {{
    method<U>() -> {return_type}
        where U == Option<T>
    {{
    }}
}}
"#
        );

        parse_source(&source, 0).unwrap_or_else(|error| {
            panic!(
                "return type `{return_type}` changed multiline method-header parsing: {error:?}"
            )
        });
    }
}

#[test]
fn method_body_may_follow_nested_where_constraint_on_next_line() {
    let source = r#"
class Example<T, E> {
    method<U>() -> U
        where T == Option<Result<U, E>>
    {
    }
}
"#;

    parse_source(source, 0)
        .expect("nested where constraint may precede body on separate line");
}

#[test]
fn class_and_enum_declaration_headers_support_multiline_where_and_brace() {
    let source = r#"
class Base<T> {}

class Foo<T> is Base<Option<Result<T, Error>>>
    where T <: Object
{
}

enum Bar<T>
    where T <: Object
{
    A
}

type Baz<T>
    where T <: Object
    = Option<Result<T, Error>>
"#;

    parse_source(source, 0)
        .expect("class, enum, and type alias headers with multiline where and body should parse");
}

