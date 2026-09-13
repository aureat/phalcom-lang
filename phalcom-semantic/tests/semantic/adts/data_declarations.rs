use std::sync::Arc;

use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::data_semantics::DataShape;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_common::selector::Selector;

fn test_module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(42), ModulePath::root())
}

#[test]
fn data_declaration_publishes_nominal_components() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data Point(x: Int, y: Int)
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));

    let point_decl = DeclarationId::new(module, "Point".into());
    let data_info = analysis.snapshot.data_semantics.data_info(&point_decl);
    assert!(data_info.is_some(), "data product for Point should be published");
    let info = data_info.unwrap();
    assert_eq!(info.owner.name.as_ref(), "Point");
    assert_eq!(info.components.len(), 2);
    assert_eq!(info.components[0].local_name.as_ref(), "x");
    assert_eq!(info.components[0].id.index, 0);
    assert_eq!(info.components[1].local_name.as_ref(), "y");
    assert_eq!(info.components[1].id.index, 1);
    assert!(matches!(info.shape, DataShape::Tuple));
    assert_eq!(info.constructor.parameters.len(), 2);
}

#[test]
fn data_record_declaration_publishes_named_components() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data User {
    name: String,
    age: Int,
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));

    let user_decl = DeclarationId::new(module, "User".into());
    let data_info = analysis.snapshot.data_semantics.data_info(&user_decl);
    assert!(data_info.is_some(), "data product for User should be published");
    let info = data_info.unwrap();
    assert_eq!(info.components.len(), 2);
    assert_eq!(info.components[0].local_name.as_ref(), "name");
    assert_eq!(info.components[1].local_name.as_ref(), "age");
    assert!(matches!(info.shape, DataShape::Record));
}

#[test]
fn data_constructor_and_component_projection_in_body() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data Point(x: Int, y: Int)

class Geometry {
    @class test() -> Int {
        let p = Point(x: 10, y: 20);
        p.x
    }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let diags = analysis.snapshot.diagnostics_for(&module);
    assert!(diags.is_none() || diags.unwrap().is_empty(), "diagnostics: {:#?}", diags);

    let geom_decl = DeclarationId::new(module, "Geometry".into());
    let test_callable = CallableId::new(geom_decl, Selector::method("test", []).unwrap(), DispatchSide::Class);
    let callable_analysis = analysis.snapshot.callable_analyses.get(&test_callable);
    assert!(callable_analysis.is_some(), "CallableAnalysis for Geometry.test should exist");
}

#[test]
fn data_component_assignment_is_rejected() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data Point(x: Int, y: Int)

class Geometry {
    @class test() {
        let p = Point(x: 10, y: 20);
        p.x = 30;
    }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let diags = analysis.snapshot.diagnostics_for(&module).expect("diagnostics for module");
    assert!(
        diags
            .iter()
            .any(|d| d.code == DiagnosticCode::DataComponentImmutable),
        "should emit DataComponentImmutable diagnostic, got: {:#?}",
        diags
    );
}

#[test]
fn data_duplicate_component_is_rejected() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data Duplicate(x: Int, x: String)
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let diags = analysis.snapshot.diagnostics_for(&module).expect("diagnostics for module");
    assert!(
        diags
            .iter()
            .any(|d| d.code == DiagnosticCode::DataDuplicateComponent),
        "should emit DataDuplicateComponent diagnostic, got: {:#?}",
        diags
    );
}

#[test]
fn data_generic_box_specialization() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data Box<T>(_ value: T)

class Container {
    @class unbox() -> Int {
        let b = Box(42);
        b.value
    }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let diags = analysis.snapshot.diagnostics_for(&module);
    assert!(diags.is_none() || diags.unwrap().is_empty(), "diagnostics: {:#?}", diags);
}

#[test]
fn data_record_construction_and_projection() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data Person {
    name: String,
    age: Int,
}

class Directory {
    @class check() -> String {
        let p = Person { name: "Alice", age: 30 };
        p.name
    }
}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let diags = analysis.snapshot.diagnostics_for(&module);
    assert!(diags.is_none() || diags.unwrap().is_empty(), "diagnostics: {:#?}", diags);
}

#[test]
fn data_phantom_applications_remain_distinct() {
    use phalcom_semantic::types::id::KindId;
    use phalcom_semantic::types::relation::{MapTypeHierarchy, is_subtype};
    use phalcom_semantic::types::store::TypeStore;

    let mut store = TypeStore::new();
    let hier = MapTypeHierarchy::new();
    let module = test_module();

    let tagged_decl = DeclarationId::new(module.clone(), "Tagged".into());
    let int_decl = DeclarationId::new(module.clone(), "Int".into());
    let str_decl = DeclarationId::new(module, "String".into());

    let int_ty = store.nominal_type(int_decl);
    let str_ty = store.nominal_type(str_decl);

    let tagged_kind = store.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let tagged_root = store.nominal_form(tagged_decl, tagged_kind);
    let tagged_int = store.apply_type_form(tagged_root, &[int_ty]).expect("apply tagged int");
    let tagged_str = store.apply_type_form(tagged_root, &[str_ty]).expect("apply tagged str");

    assert_ne!(tagged_int, tagged_str);
    assert!(is_subtype(&mut store, &hier, tagged_int, tagged_int));
    assert!(is_subtype(&mut store, &hier, tagged_str, tagged_str));
    assert!(!is_subtype(&mut store, &hier, tagged_int, tagged_str));
    assert!(!is_subtype(&mut store, &hier, tagged_str, tagged_int));
}

#[test]
fn data_declaration_cold_incremental_equivalence() {
    let module = test_module();
    let source = "data Point(x: Int, y: Int)\nclass Geometry { @class test() -> Int { let p = Point(x: 10, y: 20); p.x } }\n";
    let parsed1 = phalcom_ast::parse(source, 0);
    assert!(parsed1.errors.is_empty(), "parse errors: {:#?}", parsed1.errors);
    let cold_analysis = analyze_single_module(module.clone(), Arc::from(source), Arc::new(parsed1.program));

    let mut session = phalcom_semantic::session::SemanticWorkspaceSession::new();
    let input = crate::semantic::incremental::support::single_module_input(module.clone(), source, 1);
    let update = session.update(input);

    let cold_point = cold_analysis.snapshot.data_semantics.data_info(&DeclarationId::new(module.clone(), "Point".into()));
    let incr_point = update.snapshot.data_semantics.data_info(&DeclarationId::new(module.clone(), "Point".into()));

    assert!(cold_point.is_some());
    assert!(incr_point.is_some());
    assert_eq!(cold_point.unwrap().components.len(), incr_point.unwrap().components.len());
}
