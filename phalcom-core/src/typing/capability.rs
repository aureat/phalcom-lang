//! Explicit runtime authority for typing reflection.

use phalcom_type_meta::header::MetadataProfile;

/// Capability identities are VM-owned authority, not user-forgeable symbols.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TypingCapability {
    ObservePublicTypes,
    ObserveSignatures,
    ConstructTypeForms,
    EvaluateRelations,
    ObserveSourceUses,
    ObservePrivateTypes,
    ValidateRuntimeValues,
    InspectProofs,
    InvokeReflectively,
    ObserveImplementationProvenance,
}

impl TypingCapability {
    pub const ALL: [Self; 10] = [
        Self::ObservePublicTypes,
        Self::ObserveSignatures,
        Self::ConstructTypeForms,
        Self::EvaluateRelations,
        Self::ObserveSourceUses,
        Self::ObservePrivateTypes,
        Self::ValidateRuntimeValues,
        Self::InspectProofs,
        Self::InvokeReflectively,
        Self::ObserveImplementationProvenance,
    ];

    pub const fn display(self) -> &'static str {
        match self {
            Self::ObservePublicTypes => "OBSERVE_PUBLIC_TYPES",
            Self::ObserveSignatures => "OBSERVE_SIGNATURES",
            Self::ConstructTypeForms => "CONSTRUCT_TYPE_FORMS",
            Self::EvaluateRelations => "EVALUATE_RELATIONS",
            Self::ObserveSourceUses => "OBSERVE_SOURCE_USES",
            Self::ObservePrivateTypes => "OBSERVE_PRIVATE_TYPES",
            Self::ValidateRuntimeValues => "VALIDATE_RUNTIME_VALUES",
            Self::InspectProofs => "INSPECT_PROOFS",
            Self::InvokeReflectively => "INVOKE_REFLECTIVELY",
            Self::ObserveImplementationProvenance => "OBSERVE_IMPLEMENTATION_PROVENANCE",
        }
    }
}

/// Compact capability set carried by an immutable `TypingContext`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct TypingCapabilities(u16);

impl TypingCapabilities {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn for_profile(profile: MetadataProfile) -> Self {
        let public = (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3);
        let tooling = public | (1 << 4) | (1 << 9);
        let proof = tooling | (1 << 7);
        match profile {
            MetadataProfile::RuntimeMinimal => Self(0),
            MetadataProfile::RuntimePublic => Self(public),
            MetadataProfile::ToolingDebug => Self(tooling),
            MetadataProfile::Proof => Self(proof),
        }
    }

    pub const fn contains(self, capability: TypingCapability) -> bool {
        self.0 & (1 << capability as u8) != 0
    }

    pub const fn restricted_to(self, requested: Self) -> Self {
        Self(self.0 & requested.0)
    }

    pub const fn with(self, capability: TypingCapability) -> Self {
        Self(self.0 | (1 << capability as u8))
    }

    pub fn from_capabilities<I: IntoIterator<Item = TypingCapability>>(caps: I) -> Self {
        let mut set = Self::empty();
        for cap in caps {
            set = set.with(cap);
        }
        set
    }

    pub fn iter(self) -> impl Iterator<Item = TypingCapability> {
        TypingCapability::ALL.into_iter().filter(move |capability| self.contains(*capability))
    }
}
