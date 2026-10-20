//! Portable executable prototype rows. References are decoded structurally;
//! cross-table and control-flow safety belongs to full-image validation.

use crate::executable::instruction::PortableInstructionV1;
use crate::module_world::{Decode, SpanV1, Wire};
use crate::{ArtifactError, BinaryReader, BinaryWriter, CodecError, ValidationLimits};
use phalcom_type_meta::allocation::AllocationBudget;
use phalcom_type_meta::identity::{
    StableCallableOwnerRef, StableCallableRef, StableDeclarationRef, StableImplRef, StableModuleRef, StableProjectRef, StableVariantRef,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrototypeId(pub u32);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConstantId(pub u32);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceMapId(pub u32);
/// Index into the artifact-local executable semantic-pool table.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SemanticPoolId(pub u32);

macro_rules! id_wire {
    ($ty:ty) => {
        impl Wire for $ty {
            fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
                out.write_u32(self.0);
                Ok(())
            }
            fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
                Ok(Self(input.reader.read_u32()?))
            }
        }
    };
}
id_wire!(PrototypeId);
id_wire!(ConstantId);
id_wire!(SourceMapId);
id_wire!(SemanticPoolId);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrototypeOwnerV1 {
    ModuleInitializer { module: StableModuleRef },
    Callable { callable: StableCallableRef },
    Nested { parent: PrototypeId, ordinal: u32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UpvalueCaptureV1 {
    Local(u32),
    Upvalue(u32),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RestKindV1 {
    Positional,
    Labeled,
    Split,
    Complete,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParameterShapeV1 {
    pub fixed_positionals: u32,
    pub fixed_labels: Vec<String>,
    pub rest: Option<RestKindV1>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortablePrototypeV1 {
    pub owner: PrototypeOwnerV1,
    pub code: Vec<PortableInstructionV1>,
    pub constants: Vec<ConstantId>,
    pub max_slots: u32,
    pub arity: u32,
    pub upvalues: Vec<UpvalueCaptureV1>,
    pub parameters: ParameterShapeV1,
    pub name: String,
    pub local_names: Vec<String>,
    pub source: SourceMapId,
    pub semantics: SemanticPoolId,
    pub spans: Vec<SpanV1>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutablePrototypeLimitsV1 {
    pub max_code: usize,
    pub max_constants: usize,
    pub max_upvalues: usize,
    pub max_labels: usize,
    pub max_local_names: usize,
    pub max_spans: usize,
    pub max_frame_slots: u32,
    pub max_string_bytes: usize,
}
impl Default for ExecutablePrototypeLimitsV1 {
    fn default() -> Self {
        Self {
            max_code: 1_000_000,
            max_constants: 1_000_000,
            max_upvalues: 65_536,
            max_labels: 65_536,
            max_local_names: 65_536,
            max_spans: 1_000_000,
            max_frame_slots: 65_536,
            max_string_bytes: 16 * 1024 * 1024,
        }
    }
}

impl Wire for RestKindV1 {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        out.write_u8(match self {
            Self::Positional => 0,
            Self::Labeled => 1,
            Self::Split => 2,
            Self::Complete => 3,
        });
        Ok(())
    }
    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        match input.reader.read_u8()? {
            0 => Ok(Self::Positional),
            1 => Ok(Self::Labeled),
            2 => Ok(Self::Split),
            3 => Ok(Self::Complete),
            tag => Err(CodecError::InvalidTag {
                type_name: "RestKindV1",
                tag: u32::from(tag),
            }
            .into()),
        }
    }
}
impl Wire for UpvalueCaptureV1 {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        match self {
            Self::Local(index) => {
                out.write_u8(0);
                out.write_u32(*index);
            }
            Self::Upvalue(index) => {
                out.write_u8(1);
                out.write_u32(*index);
            }
        }
        Ok(())
    }
    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        match input.reader.read_u8()? {
            0 => Ok(Self::Local(input.reader.read_u32()?)),
            1 => Ok(Self::Upvalue(input.reader.read_u32()?)),
            tag => Err(CodecError::InvalidTag {
                type_name: "UpvalueCaptureV1",
                tag: u32::from(tag),
            }
            .into()),
        }
    }
}
impl Wire for ParameterShapeV1 {
    fn write(&self, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
        self.fixed_positionals.write(out)?;
        self.fixed_labels.write(out)?;
        match self.rest {
            None => 0u8.write(out),
            Some(rest) => {
                1u8.write(out)?;
                rest.write(out)
            }
        }
    }
    fn read(input: &mut Decode<'_>) -> Result<Self, ArtifactError> {
        Ok(Self {
            fixed_positionals: u32::read(input)?,
            fixed_labels: Vec::<String>::read(input)?,
            rest: match u8::read(input)? {
                0 => None,
                1 => Some(RestKindV1::read(input)?),
                tag => {
                    return Err(CodecError::InvalidTag {
                        type_name: "ParameterShapeV1.rest",
                        tag: u32::from(tag),
                    }
                    .into());
                }
            },
        })
    }
}

fn invalid(message: &'static str) -> ArtifactError {
    ArtifactError::LimitExceeded(message.into())
}
fn count(reader: &mut BinaryReader<'_>, cap: usize, minimum_item_bytes: usize, item: &'static str) -> Result<usize, ArtifactError> {
    let n = reader.read_u32()? as usize;
    if n > cap {
        return Err(CodecError::LimitExceeded { item, count: n, limit: cap }.into());
    }
    if n.checked_mul(minimum_item_bytes).map_or(true, |bytes| bytes > reader.remaining()) {
        return Err(CodecError::UnexpectedEof.into());
    }
    Ok(n)
}
fn read_string(input: &mut Decode<'_>, cap: usize) -> Result<String, ArtifactError> {
    let s = input.reader.read_str(cap.min(input.limits.max_string_bytes))?;
    Ok(input.allocations.string(s)?)
}
fn write_string(s: &str, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
    out.write_u32(u32::try_from(s.len()).map_err(|_| invalid("prototype string exceeds wire limit"))?);
    out.write_bytes(s.as_bytes());
    Ok(())
}

impl PortablePrototypeV1 {
    pub fn encode(&self) -> Result<Vec<u8>, ArtifactError> {
        let caps = ExecutablePrototypeLimitsV1::default();
        validate_shape(self, &caps)?;
        validate_owner_string_limits(&self.owner, caps.max_string_bytes)?;
        let mut out = BinaryWriter::new();
        match &self.owner {
            PrototypeOwnerV1::ModuleInitializer { module } => {
                out.write_u8(0);
                module.write(&mut out)?;
            }
            PrototypeOwnerV1::Callable { callable } => {
                out.write_u8(1);
                callable.write(&mut out)?;
            }
            PrototypeOwnerV1::Nested { parent, ordinal } => {
                out.write_u8(2);
                out.write_u32(parent.0);
                out.write_u32(*ordinal);
            }
        }
        put_count(self.code.len(), &mut out)?;
        for op in &self.code {
            op.write_to(&mut out);
        }
        put_count(self.constants.len(), &mut out)?;
        for id in &self.constants {
            out.write_u32(id.0);
        }
        out.write_u32(self.max_slots);
        out.write_u32(self.arity);
        put_count(self.upvalues.len(), &mut out)?;
        for capture in &self.upvalues {
            match capture {
                UpvalueCaptureV1::Local(n) => {
                    out.write_u8(0);
                    out.write_u32(*n);
                }
                UpvalueCaptureV1::Upvalue(n) => {
                    out.write_u8(1);
                    out.write_u32(*n);
                }
            }
        }
        out.write_u32(self.parameters.fixed_positionals);
        put_count(self.parameters.fixed_labels.len(), &mut out)?;
        for label in &self.parameters.fixed_labels {
            write_string(label, &mut out)?;
        }
        match self.parameters.rest {
            None => out.write_u8(0),
            Some(rest) => {
                out.write_u8(1);
                rest.write(&mut out)?;
            }
        }
        write_string(&self.name, &mut out)?;
        put_count(self.local_names.len(), &mut out)?;
        for name in &self.local_names {
            write_string(name, &mut out)?;
        }
        out.write_u32(self.source.0);
        out.write_u32(self.semantics.0);
        put_count(self.spans.len(), &mut out)?;
        for span in &self.spans {
            out.write_u64(span.start);
            out.write_u64(span.end);
        }
        Ok(out.into_vec())
    }

    /// Decodes one standalone row transactionally. A containing image must
    /// still validate prototype/constant/source-map domains and capture edges.
    pub fn decode(bytes: &[u8], common: &ValidationLimits, caps: &ExecutablePrototypeLimitsV1, budget: &mut AllocationBudget) -> Result<Self, ArtifactError> {
        let mut transaction = budget.clone();
        let result = decode_inner(bytes, common, caps, &mut transaction)?;
        *budget = transaction;
        Ok(result)
    }
}
fn put_count(n: usize, out: &mut BinaryWriter) -> Result<(), ArtifactError> {
    out.write_u32(u32::try_from(n).map_err(|_| invalid("prototype count exceeds wire limit"))?);
    Ok(())
}
fn decode_inner(
    bytes: &[u8],
    common: &ValidationLimits,
    caps: &ExecutablePrototypeLimitsV1,
    budget: &mut AllocationBudget,
) -> Result<PortablePrototypeV1, ArtifactError> {
    let mut decode_limits = *common;
    decode_limits.max_string_bytes = decode_limits.max_string_bytes.min(caps.max_string_bytes);
    let mut input = Decode {
        reader: BinaryReader::new(bytes),
        limits: decode_limits,
        allocations: budget,
        items_left: usize::MAX,
        depth: 0,
    };
    let owner = match input.reader.read_u8()? {
        0 => PrototypeOwnerV1::ModuleInitializer {
            module: StableModuleRef::read(&mut input)?,
        },
        1 => PrototypeOwnerV1::Callable {
            callable: StableCallableRef::read(&mut input)?,
        },
        2 => PrototypeOwnerV1::Nested {
            parent: PrototypeId(input.reader.read_u32()?),
            ordinal: input.reader.read_u32()?,
        },
        tag => {
            return Err(CodecError::InvalidTag {
                type_name: "PrototypeOwnerV1",
                tag: u32::from(tag),
            }
            .into());
        }
    };
    let code_n = count(&mut input.reader, caps.max_code, 2, "prototype code")?;
    let mut code = input.allocations.vector(code_n)?;
    for _ in 0..code_n {
        code.push(PortableInstructionV1::read_from(&mut input.reader)?);
    }
    let const_n = count(&mut input.reader, caps.max_constants, 4, "prototype constants")?;
    let mut constants = input.allocations.vector(const_n)?;
    for _ in 0..const_n {
        constants.push(ConstantId(input.reader.read_u32()?));
    }
    let max_slots = input.reader.read_u32()?;
    let arity = input.reader.read_u32()?;
    let up_n = count(&mut input.reader, caps.max_upvalues, 5, "prototype upvalues")?;
    let mut upvalues = input.allocations.vector(up_n)?;
    for _ in 0..up_n {
        upvalues.push(match input.reader.read_u8()? {
            0 => UpvalueCaptureV1::Local(input.reader.read_u32()?),
            1 => UpvalueCaptureV1::Upvalue(input.reader.read_u32()?),
            tag => {
                return Err(CodecError::InvalidTag {
                    type_name: "UpvalueCaptureV1",
                    tag: u32::from(tag),
                }
                .into());
            }
        });
    }
    let fixed_positionals = input.reader.read_u32()?;
    let labels_n = count(&mut input.reader, caps.max_labels, 4, "prototype labels")?;
    let mut fixed_labels = input.allocations.vector(labels_n)?;
    for _ in 0..labels_n {
        fixed_labels.push(read_string(&mut input, caps.max_string_bytes)?);
    }
    let rest = match input.reader.read_u8()? {
        0 => None,
        1 => Some(RestKindV1::read(&mut input)?),
        tag => {
            return Err(CodecError::InvalidTag {
                type_name: "RestKindV1",
                tag: u32::from(tag),
            }
            .into());
        }
    };
    let name = read_string(&mut input, caps.max_string_bytes)?;
    let locals_n = count(&mut input.reader, caps.max_local_names, 4, "prototype local names")?;
    let mut local_names = input.allocations.vector(locals_n)?;
    for _ in 0..locals_n {
        local_names.push(read_string(&mut input, caps.max_string_bytes)?);
    }
    let source = SourceMapId(input.reader.read_u32()?);
    let semantics = SemanticPoolId(input.reader.read_u32()?);
    let spans_n = count(&mut input.reader, caps.max_spans, 16, "prototype spans")?;
    let mut spans = input.allocations.vector(spans_n)?;
    for _ in 0..spans_n {
        spans.push(SpanV1 {
            start: input.reader.read_u64()?,
            end: input.reader.read_u64()?,
        });
    }
    if input.reader.has_remaining() {
        return Err(CodecError::CorruptedData("trailing prototype bytes".into()).into());
    }
    let row = PortablePrototypeV1 {
        owner,
        code,
        constants,
        max_slots,
        arity,
        upvalues,
        parameters: ParameterShapeV1 {
            fixed_positionals,
            fixed_labels,
            rest,
        },
        name,
        local_names,
        source,
        semantics,
        spans,
    };
    validate_shape(&row, caps)?;
    Ok(row)
}
fn validate_shape(row: &PortablePrototypeV1, caps: &ExecutablePrototypeLimitsV1) -> Result<(), ArtifactError> {
    if row.code.len() > caps.max_code
        || row.constants.len() > caps.max_constants
        || row.upvalues.len() > caps.max_upvalues
        || row.parameters.fixed_labels.len() > caps.max_labels
        || row.local_names.len() > caps.max_local_names
        || row.spans.len() > caps.max_spans
    {
        return Err(invalid("prototype count exceeds executable limit"));
    }
    if row.max_slots > caps.max_frame_slots || row.max_slots > 65_536 {
        return Err(invalid("prototype frame exceeds 65536 slots"));
    }
    if row.spans.len() != row.code.len() {
        return Err(invalid("prototype spans must parallel code"));
    }
    if row.spans.iter().any(|s| s.start > s.end) {
        return Err(invalid("prototype span has reversed range"));
    }
    if row.name.len() > caps.max_string_bytes
        || row
            .local_names
            .iter()
            .chain(&row.parameters.fixed_labels)
            .any(|s| s.len() > caps.max_string_bytes)
    {
        return Err(invalid("prototype string exceeds executable limit"));
    }
    Ok(())
}

fn validate_owner_string_limits(owner: &PrototypeOwnerV1, cap: usize) -> Result<(), ArtifactError> {
    fn check(value: &str, cap: usize) -> Result<(), ArtifactError> {
        if value.len() > cap {
            Err(invalid("prototype owner string exceeds executable limit"))
        } else {
            Ok(())
        }
    }
    fn project(value: &StableProjectRef, cap: usize) -> Result<(), ArtifactError> {
        match value {
            StableProjectRef::Builtin { namespace, version } => {
                check(namespace, cap)?;
                check(version, cap)
            }
            StableProjectRef::Package { package, version, .. } => {
                check(package, cap)?;
                check(version, cap)
            }
            StableProjectRef::SourceArtifact { logical_uri, .. } => check(logical_uri, cap),
            StableProjectRef::Session { .. } => Ok(()),
        }
    }
    fn module(value: &StableModuleRef, cap: usize) -> Result<(), ArtifactError> {
        project(&value.project, cap)?;
        for segment in value.path.iter() {
            check(segment, cap)?;
        }
        Ok(())
    }
    fn declaration(value: &StableDeclarationRef, cap: usize) -> Result<(), ArtifactError> {
        module(&value.module, cap)?;
        for segment in value.path.iter() {
            check(segment, cap)?;
        }
        Ok(())
    }
    fn variant(value: &StableVariantRef, cap: usize) -> Result<(), ArtifactError> {
        declaration(&value.owner, cap)?;
        check(&value.selector, cap)
    }
    fn implementation(value: &StableImplRef, cap: usize) -> Result<(), ArtifactError> {
        module(&value.module, cap)
    }
    fn callable_owner(value: &StableCallableOwnerRef, cap: usize) -> Result<(), ArtifactError> {
        match value {
            StableCallableOwnerRef::Declaration(value) => declaration(value, cap),
            StableCallableOwnerRef::Variant(value) => variant(value, cap),
            StableCallableOwnerRef::Conformance(value) => implementation(value, cap),
        }
    }
    fn callable(value: &StableCallableRef, cap: usize) -> Result<(), ArtifactError> {
        callable_owner(&value.owner, cap)?;
        check(&value.selector, cap)
    }

    match owner {
        PrototypeOwnerV1::ModuleInitializer { module: value } => module(value, cap),
        PrototypeOwnerV1::Callable { callable: value } => callable(value, cap),
        PrototypeOwnerV1::Nested { .. } => Ok(()),
    }
}
