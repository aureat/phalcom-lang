use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_modules::ModuleId;
use phalcom_semantic::types::TypeTerm;
use phalcom_semantic::{CallableId, DeclarationId, DispatchSide, SemanticTargetId, analyze_single_module};
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
