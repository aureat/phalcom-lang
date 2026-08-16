//! Runtime module registry and module lifecycle state machine.

use crate::error::PhError;
use crate::heap::ObjRef;
use phalcom_modules::{ModuleId, ProjectIdentity};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RuntimeProgramId(u64);
static NEXT_RUNTIME_PROGRAM_ID: AtomicU64 = AtomicU64::new(1);
impl RuntimeProgramId {
    pub(crate) fn fresh() -> Self {
        let id = NEXT_RUNTIME_PROGRAM_ID.fetch_add(1, Ordering::Relaxed);
        assert!(id != 0, "runtime program identity space exhausted");
        Self(id)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModulePlanFingerprint(u64);
impl ModulePlanFingerprint {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut hash = 0xcbf29ce484222325u64;
        for byte in bytes { hash ^= u64::from(*byte); hash = hash.wrapping_mul(0x100000001b3); }
        Self(hash)
    }
    pub const fn empty() -> Self { Self(0xcbf29ce484222325) }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModuleOwner { Builtin, Program(RuntimeProgramId) }
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModuleState { Prepared, Initializing, Initialized, Failed }

#[derive(Clone, Debug)]
pub enum ModuleFailure {
    Initializer { cause: Box<PhError> },
    Dependency { dependency: ModuleId, cause: Box<ModuleFailure> },
}

#[derive(Debug)]
pub struct ModuleRecord {
    pub object: ObjRef,
    pub state: ModuleState,
    pub failure: Option<ModuleFailure>,
    pub owner: ModuleOwner,
    pub plan_fingerprint: Option<ModulePlanFingerprint>,
}

impl ModuleRecord {
    pub fn prepared(object: ObjRef) -> Self {
        Self::prepared_for(object, RuntimeProgramId::fresh(), ModulePlanFingerprint::empty())
    }
    pub fn prepared_for(object: ObjRef, program: RuntimeProgramId, fingerprint: ModulePlanFingerprint) -> Self {
        Self { object, state: ModuleState::Prepared, failure: None, owner: ModuleOwner::Program(program), plan_fingerprint: Some(fingerprint) }
    }
    pub fn builtin_prepared(object: ObjRef, fingerprint: ModulePlanFingerprint) -> Self {
        Self { object, state: ModuleState::Prepared, failure: None, owner: ModuleOwner::Builtin, plan_fingerprint: Some(fingerprint) }
    }
    pub fn builtin_bootstrap(object: ObjRef) -> Self {
        Self { object, state: ModuleState::Initialized, failure: None, owner: ModuleOwner::Builtin, plan_fingerprint: None }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ModuleRegistryError {
    #[error("duplicate module identity: {0}")]
    DuplicateIdentity(ModuleId),
    #[error("module identity {module} belongs to another runtime program")]
    ProgramOwnershipConflict { module: ModuleId },
    #[error("module identity {module} was already materialized with a different immutable plan")]
    PlanFingerprintMismatch { module: ModuleId },
}

#[derive(Debug, Default)]
pub struct ModuleRegistry { by_id: HashMap<ModuleId, ModuleRecord> }

impl ModuleRegistry {
    pub fn new() -> Self { Self { by_id: HashMap::new() } }

    pub fn register_new(&mut self, id: ModuleId, record: ModuleRecord) -> Result<(), ModuleRegistryError> {
        if self.by_id.contains_key(&id) { Err(ModuleRegistryError::DuplicateIdentity(id)) } else { self.by_id.insert(id, record); Ok(()) }
    }

    /// Crate-private migration seam for VM bootstrap. It is non-overwriting.
    /// Builtin identities are converted to initialized bootstrap shells so they
    /// can later adopt one linked interface without re-running builtin source.
    pub(crate) fn insert(&mut self, id: ModuleId, mut record: ModuleRecord) {
        if matches!(id.project, ProjectIdentity::Builtin(_)) {
            record.owner = ModuleOwner::Builtin;
            record.state = ModuleState::Initialized;
            record.failure = None;
            record.plan_fingerprint = None;
        }
        self.register_new(id, record).expect("VM attempted to register a duplicate semantic module identity");
    }

    pub fn get(&self, id: &ModuleId) -> Option<&ModuleRecord> { self.by_id.get(id) }
    pub fn get_mut(&mut self, id: &ModuleId) -> Option<&mut ModuleRecord> { self.by_id.get_mut(id) }
    pub fn contains_key(&self, id: &ModuleId) -> bool { self.by_id.contains_key(id) }
    pub fn iter(&self) -> impl Iterator<Item = (&ModuleId, &ModuleRecord)> { self.by_id.iter() }
    pub fn each_handle(&self, push: &mut impl FnMut(ObjRef)) { for record in self.by_id.values() { push(record.object); } }
}
