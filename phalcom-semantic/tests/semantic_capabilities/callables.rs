use crate::support::Fixture;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;

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
    for normal in &factory.exits.normal_return_values {
        if let Some(ty) = normal.ty() {
            f.assert_union_members(ty, &[cat, dog]);
        }
    }
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let x = f.binding(run, "x");
    f.assert_union_members(x.current.ty().expect("published inferred return"), &[cat, dog]);
}

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
}
