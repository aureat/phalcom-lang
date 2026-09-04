//! C8 branch-local existential escape and exact-case reconstruction laws.

use crate::semantic::support::Fixture;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::case_instantiation::CaseInstantiation;
use phalcom_semantic::types::rigid::RigidArena;
use phalcom_semantic::types::store::TypeData;
use phalcom_common::selector::{Selector, SelectorSlot};

fn direct_escape_source(branch: &str, return_type: &str) -> String {
    format!(
        r#"
enum Expr<T> {{
  @variant Pack<U>(_ value: U) -> Expr<U> where U <: Object
}}

class Eval {{
  @class
  eval<T>(_ value: Expr<T>) -> {return_type} {{
    match value {{
      Expr::Pack(x) => {branch}
    }}
  }}
}}
"#
    )
}

#[test]
fn direct_branch_result_rejects_local_existential_escape() {
    let fixture = Fixture::new(&direct_escape_source("x", "Int"));
    fixture.assert_diagnostic(DiagnosticCode::ExistentialEscape, 1);
}

#[test]
fn structural_wrapper_cannot_hide_local_existential_escape() {
    let fixture = Fixture::new(&direct_escape_source("[x]", "Int"));
    fixture.assert_diagnostic(DiagnosticCode::ExistentialEscape, 1);
}

#[test]
fn outer_assignment_rejects_local_existential_value() {
    let fixture = Fixture::new(
        r#"
enum Expr<T> {
  @variant Pack<U>(_ value: U) -> Expr<U> where U <: Object
}

class Eval {
  @class
  eval<T>(_ value: Expr<T>) {
    let out: Int = 0
    match value {
      Expr::Pack(x) => out = x
    }
  }
}
"#,
    );
    fixture.assert_diagnostic(DiagnosticCode::ExistentialEscape, 1);
}

#[test]
fn incompatible_call_argument_rejects_local_existential_value() {
    let fixture = Fixture::new(
        r#"
enum Expr<T> {
  @variant Pack<U>(_ value: U) -> Expr<U> where U <: Object
}

class Sink {
  @class
  consume(_ value: Int) -> Int { value }
}

class Eval {
  @class
  eval<T>(_ value: Expr<T>) {
    match value {
      Expr::Pack(x) => Sink.consume(x)
    }
  }
}
"#,
    );
    fixture.assert_diagnostic(DiagnosticCode::ExistentialEscape, 1);
}

#[test]
fn rigid_free_expected_supertype_allows_sound_widening() {
    let fixture = Fixture::new(&direct_escape_source("x", "Object"));
    fixture.assert_no_diagnostic(DiagnosticCode::ExistentialEscape);
}

#[test]
fn closure_capture_cannot_package_local_existential() {
    let fixture = Fixture::new(
        r#"
enum Expr<T> {
  @variant Pack<U>(_ value: U) -> Expr<U> where U <: Object
}

class Eval {
  @class
  eval<T>(_ value: Expr<T>) {
    match value {
      Expr::Pack(x) => || { x; 0 }
    }
  }
}
"#,
    );
    fixture.assert_diagnostic(DiagnosticCode::ExistentialEscape, 1);
}

#[test]
fn exact_case_elimination_reopens_hidden_locals_without_changing_canonical_case() {
    let fixture = Fixture::new(
        r#"
enum Expr<T> {
  @variant Pack<U>(_ value: U) -> Expr<U> where U <: Object
}

class Eval {
  @class
  eval() {
    let value = Expr::Pack(1)
    match value {
      Expr::Pack(_) => 0
    }
  }
}
"#,
    );
    fixture.assert_no_diagnostic(DiagnosticCode::ExistentialEscape);
    let eval = fixture.callable("Eval", "eval", DispatchSide::Class);
    let value = fixture.expression(eval, "Expr::Pack(1)");
    let exact_case = value.knowledge.ty().expect("constructor exact case");
    assert!(matches!(fixture.analysis.snapshot.store.get(exact_case), TypeData::ExactCase { .. }));
    let value_binding = fixture.binding(eval, "value");
    assert!(matches!(value_binding.current.ty().map(|ty| fixture.analysis.snapshot.store.get(ty)), Some(TypeData::ExactCase { .. })), "value binding: {value_binding:#?}");

    let resolution = eval.match_resolutions.values().next().expect("exact-case match resolution");
    let candidate = match &resolution.arms[0].pattern {
        phalcom_semantic::match_semantics::PatternResolution::Variant(pattern) => pattern
            .candidates
            .first()
            .unwrap_or_else(|| {
                let info = fixture.analysis.snapshot.enum_semantics.enum_info(&fixture.decl("Expr")).expect("Expr enum");
                panic!("Pack candidate missing: {pattern:#?}; variants: {:#?}", info.variants)
            }),
        other => panic!("expected exact-case variant pattern, got {other:?}"),
    };
    let opened = candidate.case_instantiation.as_ref().expect("exact-case elimination must open constructor locals");
    assert_eq!(opened.local_rigids.len(), 1);

    let variant_id = phalcom_semantic::identity::VariantId::new(
        fixture.decl("Expr"),
        Selector::method("Pack", [SelectorSlot::Positional]).expect("Pack selector"),
    );
    let variant = fixture.analysis.snapshot.enum_semantics.variant_info(&variant_id).expect("Pack variant");
    let mut arena = RigidArena::new();
    let first = CaseInstantiation::open(fixture.analysis.snapshot.store.as_ref(), &mut arena, variant, None);
    let second = CaseInstantiation::open(fixture.analysis.snapshot.store.as_ref(), &mut arena, variant, None);
    assert_eq!(first.local_rigids.len(), 1);
    assert_ne!(first.scope, second.scope);
    assert!(matches!(fixture.analysis.snapshot.store.get(exact_case), TypeData::ExactCase { .. }));
}
