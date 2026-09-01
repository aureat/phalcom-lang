use phalcom_ast::parse_source;
use phalcom_modules::identity::ModuleId;
use phalcom_semantic::DeclarationId;
use phalcom_semantic::checker::check_program;
use phalcom_semantic::declarations::{DeclarationTypeTable, bootstrap_universe_declarations};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::types::annotation::SimpleTypeResolver;
use phalcom_semantic::types::relation::MapTypeHierarchy;
use phalcom_semantic::types::store::TypeStore;

pub fn setup_test_env() -> (TypeStore, MapTypeHierarchy, SimpleTypeResolver, DeclarationTypeTable, ModuleId) {
    let mut store = TypeStore::new();
    let mut hierarchy = MapTypeHierarchy::new();
    let mut resolver = SimpleTypeResolver::new();
    let module = ModuleId::universe_root();

    let declarations = bootstrap_universe_declarations(&mut store, &|k| DeclarationId::new(module.clone(), k.name().into()));

    let int_decl = DeclarationId::new(module.clone(), "Int".into());
    let string_decl = DeclarationId::new(module.clone(), "String".into());
    let bool_decl = DeclarationId::new(module.clone(), "Bool".into());
    let obj_decl = DeclarationId::new(module.clone(), "Object".into());
    let num_decl = DeclarationId::new(module.clone(), "Number".into());

    hierarchy.insert(num_decl.clone(), obj_decl.clone());
    hierarchy.insert(int_decl.clone(), num_decl.clone());
    hierarchy.insert(string_decl.clone(), obj_decl.clone());
    hierarchy.insert(bool_decl.clone(), obj_decl.clone());

    resolver.insert("Int", int_decl);
    resolver.insert("String", string_decl);
    resolver.insert("Bool", bool_decl);
    resolver.insert("Object", obj_decl);
    resolver.insert("Number", num_decl);

    (store, hierarchy, resolver, declarations, module)
}

#[test]
fn checks_valid_const_and_let_bindings() {
    let (mut store, hier, resolver, decls, module) = setup_test_env();
    let source = "const count: Int = 1\nlet name: String = \"phalcom\"\n";
    let program = parse_source(source, 0).expect("valid parse");

    let report = check_program(&mut store, &hier, &resolver, &decls, module, &program);
    assert!(!report.has_errors(), "expected no errors, got: {:?}", report.diagnostics);
}

#[test]
fn detects_binding_initializer_mismatch() {
    let (mut store, hier, resolver, decls, module) = setup_test_env();
    let source = "const count: String = 1\n";
    let program = parse_source(source, 0).expect("valid parse");

    let report = check_program(&mut store, &hier, &resolver, &decls, module, &program);
    assert!(report.has_errors(), "expected binding mismatch error");
    assert_eq!(report.diagnostics[0].code, DiagnosticCode::BindingInitializerMismatch);
}

#[test]
fn detects_method_return_mismatch() {
    let (mut store, hier, resolver, decls, module) = setup_test_env();
    let source = "class Port {\n  number -> Int {\n    \"8080\"\n  }\n}\n";
    let program = parse_source(source, 0).expect("valid parse");

    let report = check_program(&mut store, &hier, &resolver, &decls, module, &program);
    assert!(report.has_errors(), "expected return mismatch error");
    assert_eq!(report.diagnostics[0].code, DiagnosticCode::ReturnMismatch);
}

#[test]
fn detects_field_default_mismatch() {
    let (mut store, hier, resolver, decls, module) = setup_test_env();
    let source = "class Config {\n  _port: Int = \"invalid\"\n}\n";
    let program = parse_source(source, 0).expect("valid parse");

    let report = check_program(&mut store, &hier, &resolver, &decls, module, &program);
    assert!(report.has_errors(), "expected field default mismatch error");
    assert_eq!(report.diagnostics[0].code, DiagnosticCode::FieldMismatch);
}

#[test]
fn unannotated_dynamic_code_has_no_false_errors() {
    let (mut store, hier, resolver, decls, module) = setup_test_env();
    let source = "const x = 1\nlet y = \"hello\"\nclass DynamicClass {\n  compute(a, b) {\n    a + b\n  }\n}\n";
    let program = parse_source(source, 0).expect("valid parse");

    let report = check_program(&mut store, &hier, &resolver, &decls, module, &program);
    assert!(!report.has_errors(), "dynamic unannotated code should have no errors");
}
