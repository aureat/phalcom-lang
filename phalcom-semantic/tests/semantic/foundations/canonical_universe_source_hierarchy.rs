//! Canonical Universe source-authored superclass edges must participate in
//! ordinary subtype checks and contextual generic variant construction.

use crate::semantic::support::{Fixture, applied};
use phalcom_semantic::diagnostic::DiagnosticSeverity;
use phalcom_semantic::identity::{DeclarationId, DispatchSide};
use phalcom_semantic::is_subtype;
use phalcom_semantic::TypeId;
use phalcom_semantic::types::store::TypeData;
use phalcom_modules::{ModuleComponent, ModuleId, ModulePath};

fn nominal_name(fixture: &Fixture, ty: phalcom_semantic::TypeId) -> &str {
    fixture
        .analysis
        .snapshot
        .store
        .nominal_origin_declaration(ty)
        .unwrap_or_else(|| panic!("expected nominal type, got {:?}", fixture.analysis.snapshot.store.get(ty)))
        .name
        .as_ref()
}

fn canonical_source_nominal(fixture: &Fixture, module: &str, name: &str) -> TypeId {
    let path = module
        .split('.')
        .map(|component| ModuleComponent::from_identifier(component).expect("canonical module component"))
        .collect::<Vec<_>>();
    let declaration = DeclarationId::new(ModuleId::universe(ModulePath::from_components(path)), name.into());
    let mut store = (*fixture.analysis.snapshot.store).clone();
    store.nominal_type(declaration)
}

#[test]
fn ordinary_user_class_widening_is_a_control() {
    let fixture = Fixture::new(
        r#"
class Base {}
class Derived is Base {
  @constructor
  new() {}
}
class Probe {
  @class
  run() {
    let value: Base = Derived.new()
  }
}
"#,
    );

    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    fixture.assert_expression_established(&fixture.expression(run, "Derived.new()"), fixture.ty("Derived"));
    assert_eq!(fixture.binding(run, "value").contract.as_ref().unwrap().ty, fixture.ty("Base"));
    fixture.assert_subtype(fixture.ty("Derived"), fixture.ty("Base"));
    fixture.assert_no_error_diagnostics();
}

#[test]
fn generic_result_variant_uses_expected_context_with_user_subclass() {
    let fixture = Fixture::new(
        r#"
class Base {}
class Derived is Base {
  @constructor
  new() {}
}
class Probe {
  @class
  run() {
    let result: Result<Unit, Base> = Result::Error(Derived.new())
  }
}
"#,
    );

    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    fixture.assert_no_error_diagnostics();
    let expected = applied("Result", [fixture.analysis.snapshot.store.unit().into(), fixture.ty("Base").into()]);
    fixture.assert_type(fixture.binding(run, "result").contract.as_ref().unwrap().ty, expected.clone());
    let result = fixture.expression(run, "Result::Error(Derived.new())");
    let result_type = match fixture.analysis.snapshot.store.get(result.knowledge.ty().unwrap()) {
        TypeData::ExactCase { enum_type, .. } => *enum_type,
        other => panic!("expected a Result variant case type, got {other:?}"),
    };
    fixture.assert_type(result_type, expected);
    assert!(result.knowledge.status().is_some(), "contextually inferred Result type must be established");
    fixture.assert_no_error_diagnostics();
}

#[test]
fn canonical_universe_error_subclasses_widen_without_erasing_expression_types() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  run() {
    let duplicate = DuplicateKeyError.new(1)
    let base: Error = Error.new("base")
  }
}
"#,
    );

    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let duplicate = fixture.expression(run, "DuplicateKeyError.new(1)");
    let duplicate_type = duplicate.knowledge.ty().unwrap();
    assert_eq!(nominal_name(&fixture, duplicate_type), "DuplicateKeyError");
    let error_type = canonical_source_nominal(&fixture, "errors.error", "Error");
    let duplicate_source_type = canonical_source_nominal(&fixture, "errors.argument", "DuplicateKeyError");
    let index_source_type = canonical_source_nominal(&fixture, "errors.argument", "IndexError");
    assert_eq!(duplicate_type, duplicate_source_type);
    fixture.assert_subtype(duplicate_type, error_type);
    fixture.assert_subtype(index_source_type, error_type);
    fixture.assert_no_error_diagnostics();
}

#[test]
fn real_universe_error_subclasses_construct_broad_result_without_widening_payloads() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  duplicate() -> Result<Unit, Error> {
    Result::Error(DuplicateKeyError.new(1))
  }
}
"#,
    );

    fixture.assert_no_error_diagnostics();
    let duplicate = fixture.callable("Probe", "duplicate", DispatchSide::Class);
    let duplicate_signature = fixture.analysis.snapshot.callable_signatures.get_for_body(&duplicate.callable).unwrap();
    let duplicate_result_type = duplicate_signature.declared_return.canonical_type().unwrap();
    let duplicate_case = fixture.expression(duplicate, "Result::Error(DuplicateKeyError.new(1))");
    let duplicate_case_enum = match fixture.analysis.snapshot.store.get(duplicate_case.knowledge.ty().unwrap()) {
        TypeData::ExactCase { enum_type, .. } => *enum_type,
        other => panic!("expected a Result variant case type, got {other:?}"),
    };
    assert_eq!(duplicate_case_enum, duplicate_result_type);
    assert_eq!(
        nominal_name(&fixture, fixture.expression(duplicate, "DuplicateKeyError.new(1)").knowledge.ty().unwrap()),
        "DuplicateKeyError",
    );

}

#[test]
fn unrelated_nominal_is_not_an_error_or_a_valid_result_error_payload() {
    let fixture = Fixture::new_allowing_internal_incidents(
        r#"
class NotAnError {
  @constructor
  new() {}
}
class Probe {
  @class
  run() {
    let invalid: Error = NotAnError.new()
    let invalidResult: Result<Unit, Error> = Result::Error(NotAnError.new())
  }
}
"#,
    );

    let mut store = (*fixture.analysis.snapshot.store).clone();
    let error_type = fixture.binding(fixture.callable("Probe", "run", DispatchSide::Class), "invalid").contract.as_ref().unwrap().ty;
    let not_error_type = fixture.ty("NotAnError");
    assert!(
        !is_subtype(&mut store, fixture.analysis.snapshot.hierarchy.as_ref(), not_error_type, error_type),
        "an unrelated class must not be treated as an Error subclass",
    );
    let errors = fixture
        .analysis
        .snapshot
        .all_diagnostics()
        .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
        .collect::<Vec<_>>();
    assert!(!errors.is_empty(), "unrelated Error assignment and Result payload should be rejected");
}
