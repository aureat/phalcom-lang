use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::LinkedModuleInterface;
use phalcom_modules::linker::{LinkedModule, LinkedProgram, ModuleBindingLayout};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::source::ModuleKind;
use phalcom_semantic::db::QueryKey;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::source::ParsedModuleUnit;
use phalcom_semantic::types::parameter::{GenericConstraint, TypeParameterOwner, TypeTerm};
use phalcom_semantic::types::store::TypeData;
use phalcom_semantic::workspace::SemanticWorkspaceInput;
use std::collections::BTreeMap;
use std::sync::Arc;

fn test_module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(42), ModulePath::root())
}

fn single_module_input(module: ModuleId, source: &str) -> SemanticWorkspaceInput {
    let parsed = phalcom_ast::parse(source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);

    let linked_module = LinkedModule {
        interface: LinkedModuleInterface {
            module: module.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::new(),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout::default(),
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };
    let mut modules = BTreeMap::new();
    modules.insert(module.clone(), linked_module);
    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(phalcom_modules::project::ProjectUniverse::new()),
        modules,
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: module.clone(),
        initialization_order: vec![module.clone()],
    });

    let mut sources = BTreeMap::new();
    sources.insert(
        module.clone(),
        Arc::new(ParsedModuleUnit::new(
            module,
            ModuleKind::Module,
            None,
            Arc::from(source),
            Arc::new(parsed.program),
        )),
    );
    SemanticWorkspaceInput::new(linked, sources, 1)
}

#[test]
fn test_instance_self_and_private_field_access_in_impl_body() {
    let module = test_module();
    let src = r#"
class User {
  _name: String

  getName() -> String {
    self._name
  }
}

impl User {
  getNameImpl() -> String {
    self._name
  }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), src));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let callable = CallableId::new(
        DeclarationId::new(module, "User".into()),
        Selector::method("getNameImpl", []).unwrap(),
        DispatchSide::Instance,
    );
    let analysis = update.snapshot.callable_analyses.get(&callable).expect("body analysis should be present");
    assert!(analysis.diagnostics.is_empty());
}

#[test]
fn test_class_side_impl_body_and_self() {
    let module = test_module();
    let src = r#"
class Counter {
  @class
  getCountDirect() -> Int {
    10
  }
}

impl Counter {
  @class
  getCount() -> Int {
    Self.getCountDirect()
  }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), src));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let callable = CallableId::new(
        DeclarationId::new(module, "Counter".into()),
        Selector::method("getCount", []).unwrap(),
        DispatchSide::Class,
    );
    let analysis = update.snapshot.callable_analyses.get(&callable).expect("body analysis should be present");
    assert!(analysis.diagnostics.is_empty());
}

#[test]
fn test_generic_covering_permutation_body_analysis() {
    let module = test_module();
    let src = r#"
class Pair<X, Y> {
  _first: X
  _second: Y

  first() -> X { self._first }
  second() -> Y { self._second }
}

impl<A, B> Pair<B, A> {
  getFirst() -> B {
    self.first()
  }
  getSecond() -> A {
    self.second()
  }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), src));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let pair_decl = DeclarationId::new(module, "Pair".into());
    let get_first = CallableId::new(pair_decl.clone(), Selector::method("getFirst", []).unwrap(), DispatchSide::Instance);
    let get_second = CallableId::new(pair_decl, Selector::method("getSecond", []).unwrap(), DispatchSide::Instance);

    let analysis1 = update.snapshot.callable_analyses.get(&get_first).expect("getFirst body analysis");
    assert!(analysis1.diagnostics.is_empty());
    let analysis2 = update.snapshot.callable_analyses.get(&get_second).expect("getSecond body analysis");
    assert!(analysis2.diagnostics.is_empty());
}

#[test]
fn test_covering_class_impl_body_uses_impl_generic_coordinates() {
    let module = test_module();
    let src = r#"
data Token()

data Id<Tag>(
  _ token: Token,
)

impl<Tag> Id<Tag> {
  @class
  fresh() -> Id<Tag> {
    Id<Tag>(Token())
  }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), src));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let callable = CallableId::new(
        DeclarationId::new(module, "Id".into()),
        Selector::method("fresh", []).unwrap(),
        DispatchSide::Class,
    );
    let analysis = update.snapshot.callable_analyses.get(&callable).expect("fresh body analysis");
    assert!(analysis.diagnostics.is_empty());
}

#[test]
fn test_covering_class_impl_body_specializes_bare_self() {
    let module = test_module();
    let src = r#"
data Token()

data Id<Tag>(
  _ token: Token,
)

impl<Tag> Id<Tag> {
  @class
  fresh() -> Self {
    Id<Tag>(Token())
  }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), src));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let callable = CallableId::new(
        DeclarationId::new(module, "Id".into()),
        Selector::method("fresh", []).unwrap(),
        DispatchSide::Class,
    );
    let analysis = update.snapshot.callable_analyses.get(&callable).expect("fresh body analysis");
    assert!(analysis.diagnostics.is_empty());
}

#[test]
fn test_covering_permutation_body_specializes_parameters_and_return() {
    let module = test_module();
    let src = r#"
data Pair<X, Y>(
  _ first: X,
  _ second: Y,
)

impl<A, B> Pair<B, A> {
  @class
  duplicate(_ b: B, _ a: A) -> Pair<B, A> {
    Pair<B, A>(b, a)
  }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), src));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let callable = CallableId::new(
        DeclarationId::new(module, "Pair".into()),
        Selector::method(
            "duplicate",
            [
                phalcom_common::selector::SelectorSlot::Positional,
                phalcom_common::selector::SelectorSlot::Positional,
            ],
        )
        .unwrap(),
        DispatchSide::Class,
    );
    let analysis = update.snapshot.callable_analyses.get(&callable).expect("duplicate body analysis");
    assert!(analysis.diagnostics.is_empty());
}

#[test]
fn test_covering_body_specializes_callable_generic_self_constraint() {
    let module = test_module();
    let src = r#"
data Pair<X, Y>(
  _ first: X,
  _ second: Y,
)

impl<A, B> Pair<B, A> {
  @class
  retain<T>(_ value: T) -> T where T <: Self { value }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), src));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let callable = CallableId::new(
        DeclarationId::new(module, "Pair".into()),
        Selector::method("retain", [phalcom_common::selector::SelectorSlot::Positional]).unwrap(),
        DispatchSide::Class,
    );
    let definition = update.snapshot.callable_definitions.get(&callable).expect("accepted inherent definition");
    let context = definition.inherent_body_context.as_ref().expect("covering impl body context");
    let mut store = update.snapshot.store.as_ref().clone();
    let body_signature = context
        .signature_for_body(&mut store, &update.snapshot.declarations, definition.signature.clone())
        .expect("accepted covering signature materializes for its source body");
    let generics = body_signature.generics.as_ref().expect("retain generic signature");
    let parameter = generics.parameters[0];
    assert!(matches!(
        &generics.constraints[0],
        GenericConstraint::Subtype {
            lower: TypeTerm::Canonical(lower),
            upper: TypeTerm::Canonical(upper),
        } if matches!(store.get(*lower), TypeData::Parameter(actual) if *actual == parameter)
            && *upper == context.source_target_head
    ));
    assert!(matches!(
        store.type_parameter(parameter).owner,
        TypeParameterOwner::Callable(ref owner) if owner == &callable
    ));
    assert!(
        update
            .snapshot
            .callable_analyses
            .get(&callable)
            .expect("retain body analysis")
            .diagnostics
            .is_empty()
    );
}

#[test]
fn test_covering_body_incremental_permutation_recomputes_and_matches_cold_analysis() {
    let source_v1 = r#"
data Pair<X, Y>(_ first: X, _ second: Y)
impl<A, B> Pair<A, B> {
  @class
  value(_ a: A, _ b: B) -> A { a }
}
"#;
    let source_v2 = r#"
data Pair<X, Y>(_ first: X, _ second: Y)
impl<A, B> Pair<B, A> {
  @class
  value(_ a: A, _ b: B) -> A { a }
}
"#;
    let module = test_module();
    let callable = CallableId::new(
        DeclarationId::new(module.clone(), "Pair".into()),
        Selector::method(
            "value",
            [
                phalcom_common::selector::SelectorSlot::Positional,
                phalcom_common::selector::SelectorSlot::Positional,
            ],
        )
        .unwrap(),
        DispatchSide::Class,
    );
    let definition_key = QueryKey::CallableDefinition(callable.clone());
    let body_key = QueryKey::CallableBody(callable.clone());
    let mut incremental = SemanticWorkspaceSession::new();
    let first = incremental.update(single_module_input(module.clone(), source_v1));
    assert!(!first.snapshot.has_errors(), "v1 diagnostics: {:?}", first.snapshot.diagnostics);
    let first_context_fingerprint = incremental
        .db()
        .query_state(&definition_key)
        .and_then(|state| state.input_fingerprint())
        .expect("initial callable definition fingerprint");
    let second = incremental.update(single_module_input(module.clone(), source_v2));
    assert!(!second.snapshot.has_errors(), "incremental diagnostics: {:?}", second.snapshot.diagnostics);
    let second_context_fingerprint = incremental
        .db()
        .query_state(&definition_key)
        .and_then(|state| state.input_fingerprint())
        .expect("permuted callable definition fingerprint");
    assert_ne!(
        first_context_fingerprint, second_context_fingerprint,
        "covering-map permutation must change body-context input"
    );
    assert_eq!(
        incremental.db().query_state(&body_key).and_then(|state| state.revision()),
        Some(second.snapshot.id.revision()),
        "the callable body must be recomputed in the permutation revision",
    );

    let mut cold = SemanticWorkspaceSession::new();
    let cold = cold.update(single_module_input(module.clone(), source_v2));
    assert_eq!(
        second.snapshot.diagnostics, cold.snapshot.diagnostics,
        "incremental and cold diagnostics must agree"
    );
    let incremental_body = second.snapshot.callable_analyses.get(&callable).expect("incremental Pair.value body");
    let cold_body = cold.snapshot.callable_analyses.get(&callable).expect("cold Pair.value body");
    assert_eq!(
        incremental_body.dependency_fingerprint, cold_body.dependency_fingerprint,
        "incremental and cold callable body semantic products must agree",
    );
    assert_eq!(incremental_body.return_validation, cold_body.return_validation);
    assert_eq!(incremental_body.diagnostics, cold_body.diagnostics);
    assert_eq!(
        second.snapshot.callable_signatures.get(&callable),
        cold.snapshot.callable_signatures.get(&callable),
        "incremental and cold public callable signatures must agree",
    );
    let public_signature = second.snapshot.callable_signatures.get(&callable).expect("public Pair.value signature");
    let return_ty = public_signature.declared_return.canonical_type().expect("declared return type");
    let TypeData::Parameter(return_parameter) = second.snapshot.store.get(return_ty) else {
        panic!("public return must remain declaration-parameter-owned: {return_ty:?}");
    };
    assert!(matches!(
        second.snapshot.store.type_parameter(*return_parameter).owner,
        TypeParameterOwner::Declaration(ref owner) if owner == &DeclarationId::new(module, "Pair".into())
    ));
}

#[test]
fn test_subclass_inherits_inherent_impl_method() {
    let module = test_module();
    let src = r#"
class Base {}

impl Base {
  greet() -> String {
    "hello from base"
  }
}

class Child is Base {
  testCall() -> String {
    self.greet()
  }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), src));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let child_decl = DeclarationId::new(module, "Child".into());
    let test_call = CallableId::new(child_decl, Selector::method("testCall", []).unwrap(), DispatchSide::Instance);
    let analysis = update.snapshot.callable_analyses.get(&test_call).expect("testCall body analysis");
    assert!(analysis.diagnostics.is_empty());
}

#[test]
fn test_data_declaration_inherent_impl_body() {
    let module = test_module();
    let src = r#"
data Point(_ x: Int, _ y: Int)

impl Point {
  sum() -> Int {
    100
  }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), src));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let point_decl = DeclarationId::new(module, "Point".into());
    let sum_callable = CallableId::new(point_decl, Selector::method("sum", []).unwrap(), DispatchSide::Instance);
    let analysis = update.snapshot.callable_analyses.get(&sum_callable).expect("sum body analysis");
    assert!(analysis.diagnostics.is_empty());
}

#[test]
fn test_enum_root_inherent_impl_body() {
    let module = test_module();
    let src = r#"
data Status {
  @variant Active
  @variant Inactive
}

impl Status {
  isStatus() -> Bool {
    true
  }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), src));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let status_decl = DeclarationId::new(module, "Status".into());
    let is_status = CallableId::new(status_decl, Selector::method("isStatus", []).unwrap(), DispatchSide::Instance);
    let analysis = update.snapshot.callable_analyses.get(&is_status).expect("isStatus body analysis");
    assert!(analysis.diagnostics.is_empty());
}
