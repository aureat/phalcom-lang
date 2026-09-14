//! Canonical inherent implementation fragment analysis, target resolution, and contribution publication.

use crate::checker::context::CheckingContext;
use crate::checker::declaration_signature::{CallableSyntaxRef, semantic_signature_for_syntax_with_resolver};
use crate::declaration_type::DeclaredTypeState;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic, SemanticSourceSpan};
use crate::identity::{CallableId, CallableOwnerId, DeclarationId, DispatchSide, ImplId};
use crate::signature::CallableSemanticSignature;
use crate::surface::MemberVisibility;
use crate::types::annotation::{
    ScopedTypeResolver, TypeFormationSite, TypeLevelBinding, TypeResolver, resolve_type_annotation, type_level_binding_for_parameter,
};
use crate::types::evidence::TypeKnowledge;
use crate::types::id::{KindId, TypeId, TypeParameterId};
use crate::types::parameter::{GenericSignature, TypeParameterData, TypeParameterOwner, TypeTerm};
use crate::types::store::{TypeData, TypeStore};
use crate::types::substitution::TypeSubstitution;
use phalcom_ast::ast::{BehaviorMember, ImplDef, TypeAnnotationExpr};
use std::collections::HashMap;

/// Explicit bijection mapping between impl type parameters and canonical target declaration parameters.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoveringImplSubstitution {
    /// Mapping: impl-owned TypeParameterId -> declaration-owned TypeParameterId
    pub impl_to_decl: HashMap<TypeParameterId, TypeParameterId>,
    /// Inverse: declaration-owned TypeParameterId -> impl-owned TypeParameterId
    pub decl_to_impl: HashMap<TypeParameterId, TypeParameterId>,
}

impl CoveringImplSubstitution {
    pub fn new(
        impl_to_decl: HashMap<TypeParameterId, TypeParameterId>,
        decl_to_impl: HashMap<TypeParameterId, TypeParameterId>,
    ) -> Self {
        Self {
            impl_to_decl,
            decl_to_impl,
        }
    }

    /// Converts the `impl_to_decl` map into a `TypeSubstitution` replacing impl parameter types with declaration parameter types.
    pub fn to_type_substitution(&self, store: &mut TypeStore) -> TypeSubstitution {
        let mut subst = TypeSubstitution::new();
        for (&impl_param, &decl_param) in &self.impl_to_decl {
            let decl_ty = store.parameter_form(decl_param);
            subst.bind(impl_param, decl_ty);
        }
        subst
    }
}

/// Applicability regime of an inherent impl block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InherentImplApplicability {
    /// Unconditional non-generic target (e.g. `impl User`).
    Unconditional,
    /// Covering generic target (e.g. `impl<A, B> Pair<A, B>` or `impl<A, B> Pair<B, A>`).
    Covering(CoveringImplSubstitution),
}

/// Target of an inherent implementation block (nominal declaration or exact enum case).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InherentImplTarget {
    Declaration(DeclarationId),
    ExactEnumCase(VariantId),
}

impl InherentImplTarget {
    pub fn declaration(&self) -> &DeclarationId {
        match self {
            InherentImplTarget::Declaration(id) => id,
            InherentImplTarget::ExactEnumCase(variant_id) => &variant_id.owner,
        }
    }

    pub fn to_callable_owner(&self) -> CallableOwnerId {
        match self {
            InherentImplTarget::Declaration(id) => CallableOwnerId::Declaration(id.clone()),
            InherentImplTarget::ExactEnumCase(variant_id) => CallableOwnerId::Variant(variant_id.clone()),
        }
    }
}

impl From<InherentImplTarget> for CallableOwnerId {
    fn from(target: InherentImplTarget) -> Self {
        match target {
            InherentImplTarget::Declaration(id) => CallableOwnerId::Declaration(id),
            InherentImplTarget::ExactEnumCase(variant_id) => CallableOwnerId::Variant(variant_id),
        }
    }
}

/// Result of resolving the target of an inherent impl block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedInherentImplTarget {
    pub id: ImplId,
    pub target: InherentImplTarget,
    pub target_type: TypeId,
    pub generic_signature: Option<GenericSignature>,
    pub applicability: InherentImplApplicability,
    pub source: SemanticSourceSpan,
    pub diagnostics: Box<[SemanticDiagnostic]>,
}

impl ResolvedInherentImplTarget {
    pub fn declaration(&self) -> &DeclarationId {
        self.target.declaration()
    }
}

/// A single behavior-only member contributed by an inherent impl block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InherentMemberContribution {
    pub callable: CallableId,
    pub signature: CallableSemanticSignature,
    pub visibility: MemberVisibility,
    pub source_member: usize,
    pub is_requirement: bool,
}

/// Canonical contribution product for an inherent impl block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InherentImplContribution {
    pub id: ImplId,
    pub target: InherentImplTarget,
    pub generic_signature: Option<GenericSignature>,
    pub covering: Option<CoveringImplSubstitution>,
    pub members: Box<[InherentMemberContribution]>,
    pub source: SemanticSourceSpan,
    pub diagnostics: Box<[SemanticDiagnostic]>,
}

impl InherentImplContribution {
    pub fn target_declaration(&self) -> &DeclarationId {
        self.target.declaration()
    }
}

use crate::associated::AssociatedSurface;
use crate::data_semantics::DataInfo;
use crate::identity::{DataComponentId, FieldId, VariantId};
use crate::surface::DeclarationSurface;
use phalcom_common::selector::Selector;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Target-indexed set of inherent impl fragments within a module.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InherentImplSet {
    pub target: DeclarationId,
    pub impls: Box<[ImplId]>,
}

/// Origin of a callable definition accepted into the effective surface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CallableDefinitionOrigin {
    /// Defined in the primary class/enum declaration.
    PrimaryDeclaration,
    /// Defined in an inherent `impl` block fragment.
    InherentImpl(ImplId),
}

/// Canonical definition provenance for an accepted callable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectiveCallableDefinition {
    pub callable: CallableId,
    pub origin: CallableDefinitionOrigin,
    pub source_member_index: usize,
    pub signature: CallableSemanticSignature,
}

/// Product of merging a primary declared surface with inherent impl contributions.
#[derive(Clone, Debug)]
pub struct EffectiveSurfaceProduct {
    pub owner: DeclarationId,
    pub surface: Arc<DeclarationSurface>,
    pub definitions: BTreeMap<CallableId, EffectiveCallableDefinition>,
    pub diagnostics: Arc<[SemanticDiagnostic]>,
}

impl EffectiveSurfaceProduct {
    pub fn new(
        owner: DeclarationId,
        surface: Arc<DeclarationSurface>,
        definitions: BTreeMap<CallableId, EffectiveCallableDefinition>,
        diagnostics: Arc<[SemanticDiagnostic]>,
    ) -> Self {
        Self {
            owner,
            surface,
            definitions,
            diagnostics,
        }
    }
}

/// Internal origin of a reserved member entry during conflict checking.
#[derive(Clone, Debug, Eq, PartialEq)]
enum EffectiveMemberOrigin {
    PrimaryCallable(CallableId),
    InherentCallable {
        callable: CallableId,
        impl_id: ImplId,
    },
    DataComponent(DataComponentId),
    VariantConstructor(VariantId),
}

/// Checks and merges a primary declaration surface with inherent impl contributions.
pub fn build_effective_surface(
    owner: &DeclarationId,
    primary_surface: &DeclarationSurface,
    primary_signatures: &HashMap<CallableId, CallableSemanticSignature>,
    inherent_contributions: &[InherentImplContribution],
    data_info: Option<&DataInfo>,
    associated_surface: Option<&AssociatedSurface>,
) -> EffectiveSurfaceProduct {
    let mut diagnostics = Vec::new();
    let mut effective_surface = DeclarationSurface::new(Some(owner.clone()));
    let mut definitions = BTreeMap::new();

    // Reserved namespace tracking per dispatch side
    // (Side, Selector) -> EffectiveMemberOrigin
    let mut reserved_selectors: HashMap<(DispatchSide, Selector), EffectiveMemberOrigin> = HashMap::new();

    // (Side, Field Name) -> (FieldId, MemberVisibility)
    let mut reserved_fields: HashMap<(DispatchSide, String), (FieldId, MemberVisibility)> = HashMap::new();

    // 1. Reserve data components on instance side if applicable
    if let Some(info) = data_info {
        for component in &info.components {
            if let Ok(selector) = Selector::getter(component.local_name.as_ref()) {
                let origin = EffectiveMemberOrigin::DataComponent(component.id.clone());
                reserved_selectors.insert((DispatchSide::Instance, selector), origin);
            }
        }
    }

    // 2. Reserve enum variant constructors on class side if applicable
    if let Some(assoc) = associated_surface {
        for family in assoc.families.values() {
            for member in family.members.iter() {
                if let crate::associated::AssociatedMemberId::Variant(v_id) = member {
                    let origin = EffectiveMemberOrigin::VariantConstructor(v_id.clone());
                    reserved_selectors.insert((DispatchSide::Class, v_id.selector.clone()), origin);
                }
            }
        }
    }

    // 3. Register primary fields
    for (name, field_id) in &primary_surface.instance.fields_by_name {
        let ty = primary_surface.instance.fields.get(name).cloned().unwrap_or(TypeKnowledge::Dynamic(crate::types::evidence::DynamicReason::ExplicitEscape));
        let visibility = primary_surface.instance.field_visibility.get(name).copied().unwrap_or(MemberVisibility::Public);
        reserved_fields.insert((DispatchSide::Instance, name.clone()), (field_id.clone(), visibility));
        effective_surface.instance.add_field_with_visibility(Some(owner), DispatchSide::Instance, name, ty, visibility);
    }
    for (name, field_id) in &primary_surface.class.fields_by_name {
        let ty = primary_surface.class.fields.get(name).cloned().unwrap_or(TypeKnowledge::Dynamic(crate::types::evidence::DynamicReason::ExplicitEscape));
        let visibility = primary_surface.class.field_visibility.get(name).copied().unwrap_or(MemberVisibility::Public);
        reserved_fields.insert((DispatchSide::Class, name.clone()), (field_id.clone(), visibility));
        effective_surface.class.add_field_with_visibility(Some(owner), DispatchSide::Class, name, ty, visibility);
    }

    // 4. Register primary callables
    for (side, surface) in [(DispatchSide::Instance, &primary_surface.instance), (DispatchSide::Class, &primary_surface.class)] {
        for (selector, sig) in &surface.callable_signatures {
            let callable_id = CallableId::new(owner.clone(), selector.clone(), side);
            let visibility = surface.callable_visibility.get(selector).copied().unwrap_or(MemberVisibility::Public);

            // Check if selector already reserved (e.g. data component collision with primary getter)
            if let Some(existing) = reserved_selectors.get(&(side, selector.clone())) {
                match existing {
                    EffectiveMemberOrigin::DataComponent(comp_id) => {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            owner.module.clone(),
                            DiagnosticCode::ImplMemberConflict,
                            format!(
                                "declaration member `{}` conflicts with data component index {}",
                                selector, comp_id.index
                            ),
                            phalcom_common::range::SourceRange::default(),
                        ));
                        continue;
                    }
                    EffectiveMemberOrigin::VariantConstructor(v_id) => {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            owner.module.clone(),
                            DiagnosticCode::EnumFamilyCategoryConflict,
                            format!(
                                "class callable `{}` conflicts with variant constructor `{}`",
                                selector, v_id.selector
                            ),
                            phalcom_common::range::SourceRange::default(),
                        ));
                        continue;
                    }
                    _ => {}
                }
            }

            reserved_selectors.insert((side, selector.clone()), EffectiveMemberOrigin::PrimaryCallable(callable_id.clone()));
            effective_surface.surface_mut(side).add_callable_with_visibility(Some(owner), side, sig.clone(), visibility);

            if let Some(sem_sig) = primary_signatures.get(&callable_id) {
                definitions.insert(
                    callable_id.clone(),
                    EffectiveCallableDefinition {
                        callable: callable_id,
                        origin: CallableDefinitionOrigin::PrimaryDeclaration,
                        source_member_index: 0,
                        signature: sem_sig.clone(),
                    },
                );
            }
        }
    }

    // 5. Merge inherent impl contributions in deterministic order
    for contribution in inherent_contributions {
        if matches!(contribution.target, InherentImplTarget::ExactEnumCase(_)) {
            continue;
        }
        if contribution.target.declaration() != owner {
            continue;
        }
        diagnostics.extend(contribution.diagnostics.iter().cloned());

        for member in contribution.members.iter() {
            let side = member.signature.side;
            let selector = &member.signature.callable.selector;
            let callable_id = &member.signature.callable;

            // Conflict check
            if let Some(existing) = reserved_selectors.get(&(side, selector.clone())) {
                let err_msg = match existing {
                    EffectiveMemberOrigin::PrimaryCallable(_) => {
                        format!(
                            "impl member `{}` on `{}` conflicts with a primary declaration member of the same selector",
                            selector, owner.name
                        )
                    }
                    EffectiveMemberOrigin::InherentCallable { impl_id, .. } => {
                        format!(
                            "impl member `{}` on `{}` conflicts with another inherent impl member from impl {:?}",
                            selector, owner.name, impl_id
                        )
                    }
                    EffectiveMemberOrigin::DataComponent(comp_id) => {
                        format!(
                            "impl getter `{}` on `{}` conflicts with data component index {}",
                            selector, owner.name, comp_id.index
                        )
                    }
                    EffectiveMemberOrigin::VariantConstructor(v_id) => {
                        format!(
                            "class-side impl member `{}` on `{}` conflicts with enum variant constructor `{}`",
                            selector, owner.name, v_id.selector
                        )
                    }
                };

                let span_range = member.signature.source.as_ref().map(|s| s.range).unwrap_or_default();
                diagnostics.push(SemanticDiagnostic::error_in(
                    owner.module.clone(),
                    DiagnosticCode::ImplMemberConflict,
                    err_msg,
                    span_range,
                ));
                continue;
            }

            reserved_selectors.insert(
                (side, selector.clone()),
                EffectiveMemberOrigin::InherentCallable {
                    callable: callable_id.clone(),
                    impl_id: contribution.id.clone(),
                },
            );

            let projection = crate::checker::declaration_signature::project_semantic_signature(&member.signature);
            effective_surface.surface_mut(side).add_callable_with_visibility(
                Some(owner),
                side,
                projection,
                member.visibility,
            );

            if !member.is_requirement {
                definitions.insert(
                    callable_id.clone(),
                    EffectiveCallableDefinition {
                        callable: callable_id.clone(),
                        origin: CallableDefinitionOrigin::InherentImpl(contribution.id.clone()),
                        source_member_index: member.source_member,
                        signature: member.signature.clone(),
                    },
                );
            }
        }
    }

    EffectiveSurfaceProduct {
        owner: owner.clone(),
        surface: Arc::new(effective_surface),
        definitions,
        diagnostics: Arc::from(diagnostics.into_boxed_slice()),
    }
}

/// Calculates visibility for a behavior member.
pub fn behavior_member_visibility(member: &BehaviorMember) -> MemberVisibility {
    let (name, attributes) = match member {
        BehaviorMember::Method(item) => (Some(item.name.as_str()), item.attributes.as_slice()),
        BehaviorMember::Getter(item) => (Some(item.name.as_str()), item.attributes.as_slice()),
        BehaviorMember::Setter(item) => (Some(item.name.as_str()), item.attributes.as_slice()),
        BehaviorMember::Index(item) => (None, item.attributes.as_slice()),
    };
    if name.is_some_and(|name| name.starts_with("_$")) {
        MemberVisibility::Internal
    } else if attributes.iter().any(|attribute| attribute.name == "private") {
        MemberVisibility::Private
    } else if attributes.iter().any(|attribute| attribute.name == "protected") {
        MemberVisibility::Protected
    } else {
        MemberVisibility::Public
    }
}

/// Resolves the target of an `impl` statement.
pub fn resolve_inherent_impl_target(
    ctx: &mut CheckingContext<'_>,
    impl_id: &ImplId,
    impl_def: &ImplDef,
) -> Result<ResolvedInherentImplTarget, Vec<SemanticDiagnostic>> {
    let mut diagnostics = Vec::new();
    let source = SemanticSourceSpan::new(ctx.current_module.clone(), impl_def.range);

    // 1. Build impl-owned generic parameters
    let mut impl_type_parameter_ids = Vec::new();
    let mut impl_type_parameter_map: HashMap<String, TypeLevelBinding> = HashMap::new();

    for (index, param_syntax) in impl_def.generic_parameters.iter().enumerate() {
        let param_data = TypeParameterData::new(
            TypeParameterOwner::Impl(impl_id.clone()),
            index as u32,
            param_syntax.name.clone(),
            KindId::TYPE,
        );
        let param_id = ctx.store.intern_type_parameter(param_data);
        impl_type_parameter_ids.push(param_id);
        let binding = type_level_binding_for_parameter(ctx.store, param_id);
        impl_type_parameter_map.insert(param_syntax.name.clone(), binding);
    }

    let impl_generic_signature = if impl_type_parameter_ids.is_empty() {
        None
    } else {
        Some(GenericSignature::new(
            TypeParameterOwner::Impl(impl_id.clone()),
            impl_type_parameter_ids.clone().into_boxed_slice(),
        ))
    };

    // 2. Reject non-empty where clause with C2.P3 deferred diagnostic
    if let Some(ref where_clause) = impl_def.where_clause {
        if !where_clause.constraints.is_empty() {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::ImplWhereClauseUnsupported,
                "where clauses on inherent impls are deferred to LANG005.C2.P3",
                where_clause.range,
            ));
        }
    }

    // 3. Resolve target type annotation
    let parent_resolver = ctx.resolver.clone();
    let impl_resolver = ScopedTypeResolver {
        parent: &parent_resolver,
        type_parameters: impl_type_parameter_map.clone(),
    };
    let formation_site = TypeFormationSite::module(ctx.current_module.clone());

    // Check for exact enum case target syntax
    if let TypeAnnotationExpr::ExactEnumCase {
        enum_target,
        variant_name,
        variant_name_range,
        generic_arguments,
        payload_shape,
        range,
    } = &impl_def.target.expr
    {
        let knowledge = resolve_type_annotation(
            ctx.store,
            ctx.declarations,
            &impl_resolver,
            &formation_site,
            enum_target,
            &mut diagnostics,
        );
        let TypeKnowledge::Known(evidence) = knowledge else {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::ImplTargetNotNominal,
                "inherent impl target enum must be a nominal type declaration",
                enum_target.range,
            ));
            return Err(diagnostics);
        };
        let enum_ty = evidence.ty();
        let (enum_decl, enum_args) = match ctx.store.get(enum_ty).clone() {
            TypeData::Nominal { declaration } => (declaration, Vec::new()),
            TypeData::Applied { origin, arguments } => {
                let orig_decl = match ctx.store.get(origin) {
                    TypeData::Nominal { declaration } => declaration.clone(),
                    _ => {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            ctx.current_module.clone(),
                            DiagnosticCode::ImplTargetNotNominal,
                            "inherent impl target enum must be a nominal type declaration",
                            enum_target.range,
                        ));
                        return Err(diagnostics);
                    }
                };
                (orig_decl, arguments.into_vec())
            }
            _ => {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplTargetNotNominal,
                    "inherent impl target enum must be a nominal type declaration",
                    enum_target.range,
                ));
                return Err(diagnostics);
            }
        };

        // Same-module check
        if enum_decl.module != impl_id.module {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::ImplForeignTarget,
                format!("cannot define inherent impl for foreign declaration `{}` in module `{}`", enum_decl.name, enum_decl.module),
                enum_target.range,
            ));
            return Err(diagnostics);
        }

        // Type alias check
        if ctx.resolver.resolve_alias_form(&enum_decl).is_some() {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::ImplTargetTypeAlias,
                format!("cannot define inherent impl for type alias `{}`", enum_decl.name),
                enum_target.range,
            ));
            return Err(diagnostics);
        }

        // Must be an enum declaration
        let Some(enum_info) = ctx.enum_info(&enum_decl) else {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::ImplTargetNotNominal,
                format!("declaration `{}` is not an enum declaration", enum_decl.name),
                enum_target.range,
            ));
            return Err(diagnostics);
        };
        let enum_info = enum_info.clone();

        // Derive selector and VariantId
        let selector = phalcom_ast::selector::selector_from_exact_case_target(variant_name, payload_shape.as_ref());
        let variant_id = VariantId::new(enum_decl.clone(), selector.clone());

        // Check if variant exists
        let variant_info = ctx.variant_info(&variant_id).cloned();
        if variant_info.is_none() && !enum_info.variants.contains(&variant_id) {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::EnumRequirementMissing,
                format!("enum `{}` has no variant matching `{}`", enum_decl.name, selector.encode()),
                *variant_name_range,
            ));
            return Err(diagnostics);
        }

        // Covering generic check
        let enum_decl_sig = ctx.declaration_generic_signature(&enum_decl);
        let empty_params: [TypeParameterId; 0] = [];
        let decl_params = enum_decl_sig.as_ref().map_or(&empty_params[..], |s| &s.parameters);
        let variant_constructor_sig = variant_info.as_ref().and_then(|v| v.constructor.as_ref().and_then(|c| c.generic_signature.as_ref()));
        let variant_params = variant_constructor_sig.map_or(&empty_params[..], |s| &s.parameters);

        if enum_args.len() != decl_params.len() {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::ImplSpecializedTargetUnsupported,
                format!("generic arity mismatch for enum `{}` in exact-case target: expected {} arguments, got {}", enum_decl.name, decl_params.len(), enum_args.len()),
                enum_target.range,
            ));
            return Err(diagnostics);
        }

        if generic_arguments.len() != variant_params.len() {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::ImplSpecializedTargetUnsupported,
                format!("generic arity mismatch for variant `{}` in exact-case target: expected {} arguments, got {}", variant_id.selector.encode(), variant_params.len(), generic_arguments.len()),
                *range,
            ));
            return Err(diagnostics);
        }

        let total_target_params = decl_params.len() + variant_params.len();
        if impl_type_parameter_ids.len() != total_target_params {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::ImplSpecializedTargetUnsupported,
                format!("exact-case impl on `{}` declares {} generic parameters but target requires {}", variant_id.selector.encode(), impl_type_parameter_ids.len(), total_target_params),
                impl_def.range,
            ));
            return Err(diagnostics);
        }

        let mut impl_to_decl = HashMap::new();
        let mut decl_to_impl = HashMap::new();
        let mut used_impl_params = std::collections::HashSet::new();

        // 1. Process enum_args against decl_params
        for (i, &arg_ty) in enum_args.iter().enumerate() {
            let decl_param = decl_params[i];
            match ctx.store.get(arg_ty) {
                TypeData::Parameter(p_id) => {
                    let p_data = ctx.store.type_parameter(*p_id);
                    if let TypeParameterOwner::Impl(ref owner_impl) = p_data.owner {
                        if owner_impl == impl_id {
                            if !used_impl_params.insert(*p_id) {
                                diagnostics.push(SemanticDiagnostic::error_in(
                                    ctx.current_module.clone(),
                                    DiagnosticCode::ImplSpecializedTargetUnsupported,
                                    format!("repeated type parameter `{}` in generic impl target is deferred to LANG005.C2.P3", p_data.name),
                                    enum_target.range,
                                ));
                                return Err(diagnostics);
                            }
                            impl_to_decl.insert(*p_id, decl_param);
                            decl_to_impl.insert(decl_param, *p_id);
                            continue;
                        }
                    }
                    diagnostics.push(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::ImplSpecializedTargetUnsupported,
                        format!("type parameter `{}` does not belong to this impl block", p_data.name),
                        enum_target.range,
                    ));
                    return Err(diagnostics);
                }
                _ => {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::ImplSpecializedTargetUnsupported,
                        "specialized generic inherent impl target is deferred to LANG005.C2.P3",
                        enum_target.range,
                    ));
                    return Err(diagnostics);
                }
            }
        }

        // 2. Process generic_arguments against variant_params
        for (i, gen_arg_syntax) in generic_arguments.iter().enumerate() {
            let var_param = variant_params[i];
            let gen_arg_res = resolve_type_annotation(
                ctx.store,
                ctx.declarations,
                &impl_resolver,
                &formation_site,
                gen_arg_syntax,
                &mut diagnostics,
            );
            let TypeKnowledge::Known(arg_ev) = gen_arg_res else {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplSpecializedTargetUnsupported,
                    "invalid generic argument in exact case impl target",
                    gen_arg_syntax.range,
                ));
                return Err(diagnostics);
            };
            match ctx.store.get(arg_ev.ty()) {
                TypeData::Parameter(p_id) => {
                    let p_data = ctx.store.type_parameter(*p_id);
                    if let TypeParameterOwner::Impl(ref owner_impl) = p_data.owner {
                        if owner_impl == impl_id {
                            if !used_impl_params.insert(*p_id) {
                                diagnostics.push(SemanticDiagnostic::error_in(
                                    ctx.current_module.clone(),
                                    DiagnosticCode::ImplSpecializedTargetUnsupported,
                                    format!("repeated type parameter `{}` in generic impl target is deferred to LANG005.C2.P3", p_data.name),
                                    gen_arg_syntax.range,
                                ));
                                return Err(diagnostics);
                            }
                            impl_to_decl.insert(*p_id, var_param);
                            decl_to_impl.insert(var_param, *p_id);
                            continue;
                        }
                    }
                    diagnostics.push(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::ImplSpecializedTargetUnsupported,
                        format!("type parameter `{}` does not belong to this impl block", p_data.name),
                        gen_arg_syntax.range,
                    ));
                    return Err(diagnostics);
                }
                _ => {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::ImplSpecializedTargetUnsupported,
                        "specialized generic inherent impl target is deferred to LANG005.C2.P3",
                        gen_arg_syntax.range,
                    ));
                    return Err(diagnostics);
                }
            }
        }

        // Check unused parameters
        for &impl_param in &impl_type_parameter_ids {
            if !used_impl_params.contains(&impl_param) {
                let p_name = &ctx.store.type_parameter(impl_param).name;
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplUnusedTypeParameter,
                    format!("type parameter `{}` is not used in inherent impl target", p_name),
                    impl_def.range,
                ));
                return Err(diagnostics);
            }
        }

        let applicability = if total_target_params == 0 {
            InherentImplApplicability::Unconditional
        } else {
            InherentImplApplicability::Covering(CoveringImplSubstitution::new(impl_to_decl, decl_to_impl))
        };

        let target_type = variant_info.as_ref().map_or(enum_ty, |v| v.exact_case_template);

        return Ok(ResolvedInherentImplTarget {
            id: impl_id.clone(),
            target: InherentImplTarget::ExactEnumCase(variant_id),
            target_type,
            generic_signature: impl_generic_signature,
            applicability,
            source,
            diagnostics: diagnostics.into_boxed_slice(),
        });
    }

    if let TypeAnnotationExpr::Reference(sym) = &impl_def.target.expr {
        if sym.members.is_empty() {
            let sym_decl = DeclarationId::new(ctx.current_module.clone(), sym.root.clone().into());
            if ctx.resolver.resolve_alias_form(&sym_decl).is_some() {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplTargetTypeAlias,
                    format!("cannot define inherent impl for type alias `{}`", sym.root),
                    impl_def.target.range,
                ));
                return Err(diagnostics);
            }
        }
    }

    let knowledge = resolve_type_annotation(
        ctx.store,
        ctx.declarations,
        &impl_resolver,
        &formation_site,
        &impl_def.target,
        &mut diagnostics,
    );

    let TypeKnowledge::Known(evidence) = knowledge else {
        diagnostics.push(SemanticDiagnostic::error_in(
            ctx.current_module.clone(),
            DiagnosticCode::ImplTargetNotNominal,
            "inherent impl target must be a nominal type declaration (class, data, or enum)",
            impl_def.target.range,
        ));
        return Err(diagnostics);
    };

    let target_type = evidence.ty();

    // 4. Validate nominal declaration target and compute covering applicability
    let (target_decl, applicability) = match ctx.store.get(target_type).clone() {
        TypeData::Nominal { declaration } => {
            // Check same module rule
            if declaration.module != impl_id.module {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplForeignTarget,
                    format!("cannot define inherent impl for foreign declaration `{}` in module `{}`", declaration.name, declaration.module),
                    impl_def.target.range,
                ));
                return Err(diagnostics);
            }

            // Check type alias
            if ctx.resolver.resolve_alias_form(&declaration).is_some() {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplTargetTypeAlias,
                    format!("cannot define inherent impl for type alias `{}`", declaration.name),
                    impl_def.target.range,
                ));
                return Err(diagnostics);
            }

            let decl_sig = ctx.declaration_generic_signature(&declaration);
            let decl_param_count = decl_sig.map_or(0, |s| s.parameters.len());

            if decl_param_count == 0 && impl_type_parameter_ids.is_empty() {
                (declaration, InherentImplApplicability::Unconditional)
            } else {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplSpecializedTargetUnsupported,
                    format!("unapplied generic declaration `{}` in inherent impl target is invalid", declaration.name),
                    impl_def.target.range,
                ));
                return Err(diagnostics);
            }
        }
        TypeData::Applied { origin, arguments } => {
            let orig_decl = match ctx.store.get(origin) {
                TypeData::Nominal { declaration } => declaration.clone(),
                _ => {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::ImplTargetNotNominal,
                        "inherent impl target must be a nominal type declaration",
                        impl_def.target.range,
                    ));
                    return Err(diagnostics);
                }
            };

            // Check same module rule
            if orig_decl.module != impl_id.module {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplForeignTarget,
                    format!("cannot define inherent impl for foreign declaration `{}` in module `{}`", orig_decl.name, orig_decl.module),
                    impl_def.target.range,
                ));
                return Err(diagnostics);
            }

            // Check type alias
            if ctx.resolver.resolve_alias_form(&orig_decl).is_some() {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplTargetTypeAlias,
                    format!("cannot define inherent impl for type alias `{}`", orig_decl.name),
                    impl_def.target.range,
                ));
                return Err(diagnostics);
            }

            let decl_sig = ctx.declaration_generic_signature(&orig_decl);
            let Some(decl_sig) = decl_sig else {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplSpecializedTargetUnsupported,
                    format!("declaration `{}` is not generic but received type arguments", orig_decl.name),
                    impl_def.target.range,
                ));
                return Err(diagnostics);
            };

            let decl_params = decl_sig.parameters.clone();

            if arguments.len() != decl_params.len() || impl_type_parameter_ids.len() != decl_params.len() {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplSpecializedTargetUnsupported,
                    format!("generic arity mismatch for inherent impl on `{}`: target has {} parameters, impl has {}", orig_decl.name, decl_params.len(), impl_type_parameter_ids.len()),
                    impl_def.target.range,
                ));
                return Err(diagnostics);
            }

            let mut impl_to_decl = HashMap::new();
            let mut decl_to_impl = HashMap::new();
            let mut used_impl_params = std::collections::HashSet::new();

            for (i, &arg_ty) in arguments.iter().enumerate() {
                let decl_param = decl_params[i];
                match ctx.store.get(arg_ty) {
                    TypeData::Parameter(p_id) => {
                        let p_data = ctx.store.type_parameter(*p_id);
                        if let TypeParameterOwner::Impl(ref owner_impl) = p_data.owner {
                            if owner_impl == impl_id {
                                if !used_impl_params.insert(*p_id) {
                                    diagnostics.push(SemanticDiagnostic::error_in(
                                        ctx.current_module.clone(),
                                        DiagnosticCode::ImplSpecializedTargetUnsupported,
                                        format!("repeated type parameter `{}` in generic impl target is deferred to LANG005.C2.P3", p_data.name),
                                        impl_def.target.range,
                                    ));
                                    return Err(diagnostics);
                                }
                                impl_to_decl.insert(*p_id, decl_param);
                                decl_to_impl.insert(decl_param, *p_id);
                                continue;
                            }
                        }
                        diagnostics.push(SemanticDiagnostic::error_in(
                            ctx.current_module.clone(),
                            DiagnosticCode::ImplSpecializedTargetUnsupported,
                            format!("type parameter `{}` does not belong to this impl block", p_data.name),
                            impl_def.target.range,
                        ));
                        return Err(diagnostics);
                    }
                    _ => {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            ctx.current_module.clone(),
                            DiagnosticCode::ImplSpecializedTargetUnsupported,
                            "specialized generic inherent impl target is deferred to LANG005.C2.P3",
                            impl_def.target.range,
                        ));
                        return Err(diagnostics);
                    }
                }
            }

            // Check for unused impl parameters
            for &impl_param in &impl_type_parameter_ids {
                if !used_impl_params.contains(&impl_param) {
                    let p_name = &ctx.store.type_parameter(impl_param).name;
                    diagnostics.push(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::ImplUnusedTypeParameter,
                        format!("type parameter `{}` is not used in inherent impl target", p_name),
                        impl_def.range,
                    ));
                    return Err(diagnostics);
                }
            }

            let covering = CoveringImplSubstitution::new(impl_to_decl, decl_to_impl);
            (orig_decl, InherentImplApplicability::Covering(covering))
        }
        _ => {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::ImplTargetNotNominal,
                "inherent impl target must be a nominal type declaration (class, data, or enum)",
                impl_def.target.range,
            ));
            return Err(diagnostics);
        }
    };

    Ok(ResolvedInherentImplTarget {
        id: impl_id.clone(),
        target: InherentImplTarget::Declaration(target_decl),
        target_type,
        generic_signature: impl_generic_signature,
        applicability,
        source,
        diagnostics: diagnostics.into_boxed_slice(),
    })
}

/// Applies a type substitution to all types referenced in a `CallableSemanticSignature`.
pub fn apply_covering_to_signature(
    store: &mut TypeStore,
    subst: &TypeSubstitution,
    mut signature: CallableSemanticSignature,
) -> CallableSemanticSignature {
    if subst.is_empty() {
        return signature;
    }

    let mut new_params = Vec::with_capacity(signature.parameters.len());
    for mut param in signature.parameters.into_vec() {
        if let DeclaredTypeState::Known(TypeTerm::Canonical(ty)) = param.declared_type.state {
            let new_ty = subst.apply(store, ty);
            param.declared_type.state = DeclaredTypeState::Known(TypeTerm::Canonical(new_ty));
        }
        new_params.push(param);
    }
    signature.parameters = new_params.into_boxed_slice();

    if let DeclaredTypeState::Known(TypeTerm::Canonical(ret_ty)) = signature.declared_return.state {
        let new_ret_ty = subst.apply(store, ret_ty);
        signature.declared_return.state = DeclaredTypeState::Known(TypeTerm::Canonical(new_ret_ty));
    }

    signature
}

/// Checks if a type contains any parameters owned by `impl_id`.
pub fn type_contains_impl_param(store: &TypeStore, ty: TypeId, impl_id: &ImplId) -> bool {
    match store.get(ty) {
        TypeData::Parameter(p) => {
            let data = store.type_parameter(*p);
            matches!(&data.owner, TypeParameterOwner::Impl(id) if id == impl_id)
        }
        TypeData::Applied { origin, arguments } => {
            type_contains_impl_param(store, *origin, impl_id)
                || arguments.iter().any(|&arg| type_contains_impl_param(store, arg, impl_id))
        }
        TypeData::Union(members) => members.iter().any(|&m| type_contains_impl_param(store, m, impl_id)),
        TypeData::Tuple(elements) => elements.iter().any(|e| type_contains_impl_param(store, e.ty, impl_id)),
        TypeData::Record(row) => {
            let row_data = store.record_row(*row);
            row_data.fields.iter().any(|f| type_contains_impl_param(store, f.ty, impl_id))
        }
        TypeData::Callable(c) => {
            c.parameters.iter().any(|p| type_contains_impl_param(store, p.ty, impl_id))
                || type_contains_impl_param(store, c.return_type, impl_id)
        }
        _ => false,
    }
}

/// Builds the canonical `InherentImplContribution` from an `ImplDef`.
pub fn build_inherent_impl_contribution(
    ctx: &mut CheckingContext<'_>,
    impl_id: &ImplId,
    impl_def: &ImplDef,
) -> InherentImplContribution {
    let mut diagnostics = Vec::new();
    let source = SemanticSourceSpan::new(ctx.current_module.clone(), impl_def.range);

    let resolved_target = match resolve_inherent_impl_target(ctx, impl_id, impl_def) {
        Ok(target) => {
            diagnostics.extend(target.diagnostics.clone().into_vec());
            target
        }
        Err(diags) => {
            diagnostics.extend(diags);
            return InherentImplContribution {
                id: impl_id.clone(),
                target: InherentImplTarget::Declaration(DeclarationId::new(ctx.current_module.clone(), "_".into())),
                generic_signature: None,
                covering: None,
                members: Box::new([]),
                source,
                diagnostics: diagnostics.into_boxed_slice(),
            };
        }
    };

    let target_owner = resolved_target.target.to_callable_owner();
    let is_exact_case = matches!(resolved_target.target, InherentImplTarget::ExactEnumCase(_));
    let is_enum_root = match &resolved_target.target {
        InherentImplTarget::Declaration(decl_id) => ctx.enum_info(decl_id).is_some(),
        InherentImplTarget::ExactEnumCase(_) => false,
    };

    // Build the impl-scoped resolver
    let mut impl_type_parameter_map: HashMap<String, TypeLevelBinding> = HashMap::new();
    if let Some(ref sig) = resolved_target.generic_signature {
        for &param_id in sig.parameters.iter() {
            let name = ctx.store.type_parameter(param_id).name.to_string();
            let binding = type_level_binding_for_parameter(ctx.store, param_id);
            impl_type_parameter_map.insert(name, binding);
        }
    }

    let parent_resolver = ctx.resolver.clone();
    let impl_resolver = ScopedTypeResolver {
        parent: &parent_resolver,
        type_parameters: impl_type_parameter_map,
    };

    let covering_subst = match &resolved_target.applicability {
        InherentImplApplicability::Covering(cov) => Some(cov.to_type_substitution(ctx.store)),
        InherentImplApplicability::Unconditional => None,
    };

    let mut members = Vec::new();

    for (source_member_idx, member) in impl_def.members.iter().enumerate() {
        let syntax = CallableSyntaxRef::from(member);
        let visibility = behavior_member_visibility(member);

        // Reject constructors in impl
        let is_constructor = match member {
            BehaviorMember::Method(m) => m.is_constructor || m.attributes.iter().any(|a| a.name == "constructor"),
            _ => false,
        };
        if is_constructor {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::ImplConstructorUnsupported,
                "constructor lifecycle methods cannot be declared in inherent impl blocks",
                syntax.range(),
            ));
            continue;
        }

        let is_class_side = syntax.attributes().iter().any(|attr| attr.name == "class")
            || match member {
                BehaviorMember::Method(m) => m.is_static,
                BehaviorMember::Getter(g) => g.is_static,
                BehaviorMember::Setter(s) => s.is_static,
                BehaviorMember::Index(_) => false,
            };

        if is_exact_case {
            if is_class_side {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::EnumCaseStaticBehaviorUnsupported,
                    "case-local behavior on exact enum cases cannot be class-side",
                    syntax.range(),
                ));
                continue;
            }
            if !syntax.has_body() {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::EnumCaseDeclarationOnlyBehavior,
                    "case-local behavior on exact enum cases must have an executable body",
                    syntax.range(),
                ));
                continue;
            }
        }

        let is_requirement = if is_enum_root {
            if !syntax.has_body() {
                if is_class_side {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::EnumCaseStaticBehaviorUnsupported,
                        "class-side requirement is not supported on enums",
                        syntax.range(),
                    ));
                    continue;
                }
                true
            } else {
                false
            }
        } else if !is_exact_case {
            if !syntax.has_body() {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplBodylessMemberUnsupported,
                    "bodyless member declarations in inherent impl blocks are not supported",
                    syntax.range(),
                ));
                continue;
            }
            false
        } else {
            false
        };

        let side = if is_class_side { DispatchSide::Class } else { DispatchSide::Instance };

        let Some(raw_sig) = semantic_signature_for_syntax_with_resolver(ctx, &target_owner, &impl_resolver, syntax, side) else {
            continue;
        };

        // Canonicalize signature into target declaration parameter space
        let final_sig = if let Some(ref subst) = covering_subst {
            apply_covering_to_signature(ctx.store, subst, raw_sig)
        } else {
            raw_sig
        };

        // Invariant assertion: published unconditional surface signatures contain no free target-head TypeParameterOwner::Impl(current_impl)
        for param in final_sig.parameters.iter() {
            if let DeclaredTypeState::Known(TypeTerm::Canonical(ty)) = param.declared_type.state {
                debug_assert!(
                    !type_contains_impl_param(ctx.store, ty, impl_id),
                    "published signature parameter contains free impl parameter"
                );
            }
        }
        if let DeclaredTypeState::Known(TypeTerm::Canonical(ret_ty)) = final_sig.declared_return.state {
            debug_assert!(
                !type_contains_impl_param(ctx.store, ret_ty, impl_id),
                "published signature return type contains free impl parameter"
            );
        }

        members.push(InherentMemberContribution {
            callable: final_sig.callable.clone(),
            signature: final_sig,
            visibility,
            source_member: source_member_idx,
            is_requirement,
        });
    }

    InherentImplContribution {
        id: impl_id.clone(),
        target: resolved_target.target,
        generic_signature: resolved_target.generic_signature,
        covering: match resolved_target.applicability {
            InherentImplApplicability::Covering(cov) => Some(cov),
            InherentImplApplicability::Unconditional => None,
        },
        members: members.into_boxed_slice(),
        source,
        diagnostics: diagnostics.into_boxed_slice(),
    }
}

