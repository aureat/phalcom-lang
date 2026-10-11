use crate::semantic::support::Fixture;
use phalcom_ast::parse_source;
use phalcom_common::selector::Selector;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::checker::check_program;
use phalcom_semantic::declarations::{DeclarationTypeInfo, DeclarationTypeTable, bootstrap_universe_declarations};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{CallableId, CallableOwnerId, DeclarationId, DispatchSide};
use phalcom_semantic::impls::{ConformanceCompleteness, RequirementSelectionTemplate};
use phalcom_semantic::trait_dispatch::TraitDispatchSite;
use phalcom_semantic::types::annotation::SimpleTypeResolver;
use phalcom_semantic::types::id::KindId;
use phalcom_semantic::types::relation::MapTypeHierarchy;
use phalcom_semantic::types::store::TypeStore;

fn register_nominal_declaration(store: &mut TypeStore, declarations: &mut DeclarationTypeTable, declaration: DeclarationId) {
    let form = store.nominal_type(declaration.clone());
    let class_object_type = store.class_object_type(declaration.clone());
    declarations.insert(DeclarationTypeInfo {
        declaration,
        form,
        class_object_type,
        kind: KindId::TYPE,
        binding: phalcom_semantic::declarations::DeclarationTypeBinding::ordinary(),
        generic_binder_header: None,
        generic_signature: None,
        supertype_template: None,
    });
}

fn setup_class_dispatch_env() -> (
    TypeStore,
    MapTypeHierarchy,
    SimpleTypeResolver,
    phalcom_semantic::declarations::DeclarationTypeTable,
    ModuleId,
) {
    let mut store = TypeStore::new();
    let mut hierarchy = MapTypeHierarchy::new();
    let mut resolver = SimpleTypeResolver::new();
    let module = ModuleId::universe_root();

    let declarations = bootstrap_universe_declarations(&mut store, &|k| DeclarationId::new(module.clone(), k.name().into()));

    let int_decl = DeclarationId::new(module.clone(), "Int".into());
    let string_decl = DeclarationId::new(module.clone(), "String".into());
    let bool_decl = DeclarationId::new(module.clone(), "Bool".into());
    let obj_decl = DeclarationId::new(module.clone(), "Object".into());

    hierarchy.insert(int_decl.clone(), obj_decl.clone());
    hierarchy.insert(string_decl.clone(), obj_decl.clone());
    hierarchy.insert(bool_decl.clone(), obj_decl.clone());

    resolver.insert("Int", int_decl);
    resolver.insert("String", string_decl);
    resolver.insert("Bool", bool_decl);
    resolver.insert("Object", obj_decl);

    (store, hierarchy, resolver, declarations, module)
}

#[test]
fn class_object_dispatches_to_class_side_method() {
    let (mut store, hier, resolver, mut decls, module) = setup_class_dispatch_env();
    register_nominal_declaration(&mut store, &mut decls, DeclarationId::new(module.clone(), "MathUtils".into()));
    let source = r#"
class MathUtils {
  @class
  createDefault -> Int {
    42
  }
}

class Main {
  test -> Int {
    MathUtils.createDefault()
  }
}
"#;
    let program = parse_source(source, 0).expect("valid parse");
    let report = check_program(&mut store, &hier, &resolver, &decls, module, &program);
    assert!(
        !report.has_errors(),
        "expected class side dispatch to succeed, got errors: {:?}",
        report.diagnostics
    );
}

#[test]
fn instance_receiver_dispatches_to_instance_side_method() {
    let (mut store, hier, mut resolver, mut decls, module) = setup_class_dispatch_env();
    let counter_decl = DeclarationId::new(module.clone(), "Counter".into());
    register_nominal_declaration(&mut store, &mut decls, counter_decl.clone());
    resolver.insert("Counter", counter_decl);
    let source = r#"
class Counter {
  _val: Int = 0

  val -> Int {
    _val
  }

  @class
  val -> String {
    "CounterClass"
  }
}

class Main {
  test(c: Counter) -> Int {
    c.val
  }

  testClass -> String {
    Counter.val
  }
}
"#;
    let program = parse_source(source, 0).expect("valid parse");
    let report = check_program(&mut store, &hier, &resolver, &decls, module, &program);
    assert!(
        !report.has_errors(),
        "expected separate instance and class side methods to dispatch correctly, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn super_send_in_instance_method_resolves_to_superclass_instance_method() {
    let (mut store, mut hier, mut resolver, mut decls, module) = setup_class_dispatch_env();

    let base_decl = DeclarationId::new(module.clone(), "Base".into());
    let sub_decl = DeclarationId::new(module.clone(), "Sub".into());
    let obj_decl = DeclarationId::new(module.clone(), "Object".into());

    register_nominal_declaration(&mut store, &mut decls, base_decl.clone());
    register_nominal_declaration(&mut store, &mut decls, sub_decl.clone());

    hier.insert(base_decl.clone(), obj_decl);
    hier.insert(sub_decl.clone(), base_decl.clone());

    resolver.insert("Base", base_decl);
    resolver.insert("Sub", sub_decl);

    let source = r#"
class Base {
  value -> Int {
    10
  }
}

class Sub is Base {
  value -> Int {
    super.value + 1
  }
}
"#;
    let program = parse_source(source, 0).expect("valid parse");
    let report = check_program(&mut store, &hier, &resolver, &decls, module, &program);
    assert!(
        !report.has_errors(),
        "expected super send in instance method to resolve to superclass instance method, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn super_send_in_class_method_resolves_to_superclass_class_method() {
    let (mut store, mut hier, mut resolver, mut decls, module) = setup_class_dispatch_env();

    let base_decl = DeclarationId::new(module.clone(), "BaseFactory".into());
    let sub_decl = DeclarationId::new(module.clone(), "SubFactory".into());
    let obj_decl = DeclarationId::new(module.clone(), "Object".into());

    register_nominal_declaration(&mut store, &mut decls, base_decl.clone());
    register_nominal_declaration(&mut store, &mut decls, sub_decl.clone());

    hier.insert(base_decl.clone(), obj_decl);
    hier.insert(sub_decl.clone(), base_decl.clone());

    resolver.insert("BaseFactory", base_decl);
    resolver.insert("SubFactory", sub_decl);

    let source = r#"
class BaseFactory {
  @class
  createId -> Int {
    100
  }
}

class SubFactory is BaseFactory {
  @class
  createId -> Int {
    super.createId() + 5
  }
}
"#;
    let program = parse_source(source, 0).expect("valid parse");
    let report = check_program(&mut store, &hier, &resolver, &decls, module, &program);
    assert!(
        !report.has_errors(),
        "expected super send in class method to resolve to superclass class method, got: {:?}",
        report.diagnostics
    );
}

#[test]
fn class_side_methods_getters_and_setters_share_semantic_and_source_index_identity() {
    let fixture = Fixture::new(
        r#"
class DispatchSurface {
  @class make() -> Int { 1 }
  @class value -> Int { 2 }
  @class value=(_ next: Int) {}
}
"#,
    );
    let owner = fixture.decl("DispatchSurface");
    let callables = [
        CallableId::new(owner.clone(), Selector::method("make", []).unwrap(), DispatchSide::Class),
        CallableId::new(owner.clone(), Selector::getter("value").unwrap(), DispatchSide::Class),
        CallableId::new(owner.clone(), Selector::setter("value").unwrap(), DispatchSide::Class),
    ];

    for callable in callables {
        let signature = fixture
            .analysis
            .snapshot
            .callable_signatures
            .get(&callable)
            .unwrap_or_else(|| panic!("semantic checker did not publish {callable:?}"));
        assert_eq!(signature.side, DispatchSide::Class);
        let indexed = fixture
            .analysis
            .snapshot
            .source_index
            .callable_source(&callable)
            .unwrap_or_else(|| panic!("source index did not publish the same class-side identity {callable:?}"));
        assert_eq!(indexed.id, callable);
    }
}

#[test]
fn class_side_trait_index_requirement_is_rejected_and_not_indexed_as_class_side() {
    let fixture = Fixture::new_allowing_internal_incidents(
        r#"
trait Indexed {
  @class [_ index: Int] -> Int
}
"#,
    );
    let trait_decl = fixture.decl("Indexed");
    assert!(
        fixture.analysis.snapshot.all_diagnostics().any(|diagnostic| {
            diagnostic.code == DiagnosticCode::TraitMemberConflict && diagnostic.message.contains("class-side index requirements are not supported")
        }),
        "expected the class-side-index diagnostic, got {:?}",
        fixture.analysis.snapshot.all_diagnostics().collect::<Vec<_>>()
    );

    let class_side_index = fixture
        .analysis
        .snapshot
        .source_index
        .modules()
        .flat_map(|(_, module)| module.structure.callable_sources.keys())
        .any(|callable| callable.owner == CallableOwnerId::Declaration(trait_decl.clone()) && callable.side == DispatchSide::Class);
    assert!(
        !class_side_index,
        "rejected class-side index requirement must not be published in the source index"
    );
}

#[test]
fn dual_side_selector_does_not_collide() {
    let fixture = Fixture::new(
        r#"
trait Ambiguous {
  foo() -> Int
  @class foo() -> Int
}
data Both(_ value: Int)
impl Ambiguous for Both {
  foo() -> Int { self.value }
  @class foo() -> Int { 2 }
}
class Caller {
  @class classCall() -> Int { Both.foo() }
  @class instanceCall(_ value: Both) -> Int { value.foo() }
}
"#,
    );
    let trait_decl = fixture.decl("Ambiguous");
    let surface = fixture.analysis.snapshot.trait_surfaces.get(&trait_decl).expect("trait surface");
    let selector = Selector::method("foo", []).expect("foo selector");
    assert!(surface.get_by_selector(&selector, DispatchSide::Instance).is_some());
    assert!(surface.get_by_selector(&selector, DispatchSide::Class).is_some());

    let class_call = fixture.callable("Caller", "classCall", DispatchSide::Class);
    let class_expression = fixture.expression(class_call, "Both.foo()");
    let Some(TraitDispatchSite::Evidenced(class_selection)) = class_expression.trait_dispatch.as_ref() else {
        panic!("class call must retain evidenced class-side trait dispatch: {class_expression:#?}");
    };
    assert_eq!(class_selection.requirement.side, DispatchSide::Class);

    let instance_call = fixture.callable("Caller", "instanceCall", DispatchSide::Class);
    let instance_expression = fixture.expression(instance_call, "value.foo()");
    let Some(TraitDispatchSite::Evidenced(instance_selection)) = instance_expression.trait_dispatch.as_ref() else {
        panic!("instance call must retain evidenced instance-side trait dispatch: {instance_expression:#?}");
    };
    assert_eq!(instance_selection.requirement.side, DispatchSide::Instance);
}

#[test]
fn instance_witness_rejected_for_class_requirement() {
    let fixture = Fixture::new_allowing_internal_incidents(
        r#"
trait ClassOnly { @class bar() -> Int }
class InstanceWitness {}
impl ClassOnly for InstanceWitness {
  bar() -> Int { 1 }
}
"#,
    );
    let incomplete = fixture
        .analysis
        .snapshot
        .all_diagnostics()
        .filter(|diagnostic| diagnostic.code == DiagnosticCode::ImplConformanceIncomplete)
        .collect::<Vec<_>>();
    assert_eq!(
        incomplete.len(),
        1,
        "an instance witness must leave the class-side conformance incomplete: {incomplete:#?}"
    );
    assert!(
        incomplete.iter().all(|diagnostic| diagnostic
            .labels
            .iter()
            .any(|label| label.message.contains("no explicit witness or trait default"))),
        "a same-spelled instance witness must not satisfy a class-side requirement: {incomplete:#?}"
    );
}

#[test]
fn class_witness_rejected_for_instance_requirement() {
    let fixture = Fixture::new_allowing_internal_incidents(
        r#"
trait InstanceOnly { bar() -> Int }
class ClassWitness {}
impl InstanceOnly for ClassWitness {
  @class bar() -> Int { 2 }
}
"#,
    );
    let incomplete = fixture
        .analysis
        .snapshot
        .all_diagnostics()
        .filter(|diagnostic| diagnostic.code == DiagnosticCode::ImplConformanceIncomplete)
        .collect::<Vec<_>>();
    assert_eq!(incomplete.len(), 1, "a class witness must leave the instance conformance incomplete: {incomplete:#?}");
    assert!(
        incomplete[0]
            .labels
            .iter()
            .any(|label| label.message.contains("no explicit witness or trait default")),
        "a same-spelled class witness must not satisfy an instance-side requirement: {incomplete:#?}"
    );
}

#[test]
fn explicit_class_witness_satisfies_requirement() {
    let fixture = Fixture::new(
        r#"
trait ClassOnly { @class bar() -> Int }
class Subject {}
impl ClassOnly for Subject {
  @class bar() -> Int { 9 }
}
"#,
    );
    let plan = fixture
        .analysis
        .snapshot
        .conformance_witness_plans
        .values()
        .find(|plan| plan.source_module == fixture.module)
        .expect("conformance witness plan");
    assert!(matches!(plan.completeness, ConformanceCompleteness::Complete));
    assert!(plan.requirements.iter().any(|(requirement, selection)| {
        requirement.side == DispatchSide::Class && matches!(selection, RequirementSelectionTemplate::ConformanceCallable { .. })
    }));
}

#[test]
fn inherent_class_member_satisfies_trait_requirement() {
    let fixture = Fixture::new(
        r#"
trait HasPi { @class pi -> Float }
class Math {
  @class pi -> Float { 3.14159 }
}
impl HasPi for Math {}
"#,
    );
    let plan = fixture
        .analysis
        .snapshot
        .conformance_witness_plans
        .values()
        .find(|plan| plan.source_module == fixture.module)
        .expect("conformance witness plan");
    assert!(matches!(plan.completeness, ConformanceCompleteness::Complete));
    assert!(plan.requirements.iter().any(|(requirement, selection)| {
        requirement.side == DispatchSide::Class && matches!(selection, RequirementSelectionTemplate::InherentCallable { .. })
    }));
}

#[test]
fn class_side_getter_setter_requirements_round_trip() {
    let fixture = Fixture::new(
        r#"
trait HasVersion { @class mut version: Int }
class Component {}
impl HasVersion for Component {
  @class version -> Int { 42 }
  @class version=(_ next: Int) {}
}
class Caller {
  @class read() -> Int { Component.version }
  @class write() { Component.version = 7 }
}
"#,
    );
    let trait_decl = fixture.decl("HasVersion");
    let surface = fixture.analysis.snapshot.trait_surfaces.get(&trait_decl).expect("trait surface");
    let class_requirements = surface.iter().filter(|(requirement, _)| requirement.side == DispatchSide::Class).count();
    assert_eq!(
        class_requirements, 2,
        "mutable class property publishes class-side getter and setter requirements"
    );
    let plan = fixture
        .analysis
        .snapshot
        .conformance_witness_plans
        .values()
        .find(|plan| plan.source_module == fixture.module)
        .expect("conformance witness plan");
    assert!(matches!(plan.completeness, ConformanceCompleteness::Complete));
    assert!(
        !fixture.analysis.snapshot.has_errors(),
        "class-side getter/setter read/write must typecheck: {:?}",
        fixture.analysis.snapshot.all_diagnostics().collect::<Vec<_>>()
    );
}

#[test]
fn metadata_records_class_side() {
    let fixture = Fixture::new("class Metadata { @class value() -> Int { 1 } }\n");
    let callable = CallableId::new(fixture.decl("Metadata"), Selector::method("value", []).unwrap(), DispatchSide::Class);
    let signature = fixture
        .analysis
        .snapshot
        .callable_signatures
        .get(&callable)
        .expect("class-side callable metadata");
    assert_eq!(signature.side, DispatchSide::Class);
}

#[test]
fn source_index_records_class_side() {
    let fixture = Fixture::new("class IndexedMetadata { @class value() -> Int { 1 } }\n");
    let callable = CallableId::new(fixture.decl("IndexedMetadata"), Selector::method("value", []).unwrap(), DispatchSide::Class);
    let source = fixture
        .analysis
        .snapshot
        .source_index
        .callable_source(&callable)
        .expect("class-side source-index record");
    assert_eq!(source.id.side, DispatchSide::Class);
}
