//! Checking context and scope environments.

use crate::declarations::DeclarationTypeTable;
use crate::diagnostic::SemanticDiagnostic;
use crate::dispatch::{DispatchResult, SurfaceDispatchResolver};
use crate::identity::{DeclarationId, DispatchSide, ModuleId};
use crate::types::annotation::TypeResolver;
use crate::types::denotation::ValueSemanticFact;
use crate::types::evidence::TypeKnowledge;
use crate::types::id::TypeId;
use crate::types::native::register_native_surfaces;
use crate::types::relation::TypeHierarchy;
use crate::types::store::{TypeData, TypeStore};
use phalcom_common::selector::Selector;
use phalcom_native_surface::NATIVE_SURFACES;
use std::collections::HashMap;

/// Environment of local bindings in current lexical block/scope.
#[derive(Clone, Debug, Default)]
pub struct LocalEnv {
    bindings: HashMap<String, ValueSemanticFact>,
}

impl LocalEnv {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: impl Into<String>, fact: ValueSemanticFact) {
        self.bindings.insert(name.into(), fact);
    }

    pub fn get(&self, name: &str) -> Option<&ValueSemanticFact> {
        self.bindings.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut ValueSemanticFact> {
        self.bindings.get_mut(name)
    }
}

/// The active context during semantic type checking.
pub struct CheckingContext<'a> {
    pub store: &'a mut TypeStore,
    pub hierarchy: &'a dyn TypeHierarchy,
    pub resolver: &'a dyn TypeResolver,
    pub declarations: &'a DeclarationTypeTable,
    pub current_module: ModuleId,
    pub current_class: Option<DeclarationId>,
    pub current_side: DispatchSide,
    pub expected_return: Option<TypeKnowledge>,
    pub local_envs: Vec<LocalEnv>,
    pub dispatch: SurfaceDispatchResolver,
    pub diagnostics: Vec<SemanticDiagnostic>,
}

impl<'a> CheckingContext<'a> {
    pub fn new(
        store: &'a mut TypeStore,
        hierarchy: &'a dyn TypeHierarchy,
        resolver: &'a dyn TypeResolver,
        declarations: &'a DeclarationTypeTable,
        current_module: ModuleId,
    ) -> Self {
        let mut dispatch = SurfaceDispatchResolver::new();
        // Focused type-checking fixtures can intentionally provide only a
        // subset of declarations. Import the complete native core only when
        // every native owner is available; normal workspace setup always
        // satisfies this and still fails loudly on malformed native metadata.
        let has_complete_native_core = NATIVE_SURFACES.iter().all(|record| {
            let owner = resolver
                .resolve_type_name(&current_module, record.owner().name(), &[])
                .unwrap_or_else(|| DeclarationId::new(ModuleId::core(), record.owner().name().into()));
            declarations.form(&owner).is_some()
        });
        if has_complete_native_core {
            register_native_surfaces(store, declarations, resolver, &current_module, &mut dispatch)
                .expect("canonical native surface must import during checking");
        }

        Self {
            store,
            hierarchy,
            resolver,
            declarations,
            current_module,
            current_class: None,
            current_side: DispatchSide::Instance,
            expected_return: None,
            local_envs: vec![LocalEnv::new()],
            dispatch,
            diagnostics: Vec::new(),
        }
    }

    pub fn push_scope(&mut self) {
        self.local_envs.push(LocalEnv::new());
    }

    pub fn pop_scope(&mut self) {
        self.local_envs.pop();
    }

    pub fn bind_local(&mut self, name: impl Into<String>, fact: ValueSemanticFact) {
        if let Some(env) = self.local_envs.last_mut() {
            env.insert(name, fact);
        }
    }

    pub fn lookup_local(&self, name: &str) -> Option<&ValueSemanticFact> {
        for env in self.local_envs.iter().rev() {
            if let Some(k) = env.get(name) {
                return Some(k);
            }
        }
        None
    }

    pub fn lookup_local_knowledge(&self, name: &str) -> Option<TypeKnowledge> {
        self.lookup_local(name).map(|f| f.knowledge.clone())
    }

    pub fn assign_existing(&mut self, name: &str, fact: ValueSemanticFact) -> bool {
        for env in self.local_envs.iter_mut().rev() {
            if let Some(slot) = env.get_mut(name) {
                *slot = fact;
                return true;
            }
        }
        false
    }

    pub fn resolve_dispatch(&mut self, receiver: TypeId, selector: &Selector, lookup: crate::dispatch::DispatchLookup) -> DispatchResult {
        let (decl, side) = match lookup {
            crate::dispatch::DispatchLookup::Super { defining_class, side } => {
                if let Some(super_decl) = self.hierarchy.superclass(&defining_class) {
                    (super_decl.clone(), side)
                } else {
                    return DispatchResult::Missing;
                }
            }
            crate::dispatch::DispatchLookup::Normal => match self.store.get(receiver) {
                TypeData::ClassObject { declaration } => (declaration.clone(), DispatchSide::Class),
                TypeData::Nominal { declaration } => (declaration.clone(), DispatchSide::Instance),
                TypeData::Applied { origin, .. } => {
                    let mut curr_origin = *origin;
                    while let TypeData::Applied { origin: inner_origin, .. } = self.store.get(curr_origin) {
                        curr_origin = *inner_origin;
                    }
                    if let TypeData::Nominal { declaration } = self.store.get(curr_origin) {
                        (declaration.clone(), DispatchSide::Instance)
                    } else {
                        return DispatchResult::Missing;
                    }
                }
                _ => return DispatchResult::Missing,
            },
        };

        let res = self.dispatch.resolve_dispatch_on_owner(self.hierarchy, &decl, side, selector);
        if let DispatchResult::Found(mut sig) = res {
            if let Some(subst) = crate::types::substitution::substitution_for_applied(self.declarations, self.store, receiver) {
                for param in &mut sig.parameters {
                    if let TypeKnowledge::Known(ref mut ev) = param.ty {
                        ev.ty = subst.apply(self.store, ev.ty);
                    }
                }
                if let TypeKnowledge::Known(ref mut ev) = sig.return_type {
                    ev.ty = subst.apply(self.store, ev.ty);
                }
            }
            DispatchResult::Found(sig)
        } else {
            res
        }
    }

    pub fn register_surface(&mut self, decl: DeclarationId, surface: crate::surface::DeclarationSurface) {
        self.dispatch.register_surface(decl, surface);
    }

    pub fn nominal_type_of(&mut self, decl: &DeclarationId) -> TypeId {
        if let Some(form) = self.declarations.form(decl) {
            form
        } else {
            self.store.nominal_type(decl.clone())
        }
    }
}
