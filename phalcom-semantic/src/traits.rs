//! Canonical semantic products owned by first-class trait declarations.
//!
//! Traits are behavioral declarations, not nominal runtime types. This module
//! therefore stores only the declaration identity, generic contract metadata,
//! and source provenance needed by later C3 trait products.

use crate::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact, DeclaredTypeState};
use crate::diagnostic::SemanticSourceSpan;
use crate::identity::{CallableId, CallableOwnerId, DeclarationId, DispatchSide, ModuleId};
use crate::signature::{CallableParameterSemantic, CallableSemanticSignature};
use crate::surface::MemberVisibility;
use crate::types::environment::{TypeEnvironment, TypeView};
use crate::types::id::{KindId, TypeId};
use crate::types::parameter::GenericSignature;
use crate::types::parameter::{GenericConstraint, TypeTerm};
use crate::types::relation::{TypeHierarchy, is_subtype};
use crate::types::store::TypeStore;
use phalcom_ast::ast::{BehaviorMember, TraitDef, TraitMember, TraitPropertyRequirement};
use phalcom_common::selector::Selector;
use std::collections::BTreeMap;
use std::sync::Arc;

/// A canonical reference to an instantiated trait contract.
///
/// A trait reference deliberately is not a [`TypeId`]. It carries contract
/// identity and canonical arguments without introducing a nominal value type,
/// class object, runtime descriptor, or conformance evidence product.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TraitRef {
    pub declaration: DeclarationId,
    pub arguments: Box<[TypeId]>,
}

/// Formation failures for a canonical [`TraitRef`].
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum TraitRefFormationError {
    #[error("declaration {0:?} is not a trait")]
    NotTrait(DeclarationId),
    #[error("trait {declaration:?} expects {expected} type arguments, got {actual}")]
    Arity {
        declaration: DeclarationId,
        expected: usize,
        actual: usize,
    },
    #[error("trait argument kind mismatch at index {index}: expected {expected:?}, got {actual:?}")]
    Kind { index: usize, expected: KindId, actual: KindId },
    #[error("trait argument at index {index} is not a valid canonical type")]
    InvalidArgument { index: usize },
    #[error("trait generic constraint {index} is not satisfied")]
    ConstraintUnsatisfied { index: usize },
    #[error("trait generic constraint {index} cannot be evaluated")]
    ConstraintNotCanonical { index: usize },
}

impl TraitRef {
    pub fn new(declaration: DeclarationId, arguments: impl Into<Box<[TypeId]>>) -> Self {
        Self {
            declaration,
            arguments: arguments.into(),
        }
    }

    /// Forms a trait reference from already-resolved canonical type arguments.
    ///
    /// Named declaration resolution remains the caller's responsibility: this
    /// API starts at the canonical `DeclarationId` boundary and verifies that
    /// the id belongs to a published trait header before accepting arguments.
    pub fn form(
        store: &mut TypeStore,
        headers: &TraitHeaderTable,
        hierarchy: &dyn TypeHierarchy,
        declaration: DeclarationId,
        arguments: &[TypeId],
    ) -> Result<Self, TraitRefFormationError> {
        let Some(header) = headers.get(&declaration) else {
            return Err(TraitRefFormationError::NotTrait(declaration));
        };

        for (index, &argument) in arguments.iter().enumerate() {
            if argument.index() >= store.type_count() {
                return Err(TraitRefFormationError::InvalidArgument { index });
            }
        }

        let expected = header.generic_signature.as_ref().map_or(0, GenericSignature::parameter_count);
        if arguments.len() != expected {
            return Err(TraitRefFormationError::Arity {
                declaration,
                expected,
                actual: arguments.len(),
            });
        }

        if let Some(signature) = &header.generic_signature {
            for (index, &argument) in arguments.iter().enumerate() {
                let expected_kind = signature
                    .parameter_kinds
                    .get(index)
                    .copied()
                    .unwrap_or_else(|| store.type_parameter(signature.parameters[index]).kind);
                let actual_kind = store.kind_of(argument);
                if actual_kind != expected_kind {
                    return Err(TraitRefFormationError::Kind {
                        index,
                        expected: expected_kind,
                        actual: actual_kind,
                    });
                }
            }

            validate_constraints(store, hierarchy, signature, arguments)?;
        }

        Ok(Self::new(declaration, arguments.to_vec().into_boxed_slice()))
    }
}

/// Stable identity for one behavioral requirement owned by a trait.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TraitRequirementId {
    pub owner: DeclarationId,
    pub selector: Selector,
    pub side: DispatchSide,
}

/// Stable identity for one associated type declaration owned by a trait.
///
/// The source-order index, rather than the spelling, is canonical so that
/// declarations with the same name in different traits (or future repeated
/// declarations diagnosed by the surface builder) cannot alias a behavioral
/// requirement or a type parameter.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AssociatedTypeRequirementId {
    pub owner: DeclarationId,
    pub index: u32,
}

impl AssociatedTypeRequirementId {
    pub fn new(owner: DeclarationId, index: u32) -> Self {
        Self { owner, index }
    }
}

/// One plain `type Name` requirement published by a trait.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraitAssociatedTypeRequirement {
    pub requirement: AssociatedTypeRequirementId,
    pub name: Box<str>,
    pub kind: KindId,
    pub source: SemanticSourceSpan,
}

/// One published abstract contract member and its optional default source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraitSurfaceMember {
    pub requirement: TraitRequirementId,
    pub callable: CallableId,
    pub signature: CallableSemanticSignature,
    pub visibility: MemberVisibility,
    pub source: SemanticSourceSpan,
    pub default_present: bool,
    pub default_source: Option<SemanticSourceSpan>,
}

/// Immutable trait requirement view after binding the trait parameters and
/// abstract `Self` for one source or exact conformance environment.
///
/// This is a projection of the canonical trait surface. It does not create a
/// new callable identity, alter the published trait surface, or re-run
/// requirement selection. Callable-local generic parameters remain owned by
/// their original callable because materialization only traverses canonical
/// type terms.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstantiatedTraitRequirement {
    pub requirement: TraitRequirementId,
    pub callable: CallableId,
    pub signature: CallableSemanticSignature,
    pub visibility: MemberVisibility,
    pub default_callable: Option<CallableId>,
}

fn instantiate_declared_type_fact(store: &mut TypeStore, environment: &TypeEnvironment, fact: &DeclaredTypeFact) -> DeclaredTypeFact {
    let Some(ty) = fact.canonical_type() else {
        return fact.clone();
    };
    let specialized = TypeView::new(ty, environment.clone()).materialize(store);
    match &fact.state {
        DeclaredTypeState::Known(TypeTerm::Canonical(_)) => DeclaredTypeFact::known(TypeTerm::Canonical(specialized), fact.basis),
        DeclaredTypeState::Known(TypeTerm::SelfType(_) | TypeTerm::Infer(_)) | DeclaredTypeState::Dynamic(_) | DeclaredTypeState::Unknown(_) => fact.clone(),
    }
}

impl TraitSurfaceMember {
    /// Materializes this requirement under a bounded canonical environment.
    pub fn instantiate(&self, store: &mut TypeStore, environment: &TypeEnvironment) -> InstantiatedTraitRequirement {
        let mut signature = self.signature.clone();
        let parameters = signature
            .parameters
            .into_vec()
            .into_iter()
            .map(|mut parameter| {
                parameter.declared_type = instantiate_declared_type_fact(store, environment, &parameter.declared_type);
                parameter
            })
            .collect::<Vec<_>>();
        signature.parameters = parameters.into_boxed_slice();
        signature.declared_return = instantiate_declared_type_fact(store, environment, &signature.declared_return);
        InstantiatedTraitRequirement {
            requirement: self.requirement.clone(),
            callable: self.callable.clone(),
            signature,
            visibility: self.visibility,
            default_callable: self.default_present.then(|| self.callable.clone()),
        }
    }
}

/// Complete, body-independent behavioral surface for one trait declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraitSurface {
    pub declaration: DeclarationId,
    pub generic_signature: Option<GenericSignature>,
    pub members: BTreeMap<TraitRequirementId, TraitSurfaceMember>,
    pub associated_types: BTreeMap<AssociatedTypeRequirementId, TraitAssociatedTypeRequirement>,
    pub diagnostics: Box<[crate::diagnostic::SemanticDiagnostic]>,
}

impl TraitSurface {
    pub fn new(declaration: DeclarationId, generic_signature: Option<GenericSignature>) -> Self {
        Self {
            declaration,
            generic_signature,
            members: BTreeMap::new(),
            associated_types: BTreeMap::new(),
            diagnostics: Box::new([]),
        }
    }

    pub fn get(&self, requirement: &TraitRequirementId) -> Option<&TraitSurfaceMember> {
        self.members.get(requirement)
    }

    pub fn get_by_selector(&self, selector: &Selector, side: DispatchSide) -> Option<&TraitSurfaceMember> {
        self.members
            .values()
            .find(|member| member.requirement.selector == *selector && member.requirement.side == side)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&TraitRequirementId, &TraitSurfaceMember)> {
        self.members.iter()
    }

    pub fn associated_type(&self, requirement: &AssociatedTypeRequirementId) -> Option<&TraitAssociatedTypeRequirement> {
        self.associated_types.get(requirement)
    }

    pub fn associated_type_by_name(&self, name: &str) -> Option<&TraitAssociatedTypeRequirement> {
        self.associated_types.values().find(|requirement| requirement.name.as_ref() == name)
    }

    /// Materializes every requirement in stable requirement-id order.
    pub fn instantiate(&self, store: &mut TypeStore, environment: &TypeEnvironment) -> BTreeMap<TraitRequirementId, InstantiatedTraitRequirement> {
        self.members
            .iter()
            .map(|(requirement, member)| (requirement.clone(), member.instantiate(store, environment)))
            .collect()
    }
}

/// Snapshot-owned trait surfaces keyed by canonical declaration identity.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TraitSurfaceTable {
    entries: BTreeMap<DeclarationId, Arc<TraitSurface>>,
}

impl TraitSurfaceTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, surface: Arc<TraitSurface>) {
        self.entries.insert(surface.declaration.clone(), surface);
    }

    pub fn get(&self, declaration: &DeclarationId) -> Option<&Arc<TraitSurface>> {
        self.entries.get(declaration)
    }

    pub fn contains(&self, declaration: &DeclarationId) -> bool {
        self.entries.contains_key(declaration)
    }

    pub fn remove_module(&mut self, module: &ModuleId) {
        self.entries.retain(|declaration, _| &declaration.module != module);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&DeclarationId, &Arc<TraitSurface>)> {
        self.entries.iter()
    }
}

/// Builds a complete trait contract surface without analyzing any member body.
pub(crate) fn build_trait_surface(ctx: &mut crate::checker::CheckingContext<'_>, header: &TraitHeader, trait_def: &TraitDef) -> TraitSurface {
    let mut surface = TraitSurface::new(header.declaration.clone(), header.generic_signature.clone());
    let owner = CallableOwnerId::Declaration(header.declaration.clone());
    let parent_resolver = ctx.resolver.clone();
    let declaration_resolver = crate::types::annotation::ScopedTypeResolver {
        parent: &parent_resolver,
        type_parameters: header
            .generic_signature
            .as_ref()
            .map(|signature| {
                signature
                    .parameters
                    .iter()
                    .map(|&parameter| {
                        let data = ctx.store.type_parameter(parameter);
                        (
                            data.name.to_string(),
                            crate::types::annotation::type_level_binding_for_parameter(ctx.store, parameter),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default(),
    };

    let mut diagnostics = Vec::new();
    let mut associated_type_index = 0u32;
    for trait_member in &trait_def.members {
        if let TraitMember::AssociatedType(declaration) = trait_member {
            let requirement = AssociatedTypeRequirementId::new(header.declaration.clone(), associated_type_index);
            associated_type_index += 1;
            if surface.associated_types.values().any(|existing| existing.name.as_ref() == declaration.name) {
                diagnostics.push(crate::diagnostic::SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    crate::diagnostic::DiagnosticCode::AssociatedTypeDuplicate,
                    format!("duplicate associated type declaration `{}`", declaration.name),
                    declaration.range,
                ));
                continue;
            }
            surface.associated_types.insert(
                requirement.clone(),
                TraitAssociatedTypeRequirement {
                    requirement,
                    name: declaration.name.clone().into_boxed_str(),
                    kind: KindId::TYPE,
                    source: SemanticSourceSpan::new(ctx.current_module.clone(), declaration.range),
                },
            );
            continue;
        }
        if let TraitMember::Property(property) = trait_member {
            let mut property_diagnostics = Vec::new();
            let formation_site = crate::types::annotation::TypeFormationSite::member(
                ctx.current_module.clone(),
                header.declaration.clone(),
                DispatchSide::Instance,
            );
            let declared_type = crate::types::annotation::resolve_type_annotation(
                ctx.store,
                ctx.declarations,
                &declaration_resolver,
                &formation_site,
                &property.annotation,
                &mut property_diagnostics,
            );
            diagnostics.extend(property_diagnostics);
            let declared_type = DeclaredTypeFact::from_knowledge_with_basis(&declared_type, DeclaredTypeBasis::SourceAnnotation);
            let source = SemanticSourceSpan::new(ctx.current_module.clone(), property.range);
            let visibility = trait_property_visibility(property);
            let mut generated = Vec::with_capacity(if property.mutable { 2 } else { 1 });
            generated.push(trait_property_signature(ctx, &header.declaration, property, &declared_type, false));
            if property.mutable {
                generated.push(trait_property_signature(ctx, &header.declaration, property, &declared_type, true));
            }
            for signature in generated {
                let requirement = TraitRequirementId::new(header.declaration.clone(), signature.selector.clone(), DispatchSide::Instance);
                if surface.members.contains_key(&requirement) {
                    diagnostics.push(crate::diagnostic::SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        crate::diagnostic::DiagnosticCode::TraitMemberConflict,
                        format!("duplicate trait member selector `{}`", requirement.selector.encode()),
                        property.range,
                    ));
                    continue;
                }
                let callable = signature.callable.clone();
                surface.members.insert(
                    requirement.clone(),
                    TraitSurfaceMember {
                        requirement,
                        callable,
                        signature,
                        visibility,
                        source: source.clone(),
                        default_present: false,
                        default_source: None,
                    },
                );
            }
            continue;
        }
        let Some(member) = trait_member.behavior() else {
            continue;
        };
        let syntax = crate::checker::declaration_signature::CallableSyntaxRef::from(member);
        let side = DispatchSide::Instance;
        let Some(callable) = crate::checker::declaration_signature::callable_id_for_syntax(&owner, syntax, side) else {
            diagnostics.push(crate::diagnostic::SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                crate::diagnostic::DiagnosticCode::AnnotationUnresolved,
                "trait member has no valid canonical callable selector",
                trait_member.range(),
            ));
            continue;
        };
        let requirement = TraitRequirementId::new(header.declaration.clone(), callable.selector.clone(), side);
        if surface.members.contains_key(&requirement) {
            diagnostics.push(crate::diagnostic::SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                crate::diagnostic::DiagnosticCode::TraitMemberConflict,
                format!("duplicate trait member selector `{}`", requirement.selector.encode()),
                trait_member.range(),
            ));
            continue;
        }
        let Some(signature) =
            crate::checker::declaration_signature::semantic_signature_for_syntax_with_resolver(ctx, &owner, &declaration_resolver, syntax, side)
        else {
            continue;
        };
        let source = signature
            .source
            .clone()
            .unwrap_or_else(|| SemanticSourceSpan::new(ctx.current_module.clone(), trait_member.range()));
        let default_present = syntax.has_body();
        let default_source = default_present.then(|| source.clone());
        surface.members.insert(
            requirement.clone(),
            TraitSurfaceMember {
                requirement,
                callable,
                signature,
                visibility: behavior_member_visibility(member),
                source,
                default_present,
                default_source,
            },
        );
    }
    diagnostics.extend(ctx.diagnostics.iter().cloned());
    surface.diagnostics = diagnostics.into_boxed_slice();
    surface
}

fn trait_property_visibility(property: &TraitPropertyRequirement) -> MemberVisibility {
    if property.name.starts_with("_$") {
        MemberVisibility::Internal
    } else if property.attributes.iter().any(|attribute| attribute.name == "private") {
        MemberVisibility::Private
    } else if property.attributes.iter().any(|attribute| attribute.name == "protected") {
        MemberVisibility::Protected
    } else {
        MemberVisibility::Public
    }
}

fn trait_property_signature(
    ctx: &mut crate::checker::CheckingContext<'_>,
    owner: &DeclarationId,
    property: &TraitPropertyRequirement,
    declared_type: &DeclaredTypeFact,
    setter: bool,
) -> CallableSemanticSignature {
    let selector = if setter {
        Selector::setter(&property.name).expect("parser guarantees a valid property setter selector")
    } else {
        Selector::getter(&property.name).expect("parser guarantees a valid property getter selector")
    };
    let callable = CallableId::new(CallableOwnerId::Declaration(owner.clone()), selector.clone(), DispatchSide::Instance);
    let source = SemanticSourceSpan::new(ctx.current_module.clone(), property.range);
    let parameters = if setter {
        vec![CallableParameterSemantic::new(
            crate::identity::CallableParameterId::new(callable.clone(), 0),
            "_",
            declared_type.clone(),
        )
        .with_source(source.clone())]
        .into_boxed_slice()
    } else {
        Box::new([])
    };
    let declared_return = if setter {
        DeclaredTypeFact::known(TypeTerm::Canonical(ctx.store.unit()), DeclaredTypeBasis::DeclarationSemantics)
    } else {
        declared_type.clone()
    };
    CallableSemanticSignature {
        callable,
        owner: owner.clone(),
        side: DispatchSide::Instance,
        selector,
        generics: None,
        parameters,
        declared_return,
        return_validation: crate::signature::ReturnContractValidation::NotApplicable,
        inferred_return: None,
        source: Some(source),
        implementation: phalcom_native_meta::ImplementationKind::Abstract,
        native_id: None,
        effects: phalcom_native_meta::EffectSpec::Unknown,
        raises: phalcom_native_meta::RaisesSpec::Unknown,
        flow: phalcom_native_meta::ReturnFlowSpec::Value,
        lifecycle: phalcom_native_meta::NativeLifecycleSpec::UNKNOWN,
    }
}

pub(crate) fn behavior_member_visibility(member: &BehaviorMember) -> MemberVisibility {
    let name = match member {
        BehaviorMember::Method(method) => Some(method.name.as_str()),
        BehaviorMember::Getter(getter) => Some(getter.name.as_str()),
        BehaviorMember::Setter(setter) => Some(setter.name.as_str()),
        BehaviorMember::Index(_) => None,
    };
    let attributes = member.attributes();
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

impl TraitRequirementId {
    pub fn new(owner: DeclarationId, selector: Selector, side: DispatchSide) -> Self {
        Self { owner, selector, side }
    }

    /// Returns the canonical source callable identity for this requirement.
    ///
    /// The returned `CallableId` is intentionally a separate semantic product
    /// from the requirement id. Future witness callables may use other owners.
    pub fn source_callable(&self) -> CallableId {
        CallableId::new(CallableOwnerId::Declaration(self.owner.clone()), self.selector.clone(), self.side)
    }
}

fn validate_constraints(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    signature: &GenericSignature,
    arguments: &[TypeId],
) -> Result<(), TraitRefFormationError> {
    let mut substitution = crate::types::substitution::TypeSubstitution::new();
    for (&parameter, &argument) in signature.parameters.iter().zip(arguments.iter()) {
        substitution.bind(parameter, argument);
    }

    for (index, constraint) in signature.constraints.iter().enumerate() {
        let (left, right, equivalent) = match constraint {
            GenericConstraint::Subtype { lower, upper } => (
                substitute_constraint_term(store, &substitution, lower, index)?,
                substitute_constraint_term(store, &substitution, upper, index)?,
                false,
            ),
            GenericConstraint::Equivalent { left, right } => (
                substitute_constraint_term(store, &substitution, left, index)?,
                substitute_constraint_term(store, &substitution, right, index)?,
                true,
            ),
        };

        let satisfied = if equivalent {
            is_subtype(store, hierarchy, left, right) && is_subtype(store, hierarchy, right, left)
        } else {
            is_subtype(store, hierarchy, left, right)
        };
        if !satisfied {
            return Err(TraitRefFormationError::ConstraintUnsatisfied { index });
        }
    }
    Ok(())
}

fn substitute_constraint_term(
    store: &mut TypeStore,
    substitution: &crate::types::substitution::TypeSubstitution,
    term: &TypeTerm,
    index: usize,
) -> Result<TypeId, TraitRefFormationError> {
    match term {
        TypeTerm::Canonical(ty) => Ok(substitution.apply(store, *ty)),
        TypeTerm::SelfType(_) | TypeTerm::Infer(_) => Err(TraitRefFormationError::ConstraintNotCanonical { index }),
    }
}

/// The published generic header for one trait declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraitHeader {
    pub declaration: DeclarationId,
    pub generic_signature: Option<GenericSignature>,
    pub source: SemanticSourceSpan,
}

/// Immutable-by-publication collection of trait headers for one snapshot.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TraitHeaderTable {
    entries: BTreeMap<DeclarationId, Arc<TraitHeader>>,
}

impl TraitHeaderTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, header: impl Into<Arc<TraitHeader>>) {
        let header = header.into();
        self.entries.insert(header.declaration.clone(), header);
    }

    pub fn get(&self, declaration: &DeclarationId) -> Option<&Arc<TraitHeader>> {
        self.entries.get(declaration)
    }

    pub fn contains(&self, declaration: &DeclarationId) -> bool {
        self.entries.contains_key(declaration)
    }

    pub fn remove_module(&mut self, module: &ModuleId) {
        self.entries.retain(|declaration, _| &declaration.module != module);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&DeclarationId, &Arc<TraitHeader>)> {
        self.entries.iter()
    }
}
