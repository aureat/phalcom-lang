//! Portable, source-free module authority. Every enum tag is explicit wire ABI.
use crate::digest::ArtifactHasher;
use crate::{ArtifactError, BinaryReader, BinaryWriter, CodecError, ValidationLimits};
use phalcom_type_meta::Fingerprint128;
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use phalcom_type_meta::identity::{
    StableAssociatedStaticDomainDeclarationRef, StableCallableOwnerRef, StableCallableRef, StableDeclarationRef, StableDispatchSide, StableFieldRef,
    StableImplRef, StableModuleRef, StableProjectRef, StableStaticConstantRef, StableStaticDomainDeclarationRef, StableTypeMemberRef, StableVariantRef,
};

pub const MODULE_WORLD_SCHEMA_V1: u32 = 1;
pub const MODULE_WORLD_SCHEMA_V2: u32 = 2;
pub const MODULE_WORLD_SCHEMA_V3: u32 = 3;
pub const MODULE_WORLD_SCHEMA_V4: u32 = 4;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ArtifactModuleId(pub u32);

/// Decoder budgets are cumulative across the section, including nested metadata.
pub(crate) struct Decode<'a> {
    pub(crate) reader: BinaryReader<'a>,
    pub(crate) limits: ValidationLimits,
    pub(crate) allocations: &'a mut AllocationBudget,
    pub(crate) items_left: usize,
    pub(crate) depth: usize,
}
impl Decode<'_> {
    pub(crate) fn nested<T>(&mut self, f: impl FnOnce(&mut Self) -> Result<T, ArtifactError>) -> Result<T, ArtifactError> {
        if self.depth >= self.limits.max_recursion_depth {
            return Err(invalid("module payload nesting limit exceeded"));
        }
        self.depth += 1;
        let result = f(self);
        self.depth -= 1;
        result
    }
}
fn allocation_error(error: AllocationError) -> ArtifactError {
    ArtifactError::Allocation(error)
}
pub(crate) fn invalid(message: impl Into<String>) -> ArtifactError {
    ArtifactError::LimitExceeded(message.into())
}
pub(crate) trait Wire: Sized {
    fn collection_limit(limits: &ValidationLimits) -> usize {
        limits.max_semantic_entries_per_family
    }
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError>;
    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError>;
}
macro_rules! primitive {
    ($ty:ty, $write:ident, $read:ident) => {
        impl Wire for $ty {
            fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
                out.$write(*self);
                Ok(())
            }
            fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
                Ok(input.reader.$read()?)
            }
        }
    };
}
primitive!(u8, write_u8, read_u8);
primitive!(u16, write_u16, read_u16);
primitive!(u32, write_u32, read_u32);
primitive!(u64, write_u64, read_u64);
primitive!(bool, write_bool, read_bool);
impl Wire for [u8; 32] {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        out.write_fixed(self);
        Ok(())
    }
    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        Ok(input.reader.read_fixed::<32>()?)
    }
}
impl Wire for ArtifactModuleId {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        self.0.write(out)
    }
    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        Ok(Self(u32::read(input)?))
    }
}
impl Wire for String {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        let count = u32::try_from(self.len()).map_err(|_| invalid("string cannot fit wire u32"))?;
        out.write_u32(count);
        out.write_bytes(self.as_bytes());
        Ok(())
    }
    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        let text = input.reader.read_str(input.limits.max_string_bytes)?;
        input.allocations.string(text).map_err(allocation_error)
    }
}
impl<T: Wire> Wire for Vec<T> {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        out.write_u32(u32::try_from(self.len()).map_err(|_| invalid("vector cannot fit wire u32"))?);
        for value in self {
            value.write(out)?;
        }
        Ok(())
    }
    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        input.nested(|input| {
            let count = input.reader.read_u32()? as usize;
            if count > T::collection_limit(&input.limits) || count > input.reader.remaining() || count > input.items_left {
                return Err(invalid("module payload item budget exceeded"));
            }
            input.items_left -= count;
            let mut values = input.allocations.vector::<T>(count).map_err(allocation_error)?;
            for _ in 0..count {
                values.push(T::read(input)?);
            }
            Ok(values)
        })
    }
}
impl<T: Wire> Wire for Option<T> {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        match self {
            None => 0u8.write(out),
            Some(value) => {
                1u8.write(out)?;
                value.write(out)
            }
        }
    }
    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        match u8::read(input)? {
            0 => Ok(None),
            1 => T::read(input).map(Some),
            _ => Err(CodecError::CorruptedData("invalid optional tag".into()).into()),
        }
    }
}
impl<T: Wire> Wire for Box<T> {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        self.as_ref().write(out)
    }
    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        let value = T::read(input)?;
        input.allocations.boxed(value).map_err(allocation_error)
    }
}
impl Wire for Fingerprint128 {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        out.write_fixed(self.as_bytes());
        Ok(())
    }
    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        Ok(Self(input.reader.read_fixed::<16>()?))
    }
}
impl<T: Wire> Wire for Box<[T]> {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        out.write_u32(u32::try_from(self.len()).map_err(|_| invalid("boxed vector exceeds u32"))?);
        for value in self {
            value.write(out)?;
        }
        Ok(())
    }
    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        Ok(Vec::<T>::read(input)?.into_boxed_slice())
    }
}
impl Wire for Box<str> {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        out.write_u32(u32::try_from(self.len()).map_err(|_| invalid("boxed string exceeds u32"))?);
        out.write_bytes(self.as_bytes());
        Ok(())
    }
    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        Ok(String::read(input)?.into_boxed_str())
    }
}
impl Wire for StableProjectRef {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        match self {
            Self::Builtin { namespace, version } => {
                out.write_u8(0);
                namespace.write(out)?;
                version.write(out)?;
            }
            Self::Package {
                package,
                version,
                artifact_fingerprint,
            } => {
                out.write_u8(1);
                package.write(out)?;
                version.write(out)?;
                artifact_fingerprint.write(out)?;
            }
            Self::SourceArtifact {
                logical_uri,
                source_fingerprint,
            } => {
                out.write_u8(2);
                logical_uri.write(out)?;
                source_fingerprint.write(out)?;
            }
            Self::Session { .. } => return Err(invalid("session project identity is not persistent")),
        }
        Ok(())
    }
    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        match input.reader.read_u8()? {
            0 => Ok(Self::Builtin {
                namespace: Box::<str>::read(input)?,
                version: Box::<str>::read(input)?,
            }),
            1 => Ok(Self::Package {
                package: Box::<str>::read(input)?,
                version: Box::<str>::read(input)?,
                artifact_fingerprint: Fingerprint128::read(input)?,
            }),
            2 => Ok(Self::SourceArtifact {
                logical_uri: Box::<str>::read(input)?,
                source_fingerprint: Fingerprint128::read(input)?,
            }),
            _ => Err(invalid("invalid persistent project identity tag")),
        }
    }
}
impl Wire for StableDispatchSide {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        out.write_u8(match self {
            Self::Instance => 0,
            Self::Class => 1,
        });
        Ok(())
    }
    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        match input.reader.read_u8()? {
            0 => Ok(Self::Instance),
            1 => Ok(Self::Class),
            _ => Err(invalid("invalid dispatch side tag")),
        }
    }
}
macro_rules! stable_record {
    ($name:ident { $($field:ident: $ty:ty),* $(,)? }) => {
        impl Wire for $name {
            fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> { $(self.$field.write(out)?;)* Ok(()) }
            fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> { input.nested(|input| Ok(Self { $($field: <$ty>::read(input)?),* })) }
        }
    };
}

stable_record!(StableModuleRef { project: StableProjectRef, path: Box<[Box<str>]> });
stable_record!(StableDeclarationRef { module: StableModuleRef, path: Box<[Box<str>]> });
stable_record!(StableVariantRef { owner: StableDeclarationRef, selector: Box<str> });
stable_record!(StableImplRef {
    module: StableModuleRef,
    local_id: u32
});
impl Wire for StableCallableOwnerRef {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        match self {
            Self::Declaration(owner) => {
                out.write_u8(0);
                owner.write(out)?;
            }
            Self::Variant(owner) => {
                out.write_u8(1);
                owner.write(out)?;
            }
            Self::Conformance(owner) => {
                out.write_u8(2);
                owner.write(out)?;
            }
        }
        Ok(())
    }
    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        match input.reader.read_u8()? {
            0 => Ok(Self::Declaration(StableDeclarationRef::read(input)?)),
            1 => Ok(Self::Variant(StableVariantRef::read(input)?)),
            2 => Ok(Self::Conformance(StableImplRef::read(input)?)),
            _ => Err(invalid("invalid callable owner tag")),
        }
    }
}
stable_record!(StableCallableRef { owner: StableCallableOwnerRef, side: StableDispatchSide, selector: Box<str> });
stable_record!(StableFieldRef { owner: StableDeclarationRef, side: StableDispatchSide, name: Box<str> });
stable_record!(StableTypeMemberRef { owner: StableDeclarationRef, selector: Box<str> });
stable_record!(StableAssociatedStaticDomainDeclarationRef { owner: StableDeclarationRef, name: Box<str> });
stable_record!(StableStaticConstantRef { module: StableModuleRef, name: Box<str> });
impl Wire for StableStaticDomainDeclarationRef {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        match self {
            Self::Nominal(decl) => {
                out.write_u8(0);
                decl.write(out)
            }
            Self::Associated(assoc) => {
                out.write_u8(1);
                assoc.write(out)
            }
        }
    }
    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        input.nested(|input| match input.reader.read_u8()? {
            0 => Ok(Self::Nominal(StableDeclarationRef::read(input)?)),
            1 => Ok(Self::Associated(StableAssociatedStaticDomainDeclarationRef::read(input)?)),
            _ => Err(invalid("invalid StableStaticDomainDeclarationRef tag")),
        })
    }
}
macro_rules! record {
    ($name:ident { $($field:ident: $ty:ty),* $(,)? }) => { record!($name limit max_semantic_entries_per_family { $($field: $ty),* }); };
    ($name:ident limit $limit:ident { $($field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct $name { $(pub $field: $ty),* }
        impl Wire for $name {
            fn collection_limit(limits: &ValidationLimits) -> usize { limits.$limit }
            fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> { $(self.$field.write(out)?;)* Ok(()) }
            fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
                input.nested(|input| Ok(Self { $($field: <$ty>::read(input)?),* }))
            }
        }
    };
}
macro_rules! tagged {
    ($name:ident { $($tag:literal => $variant:ident $( { $($field:ident: $ty:ty),* } )?),* $(,)? }) => { tagged!($name limit max_semantic_entries_per_family { $($tag => $variant $( { $($field: $ty),* } )?),* }); };
    ($name:ident limit $limit:ident { $($tag:literal => $variant:ident $( { $($field:ident: $ty:ty),* } )?),* $(,)? }) => {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub enum $name { $($variant $( { $($field: $ty),* } )?),* }
        impl Wire for $name {
            fn collection_limit(limits: &ValidationLimits) -> usize { limits.$limit }
            fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
                match self { $(Self::$variant $( { $($field),* } )? => { out.write_u8($tag); $($( $field.write(out)?; )*)? })* } Ok(())
            }
            fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
                input.nested(|input| match input.reader.read_u8()? {
                    $($tag => Ok(Self::$variant $( { $($field: <$ty>::read(input)?),* } )?),)*
                    _ => Err($crate::CodecError::CorruptedData(concat!("invalid ", stringify!($name), " tag").into()).into()),
                })
            }
        }
    };
}
record!(SpanV1 { start: u64, end: u64 });
tagged!(ModuleKindV1 { 0 => Module, 1 => Package });
tagged!(BindingSemanticsV1 { 0 => Runtime { mutable: bool }, 1 => StaticConstant, 2 => Declaration });
record!(DeclarationV1 {
    name: String,
    semantics: BindingSemanticsV1,
    span: SpanV1
});
record!(SegmentV1 { name: String, span: SpanV1 });
tagged!(ImportRootV1 { 0 => Absolute { segment: SegmentV1 }, 1 => Relative { dots: u16, span: SpanV1 } });
record!(ImportPathV1 { root: ImportRootV1, segments: Vec<SegmentV1>, span: SpanV1 });
record!(AliasV1 { name: String, span: SpanV1 });
record!(ImportItemV1 { name: String, name_span: SpanV1, alias: Option<AliasV1>, span: SpanV1 });
tagged!(ImportV1 {
    0 => Module { path: ImportPathV1, alias: Option<AliasV1>, span: SpanV1 },
    1 => Selective { path: ImportPathV1, items: Vec<ImportItemV1>, span: SpanV1 },
    2 => ReExport { path: ImportPathV1, items: Vec<ImportItemV1>, span: SpanV1 },
});
tagged!(UnlinkedTargetV1 {
    0 => Local { name: String },
    1 => ReExport { path: ImportPathV1, remote: String },
    2 => CanonicalDeclaration { module: ArtifactModuleId, name: String },
});
record!(ExportV1 {
    exported_name: String,
    internal_name: String,
    target: UnlinkedTargetV1,
    span: SpanV1
});
tagged!(TargetV1 { 0 => Module { module: ArtifactModuleId }, 1 => Binding { module: ArtifactModuleId, name: String } });
record!(LinkedExportV1 {
    name: String,
    target: TargetV1,
    span: SpanV1
});
record!(BindingV1 { name: String, index: u32 });
tagged!(LiteralV1 {
    0 => Unit, 1 => Bool { value: bool }, 2 => Int { value: String },
    3 => Float { bits: u64 }, 4 => String { value: String }, 5 => Symbol { value: String },
    6 => Tuple { values: Vec<LiteralV1> }, 7 => Record { fields: Vec<LiteralFieldV1> },
});
record!(LiteralFieldV1 {
    name: String,
    value: LiteralV1
});
tagged!(MetadataTargetV1 { 0 => Module, 1 => Package, 2 => Project });
record!(MetadataV1 { target: MetadataTargetV1, name: String, arguments: Vec<LiteralV1>, span: SpanV1 });
tagged!(ReferenceKindV1 { 0 => WholeModuleImport, 1 => SelectiveImport, 2 => ReExport, 3 => InterfaceOnly });
record!(ReferenceV1 {
    target: ArtifactModuleId,
    kind: ReferenceKindV1,
    span: SpanV1
});
tagged!(SemanticNodeV1 { 0 => Module { module: ArtifactModuleId }, 1 => Declaration { module: ArtifactModuleId, name: String } });
tagged!(SemanticEdgeKindV1 { 0 => ModuleInterface, 1 => TypeReference, 2 => Superclass, 3 => ProtocolReference, 4 => ConstraintReference, 5 => CallbackSignature, 6 => AdtReference });
record!(SemanticEdgeV1 {
    target: SemanticNodeV1,
    kind: SemanticEdgeKindV1,
    span: SpanV1
});
record!(SemanticRowV1 { node: SemanticNodeV1, edges: Vec<SemanticEdgeV1> });
tagged!(RuntimeReasonV1 { 0 => WholeModuleImport, 1 => SelectiveValueImport, 2 => ReExport, 3 => RuntimeDeclarationReference });
record!(RuntimeEdgeV1 {
    dependency: ArtifactModuleId,
    reason: RuntimeReasonV1,
    span: SpanV1
});
record!(ModuleV1 limit max_modules {
    path: Vec<String>, kind: ModuleKindV1,
    declarations: Vec<DeclarationV1>, exports: Vec<ExportV1>, imports: Vec<ImportV1>,
    exposed_children: Vec<String>, metadata: Vec<MetadataV1>,
    linked_exports: Vec<LinkedExportV1>, globals: Vec<BindingV1>, import_bindings: Vec<BindingV1>,
    linked_reads: Vec<TargetV1>, runtime_dependencies: Vec<ArtifactModuleId>,
    references: Vec<ReferenceV1>, runtime_edges: Vec<RuntimeEdgeV1>,
    topology_parent: Option<ArtifactModuleId>, topology_children: Vec<ArtifactModuleId>,
    interface_fingerprint: u64, linked_interface_fingerprint: u64, dependency_fingerprint: u64,
});
record!(ModuleWorldV1 {
    entry: ArtifactModuleId, initialization_order: Vec<ArtifactModuleId>,
    modules: Vec<ModuleV1>, semantic_rows: Vec<SemanticRowV1>, topology_fingerprint: u64,
});
impl ModuleWorldV1 {
    pub fn encode(&self) -> Result<Vec<u8>, ArtifactError> {
        let mut out = BinaryWriter::new();
        out.write_u32(MODULE_WORLD_SCHEMA_V1);
        self.write(&mut out)?;
        Ok(out.into_vec())
    }
    pub fn decode_with_allocation_budget(bytes: &[u8], limits: &ValidationLimits, transaction: &mut AllocationBudget) -> Result<Self, ArtifactError> {
        let section_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| allocation_error(AllocationError::Overflow))?;
        let mut section = transaction.child(section_limit);
        let world = {
            let mut input = Decode {
                reader: BinaryReader::new(bytes),
                limits: *limits,
                allocations: &mut section,
                items_left: limits.max_semantic_entries_per_family,
                depth: 0,
            };
            if input.reader.read_u32()? != MODULE_WORLD_SCHEMA_V1 {
                return Err(invalid("unsupported module-world schema"));
            }
            let world = Self::read(&mut input)?;
            if input.reader.has_remaining() {
                return Err(invalid("trailing module-world payload bytes"));
            }
            world
        };
        if world.modules.len() > limits.max_modules {
            return Err(invalid("module count exceeds limit"));
        }
        world.validate_with_allocation_budget(limits, &mut section)?;
        transaction.commit_child(section).map_err(allocation_error)?;
        Ok(world)
    }
    pub fn validate(&self, limits: &ValidationLimits) -> Result<(), ArtifactError> {
        let mut allocations = AllocationBudget::new(usize::MAX);
        self.validate_with_allocation_budget(limits, &mut allocations)
    }

    fn validate_with_allocation_budget(&self, limits: &ValidationLimits, allocations: &mut AllocationBudget) -> Result<(), ArtifactError> {
        if self.modules.is_empty() {
            return Err(invalid("module world must contain at least one module"));
        }
        if self.modules.len() > limits.max_modules {
            return Err(invalid("module count exceeds limit"));
        }
        if (self.entry.0 as usize) >= self.modules.len() {
            return Err(invalid("entry module index out of bounds"));
        }
        for (i, init_id) in self.initialization_order.iter().enumerate() {
            if (init_id.0 as usize) >= self.modules.len() {
                return Err(invalid(format!("initialization order {i} references out-of-bounds module {}", init_id.0)));
            }
        }

        let mut seen_paths = allocations.sorted_set_builder::<&Vec<String>>(self.modules.len()).map_err(allocation_error)?;
        for module in &self.modules {
            seen_paths.push(&module.path).map_err(allocation_error)?;
        }
        if seen_paths.finish().is_err() {
            return Err(invalid("duplicate module identity for path"));
        }

        for (mod_idx, module) in self.modules.iter().enumerate() {
            if module.path.is_empty() && mod_idx != (self.entry.0 as usize) {
                return Err(invalid(format!("module {mod_idx} has empty path")));
            }
            for segment in &module.path {
                if segment.is_empty() {
                    return Err(invalid("empty segment in module path"));
                }
            }

            let mut seen_exports = allocations.sorted_set_builder::<&String>(module.exports.len()).map_err(allocation_error)?;
            for export in &module.exports {
                seen_exports.push(&export.exported_name).map_err(allocation_error)?;
            }
            if seen_exports.finish().is_err() {
                return Err(invalid(format!("module {mod_idx} contains duplicate exports")));
            }

            for export in &module.exports {
                if let UnlinkedTargetV1::CanonicalDeclaration { module: target_mod, .. } = &export.target {
                    if (target_mod.0 as usize) >= self.modules.len() {
                        return Err(invalid(format!("export in module {mod_idx} references missing module {}", target_mod.0)));
                    }
                }
            }

            for export in &module.linked_exports {
                match &export.target {
                    TargetV1::Module { module: target_mod } | TargetV1::Binding { module: target_mod, .. } => {
                        if (target_mod.0 as usize) >= self.modules.len() {
                            return Err(invalid(format!("linked export in module {mod_idx} references missing module {}", target_mod.0)));
                        }
                    }
                }
            }

            for read in &module.linked_reads {
                match read {
                    TargetV1::Module { module: target_mod } | TargetV1::Binding { module: target_mod, .. } => {
                        if (target_mod.0 as usize) >= self.modules.len() {
                            return Err(invalid(format!("linked read in module {mod_idx} references missing module {}", target_mod.0)));
                        }
                    }
                }
            }

            for ref_edge in &module.references {
                if (ref_edge.target.0 as usize) >= self.modules.len() {
                    return Err(invalid(format!(
                        "reference in module {mod_idx} references missing module {}",
                        ref_edge.target.0
                    )));
                }
            }

            for r_edge in &module.runtime_edges {
                if (r_edge.dependency.0 as usize) >= self.modules.len() {
                    return Err(invalid(format!(
                        "runtime edge in module {mod_idx} references missing module {}",
                        r_edge.dependency.0
                    )));
                }
            }

            for dep in &module.runtime_dependencies {
                if (dep.0 as usize) >= self.modules.len() {
                    return Err(invalid(format!("runtime dependency in module {mod_idx} references missing module {}", dep.0)));
                }
            }

            if let Some(parent) = module.topology_parent {
                if (parent.0 as usize) >= self.modules.len() {
                    return Err(invalid(format!("topology parent of module {mod_idx} references missing module {}", parent.0)));
                }
                if parent.0 as usize == mod_idx {
                    return Err(invalid(format!("module {mod_idx} has impossible self-loop as topology parent")));
                }
            }
            for child in &module.topology_children {
                if (child.0 as usize) >= self.modules.len() {
                    return Err(invalid(format!("topology child of module {mod_idx} references missing module {}", child.0)));
                }
                if child.0 as usize == mod_idx {
                    return Err(invalid(format!("module {mod_idx} has impossible self-loop as topology child")));
                }
            }
        }

        for row in &self.semantic_rows {
            let validate_node = |node: &SemanticNodeV1| -> Result<(), ArtifactError> {
                let target_mod = match node {
                    SemanticNodeV1::Module { module } => module,
                    SemanticNodeV1::Declaration { module, .. } => module,
                };
                if (target_mod.0 as usize) >= self.modules.len() {
                    return Err(invalid(format!("semantic node references missing module {}", target_mod.0)));
                }
                Ok(())
            };
            validate_node(&row.node)?;
            for edge in &row.edges {
                validate_node(&edge.target)?;
            }
        }

        Ok(())
    }
    pub fn decode_with_budget(bytes: &[u8], limits: &ValidationLimits, remaining_budget: &mut u64) -> Result<Self, ArtifactError> {
        let available = usize::try_from(*remaining_budget).map_err(|_| allocation_error(AllocationError::Overflow))?;
        let mut transaction = AllocationBudget::new(available);
        let world = Self::decode_with_allocation_budget(bytes, limits, &mut transaction)?;
        *remaining_budget = u64::try_from(transaction.remaining()).map_err(|_| allocation_error(AllocationError::Overflow))?;
        Ok(world)
    }
    pub fn decode(bytes: &[u8], limits: &ValidationLimits) -> Result<Self, ArtifactError> {
        let mut budget = limits.max_decoded_section_bytes;
        Self::decode_with_budget(bytes, limits, &mut budget)
    }
}

record!(ModuleV2 limit max_modules {
    identity: StableModuleRef,
    path: Vec<String>, kind: ModuleKindV1,
    declarations: Vec<DeclarationV1>, exports: Vec<ExportV1>, imports: Vec<ImportV1>,
    exposed_children: Vec<String>, metadata: Vec<MetadataV1>,
    linked_exports: Vec<LinkedExportV1>, globals: Vec<BindingV1>, import_bindings: Vec<BindingV1>,
    linked_reads: Vec<TargetV1>, runtime_dependencies: Vec<ArtifactModuleId>,
    references: Vec<ReferenceV1>, runtime_edges: Vec<RuntimeEdgeV1>,
    topology_parent: Option<ArtifactModuleId>, topology_children: Vec<ArtifactModuleId>,
    interface_fingerprint: u64, linked_interface_fingerprint: u64, dependency_fingerprint: u64,
});
record!(ModuleWorldV2 {
    entry: ArtifactModuleId, initialization_order: Vec<ArtifactModuleId>,
    modules: Vec<ModuleV2>, semantic_rows: Vec<SemanticRowV1>,
    external_projects: Vec<StableProjectRef>, topology_fingerprint: u64,
});

impl ModuleWorldV2 {
    /// Returns the module-relative source-binding ordinal for one import-site
    /// binding. Import sites retain their vector position even when they have
    /// no items, while each site contributes only its authored bindings.
    pub fn source_import_binding_ordinal(&self, module: &StableModuleRef, site: u32, local: u32) -> Result<Option<u32>, ArtifactError> {
        let Some(row) = self.modules.iter().find(|row| row.identity == *module) else {
            return Ok(None);
        };
        let mut base = 0u32;
        for (site_index, import) in row.imports.iter().enumerate() {
            let site_index = u32::try_from(site_index).map_err(|_| invalid("source import site ordinal exceeds u32"))?;
            let count = source_import_binding_count(import)?;
            if site_index == site {
                return if local < count {
                    base.checked_add(local)
                        .map(Some)
                        .ok_or_else(|| invalid("source import binding ordinal exceeds u32"))
                } else {
                    Ok(None)
                };
            }
            base = base.checked_add(count).ok_or_else(|| invalid("source import binding ordinal exceeds u32"))?;
        }
        Ok(None)
    }

    /// Decodes a module-relative source-binding ordinal into its import-site
    /// row and site-local binding ordinal.
    pub fn source_import_binding_at(&self, module: &StableModuleRef, ordinal: u32) -> Result<Option<(u32, u32)>, ArtifactError> {
        let Some(row) = self.modules.iter().find(|row| row.identity == *module) else {
            return Ok(None);
        };
        let mut base = 0u32;
        for (site_index, import) in row.imports.iter().enumerate() {
            let site_index = u32::try_from(site_index).map_err(|_| invalid("source import site ordinal exceeds u32"))?;
            let count = source_import_binding_count(import)?;
            let end = base.checked_add(count).ok_or_else(|| invalid("source import binding ordinal exceeds u32"))?;
            if ordinal >= base && ordinal < end {
                return Ok(Some((site_index, ordinal - base)));
            }
            base = end;
        }
        Ok(None)
    }

    /// Counts the source bindings represented by a module's ordered import
    /// rows. Empty selective imports and re-exports contribute no bindings.
    pub fn source_import_binding_count(&self, module: &StableModuleRef) -> Result<Option<u32>, ArtifactError> {
        let Some(row) = self.modules.iter().find(|row| row.identity == *module) else {
            return Ok(None);
        };
        let count = row.imports.iter().try_fold(0u32, |count, import| {
            count
                .checked_add(source_import_binding_count(import)?)
                .ok_or_else(|| invalid("source import binding count exceeds u32"))
        })?;
        Ok(Some(count))
    }

    /// Recomputes the artifact-scoped topology fingerprint from portable module
    /// identities and the module links represented in this wire image.
    pub fn portable_topology_fingerprint(&self) -> Result<u64, ArtifactError> {
        let mut out = PortableTopologyHasher::new();
        out.count(self.external_projects.len())?;
        for project in &self.external_projects {
            out.project(project)?;
        }

        out.count(self.modules.len())?;
        for module in &self.modules {
            out.module_identity(&module.identity)?;
            out.u8(match module.kind {
                ModuleKindV1::Module => 0,
                ModuleKindV1::Package => 1,
            });
            match module.topology_parent {
                None => out.u8(0),
                Some(parent) => {
                    out.u8(1);
                    out.module_id(&self.modules, parent)?;
                }
            }
            out.count(module.topology_children.len())?;
            for child in &module.topology_children {
                out.module_id(&self.modules, *child)?;
            }
            out.count(module.exposed_children.len())?;
            for child in &module.exposed_children {
                out.text(child)?;
            }

            out.count(module.exports.len())?;
            for export in &module.exports {
                match &export.target {
                    UnlinkedTargetV1::CanonicalDeclaration { module: target, name } => {
                        out.u8(1);
                        out.text(&export.exported_name)?;
                        out.module_id(&self.modules, *target)?;
                        out.text(name)?;
                    }
                    UnlinkedTargetV1::Local { .. } | UnlinkedTargetV1::ReExport { .. } => out.u8(0),
                }
            }

            out.count(module.linked_exports.len())?;
            for export in &module.linked_exports {
                out.text(&export.name)?;
                out.target_link(&self.modules, &export.target)?;
            }
            out.count(module.linked_reads.len())?;
            for read in &module.linked_reads {
                out.target_link(&self.modules, read)?;
            }

            out.count(module.runtime_dependencies.len())?;
            for dependency in &module.runtime_dependencies {
                out.module_id(&self.modules, *dependency)?;
            }
            out.count(module.references.len())?;
            for reference in &module.references {
                out.u8(match reference.kind {
                    ReferenceKindV1::WholeModuleImport => 0,
                    ReferenceKindV1::SelectiveImport => 1,
                    ReferenceKindV1::ReExport => 2,
                    ReferenceKindV1::InterfaceOnly => 3,
                });
                out.module_id(&self.modules, reference.target)?;
            }
            out.count(module.runtime_edges.len())?;
            for edge in &module.runtime_edges {
                out.u8(match edge.reason {
                    RuntimeReasonV1::WholeModuleImport => 0,
                    RuntimeReasonV1::SelectiveValueImport => 1,
                    RuntimeReasonV1::ReExport => 2,
                    RuntimeReasonV1::RuntimeDeclarationReference => 3,
                });
                out.module_id(&self.modules, edge.dependency)?;
            }
        }

        out.count(self.semantic_rows.len())?;
        for row in &self.semantic_rows {
            out.semantic_node(&self.modules, &row.node)?;
            out.count(row.edges.len())?;
            for edge in &row.edges {
                out.u8(match edge.kind {
                    SemanticEdgeKindV1::ModuleInterface => 0,
                    SemanticEdgeKindV1::TypeReference => 1,
                    SemanticEdgeKindV1::Superclass => 2,
                    SemanticEdgeKindV1::ProtocolReference => 3,
                    SemanticEdgeKindV1::ConstraintReference => 4,
                    SemanticEdgeKindV1::CallbackSignature => 5,
                    SemanticEdgeKindV1::AdtReference => 6,
                });
                out.semantic_node(&self.modules, &edge.target)?;
            }
        }

        Ok(out.finish())
    }

    /// Updates the stored fingerprint after constructing or modifying a wire image.
    pub fn refresh_portable_topology_fingerprint(&mut self) -> Result<(), ArtifactError> {
        self.topology_fingerprint = self.portable_topology_fingerprint()?;
        Ok(())
    }

    pub fn encode(&self) -> Result<Vec<u8>, ArtifactError> {
        let mut out = BinaryWriter::new();
        out.write_u32(MODULE_WORLD_SCHEMA_V2);
        self.write(&mut out)?;
        Ok(out.into_vec())
    }

    pub fn decode_with_allocation_budget(bytes: &[u8], limits: &ValidationLimits, transaction: &mut AllocationBudget) -> Result<Self, ArtifactError> {
        let section_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| allocation_error(AllocationError::Overflow))?;
        let mut section = transaction.child(section_limit);
        let world = {
            let mut input = Decode {
                reader: BinaryReader::new(bytes),
                limits: *limits,
                allocations: &mut section,
                items_left: limits.max_semantic_entries_per_family,
                depth: 0,
            };
            let schema = input.reader.read_u32()?;
            if schema != MODULE_WORLD_SCHEMA_V2 {
                return Err(crate::error::ValidationError::IncompatibleAbi {
                    kind: "module_world_schema",
                    found: schema,
                    expected: MODULE_WORLD_SCHEMA_V2,
                }
                .into());
            }
            let world = Self::read(&mut input)?;
            if input.reader.has_remaining() {
                return Err(invalid("trailing module-world payload bytes"));
            }
            world
        };
        if world.modules.len() > limits.max_modules {
            return Err(invalid("module count exceeds limit"));
        }
        world.validate(limits)?;
        transaction.commit_child(section).map_err(allocation_error)?;
        Ok(world)
    }

    pub fn validate(&self, limits: &ValidationLimits) -> Result<(), ArtifactError> {
        if self.modules.is_empty() {
            return Err(invalid("module world must contain at least one module"));
        }
        if self.modules.len() > limits.max_modules {
            return Err(invalid("module count exceeds limit"));
        }
        if (self.entry.0 as usize) >= self.modules.len() {
            return Err(invalid("entry module index out of bounds"));
        }
        for (i, init_id) in self.initialization_order.iter().enumerate() {
            if (init_id.0 as usize) >= self.modules.len() {
                return Err(invalid(format!("initialization order {i} references out-of-bounds module {}", init_id.0)));
            }
        }

        // Validate external projects: sorted, unique, and no session variant
        for adjacent in self.external_projects.windows(2) {
            if adjacent[0] == adjacent[1] {
                return Err(invalid(format!("duplicate external project identity: {:?}", adjacent[0])));
            }
            if adjacent[0] > adjacent[1] {
                return Err(invalid("external projects are not in canonical sorted order"));
            }
        }
        for project in &self.external_projects {
            crate::identity::validate_persistent_project_ref(project, limits)?;
        }

        for (mod_idx, module) in self.modules.iter().enumerate() {
            crate::identity::validate_persistent_project_ref(&module.identity.project, limits)?;
            if module.path.len() != module.identity.path.len() || !module.path.iter().zip(module.identity.path.iter()).all(|(a, b)| a.as_str() == b.as_ref()) {
                return Err(invalid(format!(
                    "module {mod_idx} path does not match stable module identity: {:?} vs {:?}",
                    module.path, module.identity.path
                )));
            }
            if module.path.is_empty() && mod_idx != (self.entry.0 as usize) {
                return Err(invalid(format!("module {mod_idx} has empty path")));
            }
            for segment in &module.path {
                if segment.is_empty() {
                    return Err(invalid("empty segment in module path"));
                }
            }

            for other_idx in (mod_idx + 1)..self.modules.len() {
                if self.modules[mod_idx].identity == self.modules[other_idx].identity {
                    return Err(invalid(format!("duplicate module identity for {:?}", self.modules[mod_idx].identity)));
                }
            }

            for (exp_idx, export) in module.exports.iter().enumerate() {
                for other_exp in &module.exports[(exp_idx + 1)..] {
                    if export.exported_name == other_exp.exported_name {
                        return Err(invalid(format!("module {mod_idx} contains duplicate export {:?}", export.exported_name)));
                    }
                }
                if let UnlinkedTargetV1::CanonicalDeclaration { module: target_mod, .. } = &export.target {
                    if (target_mod.0 as usize) >= self.modules.len() {
                        return Err(invalid(format!("export in module {mod_idx} references missing module {}", target_mod.0)));
                    }
                }
            }

            for export in &module.linked_exports {
                match &export.target {
                    TargetV1::Module { module: target_mod } | TargetV1::Binding { module: target_mod, .. } => {
                        if (target_mod.0 as usize) >= self.modules.len() {
                            return Err(invalid(format!("linked export in module {mod_idx} references missing module {}", target_mod.0)));
                        }
                    }
                }
            }

            for read in &module.linked_reads {
                match read {
                    TargetV1::Module { module: target_mod } | TargetV1::Binding { module: target_mod, .. } => {
                        if (target_mod.0 as usize) >= self.modules.len() {
                            return Err(invalid(format!("linked read in module {mod_idx} references missing module {}", target_mod.0)));
                        }
                    }
                }
            }

            for ref_edge in &module.references {
                if (ref_edge.target.0 as usize) >= self.modules.len() {
                    return Err(invalid(format!(
                        "reference in module {mod_idx} references missing module {}",
                        ref_edge.target.0
                    )));
                }
            }

            for r_edge in &module.runtime_edges {
                if (r_edge.dependency.0 as usize) >= self.modules.len() {
                    return Err(invalid(format!(
                        "runtime edge in module {mod_idx} references missing module {}",
                        r_edge.dependency.0
                    )));
                }
            }

            for dep in &module.runtime_dependencies {
                if (dep.0 as usize) >= self.modules.len() {
                    return Err(invalid(format!("runtime dependency in module {mod_idx} references missing module {}", dep.0)));
                }
            }

            if let Some(parent) = module.topology_parent {
                if (parent.0 as usize) >= self.modules.len() {
                    return Err(invalid(format!("topology parent of module {mod_idx} references missing module {}", parent.0)));
                }
                if parent.0 as usize == mod_idx {
                    return Err(invalid(format!("module {mod_idx} has impossible self-loop as topology parent")));
                }
            }
            for child in &module.topology_children {
                if (child.0 as usize) >= self.modules.len() {
                    return Err(invalid(format!("topology child of module {mod_idx} references missing module {}", child.0)));
                }
                if child.0 as usize == mod_idx {
                    return Err(invalid(format!("module {mod_idx} has impossible self-loop as topology child")));
                }
            }
        }

        for row in &self.semantic_rows {
            let validate_node = |node: &SemanticNodeV1| -> Result<(), ArtifactError> {
                let target_mod = match node {
                    SemanticNodeV1::Module { module } => module,
                    SemanticNodeV1::Declaration { module, .. } => module,
                };
                if (target_mod.0 as usize) >= self.modules.len() {
                    return Err(invalid(format!("semantic node references missing module {}", target_mod.0)));
                }
                Ok(())
            };
            validate_node(&row.node)?;
            for edge in &row.edges {
                validate_node(&edge.target)?;
            }
        }

        if self.portable_topology_fingerprint()? != self.topology_fingerprint {
            return Err(invalid("portable module topology fingerprint mismatch"));
        }

        Ok(())
    }

    pub fn decode_with_budget(bytes: &[u8], limits: &ValidationLimits, remaining_budget: &mut u64) -> Result<Self, ArtifactError> {
        let available = usize::try_from(*remaining_budget).map_err(|_| allocation_error(AllocationError::Overflow))?;
        let mut transaction = AllocationBudget::new(available);
        let world = Self::decode_with_allocation_budget(bytes, limits, &mut transaction)?;
        *remaining_budget = u64::try_from(transaction.remaining()).map_err(|_| allocation_error(AllocationError::Overflow))?;
        Ok(world)
    }

    pub fn decode(bytes: &[u8], limits: &ValidationLimits) -> Result<Self, ArtifactError> {
        let mut budget = limits.max_decoded_section_bytes;
        Self::decode_with_budget(bytes, limits, &mut budget)
    }
}

tagged!(AvailabilityProductV1 {
    0 => CanonicalIdentity,
    1 => MetadataDescriptor,
    2 => RuntimeObject,
    3 => SourceInitializedValue,
});
tagged!(AvailabilityAuthorityV1 {
    0 => SemanticIdentityCatalog,
    1 => NativeBindingCatalog,
    2 => NativeModuleMaterialization,
    3 => SemanticMetadataBundle,
    4 => CanonicalSourceInitializer,
    5 => FullyInitializedUniverse,
});
tagged!(AvailabilityTimingV1 {
    0 => BeforeSourceExecution,
    1 => DuringModuleInitializer,
    2 => AfterBootstrap,
});
tagged!(AvailabilityRequirementProducerV1 {
    0 => DeclarationRead,
    1 => ImportRead,
    2 => CompilerGeneratedRead,
    3 => CallableOwner,
    4 => PatternOwner,
    5 => ConformanceTypeOwner,
    6 => ContextualReceiverRead,
    7 => MetadataTypeOwner,
    8 => StaticConstantRead,
});

// One reusable availability guarantee supplied by the canonical execution
// environment. Unlike a V3 runtime witness, this is not tied to one consumer
// requirement or source range.
record!(AvailabilityProofV1 {
    target: TargetV1,
    product: AvailabilityProductV1,
    authority: AvailabilityAuthorityV1,
    available_at: AvailabilityTimingV1,
});

impl AvailabilityProofV1 {
    /// Orders complete environment proofs in canonical wire order.
    pub fn canonical_cmp(&self, other: &Self) -> std::cmp::Ordering {
        availability_proof_key(self).cmp(&availability_proof_key(other))
    }
}

/// One exact availability proof applied to a finalized runtime requirement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuntimeAvailabilityWitnessV1 {
    /// Importer whose eager dependency was omitted.
    pub importer: ArtifactModuleId,
    /// Runtime dependency whose product is required.
    pub dependency: ArtifactModuleId,
    /// Source location of the runtime requirement.
    pub range: SpanV1,
    /// Runtime reason that produced the requirement.
    pub reason: RuntimeReasonV1,
    /// Canonical read retained for the requirement, when it has one.
    pub read_target: Option<TargetV1>,
    /// Exact target of the availability proof.
    pub target: TargetV1,
    /// Product guaranteed by the proof.
    pub product: AvailabilityProductV1,
    /// Producer of the runtime requirement.
    pub producer: AvailabilityRequirementProducerV1,
    /// Stage at which the requirement needs the product.
    pub required_at: AvailabilityTimingV1,
    /// Authority that guarantees the product.
    pub authority: AvailabilityAuthorityV1,
    /// Latest stage at which the product is guaranteed to be available.
    pub available_at: AvailabilityTimingV1,
}

impl Wire for RuntimeAvailabilityWitnessV1 {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        self.importer.write(out)?;
        self.dependency.write(out)?;
        self.range.write(out)?;
        self.reason.write(out)?;
        self.read_target.write(out)?;
        self.target.write(out)?;
        self.product.write(out)?;
        self.producer.write(out)?;
        self.required_at.write(out)?;
        self.authority.write(out)?;
        self.available_at.write(out)
    }

    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        input.nested(|input| {
            Ok(Self {
                importer: ArtifactModuleId::read(input)?,
                dependency: ArtifactModuleId::read(input)?,
                range: SpanV1::read(input)?,
                reason: RuntimeReasonV1::read(input)?,
                read_target: Option::<TargetV1>::read(input)?,
                target: TargetV1::read(input)?,
                product: AvailabilityProductV1::read(input)?,
                producer: AvailabilityRequirementProducerV1::read(input)?,
                required_at: AvailabilityTimingV1::read(input)?,
                authority: AvailabilityAuthorityV1::read(input)?,
                available_at: AvailabilityTimingV1::read(input)?,
            })
        })
    }
}

impl RuntimeAvailabilityWitnessV1 {
    /// Orders complete witness records in canonical wire order.
    pub fn canonical_cmp(&self, other: &Self) -> std::cmp::Ordering {
        availability_witness_key(self).cmp(&availability_witness_key(other))
    }
}

/// Module-world schema carrying exact availability evidence for finalized runtime edges.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleWorldV3 {
    /// Source and semantic module authority, including the finalized runtime projection.
    pub world: ModuleWorldV2,
    /// Exact proofs that justify omitted eager edges.
    pub availability_witnesses: Vec<RuntimeAvailabilityWitnessV1>,
}

impl std::ops::Deref for ModuleWorldV3 {
    type Target = ModuleWorldV2;

    fn deref(&self) -> &Self::Target {
        &self.world
    }
}

impl ModuleWorldV3 {
    /// Wraps a module world with finalized availability evidence.
    pub fn new(world: ModuleWorldV2, availability_witnesses: Vec<RuntimeAvailabilityWitnessV1>) -> Self {
        Self { world, availability_witnesses }
    }

    /// Serializes the current module-world wire schema.
    pub fn encode(&self) -> Result<Vec<u8>, ArtifactError> {
        let mut out = BinaryWriter::new();
        out.write_u32(MODULE_WORLD_SCHEMA_V3);
        self.write(&mut out)?;
        Ok(out.into_vec())
    }

    /// Decodes and validates the current module-world wire schema under a cumulative allocation budget.
    pub fn decode_with_allocation_budget(
        bytes: &[u8],
        limits: &ValidationLimits,
        transaction: &mut AllocationBudget,
    ) -> Result<Self, ArtifactError> {
        let section_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| allocation_error(AllocationError::Overflow))?;
        let mut section = transaction.child(section_limit);
        let world = {
            let mut input = Decode {
                reader: BinaryReader::new(bytes),
                limits: *limits,
                allocations: &mut section,
                items_left: limits.max_semantic_entries_per_family,
                depth: 0,
            };
            let schema = input.reader.read_u32()?;
            if schema != MODULE_WORLD_SCHEMA_V3 {
                return Err(crate::error::ValidationError::IncompatibleAbi {
                    kind: "module_world_schema",
                    found: schema,
                    expected: MODULE_WORLD_SCHEMA_V3,
                }
                .into());
            }
            let world = Self::read(&mut input)?;
            if input.reader.has_remaining() {
                return Err(invalid("trailing module-world payload bytes"));
            }
            world
        };
        world.validate(limits)?;
        transaction.commit_child(section).map_err(allocation_error)?;
        Ok(world)
    }

    /// Validates the source-free module products and their exact availability witnesses.
    pub fn validate(&self, limits: &ValidationLimits) -> Result<(), ArtifactError> {
        self.world.validate(limits)?;
        if self.availability_witnesses.len() > limits.max_semantic_entries_per_family {
            return Err(invalid("availability witness count exceeds limit"));
        }
        if self
            .availability_witnesses
            .windows(2)
            .any(|pair| pair[0].canonical_cmp(&pair[1]) != std::cmp::Ordering::Less)
        {
            return Err(invalid("availability witnesses are not uniquely sorted"));
        }

        for (module_index, module) in self.world.modules.iter().enumerate() {
            let module_index = u32::try_from(module_index).map_err(|_| invalid("module index exceeds u32"))?;
            let dependencies_match = module
                .runtime_dependencies
                .iter()
                .all(|dependency| module.runtime_edges.iter().any(|edge| edge.dependency == *dependency))
                && module
                    .runtime_edges
                    .iter()
                    .all(|edge| module.runtime_dependencies.contains(&edge.dependency));
            if !dependencies_match {
                return Err(invalid(format!("runtime dependencies and edges disagree for module {module_index}")));
            }
            for read in &module.linked_reads {
                let dependency = match read {
                    TargetV1::Module { module } | TargetV1::Binding { module, .. } => *module,
                };
                if !module.runtime_dependencies.contains(&dependency)
                    && !self.availability_witnesses.iter().any(|witness| {
                        witness.importer.0 == module_index
                            && witness.dependency == dependency
                            && witness.read_target.as_ref() == Some(read)
                            && witness.target == *read
                    })
                {
                    return Err(invalid(format!("runtime read in module {module_index} has neither an eager edge nor an exact availability witness")));
                }
            }
        }

        for witness in &self.availability_witnesses {
            let importer = self
                .world
                .modules
                .get(witness.importer.0 as usize)
                .ok_or_else(|| invalid("availability witness importer is out of bounds"))?;
            if (witness.dependency.0 as usize) >= self.world.modules.len() {
                return Err(invalid("availability witness dependency is out of bounds"));
            }
            let target_module = match &witness.target {
                TargetV1::Module { module } | TargetV1::Binding { module, .. } => *module,
            };
            if target_module != witness.dependency {
                return Err(invalid("availability witness target and dependency disagree"));
            }
            if let Some(read_target) = &witness.read_target {
                if read_target != &witness.target || !importer.linked_reads.contains(read_target) {
                    return Err(invalid("availability witness does not match a finalized runtime read"));
                }
            }
            let available = availability_timing_tag(&witness.available_at) <= availability_timing_tag(&witness.required_at)
                || (witness.authority == AvailabilityAuthorityV1::FullyInitializedUniverse
                    && witness.required_at == AvailabilityTimingV1::DuringModuleInitializer);
            if !available {
                return Err(invalid("availability witness is later than its runtime requirement"));
            }
        }

        Ok(())
    }

    /// Validates availability witnesses against the semantic runtime requirements and import phases.
    pub fn validate_execution_authority(
        &self,
        authority: &crate::semantic::world::ModuleExecutionAuthorityV1,
    ) -> Result<(), ArtifactError> {
        for requirement in &authority.runtime_requirements {
            let importer = self
                .world
                .modules
                .iter()
                .position(|module| module.identity == requirement.importer)
                .ok_or_else(|| invalid("runtime requirement importer is absent from module world"))?;
            let dependency = self
                .world
                .modules
                .iter()
                .position(|module| module.identity == requirement.dependency)
                .ok_or_else(|| invalid("runtime requirement dependency is absent from module world"))?;
            let importer_id = ArtifactModuleId(u32::try_from(importer).map_err(|_| invalid("module index exceeds u32"))?);
            let dependency_id = ArtifactModuleId(u32::try_from(dependency).map_err(|_| invalid("module index exceeds u32"))?);
            if !self.world.modules[importer].runtime_dependencies.contains(&dependency_id)
                && !self.availability_witnesses.iter().any(|witness| {
                    witness_matches_requirement(witness, requirement, &self.world.modules)
                })
            {
                return Err(invalid("runtime requirement has neither an eager edge nor an exact availability witness"));
            }
            if requirement.provenance.consumer != requirement.importer || importer_id.0 as usize != importer || dependency_id.0 as usize != dependency {
                return Err(invalid("runtime requirement ownership is inconsistent with module world"));
            }
        }

        for witness in &self.availability_witnesses {
            let matches_requirement = authority
                .runtime_requirements
                .iter()
                .any(|requirement| witness_matches_requirement(witness, requirement, &self.world.modules));
            if !matches_requirement && !is_source_import_witness(witness, authority, &self.world.modules) {
                return Err(invalid(format!("availability witness has no matching runtime requirement or runtime source import: {witness:?}")));
            }
        }
        Ok(())
    }
}

fn witness_matches_requirement(
    witness: &RuntimeAvailabilityWitnessV1,
    requirement: &crate::semantic::world::RuntimeDependencyRequirementV1,
    modules: &[ModuleV2],
) -> bool {
    let Some(importer) = modules.get(witness.importer.0 as usize) else {
        return false;
    };
    let Some(dependency) = modules.get(witness.dependency.0 as usize) else {
        return false;
    };
    if importer.identity != requirement.importer
        || dependency.identity != requirement.dependency
        || witness.range != requirement.range
        || !runtime_reason_matches(&witness.reason, &requirement.reason)
        || !availability_producer_matches(&witness.producer, &requirement.provenance.producer)
        || !availability_product_matches(&witness.product, &requirement.provenance.product)
        || !availability_timing_matches(&witness.required_at, &requirement.provenance.timing)
    {
        return false;
    }
    let Some(target) = stable_requirement_target(&witness.target, modules) else {
        return false;
    };
    if target != requirement.provenance.target {
        return false;
    }
    let read_target = match &witness.read_target {
        Some(target) => match stable_linked_read(target, modules) {
            Some(target) => Some(target),
            None => return false,
        },
        None => None,
    };
    match (&read_target, &requirement.read_target) {
        (Some(left), Some(right)) if left == right => {}
        (None, None) => {}
        _ => return false,
    }
    true
}

fn stable_requirement_target(
    target: &TargetV1,
    modules: &[ModuleV2],
) -> Option<crate::semantic::world::RuntimeRequirementTargetV1> {
    use crate::semantic::world::RuntimeRequirementTargetV1 as Target;
    match target {
        TargetV1::Module { module } => Some(Target::Module {
            module: modules.get(module.0 as usize)?.identity.clone(),
        }),
        TargetV1::Binding { module, name } => Some(Target::Binding {
            module: modules.get(module.0 as usize)?.identity.clone(),
            name: name.clone().into_boxed_str(),
        }),
    }
}

fn stable_linked_read(target: &TargetV1, modules: &[ModuleV2]) -> Option<crate::semantic::world::LinkedReadSpecV1> {
    use crate::semantic::world::LinkedReadSpecV1 as Read;
    match target {
        TargetV1::Module { module } => Some(Read::Module {
            module: modules.get(module.0 as usize)?.identity.clone(),
        }),
        TargetV1::Binding { module, name } => Some(Read::Binding {
            symbol: StableDeclarationRef {
                module: modules.get(module.0 as usize)?.identity.clone(),
                path: vec![name.as_str().into()].into_boxed_slice(),
            },
        }),
    }
}

fn runtime_reason_matches(left: &RuntimeReasonV1, right: &crate::semantic::world::RuntimeDependencyReasonV1) -> bool {
    use crate::semantic::world::RuntimeDependencyReasonV1 as Reason;
    matches!(
        (left, right),
        (RuntimeReasonV1::WholeModuleImport, Reason::WholeModuleImport)
            | (RuntimeReasonV1::SelectiveValueImport, Reason::SelectiveValueImport)
            | (RuntimeReasonV1::ReExport, Reason::ReExport)
            | (RuntimeReasonV1::RuntimeDeclarationReference, Reason::RuntimeDeclarationReference)
    )
}

fn availability_producer_matches(
    left: &AvailabilityRequirementProducerV1,
    right: &crate::semantic::world::RuntimeRequirementProducerV1,
) -> bool {
    use crate::semantic::world::RuntimeRequirementProducerV1 as Producer;
    matches!(
        (left, right),
        (AvailabilityRequirementProducerV1::DeclarationRead, Producer::DeclarationRead)
            | (AvailabilityRequirementProducerV1::ImportRead, Producer::ImportRead)
            | (AvailabilityRequirementProducerV1::CompilerGeneratedRead, Producer::CompilerGeneratedRead)
            | (AvailabilityRequirementProducerV1::CallableOwner, Producer::CallableOwner)
            | (AvailabilityRequirementProducerV1::PatternOwner, Producer::PatternOwner)
            | (AvailabilityRequirementProducerV1::ConformanceTypeOwner, Producer::ConformanceTypeOwner)
            | (AvailabilityRequirementProducerV1::ContextualReceiverRead, Producer::ContextualReceiverRead)
            | (AvailabilityRequirementProducerV1::MetadataTypeOwner, Producer::MetadataTypeOwner)
            | (AvailabilityRequirementProducerV1::StaticConstantRead, Producer::StaticConstantRead)
    )
}

fn availability_product_matches(left: &AvailabilityProductV1, right: &crate::semantic::world::RuntimeRequirementProductV1) -> bool {
    use crate::semantic::world::RuntimeRequirementProductV1 as Product;
    matches!(
        (left, right),
        (AvailabilityProductV1::CanonicalIdentity, Product::CanonicalIdentity)
            | (AvailabilityProductV1::MetadataDescriptor, Product::MetadataDescriptor)
            | (AvailabilityProductV1::RuntimeObject, Product::RuntimeObject)
            | (AvailabilityProductV1::SourceInitializedValue, Product::SourceInitializedValue)
    )
}

fn availability_timing_matches(left: &AvailabilityTimingV1, right: &crate::semantic::world::RuntimeRequirementTimingV1) -> bool {
    use crate::semantic::world::RuntimeRequirementTimingV1 as Timing;
    matches!(
        (left, right),
        (AvailabilityTimingV1::BeforeSourceExecution, Timing::BeforeSourceExecution)
            | (AvailabilityTimingV1::DuringModuleInitializer, Timing::DuringModuleInitializer)
            | (AvailabilityTimingV1::AfterBootstrap, Timing::AfterBootstrap)
    )
}

fn is_source_import_witness(
    witness: &RuntimeAvailabilityWitnessV1,
    authority: &crate::semantic::world::ModuleExecutionAuthorityV1,
    modules: &[ModuleV2],
) -> bool {
    if witness.producer != AvailabilityRequirementProducerV1::ImportRead {
        return false;
    }
    let Some(read_target) = witness.read_target.as_ref() else {
        return false;
    };
    let Some(importer) = modules.get(witness.importer.0 as usize) else {
        return false;
    };
    let Some(phase_product) = authority.import_phase_products.iter().find(|product| product.module == importer.identity) else {
        return false;
    };
    let mut ordinal = 0u32;
    for import in &importer.imports {
        let local_names = source_import_local_names(import);
        for local_name in &local_names {
            let current = ordinal;
            ordinal = match ordinal.checked_add(1) {
                Some(next) => next,
                None => return false,
            };
            let Some(binding) = importer.import_bindings.iter().find(|binding| binding.name == local_name.as_str()) else {
                continue;
            };
            if importer.linked_reads.get(binding.index as usize) != Some(read_target) {
                continue;
            }
            let phase_runtime = phase_product.bindings.iter().find(|binding| binding.binding == current).is_some_and(|binding| {
                phase_product.coverage == crate::semantic::world::PhaseEvidenceCoverageV1::Incomplete
                    || !binding.summary.observed
                    || binding.summary.phase == crate::semantic::world::DependencyPhaseV1::Runtime
            });
            let force_runtime = matches!(import, ImportV1::Module { .. } | ImportV1::ReExport { .. });
            if force_runtime || phase_runtime {
                return true;
            }
        }
    }
    false
}

fn source_import_local_names(import: &ImportV1) -> Vec<String> {
    match import {
        ImportV1::Module { path, alias, .. } => {
            let name = alias
                .as_ref()
                .map(|alias| alias.name.clone())
                .or_else(|| path.segments.last().map(|segment| segment.name.clone()))
                .or_else(|| match &path.root {
                    ImportRootV1::Absolute { segment } => Some(segment.name.clone()),
                    ImportRootV1::Relative { .. } => None,
                });
            name.into_iter().collect()
        }
        ImportV1::Selective { items, .. } | ImportV1::ReExport { items, .. } => items
            .iter()
            .map(|item| item.alias.as_ref().map_or_else(|| item.name.clone(), |alias| alias.name.clone()))
            .collect(),
    }
}

impl Wire for ModuleWorldV3 {
    fn collection_limit(limits: &ValidationLimits) -> usize {
        limits.max_modules
    }

    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        self.world.write(out)?;
        self.availability_witnesses.write(out)
    }

    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        input.nested(|input| {
            Ok(Self {
                world: ModuleWorldV2::read(input)?,
                availability_witnesses: Vec::<RuntimeAvailabilityWitnessV1>::read(input)?,
            })
        })
    }
}

/// Module-world schema carrying the canonical post-bootstrap execution
/// environment contract in addition to V3's requirement-specific witnesses.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleWorldV4 {
    /// Prior schema products and exact requirement-specific witnesses.
    pub previous: ModuleWorldV3,
    /// Canonical Universe bootstrap schedule. Order is semantic and must never
    /// be sorted during canonicalization.
    pub bootstrap_modules: Vec<ArtifactModuleId>,
    /// Complete reusable environment contract, uniquely sorted in canonical
    /// proof order.
    pub bootstrap_availability: Vec<AvailabilityProofV1>,
}

impl std::ops::Deref for ModuleWorldV4 {
    type Target = ModuleWorldV3;

    fn deref(&self) -> &Self::Target {
        &self.previous
    }
}

impl ModuleWorldV4 {
    pub fn new(
        previous: ModuleWorldV3,
        bootstrap_modules: Vec<ArtifactModuleId>,
        bootstrap_availability: Vec<AvailabilityProofV1>,
    ) -> Self {
        Self {
            previous,
            bootstrap_modules,
            bootstrap_availability,
        }
    }

    /// Serializes the current module-world wire schema.
    pub fn encode(&self) -> Result<Vec<u8>, ArtifactError> {
        let mut out = BinaryWriter::new();
        out.write_u32(MODULE_WORLD_SCHEMA_V4);
        self.write(&mut out)?;
        Ok(out.into_vec())
    }

    /// Decodes and validates V4 under the caller's cumulative allocation
    /// transaction. Cross-section completeness checks remain the loader's
    /// responsibility because they require metadata and native catalogs.
    pub fn decode_with_allocation_budget(
        bytes: &[u8],
        limits: &ValidationLimits,
        transaction: &mut AllocationBudget,
    ) -> Result<Self, ArtifactError> {
        let section_limit = usize::try_from(limits.max_decoded_section_bytes).map_err(|_| allocation_error(AllocationError::Overflow))?;
        let mut section = transaction.child(section_limit);
        let world = {
            let mut input = Decode {
                reader: BinaryReader::new(bytes),
                limits: *limits,
                allocations: &mut section,
                items_left: limits.max_semantic_entries_per_family,
                depth: 0,
            };
            let schema = input.reader.read_u32()?;
            if schema != MODULE_WORLD_SCHEMA_V4 {
                return Err(crate::error::ValidationError::IncompatibleAbi {
                    kind: "module_world_schema",
                    found: schema,
                    expected: MODULE_WORLD_SCHEMA_V4,
                }
                .into());
            }
            let world = Self::read(&mut input)?;
            if input.reader.has_remaining() {
                return Err(invalid("trailing module-world payload bytes"));
            }
            world
        };
        world.validate(limits)?;
        transaction.commit_child(section).map_err(allocation_error)?;
        Ok(world)
    }

    /// Validates V4-local authority. Completeness against native bindings and
    /// metadata roots is intentionally performed by the canonical artifact
    /// loader, which owns those independent authorities.
    pub fn validate(&self, limits: &ValidationLimits) -> Result<(), ArtifactError> {
        self.previous.validate(limits)?;
        if self.bootstrap_modules.len() > limits.max_modules {
            return Err(invalid("bootstrap module count exceeds limit"));
        }
        if self.bootstrap_availability.len() > limits.max_semantic_entries_per_family {
            return Err(invalid("bootstrap availability proof count exceeds limit"));
        }

        for (index, module) in self.bootstrap_modules.iter().enumerate() {
            let row = self
                .previous
                .world
                .modules
                .get(module.0 as usize)
                .ok_or_else(|| invalid("bootstrap schedule references an out-of-bounds module"))?;
            if self.bootstrap_modules[..index].contains(module) {
                return Err(invalid("bootstrap schedule contains a duplicate module"));
            }
            if !matches!(&row.identity.project, StableProjectRef::Builtin { namespace, .. } if namespace.as_ref() == "universe") {
                return Err(invalid("bootstrap schedule contains a non-Universe module"));
            }
        }

        if self
            .bootstrap_availability
            .windows(2)
            .any(|pair| pair[0].canonical_cmp(&pair[1]) != std::cmp::Ordering::Less)
        {
            return Err(invalid("bootstrap availability proofs are not uniquely sorted"));
        }

        for proof in &self.bootstrap_availability {
            self.validate_bootstrap_proof(proof)?;
        }
        Ok(())
    }

    fn validate_bootstrap_proof(&self, proof: &AvailabilityProofV1) -> Result<(), ArtifactError> {
        let module = match &proof.target {
            TargetV1::Module { module } => *module,
            TargetV1::Binding { module, name } => {
                if name.is_empty() {
                    return Err(invalid("bootstrap availability binding name is empty"));
                }
                *module
            }
        };
        let row = self
            .previous
            .world
            .modules
            .get(module.0 as usize)
            .ok_or_else(|| invalid("bootstrap availability proof references an out-of-bounds module"))?;
        if !matches!(&row.identity.project, StableProjectRef::Builtin { namespace, .. } if namespace.as_ref() == "universe") {
            return Err(invalid("bootstrap availability proof targets a non-Universe module"));
        }

        use AvailabilityAuthorityV1 as Authority;
        use AvailabilityProductV1 as Product;
        use AvailabilityTimingV1 as Timing;

        let valid = match (&proof.authority, &proof.product, &proof.available_at, &proof.target) {
            (Authority::SemanticIdentityCatalog, Product::CanonicalIdentity, Timing::BeforeSourceExecution, TargetV1::Module { .. })
            | (Authority::SemanticIdentityCatalog, Product::CanonicalIdentity, Timing::BeforeSourceExecution, TargetV1::Binding { .. })
            | (Authority::NativeModuleMaterialization, Product::RuntimeObject, Timing::BeforeSourceExecution, TargetV1::Module { .. })
            | (Authority::NativeBindingCatalog, Product::RuntimeObject, Timing::BeforeSourceExecution, TargetV1::Binding { .. })
            | (Authority::FullyInitializedUniverse, Product::RuntimeObject, Timing::AfterBootstrap, TargetV1::Module { .. })
            | (Authority::FullyInitializedUniverse, Product::SourceInitializedValue, Timing::AfterBootstrap, TargetV1::Module { .. })
            | (Authority::FullyInitializedUniverse, Product::SourceInitializedValue, Timing::AfterBootstrap, TargetV1::Binding { .. })
            | (Authority::FullyInitializedUniverse, Product::MetadataDescriptor, Timing::AfterBootstrap, TargetV1::Module { .. }) => true,
            _ => false,
        };
        if !valid {
            return Err(invalid("bootstrap availability proof uses an unsupported authority/product/timing/target combination"));
        }

        let requires_bootstrap_membership = proof.authority == Authority::FullyInitializedUniverse
            && matches!(&proof.product, Product::RuntimeObject | Product::SourceInitializedValue);
        if requires_bootstrap_membership && !self.bootstrap_modules.contains(&module) {
            return Err(invalid("fully initialized Universe runtime/source proof targets a module outside the bootstrap schedule"));
        }
        Ok(())
    }
}

impl Wire for ModuleWorldV4 {
    fn collection_limit(limits: &ValidationLimits) -> usize {
        limits.max_modules
    }

    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        self.previous.write(out)?;
        self.bootstrap_modules.write(out)?;
        self.bootstrap_availability.write(out)
    }

    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        input.nested(|input| {
            Ok(Self {
                previous: ModuleWorldV3::read(input)?,
                bootstrap_modules: Vec::<ArtifactModuleId>::read(input)?,
                bootstrap_availability: Vec::<AvailabilityProofV1>::read(input)?,
            })
        })
    }
}

fn availability_target_key(target: &TargetV1) -> (u8, u32, &str) {
    match target {
        TargetV1::Module { module } => (0, module.0, ""),
        TargetV1::Binding { module, name } => (1, module.0, name.as_str()),
    }
}

fn availability_product_tag(product: &AvailabilityProductV1) -> u8 {
    match product {
        AvailabilityProductV1::CanonicalIdentity => 0,
        AvailabilityProductV1::MetadataDescriptor => 1,
        AvailabilityProductV1::RuntimeObject => 2,
        AvailabilityProductV1::SourceInitializedValue => 3,
    }
}

fn availability_authority_tag(authority: &AvailabilityAuthorityV1) -> u8 {
    match authority {
        AvailabilityAuthorityV1::SemanticIdentityCatalog => 0,
        AvailabilityAuthorityV1::NativeBindingCatalog => 1,
        AvailabilityAuthorityV1::NativeModuleMaterialization => 2,
        AvailabilityAuthorityV1::SemanticMetadataBundle => 3,
        AvailabilityAuthorityV1::CanonicalSourceInitializer => 4,
        AvailabilityAuthorityV1::FullyInitializedUniverse => 5,
    }
}

fn availability_proof_key(proof: &AvailabilityProofV1) -> ((u8, u32, &str), u8, u8, u8) {
    (
        availability_target_key(&proof.target),
        availability_product_tag(&proof.product),
        availability_authority_tag(&proof.authority),
        availability_timing_tag(&proof.available_at),
    )
}

fn availability_witness_key(
    witness: &RuntimeAvailabilityWitnessV1,
) -> (u32, u32, u64, u64, u8, Option<(u8, u32, &str)>, (u8, u32, &str), u8, u8, u8, u8, u8) {
    let reason = match &witness.reason {
        RuntimeReasonV1::WholeModuleImport => 0,
        RuntimeReasonV1::SelectiveValueImport => 1,
        RuntimeReasonV1::ReExport => 2,
        RuntimeReasonV1::RuntimeDeclarationReference => 3,
    };
    let producer = match &witness.producer {
        AvailabilityRequirementProducerV1::DeclarationRead => 0,
        AvailabilityRequirementProducerV1::ImportRead => 1,
        AvailabilityRequirementProducerV1::CompilerGeneratedRead => 2,
        AvailabilityRequirementProducerV1::CallableOwner => 3,
        AvailabilityRequirementProducerV1::PatternOwner => 4,
        AvailabilityRequirementProducerV1::ConformanceTypeOwner => 5,
        AvailabilityRequirementProducerV1::ContextualReceiverRead => 6,
        AvailabilityRequirementProducerV1::MetadataTypeOwner => 7,
        AvailabilityRequirementProducerV1::StaticConstantRead => 8,
    };
    (
        witness.importer.0,
        witness.dependency.0,
        witness.range.start,
        witness.range.end,
        reason,
        witness.read_target.as_ref().map(availability_target_key),
        availability_target_key(&witness.target),
        availability_product_tag(&witness.product),
        producer,
        availability_timing_tag(&witness.required_at),
        availability_authority_tag(&witness.authority),
        availability_timing_tag(&witness.available_at),
    )
}

fn availability_timing_tag(timing: &AvailabilityTimingV1) -> u8 {
    match timing {
        AvailabilityTimingV1::BeforeSourceExecution => 0,
        AvailabilityTimingV1::DuringModuleInitializer => 1,
        AvailabilityTimingV1::AfterBootstrap => 2,
    }
}

fn source_import_binding_count(import: &ImportV1) -> Result<u32, ArtifactError> {
    let count = match import {
        ImportV1::Module { .. } => 1,
        ImportV1::Selective { items, .. } | ImportV1::ReExport { items, .. } => {
            u32::try_from(items.len()).map_err(|_| invalid("source import item count exceeds u32"))?
        }
    };
    Ok(count)
}

struct PortableTopologyHasher {
    hasher: ArtifactHasher,
}

impl PortableTopologyHasher {
    fn new() -> Self {
        Self {
            hasher: ArtifactHasher::new_domain("telos.telosc.v2.portable-module-topology"),
        }
    }

    fn u8(&mut self, value: u8) {
        self.hasher.write_u8(value);
    }

    fn u32(&mut self, value: u32) {
        self.hasher.write_u32(value);
    }

    fn count(&mut self, value: usize) -> Result<(), ArtifactError> {
        self.u32(u32::try_from(value).map_err(|_| invalid("portable topology collection exceeds u32"))?);
        Ok(())
    }

    fn text(&mut self, value: &str) -> Result<(), ArtifactError> {
        self.count(value.len())?;
        self.hasher.write(value.as_bytes());
        Ok(())
    }

    fn project(&mut self, project: &StableProjectRef) -> Result<(), ArtifactError> {
        match project {
            StableProjectRef::Builtin { namespace, version } => {
                self.u8(0);
                self.text(namespace)?;
                self.text(version)?;
            }
            StableProjectRef::Package {
                package,
                version,
                artifact_fingerprint,
            } => {
                self.u8(1);
                self.text(package)?;
                self.text(version)?;
                self.hasher.write(artifact_fingerprint.as_bytes());
            }
            StableProjectRef::SourceArtifact {
                logical_uri,
                source_fingerprint,
            } => {
                self.u8(2);
                self.text(logical_uri)?;
                self.hasher.write(source_fingerprint.as_bytes());
            }
            StableProjectRef::Session { .. } => return Err(invalid("session project identity in portable topology")),
        }
        Ok(())
    }

    fn module_identity(&mut self, module: &StableModuleRef) -> Result<(), ArtifactError> {
        self.project(&module.project)?;
        self.count(module.path.len())?;
        for segment in &module.path {
            self.text(segment)?;
        }
        Ok(())
    }

    fn module_id(&mut self, modules: &[ModuleV2], id: ArtifactModuleId) -> Result<(), ArtifactError> {
        let module = modules.get(id.0 as usize).ok_or_else(|| invalid("module link references missing module"))?;
        self.module_identity(&module.identity)
    }

    fn target_link(&mut self, modules: &[ModuleV2], target: &TargetV1) -> Result<(), ArtifactError> {
        match target {
            TargetV1::Module { module } => {
                self.u8(0);
                self.module_id(modules, *module)?;
            }
            TargetV1::Binding { module, name } => {
                self.u8(1);
                self.module_id(modules, *module)?;
                self.text(name)?;
            }
        }
        Ok(())
    }

    fn semantic_node(&mut self, modules: &[ModuleV2], node: &SemanticNodeV1) -> Result<(), ArtifactError> {
        match node {
            SemanticNodeV1::Module { module } => {
                self.u8(0);
                self.module_id(modules, *module)?;
            }
            SemanticNodeV1::Declaration { module, name } => {
                self.u8(1);
                self.module_id(modules, *module)?;
                self.text(name)?;
            }
        }
        Ok(())
    }

    fn finish(self) -> u64 {
        let digest = self.hasher.finalize();
        u64::from_le_bytes(digest.as_bytes()[..8].try_into().expect("digest prefix is eight bytes"))
    }
}

pub(crate) use record;
pub(crate) use tagged;
