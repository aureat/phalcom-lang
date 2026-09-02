//! Constructor surface and exact variant identity scenarios.

use super::support::analyze_adt;
use crate::semantic::support::{Fixture, applied};
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::types::evidence::{TypeKnowledge, UnknownReason};
use phalcom_semantic::types::store::TypeData;
use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_semantic::checker::AssociatedResolutionKind;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::enum_semantics::VariantShape;
use phalcom_semantic::identity::DispatchSide;

#[test]
fn singleton_nullary_and_payload_constructors_remain_distinct() {
    let case = analyze_adt(
        r#"
enum Animal {
    @variant Dog
    @variant Dog()
    @variant Dog(_ name: String)
}
"#,
    );
    case.assert_no_diagnostics();

    let singleton = case.variant("Animal", Selector::getter("Dog").expect("singleton selector"));
    let nullary = case.variant("Animal", Selector::method("Dog", []).expect("nullary selector"));
    let payload = case.variant("Animal", Selector::method("Dog", [SelectorSlot::Positional]).expect("payload selector"));

    assert_eq!(singleton.shape, VariantShape::Singleton);
    assert_eq!(nullary.shape, VariantShape::Constructor);
    assert_eq!(payload.shape, VariantShape::Constructor);
    assert!(singleton.constructor.is_none());
    assert_eq!(nullary.constructor.as_ref().expect("nullary signature").parameters.len(), 0);
    assert_eq!(payload.fields.len(), 1);
    assert_eq!(payload.fields[0].id, case.field_id(&payload.id, 0));
    assert_eq!(payload.fields[0].local_name.as_ref(), "name");
}

#[test]
fn adt_constr_01_payload_constructor_publishes_exact_and_root_result() {
    let case =
        analyze_adt("enum Option<T> { @variant Some(_ value: T) -> Option<T> @variant None -> Option<T> }\n\nclass Test { run() { Option<Int>::Some(42) } }\n");
    let some = case.variant("Option", Selector::method("Some", [SelectorSlot::Positional]).expect("Some"));
    assert_eq!(some.fields.len(), 1);
    let resolution = case
        .analysis
        .snapshot
        .callable_analyses
        .values()
        .flat_map(|callable| callable.associated_resolutions.values())
        .find(|resolution| matches!(resolution.kind, AssociatedResolutionKind::StaticInvoke { .. }))
        .expect("Some constructor resolution");
    assert!(matches!(resolution.kind, AssociatedResolutionKind::StaticInvoke { .. }));
}

#[test]
fn adt_constr_02_nullary_invocation_does_not_resolve_singleton_getter() {
    let case = analyze_adt("enum Animal { @variant Dog() }\nclass Test { run() { Animal::Dog() } }\n");
    let dog = case.variant("Animal", Selector::method("Dog", []).expect("Dog"));
    assert_eq!(dog.shape, VariantShape::Constructor);
    assert!(case.diagnostics_for(DiagnosticCode::AssociatedMemberMissing).is_empty());
}

#[test]
fn adt_constr_03_singleton_access_is_exact_value_not_zero_arg_call() {
    let case = analyze_adt("enum Animal { @variant Dog }\nclass Test { run() { Animal::Dog } }\n");
    let dog = case.variant("Animal", Selector::getter("Dog").expect("Dog"));
    assert_eq!(dog.shape, VariantShape::Singleton);
    assert!(dog.constructor.is_none());
}

#[test]
fn adt_constr_04_wrong_label_reports_member_or_selector_diagnostic() {
    let case = analyze_adt("enum Animal { @variant Dog(named age: Int) }\nclass Test { run() { Animal::Dog(other: 1) } }\n");
    assert!(
        case.diagnostics()
            .any(|diagnostic| { diagnostic.code == DiagnosticCode::AssociatedCallShapeMissing || diagnostic.code == DiagnosticCode::AssociatedMemberMissing })
    );
}

#[test]
fn adt_constr_05_generic_constructor_keeps_specialized_payload_contract() {
    let case = analyze_adt("enum Result<T, E> { @variant Ok(_ value: T) -> Result<T, E> @variant Err(_ error: E) -> Result<T, E> }\n");
    let ok = case.variant("Result", Selector::method("Ok", [SelectorSlot::Positional]).expect("Ok"));
    let err = case.variant("Result", Selector::method("Err", [SelectorSlot::Positional]).expect("Err"));
    assert_eq!(ok.constructor.as_ref().expect("Ok constructor").parameters[0].field, ok.fields[0].id);
    assert_eq!(err.constructor.as_ref().expect("Err constructor").parameters[0].field, err.fields[0].id);
    assert_ne!(ok.exact_case_template, err.exact_case_template);
}

#[test]
fn adt_constr_06_exact_constructor_and_family_keep_distinct_member_identity() {
    let case = analyze_adt("enum Animal { @variant Dog @variant Dog() @variant Dog(_ name: String) }\n");
    let info = case.enum_info("Animal");
    let family = case.family_id("Animal", "Dog");
    assert_eq!(info.variant_families.as_ref(), std::slice::from_ref(&family));
    assert!(info.variants.iter().all(|variant| variant.family() == Some(family.clone())));
    assert_eq!(info.variants.len(), 3);
}

#[test]
fn adt_constr_07_payload_constructor_keeps_unmentioned_owner_parameter_underconstrained() {
    let fixture = Fixture::new(
        r#"
enum Result<T, E> {
  @variant Ok(_ value: T) -> Result<T, E>
  @variant Err(_ error: E) -> Result<T, E>
}

class Probe {
  @class
  run() {
    let value = Result::Ok(1)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Result::Ok(1)");
    assert_eq!(call.knowledge, TypeKnowledge::Unknown(UnknownReason::UnderconstrainedTypeVariable));
    assert!(matches!(call.status, AnalysisStatus::Blocked(_)));
}

#[test]
fn adt_constr_08_context_selects_all_result_constructor_parameters() {
    let fixture = Fixture::new(
        r#"
enum Result<T, E> {
  @variant Ok(_ value: T) -> Result<T, E>
  @variant Err(_ error: E) -> Result<T, E>
}

class Probe {
  @class
  run() {
    let value: Result<Int, String> = Result::Ok(1)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Result::Ok(1)");
    let enum_type = match fixture.analysis.snapshot.store.get(call.knowledge.ty().expect("contextual constructor result")) {
        TypeData::ExactCase { enum_type, .. } => *enum_type,
        other => panic!("expected exact constructor case, got {other:?}"),
    };
    fixture.assert_type(enum_type, applied("Result", [fixture.ty("Int").into(), fixture.ty("String").into()]));
    assert!(matches!(call.status, AnalysisStatus::Ready));
}

#[test]
fn adt_constr_09_nullary_constructor_uses_result_context_for_owner_parameter() {
    let fixture = Fixture::new(
        r#"
enum Option<T> {
  @variant Some(_ value: T) -> Option<T>
  @variant None() -> Option<T>
}

class Probe {
  @class
  run() {
    let value: Option<Int> = Option::None()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Option::None()");
    let enum_type = match fixture.analysis.snapshot.store.get(call.knowledge.ty().expect("contextual nullary result")) {
        TypeData::ExactCase { enum_type, .. } => *enum_type,
        other => panic!("expected exact constructor case, got {other:?}"),
    };
    fixture.assert_type(enum_type, applied("Option", [fixture.ty("Int").into()]));
}

#[test]
fn adt_constr_10_nullary_constructor_without_context_remains_underconstrained() {
    let fixture = Fixture::new(
        r#"
enum Option<T> {
  @variant Some(_ value: T) -> Option<T>
  @variant None() -> Option<T>
}

class Probe {
  @class
  run() {
    let value = Option::None()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Option::None()");
    assert_eq!(call.knowledge, TypeKnowledge::Unknown(UnknownReason::UnderconstrainedTypeVariable));
    assert!(matches!(call.status, AnalysisStatus::Blocked(_)));
    fixture.assert_diagnostic(DiagnosticCode::GenericInferenceUnderconstrained, 1);
}

#[test]
fn adt_constr_11_unresolved_payload_type_blocks_without_object_fallback() {
    let fixture = Fixture::new(
        r#"
enum Broken<T> {
  @variant Value(_ payload: MissingType) -> Broken<T>
}

class Probe {
  @class
  run() {
    let family = Broken::Value::*
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let family = fixture.expression(run, "Broken::Value::*");
    assert_eq!(family.knowledge.ty(), None);
    assert!(matches!(family.knowledge, TypeKnowledge::Unknown(UnknownReason::InferenceBlocked)));
    assert!(
        matches!(family.status, AnalysisStatus::Blocked(_) | AnalysisStatus::Invalid(_)),
        "unexpected family status: {:?}",
        family.status
    );
    fixture.assert_diagnostic(DiagnosticCode::AssociatedMemberMissing, 1);
}

#[test]
fn ordinary_constructor_infers_from_formal_parameter_types_not_argument_position() {
    let fixture = Fixture::new(
        r#"
class Pair<A, B> {
  @constructor
  new(_ second: B, _ first: A) {}
}

class Probe {
  @class
  run() {
    let value = Pair.new("text", 1)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let value = fixture.binding(run, "value");
    fixture.assert_type(value.current.ty().expect("constructor result"), applied("Pair", [fixture.ty("Int").into(), fixture.ty("String").into()]));
}
