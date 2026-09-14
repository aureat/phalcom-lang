use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::CheckingContext;
use phalcom_semantic::declarations::{DeclarationTypeTable, NominalDeclarationHeader};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::diagnostic::SemanticSourceSpan;
use phalcom_semantic::enum_semantics::{EnumInfo, EnumSemanticTable, VariantInfo, VariantShape};
use phalcom_semantic::identity::{DeclarationId, ImplId, ImplLocalId, VariantId};
use phalcom_semantic::impls::{
    ConformanceTarget, ImplApplicabilityResult, InherentImplApplicability, InherentImplDomain, InherentImplTarget, check_impl_domain_applicability,
    conformance_is_authorized, resolve_conformance_head, resolve_inherent_impl_target,
};
use phalcom_semantic::traits::{TraitHeader, TraitHeaderTable};
use phalcom_semantic::types::annotation::{SimpleTypeResolver, TypeResolver};
use phalcom_semantic::types::evidence::UnknownReason;
use phalcom_semantic::types::id::{KindId, TypeId};
use phalcom_semantic::types::parameter::{GenericConstraint, GenericSignature, TypeParameterData, TypeParameterOwner, TypeTerm};
use phalcom_semantic::types::relation::MapTypeHierarchy;
use phalcom_semantic::types::store::TypeStore;
use std::sync::Arc;

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

fn register_nominal(store: &mut TypeStore, declarations: &mut DeclarationTypeTable, decl: DeclarationId, generic_signature: Option<GenericSignature>) {
    let header = NominalDeclarationHeader::from_signature(store, decl, generic_signature);
    declarations.insert(header.into_type_info(None));
}

fn register_trait_header(headers: &mut TraitHeaderTable, declaration: DeclarationId, generic_signature: Option<GenericSignature>) {
    headers.insert(Arc::new(TraitHeader {
        source: SemanticSourceSpan::new(declaration.module.clone(), (0..0).into()),
        declaration,
        generic_signature,
    }));
}

#[test]
fn test_explicit_conformance_head_resolves_trait_and_target_separately() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();
    let mut resolver = setup_resolver_and_decls(&mut store, &mut declarations);
    let trait_decl = DeclarationId::new(module.clone(), "Converter".into());
    let value_decl = DeclarationId::new(module.clone(), "Value".into());
    let value_param = store.intern_type_parameter(TypeParameterData::new(
        TypeParameterOwner::Declaration(value_decl.clone()),
        0,
        "T",
        KindId::TYPE,
    ));
    let value_signature = GenericSignature::new(TypeParameterOwner::Declaration(value_decl.clone()), vec![value_param].into_boxed_slice());
    register_nominal(&mut store, &mut declarations, value_decl.clone(), Some(value_signature));
    resolver.insert("Converter", trait_decl.clone());
    resolver.insert("Value", value_decl.clone());
    let mut trait_headers = TraitHeaderTable::new();
    let trait_param = store.intern_type_parameter(TypeParameterData::new(
        TypeParameterOwner::Declaration(trait_decl.clone()),
        0,
        "T",
        KindId::TYPE,
    ));
    register_trait_header(
        &mut trait_headers,
        trait_decl.clone(),
        Some(GenericSignature::new(
            TypeParameterOwner::Declaration(trait_decl.clone()),
            vec![trait_param].into_boxed_slice(),
        )),
    );

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());
    let parsed = phalcom_ast::parse("impl<T> Converter<T> for Value<T> {}\n", 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let phalcom_ast::ast::Statement::Impl(impl_def) = &parsed.program.statements[0] else {
        panic!("expected impl")
    };
    let impl_id = ImplId::new(module, ImplLocalId(0));
    let head = resolve_conformance_head(&mut ctx, &trait_headers, &impl_id, impl_def).expect("head should resolve");
    assert!(head.eligible);
    assert_eq!(head.trait_ref.declaration, trait_decl);
    assert!(matches!(head.target, ConformanceTarget::Declaration(declaration) if declaration == value_decl));
    assert_eq!(head.trait_ref.arguments.len(), 1);
    let trait_argument = head.trait_ref.arguments[0];
    let target_argument = match ctx.store.get(head.target_head) {
        phalcom_semantic::types::store::TypeData::Applied { arguments, .. } => arguments[0],
        other => panic!("expected applied target, got {other:?}"),
    };
    assert_eq!(trait_argument, target_argument, "both heads must share the impl-owned parameter");
    assert!(matches!(ctx.store.get(trait_argument), phalcom_semantic::types::store::TypeData::Parameter(_)));
}

#[test]
fn test_explicit_conformance_rejects_non_trait_left_head() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();
    let mut resolver = setup_resolver_and_decls(&mut store, &mut declarations);
    let user_decl = DeclarationId::new(module.clone(), "User".into());
    let target_decl = DeclarationId::new(module.clone(), "Target".into());
    register_nominal(&mut store, &mut declarations, user_decl.clone(), None);
    register_nominal(&mut store, &mut declarations, target_decl.clone(), None);
    resolver.insert("User", user_decl);
    resolver.insert("Target", target_decl);
    let trait_headers = TraitHeaderTable::new();
    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());
    let parsed = phalcom_ast::parse("impl User for Target {}\n", 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let phalcom_ast::ast::Statement::Impl(impl_def) = &parsed.program.statements[0] else {
        panic!("expected impl")
    };
    let error = resolve_conformance_head(&mut ctx, &trait_headers, &ImplId::new(module, ImplLocalId(0)), impl_def)
        .expect_err("ordinary type cannot be a trait reference");
    assert!(error.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::AnnotationUnresolved));
}

#[test]
fn test_conformance_ownership_accepts_only_trait_or_target_owner() {
    let trait_module = ModuleId::resolved(ResolvedProjectId::from_raw(43), ModulePath::root());
    let target_module = ModuleId::resolved(ResolvedProjectId::from_raw(44), ModulePath::root());
    let third_party = ModuleId::resolved(ResolvedProjectId::from_raw(45), ModulePath::root());
    let trait_ref = phalcom_semantic::TraitRef::new(
        DeclarationId::new(trait_module.clone(), "Tagged".into()),
        Vec::<TypeId>::new().into_boxed_slice(),
    );
    let target = ConformanceTarget::Declaration(DeclarationId::new(target_module.clone(), "Value".into()));
    assert!(conformance_is_authorized(&trait_module, &trait_ref, &target));
    assert!(conformance_is_authorized(&target_module, &trait_ref, &target));
    assert!(!conformance_is_authorized(&third_party, &trait_ref, &target));
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
    let pair_sig = GenericSignature::new(TypeParameterOwner::Declaration(pair_decl.clone()), vec![param_x, param_y].into_boxed_slice());
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
fn test_specialized_repeated_target_success() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let pair_decl = DeclarationId::new(module.clone(), "Pair".into());
    let param_x = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(pair_decl.clone()), 0, "X", KindId::TYPE));
    let param_y = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(pair_decl.clone()), 1, "Y", KindId::TYPE));
    let pair_sig = GenericSignature::new(TypeParameterOwner::Declaration(pair_decl.clone()), vec![param_x, param_y].into_boxed_slice());
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
    let resolved = resolve_inherent_impl_target(&mut ctx, &impl_id, impl_stmt).expect("should resolve repeated parameter");
    assert!(resolved.diagnostics.is_empty());
    assert!(matches!(resolved.applicability, InherentImplApplicability::Conditional(_)));
}

#[test]
fn test_concrete_specialized_target_success() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let box_decl = DeclarationId::new(module.clone(), "Box".into());
    let param_t = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(box_decl.clone()), 0, "T", KindId::TYPE));
    let box_sig = GenericSignature::new(TypeParameterOwner::Declaration(box_decl.clone()), vec![param_t].into_boxed_slice());
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
    let resolved = resolve_inherent_impl_target(&mut ctx, &impl_id, impl_stmt).expect("should resolve concrete specialization");
    assert!(resolved.diagnostics.is_empty());
    assert!(matches!(resolved.applicability, InherentImplApplicability::Conditional(_)));
}

#[test]
fn test_unused_type_parameter_rejected() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let box_decl = DeclarationId::new(module.clone(), "Box".into());
    let param_t = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(box_decl.clone()), 0, "T", KindId::TYPE));
    let box_sig = GenericSignature::new(TypeParameterOwner::Declaration(box_decl.clone()), vec![param_t].into_boxed_slice());
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
    assert!(err.iter().any(|d| d.code == DiagnosticCode::ImplUnusedTypeParameter));
}

#[test]
fn test_where_clause_success() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let user_decl = DeclarationId::new(module.clone(), "User".into());
    let param_t = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(user_decl.clone()), 0, "T", KindId::TYPE));
    let user_sig = GenericSignature::new(TypeParameterOwner::Declaration(user_decl.clone()), vec![param_t].into_boxed_slice());
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
    let resolved = resolve_inherent_impl_target(&mut ctx, &impl_id, impl_stmt).expect("target resolves successfully");
    assert!(resolved.diagnostics.is_empty());
    assert!(matches!(resolved.applicability, InherentImplApplicability::Conditional(_)));
}

#[test]
fn test_unknown_constraint_is_not_treated_as_disproven() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let owner = DeclarationId::new(module.clone(), "Box".into());
    register_nominal(&mut store, &mut declarations, owner.clone(), None);

    let impl_id = ImplId::new(module.clone(), ImplLocalId(0));
    let impl_parameter = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Impl(impl_id.clone()), 0, "T", KindId::TYPE));
    let owner_parameter = store.intern_type_parameter(TypeParameterData::new(TypeParameterOwner::Declaration(owner.clone()), 0, "T", KindId::TYPE));
    let impl_parameter_type = store.parameter_form(impl_parameter);
    let owner_parameter_type = store.parameter_form(owner_parameter);
    let int = store.nominal(DeclarationId::new(universe_module(), "Int".into()));

    let domain = InherentImplDomain {
        impl_id: impl_id.clone(),
        target: InherentImplTarget::Declaration(owner),
        head_type: impl_parameter_type,
        generic_signature: None,
        constraints: vec![GenericConstraint::Subtype {
            lower: TypeTerm::Canonical(impl_parameter_type),
            upper: TypeTerm::Canonical(int),
        }]
        .into_boxed_slice(),
    };

    let result = check_impl_domain_applicability(&mut store, &hierarchy, &domain, owner_parameter_type, owner_parameter_type, &[]);
    assert!(matches!(result, ImplApplicabilityResult::Unknown(UnknownReason::UnderconstrainedTypeVariable)));
}

#[test]
fn test_invalid_constraint_is_blocked_instead_of_disproven() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();

    let owner = DeclarationId::new(module.clone(), "Box".into());
    register_nominal(&mut store, &mut declarations, owner.clone(), None);
    let impl_id = ImplId::new(module.clone(), ImplLocalId(0));
    let owner_type = store.nominal(owner.clone());
    let int = store.nominal(DeclarationId::new(universe_module(), "Int".into()));

    let domain = InherentImplDomain {
        impl_id,
        target: InherentImplTarget::Declaration(owner),
        head_type: owner_type,
        generic_signature: None,
        constraints: vec![GenericConstraint::Equivalent {
            left: TypeTerm::Infer(phalcom_semantic::types::id::InferVarId(0)),
            right: TypeTerm::Canonical(int),
        }]
        .into_boxed_slice(),
    };

    let result = check_impl_domain_applicability(&mut store, &hierarchy, &domain, owner_type, owner_type, &[]);
    assert!(matches!(
        result,
        ImplApplicabilityResult::Blocked(phalcom_semantic::types::outcome::BlockReason::InvalidAnnotation(
            DiagnosticCode::AnnotationUnresolved
        ))
    ));
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
    let alias_error =
        resolve_inherent_impl_target(&mut ctx, &ImplId::new(module.clone(), ImplLocalId(0)), alias_impl).expect_err("alias target must be rejected");
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
