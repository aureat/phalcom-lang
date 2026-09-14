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
use std::collections::{BTreeSet, HashMap};

/// Computes the conditional inherent members selected for one canonical
/// receiver form. This is the shared semantic query used by non-checker
/// consumers; callers may enumerate or render the result, but must not run a
/// second applicability solver.
pub fn receiver_effective_conditional_members(
    store: &mut TypeStore,
    hierarchy: &dyn crate::types::relation::TypeHierarchy,
    dispatch: &crate::dispatch::SurfaceDispatchResolver,
    receiver_type: TypeId,
    lookup_owner: &DeclarationId,
    side: DispatchSide,
    ambient_constraints: &[crate::types::parameter::GenericConstraint],
) -> Vec<(DeclarationId, ConditionalInherentMember)> {
    let mut selected = BTreeSet::new();
    let mut result = Vec::new();
    let ordinary_selectors = dispatch
        .dispatch_owners(hierarchy, lookup_owner, side)
        .into_iter()
        .filter_map(|owner| dispatch.surface(&owner.declaration).map(|surface| surface.surface(owner.side)))
        .flat_map(|surface| surface.callable_signatures.keys().cloned())
        .collect::<BTreeSet<_>>();

    let exact_variant = match store.get(receiver_type) {
        TypeData::ExactCase { variant, .. } => Some(store.variant_identity(*variant).clone()),
        _ => None,
    };
    if let Some(variant) = exact_variant
        && let Some(set) = dispatch.get_conditional_members(&InherentImplTarget::ExactEnumCase(variant.clone()))
    {
        for member in set.members.iter().filter(|member| member.callable.side == side) {
            let selector = member.callable.selector.clone();
            if ordinary_selectors.contains(&selector) || selected.contains(&selector) {
                continue;
            }
            if matches!(
                check_impl_domain_applicability(
                    store,
                    hierarchy,
                    &member.domain,
                    receiver_type,
                    receiver_type,
                    ambient_constraints,
                ),
                ImplApplicabilityResult::Applicable(_)
            ) {
                selected.insert(selector);
                result.push((variant.owner.clone(), member.clone()));
            }
        }
    }

    let control = crate::checker::context::CheckerControl::default();
    for owner in dispatch.dispatch_owners(hierarchy, lookup_owner, side) {
        let Ok(receiver_spec) = crate::types::specialization::specialize_receiver_to_owner(
            store,
            hierarchy,
            receiver_type,
            &owner.declaration,
            &control,
        ) else {
            continue;
        };
        let owner_view = receiver_spec
            .path
            .last()
            .map(|step| step.specialized_form)
            .unwrap_or(receiver_type);
        let Some(set) = dispatch.get_conditional_members(&InherentImplTarget::Declaration(owner.declaration.clone())) else {
            continue;
        };
        for member in set.members.iter().filter(|member| member.callable.side == owner.side) {
            let selector = member.callable.selector.clone();
            if ordinary_selectors.contains(&selector) || selected.contains(&selector) {
                continue;
            }
            if matches!(
                check_impl_domain_applicability(
                    store,
                    hierarchy,
                    &member.domain,
                    receiver_type,
                    owner_view,
                    ambient_constraints,
                ),
                ImplApplicabilityResult::Applicable(_)
            ) {
                selected.insert(selector);
                result.push((owner.declaration.clone(), member.clone()));
            }
        }
    }

    result
}

/// Explicit bijection mapping between impl type parameters and canonical target declaration parameters.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoveringImplSubstitution {
    /// Mapping: impl-owned TypeParameterId -> declaration-owned TypeParameterId
    pub impl_to_decl: HashMap<TypeParameterId, TypeParameterId>,
    /// Inverse: declaration-owned TypeParameterId -> impl-owned TypeParameterId
    pub decl_to_impl: HashMap<TypeParameterId, TypeParameterId>,
}

impl CoveringImplSubstitution {
    pub fn new(impl_to_decl: HashMap<TypeParameterId, TypeParameterId>, decl_to_impl: HashMap<TypeParameterId, TypeParameterId>) -> Self {
        Self { impl_to_decl, decl_to_impl }
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

use crate::types::parameter::GenericConstraint;
use crate::types::relation::TypeHierarchy;

/// First-class domain representing the specialized or constrained applicability scope of an inherent `impl` block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InherentImplDomain {
    pub impl_id: ImplId,
    pub target: InherentImplTarget,
    pub head_type: TypeId,
    pub generic_signature: Option<GenericSignature>,
    pub constraints: Box<[GenericConstraint]>,
}

/// Applicability regime of an inherent impl block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InherentImplApplicability {
    /// Unconditional non-generic target (e.g. `impl User`).
    Unconditional,
    /// Covering generic target (e.g. `impl<A, B> Pair<A, B>` or `impl<A, B> Pair<B, A>`).
    Covering(CoveringImplSubstitution),
    /// Receiver-specialized or constraint-conditioned target (e.g. `impl Point<Int>`, `impl<T> Pair<T, T>`, `impl<T> Point<T> where T <: Number`).
    Conditional(Arc<InherentImplDomain>),
}

/// Target of an inherent implementation block (nominal declaration or exact enum case).
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
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
    pub domain: Option<Arc<InherentImplDomain>>,
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
use crate::types::environment::TypeEnvironment;
use phalcom_common::selector::Selector;
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

/// A conditional member contributed by a specialized or constrained inherent impl block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConditionalInherentMember {
    pub impl_id: ImplId,
    pub domain: Arc<InherentImplDomain>,
    pub callable: CallableId,
    pub signature_template: CallableSemanticSignature,
    pub visibility: MemberVisibility,
    pub source_member: usize,
    pub is_requirement: bool,
}

/// Target-indexed set of conditional inherent members.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConditionalInherentMemberSet {
    pub target: Option<InherentImplTarget>,
    pub members: Vec<ConditionalInherentMember>,
    pub by_selector: HashMap<(DispatchSide, Selector), Vec<usize>>,
}

impl ConditionalInherentMemberSet {
    pub fn new(target: Option<InherentImplTarget>) -> Self {
        Self {
            target,
            members: Vec::new(),
            by_selector: HashMap::new(),
        }
    }

    pub fn add_member(&mut self, member: ConditionalInherentMember) {
        let side = member.signature_template.side;
        let selector = member.signature_template.callable.selector.clone();
        let index = self.members.len();
        self.members.push(member);
        self.by_selector.entry((side, selector)).or_default().push(index);
    }

    pub fn get_members(&self, side: DispatchSide, selector: &Selector) -> Option<&[usize]> {
        self.by_selector.get(&(side, selector.clone())).map(|v| v.as_slice())
    }

    pub fn get_member(&self, side: DispatchSide, selector: &Selector) -> Option<&ConditionalInherentMember> {
        let indices = self.get_members(side, selector)?;
        indices.first().map(|&idx| &self.members[idx])
    }
}

/// Proof evidence capturing the specialized instantiation of an inherent impl domain for a receiver.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InherentImplSpecialization {
    pub impl_id: ImplId,
    pub receiver: TypeId,
    pub owner_view: TypeId,
    pub bindings: HashMap<TypeParameterId, TypeId>,
    pub environment: TypeEnvironment,
}

/// Outcome of checking applicability of an inherent impl domain to a receiver.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImplApplicabilityResult {
    Applicable(InherentImplSpecialization),
    NotApplicable,
    Blocked,
}

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
    pub conditional_members: Arc<ConditionalInherentMemberSet>,
    pub definitions: BTreeMap<CallableId, EffectiveCallableDefinition>,
    pub diagnostics: Arc<[SemanticDiagnostic]>,
}

impl EffectiveSurfaceProduct {
    pub fn new(
        owner: DeclarationId,
        surface: Arc<DeclarationSurface>,
        conditional_members: Arc<ConditionalInherentMemberSet>,
        definitions: BTreeMap<CallableId, EffectiveCallableDefinition>,
        diagnostics: Arc<[SemanticDiagnostic]>,
    ) -> Self {
        Self {
            owner,
            surface,
            conditional_members,
            definitions,
            diagnostics,
        }
    }
}

/// Internal origin of a reserved member entry during conflict checking.
#[derive(Clone, Debug, Eq, PartialEq)]
enum EffectiveMemberOrigin {
    PrimaryCallable(CallableId),
    InherentCallable { callable: CallableId, impl_id: ImplId },
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
    let mut conditional_members = ConditionalInherentMemberSet::new(Some(InherentImplTarget::Declaration(owner.clone())));
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
        let ty = primary_surface
            .instance
            .fields
            .get(name)
            .cloned()
            .unwrap_or(TypeKnowledge::Dynamic(crate::types::evidence::DynamicReason::ExplicitEscape));
        let visibility = primary_surface.instance.field_visibility.get(name).copied().unwrap_or(MemberVisibility::Public);
        reserved_fields.insert((DispatchSide::Instance, name.clone()), (field_id.clone(), visibility));
        effective_surface
            .instance
            .add_field_with_visibility(Some(owner), DispatchSide::Instance, name, ty, visibility);
    }
    for (name, field_id) in &primary_surface.class.fields_by_name {
        let ty = primary_surface
            .class
            .fields
            .get(name)
            .cloned()
            .unwrap_or(TypeKnowledge::Dynamic(crate::types::evidence::DynamicReason::ExplicitEscape));
        let visibility = primary_surface.class.field_visibility.get(name).copied().unwrap_or(MemberVisibility::Public);
        reserved_fields.insert((DispatchSide::Class, name.clone()), (field_id.clone(), visibility));
        effective_surface
            .class
            .add_field_with_visibility(Some(owner), DispatchSide::Class, name, ty, visibility);
    }

    // 4. Register primary callables
    for (side, surface) in [
        (DispatchSide::Instance, &primary_surface.instance),
        (DispatchSide::Class, &primary_surface.class),
    ] {
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
                            format!("declaration member `{}` conflicts with data component index {}", selector, comp_id.index),
                            phalcom_common::range::SourceRange::default(),
                        ));
                        continue;
                    }
                    EffectiveMemberOrigin::VariantConstructor(v_id) => {
                        diagnostics.push(SemanticDiagnostic::error_in(
                            owner.module.clone(),
                            DiagnosticCode::EnumFamilyCategoryConflict,
                            format!("class callable `{}` conflicts with variant constructor `{}`", selector, v_id.selector),
                            phalcom_common::range::SourceRange::default(),
                        ));
                        continue;
                    }
                    _ => {}
                }
            }

            reserved_selectors.insert((side, selector.clone()), EffectiveMemberOrigin::PrimaryCallable(callable_id.clone()));
            effective_surface
                .surface_mut(side)
                .add_callable_with_visibility(Some(owner), side, sig.clone(), visibility);

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

            if let Some(domain) = &contribution.domain {
                conditional_members.add_member(ConditionalInherentMember {
                    impl_id: contribution.id.clone(),
                    domain: domain.clone(),
                    callable: callable_id.clone(),
                    signature_template: member.signature.clone(),
                    visibility: member.visibility,
                    source_member: member.source_member,
                    is_requirement: member.is_requirement,
                });
            } else {
                let projection = crate::checker::declaration_signature::project_semantic_signature(&member.signature);
                effective_surface
                    .surface_mut(side)
                    .add_callable_with_visibility(Some(owner), side, projection, member.visibility);
            }

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
        conditional_members: Arc::new(conditional_members),
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

/// Recursively collects all type parameters owned by `impl_id` in a `TypeId`.
pub fn collect_impl_params_in_type(store: &TypeStore, ty: TypeId, impl_id: &ImplId, out: &mut HashSet<TypeParameterId>) {
    match store.get(ty) {
        TypeData::Parameter(p) => {
            let data = store.type_parameter(*p);
            if matches!(&data.owner, TypeParameterOwner::Impl(id) if id == impl_id) {
                out.insert(*p);
            }
        }
        TypeData::Applied { origin, arguments } => {
            collect_impl_params_in_type(store, *origin, impl_id, out);
            for &arg in arguments.iter() {
                collect_impl_params_in_type(store, arg, impl_id, out);
            }
        }
        TypeData::ExactCase { enum_type, .. } => {
            collect_impl_params_in_type(store, *enum_type, impl_id, out);
        }
        TypeData::Union(members) => {
            for &m in members.iter() {
                collect_impl_params_in_type(store, m, impl_id, out);
            }
        }
        TypeData::Tuple(elements) => {
            for e in elements.iter() {
                collect_impl_params_in_type(store, e.ty, impl_id, out);
            }
        }
        TypeData::Record(row) => {
            let row_data = store.record_row(*row);
            for f in row_data.fields.iter() {
                collect_impl_params_in_type(store, f.ty, impl_id, out);
            }
        }
        TypeData::Callable(c) => {
            for p in c.parameters.iter() {
                collect_impl_params_in_type(store, p.ty, impl_id, out);
            }
            collect_impl_params_in_type(store, c.return_type, impl_id, out);
        }
        _ => {}
    }
}

/// Matches a domain head type structurally against a receiver form and binds impl-owned parameters.
pub fn match_impl_domain_head(
    store: &TypeStore,
    domain_head: TypeId,
    receiver_form: TypeId,
    impl_id: &ImplId,
    bindings: &mut HashMap<TypeParameterId, TypeId>,
) -> bool {
    if domain_head == receiver_form {
        if let TypeData::Parameter(p) = store.get(domain_head) {
            if store.type_parameter(*p).owner == TypeParameterOwner::Impl(impl_id.clone()) {
                if let Some(&existing) = bindings.get(p) {
                    return existing == receiver_form;
                } else {
                    bindings.insert(*p, receiver_form);
                }
            }
        }
        return true;
    }

    match (store.get(domain_head), store.get(receiver_form)) {
        (TypeData::Parameter(p), _) => {
            if store.type_parameter(*p).owner == TypeParameterOwner::Impl(impl_id.clone()) {
                if let Some(&existing) = bindings.get(p) {
                    existing == receiver_form
                } else {
                    bindings.insert(*p, receiver_form);
                    true
                }
            } else {
                domain_head == receiver_form
            }
        }
        (TypeData::Nominal { declaration: d1 }, TypeData::Nominal { declaration: d2 }) => d1 == d2,
        (TypeData::Applied { origin: o1, arguments: a1 }, TypeData::Applied { origin: o2, arguments: a2 }) => {
            if a1.len() != a2.len() {
                return false;
            }
            if !match_impl_domain_head(store, *o1, *o2, impl_id, bindings) {
                return false;
            }
            for (&arg1, &arg2) in a1.iter().zip(a2.iter()) {
                if !match_impl_domain_head(store, arg1, arg2, impl_id, bindings) {
                    return false;
                }
            }
            true
        }
        (TypeData::ExactCase { variant: v1, enum_type: e1 }, TypeData::ExactCase { variant: v2, enum_type: e2 }) => {
            if v1 != v2 {
                return false;
            }
            match_impl_domain_head(store, *e1, *e2, impl_id, bindings)
        }
        (TypeData::Tuple(elems1), TypeData::Tuple(elems2)) => {
            if elems1.len() != elems2.len() {
                return false;
            }
            for (e1, e2) in elems1.iter().zip(elems2.iter()) {
                if e1.label != e2.label {
                    return false;
                }
                if !match_impl_domain_head(store, e1.ty, e2.ty, impl_id, bindings) {
                    return false;
                }
            }
            true
        }
        (TypeData::Record(row1), TypeData::Record(row2)) => {
            let r1 = store.record_row(*row1);
            let r2 = store.record_row(*row2);
            if r1.fields.len() != r2.fields.len() {
                return false;
            }
            for (f1, f2) in r1.fields.iter().zip(r2.fields.iter()) {
                if f1.name != f2.name {
                    return false;
                }
                if !match_impl_domain_head(store, f1.ty, f2.ty, impl_id, bindings) {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}

/// Checks whether an inherent impl domain's substituted generic constraints are satisfied.
pub fn check_impl_domain_constraints(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    domain: &InherentImplDomain,
    bindings: &HashMap<TypeParameterId, TypeId>,
    ambient_constraints: &[GenericConstraint],
) -> bool {
    let mut subst = TypeSubstitution::new();
    for (&p, &t) in bindings {
        subst.bind(p, t);
    }

    for constraint in domain.constraints.iter() {
        match constraint {
            GenericConstraint::Subtype { lower, upper } => {
                let lower_ty = match lower {
                    TypeTerm::Canonical(ty) => subst.apply(store, *ty),
                    _ => return false,
                };
                let upper_ty = match upper {
                    TypeTerm::Canonical(ty) => subst.apply(store, *ty),
                    _ => return false,
                };

                if lower_ty == upper_ty {
                    continue;
                }
                if crate::types::relation::is_subtype(store, hierarchy, lower_ty, upper_ty) {
                    continue;
                }
                let mut proven = false;
                if let TypeData::Parameter(_p) = store.get(lower_ty) {
                    for amb in ambient_constraints {
                        match amb {
                            GenericConstraint::Subtype {
                                lower: amb_lower,
                                upper: amb_upper,
                            } => {
                                if let (TypeTerm::Canonical(al), TypeTerm::Canonical(au)) = (amb_lower, amb_upper) {
                                    if *al == lower_ty && crate::types::relation::is_subtype(store, hierarchy, *au, upper_ty) {
                                        proven = true;
                                        break;
                                    }
                                }
                            }
                            GenericConstraint::Equivalent {
                                left: amb_left,
                                right: amb_right,
                            } => {
                                if let (TypeTerm::Canonical(al), TypeTerm::Canonical(ar)) = (amb_left, amb_right) {
                                    if *al == lower_ty && crate::types::relation::is_subtype(store, hierarchy, *ar, upper_ty) {
                                        proven = true;
                                        break;
                                    }
                                    if *ar == lower_ty && crate::types::relation::is_subtype(store, hierarchy, *al, upper_ty) {
                                        proven = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                if !proven {
                    return false;
                }
            }
            GenericConstraint::Equivalent { left, right } => {
                let left_ty = match left {
                    TypeTerm::Canonical(ty) => subst.apply(store, *ty),
                    _ => return false,
                };
                let right_ty = match right {
                    TypeTerm::Canonical(ty) => subst.apply(store, *ty),
                    _ => return false,
                };
                if left_ty == right_ty {
                    continue;
                }
                let mut proven = false;
                for amb in ambient_constraints {
                    if let GenericConstraint::Equivalent { left: al, right: ar } = amb {
                        if let (TypeTerm::Canonical(l), TypeTerm::Canonical(r)) = (al, ar) {
                            if (*l == left_ty && *r == right_ty) || (*l == right_ty && *r == left_ty) {
                                proven = true;
                                break;
                            }
                        }
                    }
                }
                if !proven {
                    return false;
                }
            }
        }
    }
    true
}

/// Checks applicability of an inherent impl domain to a receiver and owner view.
pub fn check_impl_domain_applicability(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    domain: &InherentImplDomain,
    receiver: TypeId,
    owner_view: TypeId,
    ambient_constraints: &[GenericConstraint],
) -> ImplApplicabilityResult {
    let mut bindings = HashMap::new();
    if !match_impl_domain_head(store, domain.head_type, owner_view, &domain.impl_id, &mut bindings) {
        return ImplApplicabilityResult::NotApplicable;
    }

    if !check_impl_domain_constraints(store, hierarchy, domain, &bindings, ambient_constraints) {
        return ImplApplicabilityResult::NotApplicable;
    }

    let mut env = TypeEnvironment::new();
    for (&p, &t) in &bindings {
        env.bind_param(p, t);
    }
    env.bind_self(owner_view);

    ImplApplicabilityResult::Applicable(InherentImplSpecialization {
        impl_id: domain.impl_id.clone(),
        receiver,
        owner_view,
        bindings,
        environment: env,
    })
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
        let param_data = TypeParameterData::new(TypeParameterOwner::Impl(impl_id.clone()), index as u32, param_syntax.name.clone(), KindId::TYPE);
        let param_id = ctx.store.intern_type_parameter(param_data);
        impl_type_parameter_ids.push(param_id);
        let binding = type_level_binding_for_parameter(ctx.store, param_id);
        impl_type_parameter_map.insert(param_syntax.name.clone(), binding);
    }

    // 2. Resolve where clause constraints
    let parent_resolver = ctx.resolver.clone();
    let impl_resolver = ScopedTypeResolver {
        parent: &parent_resolver,
        type_parameters: impl_type_parameter_map.clone(),
    };
    let formation_site = TypeFormationSite::module(ctx.current_module.clone());

    let mut constraints = Vec::new();
    if let Some(ref where_clause) = impl_def.where_clause {
        for c in &where_clause.constraints {
            match c {
                phalcom_ast::ast::GenericConstraintSyntax::Subtype { lower, upper, range: _ } => {
                    let l_k = resolve_type_annotation(ctx.store, ctx.declarations, &impl_resolver, &formation_site, lower, &mut diagnostics);
                    let u_k = resolve_type_annotation(ctx.store, ctx.declarations, &impl_resolver, &formation_site, upper, &mut diagnostics);
                    if let (TypeKnowledge::Known(l_ev), TypeKnowledge::Known(u_ev)) = (l_k, u_k) {
                        constraints.push(GenericConstraint::Subtype {
                            lower: TypeTerm::Canonical(l_ev.ty()),
                            upper: TypeTerm::Canonical(u_ev.ty()),
                        });
                    }
                }
                phalcom_ast::ast::GenericConstraintSyntax::Equivalent { left, right, range: _ } => {
                    let l_k = resolve_type_annotation(ctx.store, ctx.declarations, &impl_resolver, &formation_site, left, &mut diagnostics);
                    let r_k = resolve_type_annotation(ctx.store, ctx.declarations, &impl_resolver, &formation_site, right, &mut diagnostics);
                    if let (TypeKnowledge::Known(l_ev), TypeKnowledge::Known(r_ev)) = (l_k, r_k) {
                        constraints.push(GenericConstraint::Equivalent {
                            left: TypeTerm::Canonical(l_ev.ty()),
                            right: TypeTerm::Canonical(r_ev.ty()),
                        });
                    }
                }
                phalcom_ast::ast::GenericConstraintSyntax::Invalid { message, range } => {
                    diagnostics.push(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::AnnotationUnresolved,
                        message.clone(),
                        *range,
                    ));
                }
            }
        }
    }

    let impl_generic_signature = if impl_type_parameter_ids.is_empty() && constraints.is_empty() {
        None
    } else {
        Some(GenericSignature::with_constraints(
            TypeParameterOwner::Impl(impl_id.clone()),
            impl_type_parameter_ids.clone().into_boxed_slice(),
            constraints.clone().into_boxed_slice(),
        ))
    };

    // 3. Resolve target type annotation
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
        let knowledge = resolve_type_annotation(ctx.store, ctx.declarations, &impl_resolver, &formation_site, enum_target, &mut diagnostics);
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
                format!(
                    "cannot define inherent impl for foreign declaration `{}` in module `{}`",
                    enum_decl.name, enum_decl.module
                ),
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

        let enum_decl_sig = ctx.declaration_generic_signature(&enum_decl);
        let empty_params: [TypeParameterId; 0] = [];
        let decl_params = enum_decl_sig.as_ref().map_or(&empty_params[..], |s| &s.parameters);
        let variant_constructor_sig = variant_info
            .as_ref()
            .and_then(|v| v.constructor.as_ref().and_then(|c| c.generic_signature.as_ref()));
        let variant_params = variant_constructor_sig.map_or(&empty_params[..], |s| &s.parameters);

        if enum_args.len() != decl_params.len() {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::ImplSpecializedTargetUnsupported,
                format!(
                    "generic arity mismatch for enum `{}` in exact-case target: expected {} arguments, got {}",
                    enum_decl.name,
                    decl_params.len(),
                    enum_args.len()
                ),
                enum_target.range,
            ));
            return Err(diagnostics);
        }

        if generic_arguments.len() != variant_params.len() {
            diagnostics.push(SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                DiagnosticCode::ImplSpecializedTargetUnsupported,
                format!(
                    "generic arity mismatch for variant `{}` in exact-case target: expected {} arguments, got {}",
                    variant_id.selector.encode(),
                    variant_params.len(),
                    generic_arguments.len()
                ),
                *range,
            ));
            return Err(diagnostics);
        }

        // Collect used impl params in target head
        let mut used_impl_params = HashSet::new();
        for &arg_ty in &enum_args {
            collect_impl_params_in_type(ctx.store, arg_ty, impl_id, &mut used_impl_params);
        }

        let mut var_arg_types = Vec::new();
        for gen_arg_syntax in generic_arguments.iter() {
            let gen_arg_res = resolve_type_annotation(ctx.store, ctx.declarations, &impl_resolver, &formation_site, gen_arg_syntax, &mut diagnostics);
            let TypeKnowledge::Known(arg_ev) = gen_arg_res else {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplSpecializedTargetUnsupported,
                    "invalid generic argument in exact case impl target",
                    gen_arg_syntax.range,
                ));
                return Err(diagnostics);
            };
            var_arg_types.push(arg_ev.ty());
            collect_impl_params_in_type(ctx.store, arg_ev.ty(), impl_id, &mut used_impl_params);
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

        let total_target_params = decl_params.len() + variant_params.len();
        let target_type = variant_info.as_ref().map_or(enum_ty, |v| v.exact_case_template);

        // Check if covering bijection or conditional
        let is_covering_bijection = constraints.is_empty()
            && total_target_params == impl_type_parameter_ids.len()
            && enum_args.iter().all(|&a| matches!(ctx.store.get(a), TypeData::Parameter(p) if matches!(&ctx.store.type_parameter(*p).owner, TypeParameterOwner::Impl(id) if id == impl_id)))
            && var_arg_types.iter().all(|&a| matches!(ctx.store.get(a), TypeData::Parameter(p) if matches!(&ctx.store.type_parameter(*p).owner, TypeParameterOwner::Impl(id) if id == impl_id)))
            && used_impl_params.len() == total_target_params;

        let applicability = if total_target_params == 0 && constraints.is_empty() {
            InherentImplApplicability::Unconditional
        } else if is_covering_bijection {
            let mut impl_to_decl = HashMap::new();
            let mut decl_to_impl = HashMap::new();
            for (i, &arg_ty) in enum_args.iter().enumerate() {
                if let TypeData::Parameter(p_id) = ctx.store.get(arg_ty) {
                    impl_to_decl.insert(*p_id, decl_params[i]);
                    decl_to_impl.insert(decl_params[i], *p_id);
                }
            }
            for (i, &arg_ty) in var_arg_types.iter().enumerate() {
                if let TypeData::Parameter(p_id) = ctx.store.get(arg_ty) {
                    impl_to_decl.insert(*p_id, variant_params[i]);
                    decl_to_impl.insert(variant_params[i], *p_id);
                }
            }
            InherentImplApplicability::Covering(CoveringImplSubstitution::new(impl_to_decl, decl_to_impl))
        } else {
            let domain = Arc::new(InherentImplDomain {
                impl_id: impl_id.clone(),
                target: InherentImplTarget::ExactEnumCase(variant_id.clone()),
                head_type: target_type,
                generic_signature: impl_generic_signature.clone(),
                constraints: constraints.into_boxed_slice(),
            });
            InherentImplApplicability::Conditional(domain)
        };

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

    let knowledge = resolve_type_annotation(ctx.store, ctx.declarations, &impl_resolver, &formation_site, &impl_def.target, &mut diagnostics);

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

    // 4. Validate nominal declaration target and compute covering or conditional applicability
    let (target_decl, applicability) = match ctx.store.get(target_type).clone() {
        TypeData::Nominal { declaration } => {
            // Check same module rule
            if declaration.module != impl_id.module {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplForeignTarget,
                    format!(
                        "cannot define inherent impl for foreign declaration `{}` in module `{}`",
                        declaration.name, declaration.module
                    ),
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

            if decl_param_count == 0 {
                if !impl_type_parameter_ids.is_empty() {
                    for &impl_param in &impl_type_parameter_ids {
                        let p_name = &ctx.store.type_parameter(impl_param).name;
                        diagnostics.push(SemanticDiagnostic::error_in(
                            ctx.current_module.clone(),
                            DiagnosticCode::ImplUnusedTypeParameter,
                            format!("type parameter `{}` is not used in inherent impl target", p_name),
                            impl_def.range,
                        ));
                    }
                    return Err(diagnostics);
                }
                if constraints.is_empty() {
                    (declaration, InherentImplApplicability::Unconditional)
                } else {
                    let domain = Arc::new(InherentImplDomain {
                        impl_id: impl_id.clone(),
                        target: InherentImplTarget::Declaration(declaration.clone()),
                        head_type: target_type,
                        generic_signature: impl_generic_signature.clone(),
                        constraints: constraints.into_boxed_slice(),
                    });
                    (declaration, InherentImplApplicability::Conditional(domain))
                }
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
                    format!(
                        "cannot define inherent impl for foreign declaration `{}` in module `{}`",
                        orig_decl.name, orig_decl.module
                    ),
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

            if arguments.len() != decl_params.len() {
                diagnostics.push(SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::ImplSpecializedTargetUnsupported,
                    format!(
                        "generic arity mismatch for inherent impl on `{}`: target has {} parameters, got {}",
                        orig_decl.name,
                        decl_params.len(),
                        arguments.len()
                    ),
                    impl_def.target.range,
                ));
                return Err(diagnostics);
            }

            // Collect used impl params across all arguments
            let mut used_impl_params = HashSet::new();
            for &arg_ty in arguments.iter() {
                collect_impl_params_in_type(ctx.store, arg_ty, impl_id, &mut used_impl_params);
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

            // Check if covering bijection
            let mut is_covering_bijection = constraints.is_empty() && arguments.len() == impl_type_parameter_ids.len();
            let mut impl_to_decl = HashMap::new();
            let mut decl_to_impl = HashMap::new();
            let mut seen_impl_params = HashSet::new();

            if is_covering_bijection {
                for (i, &arg_ty) in arguments.iter().enumerate() {
                    let decl_param = decl_params[i];
                    if let TypeData::Parameter(p_id) = ctx.store.get(arg_ty) {
                        let p_data = ctx.store.type_parameter(*p_id);
                        if let TypeParameterOwner::Impl(ref owner_impl) = p_data.owner {
                            if owner_impl == impl_id && seen_impl_params.insert(*p_id) {
                                impl_to_decl.insert(*p_id, decl_param);
                                decl_to_impl.insert(decl_param, *p_id);
                                continue;
                            }
                        }
                    }
                    is_covering_bijection = false;
                    break;
                }
            }

            if is_covering_bijection {
                let covering = CoveringImplSubstitution::new(impl_to_decl, decl_to_impl);
                (orig_decl, InherentImplApplicability::Covering(covering))
            } else {
                let domain = Arc::new(InherentImplDomain {
                    impl_id: impl_id.clone(),
                    target: InherentImplTarget::Declaration(orig_decl.clone()),
                    head_type: target_type,
                    generic_signature: impl_generic_signature.clone(),
                    constraints: constraints.into_boxed_slice(),
                });
                (orig_decl, InherentImplApplicability::Conditional(domain))
            }
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
pub fn apply_covering_to_signature(store: &mut TypeStore, subst: &TypeSubstitution, mut signature: CallableSemanticSignature) -> CallableSemanticSignature {
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
            type_contains_impl_param(store, *origin, impl_id) || arguments.iter().any(|&arg| type_contains_impl_param(store, arg, impl_id))
        }
        TypeData::ExactCase { enum_type, .. } => type_contains_impl_param(store, *enum_type, impl_id),
        TypeData::Union(members) => members.iter().any(|&m| type_contains_impl_param(store, m, impl_id)),
        TypeData::Tuple(elements) => elements.iter().any(|e| type_contains_impl_param(store, e.ty, impl_id)),
        TypeData::Record(row) => {
            let row_data = store.record_row(*row);
            row_data.fields.iter().any(|f| type_contains_impl_param(store, f.ty, impl_id))
        }
        TypeData::Callable(c) => {
            c.parameters.iter().any(|p| type_contains_impl_param(store, p.ty, impl_id)) || type_contains_impl_param(store, c.return_type, impl_id)
        }
        _ => false,
    }
}

/// Builds the canonical `InherentImplContribution` from an `ImplDef`.
pub fn build_inherent_impl_contribution(ctx: &mut CheckingContext<'_>, impl_id: &ImplId, impl_def: &ImplDef) -> InherentImplContribution {
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
                domain: None,
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
        InherentImplApplicability::Unconditional | InherentImplApplicability::Conditional(_) => None,
    };

    let domain = match &resolved_target.applicability {
        InherentImplApplicability::Conditional(d) => Some(d.clone()),
        _ => None,
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

        // Canonicalize signature into target declaration parameter space for covering impls
        let final_sig = if let Some(ref subst) = covering_subst {
            apply_covering_to_signature(ctx.store, subst, raw_sig)
        } else {
            raw_sig
        };

        // Invariant assertion: published unconditional surface signatures contain no free target-head TypeParameterOwner::Impl(current_impl)
        if domain.is_none() {
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
            _ => None,
        },
        domain,
        members: members.into_boxed_slice(),
        source,
        diagnostics: diagnostics.into_boxed_slice(),
    }
}
