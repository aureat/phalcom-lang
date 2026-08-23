use phalcom_ast::parse_source;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::DeclarationId;
use phalcom_semantic::checker::check_program;
use phalcom_semantic::declarations::{DeclarationTypeTable, bootstrap_universe_declarations};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::types::annotation::SimpleTypeResolver;
use phalcom_semantic::types::relation::MapTypeHierarchy;
use phalcom_semantic::types::store::TypeStore;

fn setup_phase2_env() -> (TypeStore, MapTypeHierarchy, SimpleTypeResolver, DeclarationTypeTable, ModuleId) {
    let mut store = TypeStore::new();
    let mut hierarchy = MapTypeHierarchy::new();
    let mut resolver = SimpleTypeResolver::new();
    let module = ModuleId::core();

    let declarations = bootstrap_universe_declarations(&mut store, &|k| DeclarationId::new(module.clone(), k.name().into()));

    let int_decl = DeclarationId::new(module.clone(), "Int".into());
    let float_decl = DeclarationId::new(module.clone(), "Float".into());
    let string_decl = DeclarationId::new(module.clone(), "String".into());
    let bool_decl = DeclarationId::new(module.clone(), "Bool".into());
    let list_decl = DeclarationId::new(module.clone(), "List".into());
    let map_decl = DeclarationId::new(module.clone(), "Map".into());
    let set_decl = DeclarationId::new(module.clone(), "Set".into());
    let symbol_decl = DeclarationId::new(module.clone(), "Symbol".into());
    let obj_decl = DeclarationId::new(module.clone(), "Object".into());
    let num_decl = DeclarationId::new(module.clone(), "Number".into());

    hierarchy.insert(num_decl.clone(), obj_decl.clone());
    hierarchy.insert(int_decl.clone(), num_decl.clone());
    hierarchy.insert(float_decl.clone(), num_decl.clone());
    hierarchy.insert(string_decl.clone(), obj_decl.clone());
    hierarchy.insert(bool_decl.clone(), obj_decl.clone());
    hierarchy.insert(list_decl.clone(), obj_decl.clone());
    hierarchy.insert(map_decl.clone(), obj_decl.clone());
    hierarchy.insert(set_decl.clone(), obj_decl.clone());
    hierarchy.insert(symbol_decl.clone(), obj_decl.clone());

    resolver.insert("Int", int_decl);
    resolver.insert("Float", float_decl);
    resolver.insert("String", string_decl);
    resolver.insert("Bool", bool_decl);
    resolver.insert("List", list_decl);
    resolver.insert("Map", map_decl);
    resolver.insert("Set", set_decl);
    resolver.insert("Symbol", symbol_decl);
    resolver.insert("Object", obj_decl);
    resolver.insert("Number", num_decl);

    (store, hierarchy, resolver, declarations, module)
}

#[test]
fn test_literal_and_collection_typing() {
    let (mut store, hier, resolver, decls, module) = setup_phase2_env();
    let source = r#"
const a = 42
const b = 3.14
const c = "phalcom"
const d = true
const e = #test
const f = [1, 2, 3]
const g = { key: 100 }
const i = (1, "two")
const j = #{ name: "Alice", age: 30 }
"#;
    let program = parse_source(source, 0).expect("valid parse");
    let report = check_program(&mut store, &hier, &resolver, &decls, module, &program);
    assert!(!report.has_errors(), "expected no errors, got: {:?}", report.diagnostics);
}

#[test]
fn test_block_and_control_flow_typing() {
    let (mut store, hier, resolver, decls, module) = setup_phase2_env();
    let source = r#"
class Calculator {
  compute -> Int {
    10 + 20
  }
  emptyBlock -> Unit {
  }
}
"#;
    let program = parse_source(source, 0).expect("valid parse");
    let report = check_program(&mut store, &hier, &resolver, &decls, module, &program);
    assert!(!report.has_errors(), "expected no errors, got: {:?}", report.diagnostics);
}

#[test]
fn test_binary_and_unary_as_message_sends() {
    let (mut store, hier, resolver, decls, module) = setup_phase2_env();
    let source = r#"
class Arith {
  calc -> Int {
    1 + 2 * 3
  }
  negate -> Int {
    -42
  }
  check -> Bool {
    not (1 < 2)
  }
}
"#;
    let program = parse_source(source, 0).expect("valid parse");
    let report = check_program(&mut store, &hier, &resolver, &decls, module, &program);
    assert!(!report.has_errors(), "expected no errors, got: {:?}", report.diagnostics);
}

#[test]
fn test_keyword_and_positional_callable_matching() {
    let (mut store, hier, resolver, decls, module) = setup_phase2_env();
    let source = r#"
class Navigator {
  move(from a: Int, to b: Int) -> Int {
    a + b
  }
  navigate -> Int {
    self.move(from: 10, to: 20)
  }
}
"#;
    let program = parse_source(source, 0).expect("valid parse");
    let report = check_program(&mut store, &hier, &resolver, &decls, module, &program);
    assert!(!report.has_errors(), "expected no errors, got: {:?}", report.diagnostics);
}

#[test]
fn test_keyword_argument_mismatch_detected() {
    let (mut store, hier, resolver, decls, module) = setup_phase2_env();
    let source = r#"
class Navigator {
  move(from a: Int, to b: Int) -> Int {
    a + b
  }
  navigate -> Int {
    self.move(from: "invalid", to: 20)
  }
}
"#;
    let program = parse_source(source, 0).expect("valid parse");
    let report = check_program(&mut store, &hier, &resolver, &decls, module, &program);
    assert!(report.has_errors(), "expected error on keyword argument mismatch");
    assert_eq!(report.diagnostics[0].code, DiagnosticCode::ArgumentMismatch);
}

#[test]
fn test_member_and_subscript_typing() {
    let (mut store, hier, resolver, decls, module) = setup_phase2_env();
    let source = r#"
class Buffer {
  _capacity: Int = 1024

  capacity -> Int {
    _capacity
  }

  testAccess -> Int {
    self.capacity
  }
}
"#;
    let program = parse_source(source, 0).expect("valid parse");
    let report = check_program(&mut store, &hier, &resolver, &decls, module, &program);
    assert!(!report.has_errors(), "expected no errors, got: {:?}", report.diagnostics);
}

#[test]
fn test_local_constraint_inference() {
    let (mut store, hier, resolver, decls, module) = setup_phase2_env();
    let source = r#"
class CollectionUser {
  process {
    let xs = []
    xs.add(42)
  }
}
"#;
    let program = parse_source(source, 0).expect("valid parse");
    let report = check_program(&mut store, &hier, &resolver, &decls, module, &program);
    assert!(!report.has_errors(), "expected no errors, got: {:?}", report.diagnostics);
}

#[test]
fn test_dynamic_resilience() {
    let (mut store, hier, resolver, decls, module) = setup_phase2_env();
    let source = r#"
class DynamicHandler {
  handle(rawEvent) {
    rawEvent.trigger(1, 2, 3)
    rawEvent.payload[0]
  }
}
"#;
    let program = parse_source(source, 0).expect("valid parse");
    let report = check_program(&mut store, &hier, &resolver, &decls, module, &program);
    assert!(!report.has_errors(), "dynamic message sends must not produce false errors");
}

#[test]
fn test_for_loop_protocol_custom_iterable() {
    let (mut store, hier, mut resolver, decls, module) = setup_phase2_env();
    let stream_decl = DeclarationId::new(module.clone(), "MyCustomStream".into());
    resolver.insert("MyCustomStream", stream_decl);
    let source = r#"
class MyCustomStream {
  iteratorValue -> String {
    "item"
  }
}

class StreamUser {
  process(stream: MyCustomStream) {
    for item in stream {
      const s: String = item
    }
  }
}
"#;
    let program = parse_source(source, 0).expect("valid parse");
    let report = check_program(&mut store, &hier, &resolver, &decls, module, &program);
    assert!(
        !report.has_errors(),
        "protocol-derived for loop typing should succeed for custom iterable, got: {:?}",
        report.diagnostics
    );
}
