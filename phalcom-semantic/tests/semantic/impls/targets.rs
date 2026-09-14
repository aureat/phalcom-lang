use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::declarations::{DeclarationTypeTable, NominalDeclarationHeader};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_common::selector::Selector;
use phalcom_semantic::enum_semantics::{EnumInfo, EnumSemanticTable, VariantInfo, VariantShape};
use phalcom_semantic::identity::{DeclarationId, ImplId, ImplLocalId, VariantId};
use phalcom_semantic::impls::{InherentImplApplicability, InherentImplTarget, resolve_inherent_impl_target};
use phalcom_semantic::types::annotation::{SimpleTypeResolver, TypeResolver};
use phalcom_semantic::types::id::{KindId, TypeId};
use phalcom_semantic::types::parameter::{GenericSignature, TypeParameterData, TypeParameterOwner};
use phalcom_semantic::types::relation::MapTypeHierarchy;
use phalcom_semantic::types::store::TypeStore;
use phalcom_semantic::CheckingContext;

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
        register_nominal(store, declarations, decl.clone(), None);
        resolver.insert(name, decl);
    }
    resolver
}

fn register_nominal(
    store: &mut TypeStore,
    declarations: &mut DeclarationTypeTable,
    decl: DeclarationId,
    generic_signature: Option<GenericSignature>,
) {
    let header = NominalDeclarationHeader::from_signature(store, decl, generic_signature);
    declarations.insert(header.into_type_info(None));
}

struct AliasResolver {
    base: SimpleTypeResolver,
    alias: DeclarationId,
    form: TypeId,
}

impl TypeResolver for AliasResolver {
    fn resolve_type_name(&self, current_module: &ModuleId, root: &str, members: &[String]) -> Option<DeclarationId> {
        self.base.resolve_type_name(current_module, root, members)
    }

    fn resolve_alias_form(&self, declaration: &DeclarationId) -> Option<TypeId> {
        (declaration == &self.alias).then_some(self.form)
    }
}

#[test]
fn test_same_module_class_target_success() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let user_decl = DeclarationId::new(module.clone(), "User".into());
    register_nominal(&mut store, &mut declarations, user_decl.clone(), None);

    let mut resolver = setup_resolver_and_decls(&mut store, &mut declarations);
    resolver.insert("User", user_decl.clone());

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

    let parsed = phalcom_ast::parse("impl User {\n  is_active() -> Bool { true }\n}\n", 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let impl_stmt = match &parsed.program.statements[0] {
        phalcom_ast::ast::Statement::Impl(def) => def,
        _ => panic!("expected impl"),
    };

    let impl_id = ImplId::new(module, ImplLocalId(0));
    let resolved = resolve_inherent_impl_target(&mut ctx, &impl_id, impl_stmt).expect("target should resolve");
    assert_eq!(resolved.declaration(), &user_decl);
    assert!(matches!(resolved.applicability, InherentImplApplicability::Unconditional));
}

#[test]
fn test_data_target_success() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let point_decl = DeclarationId::new(module.clone(), "Point".into());
    register_nominal(&mut store, &mut declarations, point_decl.clone(), None);

    let mut resolver = setup_resolver_and_decls(&mut store, &mut declarations);
    resolver.insert("Point", point_decl.clone());

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

    let parsed = phalcom_ast::parse("impl Point {\n  norm() -> Int { 0 }\n}\n", 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let impl_stmt = match &parsed.program.statements[0] {
        phalcom_ast::ast::Statement::Impl(def) => def,
        _ => panic!("expected impl"),
    };

    let impl_id = ImplId::new(module, ImplLocalId(0));
    let resolved = resolve_inherent_impl_target(&mut ctx, &impl_id, impl_stmt).expect("target should resolve");
    assert_eq!(resolved.declaration(), &point_decl);
    assert!(matches!(resolved.applicability, InherentImplApplicability::Unconditional));
}

#[test]
fn test_enum_root_target_success() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let status_decl = DeclarationId::new(module.clone(), "Status".into());
    register_nominal(&mut store, &mut declarations, status_decl.clone(), None);

    let mut resolver = setup_resolver_and_decls(&mut store, &mut declarations);
    resolver.insert("Status", status_decl.clone());

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

    let parsed = phalcom_ast::parse("impl Status {\n  is_terminal() -> Bool { false }\n}\n", 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let impl_stmt = match &parsed.program.statements[0] {
        phalcom_ast::ast::Statement::Impl(def) => def,
        _ => panic!("expected impl"),
    };

    let impl_id = ImplId::new(module, ImplLocalId(0));
    let resolved = resolve_inherent_impl_target(&mut ctx, &impl_id, impl_stmt).expect("target should resolve");
    assert_eq!(resolved.declaration(), &status_decl);
    assert!(matches!(resolved.applicability, InherentImplApplicability::Unconditional));
}

#[test]
fn test_generic_covering_target_success_and_bijection_permutation() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let pair_decl = DeclarationId::new(module.clone(), "Pair".into());
    let param_x = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(pair_decl.clone()), 0, "X", KindId::TYPE));
    let param_y = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(pair_decl.clone()), 1, "Y", KindId::TYPE));
    let pair_sig = GenericSignature::new(
        TypeParameterOwner::Declaration(pair_decl.clone()),
        vec![param_x, param_y].into_boxed_slice(),
    );
    register_nominal(&mut store, &mut declarations, pair_decl.clone(), Some(pair_sig));

    let mut resolver = setup_resolver_and_decls(&mut store, &mut declarations);
    resolver.insert("Pair", pair_decl.clone());

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

    let parsed = phalcom_ast::parse("impl<A, B> Pair<B, A> {\n  swap_value() -> Int { 0 }\n}\n", 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let impl_stmt = match &parsed.program.statements[0] {
        phalcom_ast::ast::Statement::Impl(def) => def,
        _ => panic!("expected impl"),
    };

    let impl_id = ImplId::new(module, ImplLocalId(0));
    let resolved = resolve_inherent_impl_target(&mut ctx, &impl_id, impl_stmt).expect("target should resolve");
    assert_eq!(resolved.declaration(), &pair_decl);

    let InherentImplApplicability::Covering(cov) = resolved.applicability else {
        panic!("expected covering applicability");
    };

    let impl_sig = resolved.generic_signature.expect("impl generic sig");
    let impl_param_a = impl_sig.parameters[0];
    let impl_param_b = impl_sig.parameters[1];

    // For impl<A, B> Pair<B, A>:
    // Position 0 of Pair<X, Y> is X -> passed B
    // Position 1 of Pair<X, Y> is Y -> passed A
    assert_eq!(cov.impl_to_decl.get(&impl_param_b), Some(&param_x));
    assert_eq!(cov.impl_to_decl.get(&impl_param_a), Some(&param_y));
    assert_eq!(cov.decl_to_impl.get(&param_x), Some(&impl_param_b));
    assert_eq!(cov.decl_to_impl.get(&param_y), Some(&impl_param_a));
}

#[test]
fn test_foreign_target_rejected() {
    let module = test_module();
    let foreign_module = ModuleId::resolved(ResolvedProjectId::from_raw(99), ModulePath::root());
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let foreign_decl = DeclarationId::new(foreign_module, "ForeignClass".into());
    register_nominal(&mut store, &mut declarations, foreign_decl.clone(), None);

    let mut resolver = setup_resolver_and_decls(&mut store, &mut declarations);
    resolver.insert("ForeignClass", foreign_decl);

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

    let parsed = phalcom_ast::parse("impl ForeignClass {\n  val() -> Int { 0 }\n}\n", 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let impl_stmt = match &parsed.program.statements[0] {
        phalcom_ast::ast::Statement::Impl(def) => def,
        _ => panic!("expected impl"),
    };

    let impl_id = ImplId::new(module, ImplLocalId(0));
    let err = resolve_inherent_impl_target(&mut ctx, &impl_id, impl_stmt).expect_err("should reject foreign target");
    assert!(err.iter().any(|d| d.code == DiagnosticCode::ImplForeignTarget));
}

#[test]
fn test_specialized_repeated_target_rejected() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let pair_decl = DeclarationId::new(module.clone(), "Pair".into());
    let param_x = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(pair_decl.clone()), 0, "X", KindId::TYPE));
    let param_y = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(pair_decl.clone()), 1, "Y", KindId::TYPE));
    let pair_sig = GenericSignature::new(
        TypeParameterOwner::Declaration(pair_decl.clone()),
        vec![param_x, param_y].into_boxed_slice(),
    );
    register_nominal(&mut store, &mut declarations, pair_decl.clone(), Some(pair_sig));

    let mut resolver = setup_resolver_and_decls(&mut store, &mut declarations);
    resolver.insert("Pair", pair_decl);

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

    let parsed = phalcom_ast::parse("impl<T> Pair<T, T> {\n  same() -> Bool { true }\n}\n", 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let impl_stmt = match &parsed.program.statements[0] {
        phalcom_ast::ast::Statement::Impl(def) => def,
        _ => panic!("expected impl"),
    };

    let impl_id = ImplId::new(module, ImplLocalId(0));
    let err = resolve_inherent_impl_target(&mut ctx, &impl_id, impl_stmt).expect_err("should reject repeated parameter");
    assert!(err.iter().any(|d| d.code == DiagnosticCode::ImplSpecializedTargetUnsupported));
}

#[test]
fn test_concrete_specialized_target_rejected() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let box_decl = DeclarationId::new(module.clone(), "Box".into());
    let param_t = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(box_decl.clone()), 0, "T", KindId::TYPE));
    let box_sig = GenericSignature::new(
        TypeParameterOwner::Declaration(box_decl.clone()),
        vec![param_t].into_boxed_slice(),
    );
    register_nominal(&mut store, &mut declarations, box_decl.clone(), Some(box_sig));

    let mut resolver = setup_resolver_and_decls(&mut store, &mut declarations);
    resolver.insert("Box", box_decl);

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

    let parsed = phalcom_ast::parse("impl Box<Int> {\n  int_val() -> Int { 0 }\n}\n", 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let impl_stmt = match &parsed.program.statements[0] {
        phalcom_ast::ast::Statement::Impl(def) => def,
        _ => panic!("expected impl"),
    };

    let impl_id = ImplId::new(module, ImplLocalId(0));
    let err = resolve_inherent_impl_target(&mut ctx, &impl_id, impl_stmt).expect_err("should reject concrete specialization");
    assert!(err.iter().any(|d| d.code == DiagnosticCode::ImplSpecializedTargetUnsupported));
}

#[test]
fn test_unused_type_parameter_rejected() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let box_decl = DeclarationId::new(module.clone(), "Box".into());
    let param_t = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(box_decl.clone()), 0, "T", KindId::TYPE));
    let box_sig = GenericSignature::new(
        TypeParameterOwner::Declaration(box_decl.clone()),
        vec![param_t].into_boxed_slice(),
    );
    register_nominal(&mut store, &mut declarations, box_decl.clone(), Some(box_sig));

    let mut resolver = setup_resolver_and_decls(&mut store, &mut declarations);
    resolver.insert("Box", box_decl);

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

    let parsed = phalcom_ast::parse("impl<T, U> Box<T> {\n  val() -> Int { 0 }\n}\n", 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let impl_stmt = match &parsed.program.statements[0] {
        phalcom_ast::ast::Statement::Impl(def) => def,
        _ => panic!("expected impl"),
    };

    let impl_id = ImplId::new(module, ImplLocalId(0));
    let err = resolve_inherent_impl_target(&mut ctx, &impl_id, impl_stmt).expect_err("should reject unused type parameter");
    assert!(err.iter().any(|d| d.code == DiagnosticCode::ImplUnusedTypeParameter || d.code == DiagnosticCode::ImplSpecializedTargetUnsupported));
}

#[test]
fn test_where_clause_unsupported() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let user_decl = DeclarationId::new(module.clone(), "User".into());
    let param_t = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(user_decl.clone()), 0, "T", KindId::TYPE));
    let user_sig = GenericSignature::new(
        TypeParameterOwner::Declaration(user_decl.clone()),
        vec![param_t].into_boxed_slice(),
    );
    register_nominal(&mut store, &mut declarations, user_decl.clone(), Some(user_sig));

    let mut resolver = setup_resolver_and_decls(&mut store, &mut declarations);
    resolver.insert("User", user_decl);

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

    let parsed = phalcom_ast::parse("impl<T> User<T> where T <: Int {\n  val() -> Int { 0 }\n}\n", 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let impl_stmt = match &parsed.program.statements[0] {
        phalcom_ast::ast::Statement::Impl(def) => def,
        _ => panic!("expected impl"),
    };

    let impl_id = ImplId::new(module, ImplLocalId(0));
    let resolved = resolve_inherent_impl_target(&mut ctx, &impl_id, impl_stmt).expect("target resolves with diagnostic");
    assert!(resolved.diagnostics.iter().any(|d| d.code == DiagnosticCode::ImplWhereClauseUnsupported));
}

#[test]
fn test_non_nominal_target_rejected() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let resolver = setup_resolver_and_decls(&mut store, &mut declarations);
    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

    let parsed = phalcom_ast::parse("impl (Int, String) {\n  val() -> Int { 0 }\n}\n", 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let impl_stmt = match &parsed.program.statements[0] {
        phalcom_ast::ast::Statement::Impl(def) => def,
        _ => panic!("expected impl"),
    };

    let impl_id = ImplId::new(module, ImplLocalId(0));
    let err = resolve_inherent_impl_target(&mut ctx, &impl_id, impl_stmt).expect_err("should reject tuple target");
    assert!(err.iter().any(|d| d.code == DiagnosticCode::ImplTargetNotNominal));
}

#[test]
fn test_type_alias_target_is_rejected() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let user = DeclarationId::new(module.clone(), "User".into());
    let alias = DeclarationId::new(module.clone(), "Alias".into());
    register_nominal(&mut store, &mut declarations, user.clone(), None);

    let mut base = setup_resolver_and_decls(&mut store, &mut declarations);
    base.insert("User", user.clone());
    base.insert("Alias", alias.clone());
    let resolver = AliasResolver {
        base,
        alias: alias.clone(),
        form: store.nominal(user),
    };
    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());
    let alias_program = phalcom_ast::parse("impl Alias { value() -> Int { 0 } }\n", 0);
    let alias_impl = match &alias_program.program.statements[0] {
        phalcom_ast::ast::Statement::Impl(def) => def,
        _ => panic!("expected impl"),
    };
    let alias_error = resolve_inherent_impl_target(&mut ctx, &ImplId::new(module.clone(), ImplLocalId(0)), alias_impl).expect_err("alias target must be rejected");
    assert!(alias_error.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::ImplTargetTypeAlias));
}

#[test]
fn test_exact_enum_case_target_success() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let status_decl = DeclarationId::new(module.clone(), "Status".into());
    register_nominal(&mut store, &mut declarations, status_decl.clone(), None);

    let mut enum_semantics = EnumSemanticTable::new();
    let ready_sel = Selector::getter("Ready").unwrap();
    let ready_var_id = VariantId::new(status_decl.clone(), ready_sel);
    let status_ty = store.nominal(status_decl.clone());
    enum_semantics.insert_enum(std::sync::Arc::new(EnumInfo {
        owner: status_decl.clone(),
        root_form: status_ty,
        generic_signature: None,
        default_result_type: status_ty,
        variants: vec![ready_var_id.clone()].into_boxed_slice(),
        variant_families: vec![].into_boxed_slice(),
        source: None,
    }));
    enum_semantics.insert_variant(std::sync::Arc::new(VariantInfo {
        id: ready_var_id.clone(),
        type_handle: phalcom_semantic::types::id::VariantTypeId::from_index(0),
        family: None,
        shape: VariantShape::Singleton,
        fields: vec![].into_boxed_slice(),
        result_type_template: status_ty,
        exact_case_template: status_ty,
        case_environment: phalcom_semantic::types::case_environment::CaseTypeEnvironment::default(),
        constructor: None,
        visibility: Default::default(),
        source: None,
    }));

    let mut resolver = setup_resolver_and_decls(&mut store, &mut declarations);
    resolver.insert("Status", status_decl.clone());

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());
    ctx.attach_enum_semantics(&enum_semantics);

    let parsed = phalcom_ast::parse("impl Status::Ready {\n  is_ready() -> Bool { true }\n}\n", 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let impl_stmt = match &parsed.program.statements[0] {
        phalcom_ast::ast::Statement::Impl(def) => def,
        _ => panic!("expected impl"),
    };

    let impl_id = ImplId::new(module, ImplLocalId(0));
    let resolved = resolve_inherent_impl_target(&mut ctx, &impl_id, impl_stmt).expect("exact case target should resolve");
    assert_eq!(resolved.target, InherentImplTarget::ExactEnumCase(ready_var_id));
    assert_eq!(resolved.declaration(), &status_decl);
}
