use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_semantic::CheckingContext;
use phalcom_semantic::declarations::{DeclarationSemanticForm, DeclarationTypeTable, IntrinsicTypeApplication, NominalDeclarationHeader};
use phalcom_semantic::diagnostic::DiagnosticCode;
use phalcom_semantic::diagnostic::SemanticSourceSpan;
use phalcom_semantic::enum_semantics::{EnumInfo, EnumSemanticTable, VariantInfo, VariantShape};
use phalcom_semantic::identity::{DeclarationId, ImplId, ImplLocalId, VariantId};
use phalcom_semantic::impls::{
    ConformanceTarget, ImplApplicabilityResult, InherentImplApplicability, InherentImplDomain, InherentImplTarget, build_effective_surface,
    build_inherent_impl_contribution, check_impl_domain_applicability, conformance_is_authorized, resolve_conformance_head, resolve_inherent_impl_target,
};
use phalcom_semantic::surface::DeclarationSurface;
use phalcom_semantic::traits::{TraitHeader, TraitHeaderTable};
use phalcom_semantic::types::annotation::{SimpleTypeResolver, TypeResolver};
use phalcom_semantic::types::evidence::UnknownReason;
use phalcom_semantic::types::id::{KindId, TypeId};
use phalcom_semantic::types::parameter::{GenericConstraint, GenericParameterMode, GenericSignature, TypeParameterData, TypeParameterOwner, TypeTerm};
use phalcom_semantic::types::relation::MapTypeHierarchy;
use phalcom_semantic::types::store::{StaticArgument, TypeData, TypeStore};
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

struct StructuralFamilySpec<'a> {
    family_name: &'a str,
    family_owner: IntrinsicTypeApplication,
    parameter_name: &'a str,
    parameter_kind: KindId,
    parameter_mode: GenericParameterMode,
    kind_name: &'a str,
}

fn register_structural_family(
    store: &mut TypeStore,
    declarations: &mut DeclarationTypeTable,
    resolver: &mut SimpleTypeResolver,
    spec: StructuralFamilySpec<'_>,
) -> DeclarationId {
    let StructuralFamilySpec {
        family_name,
        family_owner,
        parameter_name,
        parameter_kind,
        parameter_mode,
        kind_name,
    } = spec;
    let family = match family_name {
        "Tuple" => phalcom_semantic::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Tuple),
        "Record" => phalcom_semantic::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Record),
        _ => unreachable!("only canonical structural families are registered here"),
    };
    let parameter = store.intern_type_parameter(
        TypeParameterData::new(TypeParameterOwner::Declaration(family.clone()), 0, parameter_name, parameter_kind).with_mode(parameter_mode),
    );
    let signature = GenericSignature::new(TypeParameterOwner::Declaration(family.clone()), vec![parameter].into_boxed_slice())
        .with_parameter_metadata(
            vec![parameter_kind].into_boxed_slice(),
            vec![phalcom_semantic::types::variance::Variance::Invariant].into_boxed_slice(),
        )
        .with_parameter_modes(vec![parameter_mode].into_boxed_slice());
    register_nominal(store, declarations, family.clone(), Some(signature));
    declarations.register_intrinsic_semantic_form(&family, DeclarationSemanticForm::Intrinsic(family_owner));
    resolver.insert(family_name, family.clone());

    let kind_anchor = DeclarationId::new(family.module.clone(), kind_name.into());
    declarations.register_intrinsic_static_domain(kind_anchor.clone(), parameter_kind);
    resolver.insert(kind_name, kind_anchor);
    family
}

fn register_tuple_family(store: &mut TypeStore, declarations: &mut DeclarationTypeTable, resolver: &mut SimpleTypeResolver) -> DeclarationId {
    register_structural_family(
        store,
        declarations,
        resolver,
        StructuralFamilySpec {
            family_name: "Tuple",
            family_owner: IntrinsicTypeApplication::Tuple,
            parameter_name: "Es",
            parameter_kind: KindId::COMPLETE_TYPE_PACK,
            parameter_mode: GenericParameterMode::CompletePack,
            kind_name: "CompleteTypePack",
        },
    )
}

fn register_record_family(store: &mut TypeStore, declarations: &mut DeclarationTypeTable, resolver: &mut SimpleTypeResolver) -> DeclarationId {
    register_structural_family(
        store,
        declarations,
        resolver,
        StructuralFamilySpec {
            family_name: "Record",
            family_owner: IntrinsicTypeApplication::Record,
            parameter_name: "Fs",
            parameter_kind: KindId::RECORD_ROW,
            parameter_mode: GenericParameterMode::LabeledPack,
            kind_name: "RecordFields",
        },
    )
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
    let head = resolve_conformance_head(&mut ctx, &trait_headers, &impl_id, impl_def)
        .expect("head should resolve")
        .expect_nominal();
    assert!(head.eligible);
    assert_eq!(head.trait_ref.declaration, trait_decl);
    assert!(matches!(head.target, ConformanceTarget::Declaration(declaration) if declaration == value_decl));
    assert_eq!(head.trait_ref.arguments.len(), 1);
    let trait_argument = head.trait_ref.arguments[0];
    let target_argument = match ctx.store.get(head.target_head) {
        phalcom_semantic::types::store::TypeData::Applied { arguments, .. } => arguments[0],
        other => panic!("expected applied target, got {other:?}"),
    };
    assert_eq!(
        trait_argument,
        target_argument.as_type_id().expect("type-only target argument"),
        "both heads must share the impl-owned parameter"
    );
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
fn test_explicit_conformance_rejects_violated_trait_argument_constraint() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();
    let mut resolver = setup_resolver_and_decls(&mut store, &mut declarations);

    let number = DeclarationId::new(universe_module(), "Number".into());
    register_nominal(&mut store, &mut declarations, number.clone(), None);
    resolver.insert("Number", number.clone());
    let item = DeclarationId::new(module.clone(), "Item".into());
    register_nominal(&mut store, &mut declarations, item.clone(), None);
    resolver.insert("Item", item);
    let trait_decl = DeclarationId::new(module.clone(), "NumericTag".into());
    resolver.insert("NumericTag", trait_decl.clone());

    let mut headers = TraitHeaderTable::new();
    let trait_param = store.intern_type_parameter(TypeParameterData::new(
        TypeParameterOwner::Declaration(trait_decl.clone()),
        0,
        "T",
        KindId::TYPE,
    ));
    let number_type = store.nominal(number);
    headers.insert(Arc::new(TraitHeader {
        source: SemanticSourceSpan::new(module.clone(), (0..0).into()),
        declaration: trait_decl,
        generic_signature: Some(GenericSignature::with_constraints(
            TypeParameterOwner::Declaration(DeclarationId::new(module.clone(), "NumericTag".into())),
            vec![trait_param].into_boxed_slice(),
            vec![GenericConstraint::Subtype {
                lower: TypeTerm::Canonical(store.parameter_form(trait_param)),
                upper: TypeTerm::Canonical(number_type),
            }]
            .into_boxed_slice(),
        )),
    }));

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());
    let parsed = phalcom_ast::parse("impl NumericTag<String> for Item {}\n", 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let phalcom_ast::ast::Statement::Impl(impl_def) = &parsed.program.statements[0] else {
        panic!("expected impl")
    };
    let result = resolve_conformance_head(&mut ctx, &headers, &ImplId::new(module, ImplLocalId(0)), impl_def);
    assert!(result.is_err(), "violated trait argument constraint must reject conformance head");
    assert!(
        result
            .expect_err("constraint violation")
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::AnnotationUnresolved)
    );
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
fn tuple_structural_impls_use_canonical_owner_and_equivalent_spellings() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();
    let mut resolver = setup_resolver_and_decls(&mut store, &mut declarations);
    let tuple_decl = register_tuple_family(&mut store, &mut declarations, &mut resolver);
    let module = tuple_decl.module.clone();
    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

    let covering = phalcom_ast::parse("impl<***Es: CompleteTypePack> Tuple<***Es> {}\n", 0);
    assert!(covering.errors.is_empty(), "parse errors: {:?}", covering.errors);
    let phalcom_ast::ast::Statement::Impl(covering_impl) = &covering.program.statements[0] else {
        panic!("expected impl")
    };
    let covering_target = resolve_inherent_impl_target(&mut ctx, &ImplId::new(module.clone(), ImplLocalId(0)), covering_impl)
        .expect("canonical Tuple family impl should resolve");
    assert_eq!(covering_target.declaration(), &tuple_decl);
    let InherentImplApplicability::Covering(substitution) = covering_target.applicability else {
        panic!("the complete Tuple family head should be covering")
    };
    assert_eq!(substitution.impl_to_decl.len(), 1);

    let direct = phalcom_ast::parse("impl (Int, String) {}\n", 0);
    assert!(direct.errors.is_empty(), "parse errors: {:?}", direct.errors);
    let phalcom_ast::ast::Statement::Impl(direct_impl) = &direct.program.statements[0] else {
        panic!("expected impl")
    };
    let direct_target = resolve_inherent_impl_target(&mut ctx, &ImplId::new(module.clone(), ImplLocalId(1)), direct_impl)
        .expect("structural Tuple domain should resolve in its owner module");
    assert_eq!(direct_target.declaration(), &tuple_decl);
    assert!(matches!(direct_target.applicability, InherentImplApplicability::Conditional(_)));

    let repeated = phalcom_ast::parse("impl<T> (T, T) {}\n", 0);
    assert!(repeated.errors.is_empty(), "parse errors: {:?}", repeated.errors);
    let phalcom_ast::ast::Statement::Impl(repeated_impl) = &repeated.program.statements[0] else {
        panic!("expected impl")
    };
    let repeated_target = resolve_inherent_impl_target(&mut ctx, &ImplId::new(module.clone(), ImplLocalId(3)), repeated_impl)
        .expect("specialized structural Tuple domain should resolve");
    assert_eq!(repeated_target.declaration(), &tuple_decl);
    assert!(matches!(repeated_target.applicability, InherentImplApplicability::Conditional(_)));

    {
        let (impl_index, source) = (2, "impl Tuple<***(Int, String)> {}\n");
        let parsed = phalcom_ast::parse(source, 0);
        assert!(parsed.errors.is_empty(), "parse errors for {source:?}: {:?}", parsed.errors);
        let phalcom_ast::ast::Statement::Impl(impl_stmt) = &parsed.program.statements[0] else {
            panic!("expected impl")
        };
        let resolved = resolve_inherent_impl_target(&mut ctx, &ImplId::new(module.clone(), ImplLocalId(impl_index)), impl_stmt)
            .expect("equivalent Tuple spelling should resolve");
        assert_eq!(resolved.declaration(), &tuple_decl);
        assert_eq!(resolved.target_type, direct_target.target_type, "all spellings share one canonical Tuple type");
    }
}

#[test]
fn record_structural_impls_use_canonical_owner_and_equivalent_spellings() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();
    let mut resolver = setup_resolver_and_decls(&mut store, &mut declarations);
    let record_decl = register_record_family(&mut store, &mut declarations, &mut resolver);
    let module = record_decl.module.clone();
    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

    let covering = phalcom_ast::parse("impl<**Fs: RecordFields> Record<**Fs> {}\n", 0);
    assert!(covering.errors.is_empty(), "parse errors: {:?}", covering.errors);
    let phalcom_ast::ast::Statement::Impl(covering_impl) = &covering.program.statements[0] else {
        panic!("expected impl")
    };
    let covering_target = resolve_inherent_impl_target(&mut ctx, &ImplId::new(module.clone(), ImplLocalId(0)), covering_impl)
        .expect("canonical Record family impl should resolve");
    assert_eq!(covering_target.declaration(), &record_decl);
    let InherentImplApplicability::Covering(substitution) = covering_target.applicability else {
        panic!("the RecordFields family head should be covering")
    };
    assert_eq!(substitution.impl_to_decl.len(), 1);
    let implementation_parameter = *substitution.impl_to_decl.keys().next().expect("RecordFields impl binder");
    let declaration_parameter = substitution.impl_to_decl[&implementation_parameter];
    let instantiation = substitution.to_instantiation(ctx.store).expect("domain-aware RecordFields instantiation");
    assert!(
        instantiation.type_binding(implementation_parameter).is_none(),
        "RecordFields must remain outside the proper TypeId binding domain"
    );
    let row_binding = instantiation.row_binding(implementation_parameter).expect("RecordFields row binding");
    assert_eq!(
        ctx.store.record_row(row_binding).tail,
        phalcom_semantic::types::row::RecordRowTail::Parameter(declaration_parameter),
        "row-aware materialization must preserve the mapped declaration parameter"
    );
    let TypeData::Record(target_row) = ctx.store.get(covering_target.target_type) else {
        panic!("canonical structural Record family application should materialize as a Record row")
    };
    assert_eq!(
        ctx.store.record_row(*target_row).tail,
        phalcom_semantic::types::row::RecordRowTail::Parameter(implementation_parameter),
        "the selected structural target should retain the implementation row binder"
    );

    let direct = phalcom_ast::parse("impl { name: String } {}\n", 0);
    assert!(direct.errors.is_empty(), "parse errors: {:?}", direct.errors);
    let phalcom_ast::ast::Statement::Impl(direct_impl) = &direct.program.statements[0] else {
        panic!("expected impl")
    };
    let direct_target = resolve_inherent_impl_target(&mut ctx, &ImplId::new(module.clone(), ImplLocalId(1)), direct_impl)
        .expect("structural Record domain should resolve in its owner module");
    assert_eq!(direct_target.declaration(), &record_decl);
    assert!(matches!(direct_target.applicability, InherentImplApplicability::Conditional(_)));

    let generic = phalcom_ast::parse("impl<T> { value: T } {}\n", 0);
    assert!(generic.errors.is_empty(), "parse errors: {:?}", generic.errors);
    let phalcom_ast::ast::Statement::Impl(generic_impl) = &generic.program.statements[0] else {
        panic!("expected impl")
    };
    let generic_target = resolve_inherent_impl_target(&mut ctx, &ImplId::new(module.clone(), ImplLocalId(3)), generic_impl)
        .expect("specialized structural Record domain should resolve");
    assert_eq!(generic_target.declaration(), &record_decl);
    assert!(matches!(generic_target.applicability, InherentImplApplicability::Conditional(_)));

    {
        let (impl_index, source) = (2, "impl Record<**{ name: String }> {}\n");
        let parsed = phalcom_ast::parse(source, 0);
        assert!(parsed.errors.is_empty(), "parse errors for {source:?}: {:?}", parsed.errors);
        let phalcom_ast::ast::Statement::Impl(impl_stmt) = &parsed.program.statements[0] else {
            panic!("expected impl")
        };
        let resolved = resolve_inherent_impl_target(&mut ctx, &ImplId::new(module.clone(), ImplLocalId(impl_index)), impl_stmt)
            .expect("equivalent Record spelling should resolve");
        assert_eq!(resolved.declaration(), &record_decl);
        assert_eq!(resolved.target_type, direct_target.target_type, "all spellings share one canonical Record type");
    }
}

#[test]
fn foreign_modules_cannot_author_structural_tuple_or_record_impls() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();
    let resolver = setup_resolver_and_decls(&mut store, &mut declarations);
    let mut resolver = resolver;
    let _tuple_decl = register_tuple_family(&mut store, &mut declarations, &mut resolver);
    let _record_decl = register_record_family(&mut store, &mut declarations, &mut resolver);
    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

    for (impl_index, source) in [
        (0, "impl (Int, String) {}\n"),
        (1, "impl { name: String } {}\n"),
        (2, "impl Tuple<***(Int, String)> {}\n"),
        (3, "impl Record<**{ name: String }> {}\n"),
    ] {
        let parsed = phalcom_ast::parse(source, 0);
        assert!(parsed.errors.is_empty(), "parse errors for {source:?}: {:?}", parsed.errors);
        let phalcom_ast::ast::Statement::Impl(impl_stmt) = &parsed.program.statements[0] else {
            panic!("expected impl")
        };
        let errors = resolve_inherent_impl_target(&mut ctx, &ImplId::new(module.clone(), ImplLocalId(impl_index)), impl_stmt)
            .expect_err("foreign structural behavior owner must be rejected");
        assert!(errors.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::ImplForeignTarget));
        assert!(!errors.iter().any(|diagnostic| diagnostic.code == DiagnosticCode::ImplTargetNotNominal));
    }
}

#[test]
fn row_parameter_covering_impl_materializes_through_record_row_identity() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();
    let mut resolver = setup_resolver_and_decls(&mut store, &mut declarations);
    register_record_family(&mut store, &mut declarations, &mut resolver);

    let row_product = DeclarationId::new(module.clone(), "RowProduct".into());
    let declaration_parameter = store.intern_type_parameter(
        TypeParameterData::new(TypeParameterOwner::Declaration(row_product.clone()), 0, "Fs", KindId::RECORD_ROW).with_mode(GenericParameterMode::LabeledPack),
    );
    let signature = GenericSignature::new(
        TypeParameterOwner::Declaration(row_product.clone()),
        vec![declaration_parameter].into_boxed_slice(),
    )
    .with_parameter_metadata(
        vec![KindId::RECORD_ROW].into_boxed_slice(),
        vec![phalcom_semantic::types::variance::Variance::Invariant].into_boxed_slice(),
    )
    .with_parameter_modes(vec![GenericParameterMode::LabeledPack].into_boxed_slice());
    register_nominal(&mut store, &mut declarations, row_product.clone(), Some(signature));
    resolver.insert("RowProduct", row_product.clone());

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());
    let parsed = phalcom_ast::parse("impl<**Fs: RecordFields> RowProduct<**Fs> {}\n", 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let phalcom_ast::ast::Statement::Impl(implementation) = &parsed.program.statements[0] else {
        panic!("expected impl")
    };
    let resolved = resolve_inherent_impl_target(&mut ctx, &ImplId::new(module, ImplLocalId(0)), implementation)
        .expect("RecordFields family head should resolve as a covering nominal impl");
    assert_eq!(resolved.declaration(), &row_product);
    let InherentImplApplicability::Covering(substitution) = resolved.applicability else {
        panic!("the complete RowProduct family head should be covering")
    };
    let implementation_parameter = *substitution.impl_to_decl.keys().next().expect("row binder correspondence");
    assert_eq!(substitution.impl_to_decl.get(&implementation_parameter), Some(&declaration_parameter));

    let instantiation = substitution.to_instantiation(ctx.store).expect("domain-aware row instantiation");
    assert!(
        instantiation.type_binding(implementation_parameter).is_none(),
        "RecordFields must not become a TypeId binding"
    );
    let row = instantiation.row_binding(implementation_parameter).expect("RecordFields row binding");
    let TypeData::Applied { arguments, .. } = ctx.store.get(resolved.target_type) else {
        panic!("RowProduct target should retain its static application")
    };
    let StaticArgument::RecordRow(target_row) = arguments[0] else {
        panic!("the RowProduct argument must remain a RecordRowId")
    };
    assert_eq!(
        ctx.store.record_row(target_row).tail,
        phalcom_semantic::types::row::RecordRowTail::Parameter(implementation_parameter)
    );
    assert!(ctx.store.record_row(row).fields.is_empty());
    assert_eq!(
        ctx.store.record_row(row).tail,
        phalcom_semantic::types::row::RecordRowTail::Parameter(declaration_parameter)
    );
}

#[test]
fn type_alias_to_structural_family_uses_its_canonical_behavior_owner() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();
    let mut base = setup_resolver_and_decls(&mut store, &mut declarations);
    let family = register_tuple_family(&mut store, &mut declarations, &mut base);
    let alias = DeclarationId::new(family.module.clone(), "TupleAlias".into());
    let empty_tuple = store.tuple(Box::new([]));
    base.insert("TupleAlias", alias.clone());
    let aliases = AliasResolver {
        base,
        alias: alias.clone(),
        form: empty_tuple,
    };
    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &aliases, &declarations, family.module.clone());
    let parsed = phalcom_ast::parse("impl TupleAlias {}\n", 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let phalcom_ast::ast::Statement::Impl(impl_stmt) = &parsed.program.statements[0] else {
        panic!("expected impl")
    };
    let resolved = resolve_inherent_impl_target(&mut ctx, &ImplId::new(family.module.clone(), ImplLocalId(0)), impl_stmt)
        .expect("structural aliases share the canonical family behavior owner");
    assert_eq!(resolved.declaration(), &family);
    assert_eq!(resolved.target_type, empty_tuple);
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
        static_data: None,
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
        payload_kind: phalcom_semantic::enum_semantics::AlgebraicCasePayloadKind::Singleton,
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

#[test]
fn product_parameter_covering_impl_materializes_through_complete_type_pack() {
    let module = test_module();
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();
    let mut resolver = setup_resolver_and_decls(&mut store, &mut declarations);
    register_tuple_family(&mut store, &mut declarations, &mut resolver);

    let product = DeclarationId::new(module.clone(), "Product".into());
    let declaration_parameter = store.intern_type_parameter(
        TypeParameterData::new(TypeParameterOwner::Declaration(product.clone()), 0, "Es", KindId::COMPLETE_TYPE_PACK)
            .with_mode(GenericParameterMode::CompletePack),
    );
    let signature = GenericSignature::new(TypeParameterOwner::Declaration(product.clone()), vec![declaration_parameter].into_boxed_slice())
        .with_parameter_metadata(
            vec![KindId::COMPLETE_TYPE_PACK].into_boxed_slice(),
            vec![phalcom_semantic::types::variance::Variance::Invariant].into_boxed_slice(),
        )
        .with_parameter_modes(vec![GenericParameterMode::CompletePack].into_boxed_slice());
    register_nominal(&mut store, &mut declarations, product.clone(), Some(signature));
    resolver.insert("Product", product.clone());

    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());
    let parsed = phalcom_ast::parse("impl<***Es: CompleteTypePack> Product<***Es> {}\n", 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
    let phalcom_ast::ast::Statement::Impl(implementation) = &parsed.program.statements[0] else {
        panic!("expected impl")
    };
    let resolved = resolve_inherent_impl_target(&mut ctx, &ImplId::new(module, ImplLocalId(0)), implementation)
        .expect("CompleteTypePack family head should resolve as a covering nominal impl");
    assert_eq!(resolved.declaration(), &product);
    let InherentImplApplicability::Covering(substitution) = resolved.applicability else {
        panic!("the complete Product family head should be covering")
    };
    let implementation_parameter = *substitution.impl_to_decl.keys().next().expect("pack binder correspondence");
    assert_eq!(substitution.impl_to_decl.get(&implementation_parameter), Some(&declaration_parameter));
}

#[test]
fn structural_impl_equivalent_spellings_produce_identical_targets() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();
    let mut resolver = setup_resolver_and_decls(&mut store, &mut declarations);
    let tuple_decl = register_tuple_family(&mut store, &mut declarations, &mut resolver);
    let record_decl = register_record_family(&mut store, &mut declarations, &mut resolver);
    let module = tuple_decl.module.clone();
    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

    // 1. Tuple direct vs generic application vs explicit pack spread
    let tuple_direct = resolve_inherent_impl_target(
        &mut ctx,
        &ImplId::new(module.clone(), ImplLocalId(0)),
        match &phalcom_ast::parse("impl (Int, String) {}\n", 0).program.statements[0] {
            phalcom_ast::ast::Statement::Impl(d) => d,
            _ => panic!(),
        },
    )
    .expect("tuple direct");

    let tuple_spread = resolve_inherent_impl_target(
        &mut ctx,
        &ImplId::new(module.clone(), ImplLocalId(1)),
        match &phalcom_ast::parse("impl Tuple<***(Int, String)> {}\n", 0).program.statements[0] {
            phalcom_ast::ast::Statement::Impl(d) => d,
            _ => panic!(),
        },
    )
    .expect("tuple spread");

    assert_eq!(tuple_direct.declaration(), &tuple_decl);
    assert_eq!(tuple_spread.declaration(), &tuple_decl);
    assert_eq!(tuple_direct.target_type, tuple_spread.target_type);

    // 2. Record direct vs explicit row spread
    let record_module = record_decl.module.clone();
    let mut record_ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, record_module.clone());
    let record_direct = resolve_inherent_impl_target(
        &mut record_ctx,
        &ImplId::new(record_module.clone(), ImplLocalId(0)),
        match &phalcom_ast::parse("impl { name: String } {}\n", 0).program.statements[0] {
            phalcom_ast::ast::Statement::Impl(d) => d,
            _ => panic!(),
        },
    )
    .expect("record direct");

    let record_spread = resolve_inherent_impl_target(
        &mut record_ctx,
        &ImplId::new(record_module.clone(), ImplLocalId(1)),
        match &phalcom_ast::parse("impl Record<**{ name: String }> {}\n", 0).program.statements[0] {
            phalcom_ast::ast::Statement::Impl(d) => d,
            _ => panic!(),
        },
    )
    .expect("record spread");

    assert_eq!(record_direct.declaration(), &record_decl);
    assert_eq!(record_spread.declaration(), &record_decl);
    assert_eq!(record_direct.target_type, record_spread.target_type);
}

#[test]
fn equivalent_structural_impl_spellings_reach_the_same_member_collision_policy() {
    let mut store = TypeStore::new();
    let hierarchy = MapTypeHierarchy::new();
    let mut declarations = DeclarationTypeTable::new();
    let mut resolver = setup_resolver_and_decls(&mut store, &mut declarations);
    let tuple_decl = register_tuple_family(&mut store, &mut declarations, &mut resolver);
    let record_decl = register_record_family(&mut store, &mut declarations, &mut resolver);
    let module = tuple_decl.module.clone();
    let mut ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, module.clone());

    let mut contributions = Vec::new();
    for (local_id, source) in [
        (0, "impl (Int, String) { marker() -> Int { 0 } }\n"),
        (1, "impl Tuple<***(Int, String)> { marker() -> Int { 1 } }\n"),
    ] {
        let parsed = phalcom_ast::parse(source, 0);
        assert!(parsed.errors.is_empty(), "parse errors for {source:?}: {:?}", parsed.errors);
        let phalcom_ast::ast::Statement::Impl(impl_def) = &parsed.program.statements[0] else {
            panic!("expected impl")
        };
        let contribution = build_inherent_impl_contribution(&mut ctx, &ImplId::new(module.clone(), ImplLocalId(local_id)), impl_def);
        assert!(contribution.diagnostics.is_empty(), "impl diagnostics: {:?}", contribution.diagnostics);
        assert_eq!(contribution.target.declaration(), &tuple_decl);
        contributions.push(contribution);
    }

    let primary_surface = DeclarationSurface::new(Some(tuple_decl.clone()));
    let effective = build_effective_surface(&tuple_decl, &primary_surface, &Default::default(), &contributions, None, None);
    assert!(
        effective
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::ImplMemberConflict),
        "equivalent direct/family spellings must reach ordinary same-owner member collision handling: {:?}",
        effective.diagnostics
    );

    drop(ctx);
    let record_module = record_decl.module.clone();
    let mut record_ctx = CheckingContext::new(&mut store, &hierarchy, &resolver, &declarations, record_module.clone());
    let mut record_contributions = Vec::new();
    for (local_id, source) in [
        (2, "impl { name: String } { marker() -> Int { 0 } }\n"),
        (3, "impl Record<**{ name: String }> { marker() -> Int { 1 } }\n"),
    ] {
        let parsed = phalcom_ast::parse(source, 0);
        assert!(parsed.errors.is_empty(), "parse errors for {source:?}: {:?}", parsed.errors);
        let phalcom_ast::ast::Statement::Impl(impl_def) = &parsed.program.statements[0] else {
            panic!("expected impl")
        };
        let contribution = build_inherent_impl_contribution(&mut record_ctx, &ImplId::new(record_module.clone(), ImplLocalId(local_id)), impl_def);
        assert!(contribution.diagnostics.is_empty(), "impl diagnostics: {:?}", contribution.diagnostics);
        assert_eq!(contribution.target.declaration(), &record_decl);
        record_contributions.push(contribution);
    }

    let primary_surface = DeclarationSurface::new(Some(record_decl.clone()));
    let effective = build_effective_surface(&record_decl, &primary_surface, &Default::default(), &record_contributions, None, None);
    assert!(
        effective
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::ImplMemberConflict),
        "equivalent direct/spread Record spellings must reach ordinary same-owner member collision handling: {:?}",
        effective.diagnostics
    );
}
