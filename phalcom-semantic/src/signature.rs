//! Canonical callable and field semantic signatures and identity-indexed tables.

use super::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact};
use super::diagnostic::SemanticSourceSpan;
use super::identity::{CallableId, CallableParameterId, DeclarationId, DispatchSide, FieldId, ModuleId};
use super::types::evidence::{EvidenceOrigin, EvidenceStatus, TypeKnowledge};
use super::types::parameter::{GenericSignature, TypeTerm};
use phalcom_ast::ast::RestMode;
use phalcom_common::modifier::ImplementationKind;
use phalcom_common::selector::Selector;
use phalcom_native_meta::{EffectSpec, NativeLifecycleSpec, RaisesSpec, ReturnFlowSpec};
use phalcom_native_surface::NativeSurfaceId;
use std::collections::{BTreeSet, HashMap};

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

/// Canonical parameter specification for a callable signature.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallableParameterSemantic {
    pub id: CallableParameterId,
    pub local_name: Box<str>,
    pub external_label: Option<Box<str>>,
    pub rest: RestMode,
    pub declared_type: DeclaredTypeFact,
    pub source: Option<SemanticSourceSpan>,
}

impl CallableParameterSemantic {
    pub fn new(id: CallableParameterId, local_name: impl Into<Box<str>>, declared_type: DeclaredTypeFact) -> Self {
        Self {
            id,
            local_name: local_name.into(),
            external_label: None,
            rest: RestMode::None,
            declared_type,
            source: None,
        }
    }

    pub fn index(&self) -> u32 {
        self.id.index
    }

    pub fn with_label(mut self, label: impl Into<Box<str>>) -> Self {
        self.external_label = Some(label.into());
        self
    }

    pub fn with_rest(mut self, rest: RestMode) -> Self {
        self.rest = rest;
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
            if source.rest != native.rest {
                return Some(format!("parameter {index} rest mode differs"));
            }
            let parameter_types_match = if source.declared_type.is_unknown()
                || source.declared_type.is_dynamic()
                || native.declared_type.is_unknown()
                || native.declared_type.is_dynamic()
            {
                true
            } else {
                match (source.declared_type.canonical_type(), native.declared_type.canonical_type(), store) {
                    (Some(source), Some(native), Some(store)) => canonical_native_type_matches(source, native, store),
                    _ => source.declared_type.state == native.declared_type.state,
                }
            };
            if !parameter_types_match {
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
