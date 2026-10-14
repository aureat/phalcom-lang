//! Portable, source-free module authority. Every enum tag is explicit wire ABI.
use crate::{ArtifactError, BinaryReader, BinaryWriter, CodecError, ValidationLimits};

pub const MODULE_WORLD_SCHEMA_V1: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ArtifactModuleId(pub u32);

/// Decoder budgets are cumulative across the section, including nested metadata.
pub(crate) struct Decode<'a> {
    pub(crate) reader: BinaryReader<'a>,
    pub(crate) limits: ValidationLimits,
    pub(crate) allocations_left: u64,
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
    pub(crate) fn allocate(&mut self, bytes: u64) -> Result<(), ArtifactError> {
        self.allocations_left = self
            .allocations_left
            .checked_sub(bytes)
            .ok_or_else(|| invalid("module payload allocation budget exceeded"))?;
        Ok(())
    }
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
        input.allocate(text.len() as u64)?;
        Ok(text.to_owned())
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
            let bytes = (count as u64)
                .checked_mul(std::mem::size_of::<T>() as u64)
                .ok_or_else(|| invalid("module payload allocation overflow"))?;
            input.allocate(bytes)?;
            let mut values = Vec::new();
            values.try_reserve_exact(count).map_err(|_| invalid("module payload allocation failed"))?;
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
        T::read(input).map(Box::new)
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
    pub fn decode(bytes: &[u8], limits: &ValidationLimits) -> Result<Self, ArtifactError> {
        if bytes.len() as u64 > limits.max_decoded_section_bytes {
            return Err(invalid("module section exceeds decoded byte limit"));
        }
        let mut input = Decode {
            reader: BinaryReader::new(bytes),
            limits: *limits,
            allocations_left: limits.max_decoded_section_bytes,
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
        if world.modules.len() > limits.max_modules {
            return Err(invalid("module count exceeds limit"));
        }
        Ok(world)
    }
}

pub(crate) use record;
pub(crate) use tagged;
