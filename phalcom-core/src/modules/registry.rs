//! Runtime module registry and module lifecycle state machine.

use crate::error::PhError;
use crate::heap::ObjRef;
use phalcom_modules::ModuleId;
use std::collections::HashMap;

/// Lifecycle state machine for module execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModuleState {
    /// Structure allocated, initializers not yet run.
    Prepared,
    /// Currently running top-level initializer.
    Initializing,
    /// Initializer completed successfully.
    Initialized,
    /// Initializer or dependency failed.
    Failed,
}

/// Structured failure cause recorded when module initialization fails.
#[derive(Clone, Debug)]
pub enum ModuleFailure {
    /// This module's own top-level initializer threw an error.
    Initializer { cause: Box<PhError> },
    /// A required dependency failed to initialize.
    Dependency { dependency: ModuleId, cause: Box<ModuleFailure> },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RuntimeProgramId(pub u64);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ModulePlanFingerprint(pub u64);

/// Runtime lifecycle record for one materialized module.
#[derive(Debug)]
pub struct ModuleRecord {
    /// Handle to the heap ModuleObject.
    pub object: ObjRef,
    /// Runtime program generation that owns this identity.
    pub program: RuntimeProgramId,
    /// Fingerprint of the immutable linked/materialization plan.
    pub plan_fingerprint: ModulePlanFingerprint,
    /// Current initialization state.
    pub state: ModuleState,
    /// Sticky failure cause if state is Failed.
    pub failure: Option<ModuleFailure>,
}

impl ModuleRecord {
    /// Creates a record in the `Prepared` state.
    pub fn prepared(object: ObjRef, program: RuntimeProgramId, plan_fingerprint: ModulePlanFingerprint) -> Self {
        Self {
            object,
            program,
            plan_fingerprint,
            state: ModuleState::Prepared,
            failure: None,
        }
    }
}

/// Error returned when attempting to register an already existing module identity without explicit reset.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("duplicate module identity: {0}")]
pub struct DuplicateModuleIdentity(pub ModuleId);

/// Runtime module registry on the VM.
#[derive(Debug, Default)]
pub struct ModuleRegistry {
    by_id: HashMap<ModuleId, ModuleRecord>,
}

impl ModuleRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self { by_id: HashMap::new() }
    }

    /// Registers a module record under its semantic identity if not already present.
    pub fn register_new(&mut self, id: ModuleId, record: ModuleRecord) -> Result<(), DuplicateModuleIdentity> {
        if self.by_id.contains_key(&id) {
            Err(DuplicateModuleIdentity(id))
        } else {
            self.by_id.insert(id, record);
            Ok(())
        }
    }

    /// Registers or overwrites a module record under its semantic identity.
    pub fn insert(&mut self, id: ModuleId, record: ModuleRecord) {
        self.by_id.insert(id, record);
    }

    /// Returns a reference to the record for `id`.
    pub fn get(&self, id: &ModuleId) -> Option<&ModuleRecord> {
        self.by_id.get(id)
    }

    /// Returns a mutable reference to the record for `id`.
    pub fn get_mut(&mut self, id: &ModuleId) -> Option<&mut ModuleRecord> {
        self.by_id.get_mut(id)
    }

    /// Returns whether `id` is registered.
    pub fn contains_key(&self, id: &ModuleId) -> bool {
        self.by_id.contains_key(id)
    }

    /// Iterates over all registered `(ModuleId, ModuleRecord)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&ModuleId, &ModuleRecord)> {
        self.by_id.iter()
    }

    /// Traces all registered module handles for garbage collection.
    pub fn each_handle(&self, push: &mut impl FnMut(ObjRef)) {
        for record in self.by_id.values() {
            push(record.object);
        }
    }
}
