//! Dispatch models, callable signatures, and selector resolution.

use crate::identity::{CallableId, DeclarationId, ModuleId};

pub use crate::identity::DispatchSide;
use crate::surface::DeclarationSurface;
use crate::types::evidence::TypeKnowledge;
use crate::types::id::TypeId;
use crate::types::relation::TypeHierarchy;
use phalcom_common::selector::Selector;
use std::collections::{HashMap, HashSet};

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
    Found(Box<ResolvedDispatch>),
    Ambiguous(Vec<ResolvedDispatch>),
    Missing { visited_owners: Box<[DeclarationId]> },
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

    /// Replaces an inferred return type on an existing source callable while
    /// retaining its parameters, selector, and declaration identity.
    pub fn update_callable_return_type(&mut self, callable: &CallableId, return_type: TypeKnowledge) -> bool {
        let Some(surface) = self.surfaces.get_mut(&callable.owner) else {
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

    pub fn surfaces(&self) -> &HashMap<DeclarationId, DeclarationSurface> {
        &self.surfaces
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
                    declaration: DeclarationId::new(ModuleId::core(), "Class".into()),
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
                        .unwrap_or_else(|| CallableId::new(owner.declaration.clone(), selector.clone(), owner.side));
                    return ResolvedDispatchResult::Found(Box::new(ResolvedDispatch {
                        callable: callable_id,
                        signature: sig.clone(),
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
        for owner in self.dispatch_owners(hierarchy, start_decl, side) {
            if let Some(surface) = self.surfaces.get(&owner.declaration) {
                if let Some(id) = surface.get_callable_id(owner.side, selector) {
                    return Some(id.clone());
                }
            }
        }
        None
    }
}
