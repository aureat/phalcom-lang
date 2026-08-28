//! Canonical callable and field semantic signatures and identity-indexed tables.

use super::declaration_type::{DeclaredTypeBasis, DeclaredTypeFact};
use super::diagnostic::SemanticSourceSpan;
use super::identity::{CallableId, CallableParameterId, DeclarationId, DispatchSide, FieldId};
use super::types::evidence::TypeKnowledge;
use super::types::parameter::{GenericSignature, TypeTerm};
use phalcom_ast::ast::RestMode;
use phalcom_common::selector::Selector;
use phalcom_native_meta::{EffectSpec, ImplementationKind, NativeLifecycleSpec, RaisesSpec, ReturnFlowSpec};
use phalcom_native_surface::NativeSurfaceId;
use std::collections::HashMap;

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
        self.inferred_return.clone().unwrap_or_else(|| self.declared_return.to_knowledge())
    }

    pub fn published_return_term(&self) -> Option<TypeTerm> {
        self.inferred_return
            .as_ref()
            .and_then(TypeKnowledge::ty)
            .map(TypeTerm::Canonical)
            .or_else(|| self.declared_return.known_term().cloned())
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
}

impl CallableSignatureTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, sig: CallableSemanticSignature) {
        self.by_id.insert(sig.callable.clone(), sig);
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
}

impl FieldSignatureTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, sig: FieldSemanticSignature) {
        self.by_id.insert(sig.field.clone(), sig);
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
