use phalcom_ast::parse_source;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::checker::check_program;
use phalcom_semantic::declarations::{DeclarationTypeInfo, DeclarationTypeTable, bootstrap_universe_declarations};
use phalcom_semantic::identity::DeclarationId;
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
    let module = ModuleId::core();

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
