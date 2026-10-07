//! Canonical callable and field semantic signatures and identity-indexed tables.

use super::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact};
use super::diagnostic::SemanticSourceSpan;
use super::identity::{CallableId, CallableParameterId, DeclarationId, DispatchSide, FieldId, ModuleId};
use super::types::evidence::{EvidenceOrigin, EvidenceStatus, TypeKnowledge, UnknownReason};
use super::types::parameter::{GenericSignature, TypeTerm};
use crate::types::id::TypeId;
use crate::types::store::{AnonymousTupleRemainder, TypeStore};
use phalcom_ast::ast::RestMode;
use phalcom_common::modifier::ImplementationKind;
use phalcom_common::selector::Selector;
use phalcom_native_meta::{EffectSpec, NativeLifecycleSpec, RaisesSpec, ReturnFlowSpec};
use phalcom_native_surface::NativeSurfaceId;
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// Epistemic certification state for a source return contract.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ReturnContractValidation {
    /// No source-authored return contract requires body certification.
    NotApplicable,
    /// Source contract exists but no complete body proof has been published yet.
    Unchecked,
    /// Every normal return satisfies the contract. The payload is the weakest
    /// implementation evidence supporting the contract.
    Satisfied(EvidenceStatus),
    /// At least one normal return is proven incompatible with the contract.
    Refuted,
    /// Analysis could not decide the contract soundly.
    Blocked,
    /// Validation crosses an explicit Dynamic boundary.
    DynamicBoundary,
}

/// Single authoritative caller-facing contract for a callable parameter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParameterContract {
    Fixed(DeclaredTypeFact),
    Rest { mode: RestMode, contract: RestValueContract },
}

/// The value contract governing a rest parameter's lane or elements.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RestValueContract {
    Unconstrained,
    Element(DeclaredTypeFact),
    Lane(TypeId),
    /// A rest annotation was present but its semantic operand could not be
    /// formed.  Recovery must retain that failure instead of widening the
    /// callable to an unconstrained rest parameter.
    Invalid(UnknownReason),
}

impl ParameterContract {
    pub fn rest_mode(&self) -> RestMode {
        match self {
            Self::Fixed(_) => RestMode::None,
            Self::Rest { mode, .. } => *mode,
        }
    }

    pub fn declared_type(&self) -> DeclaredTypeFact {
        match self {
            Self::Fixed(fact) => fact.clone(),
            Self::Rest {
                contract: RestValueContract::Element(fact),
                ..
            } => fact.clone(),
            Self::Rest {
                contract: RestValueContract::Unconstrained,
                ..
            } => DeclaredTypeFact::unknown(UnknownReason::UnannotatedDeclaration),
            Self::Rest {
                contract: RestValueContract::Lane(_),
                ..
            } => DeclaredTypeFact::unknown(UnknownReason::NoTypeEvidence),
            Self::Rest {
                contract: RestValueContract::Invalid(reason),
                ..
            } => DeclaredTypeFact::unknown(reason.clone()),
        }
    }

    pub fn derive_body_type(&self, store: &mut TypeStore) -> DeclaredTypeFact {
        match self {
            Self::Fixed(fact) => fact.clone(),
            Self::Rest { mode, contract } => match contract {
                RestValueContract::Unconstrained => {
                    let (pos_rem, lab_rem) = match mode {
                        RestMode::Positional => (AnonymousTupleRemainder::Unconstrained, AnonymousTupleRemainder::None),
                        RestMode::Labeled => (AnonymousTupleRemainder::None, AnonymousTupleRemainder::Unconstrained),
                        RestMode::Complete => (AnonymousTupleRemainder::Unconstrained, AnonymousTupleRemainder::Unconstrained),
                        RestMode::None => unreachable!(),
                    };
                    match store.open_pack(Box::default(), pos_rem, lab_rem) {
                        Ok(ty) => DeclaredTypeFact::known(TypeTerm::Canonical(ty), DeclaredTypeBasis::DeclarationSemantics),
                        Err(_) => DeclaredTypeFact::unknown(UnknownReason::SuppressedByInvalidCause),
                    }
                }
                RestValueContract::Element(fact) => {
                    let Some(elem_ty) = fact.canonical_type() else {
                        let reason = match &fact.state {
                            super::declaration_type::DeclaredTypeState::Unknown(reason) => reason.clone(),
                            _ => UnknownReason::SuppressedByInvalidCause,
                        };
                        return DeclaredTypeFact::unknown(reason);
                    };
                    let (pos_rem, lab_rem) = match mode {
                        RestMode::Positional => (AnonymousTupleRemainder::Typed(elem_ty), AnonymousTupleRemainder::None),
                        RestMode::Labeled => (AnonymousTupleRemainder::None, AnonymousTupleRemainder::Typed(elem_ty)),
                        RestMode::Complete => (AnonymousTupleRemainder::Typed(elem_ty), AnonymousTupleRemainder::Typed(elem_ty)),
                        RestMode::None => unreachable!(),
                    };
                    match store.open_pack(Box::default(), pos_rem, lab_rem) {
                        Ok(ty) => DeclaredTypeFact::known(TypeTerm::Canonical(ty), DeclaredTypeBasis::DeclarationSemantics),
                        Err(_) => DeclaredTypeFact::unknown(UnknownReason::SuppressedByInvalidCause),
                    }
                }
                RestValueContract::Lane(lane_id) => match mode {
                    RestMode::Positional => {
                        let Ok(empty_labeled) = store.labeled_type_pack(Box::default()) else {
                            return DeclaredTypeFact::unknown(UnknownReason::SuppressedByInvalidCause);
                        };
                        match store.pack_from_packs(*lane_id, empty_labeled) {
                            Ok(ty) => DeclaredTypeFact::known(TypeTerm::Canonical(ty), DeclaredTypeBasis::DeclarationSemantics),
                            Err(_) => DeclaredTypeFact::unknown(UnknownReason::SuppressedByInvalidCause),
                        }
                    }
                    RestMode::Labeled => {
                        let Ok(empty_pos) = store.type_pack(Box::default()) else {
                            return DeclaredTypeFact::unknown(UnknownReason::SuppressedByInvalidCause);
                        };
                        match store.pack_from_packs(empty_pos, *lane_id) {
                            Ok(ty) => DeclaredTypeFact::known(TypeTerm::Canonical(ty), DeclaredTypeBasis::DeclarationSemantics),
                            Err(_) => DeclaredTypeFact::unknown(UnknownReason::SuppressedByInvalidCause),
                        }
                    }
                    RestMode::Complete => {
                        let Ok(pos) = store.project_positionals(*lane_id) else {
                            return DeclaredTypeFact::unknown(UnknownReason::SuppressedByInvalidCause);
                        };
                        let Ok(lab) = store.project_labeled(*lane_id) else {
                            return DeclaredTypeFact::unknown(UnknownReason::SuppressedByInvalidCause);
                        };
                        match store.pack_from_packs(pos, lab) {
                            Ok(ty) => DeclaredTypeFact::known(TypeTerm::Canonical(ty), DeclaredTypeBasis::DeclarationSemantics),
                            Err(_) => DeclaredTypeFact::unknown(UnknownReason::SuppressedByInvalidCause),
                        }
                    }
                    RestMode::None => unreachable!(),
                },
                RestValueContract::Invalid(reason) => DeclaredTypeFact::unknown(reason.clone()),
            },
        }
    }
}

/// Canonical parameter specification for a callable signature.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallableParameterSemantic {
    pub id: CallableParameterId,
    pub local_name: Box<str>,
    pub external_label: Option<Box<str>>,
    pub rest: RestMode,
    pub declared_type: DeclaredTypeFact,
    pub contract: ParameterContract,
    pub source: Option<SemanticSourceSpan>,
}

impl CallableParameterSemantic {
    pub fn new(id: CallableParameterId, local_name: impl Into<Box<str>>, declared_type: DeclaredTypeFact) -> Self {
        Self {
            id,
            local_name: local_name.into(),
            external_label: None,
            rest: RestMode::None,
            contract: ParameterContract::Fixed(declared_type.clone()),
            declared_type,
            source: None,
        }
    }

    pub fn index(&self) -> u32 {
        self.id.index
    }

    pub fn with_contract(mut self, contract: ParameterContract) -> Self {
        self.rest = contract.rest_mode();
        self.declared_type = contract.declared_type();
        self.contract = contract;
        self
    }

    pub fn with_label(mut self, label: impl Into<Box<str>>) -> Self {
        self.external_label = Some(label.into());
        self
    }

    pub fn with_rest(mut self, rest: RestMode) -> Self {
        self.rest = rest;
        if self.contract.rest_mode() != rest {
            self.contract = if rest == RestMode::None {
                ParameterContract::Fixed(self.declared_type.clone())
            } else {
                ParameterContract::Rest {
                    mode: rest,
                    contract: RestValueContract::Element(self.declared_type.clone()),
                }
            };
        }
        self
    }

    pub fn with_source(mut self, source: SemanticSourceSpan) -> Self {
        self.source = Some(source);
        self
    }
}

/// Canonical semantic type publication for a callable.
///
/// `declared_return` is the declaration-owned requirement (possibly unknown).
/// `return_validation` indicates body certification state for source contracts.
/// `inferred_return` is a body-derived published result and must never be fed
/// back as the callable body's declaration constraint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallableSemanticSignature {
    pub callable: CallableId,
    pub owner: DeclarationId,
    pub side: DispatchSide,
    pub selector: Selector,
    pub generics: Option<GenericSignature>,
    pub parameters: Box<[CallableParameterSemantic]>,
    pub declared_return: DeclaredTypeFact,
    pub return_validation: ReturnContractValidation,
    pub inferred_return: Option<TypeKnowledge>,
    pub source: Option<SemanticSourceSpan>,
    pub implementation: ImplementationKind,
    pub native_id: Option<NativeSurfaceId>,
    pub effects: EffectSpec,
    pub raises: RaisesSpec,
    pub flow: ReturnFlowSpec,
    pub lifecycle: NativeLifecycleSpec,
}

/// Origin of a canonical instance initializer for one class-side constructor.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ConstructorOrigin {
    /// A source-declared `@constructor` method.
    Source,
    /// The constructor generated by the class-level `@construct` attribute.
    Construct,
    /// The constructor generated by the class-level `@data` attribute.
    Data,
    /// The zero-argument path synthesized for a class whose fields all have defaults.
    DefaultOnly,
}

/// A generated constructor parameter bound to the field it initializes.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ConstructorParameterField {
    /// Canonical parameter identity in the class-side factory signature.
    pub parameter: CallableParameterId,
    /// Declaration-owned field identity written by the paired initializer.
    pub field: FieldId,
}

/// Canonical construction path paired across the factory and same-instance initializer.
///
/// The factory signature lives in [`CallableSignatureTable`]. This product
/// records the compiler-specific initializer identity and the field effects
/// that semantic checking and lowering share.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ConstructorInitializationProduct {
    /// Canonical class-side factory callable identity.
    pub factory: CallableId,
    /// Paired declaring-owner instance-side initializer identity.
    pub initializer: CallableId,
    /// Whether the path is source-declared or generated.
    pub origin: ConstructorOrigin,
    /// Fingerprint of this owner's ordered instance-field declarations,
    /// including defaults and generated-parameter field types. The compiler
    /// obtains expressions from source using the canonical `FieldId`s, while
    /// this fingerprint exposes source edits to constructor-product dependents.
    pub field_input_fingerprint: crate::db::ProductFingerprint,
    /// Own instance fields initialized from declaration defaults, in declaration order.
    pub default_fields: Box<[FieldId]>,
    /// Generated parameter-to-field bindings, in constructor parameter order.
    pub parameter_fields: Box<[ConstructorParameterField]>,
}

/// Identity-indexed canonical construction paths retained with a semantic snapshot.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConstructorInitializationTable {
    by_factory: BTreeMap<CallableId, ConstructorInitializationProduct>,
    module_factories: BTreeMap<ModuleId, BTreeSet<CallableId>>,
}

impl ConstructorInitializationTable {
    /// Inserts or replaces a canonical construction path.
    pub fn insert(&mut self, product: ConstructorInitializationProduct) {
        self.module_factories
            .entry(product.factory.module().clone())
            .or_default()
            .insert(product.factory.clone());
        self.by_factory.insert(product.factory.clone(), product);
    }

    /// Returns the path for one class-side factory identity.
    pub fn get(&self, factory: &CallableId) -> Option<&ConstructorInitializationProduct> {
        self.by_factory.get(factory)
    }

    /// Returns all paths in stable identity order.
    pub fn iter(&self) -> impl Iterator<Item = (&CallableId, &ConstructorInitializationProduct)> {
        self.by_factory.iter()
    }

    /// Removes every path contributed by a source module.
    pub fn remove_module(&mut self, module: &ModuleId) {
        if let Some(factories) = self.module_factories.remove(module) {
            for factory in factories {
                self.by_factory.remove(&factory);
            }
        }
    }
}

impl CallableSemanticSignature {
    pub fn parameter_count(&self) -> usize {
        self.parameters.len()
    }

    pub fn parameter_at(&self, index: usize) -> Option<&CallableParameterSemantic> {
        self.parameters.get(index)
    }

    pub fn parameter_declared_type_at(&self, index: usize) -> Option<&DeclaredTypeFact> {
        self.parameters.get(index).map(|parameter| &parameter.declared_type)
    }

    pub fn published_return_knowledge(&self) -> TypeKnowledge {
        if self.declared_return.is_known() {
            let declared = self.declared_return.to_knowledge();
            if self.declared_return.basis == DeclaredTypeBasis::SourceAnnotation
                && self.return_validation == ReturnContractValidation::Satisfied(EvidenceStatus::Established)
            {
                if let Some(ty) = declared.ty() {
                    return TypeKnowledge::established(ty, EvidenceOrigin::CallableSignature);
                }
            }
            return declared;
        }

        if self.declared_return.is_dynamic() {
            return self.declared_return.to_knowledge();
        }

        self.inferred_return
            .as_ref()
            .filter(|knowledge| knowledge.is_known() || knowledge.is_dynamic())
            .cloned()
            .unwrap_or_else(|| self.declared_return.to_knowledge())
    }

    pub fn published_return_term(&self) -> Option<TypeTerm> {
        self.declared_return
            .known_term()
            .cloned()
            .or_else(|| self.inferred_return.as_ref().and_then(TypeKnowledge::ty).map(TypeTerm::Canonical))
    }

    /// Whether this declaration is a constructor according to declaration-owned
    /// semantic facts. Dispatch kind is a projection of this fact, never its source.
    pub fn is_constructor(&self) -> bool {
        self.declared_return.basis == DeclaredTypeBasis::ConstructorSemantics
    }

    pub fn is_complete(&self) -> bool {
        self.parameters.iter().all(|parameter| parameter.declared_type.is_known())
            && (self.declared_return.is_known() || self.inferred_return.as_ref().is_some_and(TypeKnowledge::is_known))
    }

    /// Compares the declaration-owned callable contract while ignoring
    /// evidence provenance and source-local parameter names.  Native/source
    /// conformance must compare the canonical type facts already published by
    /// this signature rather than re-lowering type syntax in a backend.
    pub fn native_contract_mismatch(&self, expected: &Self) -> Option<String> {
        self.native_contract_mismatch_with_store(expected, None)
    }

    /// Store-aware native/source contract comparison.  A native receiver-flow
    /// constructor is canonically a `Self` type, while source syntax may spell
    /// the same contract as the owner's applied nominal type.  The shared
    /// store is the authority for that one representation equivalence.
    pub fn native_contract_mismatch_with_store(&self, expected: &Self, store: Option<&crate::types::store::TypeStore>) -> Option<String> {
        if self.owner != expected.owner {
            return Some("owner identity differs".into());
        }
        if self.side != expected.side {
            return Some("dispatch side differs".into());
        }
        if self.selector != expected.selector {
            return Some("selector differs".into());
        }
        if self.generics != expected.generics {
            return Some("generic parameter contract differs".into());
        }
        if self.parameters.len() != expected.parameters.len() {
            return Some(format!(
                "parameter count differs (source {}, native {})",
                self.parameters.len(),
                expected.parameters.len()
            ));
        }
        for (index, (source, native)) in self.parameters.iter().zip(expected.parameters.iter()).enumerate() {
            if source.external_label != native.external_label {
                return Some(format!("parameter {index} label differs"));
            }
            if source.contract.rest_mode() != native.contract.rest_mode() {
                return Some(format!("parameter {index} rest mode differs"));
            }
            if !native_parameter_contract_matches(&source.contract, &native.contract, store) {
                return Some(format!("parameter {index} type differs"));
            }
        }
        let return_matches = if self.declared_return.is_unknown() || self.declared_return.is_dynamic() {
            true
        } else if self.selector.kind == phalcom_common::selector::SelectorKind::Setter {
            // Setter syntax publishes the language's assignment result (Unit),
            // while a native setter may be a terminating validation boundary.
            // Its parameter contract remains exact; the backend termination
            // fact is carried separately by the native flow metadata.
            true
        } else if let (Some(source), Some(native), Some(store)) = (self.declared_return.canonical_type(), expected.declared_return.canonical_type(), store) {
            if matches!(store.get(source), crate::types::store::TypeData::SelfType(_))
                && matches!(store.get(native), crate::types::store::TypeData::SelfType(_))
            {
                canonical_native_type_matches(source, native, store)
            } else if expected.flow == ReturnFlowSpec::Receiver {
                match store.get(native) {
                    crate::types::store::TypeData::SelfType(self_term) => store
                        .nominal_origin_declaration(source)
                        .is_some_and(|declaration| declaration == &self_term.owner),
                    _ => canonical_native_type_matches(source, native, store),
                }
            } else {
                canonical_native_type_matches(source, native, store)
            }
        } else if expected.flow == ReturnFlowSpec::Receiver {
            match (self.declared_return.canonical_type(), expected.declared_return.canonical_type(), store) {
                (Some(source), Some(native), Some(store)) => match store.get(native) {
                    crate::types::store::TypeData::SelfType(self_term) => store
                        .nominal_origin_declaration(source)
                        .is_some_and(|declaration| declaration == &self_term.owner),
                    _ => canonical_native_type_matches(source, native, store),
                },
                _ => self.declared_return.state == expected.declared_return.state,
            }
        } else {
            match (self.declared_return.canonical_type(), expected.declared_return.canonical_type(), store) {
                (Some(source), Some(native), Some(store)) => canonical_native_type_matches(source, native, store),
                _ => {
                    expected.declared_return.is_unknown()
                        || expected.declared_return.is_dynamic()
                        || self.declared_return.state == expected.declared_return.state
                }
            }
        };
        if !return_matches {
            return Some("return type differs".into());
        }
        None
    }
}

fn native_parameter_contract_matches(source: &ParameterContract, native: &ParameterContract, store: Option<&crate::types::store::TypeStore>) -> bool {
    match (source, native) {
        (ParameterContract::Fixed(source), ParameterContract::Fixed(native)) => native_declared_fact_matches(source, native, store),
        (
            ParameterContract::Rest {
                mode: source_mode,
                contract: source,
            },
            ParameterContract::Rest {
                mode: native_mode,
                contract: native,
            },
        ) if source_mode == native_mode => match (source, native) {
            (RestValueContract::Unconstrained, RestValueContract::Unconstrained) => true,
            (RestValueContract::Element(source), RestValueContract::Element(native)) => native_declared_fact_matches(source, native, store),
            (RestValueContract::Lane(source), RestValueContract::Lane(native)) => source == native,
            (RestValueContract::Invalid(source), RestValueContract::Invalid(native)) => source == native,
            _ => false,
        },
        _ => false,
    }
}

fn native_declared_fact_matches(source: &DeclaredTypeFact, native: &DeclaredTypeFact, store: Option<&crate::types::store::TypeStore>) -> bool {
    if source.is_unknown() || source.is_dynamic() || native.is_unknown() || native.is_dynamic() {
        return true;
    }
    match (source.canonical_type(), native.canonical_type(), store) {
        (Some(source), Some(native), Some(store)) => canonical_native_type_matches(source, native, store),
        _ => source.state == native.state,
    }
}

fn canonical_native_type_matches(source: crate::types::id::TypeId, native: crate::types::id::TypeId, store: &crate::types::store::TypeStore) -> bool {
    if source == native {
        return true;
    }
    if let (crate::types::store::TypeData::SelfType(source_self), crate::types::store::TypeData::SelfType(native_self)) = (store.get(source), store.get(native))
    {
        return source_self.owner == native_self.owner && source_self.side == native_self.side;
    }
    let Some((source_decl, source_args)) = store.applied_nominal_parts(source) else {
        return false;
    };
    let Some((native_decl, _native_args)) = store.applied_nominal_parts(native) else {
        return false;
    };
    if native_decl == crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Object) {
        return true;
    }
    // Native surface metadata names a generic Universe type by its root
    // `UniverseKey` (for example `Option`), while source annotations retain
    // the applied arguments (`Option<String>`).  The canonical descriptor
    // contract therefore compares the shared nominal origin; the descriptor
    // has no argument-level fact to compare.
    source_decl == native_decl && !source_args.is_empty()
}

/// Canonical field signature.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldSemanticSignature {
    pub field: FieldId,
    pub owner: DeclarationId,
    pub side: DispatchSide,
    pub name: Box<str>,
    pub mutable: bool,
    pub declared_type: DeclaredTypeFact,
    pub source: Option<SemanticSourceSpan>,
}

/// Identity-indexed table of callable signatures.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CallableSignatureTable {
    by_id: HashMap<CallableId, CallableSemanticSignature>,
    module_callables: HashMap<ModuleId, BTreeSet<CallableId>>,
}

impl CallableSignatureTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, sig: CallableSemanticSignature) {
        self.module_callables
            .entry(sig.callable.module().clone())
            .or_default()
            .insert(sig.callable.clone());
        self.by_id.insert(sig.callable.clone(), sig);
    }

    pub fn remove_module(&mut self, module: &phalcom_modules::identity::ModuleId) {
        if let Some(callables) = self.module_callables.remove(module) {
            for callable in callables {
                self.by_id.remove(&callable);
            }
        }
    }

    /// Removes one declaration-owned signature while retaining other module
    /// contributions.
    pub fn remove(&mut self, callable: &CallableId) {
        let Some(signature) = self.by_id.remove(callable) else {
            return;
        };
        if let Some(callables) = self.module_callables.get_mut(signature.callable.module()) {
            callables.remove(callable);
            if callables.is_empty() {
                self.module_callables.remove(signature.callable.module());
            }
        }
    }

    pub fn get(&self, callable: &CallableId) -> Option<&CallableSemanticSignature> {
        self.by_id.get(callable)
    }

    /// Resolves the declaration-owned signature consumed by one analyzed body.
    ///
    /// Constructor bodies execute instance-side while their public declaration
    /// and parameter identities are class-side. Ordinary bodies retain exact
    /// callable identity and never fall through to an unrelated class method.
    pub fn get_for_body(&self, callable: &CallableId) -> Option<&CallableSemanticSignature> {
        self.get(callable).or_else(|| {
            if callable.side != DispatchSide::Instance {
                return None;
            }
            let declared = CallableId::new(callable.owner.clone(), callable.selector.clone(), DispatchSide::Class);
            self.get(&declared).filter(|signature| signature.is_constructor())
        })
    }

    pub fn get_mut(&mut self, callable: &CallableId) -> Option<&mut CallableSemanticSignature> {
        self.by_id.get_mut(callable)
    }

    /// Returns the canonical declaration identity corresponding to one body.
    pub fn id_for_body(&self, callable: &CallableId) -> Option<CallableId> {
        self.get_for_body(callable).map(|signature| signature.callable.clone())
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&CallableId, &CallableSemanticSignature)> {
        self.by_id.iter()
    }
}

/// Identity-indexed table of field signatures.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FieldSignatureTable {
    by_id: HashMap<FieldId, FieldSemanticSignature>,
    module_fields: HashMap<ModuleId, BTreeSet<FieldId>>,
}

impl FieldSignatureTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, sig: FieldSemanticSignature) {
        self.module_fields.entry(sig.field.owner.module.clone()).or_default().insert(sig.field.clone());
        self.by_id.insert(sig.field.clone(), sig);
    }

    pub fn remove_module(&mut self, module: &phalcom_modules::identity::ModuleId) {
        if let Some(fields) = self.module_fields.remove(module) {
            for field in fields {
                self.by_id.remove(&field);
            }
        }
    }

    /// Removes one declaration-owned field signature while retaining other
    /// module contributions.
    pub fn remove(&mut self, field: &FieldId) {
        let Some(signature) = self.by_id.remove(field) else {
            return;
        };
        if let Some(fields) = self.module_fields.get_mut(&signature.owner.module) {
            fields.remove(field);
            if fields.is_empty() {
                self.module_fields.remove(&signature.owner.module);
            }
        }
    }

    pub fn get(&self, field: &FieldId) -> Option<&FieldSemanticSignature> {
        self.by_id.get(field)
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&FieldId, &FieldSemanticSignature)> {
        self.by_id.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::id::TypeId;

    #[test]
    fn published_return_knowledge_source_annotation_is_assumed_until_satisfied_established() {
        let owner = DeclarationId::new(crate::identity::ModuleId::universe_root(), "Test".into());
        let selector = Selector::method("run", vec![]).unwrap();
        let callable = CallableId::new(owner.clone(), selector.clone(), DispatchSide::Instance);
        let int_fact = DeclaredTypeFact::known(TypeTerm::Canonical(TypeId(1)), DeclaredTypeBasis::SourceAnnotation);

        let mut sig = CallableSemanticSignature {
            callable: callable.clone(),
            owner,
            side: DispatchSide::Instance,
            selector,
            generics: None,
            parameters: Box::new([]),
            declared_return: int_fact,
            return_validation: ReturnContractValidation::Unchecked,
            inferred_return: None,
            source: None,
            implementation: ImplementationKind::Phalcom,
            native_id: None,
            effects: EffectSpec::Unknown,
            raises: RaisesSpec::Unknown,
            flow: ReturnFlowSpec::Value,
            lifecycle: NativeLifecycleSpec::UNKNOWN,
        };

        assert_eq!(sig.published_return_knowledge().status(), Some(EvidenceStatus::Assumed));

        sig.return_validation = ReturnContractValidation::Satisfied(EvidenceStatus::Assumed);
        assert_eq!(sig.published_return_knowledge().status(), Some(EvidenceStatus::Assumed));

        sig.return_validation = ReturnContractValidation::Satisfied(EvidenceStatus::Established);
        assert_eq!(sig.published_return_knowledge().status(), Some(EvidenceStatus::Established));
    }

    #[test]
    fn published_return_term_preserves_declared_term() {
        let owner = DeclarationId::new(crate::identity::ModuleId::universe_root(), "Test".into());
        let selector = Selector::method("run", vec![]).unwrap();
        let callable = CallableId::new(owner.clone(), selector.clone(), DispatchSide::Instance);
        let declared = DeclaredTypeFact::known(TypeTerm::Canonical(TypeId(10)), DeclaredTypeBasis::SourceAnnotation);

        let sig = CallableSemanticSignature {
            callable: callable.clone(),
            owner,
            side: DispatchSide::Instance,
            selector,
            generics: None,
            parameters: Box::new([]),
            declared_return: declared,
            return_validation: ReturnContractValidation::Satisfied(EvidenceStatus::Established),
            inferred_return: Some(TypeKnowledge::established(TypeId(20), EvidenceOrigin::Flow)),
            source: None,
            implementation: ImplementationKind::Phalcom,
            native_id: None,
            effects: EffectSpec::Unknown,
            raises: RaisesSpec::Unknown,
            flow: ReturnFlowSpec::Value,
            lifecycle: NativeLifecycleSpec::UNKNOWN,
        };

        assert_eq!(sig.published_return_term(), Some(TypeTerm::Canonical(TypeId(10))));
    }

    fn native_contract_test_signature(parameter: TypeId, return_type: TypeId) -> CallableSemanticSignature {
        let owner = DeclarationId::new(crate::identity::ModuleId::universe_root(), "Test".into());
        let selector = Selector::method("run", vec![]).unwrap();
        let callable = CallableId::new(owner.clone(), selector.clone(), DispatchSide::Instance);
        let parameter_id = CallableParameterId::new(callable.clone(), 0);
        CallableSemanticSignature {
            callable,
            owner,
            side: DispatchSide::Instance,
            selector,
            generics: None,
            parameters: vec![CallableParameterSemantic::new(
                parameter_id,
                "value",
                DeclaredTypeFact::known(TypeTerm::Canonical(parameter), DeclaredTypeBasis::NativeSignature),
            )]
            .into_boxed_slice(),
            declared_return: DeclaredTypeFact::known(TypeTerm::Canonical(return_type), DeclaredTypeBasis::NativeSignature),
            return_validation: ReturnContractValidation::NotApplicable,
            inferred_return: None,
            source: None,
            implementation: ImplementationKind::Native,
            native_id: None,
            effects: EffectSpec::Unknown,
            raises: RaisesSpec::Unknown,
            flow: ReturnFlowSpec::Value,
            lifecycle: NativeLifecycleSpec::UNKNOWN,
        }
    }

    #[test]
    fn native_contract_rejects_parameter_and_return_type_mismatches() {
        let source = native_contract_test_signature(TypeId(1), TypeId(2));
        let matching = native_contract_test_signature(TypeId(1), TypeId(2));
        assert_eq!(source.native_contract_mismatch(&matching), None);

        let parameter_mismatch = native_contract_test_signature(TypeId(3), TypeId(2));
        assert_eq!(
            source.native_contract_mismatch(&parameter_mismatch).as_deref(),
            Some("parameter 0 type differs")
        );

        let return_mismatch = native_contract_test_signature(TypeId(1), TypeId(4));
        assert_eq!(source.native_contract_mismatch(&return_mismatch).as_deref(), Some("return type differs"));
    }
}
