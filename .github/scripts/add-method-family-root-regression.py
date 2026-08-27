from pathlib import Path

p = Path("phalcom-semantic/tests/constructor_factory_probe.rs")
text = p.read_text()
marker = "\nfn builtin_annotation_snapshot()"
if marker not in text:
    raise SystemExit("constructor probe insertion marker missing")
test = r'''

#[test]
fn class_object_method_family_uses_canonical_dispatch_owner_chain() {
    let module = ModuleId::resolved(ResolvedProjectId::from_raw(91), ModulePath::root());
    let source: Arc<str> = Arc::from(
        r#"
class Person {}
const family = Person::new(...)
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    let snapshot = analysis.snapshot;

    let default_new = CallableId::new(
        DeclarationId::new(ModuleId::core(), "Class".into()),
        Selector::method("new", Vec::new()).unwrap(),
        DispatchSide::Instance,
    );
    let module_index = snapshot.source_index.module(&module).expect("source index");
    let family_binding = module_index
        .structure
        .bindings
        .values()
        .find(|binding| binding.name.as_ref() == "family")
        .expect("family binding");
    let fact = snapshot.advisory_fact(&family_binding.declaration_site).expect("family advisory fact");
    let ValueShape::MethodFamily(family) = &fact.shape else {
        panic!("expected captured method family, got {fact:#?}");
    };
    assert!(
        family.exact.iter().any(|(_, callable)| callable == &default_new),
        "class-object family capture must traverse into canonical Class instance behavior: {family:#?}"
    );
}
'''
text = text.replace(marker, test + marker, 1)
p.write_text(text)
print("method family root regression added")
