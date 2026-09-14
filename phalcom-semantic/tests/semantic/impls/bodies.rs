use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::LinkedModuleInterface;
use phalcom_modules::linker::{LinkedModule, LinkedProgram, ModuleBindingLayout};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::source::ModuleKind;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::source::ParsedModuleUnit;
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
enum Status {
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
