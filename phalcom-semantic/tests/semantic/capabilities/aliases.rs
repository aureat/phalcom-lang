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

#[test]
fn variadic_alias_tuple_body_beta_reduces_through_pack_terms() {
    let fixture = Fixture::new(
        r#"
type AddHead<*Ts> = (Int, *Ts);
"#,
    );
    let alias = fixture.analysis.snapshot.type_aliases.get(&fixture.decl("AddHead")).expect("variadic alias");
    let TypeData::Lambda(lambda) = fixture.analysis.snapshot.store.get(alias.form) else {
        panic!("variadic alias must retain a lambda")
    };
    let lambda = fixture.analysis.snapshot.store.arena().get_lambda(*lambda);
    assert_eq!(
        lambda.parameter_modes.as_ref(),
        [phalcom_semantic::types::parameter::GenericParameterMode::PositionalPack]
    );

    let mut store = (*fixture.analysis.snapshot.store).clone();
    let pack = store.type_pack(vec![fixture.ty("String")].into_boxed_slice()).expect("concrete type pack");
    let applied = store.apply_type_form(alias.form, &[pack]).expect("pack alias application");
    let TypeData::Tuple(elements) = store.get(applied) else {
        panic!("concrete pack alias must reduce to the canonical Tuple")
    };
    assert_eq!(elements.elements.len(), 2);
    assert_eq!(elements.elements[0].ty, fixture.ty("Int"));
    assert_eq!(elements.elements[1].ty, fixture.ty("String"));
}

#[test]
fn variadic_aliases_forward_nominal_and_lambda_application_contracts() {
    for (name, source) in [
        (
            "forward nominal declaration",
            r#"
type Forward<*Ts> = Target<*Ts>;
class Target<*Ts> {}
"#,
        ),
        (
            "reverse nominal declaration",
            r#"
class Target<*Ts> {}
type Forward<*Ts> = Target<*Ts>;
"#,
        ),
        (
            "transparent alias lambda",
            r#"
type Base<*Ts> = (*Ts,);
type Forward<*Ts> = Base<*Ts>;
"#,
        ),
    ] {
        let fixture = Fixture::new(source);
        assert!(!fixture.analysis.snapshot.has_errors(), "{name}: {:?}", fixture.analysis.snapshot.diagnostics);
        let forward = fixture.analysis.snapshot.type_aliases.get(&fixture.decl("Forward")).expect("Forward alias");
        let TypeData::Lambda(lambda) = fixture.analysis.snapshot.store.get(forward.form) else {
            panic!("{name}: Forward must retain a lambda")
        };
        assert_eq!(
            fixture.analysis.snapshot.store.arena().get_lambda(*lambda).parameter_modes.as_ref(),
            [phalcom_semantic::types::parameter::GenericParameterMode::PositionalPack],
            "{name}: Forward must retain its positional pack binder"
        );
    }
}

#[test]
fn single_type_pack_parameter_is_not_a_variadic_binder() {
    let fixture = Fixture::new(
        r#"
class Single<P: TypePack> {}
type Forward<P: TypePack> = Single<P>;
"#,
    );
    assert!(!fixture.analysis.snapshot.has_errors(), "{:?}", fixture.analysis.snapshot.diagnostics);
    assert!(fixture.analysis.snapshot.type_aliases.get(&fixture.decl("Forward")).is_some());
}

#[test]
fn nominal_binder_header_does_not_require_alias_dependent_constraints() {
    let fixture = Fixture::new(
        r#"
type Bound = Object;
type Forward<T, *Ts> = Target<T, *Ts>;

class Target<T, *Ts>
where T <: Bound
{}
"#,
    );
    assert!(!fixture.analysis.snapshot.has_errors(), "{:?}", fixture.analysis.snapshot.diagnostics);
    assert!(fixture.analysis.snapshot.type_aliases.get(&fixture.decl("Forward")).is_some());
    assert!(
        fixture
            .analysis
            .snapshot
            .declarations
            .generic_signature(&fixture.decl("Target"))
            .is_some_and(|signature| !signature.constraints.is_empty())
    );
}

#[test]
fn variadic_callable_alias_lowers_and_beta_reduces() {
    let fixture = Fixture::new(
        r#"
type Fn<*Ps> = (*Ps,) -> Bool;
class Consumer<T> {}
type User = Consumer<Fn<*(Int, String)>>;
"#,
    );
    assert!(!fixture.analysis.snapshot.has_errors(), "{:?}", fixture.analysis.snapshot.diagnostics);
    let user_decl = fixture.decl("User");
    let user_alias = fixture.analysis.snapshot.type_aliases.get(&user_decl).expect("User alias");
    let store = &fixture.analysis.snapshot.store;
    let TypeData::Applied { origin: _, arguments } = store.get(user_alias.form) else {
        panic!("expected applied type, got {:?}", store.get(user_alias.form));
    };
    assert_eq!(arguments.len(), 1);
    let argument = arguments[0].as_type_id().expect("type-only alias argument");
    let TypeData::Callable(call) = store.get(argument) else {
        panic!("expected beta-reduced callable type, got {:?}", store.get(argument));
    };
    assert_eq!(call.parameters.len(), 2);
    let int_decl = fixture.decl("Int");
    let int_form = fixture.analysis.snapshot.declarations.form(&int_decl).unwrap();
    let string_decl = fixture.decl("String");
    let string_form = fixture.analysis.snapshot.declarations.form(&string_decl).unwrap();
    assert_eq!(call.parameters[0].ty, int_form);
    assert_eq!(call.parameters[1].ty, string_form);
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
