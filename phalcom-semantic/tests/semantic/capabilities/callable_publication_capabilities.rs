use crate::semantic::support::{Fixture, known, union};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;

/// LAW: unannotated callable publication preserves its normal branch return union.
#[test]
fn branch_derived_tail_type_is_published_to_unannotated_callable_signature() {
    let f = Fixture::new(
        r#"
class Animal {}
class Cat is Animal { @constructor new() {} }
class Dog is Animal { @constructor new() {} }
class Factory {
  @class
  make(_ flag: Bool) {
    if flag { Cat.new() } else { Dog.new() }
  }
}
class Probe {
  @class
  run(_ flag: Bool) {
    let x = Factory.make(flag)
  }
}
"#,
    );
    let cat = f.ty("Cat");
    let dog = f.ty("Dog");
    let factory = f.callable("Factory", "make", DispatchSide::Class);
    for normal in &factory.exits.normal_returns {
        if let Some(ty) = normal.knowledge.ty() {
            f.assert_union_members(ty, &[cat, dog]);
        }
    }
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let x = f.binding(run, "x");
    f.assert_union_members(x.current.ty().expect("published inferred return"), &[cat, dog]);
    f.assert_normal_return(
        factory,
        known(union([cat.into(), dog.into()]))
            .established()
            .origin(phalcom_semantic::EvidenceOrigin::Flow),
    );
    f.assert_callable_dependency(run, &f.callable_id("Factory", "make", DispatchSide::Class));
    f.assert_no_error_diagnostics();
}

/// LAW: broad public return contract accepts narrower branch evidence.
#[test]
fn explicit_broad_return_contract_preserves_narrow_branch_evidence() {
    let f = Fixture::new(
        r#"
class Animal {}
class Cat is Animal { @constructor new() {} }
class Dog is Animal { @constructor new() {} }
class Factory {
  @class
  make(_ flag: Bool) -> Animal {
    if flag { Cat.new() } else { Dog.new() }
  }
}
"#,
    );
    let animal = f.ty("Animal");
    let cat = f.ty("Cat");
    let dog = f.ty("Dog");
    let make_id = f.callable_id("Factory", "make", DispatchSide::Class);
    let surface = f.analysis.snapshot.surfaces.get(&f.decl("Factory")).expect("Factory surface");
    let signature = surface.get_callable(DispatchSide::Class, &make_id.selector).expect("Factory.make signature");
    assert_eq!(signature.return_type.ty(), Some(animal), "public return contract must remain Animal");
    let make = f.callable("Factory", "make", DispatchSide::Class);
    f.assert_expression_established(f.expression(make, "Cat.new()"), cat);
    f.assert_expression_established(f.expression(make, "Dog.new()"), dog);
    f.assert_no_diagnostic(DiagnosticCode::ReturnMismatch);
    f.assert_normal_return(
        make,
        known(union([cat.into(), dog.into()]))
            .established()
            .origin(phalcom_semantic::EvidenceOrigin::Flow),
    );
    f.assert_no_error_diagnostics();
}

#[test]
fn one_bad_return_branch_is_refuted_without_rewriting_branch_fact() {
    let f = Fixture::new(
        r#"
class Animal {}
class Cat is Animal { @constructor new() {} }
class Factory {
  @class
  make(_ flag: Bool) -> Animal {
    if flag { Cat.new() } else { "bad" }
  }
}
"#,
    );
    let cat = f.ty("Cat");
    let string_ty = f.ty("String");
    let make = f.callable("Factory", "make", DispatchSide::Class);
    f.assert_expression_established(f.expression(make, "Cat.new()"), cat);
    f.assert_expression_established(f.expression(make, "\"bad\""), string_ty);
    assert_eq!(f.diagnostics(DiagnosticCode::ReturnMismatch).len(), 1);
}

#[test]
fn recursive_inference_fails_honestly_without_inventing_unit_or_nominal_type() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  f() { g() }

  @class
  g() { f() }
}
"#,
    );
    let f_analysis = f.callable("Probe", "f", DispatchSide::Class);
    let g_analysis = f.callable("Probe", "g", DispatchSide::Class);
    let g_call = f.expression(f_analysis, "g()");
    let f_call = f.expression(g_analysis, "f()");
    assert!(
        g_call.knowledge.is_unknown() || g_call.knowledge.is_dynamic(),
        "recursive call must remain epistemically incomplete: {g_call:#?}"
    );
    assert!(
        f_call.knowledge.is_unknown() || f_call.knowledge.is_dynamic(),
        "recursive call must remain epistemically incomplete: {f_call:#?}"
    );
    let cycle_diagnostics = f.diagnostics(DiagnosticCode::CallableResultCycle);
    assert_eq!(
        cycle_diagnostics.len(),
        1,
        "one diagnostic should represent the mutual result SCC: {cycle_diagnostics:#?}"
    );
    assert_eq!(
        cycle_diagnostics[0].primary_range, f_analysis.body_range,
        "the diagnostic anchor should be the earliest source member"
    );
    assert_eq!(
        cycle_diagnostics[0].labels.len(),
        1,
        "the other SCC member should be related to the source-first anchor"
    );
    assert_eq!(
        f_analysis.exits.normal_returns[0].direct_result_dependency.as_ref(),
        Some(&f.callable_id("Probe", "g", DispatchSide::Class))
    );
    assert_eq!(
        g_analysis.exits.normal_returns[0].direct_result_dependency.as_ref(),
        Some(&f.callable_id("Probe", "f", DispatchSide::Class))
    );
}

#[test]
fn direct_unseeded_result_cycle_is_reported_deterministically_without_fabricating_a_type() {
    const SOURCE: &str = r#"
class Probe {
  @class
  loop() { Probe.loop() }
}
"#;
    let first = Fixture::new(SOURCE);
    let loop_analysis = first.callable("Probe", "loop", DispatchSide::Class);
    assert_eq!(
        loop_analysis.exits.normal_returns[0].direct_result_dependency.as_ref(),
        Some(&first.callable_id("Probe", "loop", DispatchSide::Class))
    );
    assert!(
        first
            .analysis
            .snapshot
            .callable_signatures
            .get(&first.callable_id("Probe", "loop", DispatchSide::Class))
            .expect("loop signature")
            .published_return_knowledge()
            .is_unknown()
    );
    let first_diagnostics = first.diagnostics(DiagnosticCode::CallableResultCycle);
    assert_eq!(first_diagnostics.len(), 1);
    assert_eq!(first_diagnostics[0].primary_range, loop_analysis.body_range);

    let second = Fixture::new(SOURCE);
    let second_diagnostics = second.diagnostics(DiagnosticCode::CallableResultCycle);
    assert_eq!(second_diagnostics.len(), 1);
    assert_eq!(second_diagnostics[0].code, first_diagnostics[0].code);
    assert_eq!(second_diagnostics[0].message, first_diagnostics[0].message);
    assert_eq!(second_diagnostics[0].primary_range, first_diagnostics[0].primary_range);
    assert_eq!(second_diagnostics[0].labels, first_diagnostics[0].labels);
}

#[test]
fn non_result_recursive_calls_do_not_form_callable_result_cycles() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  selfCall() { Probe.selfCall(); 1 }

  @class
  left() { Probe.right(); 1 }

  @class
  right() { Probe.left(); 2 }

  @class
  localCall() {
    let value = Probe.localCall()
    value
  }
}
"#,
    );
    let int_ty = f.ty("Int");
    for name in ["selfCall", "left", "right"] {
        let signature = f
            .analysis
            .snapshot
            .callable_signatures
            .get(&f.callable_id("Probe", name, DispatchSide::Class))
            .expect("callable signature");
        assert_eq!(
            signature.published_return_knowledge().ty(),
            Some(int_ty),
            "{name} should publish its independent literal result"
        );
    }
    let local_signature = f
        .analysis
        .snapshot
        .callable_signatures
        .get(&f.callable_id("Probe", "localCall", DispatchSide::Class))
        .expect("local call signature");
    assert!(
        local_signature.published_return_knowledge().is_unknown(),
        "a returned local must not be mistaken for a direct result edge"
    );
    let local_analysis = f.callable("Probe", "localCall", DispatchSide::Class);
    assert_eq!(local_analysis.exits.normal_returns[0].direct_result_dependency.as_ref(), None);
    assert!(f.diagnostics(DiagnosticCode::CallableResultCycle).is_empty());
}

#[test]
fn explicit_mutual_recursive_publication_uses_declared_results() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  f() -> Int { g() }

  @class
  g() -> Int { f() }
}
"#,
    );
    let int_ty = f.ty("Int");
    let f_id = f.callable_id("Probe", "f", DispatchSide::Class);
    let g_id = f.callable_id("Probe", "g", DispatchSide::Class);
    for callable_id in [&f_id, &g_id] {
        let signature = f
            .analysis
            .snapshot
            .callable_signatures
            .get(callable_id)
            .expect("explicit mutual-recursion signature");
        assert_eq!(signature.declared_return.canonical_type(), Some(int_ty));
        assert_eq!(signature.published_return_knowledge().ty(), Some(int_ty));
    }

    let f_body = f.callable("Probe", "f", DispatchSide::Class);
    let g_body = f.callable("Probe", "g", DispatchSide::Class);
    for (caller, source) in [(f_body, "g()"), (g_body, "f()")] {
        let call = f.expression(caller, source);
        assert_eq!(call.knowledge.ty(), Some(int_ty), "mutual recursive call `{source}`: {call:#?}");
    }
    assert!(
        f.diagnostics(DiagnosticCode::CallableResultCycle).is_empty(),
        "declared results are authoritative seeds"
    );
    f.assert_no_error_diagnostics();
}
