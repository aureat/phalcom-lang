use phalcom_ast::parse;
use phalcom_common::range::SourceRange;
use phalcom_semantic::{
    EditorTypeHintKind, FormalPresentation, ModuleId, SourceBindingKind, SourceIndexContext, analyze_single_module, build_source_scope_index,
};
use std::sync::Arc;

#[test]
fn source_index_retains_explicit_annotation_truth() {
    let source = "const plain = 1\nconst typed: Int = 2\nclass Sample { run(annotated: Int, inferred) { inferred } }\n";
    let parsed = parse(source, 0);
    assert!(parsed.errors.is_empty(), "parser errors: {:?}", parsed.errors);
    let index = build_source_scope_index(ModuleId::core(), &parsed.program, &SourceIndexContext::default());

    let binding = |name: &str, kind: SourceBindingKind| {
        index
            .bindings
            .values()
            .find(|binding| binding.name.as_ref() == name && binding.kind == kind)
            .unwrap_or_else(|| panic!("missing {kind:?} binding {name}"))
    };

    assert!(!binding("plain", SourceBindingKind::TopLevelConst).has_explicit_annotation);
    assert!(binding("typed", SourceBindingKind::TopLevelConst).has_explicit_annotation);
    assert!(binding("annotated", SourceBindingKind::MethodParameter).has_explicit_annotation);
    assert!(!binding("inferred", SourceBindingKind::MethodParameter).has_explicit_annotation);
}

#[test]
fn editor_type_hints_suppress_explicit_local_annotations() {
    let source = "class Sample {\n  run() {\n    const inferred = 1\n    const explicit: Int = 2\n    inferred\n  }\n}\n";
    let parsed = parse(source, 0);
    assert!(parsed.errors.is_empty(), "parser errors: {:?}", parsed.errors);
    let module = ModuleId::core();
    let analysis = analyze_single_module(module.clone(), Arc::from(source), Arc::new(parsed.program));
    let hints = analysis.snapshot.editor().type_hints(&module, SourceRange::new(0, source.len()));

    let inferred_end = source.find("inferred").expect("inferred declaration") + "inferred".len();
    let explicit_end = source.find("explicit").expect("explicit declaration") + "explicit".len();
    let inferred = hints
        .iter()
        .find(|hint| hint.kind == EditorTypeHintKind::Binding && hint.insertion_offset == inferred_end)
        .expect("unannotated inferred binding should receive a compiler-owned type hint");

    assert!(
        matches!(inferred.formal, Some(FormalPresentation::Known(ref ty)) if ty == "Int"),
        "inferred local should retain its formal Int result: {inferred:?}"
    );
    assert!(
        hints.iter().all(|hint| hint.insertion_offset != explicit_end),
        "explicitly annotated binding must not receive a duplicate type hint: {hints:#?}"
    );
}
