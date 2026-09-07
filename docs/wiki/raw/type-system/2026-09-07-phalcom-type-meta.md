# Phalcom Type Meta source snapshot

> Source: Local repository snapshot at `phalcom-type-meta/`
> Collected: 2026-09-07
> Published: Unknown

## `phalcom-type-meta/Cargo.toml`

```toml
[package]
name = "phalcom-type-meta"
version = "0.1.0"
edition = "2024"

[dependencies]
#phalcom-common = { path = "../phalcom-common" }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
```

## `phalcom-type-meta/src/bundle.rs`

```rust
//! Module roots, runtime type roots, bundle structure, and occurrences.

use crate::declaration::{
    AliasRecordId, CallableRecordId, CallableSemanticRecord, DeclarationRecordId, DeclarationTypeRecord, FieldRecordId, FieldSemanticRecord, TypeAliasRecord,
};
use crate::fingerprint::Fingerprint128;
use crate::generic::{GenericSignatureRecord, TypeParameterRecord};
use crate::header::{FeatureSectionId, SemanticMetadataHeader};
use crate::identity::{SourceSpanRef, StableDeclarationRef, StableModuleRef};
use crate::kind::KindNodeEntry;
use crate::scoped_type::ScopedTypeNodeEntry;
use crate::type_node::{TypeNodeEntry, TypeNodeId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct ModuleMetadataRoot {
    pub module: StableModuleRef,
    pub declarations: Box<[DeclarationRecordId]>,
    pub aliases: Box<[AliasRecordId]>,
    pub callables: Box<[CallableRecordId]>,
    pub fields: Box<[FieldRecordId]>,
    pub interface_fingerprint: Fingerprint128,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct RuntimeTypeFormKey(pub Box<str>);

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct RuntimeTypeFormRoot {
    pub module: StableModuleRef,
    pub local_key: RuntimeTypeFormKey,
    pub form: TypeNodeId,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct MetadataExtensionSection {
    pub feature: FeatureSectionId,
    pub schema_version: u32,
    pub required: bool,
    pub semantic_fingerprint: Fingerprint128,
    pub payload: Box<[u8]>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum TypeUseRoleRef {
    Parameter,
    Return,
    Field,
    Superclass,
    TypeArgument,
    TypeConstant,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum TypeUseStatusRef {
    Known(TypeNodeId),
    InternalClassObject(StableDeclarationRef),
    Dynamic(crate::declaration::DynamicReasonRef),
    Missing,
    Unknown(crate::declaration::UnknownReasonRef),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct TypeUseRecord {
    pub role: TypeUseRoleRef,
    pub status: TypeUseStatusRef,
    pub written: Option<Box<str>>,
    pub source: Option<SourceSpanRef>,
}

/// The complete immutable metadata bundle for a compiled artifact / program.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct SemanticMetadataBundle {
    pub header: SemanticMetadataHeader,
    pub kinds: Box<[KindNodeEntry]>,
    pub types: Box<[TypeNodeEntry]>,
    pub scoped_types: Box<[ScopedTypeNodeEntry]>,
    pub parameters: Box<[TypeParameterRecord]>,
    pub generic_signatures: Box<[GenericSignatureRecord]>,
    pub declarations: Box<[DeclarationTypeRecord]>,
    pub aliases: Box<[TypeAliasRecord]>,
    pub callables: Box<[CallableSemanticRecord]>,
    pub fields: Box<[FieldSemanticRecord]>,
    pub module_roots: Box<[ModuleMetadataRoot]>,
    pub runtime_roots: Box<[RuntimeTypeFormRoot]>,
    pub occurrences: Box<[TypeUseRecord]>,
    pub extensions: Box<[MetadataExtensionSection]>,
}
```

## `phalcom-type-meta/src/declaration.rs`

```rust
//! Declaration records, superclasses, aliases, callables, and fields.

use crate::generic::GenericSignatureRecordId;
use crate::identity::{SourceSpanRef, StableCallableRef, StableDeclarationRef, StableFieldRef};
use crate::kind::KindNodeId;
use crate::type_node::TypeNodeId;
use serde::{Deserialize, Serialize};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct DeclarationRecordId(pub u32);

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct DeclarationTypeFlags {
    pub is_abstract: bool,
    pub is_final: bool,
    pub is_trait: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct DeclarationTypeRecord {
    pub declaration: StableDeclarationRef,
    pub form: TypeNodeId,
    pub kind: KindNodeId,
    pub generic_signature: Option<GenericSignatureRecordId>,
    pub superclass_template: Option<TypeNodeId>,
    pub instance_callables: Box<[StableCallableRef]>,
    pub class_callables: Box<[StableCallableRef]>,
    pub instance_fields: Box<[StableFieldRef]>,
    pub class_fields: Box<[StableFieldRef]>,
    pub flags: DeclarationTypeFlags,
    pub source: Option<SourceSpanRef>,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct AliasRecordId(pub u32);

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct TypeAliasRecord {
    pub declaration: StableDeclarationRef,
    pub generic_signature: Option<GenericSignatureRecordId>,
    pub target: TypeNodeId,
    pub source: Option<SourceSpanRef>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum PublishedTypeAuthority {
    DeclaredAnnotation,
    TrustedNative,
    GeneratedDeclaration,
    CompilerInferred,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum DynamicReasonRef {
    ExplicitEscape,
    UncheckedBoundary,
    UnsupportedNative,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum UnknownReasonRef {
    UnannotatedDeclaration,
    InferenceFailed,
    OpaqueNative,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum MetadataUnavailableReason {
    StrippedByProfile,
    UnloadedModule,
    IncompatibleModel,
    DynamicReplacement,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum PublishedTypeSlot {
    Known { form: TypeNodeId, authority: PublishedTypeAuthority },
    Dynamic { reason: DynamicReasonRef },
    Unknown { reason: UnknownReasonRef },
    Unavailable { reason: MetadataUnavailableReason },
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct CallableRecordId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum RestModeRef {
    None,
    Anonymous,
    Named,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct CallableParameterRecord {
    pub index: u32,
    pub local_name: Box<str>,
    pub external_label: Option<Box<str>>,
    pub rest: RestModeRef,
    pub ty: PublishedTypeSlot,
    pub source: Option<SourceSpanRef>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct CallableSemanticRecord {
    pub callable: StableCallableRef,
    pub generic_signature: Option<GenericSignatureRecordId>,
    pub parameters: Box<[CallableParameterRecord]>,
    pub return_type: PublishedTypeSlot,
    pub source: Option<SourceSpanRef>,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct FieldRecordId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum FieldMutabilityRef {
    Immutable,
    Mutable,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct FieldSemanticRecord {
    pub field: StableFieldRef,
    pub mutability: FieldMutabilityRef,
    pub ty: PublishedTypeSlot,
    pub source: Option<SourceSpanRef>,
}
```

## `phalcom-type-meta/src/encode.rs`

```rust
//! Deterministic JSON / binary serialization wrappers.

use crate::bundle::SemanticMetadataBundle;
use crate::validate::{MetadataValidationError, ValidationLimits, validate_metadata_bundle};

/// Encodes a metadata bundle to a JSON string.
pub fn encode_metadata_json(bundle: &SemanticMetadataBundle) -> Result<String, serde_json::Error> {
    serde_json::to_string(bundle)
}

/// Decodes and validates a metadata bundle from a JSON string.
pub fn decode_metadata_json(json: &str, limits: &ValidationLimits) -> Result<SemanticMetadataBundle, MetadataDecodeError> {
    if json.len() > limits.max_total_bytes {
        return Err(MetadataDecodeError::Validation(MetadataValidationError::BudgetExceeded {
            resource: "bytes",
            count: json.len(),
            limit: limits.max_total_bytes,
        }));
    }
    let bundle: SemanticMetadataBundle = serde_json::from_str(json).map_err(MetadataDecodeError::Json)?;
    validate_metadata_bundle(&bundle, limits).map_err(MetadataDecodeError::Validation)?;
    Ok(bundle)
}

#[derive(Debug, thiserror::Error)]
pub enum MetadataDecodeError {
    #[error("json decode error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("validation error: {0}")]
    Validation(#[from] MetadataValidationError),
}
```

## `phalcom-type-meta/src/fingerprint.rs`

```rust
//! 128-bit structural fingerprint implementation for deterministic hashing.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Deterministic 128-bit hash used for structural type/signature equivalence and cache validation.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Fingerprint128(pub [u8; 16]);

impl Fingerprint128 {
    pub const ZERO: Self = Self([0u8; 16]);

    pub fn from_u128(val: u128) -> Self {
        Self(val.to_be_bytes())
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl fmt::Debug for Fingerprint128 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fingerprint128(")?;
        for b in &self.0 {
            write!(f, "{:02x}", b)?;
        }
        write!(f, ")")
    }
}

impl fmt::Display for Fingerprint128 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in &self.0 {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

/// Simple deterministic streaming hasher producing a 128-bit fingerprint.
#[derive(Clone, Debug)]
pub struct FingerprintBuilder {
    state: u128,
}

impl Default for FingerprintBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FingerprintBuilder {
    pub fn new() -> Self {
        Self {
            // FNV-1a 128-bit offset basis
            state: 0x6c62272e07bb014262b821756295c58d,
        }
    }

    pub fn write_u8(&mut self, b: u8) {
        self.state ^= b as u128;
        self.state = self.state.wrapping_mul(0x1000000000000000000013b);
    }

    pub fn write_u32(&mut self, val: u32) {
        for b in val.to_be_bytes() {
            self.write_u8(b);
        }
    }

    pub fn write_u64(&mut self, val: u64) {
        for b in val.to_be_bytes() {
            self.write_u8(b);
        }
    }

    pub fn write_str(&mut self, s: &str) {
        self.write_u32(s.len() as u32);
        for b in s.as_bytes() {
            self.write_u8(*b);
        }
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.write_u32(bytes.len() as u32);
        for &b in bytes {
            self.write_u8(b);
        }
    }

    pub fn write_fingerprint(&mut self, fp: Fingerprint128) {
        for b in fp.0 {
            self.write_u8(b);
        }
    }

    pub fn finish(self) -> Fingerprint128 {
        Fingerprint128(self.state.to_be_bytes())
    }
}
```

## `phalcom-type-meta/src/generic.rs`

```rust
//! Canonical generic parameters, variance, and signature-owned constraints.

use crate::identity::{SourceSpanRef, StableCallableRef, StableDeclarationRef};
use crate::kind::KindNodeId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum StableTypeParameterOwnerRef {
    Declaration(StableDeclarationRef),
    Callable(StableCallableRef),
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct StableTypeParameterRef {
    pub owner: StableTypeParameterOwnerRef,
    pub index: u32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum VarianceRef {
    Covariant,
    Contravariant,
    Invariant,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct TypeParameterRecord {
    pub id: StableTypeParameterRef,
    pub name: Box<str>,
    pub kind: KindNodeId,
    pub variance: VarianceRef,
    pub source: Option<SourceSpanRef>,
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct GenericSignatureRecordId(pub u32);

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct GenericSignatureRecord {
    pub owner: StableTypeParameterOwnerRef,
    pub parameters: Box<[StableTypeParameterRef]>,
    pub constraints: Box<[GenericConstraintRef]>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum GenericConstraintRef {
    Subtype {
        lower: crate::type_node::TypeNodeId,
        upper: crate::type_node::TypeNodeId,
    },
    Equivalent {
        left: crate::type_node::TypeNodeId,
        right: crate::type_node::TypeNodeId,
    },
}
```

## `phalcom-type-meta/src/header.rs`

```rust
//! Metadata header, versions, profile, and features.

use crate::fingerprint::Fingerprint128;
use serde::{Deserialize, Serialize};

pub const MIN_SUPPORTED_TYPE_METADATA_SCHEMA_VERSION: u32 = 1;
pub const TYPE_METADATA_SCHEMA_VERSION: u32 = 2;
pub const SEMANTIC_MODEL_VERSION: u32 = 1;
pub const NATIVE_SURFACE_SCHEMA_VERSION: u32 = 1;

#[inline]
pub const fn supports_type_metadata_schema(version: u32) -> bool {
    version >= MIN_SUPPORTED_TYPE_METADATA_SCHEMA_VERSION && version <= TYPE_METADATA_SCHEMA_VERSION
}

/// Metadata retention profile.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum MetadataProfile {
    RuntimeMinimal,
    RuntimePublic,
    ToolingDebug,
    Proof,
}

/// Feature section identifier.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct FeatureSectionId(pub Box<str>);

/// Enabled features in this metadata artifact.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct MetadataFeatures {
    pub type_lambdas: bool,
    pub record_rows: bool,
    pub runtime_type_constants: bool,
    pub source_occurrences: bool,
    pub advanced_sections: Box<[FeatureSectionId]>,
}

/// Producer identity (e.g. phalcomc).
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ProducerIdentity(pub Box<str>);

/// Artifact identity scheme.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum ArtifactIdentityScheme {
    V1Standard,
    SessionLocal,
}

/// Root metadata header.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct SemanticMetadataHeader {
    pub schema_version: u32,
    pub semantic_model_version: u32,
    pub producer: ProducerIdentity,
    pub producer_version: Box<str>,
    pub native_surface_schema_version: u32,
    pub profile: MetadataProfile,
    pub features: MetadataFeatures,
    pub identity_scheme: ArtifactIdentityScheme,
    pub source_fingerprint: Fingerprint128,
    pub interface_fingerprint: Fingerprint128,
}
```

## `phalcom-type-meta/src/identity.rs`

```rust
//! Stable artifact/project/module/declaration/member identities.

use crate::fingerprint::Fingerprint128;
use serde::{Deserialize, Serialize};

/// Stable project reference.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum StableProjectRef {
    Builtin {
        namespace: Box<str>,
        version: Box<str>,
    },
    Package {
        package: Box<str>,
        version: Box<str>,
        artifact_fingerprint: Fingerprint128,
    },
    SourceArtifact {
        logical_uri: Box<str>,
        source_fingerprint: Fingerprint128,
    },
    Session {
        session_fingerprint: Fingerprint128,
    },
}

/// Stable module reference within a project.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct StableModuleRef {
    pub project: StableProjectRef,
    pub path: Box<[Box<str>]>,
}

/// Stable declaration reference.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct StableDeclarationRef {
    pub module: StableModuleRef,
    pub path: Box<[Box<str>]>,
}

/// Stable dispatch side (instance or class).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum StableDispatchSide {
    Instance,
    Class,
}

/// Stable callable reference.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct StableCallableRef {
    pub owner: StableDeclarationRef,
    pub side: StableDispatchSide,
    pub selector: Box<str>,
}

/// Stable field reference.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct StableFieldRef {
    pub owner: StableDeclarationRef,
    pub side: StableDispatchSide,
    pub name: Box<str>,
}

/// Stable source span reference.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct SourceSpanRef {
    pub start: u32,
    pub end: u32,
}
```

## `phalcom-type-meta/src/kind.rs`

```rust
//! Indexed kind graph.

use crate::fingerprint::Fingerprint128;
use serde::{Deserialize, Serialize};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct KindNodeId(pub u32);

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum KindNode {
    Type,
    RecordRow,
    Arrow { parameters: Box<[KindNodeId]>, result: KindNodeId },
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct KindNodeEntry {
    pub node: KindNode,
    pub structural_fingerprint: Fingerprint128,
}
```

## `phalcom-type-meta/src/lib.rs`

```rust
//! Standalone, store-independent, versioned semantic metadata models, indexed graphs, and artifact schema.

pub mod bundle;
pub mod declaration;
pub mod encode;
pub mod fingerprint;
pub mod generic;
pub mod header;
pub mod identity;
pub mod kind;
pub mod scoped_type;
pub mod type_node;
pub mod validate;

pub use bundle::*;
pub use declaration::*;
pub use encode::*;
pub use fingerprint::*;
pub use generic::*;
pub use header::*;
pub use identity::*;
pub use kind::*;
pub use scoped_type::*;
pub use type_node::*;
pub use validate::*;
```

## `phalcom-type-meta/src/scoped_type.rs`

```rust
//! Scoped node graph for alpha-normalized type lambda bodies.

use crate::fingerprint::Fingerprint128;
use crate::kind::KindNodeId;
use crate::type_node::TypeNodeId;
use serde::{Deserialize, Serialize};

use crate::generic::StableTypeParameterRef;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct ScopedTypeNodeId(pub u32);

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ScopedTupleElementRef {
    pub label: Option<Box<str>>,
    pub ty: ScopedTypeNodeId,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ScopedRecordFieldRef {
    pub name: Box<str>,
    pub ty: ScopedTypeNodeId,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum ScopedRecordTailRef {
    Bound { depth: u32, index: u32 },
    FreeParameter(StableTypeParameterRef),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ScopedOpenRecordTypeRef {
    pub fields: Box<[ScopedRecordFieldRef]>,
    pub tail: ScopedRecordTailRef,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ScopedCallableParamRef {
    pub label: Option<Box<str>>,
    pub ty: ScopedTypeNodeId,
    pub rest: bool,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct ScopedCallableTypeRef {
    pub parameters: Box<[ScopedCallableParamRef]>,
    pub return_type: ScopedTypeNodeId,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum ScopedTypeNode {
    /// Lambda-bound variable; depth 0 is the innermost binder.
    Bound {
        depth: u32,
        index: u32,
    },
    /// Canonical free term from the enclosing global type graph.
    Free(TypeNodeId),
    Applied {
        origin: ScopedTypeNodeId,
        arguments: Box<[ScopedTypeNodeId]>,
    },
    Union(Box<[ScopedTypeNodeId]>),
    Tuple(Box<[ScopedTupleElementRef]>),
    Record(Box<[ScopedRecordFieldRef]>),
    Callable(ScopedCallableTypeRef),
    /// Nested lambda.
    Lambda {
        parameter_kinds: Box<[KindNodeId]>,
        body: ScopedTypeNodeId,
    },
    OpenRecord(ScopedOpenRecordTypeRef),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct ScopedTypeNodeEntry {
    pub kind: KindNodeId,
    pub form: ScopedTypeNode,
    pub structural_fingerprint: Fingerprint128,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct TypeLambdaRef {
    pub parameter_kinds: Box<[KindNodeId]>,
    pub body: ScopedTypeNodeId,
}
```

## `phalcom-type-meta/src/type_node.rs`

```rust
//! Canonical global type-term graph.

use crate::fingerprint::Fingerprint128;
use crate::generic::StableTypeParameterRef;
use crate::identity::{StableDeclarationRef, StableDispatchSide};
use crate::kind::KindNodeId;
use crate::scoped_type::TypeLambdaRef;
use serde::{Deserialize, Serialize};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct TypeNodeId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum SelfRoleRef {
    InstanceType,
    ReceiverValue,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct SelfTypeRef {
    pub owner: StableDeclarationRef,
    pub side: StableDispatchSide,
    pub role: SelfRoleRef,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct TupleElementRef {
    pub label: Option<Box<str>>,
    pub ty: TypeNodeId,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct RecordFieldRef {
    pub name: Box<str>,
    pub ty: TypeNodeId,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct CallableParamRef {
    pub label: Option<Box<str>>,
    pub ty: TypeNodeId,
    pub rest: bool,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct CallableTypeRef {
    pub parameters: Box<[CallableParamRef]>,
    pub return_type: TypeNodeId,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub struct OpenRecordTypeRef {
    pub fields: Box<[RecordFieldRef]>,
    pub tail: StableTypeParameterRef,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum TypeNode {
    Never,
    Unit,
    Nominal { declaration: StableDeclarationRef },
    Applied { origin: TypeNodeId, arguments: Box<[TypeNodeId]> },
    Union(Box<[TypeNodeId]>),
    Tuple(Box<[TupleElementRef]>),
    Record(Box<[RecordFieldRef]>),
    Callable(CallableTypeRef),
    Parameter(StableTypeParameterRef),
    SelfType(SelfTypeRef),
    TypeLambda(TypeLambdaRef),
    OpenRecord(OpenRecordTypeRef),
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct TypeNodeEntry {
    pub kind: KindNodeId,
    pub form: TypeNode,
    pub structural_fingerprint: Fingerprint128,
}
```

## `phalcom-type-meta/src/validate.rs`

```rust
use crate::bundle::SemanticMetadataBundle;
use crate::declaration::PublishedTypeSlot;
use crate::header::{MIN_SUPPORTED_TYPE_METADATA_SCHEMA_VERSION, SEMANTIC_MODEL_VERSION, TYPE_METADATA_SCHEMA_VERSION, supports_type_metadata_schema};
use crate::kind::{KindNode, KindNodeId};
use crate::scoped_type::{ScopedRecordTailRef, ScopedTypeNode};
use crate::type_node::TypeNode;
use std::collections::HashSet;
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationLimits {
    pub max_total_bytes: usize,
    pub max_kind_nodes: usize,
    pub max_type_nodes: usize,
    pub max_scoped_nodes: usize,
    pub max_parameters: usize,
    pub max_signatures: usize,
    pub max_declarations: usize,
    pub max_callables: usize,
    pub max_fields: usize,
    pub max_occurrences: usize,
    pub max_lambda_depth: u32,
}

impl Default for ValidationLimits {
    fn default() -> Self {
        Self {
            max_total_bytes: 32 * 1024 * 1024,
            max_kind_nodes: 65536,
            max_type_nodes: 262144,
            max_scoped_nodes: 131072,
            max_parameters: 65536,
            max_signatures: 65536,
            max_declarations: 65536,
            max_callables: 262144,
            max_fields: 262144,
            max_occurrences: 524288,
            max_lambda_depth: 32,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum MetadataValidationError {
    #[error("unsupported schema version: found {found}, supported range [{minimum}..={maximum}]")]
    UnsupportedSchemaVersion { found: u32, minimum: u32, maximum: u32 },
    #[error("unsupported semantic model version: {0} (expected {SEMANTIC_MODEL_VERSION})")]
    UnsupportedSemanticModelVersion(u32),
    #[error("record rows feature required for record row kinds and open records")]
    RecordRowFeatureRequired,
    #[error("record tail parameter missing: owner {owner:?}, index {index}")]
    RecordTailParameterMissing { owner: String, index: u32 },
    #[error("record tail kind mismatch: expected RecordRow, found kind index {actual_kind}")]
    RecordTailKindMismatch { actual_kind: u32 },
    #[error("record field order invalid at node {node}: fields must be sorted")]
    RecordFieldOrderInvalid { node: u32 },
    #[error("duplicate record field '{field}' at node {node}")]
    RecordDuplicateField { node: u32, field: Box<str> },
    #[error("scoped record tail out of scope at node {node}: depth {depth} index {index}")]
    ScopedRecordTailOutOfScope { node: u32, depth: u32, index: u32 },
    #[error("scoped record tail kind mismatch at node {node}: depth {depth} index {index} is not RecordRow")]
    ScopedRecordTailKindMismatch { node: u32, depth: u32, index: u32 },
    #[error("budget exceeded: {resource} count {count} exceeds limit {limit}")]
    BudgetExceeded { resource: &'static str, count: usize, limit: usize },
    #[error("invalid kind index: {index} (max {total})")]
    InvalidKindIndex { index: u32, total: usize },
    #[error("invalid type index: {index} (max {total})")]
    InvalidTypeIndex { index: u32, total: usize },
    #[error("invalid scoped type index: {index} (max {total})")]
    InvalidScopedTypeIndex { index: u32, total: usize },
    #[error("invalid generic signature index: {index} (max {total})")]
    InvalidSignatureIndex { index: u32, total: usize },
    #[error("topological order violation: node {index} references future node {target}")]
    TopologicalOrderViolation { index: u32, target: u32 },
    #[error("lambda scope violation: bound variable at depth {depth} index {index} exceeds max depth")]
    LambdaScopeViolation { depth: u32, index: u32 },
    #[error("duplicate type parameter owner: {0:?} index {1}")]
    DuplicateParameterOwner(String, u32),
    #[error("malformed metadata: {0}")]
    Malformed(String),
}

fn validate_schema_v1_feature_floor(bundle: &SemanticMetadataBundle) -> Result<(), MetadataValidationError> {
    if bundle.header.features.record_rows {
        return Err(MetadataValidationError::Malformed("schema v1 cannot enable record_rows feature".to_string()));
    }
    for entry in bundle.kinds.iter() {
        if matches!(entry.node, KindNode::RecordRow) {
            return Err(MetadataValidationError::Malformed("schema v1 cannot contain KindNode::RecordRow".to_string()));
        }
    }
    for entry in bundle.types.iter() {
        if matches!(entry.form, TypeNode::OpenRecord(_)) {
            return Err(MetadataValidationError::Malformed("schema v1 cannot contain TypeNode::OpenRecord".to_string()));
        }
    }
    for entry in bundle.scoped_types.iter() {
        if matches!(entry.form, ScopedTypeNode::OpenRecord(_)) {
            return Err(MetadataValidationError::Malformed(
                "schema v1 cannot contain ScopedTypeNode::OpenRecord".to_string(),
            ));
        }
    }
    Ok(())
}

fn validate_schema_v2_feature_floor(bundle: &SemanticMetadataBundle) -> Result<(), MetadataValidationError> {
    let mut has_row_construct = false;
    for entry in bundle.kinds.iter() {
        if matches!(entry.node, KindNode::RecordRow) {
            has_row_construct = true;
            break;
        }
    }
    if !has_row_construct {
        for entry in bundle.types.iter() {
            if matches!(entry.form, TypeNode::OpenRecord(_)) {
                has_row_construct = true;
                break;
            }
        }
    }
    if !has_row_construct {
        for entry in bundle.scoped_types.iter() {
            if matches!(entry.form, ScopedTypeNode::OpenRecord(_)) {
                has_row_construct = true;
                break;
            }
        }
    }

    if has_row_construct && !bundle.header.features.record_rows {
        return Err(MetadataValidationError::RecordRowFeatureRequired);
    }

    Ok(())
}

/// Iteratively validates a [`SemanticMetadataBundle`].
pub fn validate_metadata_bundle(bundle: &SemanticMetadataBundle, limits: &ValidationLimits) -> Result<(), MetadataValidationError> {
    if !supports_type_metadata_schema(bundle.header.schema_version) {
        return Err(MetadataValidationError::UnsupportedSchemaVersion {
            found: bundle.header.schema_version,
            minimum: MIN_SUPPORTED_TYPE_METADATA_SCHEMA_VERSION,
            maximum: TYPE_METADATA_SCHEMA_VERSION,
        });
    }
    if bundle.header.semantic_model_version != SEMANTIC_MODEL_VERSION {
        return Err(MetadataValidationError::UnsupportedSemanticModelVersion(bundle.header.semantic_model_version));
    }

    match bundle.header.schema_version {
        1 => validate_schema_v1_feature_floor(bundle)?,
        2 => validate_schema_v2_feature_floor(bundle)?,
        _ => unreachable!("version range checked above"),
    }

    if bundle.kinds.len() > limits.max_kind_nodes {
        return Err(MetadataValidationError::BudgetExceeded {
            resource: "kinds",
            count: bundle.kinds.len(),
            limit: limits.max_kind_nodes,
        });
    }
    if bundle.types.len() > limits.max_type_nodes {
        return Err(MetadataValidationError::BudgetExceeded {
            resource: "types",
            count: bundle.types.len(),
            limit: limits.max_type_nodes,
        });
    }
    if bundle.scoped_types.len() > limits.max_scoped_nodes {
        return Err(MetadataValidationError::BudgetExceeded {
            resource: "scoped_types",
            count: bundle.scoped_types.len(),
            limit: limits.max_scoped_nodes,
        });
    }

    // Validate kinds graph (strictly topologically sorted)
    for (i, entry) in bundle.kinds.iter().enumerate() {
        match &entry.node {
            KindNode::Type | KindNode::RecordRow => {}
            KindNode::Arrow { parameters, result } => {
                for &p in parameters.iter() {
                    if p.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation { index: i as u32, target: p.0 });
                    }
                }
                if result.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: result.0,
                    });
                }
            }
        }
    }

    // Index parameters by (owner_key, index) -> param_entry for fast tail lookup
    let mut param_map = std::collections::HashMap::new();
    let mut param_set = HashSet::new();
    for param in bundle.parameters.iter() {
        if param.kind.0 as usize >= bundle.kinds.len() {
            return Err(MetadataValidationError::InvalidKindIndex {
                index: param.kind.0,
                total: bundle.kinds.len(),
            });
        }
        let owner_key = format!("{:?}", param.id.owner);
        if !param_set.insert((owner_key.clone(), param.id.index)) {
            return Err(MetadataValidationError::DuplicateParameterOwner(owner_key.clone(), param.id.index));
        }
        param_map.insert((owner_key, param.id.index), param);
    }

    // Validate global types graph (strictly topologically sorted)
    for (i, entry) in bundle.types.iter().enumerate() {
        if entry.kind.0 as usize >= bundle.kinds.len() {
            return Err(MetadataValidationError::InvalidKindIndex {
                index: entry.kind.0,
                total: bundle.kinds.len(),
            });
        }
        match &entry.form {
            TypeNode::Never | TypeNode::Unit | TypeNode::Nominal { .. } | TypeNode::Parameter(_) | TypeNode::SelfType(_) => {}
            TypeNode::Applied { origin, arguments } => {
                if origin.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: origin.0,
                    });
                }
                for &arg in arguments.iter() {
                    if arg.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: arg.0,
                        });
                    }
                }
            }
            TypeNode::Union(members) => {
                for &m in members.iter() {
                    if m.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation { index: i as u32, target: m.0 });
                    }
                }
            }
            TypeNode::Tuple(elements) => {
                for el in elements.iter() {
                    if el.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: el.ty.0,
                        });
                    }
                }
            }
            TypeNode::Record(fields) => {
                let mut prev_name: Option<&str> = None;
                for f in fields.iter() {
                    if f.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: f.ty.0,
                        });
                    }
                    if let Some(prev) = prev_name {
                        if &*f.name < prev {
                            return Err(MetadataValidationError::RecordFieldOrderInvalid { node: i as u32 });
                        } else if &*f.name == prev {
                            return Err(MetadataValidationError::RecordDuplicateField {
                                node: i as u32,
                                field: f.name.clone(),
                            });
                        }
                    }
                    prev_name = Some(&f.name);
                }
            }
            TypeNode::OpenRecord(open_rec) => {
                let mut prev_name: Option<&str> = None;
                for f in open_rec.fields.iter() {
                    if f.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: f.ty.0,
                        });
                    }
                    if let Some(prev) = prev_name {
                        if &*f.name < prev {
                            return Err(MetadataValidationError::RecordFieldOrderInvalid { node: i as u32 });
                        } else if &*f.name == prev {
                            return Err(MetadataValidationError::RecordDuplicateField {
                                node: i as u32,
                                field: f.name.clone(),
                            });
                        }
                    }
                    prev_name = Some(&f.name);
                }
                let owner_key = format!("{:?}", open_rec.tail.owner);
                let tail_param = param_map
                    .get(&(owner_key.clone(), open_rec.tail.index))
                    .ok_or(MetadataValidationError::RecordTailParameterMissing {
                        owner: owner_key,
                        index: open_rec.tail.index,
                    })?;
                let tail_kind_entry = &bundle.kinds[tail_param.kind.0 as usize];
                if !matches!(tail_kind_entry.node, KindNode::RecordRow) {
                    return Err(MetadataValidationError::RecordTailKindMismatch {
                        actual_kind: tail_param.kind.0,
                    });
                }
            }
            TypeNode::Callable(call) => {
                for p in call.parameters.iter() {
                    if p.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: p.ty.0,
                        });
                    }
                }
                if call.return_type.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: call.return_type.0,
                    });
                }
            }
            TypeNode::TypeLambda(lambda) => {
                for &pk in lambda.parameter_kinds.iter() {
                    if pk.0 as usize >= bundle.kinds.len() {
                        return Err(MetadataValidationError::InvalidKindIndex {
                            index: pk.0,
                            total: bundle.kinds.len(),
                        });
                    }
                }
                if lambda.body.0 as usize >= bundle.scoped_types.len() {
                    return Err(MetadataValidationError::InvalidScopedTypeIndex {
                        index: lambda.body.0,
                        total: bundle.scoped_types.len(),
                    });
                }
            }
        }
    }

    // Validate scoped lambda graph (strictly topologically sorted)
    // We maintain a stack/mapping of lambda scope parameter kinds
    let mut lambda_param_kinds: Vec<Box<[KindNodeId]>> = Vec::new();
    for (i, entry) in bundle.scoped_types.iter().enumerate() {
        if entry.kind.0 as usize >= bundle.kinds.len() {
            return Err(MetadataValidationError::InvalidKindIndex {
                index: entry.kind.0,
                total: bundle.kinds.len(),
            });
        }
        match &entry.form {
            ScopedTypeNode::Bound { depth, index } => {
                if *depth > limits.max_lambda_depth {
                    return Err(MetadataValidationError::LambdaScopeViolation { depth: *depth, index: *index });
                }
            }
            ScopedTypeNode::Free(t) => {
                if t.0 as usize >= bundle.types.len() {
                    return Err(MetadataValidationError::InvalidTypeIndex {
                        index: t.0,
                        total: bundle.types.len(),
                    });
                }
            }
            ScopedTypeNode::Applied { origin, arguments } => {
                if origin.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: origin.0,
                    });
                }
                for &arg in arguments.iter() {
                    if arg.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: arg.0,
                        });
                    }
                }
            }
            ScopedTypeNode::Union(members) => {
                for &m in members.iter() {
                    if m.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation { index: i as u32, target: m.0 });
                    }
                }
            }
            ScopedTypeNode::Tuple(elements) => {
                for el in elements.iter() {
                    if el.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: el.ty.0,
                        });
                    }
                }
            }
            ScopedTypeNode::Record(fields) => {
                let mut prev_name: Option<&str> = None;
                for f in fields.iter() {
                    if f.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: f.ty.0,
                        });
                    }
                    if let Some(prev) = prev_name {
                        if &*f.name < prev {
                            return Err(MetadataValidationError::RecordFieldOrderInvalid { node: i as u32 });
                        } else if &*f.name == prev {
                            return Err(MetadataValidationError::RecordDuplicateField {
                                node: i as u32,
                                field: f.name.clone(),
                            });
                        }
                    }
                    prev_name = Some(&f.name);
                }
            }
            ScopedTypeNode::OpenRecord(open_rec) => {
                let mut prev_name: Option<&str> = None;
                for f in open_rec.fields.iter() {
                    if f.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: f.ty.0,
                        });
                    }
                    if let Some(prev) = prev_name {
                        if &*f.name < prev {
                            return Err(MetadataValidationError::RecordFieldOrderInvalid { node: i as u32 });
                        } else if &*f.name == prev {
                            return Err(MetadataValidationError::RecordDuplicateField {
                                node: i as u32,
                                field: f.name.clone(),
                            });
                        }
                    }
                    prev_name = Some(&f.name);
                }
                match &open_rec.tail {
                    ScopedRecordTailRef::Bound { depth, index } => {
                        if *depth as usize >= lambda_param_kinds.len() {
                            // Depth exceeds known lambda scopes at this node
                            // (Conservative fallback: still check depth limit)
                            if *depth > limits.max_lambda_depth {
                                return Err(MetadataValidationError::LambdaScopeViolation { depth: *depth, index: *index });
                            }
                        } else {
                            let kinds = &lambda_param_kinds[lambda_param_kinds.len() - 1 - *depth as usize];
                            if *index as usize >= kinds.len() {
                                return Err(MetadataValidationError::ScopedRecordTailOutOfScope {
                                    node: i as u32,
                                    depth: *depth,
                                    index: *index,
                                });
                            }
                            let k_id = kinds[*index as usize];
                            if k_id.0 as usize >= bundle.kinds.len() || !matches!(bundle.kinds[k_id.0 as usize].node, KindNode::RecordRow) {
                                return Err(MetadataValidationError::ScopedRecordTailKindMismatch {
                                    node: i as u32,
                                    depth: *depth,
                                    index: *index,
                                });
                            }
                        }
                    }
                    ScopedRecordTailRef::FreeParameter(param_ref) => {
                        let owner_key = format!("{:?}", param_ref.owner);
                        let tail_param = param_map
                            .get(&(owner_key.clone(), param_ref.index))
                            .ok_or(MetadataValidationError::RecordTailParameterMissing {
                                owner: owner_key,
                                index: param_ref.index,
                            })?;
                        let tail_kind_entry = &bundle.kinds[tail_param.kind.0 as usize];
                        if !matches!(tail_kind_entry.node, KindNode::RecordRow) {
                            return Err(MetadataValidationError::RecordTailKindMismatch {
                                actual_kind: tail_param.kind.0,
                            });
                        }
                    }
                }
            }
            ScopedTypeNode::Callable(call) => {
                for p in call.parameters.iter() {
                    if p.ty.0 as usize >= i {
                        return Err(MetadataValidationError::TopologicalOrderViolation {
                            index: i as u32,
                            target: p.ty.0,
                        });
                    }
                }
                if call.return_type.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: call.return_type.0,
                    });
                }
            }
            ScopedTypeNode::Lambda { parameter_kinds, body } => {
                for &pk in parameter_kinds.iter() {
                    if pk.0 as usize >= bundle.kinds.len() {
                        return Err(MetadataValidationError::InvalidKindIndex {
                            index: pk.0,
                            total: bundle.kinds.len(),
                        });
                    }
                }
                if body.0 as usize >= i {
                    return Err(MetadataValidationError::TopologicalOrderViolation {
                        index: i as u32,
                        target: body.0,
                    });
                }
                lambda_param_kinds.push(parameter_kinds.clone());
            }
        }
    }

    // Validate parameters
    let mut param_set = HashSet::new();
    for param in bundle.parameters.iter() {
        if param.kind.0 as usize >= bundle.kinds.len() {
            return Err(MetadataValidationError::InvalidKindIndex {
                index: param.kind.0,
                total: bundle.kinds.len(),
            });
        }
        if !param_set.insert((format!("{:?}", param.id.owner), param.id.index)) {
            return Err(MetadataValidationError::DuplicateParameterOwner(
                format!("{:?}", param.id.owner),
                param.id.index,
            ));
        }
    }

    // Validate generic signatures & constraints
    for sig in bundle.generic_signatures.iter() {
        for c in sig.constraints.iter() {
            match c {
                crate::generic::GenericConstraintRef::Subtype { lower, upper } => {
                    if lower.0 as usize >= bundle.types.len() {
                        return Err(MetadataValidationError::InvalidTypeIndex {
                            index: lower.0,
                            total: bundle.types.len(),
                        });
                    }
                    if upper.0 as usize >= bundle.types.len() {
                        return Err(MetadataValidationError::InvalidTypeIndex {
                            index: upper.0,
                            total: bundle.types.len(),
                        });
                    }
                }
                crate::generic::GenericConstraintRef::Equivalent { left, right } => {
                    if left.0 as usize >= bundle.types.len() {
                        return Err(MetadataValidationError::InvalidTypeIndex {
                            index: left.0,
                            total: bundle.types.len(),
                        });
                    }
                    if right.0 as usize >= bundle.types.len() {
                        return Err(MetadataValidationError::InvalidTypeIndex {
                            index: right.0,
                            total: bundle.types.len(),
                        });
                    }
                }
            }
        }
    }

    // Validate declarations
    for decl in bundle.declarations.iter() {
        if decl.form.0 as usize >= bundle.types.len() {
            return Err(MetadataValidationError::InvalidTypeIndex {
                index: decl.form.0,
                total: bundle.types.len(),
            });
        }
        if decl.kind.0 as usize >= bundle.kinds.len() {
            return Err(MetadataValidationError::InvalidKindIndex {
                index: decl.kind.0,
                total: bundle.kinds.len(),
            });
        }
        if let Some(sig_id) = decl.generic_signature {
            if sig_id.0 as usize >= bundle.generic_signatures.len() {
                return Err(MetadataValidationError::InvalidSignatureIndex {
                    index: sig_id.0,
                    total: bundle.generic_signatures.len(),
                });
            }
        }
        if let Some(sup) = decl.superclass_template {
            if sup.0 as usize >= bundle.types.len() {
                return Err(MetadataValidationError::InvalidTypeIndex {
                    index: sup.0,
                    total: bundle.types.len(),
                });
            }
        }
    }

    // Validate callables
    for call in bundle.callables.iter() {
        for p in call.parameters.iter() {
            if let PublishedTypeSlot::Known { form, .. } = p.ty {
                if form.0 as usize >= bundle.types.len() {
                    return Err(MetadataValidationError::InvalidTypeIndex {
                        index: form.0,
                        total: bundle.types.len(),
                    });
                }
            }
        }
        if let PublishedTypeSlot::Known { form, .. } = call.return_type {
            if form.0 as usize >= bundle.types.len() {
                return Err(MetadataValidationError::InvalidTypeIndex {
                    index: form.0,
                    total: bundle.types.len(),
                });
            }
        }
    }

    Ok(())
}
```

## `phalcom-type-meta/tests/fixtures/schema-v1/basic.json`

```json
{
  "header": {
    "schema_version": 1,
    "semantic_model_version": 1,
    "producer": "phalcom-semantic",
    "producer_version": "0.1.0",
    "native_surface_schema_version": 1,
    "profile": "RuntimePublic",
    "features": {
      "type_lambdas": true,
      "record_rows": false,
      "runtime_type_constants": true,
      "source_occurrences": false,
      "advanced_sections": []
    },
    "identity_scheme": "V1Standard",
    "source_fingerprint": [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    "interface_fingerprint": [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]
  },
  "kinds": [
    {
      "node": "Type",
      "structural_fingerprint": [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]
    }
  ],
  "types": [
    {
      "kind": 0,
      "form": {
        "Nominal": {
          "declaration": {
            "module": {
              "project": {
                "Builtin": {
                  "namespace": "std",
                  "version": "0.1.0"
                }
              },
              "path": ["collections", "list"]
            },
            "path": ["Int"]
          }
        }
      },
      "structural_fingerprint": [3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]
    },
    {
      "kind": 0,
      "form": {
        "Record": [
          {
            "name": "age",
            "ty": 0
          }
        ]
      },
      "structural_fingerprint": [7,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]
    }
  ],
  "scoped_types": [],
  "parameters": [],
  "generic_signatures": [],
  "declarations": [],
  "aliases": [],
  "callables": [],
  "fields": [],
  "module_roots": [],
  "runtime_roots": [
    {
      "module": {
        "project": {
          "Builtin": {
            "namespace": "std",
            "version": "0.1.0"
          }
        },
        "path": ["collections", "list"]
      },
      "local_key": "root",
      "form": 1
    }
  ],
  "occurrences": [],
  "extensions": []
}
```

## `phalcom-type-meta/tests/schema_compat.rs`

```rust
use phalcom_type_meta::bundle::SemanticMetadataBundle;
use phalcom_type_meta::encode::decode_metadata_json;
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::generic::{StableTypeParameterOwnerRef, StableTypeParameterRef, TypeParameterRecord, VarianceRef};
use phalcom_type_meta::header::{
    ArtifactIdentityScheme, MIN_SUPPORTED_TYPE_METADATA_SCHEMA_VERSION, MetadataFeatures, MetadataProfile, NATIVE_SURFACE_SCHEMA_VERSION, ProducerIdentity,
    SEMANTIC_MODEL_VERSION, SemanticMetadataHeader, TYPE_METADATA_SCHEMA_VERSION,
};
use phalcom_type_meta::identity::{StableDeclarationRef, StableModuleRef, StableProjectRef};
use phalcom_type_meta::kind::{KindNode, KindNodeEntry, KindNodeId};
use phalcom_type_meta::scoped_type::{ScopedOpenRecordTypeRef, ScopedRecordTailRef, ScopedTypeNode, ScopedTypeNodeEntry, ScopedTypeNodeId};
use phalcom_type_meta::type_node::{OpenRecordTypeRef, RecordFieldRef, TypeNode, TypeNodeEntry, TypeNodeId};
use phalcom_type_meta::validate::{MetadataValidationError, ValidationLimits, validate_metadata_bundle};

fn sample_header(version: u32, record_rows: bool) -> SemanticMetadataHeader {
    SemanticMetadataHeader {
        schema_version: version,
        semantic_model_version: SEMANTIC_MODEL_VERSION,
        producer: ProducerIdentity("phalcom-test".into()),
        producer_version: "0.1.0".into(),
        native_surface_schema_version: NATIVE_SURFACE_SCHEMA_VERSION,
        profile: MetadataProfile::RuntimePublic,
        features: MetadataFeatures {
            type_lambdas: true,
            record_rows,
            runtime_type_constants: false,
            source_occurrences: false,
            advanced_sections: Box::new([]),
        },
        identity_scheme: ArtifactIdentityScheme::V1Standard,
        source_fingerprint: Fingerprint128::ZERO,
        interface_fingerprint: Fingerprint128::ZERO,
    }
}

fn sample_bundle(version: u32, record_rows: bool) -> SemanticMetadataBundle {
    SemanticMetadataBundle {
        header: sample_header(version, record_rows),
        kinds: Box::new([KindNodeEntry {
            node: KindNode::Type,
            structural_fingerprint: Fingerprint128::ZERO,
        }]),
        types: Box::new([TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::Unit,
            structural_fingerprint: Fingerprint128::ZERO,
        }]),
        scoped_types: Box::new([]),
        parameters: Box::new([]),
        generic_signatures: Box::new([]),
        declarations: Box::new([]),
        aliases: Box::new([]),
        callables: Box::new([]),
        fields: Box::new([]),
        module_roots: Box::new([]),
        runtime_roots: Box::new([]),
        occurrences: Box::new([]),
        extensions: Box::new([]),
    }
}

#[test]
fn test_v1_fixture_decodes_and_validates_under_v2_decoder() {
    let fixture_str = include_str!("fixtures/schema-v1/basic.json");
    let limits = ValidationLimits::default();
    let bundle = decode_metadata_json(fixture_str, &limits).expect("failed to decode v1 fixture");
    assert_eq!(bundle.header.schema_version, 1);
    assert_eq!(bundle.types.len(), 2);
    assert!(matches!(bundle.types[1].form, TypeNode::Record(_)));
}

#[test]
fn test_unsupported_schema_versions_rejected() {
    let limits = ValidationLimits::default();

    // Version 0
    let b0 = sample_bundle(0, false);
    let err0 = validate_metadata_bundle(&b0, &limits).unwrap_err();
    assert_eq!(
        err0,
        MetadataValidationError::UnsupportedSchemaVersion {
            found: 0,
            minimum: MIN_SUPPORTED_TYPE_METADATA_SCHEMA_VERSION,
            maximum: TYPE_METADATA_SCHEMA_VERSION,
        }
    );

    // Version 3 (future)
    let b3 = sample_bundle(3, false);
    let err3 = validate_metadata_bundle(&b3, &limits).unwrap_err();
    assert_eq!(
        err3,
        MetadataValidationError::UnsupportedSchemaVersion {
            found: 3,
            minimum: MIN_SUPPORTED_TYPE_METADATA_SCHEMA_VERSION,
            maximum: TYPE_METADATA_SCHEMA_VERSION,
        }
    );
}

#[test]
fn test_schema_v1_cannot_contain_record_row_constructs() {
    let limits = ValidationLimits::default();

    // v1 with record_rows feature bit = true -> error
    let b_feat = sample_bundle(1, true);
    assert!(validate_metadata_bundle(&b_feat, &limits).is_err());

    // v1 with KindNode::RecordRow -> error
    let mut b_kind = sample_bundle(1, false);
    b_kind.kinds = Box::new([KindNodeEntry {
        node: KindNode::RecordRow,
        structural_fingerprint: Fingerprint128::ZERO,
    }]);
    assert!(validate_metadata_bundle(&b_kind, &limits).is_err());
}

#[test]
fn test_schema_v2_open_record_validation() {
    let limits = ValidationLimits::default();
    let mut b = sample_bundle(2, true);

    b.kinds = Box::new([
        KindNodeEntry {
            node: KindNode::Type,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        KindNodeEntry {
            node: KindNode::RecordRow,
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);

    let decl_ref = StableDeclarationRef {
        module: StableModuleRef {
            project: StableProjectRef::Builtin {
                namespace: "test".into(),
                version: "0.1.0".into(),
            },
            path: Box::new(["mod".into()]),
        },
        path: Box::new(["MyType".into()]),
    };

    let param_ref = StableTypeParameterRef {
        owner: StableTypeParameterOwnerRef::Declaration(decl_ref.clone()),
        index: 0,
    };

    b.parameters = Box::new([TypeParameterRecord {
        id: param_ref.clone(),
        name: "R".into(),
        kind: KindNodeId(1), // RecordRow
        variance: VarianceRef::Invariant,
        source: None,
    }]);

    // Valid open record
    b.types = Box::new([
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::Unit,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::OpenRecord(OpenRecordTypeRef {
                fields: Box::new([
                    RecordFieldRef {
                        name: "a".into(),
                        ty: TypeNodeId(0),
                    },
                    RecordFieldRef {
                        name: "b".into(),
                        ty: TypeNodeId(0),
                    },
                ]),
                tail: param_ref.clone(),
            }),
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);

    validate_metadata_bundle(&b, &limits).expect("v2 open record should validate");

    // Negative test: unsorted fields
    let mut b_unsorted = b.clone();
    b_unsorted.types = Box::new([
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::Unit,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::OpenRecord(OpenRecordTypeRef {
                fields: Box::new([
                    RecordFieldRef {
                        name: "z".into(),
                        ty: TypeNodeId(0),
                    },
                    RecordFieldRef {
                        name: "a".into(),
                        ty: TypeNodeId(0),
                    },
                ]),
                tail: param_ref.clone(),
            }),
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);
    assert!(matches!(
        validate_metadata_bundle(&b_unsorted, &limits).unwrap_err(),
        MetadataValidationError::RecordFieldOrderInvalid { .. }
    ));

    // Negative test: duplicate fields
    let mut b_dup = b.clone();
    b_dup.types = Box::new([
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::Unit,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::OpenRecord(OpenRecordTypeRef {
                fields: Box::new([
                    RecordFieldRef {
                        name: "a".into(),
                        ty: TypeNodeId(0),
                    },
                    RecordFieldRef {
                        name: "a".into(),
                        ty: TypeNodeId(0),
                    },
                ]),
                tail: param_ref.clone(),
            }),
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);
    assert!(matches!(
        validate_metadata_bundle(&b_dup, &limits).unwrap_err(),
        MetadataValidationError::RecordDuplicateField { .. }
    ));

    // Negative test: tail parameter is not kind RecordRow (point it to Type kind 0)
    let mut b_kind_mismatch = b.clone();
    b_kind_mismatch.parameters = Box::new([TypeParameterRecord {
        id: param_ref.clone(),
        name: "T".into(),
        kind: KindNodeId(0), // Type instead of RecordRow
        variance: VarianceRef::Invariant,
        source: None,
    }]);
    assert!(matches!(
        validate_metadata_bundle(&b_kind_mismatch, &limits).unwrap_err(),
        MetadataValidationError::RecordTailKindMismatch { .. }
    ));

    // Negative test: missing record_rows feature
    let mut b_no_feat = b.clone();
    b_no_feat.header.features.record_rows = false;
    assert_eq!(
        validate_metadata_bundle(&b_no_feat, &limits).unwrap_err(),
        MetadataValidationError::RecordRowFeatureRequired
    );
}

#[test]
fn test_scoped_open_record_validation() {
    let limits = ValidationLimits::default();
    let mut b = sample_bundle(2, true);

    b.kinds = Box::new([
        KindNodeEntry {
            node: KindNode::Type,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        KindNodeEntry {
            node: KindNode::RecordRow,
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);

    // Scoped lambda with row binder at depth 0, index 0
    b.scoped_types = Box::new([
        ScopedTypeNodeEntry {
            kind: KindNodeId(0),
            form: ScopedTypeNode::OpenRecord(ScopedOpenRecordTypeRef {
                fields: Box::new([]),
                tail: ScopedRecordTailRef::Bound { depth: 0, index: 0 },
            }),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        ScopedTypeNodeEntry {
            kind: KindNodeId(0),
            form: ScopedTypeNode::Lambda {
                parameter_kinds: Box::new([KindNodeId(1)]), // RecordRow binder
                body: ScopedTypeNodeId(0),
            },
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);

    validate_metadata_bundle(&b, &limits).expect("scoped open record with row binder should validate");
}
```


