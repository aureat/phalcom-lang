use crate::semantic::support::{Fixture, applied, nominal};
use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DeclarationId;
use phalcom_semantic::identity::{CallableId, InvocationTargetId};
use phalcom_semantic::types::case_environment::derive_case_environment;
use phalcom_semantic::types::case_instantiation::CaseInstantiation;
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::parameter::{TypeParameterData, TypeParameterOwner};
use phalcom_semantic::types::rigid::RigidArena;
use phalcom_semantic::types::store::{TypeData, TypeStore};

fn test_module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(42), ModulePath::root())
}

#[test]
fn gadt_case_result_binds_multiple_enum_parameters_in_declaration_order() {
    let mut store = TypeStore::new();
    let module = test_module();
    let owner = DeclarationId::new(module.clone(), "Pair".into());
    let first = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(owner.clone()), 0, "A", KindId::TYPE));
    let second = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(owner.clone()), 1, "B", KindId::TYPE));
    let int_ty = store.nominal_type(DeclarationId::new(module.clone(), "Int".into()));
    let string_ty = store.nominal_type(DeclarationId::new(module, "String".into()));
    let pair_kind = store.arrow_kind(vec![KindId::TYPE, KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let pair_form = store.nominal_form(owner.clone(), pair_kind);
    let result = store.apply_type_form(pair_form, &[int_ty, string_ty]).expect("Pair<Int, String>");

    let environment = derive_case_environment(&mut store, &owner, &[first, second], Some(result)).expect("GADT environment");

    assert_eq!(environment.bindings.get(&first), Some(&int_ty));
    assert_eq!(environment.bindings.get(&second), Some(&string_ty));
}

#[test]
fn default_gadt_result_keeps_case_environment_empty() {
    let mut store = TypeStore::new();
    let module = test_module();
    let owner = DeclarationId::new(module, "Expr".into());
    let parameter = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(owner.clone()), 0, "T", KindId::TYPE));

    let environment = derive_case_environment(&mut store, &owner, &[parameter], None).expect("default GADT environment");

    assert!(environment.is_empty());
}

#[test]
fn adt_gen_01_option_payload_substitutes_to_concrete_type() {
    let case = super::support::analyze_adt("enum Option<T> { @variant Some(_ value: T) -> Option<T> @variant None -> Option<T> }\n");
    let option = case.declaration("Option");
    let signature = option.generic_signature.as_ref().expect("Option<T>");
    let some = case.variant(
        "Option",
        phalcom_common::selector::Selector::method("Some", [phalcom_common::selector::SelectorSlot::Positional]).expect("Some"),
    );
    assert_eq!(signature.parameter_count(), 1);
    assert!(
        matches!(case.type_data(some.fields[0].declared_type.canonical_type().expect("payload")), phalcom_semantic::types::store::TypeData::Parameter(parameter) if *parameter == signature.parameters[0])
    );
}

#[test]
fn adt_gen_02_result_parameters_are_independent_per_variant() {
    let case = super::support::analyze_adt("enum Result<T, E> { @variant Ok(_ value: T) -> Result<T, E> @variant Err(_ error: E) -> Result<T, E> }\n");
    let signature = case.declaration("Result").generic_signature.as_ref().expect("Result<T,E>");
    let ok = case.variant(
        "Result",
        phalcom_common::selector::Selector::method("Ok", [phalcom_common::selector::SelectorSlot::Positional]).expect("Ok"),
    );
    let err = case.variant(
        "Result",
        phalcom_common::selector::Selector::method("Err", [phalcom_common::selector::SelectorSlot::Positional]).expect("Err"),
    );
    assert!(
        matches!(case.type_data(ok.fields[0].declared_type.canonical_type().expect("Ok payload")), phalcom_semantic::types::store::TypeData::Parameter(parameter) if *parameter == signature.parameters[0])
    );
    assert!(
        matches!(case.type_data(err.fields[0].declared_type.canonical_type().expect("Err payload")), phalcom_semantic::types::store::TypeData::Parameter(parameter) if *parameter == signature.parameters[1])
    );
}

#[test]
fn adt_gen_03_nested_generic_payload_keeps_application_shape() {
    let case = super::support::analyze_adt("enum Boxed<T> { @variant Value(_ value: Option<T>) -> Boxed<T> }\n");
    let value = case.variant(
        "Boxed",
        phalcom_common::selector::Selector::method("Value", [phalcom_common::selector::SelectorSlot::Positional]).expect("Value"),
    );
    assert!(matches!(
        case.type_data(value.fields[0].declared_type.canonical_type().expect("nested payload")),
        phalcom_semantic::types::store::TypeData::Applied { .. }
    ));
}

#[test]
fn adt_gen_04_variant_local_generic_signature_owns_payload_and_result_scope() {
    let case = super::support::analyze_adt("enum Expr<T> { @variant Pack<U>(_ value: U) -> Expr<U> where U <: Object }\n");
    let pack = case.variant(
        "Expr",
        phalcom_common::selector::Selector::method("Pack", [phalcom_common::selector::SelectorSlot::Positional]).expect("Pack"),
    );
    let constructor = pack.constructor.as_ref().expect("Pack constructor");
    let generic_signature = constructor.generic_signature.as_ref().expect("Pack<U>");
    let callable = CallableId::variant_constructor(pack.id.clone());

    assert_eq!(generic_signature.parameter_count(), 1);
    assert_eq!(generic_signature.owner, TypeParameterOwner::Callable(callable.clone()));
    assert_eq!(generic_signature.constraint_count(), 1);
    let parameter = generic_signature.parameters[0];
    assert_eq!(
        case.analysis.snapshot.store.type_parameter(parameter).owner,
        TypeParameterOwner::Callable(callable)
    );
    assert!(matches!(
        case.type_data(pack.fields[0].declared_type.canonical_type().expect("Pack payload")),
        phalcom_semantic::types::store::TypeData::Parameter(found) if *found == parameter
    ));
    assert!(matches!(
        case.type_data(pack.result_type_template),
        phalcom_semantic::types::store::TypeData::Applied { arguments, .. }
            if arguments.iter().any(|argument| matches!(case.type_data(*argument), phalcom_semantic::types::store::TypeData::Parameter(found) if *found == parameter))
    ));
}

#[test]
fn adt_gadt_04_case_instantiation_shares_one_rigid_per_variant_binder() {
    let case = super::support::analyze_adt("enum Expr<T> { @variant Equal<U>(_ left: U, _ right: U) -> Expr<T> where U <: Object }\n");
    let variant = case.variant(
        "Expr",
        phalcom_common::selector::Selector::method(
            "Equal",
            [
                phalcom_common::selector::SelectorSlot::Positional,
                phalcom_common::selector::SelectorSlot::Positional,
            ],
        )
        .expect("Equal constructor"),
    );
    let mut arena = RigidArena::new();
    let first = CaseInstantiation::open(case.analysis.snapshot.store.as_ref(), &mut arena, variant, None);
    let second = CaseInstantiation::open(case.analysis.snapshot.store.as_ref(), &mut arena, variant, None);

    assert_eq!(first.local_rigids.len(), 1);
    assert_eq!(first.payload_types.len(), 2);
    assert_eq!(first.payload_types[0], first.payload_types[1], "repeated U occurrences must share one rigid");
    assert_eq!(first.payload_types[0].free_rigids().len(), 1);
    assert_eq!(first.constraints.len(), 1, "variant-local where clause must remain branch-local evidence");
    assert_ne!(first.scope, second.scope, "independent eliminations need fresh scopes");
    assert_ne!(first.local_rigids.values().next(), second.local_rigids.values().next());
}

#[test]
fn adt_gadt_01_case_environment_is_owned_by_specialized_variant() {
    let case = super::support::analyze_adt("enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\n");
    let int = case.variant(
        "Expr",
        phalcom_common::selector::Selector::method("Int", [phalcom_common::selector::SelectorSlot::Positional]).expect("Int"),
    );
    let bool_case = case.variant(
        "Expr",
        phalcom_common::selector::Selector::method("Bool", [phalcom_common::selector::SelectorSlot::Positional]).expect("Bool"),
    );
    assert_eq!(int.case_environment.bindings.len(), 1);
    assert_eq!(bool_case.case_environment.bindings.len(), 1);
    assert_ne!(int.case_environment, bool_case.case_environment);
}

#[test]
fn adt_gadt_02_multi_parameter_relationship_is_recorded() {
    let mut store = TypeStore::new();
    let module = test_module();
    let owner = DeclarationId::new(module.clone(), "Equal".into());
    let a = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(owner.clone()), 0, "A", KindId::TYPE));
    let b = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(owner.clone()), 1, "B", KindId::TYPE));
    let a_ty = store.parameter_form(a);
    let kind = store.arrow_kind(vec![KindId::TYPE, KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let form = store.nominal_form(owner.clone(), kind);
    let result = store.apply_type_form(form, &[a_ty, a_ty]).expect("Equal<A,A>");
    let environment = derive_case_environment(&mut store, &owner, &[a, b], Some(result)).expect("equality environment");
    assert_eq!(environment.bindings.get(&b), Some(&a_ty));
}

#[test]
fn adt_gadt_03_contradictory_specialization_is_not_ordinary_subtyping_failure() {
    let case = super::support::analyze_adt(
        "enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nclass Eval { run(_ value: Expr<Bool>) { match value { Expr::Int(x) => x } } }\n",
    );
    assert_eq!(
        case.only_match().arm(0).resolution().usefulness,
        phalcom_semantic::match_semantics::PatternUsefulness::Impossible
    );
}

#[test]
fn adt_gen_05_constructor_solves_enum_and_variant_domains() {
    let fixture = Fixture::new(
        r#"
enum Expr<T> {
  @variant Pair<U>(_ first: T, _ second: U) -> Expr<T>
}

class Probe {
  @class
  run() {
    let first = Expr::Pair(1, "text")
    let second = Expr::Pair("text", 1)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", phalcom_semantic::identity::DispatchSide::Class);
    let first = fixture.expression(run, "Expr::Pair(1, \"text\")");
    let second = fixture.expression(run, "Expr::Pair(\"text\", 1)");

    let first_type = first.knowledge.ty().expect("first constructor result");
    let second_type = second.knowledge.ty().expect("second constructor result");
    let first_enum = match fixture.analysis.snapshot.store.get(first_type) {
        TypeData::ExactCase { enum_type, .. } => *enum_type,
        other => panic!("expected exact first case, got {other:?}"),
    };
    let second_enum = match fixture.analysis.snapshot.store.get(second_type) {
        TypeData::ExactCase { enum_type, .. } => *enum_type,
        other => panic!("expected exact second case, got {other:?}"),
    };
    fixture.assert_type(first_enum, applied("Expr", [nominal("Int")]));
    fixture.assert_type(second_enum, applied("Expr", [nominal("String")]));
    assert!(matches!(first.status, AnalysisStatus::Ready));
    assert!(matches!(second.status, AnalysisStatus::Ready));

    let first_resolution = run.associated_resolutions.get(&first.id).expect("first constructor resolution");
    let second_resolution = run.associated_resolutions.get(&second.id).expect("second constructor resolution");
    let first_target = match &first_resolution.kind {
        phalcom_semantic::checker::AssociatedResolutionKind::StaticInvoke { target, .. } => target,
        other => panic!("expected first static constructor, got {other:?}"),
    };
    let second_target = match &second_resolution.kind {
        phalcom_semantic::checker::AssociatedResolutionKind::StaticInvoke { target, .. } => target,
        other => panic!("expected second static constructor, got {other:?}"),
    };
    assert_eq!(first_target, second_target);
    assert!(matches!(first_target, InvocationTargetId::VariantConstructor(_)));
}

#[test]
fn adt_gen_06_constructor_payload_result_conflict_is_rejected() {
    let fixture = Fixture::new(
        r#"
enum Expr<T> {
  @variant Pair<U>(_ first: T, _ second: U) -> Expr<T>
}

class Probe {
  @class
  run() {
    let value: Expr<String> = Expr::Pair(1, "text")
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", phalcom_semantic::identity::DispatchSide::Class);
    let call = fixture.expression(run, "Expr::Pair(1, \"text\")");
    assert!(call.knowledge.ty().is_none());
    assert!(matches!(call.status, AnalysisStatus::Invalid(_)));
    fixture.assert_diagnostic(DiagnosticCode::GenericInferenceConflict, 1);
}

#[test]
fn adt_gen_07_generic_variant_family_instantiates_at_invocation() {
    let fixture = Fixture::new(
        r#"
enum Expr<T> {
  @variant Pair<U>(_ first: T, _ second: U) -> Expr<T>
}

class Probe {
  @class
  run() {
    let family = Expr<Int>::Pair::*;
    let value = family(1, "text");
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", phalcom_semantic::identity::DispatchSide::Class);
    let call = fixture.expression(run, "family(1, \"text\")");
    let result = call.knowledge.ty().expect("family constructor result");
    let enum_type = match fixture.analysis.snapshot.store.get(result) {
        TypeData::ExactCase { enum_type, .. } => *enum_type,
        other => panic!("expected exact family case, got {other:?}"),
    };
    fixture.assert_type(enum_type, applied("Expr", [nominal("Int")]));
    assert!(matches!(call.status, AnalysisStatus::Ready));
}
