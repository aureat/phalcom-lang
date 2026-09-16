//! Runtime carrier for compiler-selected conformance targets.
//!
//! This registry contains only executable method handles.  It is deliberately
//! not a semantic lookup table: semantic analysis has already selected every
//! requirement target before an environment is interned.

use crate::heap::ObjRef;
use phalcom_semantic::traits::TraitRequirementId;
use std::collections::HashMap;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RuntimeConformanceEnvironmentId(pub u32);

impl RuntimeConformanceEnvironmentId {
    pub const EMPTY: Self = Self(0);
}

/// One executable conformance evidence slot holding a method handle and its
/// associated invocation type and conformance environments.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RuntimeConformanceSlot {
    pub method: ObjRef,
    pub type_environment: crate::typing::environment::RuntimeTypeEnvironmentId,
    pub conformance_environment: RuntimeConformanceEnvironmentId,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RuntimeConformanceEnvironment {
    pub slots: Box<[Option<RuntimeConformanceSlot>]>,
}

impl RuntimeConformanceEnvironment {
    pub fn new(slots: Vec<Option<RuntimeConformanceSlot>>) -> Self {
        Self {
            slots: slots.into_boxed_slice(),
        }
    }

    pub fn slot(&self, slot: u16) -> Option<RuntimeConformanceSlot> {
        self.slots.get(slot as usize).copied().flatten()
    }

    pub fn target(&self, slot: u16) -> Option<ObjRef> {
        self.slot(slot).map(|s| s.method)
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeConformanceEnvironmentRegistry {
    environments: Vec<RuntimeConformanceEnvironment>,
    interner: HashMap<RuntimeConformanceEnvironment, RuntimeConformanceEnvironmentId>,
}

impl Default for RuntimeConformanceEnvironmentRegistry {
    fn default() -> Self {
        let empty = RuntimeConformanceEnvironment::new(Vec::new());
        let mut interner = HashMap::new();
        interner.insert(empty.clone(), RuntimeConformanceEnvironmentId::EMPTY);
        Self {
            environments: vec![empty],
            interner,
        }
    }
}

impl RuntimeConformanceEnvironmentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intern(&mut self, environment: RuntimeConformanceEnvironment) -> RuntimeConformanceEnvironmentId {
        if let Some(&id) = self.interner.get(&environment) {
            return id;
        }
        let id = RuntimeConformanceEnvironmentId(self.environments.len() as u32);
        self.environments.push(environment.clone());
        self.interner.insert(environment, id);
        id
    }

    pub fn get(&self, id: RuntimeConformanceEnvironmentId) -> Option<&RuntimeConformanceEnvironment> {
        self.environments.get(id.0 as usize)
    }

    pub fn slot(&self, id: RuntimeConformanceEnvironmentId, slot: u16) -> Option<RuntimeConformanceSlot> {
        self.get(id).and_then(|environment| environment.slot(slot))
    }

    pub fn target(&self, id: RuntimeConformanceEnvironmentId, slot: u16) -> Option<ObjRef> {
        self.get(id).and_then(|environment| environment.target(slot))
    }

    pub fn handles(&self) -> impl Iterator<Item = ObjRef> + '_ {
        self.environments.iter().flat_map(|environment| environment.slots.iter().copied().flatten().map(|s| s.method))
    }

    pub fn len(&self) -> usize {
        self.environments.len()
    }
}

#[allow(dead_code)]
fn _requirement_identity_is_semantic(_: TraitRequirementId) {}
