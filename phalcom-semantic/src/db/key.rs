//! Query keys, input fingerprints, and product fingerprints.

use crate::identity::{CallableId, DeclarationId, FieldId, ModuleId};

/// Fingerprint representing the input parameters to a query evaluation.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InputFingerprint(pub u64);

impl InputFingerprint {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

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
    ResolvedImports(ModuleId),
    LinkedInterface(ModuleId),
    DeclarationShell(DeclarationId),
    SemanticComponent(ModuleId),
    DeclarationSurface(DeclarationId),
    HierarchyEdge(DeclarationId),
    FieldSignature(FieldId),
    CallableSignature(CallableId),
    CallableBody(CallableId),
    CallableEffects(CallableId),
    CallableControl(CallableId),
    CallableTermination(CallableId),
    CallableContracts(CallableId),
    VerificationConditions(CallableId),
    SourceStructure(ModuleId),
    SourceFormalAttachment(CallableId),
    AdvisoryCallable(CallableId),
    AdvisoryModule(ModuleId),
    ModuleDiagnostics(ModuleId),
    ModuleMetadata(ModuleId),
}
