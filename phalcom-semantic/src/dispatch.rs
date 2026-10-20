//! Dispatch models, callable signatures, and selector resolution.

use crate::identity::{CallableId, DeclarationId, ImplId, ModuleId};

pub use crate::identity::DispatchSide;
use crate::surface::DeclarationSurface;
use crate::types::environment::TypeEnvironment;
use crate::types::evidence::TypeKnowledge;
use crate::types::id::TypeId;
use crate::types::relation::TypeHierarchy;
use crate::types::specialization::ReceiverSpecializationStep;
use phalcom_common::selector::Selector;
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use std::collections::{BTreeSet, HashMap, HashSet};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispatchTarget {
    pub selector: Selector,
    pub side: DispatchSide,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DispatchLookup {
    Normal,
    Super { defining_class: DeclarationId, side: DispatchSide },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispatchOwner {
    pub declaration: DeclarationId,
    pub side: DispatchSide,
}

use phalcom_ast::ast::RestMode;

/// Caller-facing parameter contract representation in dispatch signatures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CallableParameterContract {
    Fixed(TypeKnowledge),
    Rest { mode: RestMode, contract: RestValueContractKnowledge },
}

/// The value contract governing a rest parameter's lane or elements in dispatch signatures.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RestValueContractKnowledge {
    Unconstrained,
    Element(TypeKnowledge),
    Lane(TypeId),
    Invalid(crate::types::evidence::UnknownReason),
}

impl CallableParameterContract {
    pub fn rest_mode(&self) -> RestMode {
        match self {
            Self::Fixed(_) => RestMode::None,
            Self::Rest { mode, .. } => *mode,
        }
    }

    pub fn ty(&self) -> TypeKnowledge {
        match self {
            Self::Fixed(ty) => ty.clone(),
            Self::Rest {
                contract: RestValueContractKnowledge::Element(ty),
                ..
            } => ty.clone(),
            Self::Rest {
                contract: RestValueContractKnowledge::Invalid(reason),
                ..
            } => TypeKnowledge::Unknown(reason.clone()),
            Self::Rest { .. } => TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::NoTypeEvidence),
        }
    }
}

/// Parameter in a callable signature.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallableParameter {
    pub external_label: Option<String>,
    pub local_name: String,
    pub ty: TypeKnowledge,
    pub rest: RestMode,
    pub contract: CallableParameterContract,
}

impl CallableParameter {
    pub fn new(local_name: impl Into<String>, ty: TypeKnowledge) -> Self {
        Self {
            external_label: None,
            local_name: local_name.into(),
            contract: CallableParameterContract::Fixed(ty.clone()),
            ty,
            rest: RestMode::None,
        }
    }

    pub fn with_contract(mut self, contract: CallableParameterContract) -> Self {
        self.rest = contract.rest_mode();
        self.ty = contract.ty();
        self.contract = contract;
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.external_label = Some(label.into());
        self
    }

    pub fn with_rest(mut self, rest: RestMode) -> Self {
        self.rest = rest;
        if self.contract.rest_mode() != rest {
            self.contract = if rest == RestMode::None {
                CallableParameterContract::Fixed(self.ty.clone())
            } else {
                CallableParameterContract::Rest {
                    mode: rest,
                    contract: RestValueContractKnowledge::Element(self.ty.clone()),
                }
            };
        }
        self
    }

    pub fn is_rest(&self) -> bool {
        self.rest != RestMode::None
    }
}

/// Complete callable contract for a method, getter, setter, or indexer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallableSemanticKind {
    Ordinary,
    Constructor,
    Native,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallableSignature {
    pub selector: Selector,
    pub parameters: Vec<CallableParameter>,
    pub return_type: TypeKnowledge,
    pub generics: Option<crate::types::parameter::GenericSignature>,
    pub kind: CallableSemanticKind,
}

impl CallableSignature {
    pub(crate) fn clone_with_allocation_budget(&self, allocations: &mut AllocationBudget) -> Result<Self, AllocationError> {
        let mut parameters = allocations.vector(self.parameters.len())?;
        for parameter in &self.parameters {
            let contract = match &parameter.contract {
                CallableParameterContract::Fixed(ty) => CallableParameterContract::Fixed(ty.clone_with_allocation_budget(allocations)?),
                CallableParameterContract::Rest { mode, contract } => CallableParameterContract::Rest {
                    mode: *mode,
                    contract: match contract {
                        RestValueContractKnowledge::Unconstrained => RestValueContractKnowledge::Unconstrained,
                        RestValueContractKnowledge::Element(ty) => RestValueContractKnowledge::Element(ty.clone_with_allocation_budget(allocations)?),
                        RestValueContractKnowledge::Lane(ty) => RestValueContractKnowledge::Lane(*ty),
                        RestValueContractKnowledge::Invalid(reason) => RestValueContractKnowledge::Invalid(reason.clone_with_allocation_budget(allocations)?),
                    },
                },
            };
            parameters.push(CallableParameter {
                external_label: parameter.external_label.as_ref().map(|label| allocations.string(label)).transpose()?,
                local_name: allocations.string(&parameter.local_name)?,
                ty: parameter.ty.clone_with_allocation_budget(allocations)?,
                rest: parameter.rest,
                contract,
            });
        }
        Ok(Self {
            selector: crate::traits::clone_selector_with_allocation_budget(&self.selector, allocations)?,
            parameters,
            return_type: self.return_type.clone_with_allocation_budget(allocations)?,
            generics: self
                .generics
                .as_ref()
                .map(|generics| generics.clone_with_allocation_budget(allocations))
                .transpose()?,
            kind: self.kind,
        })
    }

    pub fn new(selector: Selector, parameters: Vec<CallableParameter>, return_type: TypeKnowledge) -> Self {
        Self {
            selector,
            parameters,
            return_type,
            generics: None,
            kind: CallableSemanticKind::Ordinary,
        }
    }

    pub fn with_generics(mut self, generics: crate::types::parameter::GenericSignature) -> Self {
        self.generics = Some(generics);
        self
    }

    pub fn with_kind(mut self, kind: CallableSemanticKind) -> Self {
        self.kind = kind;
        self
    }

    /// Returns whether every slot in this compatibility dispatch projection
    /// currently has a known type. Canonical signature publication does not
    /// depend on this predicate; partial declarations are first-class products.
    pub fn has_complete_types(&self) -> bool {
        self.return_type.ty().is_some() && self.parameters.iter().all(|parameter| parameter.ty.ty().is_some())
    }

    /// Returns whether this callable signature accepts the incoming selector,
    /// either via exact match or by accepting remaining slots through rest parameters.
    pub fn accepts_selector(&self, selector: &Selector) -> bool {
        if self.selector == *selector {
            return true;
        }
        if !self.parameters.iter().any(|p| p.is_rest()) {
            return false;
        }
        if self.selector.base != selector.base {
            return false;
        }
        use phalcom_common::selector::{SelectorKind, SelectorSlot};
        if !matches!((&self.selector.kind, &selector.kind), (SelectorKind::Method, SelectorKind::Method)) {
            return false;
        }
        let positional_rest = self.parameters.iter().any(|p| matches!(p.rest, RestMode::Positional | RestMode::Complete));
        let labeled_rest = self.parameters.iter().any(|p| matches!(p.rest, RestMode::Labeled | RestMode::Complete));

        let fixed_positional_count = self
            .parameters
            .iter()
            .filter(|p| p.rest == RestMode::None && p.external_label.is_none())
            .count();
        let fixed_labels = self
            .parameters
            .iter()
            .filter_map(|p| if p.rest == RestMode::None { p.external_label.as_deref() } else { None })
            .collect::<Vec<_>>();

        let positional_count = selector.slots.iter().filter(|s| matches!(s, SelectorSlot::Positional)).count();
        let labels: Vec<&str> = selector
            .slots
            .iter()
            .filter_map(|s| match s {
                SelectorSlot::Label(l) => Some(l.as_str()),
                _ => None,
            })
            .collect();

        if positional_rest {
            if positional_count < fixed_positional_count {
                return false;
            }
        } else if positional_count != fixed_positional_count {
            return false;
        }

        for fixed_label in &fixed_labels {
            if !labels.contains(fixed_label) {
                return false;
            }
        }

        if labeled_rest {
            if labels.len() < fixed_labels.len() {
                return false;
            }
        } else if labels.len() != fixed_labels.len() {
            return false;
        }

        true
    }
}

/// Complete result of resolving dispatch, capturing visited hierarchy trace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispatchSignatureSpecialization {
    pub receiver: TypeId,
    pub declaring_owner: DeclarationId,
    pub environment: TypeEnvironment,
    pub path: Box<[ReceiverSpecializationStep]>,
    pub unspecialized_return: TypeKnowledge,
}

/// Canonical evidence that a dispatch result came from a conditional
/// inherent member.  Consumers may carry this selection through lowering,
/// but must not re-run the domain matcher.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ConditionalDispatchSelection {
    pub impl_id: ImplId,
    pub callable: CallableId,
    pub declaring_owner: DeclarationId,
    pub side: DispatchSide,
}

/// Proof-bearing authority for executing one selected conditional inherent
/// callable. The applicability product owns exact impl substitutions and the
/// evidence provenance for its conformance requirements.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConditionalSelectedCallCapability {
    pub selection: ConditionalDispatchSelection,
    pub applicability: crate::impls::InherentImplSpecialization,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedDispatch {
    pub callable: CallableId,
    pub signature: CallableSignature,
    pub visibility: phalcom_common::modifier::Visibility,
    pub access_denied: bool,
    pub specialization: Option<DispatchSignatureSpecialization>,
    pub conditional: Option<ConditionalDispatchSelection>,
    /// Published capability for a selected conditional implementation.
    pub conditional_capability: Option<ConditionalSelectedCallCapability>,
    pub trait_dispatch: Option<crate::trait_dispatch::TraitDispatchSelection>,
    /// True when the result is a semantic trait-contract lookup rather than
    /// executable nominal/conditional dispatch.
    pub abstract_contract: bool,
    pub visited_owners: Box<[DeclarationId]>,
}

/// Trace-aware result of resolving a message send selector against a receiver.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolvedDispatchResult {
    Found(Box<ResolvedDispatch>),
    Ambiguous(Vec<ResolvedDispatch>),
    Missing {
        visited_owners: Box<[DeclarationId]>,
    },
    /// A trait candidate existed, but exact conformance proof terminated
    /// without an executable selection. This is not runtime-dynamic dispatch.
    TraitTerminal(crate::trait_dispatch::TraitDispatchTerminal),
    Dynamic,
}

/// The result of resolving a message send selector against a receiver.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DispatchResult {
    Found(Box<CallableSignature>),
    Ambiguous(Vec<CallableSignature>),
    Missing,
    Dynamic,
}

impl DispatchResult {
    pub fn is_found(&self) -> bool {
        matches!(self, Self::Found(_))
    }

    pub fn signature(&self) -> Option<&CallableSignature> {
        match self {
            Self::Found(sig) => Some(sig),
            _ => None,
        }
    }
}

use crate::impls::{ConditionalInherentMemberSet, InherentImplTarget};
use std::sync::Arc;

/// Trait for querying semantic dispatch targets.
pub trait DispatchResolver {
    fn resolve_dispatch(&self, receiver: TypeId, selector: &Selector, lookup: DispatchLookup) -> DispatchResult;
}

/// A concrete dispatch resolver backed by nominal class surfaces and type interning.
#[derive(Clone, Debug, Default)]
pub struct SurfaceDispatchResolver {
    surfaces: Arc<crate::parent_map::ParentMap<DeclarationId, DeclarationSurface>>,
    conditional_members: Arc<crate::parent_map::ParentMap<InherentImplTarget, Arc<ConditionalInherentMemberSet>>>,
    conditional_targets_by_owner: HashMap<DeclarationId, BTreeSet<InherentImplTarget>>,
    type_declarations: Arc<crate::parent_map::ParentMap<TypeId, DeclarationId>>,
    inherent_impl_domains: Arc<crate::parent_map::ParentMap<crate::identity::ImplId, Arc<crate::impls::InherentImplDomain>>>,
    module_surfaces: HashMap<ModuleId, BTreeSet<DeclarationId>>,
    declaration_types: HashMap<DeclarationId, BTreeSet<TypeId>>,
}

impl SurfaceDispatchResolver {
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds the immutable dispatch authority retained by an imported
    /// semantic snapshot without growing source-oriented native maps.
    pub(crate) fn from_frozen_import_entries_with_allocation_budget(
        surfaces: Vec<(DeclarationId, DeclarationSurface)>,
        type_declarations: Vec<(TypeId, DeclarationId)>,
        allocations: &mut phalcom_type_meta::allocation::AllocationBudget,
    ) -> Result<Self, FrozenDispatchImportError> {
        let surfaces = crate::parent_map::ParentMap::from_frozen_entries_with_allocation_budget(surfaces, allocations).map_err(|error| match error {
            crate::parent_map::FrozenParentMapError::Allocation(error) => FrozenDispatchImportError::Allocation(error),
            crate::parent_map::FrozenParentMapError::DuplicateKey => FrozenDispatchImportError::DuplicateOwner,
        })?;
        let type_declarations =
            crate::parent_map::ParentMap::from_frozen_entries_with_allocation_budget(type_declarations, allocations).map_err(|error| match error {
                crate::parent_map::FrozenParentMapError::Allocation(error) => FrozenDispatchImportError::Allocation(error),
                crate::parent_map::FrozenParentMapError::DuplicateKey => FrozenDispatchImportError::DuplicateType,
            })?;
        Ok(Self {
            surfaces: allocations.arc(surfaces).map_err(FrozenDispatchImportError::Allocation)?,
            conditional_members: allocations
                .arc(crate::parent_map::ParentMap::default())
                .map_err(FrozenDispatchImportError::Allocation)?,
            type_declarations: allocations.arc(type_declarations).map_err(FrozenDispatchImportError::Allocation)?,
            inherent_impl_domains: allocations
                .arc(crate::parent_map::ParentMap::default())
                .map_err(FrozenDispatchImportError::Allocation)?,
            conditional_targets_by_owner: HashMap::new(),
            module_surfaces: HashMap::new(),
            declaration_types: HashMap::new(),
        })
    }

    pub(crate) fn with_frozen_conditional_import_entries(
        mut self,
        conditional_members: Vec<(InherentImplTarget, Arc<ConditionalInherentMemberSet>)>,
        inherent_impl_domains: Vec<(crate::identity::ImplId, Arc<crate::impls::InherentImplDomain>)>,
        allocations: &mut phalcom_type_meta::allocation::AllocationBudget,
    ) -> Result<Self, FrozenDispatchImportError> {
        let conditional_members =
            crate::parent_map::ParentMap::from_frozen_entries_with_allocation_budget(conditional_members, allocations).map_err(|error| match error {
                crate::parent_map::FrozenParentMapError::Allocation(error) => FrozenDispatchImportError::Allocation(error),
                crate::parent_map::FrozenParentMapError::DuplicateKey => FrozenDispatchImportError::DuplicateConditionalTarget,
            })?;
        let inherent_impl_domains =
            crate::parent_map::ParentMap::from_frozen_entries_with_allocation_budget(inherent_impl_domains, allocations).map_err(|error| match error {
                crate::parent_map::FrozenParentMapError::Allocation(error) => FrozenDispatchImportError::Allocation(error),
                crate::parent_map::FrozenParentMapError::DuplicateKey => FrozenDispatchImportError::DuplicateDomain,
            })?;
        self.conditional_members = allocations.arc(conditional_members).map_err(FrozenDispatchImportError::Allocation)?;
        self.inherent_impl_domains = allocations.arc(inherent_impl_domains).map_err(FrozenDispatchImportError::Allocation)?;
        Ok(self)
    }

    /// Attaches canonical dispatch products without copying surfaces or registrations.
    pub fn with_parent(parent: Arc<Self>) -> Self {
        Self {
            surfaces: Arc::new(crate::parent_map::ParentMap::with_parent(parent.surfaces.clone())),
            conditional_members: Arc::new(crate::parent_map::ParentMap::with_parent(parent.conditional_members.clone())),
            type_declarations: Arc::new(crate::parent_map::ParentMap::with_parent(parent.type_declarations.clone())),
            inherent_impl_domains: Arc::new(crate::parent_map::ParentMap::with_parent(parent.inherent_impl_domains.clone())),
            ..Self::default()
        }
    }

    pub fn register_conditional_members(&mut self, target: InherentImplTarget, members: Arc<ConditionalInherentMemberSet>) {
        self.conditional_targets_by_owner
            .entry(target.declaration().clone())
            .or_default()
            .insert(target.clone());
        let domain_map = Arc::make_mut(&mut self.inherent_impl_domains);
        for member in &members.members {
            if !domain_map.contains_key(&member.impl_id) {
                domain_map.insert(member.impl_id.clone(), member.domain.clone());
            }
        }
        Arc::make_mut(&mut self.conditional_members).insert(target, members);
    }

    pub fn inherent_impl_domain(&self, impl_id: &crate::identity::ImplId) -> Option<&Arc<crate::impls::InherentImplDomain>> {
        self.inherent_impl_domains.get(impl_id)
    }

    pub fn get_conditional_members(&self, target: &InherentImplTarget) -> Option<&Arc<ConditionalInherentMemberSet>> {
        self.conditional_members.get(target)
    }

    pub fn conditional_members(&self) -> &crate::parent_map::ParentMap<InherentImplTarget, Arc<ConditionalInherentMemberSet>> {
        &self.conditional_members
    }

    pub fn register_surface(&mut self, decl: DeclarationId, surface: DeclarationSurface) {
        self.module_surfaces.entry(decl.module.clone()).or_default().insert(decl.clone());
        Arc::make_mut(&mut self.surfaces).insert(decl, surface);
    }

    pub fn register_type(&mut self, ty: TypeId, decl: DeclarationId) {
        self.declaration_types.entry(decl.clone()).or_default().insert(ty);
        Arc::make_mut(&mut self.type_declarations).insert(ty, decl);
    }

    /// Removes declaration-owned dispatch surfaces and type registrations.
    pub fn remove_module(&mut self, module: &phalcom_modules::identity::ModuleId) {
        Arc::make_mut(&mut self.inherent_impl_domains).retain(|impl_id, _| &impl_id.module != module);
        let removed = self.module_surfaces.remove(module).unwrap_or_default();
        for declaration in removed {
            self.remove_surface(&declaration);
        }
    }

    /// Removes one declaration surface and only the type registrations owned
    /// by that declaration.
    pub fn remove_surface(&mut self, declaration: &DeclarationId) {
        if let Some(targets) = self.conditional_targets_by_owner.remove(declaration) {
            for target in targets {
                Arc::make_mut(&mut self.conditional_members).remove(&target);
            }
        } else {
            Arc::make_mut(&mut self.conditional_members).remove(&InherentImplTarget::Declaration(declaration.clone()));
        }
        let Some(_) = Arc::make_mut(&mut self.surfaces).remove(declaration) else {
            return;
        };
        if let Some(declarations) = self.module_surfaces.get_mut(&declaration.module) {
            declarations.remove(declaration);
            if declarations.is_empty() {
                self.module_surfaces.remove(&declaration.module);
            }
        }
        if let Some(types) = self.declaration_types.remove(declaration) {
            for ty in types {
                Arc::make_mut(&mut self.type_declarations).remove(&ty);
            }
        }
    }

    pub fn get_surface(&self, decl: &DeclarationId) -> Option<&DeclarationSurface> {
        self.surfaces.get(decl)
    }

    pub fn get_surface_mut(&mut self, decl: &DeclarationId) -> Option<&mut DeclarationSurface> {
        Arc::make_mut(&mut self.surfaces).get_mut(decl)
    }

    /// Replaces an inferred return type on an existing source callable while
    /// retaining its parameters, selector, and declaration identity.
    pub fn update_callable_return_type(&mut self, callable: &CallableId, return_type: TypeKnowledge) -> bool {
        if callable.try_declaration_owner().is_none() {
            // Conformance-local witnesses are semantic proof products, not
            // members of a target-owned inherent dispatch surface.
            return false;
        }
        let Some(surface) = Arc::make_mut(&mut self.surfaces).get_mut(callable.declaration_owner()) else {
            return false;
        };
        let member_surface = surface.surface_mut(callable.side);
        let Some(signature) = member_surface.callable_signatures.get_mut(&callable.selector) else {
            return false;
        };
        if signature.return_type == return_type {
            return false;
        }
        signature.return_type = return_type.clone();
        if let Some(id) = member_surface.callables_by_selector.get(&callable.selector).cloned() {
            member_surface.callables.insert(id, return_type);
        }
        true
    }

    pub fn surfaces(&self) -> &Arc<crate::parent_map::ParentMap<DeclarationId, DeclarationSurface>> {
        &self.surfaces
    }

    pub fn surface(&self, decl_id: &DeclarationId) -> Option<&DeclarationSurface> {
        self.surfaces.get(decl_id)
    }

    /// Returns the declaration selected as the runtime/native owner for a registered type form.
    pub fn declaration_for_type(&self, ty: TypeId) -> Option<&DeclarationId> {
        self.type_declarations.get(&ty)
    }

    /// Returns the canonical owner/side traversal for one dispatch receiver.
    ///
    /// Class-object dispatch first walks the parallel class-side hierarchy.
    /// When that hierarchy is exhausted it enters the canonical `Class`
    /// instance behavior root, mirroring the runtime metaclass tower without
    /// materializing semantic metaclass objects.
    pub fn dispatch_owners(&self, hierarchy: &dyn TypeHierarchy, start_decl: &DeclarationId, side: DispatchSide) -> Vec<DispatchOwner> {
        let mut owners = Vec::new();
        let mut visited = HashSet::new();
        let mut current = Some(DispatchOwner {
            declaration: start_decl.clone(),
            side,
        });
        let mut entered_class_object_root = false;

        while let Some(owner) = current {
            if !visited.insert((owner.declaration.clone(), owner.side)) {
                break;
            }
            owners.push(owner.clone());
            current = if let Some(superclass) = hierarchy.superclass(&owner.declaration) {
                Some(DispatchOwner {
                    declaration: superclass.clone(),
                    side: owner.side,
                })
            } else if owner.side == DispatchSide::Class && !entered_class_object_root {
                entered_class_object_root = true;
                Some(DispatchOwner {
                    declaration: crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Class),
                    side: DispatchSide::Instance,
                })
            } else {
                None
            };
        }

        owners
    }

    pub fn resolve_dispatch_with_trace(
        &self,
        hierarchy: &dyn TypeHierarchy,
        start_decl: &DeclarationId,
        side: DispatchSide,
        selector: &Selector,
    ) -> ResolvedDispatchResult {
        let mut visited = Vec::new();
        for owner in self.dispatch_owners(hierarchy, start_decl, side) {
            visited.push(owner.declaration.clone());
            if let Some(surface) = self.surfaces.get(&owner.declaration) {
                if let Some(sig) = surface.get_callable(owner.side, selector) {
                    let callable_id = surface
                        .get_callable_id(owner.side, selector)
                        .cloned()
                        .unwrap_or_else(|| CallableId::new(owner.declaration.clone(), sig.selector.clone(), owner.side));
                    return ResolvedDispatchResult::Found(Box::new(ResolvedDispatch {
                        callable: callable_id,
                        signature: sig.clone(),
                        visibility: surface.surface(owner.side).callable_visibility.get(&sig.selector).copied().unwrap_or_default(),
                        access_denied: false,
                        specialization: None,
                        conditional: None,
                        conditional_capability: None,
                        trait_dispatch: None,
                        abstract_contract: false,
                        visited_owners: visited.into_boxed_slice(),
                    }));
                }
            }
        }
        ResolvedDispatchResult::Missing {
            visited_owners: visited.into_boxed_slice(),
        }
    }

    pub fn resolve_dispatch_on_owner(
        &self,
        hierarchy: &dyn TypeHierarchy,
        start_decl: &DeclarationId,
        side: DispatchSide,
        selector: &Selector,
    ) -> DispatchResult {
        match self.resolve_dispatch_with_trace(hierarchy, start_decl, side, selector) {
            ResolvedDispatchResult::Found(rd) => DispatchResult::Found(Box::new(rd.signature)),
            ResolvedDispatchResult::Ambiguous(amb) => DispatchResult::Ambiguous(amb.into_iter().map(|rd| rd.signature).collect()),
            ResolvedDispatchResult::Missing { .. } => DispatchResult::Missing,
            ResolvedDispatchResult::TraitTerminal(_) | ResolvedDispatchResult::Dynamic => DispatchResult::Dynamic,
        }
    }

    pub fn resolve_callable_id(
        &self,
        hierarchy: &dyn TypeHierarchy,
        start_decl: &DeclarationId,
        side: DispatchSide,
        selector: &Selector,
    ) -> Option<CallableId> {
        for owner in self.dispatch_owners(hierarchy, start_decl, side) {
            if let Some(surface) = self.surfaces.get(&owner.declaration) {
                if let Some(sig) = surface.get_callable(owner.side, selector) {
                    return Some(
                        surface
                            .get_callable_id(owner.side, selector)
                            .cloned()
                            .unwrap_or_else(|| CallableId::new(owner.declaration.clone(), sig.selector.clone(), owner.side)),
                    );
                }
            }
        }
        None
    }
}

#[derive(Debug)]
pub(crate) enum FrozenDispatchImportError {
    Allocation(phalcom_type_meta::allocation::AllocationError),
    DuplicateOwner,
    DuplicateType,
    DuplicateConditionalTarget,
    DuplicateDomain,
}

#[cfg(test)]
mod parent_tests {
    use super::*;

    #[test]
    fn dispatch_overlay_resolves_parent_members_and_removes_only_local_registrations() {
        let canonical = DeclarationId::new(ModuleId::universe_root(), "Canonical".into());
        let local = DeclarationId::new(ModuleId::universe_root(), "Local".into());
        let selector = Selector::method("value", []).unwrap();
        let mut surface = DeclarationSurface::new(Some(canonical.clone()));
        surface.add_callable(
            DispatchSide::Instance,
            CallableSignature::new(
                selector.clone(),
                vec![],
                TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::NoTypeEvidence),
            ),
        );
        let mut parent = SurfaceDispatchResolver::new();
        parent.register_surface(canonical.clone(), surface);
        parent.register_type(TypeId(1), canonical.clone());
        let parent = Arc::new(parent);
        let mut overlay = SurfaceDispatchResolver::with_parent(parent.clone());
        assert_eq!(overlay.surfaces().local_len(), 0);
        assert_eq!(overlay.declaration_for_type(TypeId(1)), Some(&canonical));
        assert!(overlay.get_surface_mut(&canonical).is_none());
        let hierarchy = crate::types::relation::MapTypeHierarchy::new();
        assert!(
            overlay
                .resolve_dispatch_on_owner(&hierarchy, &canonical, DispatchSide::Instance, &selector)
                .is_found()
        );
        overlay.register_surface(local.clone(), DeclarationSurface::new(Some(local.clone())));
        overlay.register_type(TypeId(2), local.clone());
        let published = overlay.clone();
        overlay.remove_module(&canonical.module);
        assert!(overlay.get_surface(&local).is_none());
        assert!(published.get_surface(&local).is_some());
        assert!(std::ptr::eq(overlay.get_surface(&canonical).unwrap(), parent.get_surface(&canonical).unwrap()));
        assert_eq!(overlay.declaration_for_type(TypeId(2)), None);
        assert_eq!(overlay.declaration_for_type(TypeId(1)), Some(&canonical));
        assert_eq!(parent.surfaces().len(), 1);
    }
}
