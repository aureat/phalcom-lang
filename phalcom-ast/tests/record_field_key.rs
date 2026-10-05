use phalcom_ast::{
    ast::{
        AnonymousRecordRemainderSyntax, Expr, Pattern, RecordKeySyntax, RecordRowTailSyntax, Statement,
        TypeAnnotationExpr,
    },
    parse_source,
};

#[test]
fn parse_record_type_symbol_and_computed_keys() {
    let src = "type R = { name: String, [\"mime\"]: String, [42]: Handler, [true]: Enabled }\n";
    let program = parse_source(src, 0).expect("record type with mixed keys should parse");
    let Statement::TypeDefinition(alias) = &program.statements[0] else { panic!() };
    let ann = alias.type_form_body().expect("type body");
    let TypeAnnotationExpr::Record { fields, tail, .. } = &ann.expr else { panic!() };
    assert_eq!(fields.len(), 4);
    assert!(tail.is_none());

    // Field 0: Symbol key
    assert!(matches!(&fields[0].key, RecordKeySyntax::Symbol { name, .. } if name == "name"));
    assert!(!fields[0].optional);

    // Field 1: Computed string literal
    assert!(matches!(&fields[1].key, RecordKeySyntax::Computed { expr, .. } if matches!(expr.as_ref(), Expr::String { value, .. } if value == "mime")));
    assert!(!fields[1].optional);

    // Field 2: Computed int literal
    assert!(matches!(&fields[2].key, RecordKeySyntax::Computed { expr, .. } if matches!(expr.as_ref(), Expr::Int { digits, .. } if digits == "42")));
    assert!(!fields[2].optional);

    // Field 3: Computed bool literal
    assert!(matches!(&fields[3].key, RecordKeySyntax::Computed { expr, .. } if matches!(expr.as_ref(), Expr::Boolean { value: true, .. })));
    assert!(!fields[3].optional);
}

#[test]
fn parse_record_type_optional_keys() {
    let src = "type R = { name?: String, [\"mime\"]?: String }\n";
    let program = parse_source(src, 0).expect("record type with optional keys should parse");
    let Statement::TypeDefinition(alias) = &program.statements[0] else { panic!() };
    let ann = alias.type_form_body().expect("type body");
    let TypeAnnotationExpr::Record { fields, .. } = &ann.expr else { panic!() };
    assert_eq!(fields.len(), 2);
    assert!(fields[0].optional);
    assert!(fields[1].optional);
}

#[test]
fn parse_record_type_tails() {
    // 1. Unconstrained: { ... }
    let p1 = parse_source("type R = { ... }\n", 0).unwrap();
    let Statement::TypeDefinition(a1) = &p1.statements[0] else { panic!() };
    let TypeAnnotationExpr::Record { tail: Some(RecordRowTailSyntax::Anonymous(AnonymousRecordRemainderSyntax::Unconstrained(_))), .. } = &a1.type_form_body().unwrap().expr else { panic!() };

    // 2. TypedValue: { **: Value }
    let p2 = parse_source("type R = { **: Value }\n", 0).unwrap();
    let Statement::TypeDefinition(a2) = &p2.statements[0] else { panic!() };
    let TypeAnnotationExpr::Record { tail: Some(RecordRowTailSyntax::Anonymous(AnonymousRecordRemainderSyntax::TypedValue { value_type, .. })), .. } = &a2.type_form_body().unwrap().expr else { panic!() };
    assert!(matches!(&value_type.expr, TypeAnnotationExpr::Reference(n) if n.root == "Value"));

    // 3. TypedKey: { **[String]: ... }
    let p3 = parse_source("type R = { **[String]: ... }\n", 0).unwrap();
    let Statement::TypeDefinition(a3) = &p3.statements[0] else { panic!() };
    let TypeAnnotationExpr::Record { tail: Some(RecordRowTailSyntax::Anonymous(AnonymousRecordRemainderSyntax::TypedKey { key_type, .. })), .. } = &a3.type_form_body().unwrap().expr else { panic!() };
    assert!(matches!(&key_type.expr, TypeAnnotationExpr::Reference(n) if n.root == "String"));

    // 4. Typed: { **[String]: Handler }
    let p4 = parse_source("type R = { **[String]: Handler }\n", 0).unwrap();
    let Statement::TypeDefinition(a4) = &p4.statements[0] else { panic!() };
    let TypeAnnotationExpr::Record { tail: Some(RecordRowTailSyntax::Anonymous(AnonymousRecordRemainderSyntax::Typed { key_type, value_type, .. })), .. } = &a4.type_form_body().unwrap().expr else { panic!() };
    assert!(matches!(&key_type.expr, TypeAnnotationExpr::Reference(n) if n.root == "String"));
    assert!(matches!(&value_type.expr, TypeAnnotationExpr::Reference(n) if n.root == "Handler"));

    // 5. Typed with Int / FieldKey: { **[Int]: Value }, { **[FieldKey]: Value }
    let p5 = parse_source("type R = { **[Int]: Value }\n", 0).unwrap();
    let Statement::TypeDefinition(a5) = &p5.statements[0] else { panic!() };
    let TypeAnnotationExpr::Record { tail: Some(RecordRowTailSyntax::Anonymous(AnonymousRecordRemainderSyntax::Typed { key_type, .. })), .. } = &a5.type_form_body().unwrap().expr else { panic!() };
    assert!(matches!(&key_type.expr, TypeAnnotationExpr::Reference(n) if n.root == "Int"));

    let p5b = parse_source("type R = { **[FieldKey]: Value }\n", 0).unwrap();
    let Statement::TypeDefinition(a5b) = &p5b.statements[0] else { panic!() };
    let TypeAnnotationExpr::Record { tail: Some(RecordRowTailSyntax::Anonymous(AnonymousRecordRemainderSyntax::Typed { key_type, .. })), .. } = &a5b.type_form_body().unwrap().expr else { panic!() };
    assert!(matches!(&key_type.expr, TypeAnnotationExpr::Reference(n) if n.root == "FieldKey"));

    // 6. Captured: { **Tail }
    let p6 = parse_source("type R<Tail> = { **Tail }\n", 0).unwrap();
    let Statement::TypeDefinition(a6) = &p6.statements[0] else { panic!() };
    let TypeAnnotationExpr::Record { tail: Some(RecordRowTailSyntax::Captured(cap)), .. } = &a6.type_form_body().unwrap().expr else { panic!() };
    assert_eq!(cap.name, "Tail");

    // 7. Mixed fields and tail: { x: Int, **[String]: Value }
    let p7 = parse_source("type R = { x: Int, **[String]: Value }\n", 0).unwrap();
    let Statement::TypeDefinition(a7) = &p7.statements[0] else { panic!() };
    let TypeAnnotationExpr::Record { fields, tail: Some(RecordRowTailSyntax::Anonymous(AnonymousRecordRemainderSyntax::Typed { .. })), .. } = &a7.type_form_body().unwrap().expr else { panic!() };
    assert_eq!(fields.len(), 1);
}

#[test]
fn parse_record_patterns() {
    let src = "let { name, [\"mime\"]: mime, [42]: handler, [true]: enabled, **rest } = val\n";
    let program = parse_source(src, 0).expect("record pattern with mixed keys should parse");
    let Statement::Let(stmt) = &program.statements[0] else { panic!() };
    let Pattern::Record { entries, rest: Some(rest), .. } = &stmt.pattern else { panic!() };
    assert_eq!(entries.len(), 4);

    // Shorthand symbol entry
    assert!(matches!(&entries[0].key, RecordKeySyntax::Symbol { name, .. } if name == "name"));
    assert!(matches!(&entries[0].pattern, Pattern::Name { name, .. } if name == "name"));

    // Computed string entry
    assert!(matches!(&entries[1].key, RecordKeySyntax::Computed { expr, .. } if matches!(expr.as_ref(), Expr::String { value, .. } if value == "mime")));
    assert!(matches!(&entries[1].pattern, Pattern::Name { name, .. } if name == "mime"));

    // Computed int entry
    assert!(matches!(&entries[2].key, RecordKeySyntax::Computed { expr, .. } if matches!(expr.as_ref(), Expr::Int { digits, .. } if digits == "42")));
    assert!(matches!(&entries[2].pattern, Pattern::Name { name, .. } if name == "handler"));

    // Computed bool entry
    assert!(matches!(&entries[3].key, RecordKeySyntax::Computed { expr, .. } if matches!(expr.as_ref(), Expr::Boolean { value: true, .. })));
    assert!(matches!(&entries[3].pattern, Pattern::Name { name, .. } if name == "enabled"));

    // Rest pattern
    assert!(matches!(rest.as_ref(), Pattern::Name { name, .. } if name == "rest"));
}

#[test]
fn reject_malformed_record_syntax() {
    // Computed pattern entry without `: pattern` must fail
    assert!(parse_source("let { [42] } = val\n", 0).is_err());
    assert!(parse_source("let { [\"key\"] } = val\n", 0).is_err());

    // Incomplete remainder syntax in record type
    assert!(parse_source("type R = { **[: Value }\n", 0).is_err());
    assert!(parse_source("type R = { **[String] }\n", 0).is_err());
}

#[test]
fn tuple_grammar_remains_unchanged() {
    // Standard tuple types parse
    assert!(parse_source("type T = (Int, String)\n", 0).is_ok());
    assert!(parse_source("type T = (name: Int, enabled: Bool)\n", 0).is_ok());
    assert!(parse_source("type T = (Int, ...)\n", 0).is_ok());
    assert!(parse_source("type T = (Int, *: String)\n", 0).is_ok());

    // Tuple does NOT accept computed keys
    assert!(parse_source("type T = ([42]: Int)\n", 0).is_err());
    assert!(parse_source("type T = ([\"mime\"]: String)\n", 0).is_err());

    // Tuple does NOT accept record remainder syntax
    assert!(parse_source("type T = (Int, **[String]: Value)\n", 0).is_err());
}
