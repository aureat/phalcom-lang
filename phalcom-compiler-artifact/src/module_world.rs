//! Portable, source-free module authority. Every enum tag is explicit wire ABI.
use crate::{ArtifactError, BinaryReader, BinaryWriter, CodecError, ValidationLimits};
use phalcom_type_meta::Fingerprint128;
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};
use phalcom_type_meta::identity::{
    StableAssociatedStaticDomainDeclarationRef, StableCallableOwnerRef, StableCallableRef, StableDeclarationRef, StableDispatchSide, StableFieldRef,
    StableImplRef, StableModuleRef, StableProjectRef, StableStaticConstantRef, StableStaticDomainDeclarationRef, StableTypeMemberRef, StableVariantRef,
};

pub const MODULE_WORLD_SCHEMA_V1: u32 = 1;
pub const MODULE_WORLD_SCHEMA_V2: u32 = 2;

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
    pub fn encode(&self) -> Result<Vec<u8>, ArtifactError> {
        let mut out = BinaryWriter::new();
        out.write_u32(MODULE_WORLD_SCHEMA_V2);
        self.write(&mut out)?;
        Ok(out.into_vec())
    }

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
            if matches!(project, StableProjectRef::Session { .. }) {
                return Err(invalid("session project identity in external_projects is not permitted"));
            }
        }

        for (mod_idx, module) in self.modules.iter().enumerate() {
            if matches!(module.identity.project, StableProjectRef::Session { .. }) {
                return Err(invalid("session project identity in module identity is not permitted"));
            }
            if module.path.len() != module.identity.path.len()
                || !module.path.iter().zip(module.identity.path.iter()).all(|(a, b)| a.as_str() == b.as_ref())
            {
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

pub(crate) use record;
pub(crate) use tagged;
