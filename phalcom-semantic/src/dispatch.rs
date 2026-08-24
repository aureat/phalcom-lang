//! Dispatch models, callable signatures, and selector resolution.

use crate::identity::{CallableId, DeclarationId};

pub use crate::identity::DispatchSide;
use crate::surface::DeclarationSurface;
use crate::types::evidence::TypeKnowledge;
use crate::types::id::TypeId;
use crate::types::relation::TypeHierarchy;
use phalcom_common::selector::Selector;
use std::collections::HashMap;

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

/// Parameter in a callable signature.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallableParameter {
    pub external_label: Option<String>,
    pub local_name: String,
    pub ty: TypeKnowledge,
    pub rest: bool,
}

impl CallableParameter {
    pub fn new(local_name: impl Into<String>, ty: TypeKnowledge) -> Self {
        Self {
            external_label: None,
            local_name: local_name.into(),
            ty,
            rest: false,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.external_label = Some(label.into());
        self
    }

    pub fn with_rest(mut self, rest: bool) -> Self {
        self.rest = rest;
        self
    }
}

/// Complete callable contract for a method, getter, setter, or indexer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallableSignature {
    pub selector: Selector,
    pub parameters: Vec<CallableParameter>,
    pub return_type: TypeKnowledge,
    pub generics: Option<crate::types::parameter::GenericSignature>,
}

impl CallableSignature {
    pub fn new(selector: Selector, parameters: Vec<CallableParameter>, return_type: TypeKnowledge) -> Self {
        Self {
            selector,
            parameters,
            return_type,
            generics: None,
        }
    }

    pub fn with_generics(mut self, generics: crate::types::parameter::GenericSignature) -> Self {
        self.generics = Some(generics);
        self
    }

    /// Returns whether every parameter and the return value have canonical known types.
    ///
    /// Only complete source contracts can be projected into a
    /// [`CallableSemanticSignature`](crate::signature::CallableSemanticSignature).
    /// Partial source signatures stay represented by their declaration surface until
    /// inference publishes a richer canonical signature product.
    pub fn has_complete_types(&self) -> bool {
        self.return_type.ty().is_some()
            && self.parameters.iter().all(|parameter| parameter.ty.ty().is_some())
    }
}

/// Complete result of resolving dispatch, capturing visited hierarchy trace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedDispatch {
    pub callable: CallableId,
    pub signature: CallableSignature,
    pub visited_owners: Box<[DeclarationId]>,
}

/// Trace-aware result of resolving a message send selector against a receiver.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolvedDispatchResult {
    Found(ResolvedDispatch),
    Ambiguous(Vec<ResolvedDispatch>),
    Missing {
        visited_owners: Box<[DeclarationId]>,
    },
    Dynamic,
}

/// The result of resolving a message send selector against a receiver.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DispatchResult {
    Found(CallableSignature),
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

/// Trait for querying semantic dispatch targets.
pub trait DispatchResolver {
    fn resolve_dispatch(&self, receiver: TypeId, selector: &Selector, lookup: DispatchLookup) -> DispatchResult;
}

/// A concrete dispatch resolver backed by nominal class surfaces and type interning.
#[derive(Clone, Debug, Default)]
pub struct SurfaceDispatchResolver {
    surfaces: HashMap<DeclarationId, DeclarationSurface>,
    type_declarations: HashMap<TypeId, DeclarationId>,
}

impl SurfaceDispatchResolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_surface(&mut self, decl: DeclarationId, surface: DeclarationSurface) {
        self.surfaces.insert(decl, surface);
    }

    pub fn register_type(&mut self, ty: TypeId, decl: DeclarationId) {
        self.type_declarations.insert(ty, decl);
    }

    pub fn get_surface(&self, decl: &DeclarationId) -> Option<&DeclarationSurface> {
        self.surfaces.get(decl)
    }

    pub fn get_surface_mut(&mut self, decl: &DeclarationId) -> Option<&mut DeclarationSurface> {
        self.surfaces.get_mut(decl)
    }

    pub fn surfaces(&self) -> &HashMap<DeclarationId, DeclarationSurface> {
        &self.surfaces
    }

    pub fn resolve_dispatch_with_trace(
        &self,
        hierarchy: &dyn TypeHierarchy,
        start_decl: &DeclarationId,
        side: DispatchSide,
        selector: &Selector,
    ) -> ResolvedDispatchResult {
        let mut visited = Vec::new();
        let mut curr = Some(start_decl);
        while let Some(decl) = curr {
            visited.push(decl.clone());
            if let Some(surface) = self.surfaces.get(decl) {
                if let Some(sig) = surface.get_callable(side, selector) {
                    let callable_id = surface
                        .get_callable_id(side, selector)
                        .cloned()
                        .unwrap_or_else(|| CallableId::new(decl.clone(), selector.clone(), side));
                    return ResolvedDispatchResult::Found(ResolvedDispatch {
                        callable: callable_id,
                        signature: sig.clone(),
                        visited_owners: visited.into_boxed_slice(),
                    });
                }
            }
            curr = hierarchy.superclass(decl);
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
            ResolvedDispatchResult::Found(rd) => DispatchResult::Found(rd.signature),
            ResolvedDispatchResult::Ambiguous(amb) => {
                DispatchResult::Ambiguous(amb.into_iter().map(|rd| rd.signature).collect())
            }
            ResolvedDispatchResult::Missing { .. } => DispatchResult::Missing,
            ResolvedDispatchResult::Dynamic => DispatchResult::Dynamic,
        }
    }

    pub fn resolve_callable_id(
        &self,
        hierarchy: &dyn TypeHierarchy,
        start_decl: &DeclarationId,
        side: DispatchSide,
        selector: &Selector,
    ) -> Option<CallableId> {
        let mut curr = Some(start_decl);
        while let Some(decl) = curr {
            if let Some(surface) = self.surfaces.get(decl) {
                if let Some(id) = surface.get_callable_id(side, selector) {
                    return Some(id.clone());
                }
            }
            curr = hierarchy.superclass(decl);
        }
        None
    }
}
