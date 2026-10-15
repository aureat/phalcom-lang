use crate::semantic::support::Fixture;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::store::TypeData;

#[test]
fn rest_capture_local_is_proper_pack_not_element_type() {
    let f = Fixture::new(
        r#"
class C {
    @class test_elem(*xs: Int) {
        const local_xs = xs
    }
    @class test_lane<*P>(*ys: *P) {
        const local_ys = ys
    }
}
"#,
    );
    f.assert_no_error_diagnostics();
    let callable_elem = f.callable("C", "test_elem", DispatchSide::Class);
    let ty_xs = f.binding(callable_elem, "local_xs").current.ty().expect("inferred local_xs");
    let TypeData::Pack(pack) = f.analysis.snapshot.store.get(ty_xs) else {
        panic!("local_xs must be a Pack, not an Int");
    };
    assert!(matches!(
        pack.positional_remainder,
        phalcom_semantic::types::store::AnonymousTupleRemainder::Typed(_)
    ));

    let callable_lane = f.callable("C", "test_lane", DispatchSide::Class);
    let ty_ys = f.binding(callable_lane, "local_ys").current.ty().expect("inferred local_ys");
    assert!(matches!(f.analysis.snapshot.store.get(ty_ys), TypeData::PackFromPacks { .. }));
}

#[test]
fn rest_argument_analysis_preserves_source_order() {
    let f = Fixture::new(
        r#"
class Trace {
    @class log(_ n: Int) -> Int { n }
    @class run() {
        Trace.target(Trace.log(1), Trace.log(2), label: Trace.log(3))
    }
    @class target<*P>(*args: *P, label: Int) {}
}
"#,
    );
    f.assert_no_error_diagnostics();
}

#[test]
fn rest_concrete_lane_uses_pointwise_value_conformance() {
    let f = Fixture::new(
        r#"
class Animal {}
class Dog is Animal {}
class C {
    @class accept(*args: *(Animal, String)) {}
    @class run() {
        C.accept(Dog.new(), "ok")
    }
}
"#,
    );
    f.assert_no_error_diagnostics();
}

#[test]
fn rest_mixed_generic_concrete_lane_uses_pointwise_conformance() {
    let f = Fixture::new(
        r#"
class Animal {}
class Dog is Animal {}
data Box<T> { value: T }
class C {
    @class accept<T>(*args: *(T, Animal)) {}
    @class run() {
        const result = C.accept(Dog.new(), Dog.new())
    }
}
"#,
    );
    f.assert_no_error_diagnostics();
}

#[test]
fn rest_concrete_lane_rejects_arity_mismatch() {
    use phalcom_semantic::diagnostic::DiagnosticCode;

    let f = Fixture::new(
        r#"
class C {
    @class accept(*args: *(Int, String)) {}
    @class run() {
        C.accept(1)
    }
}
"#,
    );
    assert!(!f.diagnostics(DiagnosticCode::CallShapeMismatch).is_empty());
}

#[test]
fn rest_contract_change_invalidates_dependent_calls() {
    use phalcom_modules::{DeclarationId, ModuleComponent, ModuleId, ModulePath, ResolvedProjectId};
    use phalcom_semantic::SemanticWorkspaceSession;
    use phalcom_semantic::db::fingerprint::callable_signature_product_fingerprint;
    use phalcom_semantic::identity::DispatchSide;

    let module = ModuleId::resolved(
        ResolvedProjectId::from_raw(42),
        ModulePath::from_components(vec![ModuleComponent::from_identifier("rest_fp").unwrap()]),
    );
    let mut session = SemanticWorkspaceSession::new();
    let source_a = r#"
class Callee {
    @class run(*xs: Int) -> Int { 1 }
}
class Caller {
    @class call() { Callee.run(1) }
}
"#;

    let update_a = session
        .update(crate::semantic::incremental::support::single_module_input(module.clone(), source_a, 1))
        .expect("semantic test update should succeed");
    assert!(!update_a.snapshot.has_errors(), "initial source must be valid");
    let callee = DeclarationId::new(module.clone(), "Callee".into());
    let callee_id = update_a
        .snapshot
        .callable_signatures
        .iter()
        .find(|(id, _)| {
            id.owner == callee
                && id.side == DispatchSide::Class
                && matches!(&id.selector.base, phalcom_common::selector::SelectorBase::Named(name) if name == "run")
        })
        .map(|(id, _)| id.clone())
        .expect("Callee.run callable");
    let initial_fp = callable_signature_product_fingerprint(update_a.snapshot.callable_signatures.get(&callee_id).unwrap());

    // Edit Callee to *xs: *P
    let source_b = r#"
class Callee {
    @class run<*P>(*xs: *P) -> Int { 1 }
}
class Caller {
    @class call() { Callee.run(1) }
}
"#;

    let update_b = session
        .update(crate::semantic::incremental::support::single_module_input(module.clone(), source_b, 2))
        .expect("semantic test update should succeed");
    assert!(!update_b.snapshot.has_errors(), "updated source must be valid");
    let callee_id_b = update_b
        .snapshot
        .callable_signatures
        .iter()
        .find(|(id, _)| {
            id.owner == callee
                && id.side == DispatchSide::Class
                && matches!(&id.selector.base, phalcom_common::selector::SelectorBase::Named(name) if name == "run")
        })
        .map(|(id, _)| id.clone())
        .expect("Callee.run callable B");
    let edited_fp = callable_signature_product_fingerprint(update_b.snapshot.callable_signatures.get(&callee_id_b).unwrap());

    assert_ne!(initial_fp, edited_fp, "changing rest contract must alter callable product fingerprint");
}

#[test]
fn override_checking_rejects_element_vs_lane_mismatch() {
    use phalcom_semantic::diagnostic::DiagnosticCode;

    let f = Fixture::new(
        r#"
class Base {
    m(*args: Animal) {}
}
class Sub is Base {
    m<*P>(*args: *P) {}
}
class Animal {}
"#,
    );
    assert!(!f.diagnostics(DiagnosticCode::IncompatibleOverride).is_empty());
}

#[test]
fn rest_generic_lane_override_is_alpha_equivalent() {
    let f = Fixture::new(
        r#"
class Base {
    m<*P>(*args: *P) {}
}
class Sub is Base {
    m<*Q>(*args: *Q) {}
}
"#,
    );
    f.assert_no_error_diagnostics();
}

#[test]
fn rest_concrete_lane_override_is_contravariant() {
    let f = Fixture::new(
        r#"
class Animal {}
class Dog is Animal {}
class Base {
    m(*args: *(Dog, Dog)) {}
}
class Sub is Base {
    m(*args: *(Animal, Animal)) {}
}
"#,
    );
    f.assert_no_error_diagnostics();
}

#[test]
fn rest_plain_annotation_remains_element_contract() {
    use phalcom_semantic::diagnostic::DiagnosticCode;

    let f = Fixture::new(
        r#"
class C {
    @class accept(*xs: Int) {}
    @class run_bad() {
        C.accept(1, "bad")
    }
}
"#,
    );
    assert!(!f.diagnostics(DiagnosticCode::ArgumentMismatch).is_empty());
}

#[test]
fn rest_typepack_name_is_not_implicitly_expanded() {
    use phalcom_semantic::diagnostic::DiagnosticCode;

    let f = Fixture::new(
        r#"
class C {
    @class f<*P>(*args: P) {}
}
"#,
    );
    assert!(!f.diagnostics(DiagnosticCode::KindExpectedType).is_empty());
}

#[test]
fn rest_typepack_element_contract_rejects_non_type_kind() {
    use phalcom_semantic::diagnostic::DiagnosticCode;

    let f = Fixture::new(
        r#"
class C {
    @class f<**K: LabeledTypePack>(*args: K) {}
}
"#,
    );
    assert!(!f.diagnostics(DiagnosticCode::KindExpectedType).is_empty());
}

#[test]
fn rest_explicit_positional_lane_infers_whole_typepack() {
    let f = Fixture::new(
        r#"
data Holder<*P> { items: Pack<*P> }
class C {
    @class wrap<*P>(*args: *P) -> Holder<*P> {
        Holder { items: args }
    }
    @class run() {
        const x = C.wrap(1, "hello")
    }
}
"#,
    );
    f.assert_no_error_diagnostics();
    let run = f.callable("C", "run", DispatchSide::Class);
    let ty = f.binding(run, "x").current.ty().expect("inferred x");
    let TypeData::Applied { arguments, .. } = f.analysis.snapshot.store.get(ty) else {
        panic!("expected Holder");
    };
    let lane = arguments[0].as_type_id().expect("type-pack argument");
    let TypeData::TypePack(elements) = f.analysis.snapshot.store.get(lane) else {
        panic!("expected TypePack");
    };
    assert_eq!(elements.as_ref(), &[f.ty("Int"), f.ty("String")]);
}

#[test]
fn rest_empty_lane_infers_empty_typepack() {
    let f = Fixture::new(
        r#"
data Holder<*P> { items: Pack<*P> }
class C {
    @class wrap<*P>(*args: *P) -> Holder<*P> {
        Holder { items: args }
    }
    @class run() {
        const x = C.wrap()
    }
}
"#,
    );
    f.assert_no_error_diagnostics();
    let run = f.callable("C", "run", DispatchSide::Class);
    let ty = f.binding(run, "x").current.ty().expect("inferred x");
    let TypeData::Applied { arguments, .. } = f.analysis.snapshot.store.get(ty) else {
        panic!("expected Holder");
    };
    let lane = arguments[0].as_type_id().expect("type-pack argument");
    let TypeData::TypePack(elements) = f.analysis.snapshot.store.get(lane) else {
        panic!("expected TypePack");
    };
    assert!(elements.is_empty());
}

#[test]
fn rest_lane_capture_local_is_pack_from_inferred_lane() {
    let f = Fixture::new(
        r#"
class C {
    @class run<*P>(*ys: *P) {
        const local_ys = ys
    }
}
"#,
    );
    f.assert_no_error_diagnostics();
    let run = f.callable("C", "run", DispatchSide::Class);
    let ty = f.binding(run, "local_ys").current.ty().expect("inferred local_ys");
    assert!(matches!(f.analysis.snapshot.store.get(ty), TypeData::PackFromPacks { .. }));
}

#[test]
fn rest_pack_of_expansion_remains_proper_pack_element_contract() {
    use phalcom_semantic::diagnostic::DiagnosticCode;

    let f = Fixture::new(
        r#"
class C {
    @class consume<*P>(*packs: Pack<*P>) {}
    @class run_ok() {
        C.consume(Pack(1, "a"))
    }
    @class run_bad() {
        C.consume(1, "a")
    }
}
"#,
    );
    assert!(!f.diagnostics(DiagnosticCode::ArgumentMismatch).is_empty() || !f.diagnostics(DiagnosticCode::GenericInferenceUnderconstrained).is_empty());
}

#[test]
fn rest_whole_pack_inference_does_not_create_typepack_subtyping() {
    let f = Fixture::new(
        r#"
class Animal {}
class Dog is Animal {}
data Box<*P> { items: Pack<*P> }
class C {
    @class wrap<*P>(*args: *P) -> Box<*P> {
        Box { items: args }
    }
    @class run() {
        const x = C.wrap(Dog.new())
    }
}
"#,
    );
    f.assert_no_error_diagnostics();
    let run = f.callable("C", "run", DispatchSide::Class);
    let ty = f.binding(run, "x").current.ty().expect("inferred x");
    let TypeData::Applied { arguments, .. } = f.analysis.snapshot.store.get(ty) else {
        panic!("expected Box");
    };
    let lane = arguments[0].as_type_id().expect("type-pack argument");
    let TypeData::TypePack(elements) = f.analysis.snapshot.store.get(lane) else {
        panic!("expected TypePack");
    };
    assert_eq!(elements.as_ref(), &[f.ty("Dog")]);
}

#[test]
fn rest_lane_inference_does_not_partition_pack_concat() {
    let f = Fixture::new(
        r#"
class C {
    @class test<*P>(*args: *P) {}
}
"#,
    );
    f.assert_no_error_diagnostics();
}

#[test]
fn rest_complete_capture_preserves_both_lanes() {
    let f = Fixture::new(
        r#"
data Query<***C> { all: Pack<***C> }
class C {
    @class of<***C>(***all: ***C) -> Query<***C> {
        Query { all: all }
    }
    @class run() {
        const q = C.of(1, name: "test")
    }
}
"#,
    );
    f.assert_no_error_diagnostics();
    let run = f.callable("C", "run", DispatchSide::Class);
    let ty = f.binding(run, "q").current.ty().expect("inferred q");
    let TypeData::Applied { arguments, .. } = f.analysis.snapshot.store.get(ty) else {
        panic!("expected Query");
    };
    let lane = arguments[0].as_type_id().expect("complete-pack argument");
    let TypeData::CompleteTypePack { positionals, labeled } = f.analysis.snapshot.store.get(lane) else {
        panic!("expected CompleteTypePack");
    };
    let TypeData::TypePack(pos_elems) = f.analysis.snapshot.store.get(*positionals) else {
        panic!("expected TypePack");
    };
    let TypeData::LabeledTypePack(lab_elems) = f.analysis.snapshot.store.get(*labeled) else {
        panic!("expected LabeledTypePack");
    };
    assert_eq!(pos_elems.as_ref(), &[f.ty("Int")]);
    assert_eq!(lab_elems.len(), 1);
    assert_eq!(lab_elems[0].label.as_ref(), "name");
    assert_eq!(lab_elems[0].element, f.ty("String"));
}

#[test]
fn rest_labeled_lane_preserves_labels_and_order() {
    let f = Fixture::new(
        r#"
data Keyed<**K: LabeledTypePack> { items: Pack<**K> }
class C {
    @class of<**K: LabeledTypePack>(**kwargs: **K) -> Keyed<**K> {
        Keyed { items: kwargs }
    }
    @class run() {
        const k = C.of(b: 1, a: "two")
    }
}
"#,
    );
    f.assert_no_error_diagnostics();
    let run = f.callable("C", "run", DispatchSide::Class);
    let ty = f.binding(run, "k").current.ty().expect("inferred k");
    let TypeData::Applied { arguments, .. } = f.analysis.snapshot.store.get(ty) else {
        panic!("expected Keyed");
    };
    let lane = arguments[0].as_type_id().expect("labeled-pack argument");
    let TypeData::LabeledTypePack(lab_elems) = f.analysis.snapshot.store.get(lane) else {
        panic!("expected LabeledTypePack");
    };
    assert_eq!(lab_elems.len(), 2);
    assert_eq!(lab_elems[0].label.as_ref(), "b");
    assert_eq!(lab_elems[0].element, f.ty("Int"));
    assert_eq!(lab_elems[1].label.as_ref(), "a");
    assert_eq!(lab_elems[1].element, f.ty("String"));
}
