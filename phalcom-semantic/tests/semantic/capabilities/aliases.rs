//! Source-level transparent alias and type-lambda publication laws.

use crate::semantic::support::Fixture;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::DispatchSide;
use phalcom_semantic::types::store::TypeData;

#[test]
fn source_aliases_preserve_canonical_forms_and_generic_inference() {
    let fixture = Fixture::new(
        r#"
type UserId = Int;
type BoxOf<T> = Box<T>;
type SameBox<U> = Box<U>;
type Nested<U> = BoxOf<Box<U>>;
type Constructor = <T> =>> Box<T>;

class Box<T> {}

class Probe {
  @class
  identity<A>(_ value: A) -> A { value }

  @class
  use(_ value: BoxOf<Int>) -> BoxOf<Int> { value }

  @class
  build(_ value: Constructor<Int>) -> Box<Int> { value }

  @class
  run(_ value: UserId) {
    let result = Probe.identity(value)
  }
}
"#,
    );

    let user_id = fixture.analysis.snapshot.type_aliases.get(&fixture.decl("UserId")).expect("UserId alias");
    assert_eq!(user_id.form, fixture.ty("Int"));
    assert_eq!(fixture.analysis.snapshot.store.format_type(user_id.form), "Int");

    for name in ["BoxOf", "Nested", "Constructor"] {
        let alias = fixture
            .analysis
            .snapshot
            .type_aliases
            .get(&fixture.decl(name))
            .expect("generic/type-lambda alias");
        assert!(
            matches!(fixture.analysis.snapshot.store.get(alias.form), TypeData::Lambda(_)),
            "{name} must retain a canonical lambda form"
        );
        assert_ne!(alias.kind, phalcom_semantic::types::id::KindId::TYPE, "{name} must retain constructor kind");
    }
    let nested = fixture.analysis.snapshot.type_aliases.get(&fixture.decl("Nested")).expect("Nested alias");
    assert!(nested.dependencies.iter().any(|dependency| dependency.name.as_ref() == "BoxOf"));
    let same_box = fixture
        .analysis
        .snapshot
        .type_aliases
        .get(&fixture.decl("SameBox"))
        .expect("alpha-equivalent alias");
    let box_of = fixture.analysis.snapshot.type_aliases.get(&fixture.decl("BoxOf")).expect("generic alias");
    assert_eq!(box_of.form, same_box.form, "source aliases with renamed binders must alpha-normalize");

    let use_callable = fixture.callable("Probe", "use", DispatchSide::Class);
    let use_signature = fixture
        .analysis
        .snapshot
        .callable_signatures
        .get(&use_callable.callable)
        .expect("use signature");
    assert_eq!(
        use_signature.parameter_declared_type_at(0).and_then(|state| state.canonical_type()),
        Some(applied_type(&fixture, "Box", &[fixture.ty("Int")]))
    );
    assert_eq!(
        use_signature.declared_return.canonical_type(),
        Some(applied_type(&fixture, "Box", &[fixture.ty("Int")]))
    );

    let construct = fixture.callable("Probe", "build", DispatchSide::Class);
    let construct_signature = fixture
        .analysis
        .snapshot
        .callable_signatures
        .get(&construct.callable)
        .expect("construct signature");
    assert_eq!(
        construct_signature.parameter_declared_type_at(0).and_then(|state| state.canonical_type()),
        Some(applied_type(&fixture, "Box", &[fixture.ty("Int")]))
    );

    let run = fixture.callable("Probe", "run", DispatchSide::Class);
    fixture.assert_binding_type(run, "result", fixture.ty("Int"));
}

#[test]
fn source_alias_cycle_and_malformed_target_are_quarantined() {
    let fixture = Fixture::new_allowing_internal_incidents(
        r#"
type Broken = Missing;
type First<T> = Second<T>;
type Second<T> = First<T>;
"#,
    );
    assert!(
        fixture
            .diagnostics(DiagnosticCode::AnnotationUnresolved)
            .iter()
            .any(|diagnostic| diagnostic.message.contains("Missing"))
    );
    assert!(!fixture.diagnostics(DiagnosticCode::TypeAliasCycle).is_empty());
    assert!(fixture.analysis.snapshot.type_aliases.iter().next().is_none());
    assert!(fixture.analysis.snapshot.declarations.get(&fixture.decl("Broken")).is_none());
    assert!(fixture.analysis.snapshot.declarations.get(&fixture.decl("First")).is_none());
    assert!(fixture.analysis.snapshot.declarations.get(&fixture.decl("Second")).is_none());
}

fn applied_type(fixture: &Fixture, name: &str, arguments: &[phalcom_semantic::TypeId]) -> phalcom_semantic::TypeId {
    let form = fixture
        .analysis
        .snapshot
        .declarations
        .form(&fixture.decl(name))
        .or_else(|| fixture.analysis.snapshot.type_aliases.form(&fixture.decl(name)))
        .expect("type form");
    let mut store = (*fixture.analysis.snapshot.store).clone();
    store.apply_type_form(form, arguments).expect("well-kinded application")
}
