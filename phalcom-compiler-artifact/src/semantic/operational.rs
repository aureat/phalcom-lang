//! Source-independent operational template DTOs for checked static definitions (`PERF003.C2.P7.T7.B`).

use crate::module_world::{Decode, Wire, invalid, record, tagged};
use crate::semantic::static_value::ArtifactStaticValueId;
use crate::semantic::type_store::ArtifactTypeId;
use crate::{ArtifactError, BinaryReader, BinaryWriter, ValidationError, ValidationLimits};
use phalcom_type_meta::allocation::AllocationBudget;
use phalcom_type_meta::identity::StableDeclarationRef;

pub const OPERATIONAL_TEMPLATE_SCHEMA_V1: u32 = 1;

tagged!(StaticBinaryOpV1 {
    0 => Add,
    1 => Sub,
    2 => Mul,
    3 => Div,
    4 => Mod,
    5 => BitAnd,
    6 => BitOr,
    7 => BitXor,
    8 => Shl,
    9 => Shr,
    10 => Eq,
    11 => NotEq,
    12 => Less,
    13 => LessEq,
    14 => Greater,
    15 => GreaterEq,
    16 => And,
    17 => Or,
});

tagged!(StaticUnaryOpV1 {
    0 => Neg,
    1 => Not,
    2 => BitNot,
});

tagged!(OperationalTargetV1 {
    0 => Declaration { target: StableDeclarationRef },
    1 => DataConstructor { owner: StableDeclarationRef },
    2 => VariantConstructor { owner: StableDeclarationRef, selector: Box<str> },
});

tagged!(OperationalOpV1 limit max_semantic_entries_per_family {
    0 => Constant { value: ArtifactStaticValueId },
    1 => ReadParameter { index: u32 },
    2 => ReadLocal { slot: u32 },
    3 => WriteLocal { slot: u32, value: Box<OperationalOpV1> },
    4 => TypeForm { ty: ArtifactTypeId },
    5 => Binary { op: StaticBinaryOpV1, left: Box<OperationalOpV1>, right: Box<OperationalOpV1> },
    6 => Unary { op: StaticUnaryOpV1, operand: Box<OperationalOpV1> },
    7 => StaticApplication { target: StableDeclarationRef, arguments: Vec<OperationalOpV1> },
    8 => StaticConstruction { target: OperationalTargetV1, result_type: ArtifactTypeId, arguments: Vec<OperationalOpV1> },
    9 => MemberProjection { target: Box<OperationalOpV1>, member: Box<str> },
    10 => If { condition: Box<OperationalOpV1>, then_branch: Box<OperationalOpV1>, else_branch: Option<Box<OperationalOpV1>> },
    11 => Sequence { result_type: ArtifactTypeId, elements: Vec<OperationalOpV1> },
    12 => Tuple { elements: Vec<OperationalOpV1> },
    13 => Block { local_count: u32, statements: Vec<OperationalOpV1>, result: Option<Box<OperationalOpV1>> },
    14 => Return { value: Option<Box<OperationalOpV1>> },
});

record!(OperationalTemplateV1 {
    owner: StableDeclarationRef,
    parameter_count: u32,
    local_count: u32,
    root: OperationalOpV1,
});

record!(CheckedStaticDefinitionsV1 {
    definitions: Vec<OperationalTemplateV1>,
});

impl CheckedStaticDefinitionsV1 {
    pub fn encode(&self) -> Result<Vec<u8>, ArtifactError> {
        self.validate(&ValidationLimits::default())?;
        let mut out = BinaryWriter::new();
        out.write_u32(OPERATIONAL_TEMPLATE_SCHEMA_V1);
        self.write(&mut out)?;
        Ok(out.into_vec())
    }

    pub fn decode(bytes: &[u8], limits: &ValidationLimits) -> Result<Self, ArtifactError> {
        if bytes.len() as u64 > limits.max_decoded_section_bytes {
            return Err(invalid("operational template byte budget exceeded"));
        }
        let allocation_limit = usize::try_from(limits.max_decoded_section_bytes)
            .map_err(|_| invalid("operational template allocation limit exceeds address space"))?;
        let mut allocations = AllocationBudget::new(allocation_limit);
        let mut input = Decode {
            reader: BinaryReader::new(bytes),
            limits: *limits,
            allocations: &mut allocations,
            items_left: limits.max_semantic_entries_per_family,
            depth: 0,
        };
        if input.reader.read_u32()? != OPERATIONAL_TEMPLATE_SCHEMA_V1 {
            return Err(invalid("unsupported operational template schema"));
        }
        let definitions = Self::read(&mut input)?;
        if input.reader.has_remaining() {
            return Err(invalid("trailing operational template bytes"));
        }
        definitions.validate(limits)?;
        Ok(definitions)
    }

    pub fn validate(&self, limits: &ValidationLimits) -> Result<(), ArtifactError> {
        if self.definitions.len() > limits.max_semantic_entries_per_family {
            return Err(invalid("operational template count budget exceeded"));
        }
        for def in &self.definitions {
            validate_owner(&def.owner, limits)?;
            validate_op(&def.root, def.parameter_count, def.local_count, 0, limits)?;
        }
        Ok(())
    }
}

fn validate_op(
    op: &OperationalOpV1,
    param_count: u32,
    local_count: u32,
    depth: usize,
    limits: &ValidationLimits,
) -> Result<(), ArtifactError> {
    if depth > limits.max_recursion_depth {
        return Err(invalid("operational template recursion depth exceeded"));
    }
    match op {
        OperationalOpV1::Constant { .. } => {}
        OperationalOpV1::ReadParameter { index } => {
            if *index >= param_count {
                return Err(invalid("operational parameter read out of range"));
            }
        }
        OperationalOpV1::ReadLocal { slot } => {
            if *slot >= local_count {
                return Err(invalid("operational local slot read out of range"));
            }
        }
        OperationalOpV1::WriteLocal { slot, value } => {
            if *slot >= local_count {
                return Err(invalid("operational local slot write out of range"));
            }
            validate_op(value, param_count, local_count, depth + 1, limits)?;
        }
        OperationalOpV1::TypeForm { .. } => {}
        OperationalOpV1::Binary { left, right, .. } => {
            validate_op(left, param_count, local_count, depth + 1, limits)?;
            validate_op(right, param_count, local_count, depth + 1, limits)?;
        }
        OperationalOpV1::Unary { operand, .. } => {
            validate_op(operand, param_count, local_count, depth + 1, limits)?;
        }
        OperationalOpV1::StaticApplication { target, arguments } => {
            validate_owner(target, limits)?;
            if arguments.len() > limits.max_semantic_entries_per_family {
                return Err(invalid("static application argument budget exceeded"));
            }
            for arg in arguments {
                validate_op(arg, param_count, local_count, depth + 1, limits)?;
            }
        }
        OperationalOpV1::StaticConstruction { target, arguments, .. } => {
            match target {
                OperationalTargetV1::Declaration { target } => validate_owner(target, limits)?,
                OperationalTargetV1::DataConstructor { owner } => validate_owner(owner, limits)?,
                OperationalTargetV1::VariantConstructor { owner, selector } => {
                    validate_owner(owner, limits)?;
                    if selector.is_empty() || selector.len() > limits.max_string_bytes {
                        return Err(invalid("invalid constructor selector"));
                    }
                }
            }
            if arguments.len() > limits.max_semantic_entries_per_family {
                return Err(invalid("static construction argument budget exceeded"));
            }
            for arg in arguments {
                validate_op(arg, param_count, local_count, depth + 1, limits)?;
            }
        }
        OperationalOpV1::MemberProjection { target, member } => {
            if member.is_empty() || member.len() > limits.max_string_bytes {
                return Err(invalid("invalid member projection name"));
            }
            validate_op(target, param_count, local_count, depth + 1, limits)?;
        }
        OperationalOpV1::If { condition, then_branch, else_branch } => {
            validate_op(condition, param_count, local_count, depth + 1, limits)?;
            validate_op(then_branch, param_count, local_count, depth + 1, limits)?;
            if let Some(eb) = else_branch {
                validate_op(eb, param_count, local_count, depth + 1, limits)?;
            }
        }
        OperationalOpV1::Sequence { elements, .. } => {
            if elements.len() > limits.max_semantic_entries_per_family {
                return Err(invalid("sequence element budget exceeded"));
            }
            for el in elements {
                validate_op(el, param_count, local_count, depth + 1, limits)?;
            }
        }
        OperationalOpV1::Tuple { elements } => {
            if elements.len() > limits.max_semantic_entries_per_family {
                return Err(invalid("tuple element budget exceeded"));
            }
            for el in elements {
                validate_op(el, param_count, local_count, depth + 1, limits)?;
            }
        }
        OperationalOpV1::Block { local_count: block_locals, statements, result } => {
            let total_locals = local_count.checked_add(*block_locals).ok_or_else(|| invalid("local count overflow"))?;
            for s in statements {
                validate_op(s, param_count, total_locals, depth + 1, limits)?;
            }
            if let Some(res) = result {
                validate_op(res, param_count, total_locals, depth + 1, limits)?;
            }
        }
        OperationalOpV1::Return { value } => {
            if let Some(val) = value {
                validate_op(val, param_count, local_count, depth + 1, limits)?;
            }
        }
    }
    Ok(())
}

fn validate_owner(value: &StableDeclarationRef, limits: &ValidationLimits) -> Result<(), ArtifactError> {
    match &value.module.project {
        phalcom_type_meta::identity::StableProjectRef::Builtin { namespace, version } => {
            if namespace.is_empty() || version.is_empty() {
                return Err(invalid("empty namespace or version in structural owner"));
            }
            if namespace.len() > limits.max_string_bytes || version.len() > limits.max_string_bytes {
                return Err(invalid("structural owner identity exceeds string budget"));
            }
        }
        _ => {
            return Err(ValidationError::ReferentialIntegrity("structural owner is not a builtin Universe identity".into()).into());
        }
    }
    if value.module.path.is_empty() || value.path.is_empty() {
        return Err(invalid("structural owner identity has an empty path"));
    }
    for segment in value.module.path.iter().chain(value.path.iter()) {
        if segment.is_empty() || segment.len() > limits.max_string_bytes {
            return Err(invalid("empty or oversized structural owner path segment"));
        }
    }
    Ok(())
}
