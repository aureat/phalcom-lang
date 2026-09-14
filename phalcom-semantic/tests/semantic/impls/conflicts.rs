use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::associated::build_associated_surface;
use phalcom_semantic::checker::declaration::register_class_surface;
use phalcom_semantic::data_semantics::{DataComponentSemantic, DataConstructorSignature, DataInfo, DataShape};
use phalcom_semantic::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact};
use phalcom_semantic::declarations::{DeclarationTypeTable, NominalDeclarationHeader};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::identity::{CallableId, DataComponentId, DataConstructorId, DeclarationId, DispatchSide, ImplId, ImplLocalId, VariantId};
use phalcom_semantic::impls::{build_effective_surface, build_inherent_impl_contribution, CallableDefinitionOrigin};
use phalcom_semantic::surface::DeclarationSurface;
use phalcom_semantic::types::annotation::SimpleTypeResolver;
use phalcom_semantic::types::parameter::TypeTerm;
use phalcom_semantic::types::relation::MapTypeHierarchy;
use phalcom_semantic::types::store::TypeStore;
use phalcom_semantic::CheckingContext;
use std::collections::{HashMap, HashSet};

fn test_module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(42), ModulePath::root())
}

fn universe_module() -> ModuleId {
    ModuleId::universe_root()
}

fn setup_resolver_and_decls(store: &mut TypeStore, declarations: &mut DeclarationTypeTable) -> SimpleTypeResolver {
    let mut resolver = SimpleTypeResolver::new();
    let u = universe_module();
    for name in ["Int", "Bool", "Float", "String"] {
        let decl = DeclarationId::new(u.clone(), name.into());
        let header = NominalDeclarationHeader::from_signature(store, decl.clone(), None);
        declarations.insert(header.into_type_info(None));
        resolver.insert(name, decl);
    }
    resolver
}

fn register_nominal(
    store: &mut TypeStore,
    declarations: &mut DeclarationTypeTable,
    decl: DeclarationId,
) {
    let header = NominalDeclarationHeader::from_signature(store, decl, None);
    declarations.insert(header.into_type_info(None));
}

#[test]
fn test_primary_vs_impl_duplicate_rejected() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let user_decl = DeclarationId::new(module.clone(), "User".into());
    register_nominal(&mut store, &mut declarations, user_decl.clone());

    let mut resolver = setup_resolver_and_decls(&mut store, &mut declarations);
    resolver.insert("User", user_decl.clone());

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

    let class_src = phalcom_ast::parse("class User {\n  name() -> String { \"a\" }\n}\n", 0);
    assert!(class_src.errors.is_empty());
    let class_def = match &class_src.program.statements[0] {
        phalcom_ast::ast::Statement::Class(c) => c,
        _ => panic!("expected class"),
    };

    let primary_sigs = register_class_surface(&mut ctx, class_def);
    let primary_surface = ctx.dispatch.make_mut().get_surface(&user_decl).unwrap().clone();

    let impl_src = phalcom_ast::parse("impl User {\n  name() -> String { \"b\" }\n}\n", 0);
    assert!(impl_src.errors.is_empty());
    let impl_def = match &impl_src.program.statements[0] {
        phalcom_ast::ast::Statement::Impl(d) => d,
        _ => panic!("expected impl"),
    };

    let impl_id = ImplId::new(module, ImplLocalId(0));
    let contribution = build_inherent_impl_contribution(&mut ctx, &impl_id, impl_def);

    let effective = build_effective_surface(
        &user_decl,
        &primary_surface,
        &primary_sigs,
        &[contribution],
        None,
        None,
    );

    assert!(effective.diagnostics.iter().any(|d| d.code == DiagnosticCode::ImplMemberConflict), "expected conflict diagnostic");
    let sel = Selector::method("name", []).unwrap();
    let callable_id = CallableId::new(user_decl.clone(), sel, DispatchSide::Instance);
    let def = effective.definitions.get(&callable_id).expect("primary definition should survive");
    assert_eq!(def.origin, CallableDefinitionOrigin::PrimaryDeclaration);
}

#[test]
fn test_impl_vs_impl_duplicate_rejected() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let user_decl = DeclarationId::new(module.clone(), "User".into());
    register_nominal(&mut store, &mut declarations, user_decl.clone());

    let mut resolver = setup_resolver_and_decls(&mut store, &mut declarations);
    resolver.insert("User", user_decl.clone());

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

    let primary_surface = DeclarationSurface::new(Some(user_decl.clone()));
    let primary_sigs = HashMap::new();

    let impl1_src = phalcom_ast::parse("impl User {\n  compute() -> Int { 1 }\n}\n", 0);
    let impl1_def = match &impl1_src.program.statements[0] {
        phalcom_ast::ast::Statement::Impl(d) => d,
        _ => panic!("expected impl"),
    };
    let impl1_id = ImplId::new(module.clone(), ImplLocalId(0));
    let contrib1 = build_inherent_impl_contribution(&mut ctx, &impl1_id, impl1_def);

    let impl2_src = phalcom_ast::parse("impl User {\n  compute() -> Int { 2 }\n}\n", 0);
    let impl2_def = match &impl2_src.program.statements[0] {
        phalcom_ast::ast::Statement::Impl(d) => d,
        _ => panic!("expected impl"),
    };
    let impl2_id = ImplId::new(module.clone(), ImplLocalId(1));
    let contrib2 = build_inherent_impl_contribution(&mut ctx, &impl2_id, impl2_def);

    let effective = build_effective_surface(
        &user_decl,
        &primary_surface,
        &primary_sigs,
        &[contrib1, contrib2],
        None,
        None,
    );

    assert!(effective.diagnostics.iter().any(|d| d.code == DiagnosticCode::ImplMemberConflict), "expected conflict diagnostic");
    let sel = Selector::method("compute", []).unwrap();
    let callable_id = CallableId::new(user_decl.clone(), sel, DispatchSide::Instance);
    let def = effective.definitions.get(&callable_id).expect("first impl definition should survive for recovery");
    assert_eq!(def.origin, CallableDefinitionOrigin::InherentImpl(impl1_id));
}

#[test]
fn test_getter_and_setter_distinct_selectors_success() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let user_decl = DeclarationId::new(module.clone(), "User".into());
    register_nominal(&mut store, &mut declarations, user_decl.clone());

    let mut resolver = setup_resolver_and_decls(&mut store, &mut declarations);
    resolver.insert("User", user_decl.clone());

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

    let primary_surface = DeclarationSurface::new(Some(user_decl.clone()));
    let primary_sigs = HashMap::new();

    let impl_src = phalcom_ast::parse("impl User {\n  value -> Int { 0 }\n  value=(_ next: Int) { }\n}\n", 0);
    let impl_def = match &impl_src.program.statements[0] {
        phalcom_ast::ast::Statement::Impl(d) => d,
        _ => panic!("expected impl"),
    };
    let impl_id = ImplId::new(module, ImplLocalId(0));
    let contrib = build_inherent_impl_contribution(&mut ctx, &impl_id, impl_def);

    let effective = build_effective_surface(
        &user_decl,
        &primary_surface,
        &primary_sigs,
        &[contrib],
        None,
        None,
    );

    assert!(effective.diagnostics.is_empty(), "unexpected diagnostics: {:?}", effective.diagnostics);
    assert_eq!(effective.definitions.len(), 2);
    assert!(effective
        .definitions
        .contains_key(&CallableId::new(user_decl.clone(), Selector::getter("value").unwrap(), DispatchSide::Instance)));
    assert!(effective
        .definitions
        .contains_key(&CallableId::new(user_decl, Selector::setter("value").unwrap(), DispatchSide::Instance)));
}

#[test]
fn test_data_component_collision_rejected() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let point_decl = DeclarationId::new(module.clone(), "Point".into());
    register_nominal(&mut store, &mut declarations, point_decl.clone());

    let mut resolver = setup_resolver_and_decls(&mut store, &mut declarations);
    resolver.insert("Point", point_decl.clone());

    let primary_surface = DeclarationSurface::new(Some(point_decl.clone()));
    let primary_sigs = HashMap::new();

    let int_ty = store.nominal(DeclarationId::new(universe_module(), "Int".into()));
    let comp_id = DataComponentId::new(point_decl.clone(), 0);
    let data_comp = DataComponentSemantic {
        id: comp_id.clone(),
        local_name: "x".into(),
        external_label: None,
        declared_type: DeclaredTypeFact::known(
            TypeTerm::Canonical(int_ty),
            DeclaredTypeBasis::SourceAnnotation,
        ),
        source: None,
    };

    let data_info = DataInfo {
        owner: point_decl.clone(),
        root_form: store.nominal(point_decl.clone()),
        generic_signature: None,
        shape: DataShape::Record,
        components: vec![data_comp].into_boxed_slice(),
        constructor: DataConstructorSignature {
            constructor: DataConstructorId::new(point_decl.clone()),
            parameters: Box::new([]),
            result_type_template: store.nominal(point_decl.clone()),
            source: None,
        },
        source: None,
    };

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

    let impl_src = phalcom_ast::parse("impl Point {\n  x -> Int { 42 }\n}\n", 0);
    let impl_def = match &impl_src.program.statements[0] {
        phalcom_ast::ast::Statement::Impl(d) => d,
        _ => panic!("expected impl"),
    };
    let impl_id = ImplId::new(module, ImplLocalId(0));
    let contrib = build_inherent_impl_contribution(&mut ctx, &impl_id, impl_def);

    let effective = build_effective_surface(
        &point_decl,
        &primary_surface,
        &primary_sigs,
        &[contrib],
        Some(&data_info),
        None,
    );

    assert!(effective.diagnostics.iter().any(|d| d.code == DiagnosticCode::ImplMemberConflict), "expected data component conflict diagnostic");
    let sel = Selector::getter("x").unwrap();
    let callable_id = CallableId::new(point_decl.clone(), sel, DispatchSide::Instance);
    assert!(!effective.definitions.contains_key(&callable_id), "rejected duplicate must be absent from definitions");
}

#[test]
fn test_enum_class_side_variant_collision_rejected() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let status_decl = DeclarationId::new(module.clone(), "Status".into());
    register_nominal(&mut store, &mut declarations, status_decl.clone());

    let mut resolver = setup_resolver_and_decls(&mut store, &mut declarations);
    resolver.insert("Status", status_decl.clone());

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

    let primary_surface = DeclarationSurface::new(Some(status_decl.clone()));
    let primary_sigs = HashMap::new();

    let ok_sel = Selector::method("Ok", []).unwrap();
    let variant_id = VariantId::new(status_decl.clone(), ok_sel.clone());
    let (assoc_surface, _) = build_associated_surface(
        &status_decl,
        Some(&[variant_id]),
        &HashSet::new(),
        &HashSet::new(),
        &module,
        None,
    );

    let impl_src = phalcom_ast::parse("impl Status {\n  @class\n  Ok() -> Int { 0 }\n}\n", 0);
    let impl_def = match &impl_src.program.statements[0] {
        phalcom_ast::ast::Statement::Impl(d) => d,
        _ => panic!("expected impl"),
    };
    let impl_id = ImplId::new(module, ImplLocalId(0));
    let contrib = build_inherent_impl_contribution(&mut ctx, &impl_id, impl_def);

    let effective = build_effective_surface(
        &status_decl,
        &primary_surface,
        &primary_sigs,
        &[contrib],
        None,
        Some(&assoc_surface),
    );

    assert!(effective.diagnostics.iter().any(|d| d.code == DiagnosticCode::ImplMemberConflict), "expected variant conflict diagnostic");
    let callable_id = CallableId::new(status_decl.clone(), ok_sel, DispatchSide::Class);
    assert!(!effective.definitions.contains_key(&callable_id), "rejected duplicate must be absent from definitions");
}
