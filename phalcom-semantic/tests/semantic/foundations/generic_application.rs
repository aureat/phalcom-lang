//! SC-2 generic application and expected-context laws.

use crate::semantic::support::{Fixture, applied, nominal};
use phalcom_semantic::checker::analysis::AnalysisStatus;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::evidence::{EvidenceOrigin, EvidenceStatus, UnknownReason};
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::parameter::{GenericParameterMode, GenericSignature, TypeParameterData, TypeParameterOwner};
use phalcom_semantic::types::store::TypeStore;

#[test]
fn result_only_generic_uses_expected_context_as_assumed_selection() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  make<T>() -> T { 42 }

  @class
  run() {
    let value: Int = Probe.make()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.make()");
    assert_eq!(call.knowledge.ty(), Some(fixture.ty("Int")));
    assert_eq!(call.knowledge.status(), Some(EvidenceStatus::Assumed));
    assert_eq!(call.knowledge.origin(), Some(EvidenceOrigin::GenericInference));
    assert!(matches!(call.status, AnalysisStatus::Ready));
    fixture.assert_no_diagnostic(DiagnosticCode::GenericInferenceUnderconstrained);
}

#[test]
fn solved_generic_call_publishes_canonical_call_specialization() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  make<T>() -> T { 42 }

  @class
  run() {
    let value: Int = Probe.make()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.make()");
    let specialization = call.call_specialization.as_ref().expect("solved generic call product");
    assert_eq!(specialization.callable, call.callable.clone().expect("callable identity"));
    assert_eq!(specialization.bindings.len(), 1);
    assert_eq!(specialization.bindings[0].1, fixture.ty("Int"));
}

#[test]
fn result_only_generic_without_context_remains_underconstrained() {
    let fixture = Fixture::new(
        r#"
class Probe {
  @class
  make<T>() -> T { 42 }

  @class
  run() {
    let value = Probe.make()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.make()");
    assert_eq!(
        call.knowledge,
        phalcom_semantic::types::evidence::TypeKnowledge::Unknown(UnknownReason::UnderconstrainedTypeVariable)
    );
    assert!(matches!(call.status, AnalysisStatus::Blocked(_)));
    fixture.assert_diagnostic(DiagnosticCode::GenericInferenceUnderconstrained, 1);
}

#[test]
fn declaration_upper_bound_does_not_default_result_only_generic() {
    let fixture = Fixture::new(
        r#"
class Number {}
class Probe {
  @class
  make<T>() -> T where T <: Number { 42 }

  @class
  run() {
    let value = Probe.make()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.make()");
    assert!(call.knowledge.ty().is_none());
    assert!(matches!(call.status, AnalysisStatus::Blocked(_)));
    fixture.assert_diagnostic(DiagnosticCode::GenericInferenceUnderconstrained, 1);
}

#[test]
fn candidate_is_checked_against_declaration_upper_bound() {
    let fixture = Fixture::new(
        r#"
class Number {}
class Allowed is Number { @constructor new() {} }
class Probe {
  @class
  keep<T>(_ value: T) -> T where T <: Number { value }

  @class
  run(_ value: Allowed) {
    let result = Probe.keep(value)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.keep(value)");
    assert_eq!(call.knowledge.ty(), Some(fixture.ty("Allowed")));
    assert!(matches!(call.status, AnalysisStatus::Ready));
}

#[test]
fn f_bound_is_checked_after_argument_selects_candidate() {
    let fixture = Fixture::new(
        r#"
class Comparable<T> {}
class User is Comparable<User> {}
class Probe {
  @class
  keep<T>(_ value: T) -> T where T <: Comparable<T> { value }

  @class
  run(_ value: User) {
    let result = Probe.keep(value)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.keep(value)");
    assert_eq!(call.knowledge.ty(), Some(fixture.ty("User")));
    assert!(matches!(call.status, AnalysisStatus::Ready));
    fixture.assert_no_diagnostic(DiagnosticCode::GenericConstraintUnsatisfied);
}

#[test]
fn f_bound_refutation_is_reported_as_constraint_failure() {
    let fixture = Fixture::new(
        r#"
class Comparable<T> {}
class Other {}
class Probe {
  @class
  keep<T>(_ value: T) -> T where T <: Comparable<T> { value }

  @class
  run(_ value: Other) {
    let result = Probe.keep(value)
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.keep(value)");
    assert!(call.knowledge.ty().is_none());
    assert!(matches!(call.status, AnalysisStatus::Invalid(_)));
    fixture.assert_diagnostic(DiagnosticCode::GenericConstraintUnsatisfied, 1);
}

#[test]
fn expected_context_solves_nested_generic_result() {
    let fixture = Fixture::new(
        r#"
class Box<T> {
  @constructor new(_ value: T) {}
}

class Probe {
  @class
  make<T>() -> Box<T> { Box<Int>.new(1) }

  @class
  run() {
    let result: Box<Int> = Probe.make()
  }
}
"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Probe.make()");
    fixture.assert_type(call.knowledge.ty().expect("nested generic result"), applied("Box", [nominal("Int")]));
    assert_eq!(call.knowledge.status(), Some(EvidenceStatus::Assumed));
    assert!(matches!(call.status, AnalysisStatus::Ready), "{call:#?}");
}

#[test]
fn generic_constructor_composes_declaration_and_callable_domains() {
    let fixture = Fixture::new(
        r#"
class Box<T> {
  @constructor new<U>(_ value: T, _ metadata: U) {}
}

class Probe {
  @class
  run() {
    let result: Box<Int> = Box.new(1, "metadata")
  }
}

"#,
    );
    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    let call = fixture.expression(run, "Box.new(1, \"metadata\")");
    fixture.assert_type(call.knowledge.ty().expect("generic constructor result"), applied("Box", [nominal("Int")]));
    assert!(matches!(call.status, AnalysisStatus::Ready), "{call:#?}");

    let constructor = fixture
        .analysis
        .snapshot
        .callable_signatures
        .iter()
        .map(|(_, signature)| signature)
        .find(|signature| signature.owner.name.as_ref() == "Box" && signature.side == DispatchSide::Class)
        .expect("Box constructor signature");
    let generic_signature = constructor.generics.as_ref().expect("constructor-local generic signature");
    assert_eq!(generic_signature.parameter_count(), 1);
    assert!(matches!(
        generic_signature.owner,
        phalcom_semantic::types::parameter::TypeParameterOwner::Callable(_)
    ));
}

#[test]
fn complete_pack_generic_parameter_uses_an_ordinary_kind_aware_inference_variable() {
    use phalcom_modules::identity::ModuleId;
    use phalcom_semantic::checker::row_inference::{GenericApplicationSession, GenericInferenceBinding};
    use phalcom_semantic::identity::DeclarationId;

    let mut store = TypeStore::new();
    let owner = TypeParameterOwner::Declaration(DeclarationId::new(ModuleId::universe_root(), "Complete".into()));
    let parameter =
        store.intern_type_parameter(TypeParameterData::new(owner.clone(), 0, "P", KindId::COMPLETE_TYPE_PACK).with_mode(GenericParameterMode::CompletePack));
    let signature = GenericSignature::new(owner, Box::new([parameter])).with_parameter_modes(Box::new([GenericParameterMode::CompletePack]));
    let session = GenericApplicationSession::new(&signature, &store);

    assert!(matches!(session.parameter_bindings.get(&parameter), Some(GenericInferenceBinding::Type(_))));
}

#[test]
fn p3_generic_application_with_pack_parameters_solves_whole_pack() {
    use phalcom_modules::identity::ModuleId;
    use phalcom_semantic::checker::context::CheckerControl;
    use phalcom_semantic::checker::inference::{ConstraintOrigin, InferenceOutcome, InferenceRelation, InferenceTerm};
    use phalcom_semantic::checker::row_inference::{GenericApplicationSession, GenericInferenceBinding};
    use phalcom_semantic::identity::DeclarationId;
    use phalcom_semantic::types::relation::MapTypeHierarchy;

    let mut store = TypeStore::new();
    let hier = MapTypeHierarchy::new();
    let control = CheckerControl::default();

    let int_ty = store.nominal(DeclarationId::new(ModuleId::universe_root(), "Int".into()));
    let str_ty = store.nominal(DeclarationId::new(ModuleId::universe_root(), "String".into()));
    let concrete_p = store.type_pack(vec![int_ty, str_ty].into_boxed_slice()).unwrap();

    let owner = TypeParameterOwner::Declaration(DeclarationId::new(ModuleId::universe_root(), "BoxPack".into()));
    let param_p = store.intern_type_parameter(TypeParameterData::new(owner.clone(), 0, "P", KindId::TYPE_PACK).with_mode(GenericParameterMode::PositionalPack));
    let signature = GenericSignature::new(owner, Box::new([param_p])).with_parameter_modes(Box::new([GenericParameterMode::PositionalPack]));

    let mut session = GenericApplicationSession::new(&signature, &store);
    let var_p = match session.parameter_bindings.get(&param_p).unwrap() {
        GenericInferenceBinding::Type(v) => *v,
        _ => panic!("expected Type binding"),
    };

    session.types.add_constraint(
        InferenceRelation::Equivalent(InferenceTerm::Var(var_p), InferenceTerm::Canonical(concrete_p)),
        ConstraintOrigin::Explicit,
        None,
    );

    let res = session.types.propagate_with_control(&mut store, &hier, &control);
    assert!(res.is_ok());
    let outcome = session.types.finish_root();
    assert!(outcome.is_solved());
    if let InferenceOutcome::Solved(solution) = outcome {
        assert_eq!(session.types.solved_type_for(&solution, var_p), Some(concrete_p));
    }
}
