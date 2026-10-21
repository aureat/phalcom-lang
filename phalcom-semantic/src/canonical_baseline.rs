use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, OnceLock};

use phalcom_ast::ast::{Statement, TraitMember};
use phalcom_modules::ModuleId;
use phalcom_modules::linker::LinkedProgram;

use crate::associated::{AssociatedFamilyTable, AssociatedSurface, build_associated_surface};
use crate::checker::context::CheckingContext;
use crate::data_semantics::DataSemanticTable;
use crate::db::product::{DataDeclarationProduct, EnumDeclarationProduct, EnumRequirementsProduct, TypeMemberDeclarationProduct};
use crate::declarations::{DeclarationTypeInfo, DeclarationTypeTable, GenericSupertypeTemplate, bootstrap_universe_declarations};
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::dispatch::{DispatchSide, SurfaceDispatchResolver};
use crate::enum_requirements::{EnumRequirementTable, check_enum_requirements};
use crate::enum_semantics::{EnumSemanticTable, VariantInfo};
use crate::identity::{DeclarationId, WorkspaceId};
use crate::resolver::LinkedTypeResolver;
use crate::session::{SemanticWorkspaceSession, canonical_universe_linked_program, implementation_owner, ready_kind_for_predeclaration};
use crate::signature::CallableSignatureTable;
use crate::types::id::KindId;
use crate::types::native::register_native_surfaces;
use crate::types::parameter::{GenericBinderHeader, TypeParameterData, TypeParameterOwner};
use crate::types::relation::MapTypeHierarchy;
use crate::types::store::TypeStore;
use crate::types::type_member::TypeMemberDeclarationId;

/// Process-shared, precomputed semantic products derived from the canonical Universe source.
///
/// Constructed at most once per process and reused across all workspace sessions,
/// eliminating repeated cold Universe semantic analysis.
#[derive(Clone, Debug)]
pub struct CanonicalUniverseSemanticBaseline {
    pub(crate) provider: phalcom_modules::UniverseSourceProvider,
    pub(crate) linked: Arc<LinkedProgram>,
    pub(crate) store: TypeStore,
    pub(crate) base_declarations: DeclarationTypeTable,
    pub(crate) base_hierarchy: MapTypeHierarchy,
    pub(crate) base_dispatch: SurfaceDispatchResolver,
    pub(crate) base_callable_signatures: CallableSignatureTable,
    pub(crate) base_data_semantics: DataSemanticTable,
    pub(crate) base_data_products: Vec<Arc<DataDeclarationProduct>>,
    pub(crate) base_enum_semantics: EnumSemanticTable,
    pub(crate) base_enum_products: Vec<Arc<EnumDeclarationProduct>>,
    pub(crate) base_associated_surfaces: AssociatedFamilyTable,
    pub(crate) base_associated_surface_products: Vec<Arc<AssociatedSurface>>,
    pub(crate) base_type_member_declarations: BTreeMap<TypeMemberDeclarationId, Arc<TypeMemberDeclarationProduct>>,
    pub(crate) base_type_member_products: Vec<Arc<TypeMemberDeclarationProduct>>,
    pub(crate) base_enum_requirements: EnumRequirementTable,
    pub(crate) base_enum_requirement_products: Vec<(DeclarationId, Arc<EnumRequirementsProduct>)>,
    pub(crate) canonical_alias_sources: BTreeMap<DeclarationId, (ModuleId, phalcom_ast::ast::TypeAliasDef)>,
    pub(crate) canonical_native_diagnostics: BTreeMap<ModuleId, Vec<SemanticDiagnostic>>,
}

static CANONICAL_UNIVERSE_SEMANTIC_BASELINE: OnceLock<CanonicalUniverseSemanticBaseline> = OnceLock::new();

fn canonical_source_class_supertype_template(
    store: &mut TypeStore,
    declaration: &DeclarationId,
    class_def: &phalcom_ast::ast::ClassDef,
    superclass: &DeclarationId,
) -> Option<GenericSupertypeTemplate> {
    let annotation = class_def.superclass.as_ref()?;
    assert!(
        matches!(&annotation.expr, phalcom_ast::ast::TypeAnnotationExpr::Reference(_)),
        "canonical Universe class {declaration:?} has an applied or non-nominal superclass; generic template support must be reconciled before baselining it"
    );
    // All current explicit Universe superclass forms are nominal references.
    // The resolved parent identity comes from the shared source hierarchy
    // resolver; GenericSupertypeTemplate remains the canonical owner-relative
    // representation, including when the child itself has generic parameters.
    let supertype = store.nominal_type(superclass.clone());
    Some(GenericSupertypeTemplate::from_type(store, declaration.clone(), supertype))
}

/// Returns the process-shared canonical semantic Universe baseline, building it on first access.
pub fn canonical_universe_semantic_baseline() -> &'static CanonicalUniverseSemanticBaseline {
    CANONICAL_UNIVERSE_SEMANTIC_BASELINE.get_or_init(|| {
        #[cfg(any(test, feature = "test-timings"))]
        crate::instrumentation::record_baseline_build();
        CanonicalUniverseSemanticBaseline::build_with_provider(phalcom_modules::UniverseSourceProvider::new())
    })
}

impl CanonicalUniverseSemanticBaseline {
    /// Builds a new baseline for the default canonical Universe provider.
    pub fn build() -> Self {
        Self::build_with_provider(phalcom_modules::UniverseSourceProvider::new())
    }

    /// Builds a baseline for the given provider.
    #[allow(deprecated)]
    pub fn build_with_provider(provider: phalcom_modules::UniverseSourceProvider) -> Self {
        let mut store = TypeStore::new();

        let mut base_declarations = bootstrap_universe_declarations(&mut store, &crate::core_surface::universe_declaration);

        // `Some` and `None` are runtime support classes for the canonical
        // Option enum rather than exported class bindings. They still need
        // declaration-backed forms so superclass checks can report the
        // canonical sealed-inheritance diagnostic without making `None` a
        // value binding in expression resolution.
        for key in [phalcom_native_meta::UniverseKey::Some, phalcom_native_meta::UniverseKey::None] {
            let declaration = crate::core_surface::universe_declaration(key);
            let form = store.nominal_type(declaration.clone());
            let class_object_type = store.class_object_type(declaration.clone());
            base_declarations.replace_authoritative(DeclarationTypeInfo {
                declaration,
                form,
                class_object_type,
                kind: KindId::TYPE,
                binding: crate::declarations::DeclarationTypeBinding::ordinary(),
                generic_binder_header: None,
                generic_signature: None,
                supertype_template: None,
            });
        }

        let mut base_hierarchy = MapTypeHierarchy::new();
        for relation in phalcom_native_meta::UNIVERSE_CLASS_RELATIONS {
            if let Some(superclass) = relation.superclass {
                base_hierarchy.insert(
                    crate::core_surface::universe_declaration(relation.class),
                    crate::core_surface::universe_declaration(superclass),
                );
            }
        }

        let mut base_dispatch = SurfaceDispatchResolver::new();
        let mut known_declarations: HashSet<DeclarationId> = base_declarations.iter().map(|(decl_id, _)| decl_id.clone()).collect();
        known_declarations.extend(base_declarations.intrinsic_static_domain_declarations().cloned());
        for kind in [
            crate::checker::proof_predicate::ProofPredicateKind::Equivalent,
            crate::checker::proof_predicate::ProofPredicateKind::Subtype,
            crate::checker::proof_predicate::ProofPredicateKind::Conforms,
        ] {
            known_declarations.insert(crate::checker::proof_predicate::proof_predicate_declaration(kind));
        }
        let canonical_linked = canonical_universe_linked_program(&provider);
        let resolver = LinkedTypeResolver::new(canonical_linked.clone(), known_declarations, ModuleId::universe_root());
        let native_report = register_native_surfaces(&mut store, &base_declarations, &resolver, &ModuleId::universe_root(), &mut base_dispatch)
            .expect("canonical native surface must import during semantic bootstrap");
        crate::checker::context::ensure_core_object_type_tests(&mut store, &base_declarations, &mut base_dispatch);

        let mut base_callable_signatures = CallableSignatureTable::new();
        for (_, signature) in native_report.callable_signatures {
            base_callable_signatures.insert(signature);
        }
        let core_class_new = crate::checker::declaration_signature::canonical_core_class_new_signature(&mut store);
        if base_callable_signatures.get(&core_class_new.callable).is_none() {
            base_callable_signatures.insert(core_class_new);
        }

        let mut base_data_semantics = DataSemanticTable::new();
        let mut base_data_products = Vec::new();
        let mut base_enum_semantics = EnumSemanticTable::new();
        let mut base_enum_products = Vec::new();
        let mut base_associated_surfaces = AssociatedFamilyTable::new();
        let mut base_associated_surface_products = Vec::new();
        let mut base_enum_requirements = EnumRequirementTable::new();
        let mut base_enum_requirement_products = Vec::new();

        let mut canonical_alias_sources = BTreeMap::new();

        // Canonical source enums and data are not native-meta classes. Add their
        // declaration forms before constructing the linked resolver so
        // source users can resolve them through the same
        // core identity path as native declarations.
        for node in provider.nodes() {
            let path = phalcom_modules::ModulePath::from_components(
                node.path
                    .iter()
                    .map(|p| phalcom_modules::ModuleComponent::from_identifier(p).expect("valid component"))
                    .collect::<Vec<_>>(),
            );
            let module_id = phalcom_modules::ModuleId::universe(path);
            let parsed = provider
                .load_parsed(&module_id)
                .unwrap_or_else(|e| panic!("failed to load universe module {module_id}: {e}"));
            // Phase 1: publish structural data/enum products for the complete
            // module before resolving any inherent impl target in that module.
            for stmt in &parsed.program.statements {
                if let Statement::TypeAlias(alias) | Statement::TypeDefinition(alias) = stmt {
                    let declaration = DeclarationId::new(module_id.clone(), alias.name.clone().into());
                    canonical_alias_sources.insert(declaration, (module_id.clone(), alias.clone()));
                    continue;
                }
                if let phalcom_ast::ast::Statement::Data(data_def) = stmt {
                    let decl_id = DeclarationId::new(module_id.clone(), data_def.name.clone().into());
                    if base_declarations.get(&decl_id).is_some() || base_declarations.is_intrinsic_static_domain(&decl_id) {
                        continue;
                    }
                    let mut parameter_ids = Vec::new();
                    let mut parameter_kinds = Vec::new();
                    let mut parameter_modes = Vec::new();
                    for (index, parameter) in data_def.generic_parameters.iter().enumerate() {
                        let Some(kind) = ready_kind_for_predeclaration(&mut store, parameter) else {
                            parameter_ids.clear();
                            break;
                        };
                        let mode = match parameter.mode {
                            None => crate::types::parameter::GenericParameterMode::Single,
                            Some(phalcom_ast::ast::ExpansionMode::Positional) => crate::types::parameter::GenericParameterMode::PositionalPack,
                            Some(phalcom_ast::ast::ExpansionMode::Labeled) => crate::types::parameter::GenericParameterMode::LabeledPack,
                            Some(phalcom_ast::ast::ExpansionMode::Complete) => crate::types::parameter::GenericParameterMode::CompletePack,
                        };
                        let parameter_id = store.intern_type_parameter(
                            TypeParameterData::new(TypeParameterOwner::Declaration(decl_id.clone()), index as u32, parameter.name.clone(), kind)
                                .with_mode(mode),
                        );
                        parameter_ids.push(parameter_id);
                        parameter_kinds.push(kind);
                        parameter_modes.push(mode);
                    }

                    if parameter_ids.len() != data_def.generic_parameters.len() {
                        continue;
                    }

                    let result_kind = if data_def.phase == phalcom_ast::ast::AlgebraicPhase::TypeOnly {
                        let domain = store.nominal_static_domain(decl_id.clone());
                        store.static_domain_kind(domain)
                    } else {
                        KindId::TYPE
                    };
                    let (form, kind, generic_signature) = if parameter_ids.is_empty() {
                        (store.nominal_form(decl_id.clone(), result_kind), result_kind, None)
                    } else {
                        let kind = store.arrow_kind(parameter_kinds.clone().into_boxed_slice(), result_kind);
                        let form = store.nominal_form(decl_id.clone(), kind);
                        let signature =
                            crate::types::parameter::GenericSignature::new(TypeParameterOwner::Declaration(decl_id.clone()), parameter_ids.into_boxed_slice())
                                .with_parameter_metadata(
                                    parameter_kinds.clone().into_boxed_slice(),
                                    vec![crate::types::variance::Variance::Invariant; parameter_kinds.len()].into_boxed_slice(),
                                )
                                .with_parameter_modes(parameter_modes.into_boxed_slice());
                        (form, kind, Some(signature))
                    };

                    base_declarations.replace_authoritative(DeclarationTypeInfo {
                        declaration: decl_id.clone(),
                        form,
                        class_object_type: store.class_object_type(decl_id.clone()),
                        kind,
                        binding: crate::declarations::DeclarationTypeBinding::ordinary(),
                        generic_binder_header: generic_signature
                            .as_ref()
                            .map(|signature| GenericBinderHeader::from_signature(&store, signature)),
                        generic_signature,
                        supertype_template: None,
                    });
                    base_hierarchy.insert(decl_id, crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Object));
                }
            }
        }

        // Rebuild the resolver after source-owned enum/data forms have been
        // predeclared. The initial native registration only needs the
        // declaration set available from the native/source bootstrap; source
        // products must see the complete canonical declaration universe.
        let mut known_declarations: HashSet<DeclarationId> = base_declarations.iter().map(|(decl_id, _)| decl_id.clone()).collect();
        known_declarations.extend(base_declarations.intrinsic_static_domain_declarations().cloned());
        known_declarations.extend(canonical_alias_sources.keys().cloned());
        for kind in [
            crate::checker::proof_predicate::ProofPredicateKind::Equivalent,
            crate::checker::proof_predicate::ProofPredicateKind::Subtype,
            crate::checker::proof_predicate::ProofPredicateKind::Conforms,
        ] {
            known_declarations.insert(crate::checker::proof_predicate::proof_predicate_declaration(kind));
        }
        let mut trait_associated_types: BTreeMap<DeclarationId, BTreeMap<String, crate::traits::AssociatedTypeRequirementId>> = BTreeMap::new();
        for node in provider.nodes() {
            let path = phalcom_modules::ModulePath::from_components(
                node.path
                    .iter()
                    .map(|p| phalcom_modules::ModuleComponent::from_identifier(p).expect("valid component"))
                    .collect::<Vec<_>>(),
            );
            let module_id = phalcom_modules::ModuleId::universe(path);
            let parsed = provider
                .load_parsed(&module_id)
                .unwrap_or_else(|e| panic!("failed to load universe module {module_id}: {e}"));
            for stmt in &parsed.program.statements {
                let Statement::Trait(trait_def) = stmt else { continue };
                let declaration = DeclarationId::new(module_id.clone(), trait_def.name.clone().into());
                known_declarations.insert(declaration.clone());
                let mut associated_index = 0u32;
                for member in &trait_def.members {
                    if let TraitMember::AssociatedType(associated) = member {
                        trait_associated_types.entry(declaration.clone()).or_default().insert(
                            associated.name.clone(),
                            crate::traits::AssociatedTypeRequirementId::new(declaration.clone(), associated_index),
                        );
                        associated_index += 1;
                    }
                }
            }
        }
        let resolver = LinkedTypeResolver::new(canonical_linked.clone(), known_declarations, ModuleId::universe_root())
            .with_trait_associated_types(Arc::new(trait_associated_types.clone()));

        // Universe source owns class inheritance. Native metadata contributes
        // implementation attachment and native-only relations; add each
        // source-authored class edge from the same canonical resolver used by
        // workspace HierarchyEdge queries before publishing inherited class
        // surfaces. The current source census has 68 explicit `is` clauses,
        // none with applied generic parents. Type-form resolution below also
        // preserves templates for generic classes and future applied forms.
        for node in provider.nodes() {
            let path = phalcom_modules::ModulePath::from_components(
                node.path
                    .iter()
                    .map(|p| phalcom_modules::ModuleComponent::from_identifier(p).expect("valid component"))
                    .collect::<Vec<_>>(),
            );
            let module_id = phalcom_modules::ModuleId::universe(path);
            let parsed = provider
                .load_parsed(&module_id)
                .unwrap_or_else(|e| panic!("failed to load universe module {module_id}: {e}"));
            for stmt in &parsed.program.statements {
                let Statement::Class(class_def) = stmt else { continue };
                let Some(declaration) = base_declarations
                    .iter()
                    .find_map(|(declaration, _)| (declaration.module == module_id && declaration.name.as_ref() == class_def.name).then(|| declaration.clone()))
                else {
                    // Source-only classes in a private Universe provider are
                    // discovered by the workspace update. This baseline pass
                    // enriches only declarations owned by the native core
                    // surface with canonical superclass authority.
                    continue;
                };
                let (resolved_super_decl, _) = crate::checker::context::resolve_source_class_superclass(&declaration, class_def, &resolver);
                let source_super_decl = resolved_super_decl.or_else(|| {
                    // A few runtime-owned source classes spell a parent that
                    // is also represented by an exact native-meta edge. The
                    // canonical linked program may not expose that implicit
                    // prelude name as an import; accept it only when the
                    // existing edge exactly matches the source name's
                    // canonical Universe identity.
                    let super_ref = class_def.superclass_ref()?;
                    let key = phalcom_native_meta::UniverseKey::from_name(&super_ref.root)?;
                    let expected = crate::core_surface::universe_declaration(key);
                    (base_hierarchy.superclasses.get(&declaration) == Some(&expected)).then_some(expected)
                });
                if class_def.superclass.is_some() && source_super_decl.is_none() {
                    panic!(
                        "could not resolve canonical Universe superclass for {declaration:?}: {:?}",
                        class_def.superclass
                    );
                }
                if let Some(super_decl) = source_super_decl.as_ref() {
                    base_hierarchy.insert(declaration.clone(), super_decl.clone());
                }
                if let Some(super_decl) = source_super_decl.as_ref()
                    && let Some(template) = canonical_source_class_supertype_template(&mut store, &declaration, class_def, super_decl)
                {
                    let mut info = base_declarations
                        .get(&declaration)
                        .cloned()
                        .unwrap_or_else(|| panic!("canonical Universe declaration form is missing for {declaration:?}"));
                    info.supertype_template = Some(template.clone());
                    base_declarations.replace_authoritative(info);
                    base_hierarchy.insert_template(template);
                }
            }
        }

        // Publish source-authored class members into the reusable baseline.
        // Native metadata remains authoritative for colliding native members;
        // source-only members (including generic iterator constructors) are
        // added to the same declaration surface consumed by user analysis.
        let mut canonical_native_diagnostics: BTreeMap<ModuleId, Vec<SemanticDiagnostic>> = BTreeMap::new();
        for node in provider.nodes() {
            let path = phalcom_modules::ModulePath::from_components(
                node.path
                    .iter()
                    .map(|p| phalcom_modules::ModuleComponent::from_identifier(p).expect("valid component"))
                    .collect::<Vec<_>>(),
            );
            let module_id = phalcom_modules::ModuleId::universe(path);
            let parsed = provider
                .load_parsed(&module_id)
                .unwrap_or_else(|e| panic!("failed to load universe module {module_id}: {e}"));
            for stmt in &parsed.program.statements {
                let Statement::Class(class_def) = stmt else { continue };
                let declaration = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                let mut context =
                    CheckingContext::new_with_dispatch_ref(&mut store, &base_hierarchy, &resolver, &base_declarations, &base_dispatch, module_id.clone());
                let (source_surface, source_signatures) = crate::checker::declaration::build_declared_class_surface(&mut context, class_def);
                drop(context);
                let mut merged = base_dispatch
                    .surface(&declaration)
                    .cloned()
                    .unwrap_or_else(|| crate::surface::DeclarationSurface::new(Some(declaration.clone())));
                for side in [DispatchSide::Instance, DispatchSide::Class] {
                    let source_side = source_surface.surface(side);
                    for (name, ty) in &source_side.fields {
                        if !merged.surface(side).fields.contains_key(name) {
                            merged.add_field_with_visibility(
                                side,
                                name.clone(),
                                ty.clone(),
                                *source_side.field_visibility.get(name).unwrap_or(&crate::surface::MemberVisibility::Public),
                            );
                        }
                    }
                    for (selector, signature) in &source_side.callable_signatures {
                        if merged.surface(side).get_callable(selector).is_none() {
                            merged.add_callable_with_visibility(
                                side,
                                signature.clone(),
                                *source_side
                                    .callable_visibility
                                    .get(selector)
                                    .unwrap_or(&crate::surface::MemberVisibility::Public),
                            );
                        }
                    }
                }
                for signature in source_signatures.into_values() {
                    if signature.implementation == phalcom_common::modifier::ImplementationKind::Native {
                        let range = signature.source.as_ref().map(|source| source.range).unwrap_or(class_def.range);
                        let native = base_callable_signatures.get(&signature.callable);
                        let mismatch = match native {
                            None => Some("canonical native signature is missing".to_string()),
                            Some(native) if native.implementation != phalcom_common::modifier::ImplementationKind::Native => {
                                Some("canonical implementation category is not native".to_string())
                            }
                            Some(native) => signature.native_contract_mismatch_with_store(native, Some(&store)),
                        };
                        if let Some(reason) = mismatch {
                            canonical_native_diagnostics
                                .entry(module_id.clone())
                                .or_default()
                                .push(SemanticDiagnostic::error_in(
                                    module_id.clone(),
                                    DiagnosticCode::NativeSignatureMismatch,
                                    format!(
                                        "native.signature_mismatch: {}::{} ({reason})",
                                        signature.owner.name,
                                        signature.selector.encode()
                                    ),
                                    range,
                                ));
                        }
                    }
                    if base_callable_signatures.get(&signature.callable).is_none() {
                        base_callable_signatures.insert(signature);
                    }
                }
                base_dispatch.register_surface(declaration.clone(), merged);
                if let Some(ty) = base_declarations.form(&declaration) {
                    base_dispatch.register_type(ty, declaration);
                }
            }
        }

        for node in provider.nodes() {
            let path = phalcom_modules::ModulePath::from_components(
                node.path
                    .iter()
                    .map(|p| phalcom_modules::ModuleComponent::from_identifier(p).expect("valid component"))
                    .collect::<Vec<_>>(),
            );
            let module_id = phalcom_modules::ModuleId::universe(path);
            let parsed = provider
                .load_parsed(&module_id)
                .unwrap_or_else(|e| panic!("failed to load universe module {module_id}: {e}"));
            for stmt in &parsed.program.statements {
                if let phalcom_ast::ast::Statement::Data(data_def) = stmt {
                    let decl_id = DeclarationId::new(module_id.clone(), data_def.name.clone().into());
                    if data_def.is_sum() {
                        let Some(enum_product) =
                            crate::checker::enum_declaration::build_enum_semantics(&decl_id, data_def, &mut store, &base_declarations, &resolver, &module_id)
                        else {
                            continue;
                        };
                        base_enum_semantics.insert_enum(enum_product.info.clone());
                        for v in enum_product.variants.iter() {
                            base_enum_semantics.insert_variant(v.clone());
                        }
                        base_enum_products.push(Arc::new(enum_product));
                    } else {
                        let Some(data_product) =
                            crate::checker::data_declaration::build_data_semantics(&decl_id, data_def, &mut store, &base_declarations, &resolver, &module_id)
                        else {
                            continue;
                        };
                        base_data_semantics.insert_data(data_product.info.clone());
                        base_data_products.push(Arc::new(data_product));

                        let assoc_surface = crate::associated::build_data_associated_surface(&decl_id);
                        base_associated_surfaces.insert(decl_id.clone(), assoc_surface.clone());
                        base_associated_surface_products.push(assoc_surface);
                    }
                }
            }

            // Phase 2: form canonical impl contributions only after exact-case
            // target metadata exists. ImplId remains the source-statement ID;
            // the contribution owns all target and callable identity decisions.
            let mut base_inherent_contributions = Vec::new();
            for (statement_index, stmt) in parsed.program.statements.iter().enumerate() {
                let phalcom_ast::ast::Statement::Impl(impl_def) = stmt else {
                    continue;
                };
                if !matches!(impl_def.kind, phalcom_ast::ast::ImplKind::Inherent) {
                    continue;
                }
                let impl_id = crate::identity::ImplId::new(module_id.clone(), crate::identity::ImplLocalId(statement_index as u32));
                let mut impl_ctx = crate::checker::CheckingContext::new_with_dispatch_ref(
                    &mut store,
                    &base_hierarchy,
                    &resolver,
                    &base_declarations,
                    &base_dispatch,
                    module_id.clone(),
                );
                impl_ctx.attach_data_semantics(&base_data_semantics);
                impl_ctx.attach_enum_semantics(&base_enum_semantics);
                base_inherent_contributions.push(crate::impls::build_inherent_impl_contribution(&mut impl_ctx, &impl_id, impl_def));
            }

            // Canonical data declarations have no source class surface of
            // their own, but their inherent impls still define ordinary
            // dispatch members. Merge those contributions into the baseline
            // surface before publishing the reusable Universe snapshot.
            let mut contributions_by_target = BTreeMap::<DeclarationId, Vec<crate::impls::InherentImplContribution>>::new();
            for contribution in &base_inherent_contributions {
                if matches!(contribution.target, crate::impls::InherentImplTarget::Declaration(_)) {
                    contributions_by_target
                        .entry(contribution.target.declaration().clone())
                        .or_default()
                        .push(contribution.clone());
                }
            }
            for (target, contributions) in contributions_by_target {
                let primary_surface = base_dispatch
                    .surface(&target)
                    .cloned()
                    .unwrap_or_else(|| crate::surface::DeclarationSurface::new(Some(target.clone())));
                let primary_signatures = base_callable_signatures
                    .iter()
                    .filter(|(callable, _)| callable.try_declaration_owner() == Some(&target))
                    .map(|(callable, signature)| (callable.clone(), signature.clone()))
                    .collect::<HashMap<_, _>>();
                let effective = crate::impls::build_effective_surface(
                    &target,
                    &primary_surface,
                    &primary_signatures,
                    &contributions,
                    base_data_semantics.data_info(&target).map(AsRef::as_ref),
                    base_associated_surfaces.surfaces.get(&target).map(AsRef::as_ref),
                );
                base_dispatch.register_surface(target.clone(), (*effective.surface).clone());
                base_dispatch.register_conditional_members(
                    crate::impls::InherentImplTarget::Declaration(target.clone()),
                    effective.conditional_members.clone(),
                );
                for contribution in &contributions {
                    for member in &contribution.members {
                        if member.signature.native_id.is_none() {
                            base_callable_signatures.insert(member.signature.clone());
                        }
                    }
                }
                for signature in effective.native_anchors.values() {
                    base_callable_signatures.insert(signature.clone());
                }
            }

            // Phase 3: derive enum behavior, requirements, and associated
            // surfaces from those semantic contributions. Enum syntax is
            // structural and supplies no callable fallback.
            for stmt in &parsed.program.statements {
                if let phalcom_ast::ast::Statement::Data(enum_def) = stmt {
                    if !enum_def.is_sum() {
                        continue;
                    }
                    let decl_id = DeclarationId::new(module_id.clone(), enum_def.name.clone().into());
                    let enum_contributions = base_inherent_contributions
                        .iter()
                        .filter(|contribution| contribution.target.declaration() == &decl_id)
                        .cloned()
                        .collect::<Vec<_>>();
                    let mut behavior_ctx = crate::checker::CheckingContext::new_with_dispatch_ref(
                        &mut store,
                        &base_hierarchy,
                        &resolver,
                        &base_declarations,
                        &base_dispatch,
                        module_id.clone(),
                    );
                    behavior_ctx.attach_data_semantics(&base_data_semantics);
                    behavior_ctx.attach_enum_semantics(&base_enum_semantics);
                    let behavior_product = crate::checker::enum_behavior::build_enum_behavior(&mut behavior_ctx, &decl_id, Some(enum_def), &enum_contributions);

                    let mut surface = base_dispatch.surface(&decl_id).cloned().unwrap_or_default();
                    for default_sig in behavior_product.root_defaults.iter() {
                        base_callable_signatures.insert(default_sig.clone());
                        let projection = crate::checker::declaration_signature::project_semantic_signature(default_sig);
                        surface.add_callable(default_sig.side, projection);
                    }
                    base_dispatch.register_surface(decl_id.clone(), surface);
                    if let Some(ty) = base_declarations.form(&decl_id) {
                        base_dispatch.register_type(ty, decl_id.clone());
                    }

                    for case_sigs in behavior_product.case_implementations.values() {
                        for case_sig in case_sigs.iter() {
                            base_callable_signatures.insert(case_sig.clone());
                        }
                    }

                    let mut behavior_bases: std::collections::HashSet<phalcom_common::selector::SelectorBase> = std::collections::HashSet::new();
                    for c in &enum_contributions {
                        if c.target == crate::impls::InherentImplTarget::Declaration(decl_id.clone()) {
                            for m in &c.members {
                                behavior_bases.insert(m.signature.selector.base.clone());
                            }
                        }
                    }

                    let (assoc_surface, _assoc_diags) = build_associated_surface(
                        &decl_id,
                        Some(
                            &enum_def
                                .cases()
                                .unwrap_or_default()
                                .iter()
                                .map(|v| {
                                    let sel = phalcom_ast::selector::selector_from_variant(v);
                                    crate::identity::VariantId::new(decl_id.clone(), sel)
                                })
                                .collect::<Vec<_>>(),
                        ),
                        &behavior_bases,
                        &std::collections::HashSet::new(),
                        &module_id,
                        Some(crate::diagnostic::SemanticSourceSpan::new(module_id.clone(), enum_def.range)),
                    );
                    base_associated_surfaces.insert(decl_id.clone(), assoc_surface.clone());
                    base_associated_surface_products.push(assoc_surface);

                    let variants_info: Vec<VariantInfo> = enum_def
                        .cases()
                        .unwrap_or_default()
                        .iter()
                        .filter_map(|v| {
                            let sel = phalcom_ast::selector::selector_from_variant(v);
                            let vid = crate::identity::VariantId::new(decl_id.clone(), sel);
                            base_enum_semantics.variant_info(&vid).cloned()
                        })
                        .collect();

                    let mut case_methods_map: HashMap<crate::identity::VariantId, Vec<crate::signature::CallableSemanticSignature>> = HashMap::new();
                    for (v_id, sigs) in &behavior_product.case_implementations {
                        case_methods_map.insert(v_id.clone(), sigs.to_vec());
                    }

                    let (case_statuses, req_diags) = check_enum_requirements(
                        &decl_id,
                        base_enum_semantics.enum_info(&decl_id).unwrap(),
                        &variants_info,
                        &behavior_product.root_requirements,
                        &case_methods_map,
                        &mut store,
                        &base_hierarchy,
                        &module_id,
                    );
                    base_enum_requirements.insert(decl_id.clone(), Arc::from(behavior_product.root_requirements.clone()), case_statuses.clone());
                    let req_product = Arc::new(EnumRequirementsProduct {
                        requirements: Arc::from(behavior_product.root_requirements),
                        case_statuses,
                        diagnostics: req_diags,
                    });
                    base_enum_requirement_products.push((decl_id, req_product));
                }
            }
        }

        let mut base_type_member_declarations = BTreeMap::new();
        let mut base_type_member_products = Vec::new();
        let mut base_type_member_shells = BTreeMap::<DeclarationId, Vec<crate::associated::AssociatedMemberShell>>::new();

        for node in provider.nodes() {
            let path = phalcom_modules::ModulePath::from_components(
                node.path
                    .iter()
                    .map(|p| phalcom_modules::ModuleComponent::from_identifier(p).expect("valid component"))
                    .collect::<Vec<_>>(),
            );
            let module_id = phalcom_modules::ModuleId::universe(path);
            let parsed = provider
                .load_parsed(&module_id)
                .unwrap_or_else(|e| panic!("failed to load universe module {module_id}: {e}"));

            for stmt in &parsed.program.statements {
                match stmt {
                    phalcom_ast::ast::Statement::Class(class_def) => {
                        let owner = DeclarationId::new(module_id.clone(), class_def.name.clone().into());
                        for member in &class_def.members {
                            let phalcom_ast::ast::ClassMember::TypeMember(definition) = member else {
                                continue;
                            };
                            let (product, _diags) = crate::checker::type_member_declaration::build_type_member_declaration(
                                &mut store,
                                &owner,
                                definition,
                                &base_declarations,
                                &resolver,
                                &module_id,
                            );
                            if let Some(product) = product {
                                let selector = crate::types::type_member::TypeMemberSelector::from_syntax(&definition.selector);
                                let base = match &selector {
                                    crate::types::type_member::TypeMemberSelector::Named(name) => {
                                        phalcom_common::selector::SelectorBase::Named(name.to_string())
                                    }
                                    crate::types::type_member::TypeMemberSelector::Subscript => phalcom_common::selector::SelectorBase::Subscript,
                                };
                                let kind = if base_declarations
                                    .intrinsic_type_owner(&owner)
                                    .and_then(|owner| crate::types::type_member::lookup_intrinsic_type_operation(owner, &selector))
                                    .is_some()
                                {
                                    crate::associated::AssociatedFamilyKind::IntrinsicTypeMember
                                } else {
                                    crate::associated::AssociatedFamilyKind::TypeMember
                                };
                                base_type_member_shells
                                    .entry(owner.clone())
                                    .or_default()
                                    .push(crate::associated::AssociatedMemberShell {
                                        base,
                                        kind,
                                        member: crate::associated::AssociatedMemberId::TypeMember(product.id.clone()),
                                        module: module_id.clone(),
                                        source_range: definition.range,
                                    });
                                let arc_prod = Arc::new(product);
                                base_type_member_declarations.insert(arc_prod.id.clone(), arc_prod.clone());
                                base_type_member_products.push(arc_prod);
                            }
                        }
                    }
                    phalcom_ast::ast::Statement::Impl(impl_def) if matches!(impl_def.kind, phalcom_ast::ast::ImplKind::Inherent) => {
                        let Some(owner) = implementation_owner(&module_id, impl_def, &resolver, &base_declarations, &store) else {
                            continue;
                        };
                        for member in &impl_def.members {
                            let phalcom_ast::ast::ImplMember::TypeMember(definition) = member else {
                                continue;
                            };
                            let (product, _diags) = crate::checker::type_member_declaration::build_type_member_declaration(
                                &mut store,
                                &owner,
                                definition,
                                &base_declarations,
                                &resolver,
                                &module_id,
                            );
                            if let Some(product) = product {
                                let selector = crate::types::type_member::TypeMemberSelector::from_syntax(&definition.selector);
                                let base = match &selector {
                                    crate::types::type_member::TypeMemberSelector::Named(name) => {
                                        phalcom_common::selector::SelectorBase::Named(name.to_string())
                                    }
                                    crate::types::type_member::TypeMemberSelector::Subscript => phalcom_common::selector::SelectorBase::Subscript,
                                };
                                let kind = if base_declarations
                                    .intrinsic_type_owner(&owner)
                                    .and_then(|owner| crate::types::type_member::lookup_intrinsic_type_operation(owner, &selector))
                                    .is_some()
                                {
                                    crate::associated::AssociatedFamilyKind::IntrinsicTypeMember
                                } else {
                                    crate::associated::AssociatedFamilyKind::TypeMember
                                };
                                base_type_member_shells
                                    .entry(owner.clone())
                                    .or_default()
                                    .push(crate::associated::AssociatedMemberShell {
                                        base,
                                        kind,
                                        member: crate::associated::AssociatedMemberId::TypeMember(product.id.clone()),
                                        module: module_id.clone(),
                                        source_range: definition.range,
                                    });
                                let arc_prod = Arc::new(product);
                                base_type_member_declarations.insert(arc_prod.id.clone(), arc_prod.clone());
                                base_type_member_products.push(arc_prod);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        for (owner, shells) in base_type_member_shells {
            let base_surface = base_associated_surfaces
                .surfaces
                .get(&owner)
                .cloned()
                .unwrap_or_else(|| Arc::new(crate::associated::AssociatedSurface::new(owner.clone())));
            let (surface, _diags) =
                crate::associated::extend_associated_surface_with_shells(&owner, base_surface, &shells, &HashSet::new(), &HashSet::new(), &owner.module);
            base_associated_surfaces.insert(owner.clone(), surface.clone());
            base_associated_surface_products.push(surface);
        }

        let base_type_member_declarations_index = base_type_member_declarations
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<crate::parent_map::ParentMap<_, _>>();
        resolver.update_associated_type_members(&base_associated_surfaces, &base_type_member_declarations_index);

        Self {
            provider,
            linked: canonical_linked,
            store,
            base_declarations,
            base_hierarchy,
            base_dispatch,
            base_callable_signatures,
            base_data_semantics,
            base_data_products,
            base_enum_semantics,
            base_enum_products,
            base_associated_surfaces,
            base_associated_surface_products,
            base_type_member_declarations,
            base_type_member_products,
            base_enum_requirements,
            base_enum_requirement_products,
            canonical_alias_sources,
            canonical_native_diagnostics,
        }
    }

    pub fn instantiate(&self, workspace: WorkspaceId) -> SemanticWorkspaceSession {
        #[cfg(any(test, feature = "test-timings"))]
        crate::instrumentation::record_session_instantiation();
        SemanticWorkspaceSession::from_baseline(self, workspace)
    }

    pub fn type_store(&self) -> &TypeStore {
        &self.store
    }

    pub fn declarations(&self) -> &DeclarationTypeTable {
        &self.base_declarations
    }

    pub fn hierarchy(&self) -> &MapTypeHierarchy {
        &self.base_hierarchy
    }

    pub fn dispatch(&self) -> &SurfaceDispatchResolver {
        &self.base_dispatch
    }

    pub fn callable_signatures(&self) -> &CallableSignatureTable {
        &self.base_callable_signatures
    }

    pub fn data_semantics(&self) -> &DataSemanticTable {
        &self.base_data_semantics
    }

    pub fn enum_semantics(&self) -> &EnumSemanticTable {
        &self.base_enum_semantics
    }

    pub fn associated_surfaces(&self) -> &AssociatedFamilyTable {
        &self.base_associated_surfaces
    }

    pub fn enum_requirements(&self) -> &EnumRequirementTable {
        &self.base_enum_requirements
    }

    pub fn linked_program(&self) -> &Arc<LinkedProgram> {
        &self.linked
    }
}
