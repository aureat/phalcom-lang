//! Canonical callable and field semantic signatures and identity-indexed tables.

use super::diagnostic::SemanticSourceSpan;
use super::identity::{CallableId, DeclarationId, DispatchSide, FieldId};
use super::types::parameter::{GenericSignature, TypeTerm};
use phalcom_ast::ast::RestMode;
use phalcom_common::selector::Selector;
use phalcom_native_meta::{EffectSpec, ImplementationKind, NativeLifecycleSpec, RaisesSpec, ReturnFlowSpec};
use phalcom_native_surface::NativeSurfaceId;
use std::collections::HashMap;

/// Canonical parameter specification for a callable signature.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallableParameterSemantic {
    pub index: u32,
    pub local_name: Box<str>,
    pub external_label: Option<Box<str>>,
    pub rest: RestMode,
    pub ty: TypeTerm,
    pub source: Option<SemanticSourceSpan>,
}

impl CallableParameterSemantic {
    pub fn new(index: u32, local_name: impl Into<Box<str>>, ty: TypeTerm) -> Self {
        Self {
            index,
            local_name: local_name.into(),
            external_label: None,
            rest: RestMode::None,
            ty,
            source: None,
        }
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

/// Canonical semantic contract for a callable (method, getter, setter, indexer).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallableSemanticSignature {
    pub callable: CallableId,
    pub owner: DeclarationId,
    pub side: DispatchSide,
    pub selector: Selector,
    pub generics: Option<GenericSignature>,
    pub parameters: Box<[CallableParameterSemantic]>,
    pub return_type: TypeTerm,
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

    pub fn parameter_type_at(&self, index: usize) -> Option<&TypeTerm> {
        self.parameters.get(index).map(|p| &p.ty)
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
    pub ty: TypeTerm,
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
