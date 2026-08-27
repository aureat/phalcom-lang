from pathlib import Path

path = Path("phalcom-semantic/tests/constructor_factory_probe.rs")
text = path.read_text()
marker = "fn untyped_parameterized_callable_publishes_source_identity_even_without_complete_signature()"
if marker in text:
    raise SystemExit("probe already present")

text += r'''

#[test]
fn untyped_parameterized_callable_publishes_source_identity_even_without_complete_signature() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(
        r#"
class Point {
  move(_ x) { }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    let snapshot = analysis.snapshot;

    let point = DeclarationId::new(module.clone(), "Point".into());
    let callable = CallableId::new(
        point,
        Selector::method("move", vec![SelectorSlot::Positional]).unwrap(),
        DispatchSide::Instance,
    );
    let name_start = source.find("move(_ x)").expect("method declaration");

    let source_info = snapshot
        .source_index()
        .callable_source(&callable)
        .expect("source index must publish exact callable declaration identity");
    assert_eq!(source_info.name_range.start, name_start);
    assert_eq!(
        snapshot.editor().target_at(&module, name_start),
        Some(SemanticTargetId::Callable(callable.clone())),
        "editor target query must resolve the untyped method declaration"
    );

    eprintln!("SOURCE_INFO={source_info:#?}");
    eprintln!("SIGNATURE={:#?}", snapshot.callable_signatures().get(&callable));
    assert!(
        snapshot.callable_signatures().get(&callable).is_none(),
        "probe expects the hover null to be caused by complete signature absence"
    );
}
'''
path.write_text(text)
print("untyped callable hover probe added")
