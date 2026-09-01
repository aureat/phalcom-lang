use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_modules::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::types::TypeTerm;
use phalcom_semantic::{CallableId, DeclarationId, DispatchSide, EditorMemberTarget, SemanticTargetId, ValueShape, analyze_single_module};
use std::sync::Arc;

#[test]
fn constructor_factory_publishes_target_and_inferred_signature() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
class CellNum {
  _raw: Int

  @constructor
  new(_ raw: Int) {
    _raw = raw
  }

  @class
  of(_ raw: Int) {
    CellNum.new(raw)
  }

  value() -> Int {
    _raw
  }
}

const x: Int = CellNum.of(42)
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    let snapshot = analysis.snapshot;

    let cell = DeclarationId::new(module.clone(), "CellNum".into());
    let cell_ty = snapshot.declarations.form(&cell).expect("CellNum type");
    let constructor = CallableId::new(
        cell.clone(),
        Selector::method("new", vec![SelectorSlot::Positional]).unwrap(),
        DispatchSide::Class,
    );
    let factory = CallableId::new(
        cell.clone(),
        Selector::method("of", vec![SelectorSlot::Positional]).unwrap(),
        DispatchSide::Class,
    );

    let factory_analysis = snapshot.callable_analyses.get(&factory).expect("CellNum.of body analysis");
    let constructor_call = factory_analysis
        .expressions
        .values()
        .find(|expression| source.get(expression.range.start..expression.range.end) == Some("CellNum.new(raw)"))
        .expect("constructor call expression in CellNum.of");
    assert_eq!(
        constructor_call.callable.as_ref(),
        Some(&constructor),
        "factory tail must resolve to the public class-side constructor identity: {constructor_call:#?}"
    );
    assert_eq!(
        constructor_call.knowledge.ty(),
        Some(cell_ty),
        "constructor call must formally produce CellNum: {constructor_call:#?}"
    );
    assert!(
        factory_analysis.exits.normal_returns.iter().any(|fact| fact.knowledge.ty() == Some(cell_ty)),
        "CellNum.of must publish the constructor result as a normal return: {factory_analysis:#?}"
    );

    let signature = snapshot.callable_signatures.get(&factory).expect("inferred CellNum.of signature");
    assert_eq!(
        signature.published_return_term(),
        Some(TypeTerm::Canonical(cell_ty)),
        "inferred factory signature must return CellNum"
    );

    let selector_offset = source.rfind("of(42)").expect("factory call") + 1;
    assert_eq!(
        snapshot.editor().target_at(&module, selector_offset),
        Some(SemanticTargetId::Callable(factory)),
        "top-level factory call must retain its exact canonical callable target"
    );
}

#[test]
fn constructorless_class_inherits_canonical_class_new() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
class Person {}

const person = Person.new()

class Probe {
  make() {
    Person.new()
  }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    let snapshot = analysis.snapshot;

    let person = DeclarationId::new(module.clone(), "Person".into());
    let person_ty = snapshot.declarations.form(&person).expect("Person type");
    let probe = DeclarationId::new(module.clone(), "Probe".into());
    let make = CallableId::new(probe, Selector::method("make", Vec::new()).unwrap(), DispatchSide::Instance);
    let default_new = CallableId::new(
        DeclarationId::new(ModuleId::universe_root(), "Class".into()),
        Selector::method("new", Vec::new()).unwrap(),
        DispatchSide::Instance,
    );

    let make_analysis = snapshot.callable_analyses.get(&make).expect("Probe.make body analysis");
    let constructor_call = make_analysis
        .expressions
        .values()
        .find(|expression| source.get(expression.range.start..expression.range.end) == Some("Person.new()"))
        .expect("Person.new expression");
    assert_eq!(
        constructor_call.callable.as_ref(),
        Some(&default_new),
        "constructor-less classes must resolve new() through canonical Class behavior: {constructor_call:#?}"
    );
    assert_eq!(
        constructor_call.knowledge.ty(),
        Some(person_ty),
        "Class.new Self result must specialize to the concrete class object receiver: {constructor_call:#?}"
    );

    let receiver_start = source.rfind("Person.new()").expect("Person.new call");
    let receiver_range = SourceRange {
        start: receiver_start,
        end: receiver_start + "Person".len(),
    };
    let editor = snapshot.editor();
    let receiver = editor.resolve_receiver_at(&module, receiver_range).expect("Person class-object receiver");
    let access = editor.access_context_at(&module, receiver_start);
    assert!(
        editor
            .members_for_receiver(&receiver, &access)
            .iter()
            .any(|member| member.target == EditorMemberTarget::Callable(default_new.clone())),
        "editor member enumeration must share formal class-object root dispatch"
    );

    let module_index = snapshot.source_index.module(&module).expect("source index");
    let person_binding = module_index
        .structure
        .bindings
        .values()
        .find(|binding| binding.name.as_ref() == "person")
        .expect("top-level person binding");
    assert_eq!(
        snapshot.advisory_fact(&person_binding.declaration_site).map(|fact| &fact.shape),
        Some(&ValueShape::Instance(person)),
        "known Self-returning Class.new must project the concrete receiver into advisory binding shape"
    );
}

#[test]
fn constructorless_class_inherits_core_class_new_from_resolved_module() {
    let module = ModuleId::resolved(ResolvedProjectId::from_raw(88), ModulePath::root());
    let source: Arc<str> = Arc::from(
        r#"
class Person {}

const person = Person.new()

class Probe {
  make() {
    Person.new()
  }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    let snapshot = analysis.snapshot;

    let person = DeclarationId::new(module.clone(), "Person".into());
    let person_ty = snapshot.declarations.form(&person).expect("Person type");
    let probe = DeclarationId::new(module.clone(), "Probe".into());
    let make = CallableId::new(probe, Selector::method("make", Vec::new()).unwrap(), DispatchSide::Instance);
    let default_new = CallableId::new(
        DeclarationId::new(ModuleId::universe_root(), "Class".into()),
        Selector::method("new", Vec::new()).unwrap(),
        DispatchSide::Instance,
    );

    let make_analysis = snapshot.callable_analyses.get(&make).expect("Probe.make body analysis");
    let constructor_call = make_analysis
        .expressions
        .values()
        .find(|expression| source.get(expression.range.start..expression.range.end) == Some("Person.new()"))
        .expect("Person.new expression");
    assert_eq!(
        constructor_call.callable.as_ref(),
        Some(&default_new),
        "resolved user modules must dispatch constructor-less new() to canonical core Class.new: {constructor_call:#?}"
    );
    assert_eq!(
        constructor_call.knowledge.ty(),
        Some(person_ty),
        "core Class.new Self return must specialize to a class object declared in another module: {constructor_call:#?}"
    );

    let module_index = snapshot.source_index.module(&module).expect("source index");
    let person_binding = module_index
        .structure
        .bindings
        .values()
        .find(|binding| binding.name.as_ref() == "person")
        .expect("top-level person binding");
    assert_eq!(
        snapshot.advisory_fact(&person_binding.declaration_site).map(|fact| &fact.shape),
        Some(&ValueShape::Instance(person)),
        "advisory top-level binding must preserve the concrete receiver of inherited Class.new across module boundaries"
    );
}

#[test]
fn receiver_query_falls_back_to_authoritative_binding_fact() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
class CellNum {
  _raw: Int

  @constructor
  new(_ raw: Int) {
    _raw = raw
  }

  @class
  of(_ raw: Int) {
    CellNum.new(raw)
  }

  value() -> Int {
    _raw
  }
}

const x: Int = CellNum.of(42)

class Probe {
  run() {
    x.value()
  }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    let snapshot = analysis.snapshot;

    let cell = DeclarationId::new(module.clone(), "CellNum".into());
    let module_index = snapshot.source_index.module(&module).expect("source index");
    let binding = module_index
        .structure
        .bindings
        .values()
        .find(|binding| binding.name.as_ref() == "x")
        .expect("top-level x binding");
    assert_eq!(
        snapshot.advisory_fact(&binding.declaration_site).map(|fact| &fact.shape),
        Some(&ValueShape::Instance(cell.clone())),
        "factory result must remain the authoritative advisory binding shape despite the bad Int annotation"
    );

    let receiver_start = source.rfind("x.value()").expect("x receiver");
    let receiver = snapshot
        .editor()
        .resolve_receiver_at(
            &module,
            SourceRange {
                start: receiver_start,
                end: receiver_start + 1,
            },
        )
        .expect("binding receiver must resolve from compiler-owned facts");
    assert!(
        receiver
            .alternatives
            .iter()
            .any(|alternative| { alternative.declaration == cell && matches!(alternative.mode, phalcom_semantic::ReceiverMode::Instance) }),
        "receiver query must consult the binding declaration fact when the use-site expression has no usable fact: {receiver:#?}"
    );
}

#[test]
fn advisory_inherited_call_publishes_defining_callable_target() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
class Animal {
  speak() {}
}

class Dog is Animal {}

const dog = Dog.new()
dog.speak()
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let snapshot = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program)).snapshot;

    let animal = DeclarationId::new(module.clone(), "Animal".into());
    let speak = CallableId::new(animal, Selector::method("speak", Vec::new()).unwrap(), DispatchSide::Instance);
    let selector_offset = source.rfind("speak()").expect("call-site speak") + 1;
    assert_eq!(
        snapshot.editor().target_at(&module, selector_offset),
        Some(SemanticTargetId::Callable(speak)),
        "advisory dispatch must publish the canonical defining callable at the selector occurrence"
    );
}

#[test]
fn partial_call_candidates_are_selected_by_canonical_receiver_surface() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from(
        r#"
class Service {
  compute(_ x: Int, label y: Int) -> Int { x }
  run() { self.compute(1) }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let snapshot = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program)).snapshot;
    let service = DeclarationId::new(module.clone(), "Service".into());
    let compute = CallableId::new(
        service,
        Selector::method("compute", vec![SelectorSlot::Positional, SelectorSlot::Label("label".to_string())]).unwrap(),
        DispatchSide::Instance,
    );
    let receiver_start = source.find("self.compute").expect("self receiver");
    let receiver = snapshot
        .editor()
        .resolve_receiver_at(
            &module,
            SourceRange {
                start: receiver_start,
                end: receiver_start + "self".len(),
            },
        )
        .expect("self receiver");
    let access = snapshot.editor().access_context_at(&module, receiver_start);
    let prefix = Selector::method("compute", Vec::new()).unwrap();
    let candidates = snapshot
        .editor()
        .callable_candidates(&receiver, &phalcom_semantic::PartialCallPattern::from_selector_prefix(&prefix), &access);
    assert_eq!(
        candidates,
        vec![compute],
        "empty written slot prefix must select the receiver's compatible canonical method"
    );
}

#[test]
fn class_object_method_family_uses_canonical_dispatch_owner_chain() {
    let module = ModuleId::resolved(ResolvedProjectId::from_raw(91), ModulePath::root());
    let source: Arc<str> = Arc::from(
        r#"
class Person {}
const family = Person::new
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    let snapshot = analysis.snapshot;

    let default_new = CallableId::new(
        DeclarationId::new(ModuleId::universe_root(), "Class".into()),
        Selector::method("new", Vec::new()).unwrap(),
        DispatchSide::Instance,
    );
    let module_index = snapshot.source_index.module(&module).expect("source index");
    let family_binding = module_index
        .structure
        .bindings
        .values()
        .find(|binding| binding.name.as_ref() == "family")
        .expect("family binding");
    let fact = snapshot.advisory_fact(&family_binding.declaration_site).expect("family advisory fact");
    let ValueShape::MethodFamily(family) = &fact.shape else {
        panic!("expected captured method family, got {fact:#?}");
    };
    assert!(
        family.exact.iter().any(|(_, callable)| callable == &default_new),
        "class-object family capture must traverse into canonical Class instance behavior: {family:#?}"
    );
}

fn builtin_annotation_snapshot() -> (ModuleId, Arc<str>, Arc<phalcom_semantic::SemanticSnapshot>, SemanticTargetId) {
    let module = ModuleId::resolved(ResolvedProjectId::from_raw(77), ModulePath::root());
    let source: Arc<str> = Arc::from("const x: Int = 42\n");
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    let target = SemanticTargetId::Declaration(DeclarationId::new(ModuleId::universe_root(), "Int".into()));
    (module, source, analysis.snapshot, target)
}

#[test]
fn builtin_type_annotation_has_canonical_target() {
    let (module, source, snapshot, target) = builtin_annotation_snapshot();
    let annotation_offset = source.find("Int").expect("Int annotation") + 1;
    assert_eq!(
        snapshot.editor().target_at(&module, annotation_offset),
        Some(target),
        "type annotation token must retain the exact canonical declaration target"
    );
}

#[test]
fn builtin_declaration_has_canonical_definition_site() {
    let (_, _, snapshot, target) = builtin_annotation_snapshot();
    let sites = snapshot.editor().definition_sites(&target);
    assert_eq!(
        sites.len(),
        1,
        "builtin declaration must publish exactly one canonical definition site: {sites:#?}"
    );
    let site = snapshot.source_site(&sites[0]).expect("builtin declaration source site");
    assert!(
        matches!(&site.id.owner, phalcom_semantic::SourceOwner::Module(owner) if owner == &ModuleId::universe_root()),
        "builtin definition site must belong to the compiler-owned core presentation shard: {site:#?}"
    );
}

#[test]
fn native_callable_presentation_is_compiler_owned() {
    let module = ModuleId::universe_root();
    let source: Arc<str> = Arc::from("let x = true.ifTrue || { 1 };\n");
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source.clone(), Arc::new(parsed.program));
    let snapshot = analysis.snapshot;
    let offset = source.find("ifTrue").expect("ifTrue call");
    let target = snapshot.editor().target_at(&module, offset).expect("native callable target");
    let SemanticTargetId::Callable(callable) = target else {
        panic!("expected callable target, got {target:#?}");
    };
    let native = snapshot.editor().native_callable_presentation(&callable).expect("native presentation metadata");
    assert!(
        native
            .documentation
            .is_some_and(|documentation| documentation.contains("Executes block if receiver is true.")),
        "unexpected native documentation: {native:#?}"
    );
}
