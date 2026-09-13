use std::sync::Arc;

use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::analyze_single_module;
use phalcom_semantic::data_semantics::DataShape;
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};

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
        diags.iter().any(|d| d.code == DiagnosticCode::DataComponentImmutable),
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
        diags.iter().any(|d| d.code == DiagnosticCode::DataDuplicateComponent),
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

    let cold_point = cold_analysis
        .snapshot
        .data_semantics
        .data_info(&DeclarationId::new(module.clone(), "Point".into()));
    let incr_point = update.snapshot.data_semantics.data_info(&DeclarationId::new(module.clone(), "Point".into()));

    assert!(cold_point.is_some());
    assert!(incr_point.is_some());
    assert_eq!(cold_point.unwrap().components.len(), incr_point.unwrap().components.len());
}

#[test]
fn class_cannot_inherit_from_data() {
    let module = test_module();
    let source: Arc<str> = Arc::from(
        r#"
data Point(x: Int, y: Int)
class StrangePoint is Point {}
"#,
    );
    let parsed = phalcom_ast::parse(&source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:#?}", parsed.errors);
    let analysis = analyze_single_module(module.clone(), source, Arc::new(parsed.program));
    let diags = analysis.snapshot.diagnostics_for(&module).expect("diagnostics expected");
    assert!(
        diags.iter().any(|d| d.code == DiagnosticCode::DataUsedAsSuperclass),
        "expected DataUsedAsSuperclass diagnostic, got: {:#?}",
        diags
    );
}

#[test]
fn class_cannot_inherit_from_data_incremental() {
    let module = test_module();
    let mut session = phalcom_semantic::session::SemanticWorkspaceSession::new();
    let source1 = "class Base {}\nclass Derived is Base {}\n";
    let update1 = session.update(crate::semantic::incremental::support::single_module_input(module.clone(), source1, 1));
    let diags1 = update1.snapshot.diagnostics_for(&module);
    assert!(diags1.is_none_or(|d| d.is_empty()));

    let source2 = "data Base(x: Int)\nclass Derived is Base {}\n";
    let update2 = session.update(crate::semantic::incremental::support::single_module_input(module.clone(), source2, 2));
    let diags2 = update2.snapshot.diagnostics_for(&module).expect("diagnostics expected in update 2");
    assert!(
        diags2.iter().any(|d| d.code == DiagnosticCode::DataUsedAsSuperclass),
        "expected DataUsedAsSuperclass diagnostic after Base became data, got: {:#?}",
        diags2
    );
}

#[test]
fn data_cross_module_record_construction_order() {
    let module_a = ModuleId::resolved(
        ResolvedProjectId::from_raw(42),
        ModulePath::from_components(vec![phalcom_modules::identity::ModuleComponent::from_identifier("schema").unwrap()]),
    );
    let module_b = ModuleId::resolved(
        ResolvedProjectId::from_raw(42),
        ModulePath::from_components(vec![phalcom_modules::identity::ModuleComponent::from_identifier("app").unwrap()]),
    );

    let source_a = "data User { name: String, age: Int }\nexport User\n";
    let source_b = "import schema.User\nclass App { @class make() -> User { User(age: 30, name: \"Alice\") } }\n";

    let mut session = phalcom_semantic::session::SemanticWorkspaceSession::new();
    let update = session.update(crate::semantic::incremental::support::multi_module_input(
        vec![(module_a.clone(), source_a.into()), (module_b.clone(), source_b.into())],
        1,
    ));

    let user_decl = DeclarationId::new(module_a, "User".into());
    let user_info = update.snapshot.data_semantics.data_info(&user_decl);
    assert!(user_info.is_some(), "data product for User should exist");
    let info = user_info.unwrap();
    assert_eq!(info.components.len(), 2);
    assert_eq!(info.components[0].local_name.as_ref(), "name");
    assert_eq!(info.components[1].local_name.as_ref(), "age");
}

#[test]
fn data_incremental_imported_component_type_mutation() {
    let module_a = ModuleId::resolved(
        ResolvedProjectId::from_raw(42),
        ModulePath::from_components(vec![phalcom_modules::identity::ModuleComponent::from_identifier("model").unwrap()]),
    );
    let module_b = ModuleId::resolved(
        ResolvedProjectId::from_raw(42),
        ModulePath::from_components(vec![phalcom_modules::identity::ModuleComponent::from_identifier("client").unwrap()]),
    );

    let source_a_v1 = "data Container(x: Int)\n";
    let source_b = "data Other(y: Int)\n";

    let mut session = phalcom_semantic::session::SemanticWorkspaceSession::new();
    let update1 = session.update(crate::semantic::incremental::support::multi_module_input(
        vec![(module_a.clone(), source_a_v1.into()), (module_b.clone(), source_b.into())],
        1,
    ));
    let container_decl = DeclarationId::new(module_a.clone(), "Container".into());
    let info1 = update1.snapshot.data_semantics.data_info(&container_decl).unwrap();
    assert_eq!(info1.components.len(), 1);

    // Now change Container.x from Int to String in module A
    let source_a_v2 = "data Container(x: String)\n";
    let update2 = session.update(crate::semantic::incremental::support::multi_module_input(
        vec![(module_a.clone(), source_a_v2.into()), (module_b.clone(), source_b.into())],
        2,
    ));

    // Data declaration product in module A must reflect String
    let container_info = update2.snapshot.data_semantics.data_info(&container_decl).unwrap();
    assert_eq!(container_info.components.len(), 1);
    assert_ne!(info1, container_info);
}
