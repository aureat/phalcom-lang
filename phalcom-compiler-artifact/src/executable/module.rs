//! Portable module, class, source-map, and bootstrap construction records.

use crate::executable::constant::StableClassRefV1;
use crate::executable::prototype::{PrototypeId, SourceMapId};
use crate::module_world::{Decode, SpanV1, Wire, record, tagged};
use crate::{ArtifactError, BinaryWriter, ValidationLimits};
use phalcom_type_meta::identity::{StableDeclarationRef, StableModuleRef};

tagged!(PortableLinkedReadV1 {
    0 => Module { module: StableModuleRef },
    1 => Binding { module: StableModuleRef, name: String },
    2 => StaticConstant { module: StableModuleRef, name: String },
});

record!(PortableModuleV1 limit max_modules {
    identity: StableModuleRef,
    initializer: PrototypeId,
    linked_reads: Vec<PortableLinkedReadV1>,
    source: SourceMapId,
    eager: bool,
});

record!(PortableSourceMapV1 {
    logical_uri: String,
    module: StableModuleRef,
    line_starts: Vec<u64>,
    source_span: Option<SpanV1>,
});

tagged!(PortableClassAllocationV1 {
    0 => Primordial { role: String },
    1 => SourceDeclared,
});

record!(PortableFieldLayoutV1 { name: String, slot: u32 });

record!(PortableClassRecipeV1 {
    class: StableClassRefV1,
    allocation: PortableClassAllocationV1,
    superclass: Option<StableClassRefV1>,
    metaclass: Option<StableClassRefV1>,
    fields: Vec<PortableFieldLayoutV1>,
    field_count: u32,
    immutable_slots: Vec<u32>,
    native_repr: bool,
    sealed: bool,
    is_abstract: bool,
});

record!(PortableDetachedMethodV1 {
    callable: phalcom_type_meta::identity::StableCallableRef,
    method_constant: crate::executable::prototype::ConstantId,
});

record!(PortableBootstrapModuleV1 {
    module: StableModuleRef,
    initializer: PrototypeId,
});

record!(PortableBootstrapPlanV1 {
    native_runtime_roles: Vec<String>,
    eager_modules: Vec<PortableBootstrapModuleV1>,
    deferred_modules: Vec<StableModuleRef>,
    required_declarations: Vec<StableDeclarationRef>,
});
