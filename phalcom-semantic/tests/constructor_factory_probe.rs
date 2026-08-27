use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_modules::ModuleId;
use phalcom_semantic::types::TypeTerm;
use phalcom_semantic::{CallableId, DeclarationId, DispatchSide, EditorMemberTarget, SemanticTargetId, ValueShape, analyze_single_module};
use std::sync::Arc;

#[test]
fn constructor_factory_publishes_target_and_inferred_signature() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(
        r#"
class CellNum {
  _raw: Int

  @constructor
  new(_ raw: Int) {
    _raw = raw
  }

  @class
  of(_ raw: Int) {
    CellNum.new(raw)
  }

  value() -> Int {
    _raw
  }
}

const x: Int = CellNum.of(42)
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    let snapshot = analysis.snapshot;

    let cell = DeclarationId::new(module.clone(), "CellNum".into());
    let cell_ty = snapshot.declarations.form(&cell).expect("CellNum type");
    let constructor = CallableId::new(
        cell.clone(),
        Selector::method("new", vec![SelectorSlot::Positional]).unwrap(),
        DispatchSide::Class,
    );
    let factory = CallableId::new(
        cell.clone(),
        Selector::method("of", vec![SelectorSlot::Positional]).unwrap(),
        DispatchSide::Class,
    );

    let factory_analysis = snapshot.callable_analyses.get(&factory).expect("CellNum.of body analysis");
    let constructor_call = factory_analysis
        .expressions
        .values()
        .find(|expression| source.get(expression.range.start..expression.range.end) == Some("CellNum.new(raw)"))
        .expect("constructor call expression in CellNum.of");
    assert_eq!(
        constructor_call.callable.as_ref(),
        Some(&constructor),
        "factory tail must resolve to the public class-side constructor identity: {constructor_call:#?}"
    );
    assert_eq!(
        constructor_call.knowledge.ty(),
        Some(cell_ty),
        "constructor call must formally produce CellNum: {constructor_call:#?}"
    );
    assert!(
        factory_analysis.exits.normal_return_values.iter().any(|knowledge| knowledge.ty() == Some(cell_ty)),
        "CellNum.of must publish the constructor result as a normal return: {factory_analysis:#?}"
    );

    let signature = snapshot.callable_signatures.get(&factory).expect("inferred CellNum.of signature");
    assert_eq!(
        signature.return_type,
        TypeTerm::Canonical(cell_ty),
        "inferred factory signature must return CellNum"
    );

    let selector_offset = source.rfind("of(42)").expect("factory call") + 1;
    assert_eq!(
        snapshot.editor().target_at(&module, selector_offset),
        Some(SemanticTargetId::Callable(factory)),
        "top-level factory call must retain its exact canonical callable target"
    );
}

#[test]
fn constructorless_class_inherits_canonical_class_new() {
    let module = ModuleId::core();
    let source: Arc<str> = Arc::from(
        r#"
class Person {}

const person = Person.new()

class Probe {
  make() {
    Person.new()
  }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    let snapshot = analysis.snapshot;

    let person = DeclarationId::new(module.clone(), "Person".into());
    let person_ty = snapshot.declarations.form(&person).expect("Person type");
    let probe = DeclarationId::new(module.clone(), "Probe".into());
    let make = CallableId::new(
        probe,
        Selector::method("make", Vec::new()).unwrap(),
        DispatchSide::Instance,
    );
    let default_new = CallableId::new(
        DeclarationId::new(ModuleId::core(), "Class".into()),
        Selector::method("new", Vec::new()).unwrap(),
        DispatchSide::Instance,
    );

    let make_analysis = snapshot.callable_analyses.get(&make).expect("Probe.make body analysis");
    let constructor_call = make_analysis
        .expressions
        .values()
        .find(|expression| source.get(expression.range.start..expression.range.end) == Some("Person.new()"))
        .expect("Person.new expression");
    assert_eq!(
        constructor_call.callable.as_ref(),
        Some(&default_new),
        "constructor-less classes must resolve new() through canonical Class behavior: {constructor_call:#?}"
    );
    assert_eq!(
        constructor_call.knowledge.ty(),
        Some(person_ty),
        "Class.new Self result must specialize to the concrete class object receiver: {constructor_call:#?}"
    );

    let receiver_start = source.rfind("Person.new()").expect("Person.new call");
    let receiver_range = SourceRange {
        start: receiver_start,
        end: receiver_start + "Person".len(),
    };
    let editor = snapshot.editor();
    let receiver = editor
        .resolve_receiver_at(&module, receiver_range)
        .expect("Person class-object receiver");
    let access = editor.access_context_at(&module, receiver_start);
    assert!(
        editor
            .members_for_receiver(&receiver, &access)
            .iter()
            .any(|member| member.target == EditorMemberTarget::Callable(default_new.clone())),
        "editor member enumeration must share formal class-object root dispatch"
    );

    let module_index = snapshot.source_index.module(&module).expect("source index");
    let person_binding = module_index
        .structure
        .bindings
        .values()
        .find(|binding| binding.name.as_ref() == "person")
        .expect("top-level person binding");
    assert_eq!(
        snapshot.advisory_fact(&person_binding.declaration_site).map(|fact| &fact.shape),
        Some(&ValueShape::Instance(person)),
        "known Self-returning Class.new must project the concrete receiver into advisory binding shape"
    );
}
