//! Query keys and product fingerprints.

use crate::identity::{CallableId, DeclarationId, ModuleId};

/// Fingerprint representing the content or result of a computed query product.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProductFingerprint(pub u64);

impl ProductFingerprint {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Strongly-typed key identifying a semantic database query product.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum QueryKey {
    ParsedModule(ModuleId),
    UnlinkedInterface(ModuleId),
    LinkedInterface(ModuleId),
    DeclarationShell(DeclarationId),
    SemanticComponent(ModuleId),
    DeclarationSurface(DeclarationId),
    CallableBody(CallableId),
    CallableEffects(CallableId),
    CallableControl(CallableId),
    CallableTermination(CallableId),
    CallableContracts(CallableId),
    VerificationConditions(CallableId),
    ModuleDiagnostics(ModuleId),
    ModuleMetadata(ModuleId),
}
