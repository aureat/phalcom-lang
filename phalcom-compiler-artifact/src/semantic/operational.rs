//! Source-independent operational template DTOs for checked static definitions (`PERF003.C2.P7.T7.B`).

use crate::module_world::{Decode, Wire, invalid, tagged};
use crate::semantic::static_value::ArtifactStaticValueId;
use crate::semantic::type_store::ArtifactTypeId;
use crate::{ArtifactError, BinaryWriter, ValidationLimits};
use phalcom_type_meta::identity::{StableDeclarationRef, StableModuleRef, StableProjectRef};

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
    7 => StaticApplication { application_ordinal: u32, arguments: Vec<OperationalOpV1> },
    8 => StaticConstruction { target: OperationalTargetV1, result_type: ArtifactTypeId, arguments: Vec<OperationalOpV1> },
    9 => MemberProjection { target: Box<OperationalOpV1>, member: Box<str> },
    10 => If { condition: Box<OperationalOpV1>, then_branch: Box<OperationalOpV1>, else_branch: Option<Box<OperationalOpV1>> },
    11 => Sequence { result_type: ArtifactTypeId, elements: Vec<OperationalOpV1> },
    12 => Tuple { elements: Vec<OperationalOpV1> },
    13 => Block { local_count: u32, statements: Vec<OperationalOpV1>, result: Option<Box<OperationalOpV1>> },
    14 => Return { value: Option<Box<OperationalOpV1>> },
    15 => ReadStaticConstant { module: StableModuleRef, name: String },
});

pub(crate) fn validate_op(
    op: &OperationalOpV1,
    param_count: u32,
    local_count: u32,
    applications: &[super::world::CanonicalCheckedStaticApplicationV1],
    depth: usize,
    limits: &ValidationLimits,
) -> Result<(), ArtifactError> {
    if depth > limits.max_recursion_depth {
        return Err(invalid("operational template recursion depth exceeded"));
    }
    match op {
        OperationalOpV1::Constant { .. } => {}
        OperationalOpV1::ReadStaticConstant { module, name } => {
            validate_module(module, limits)?;
            if name.is_empty() || name.len() > limits.max_string_bytes {
                return Err(invalid("invalid static constant identity"));
            }
        }
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
            validate_op(value, param_count, local_count, applications, depth + 1, limits)?;
        }
        OperationalOpV1::TypeForm { .. } => {}
        OperationalOpV1::Binary { left, right, .. } => {
            validate_op(left, param_count, local_count, applications, depth + 1, limits)?;
            validate_op(right, param_count, local_count, applications, depth + 1, limits)?;
        }
        OperationalOpV1::Unary { operand, .. } => {
            validate_op(operand, param_count, local_count, applications, depth + 1, limits)?;
        }
        OperationalOpV1::StaticApplication {
            application_ordinal,
            arguments,
        } => {
            if arguments.len() > limits.max_semantic_entries_per_family {
                return Err(invalid("static application argument budget exceeded"));
            }
            let row_index = usize::try_from(*application_ordinal).map_err(|_| invalid("static application ordinal exceeds address space"))?;
            let row = applications
                .get(row_index)
                .ok_or_else(|| invalid("operational static application references an absent checked row"))?;
            if row.ordinal != *application_ordinal || row.arguments.len() != arguments.len() {
                return Err(invalid("operational static application disagrees with its checked row"));
            }
            for (expected, actual) in row.arguments.iter().zip(arguments) {
                let agrees = match (expected, actual) {
                    (super::world::CanonicalStaticApplicationArgumentV1::KindedTerm { term, .. }, OperationalOpV1::TypeForm { ty }) => term == ty,
                    (
                        super::world::CanonicalStaticApplicationArgumentV1::KindedExpression { application_ordinal, .. },
                        OperationalOpV1::StaticApplication {
                            application_ordinal: actual_ordinal,
                            ..
                        },
                    ) => application_ordinal == actual_ordinal,
                    (super::world::CanonicalStaticApplicationArgumentV1::Value { .. }, _) => true,
                    _ => false,
                };
                if !agrees {
                    return Err(invalid("operational static argument disagrees with its checked row"));
                }
            }
            for arg in arguments {
                validate_op(arg, param_count, local_count, applications, depth + 1, limits)?;
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
                validate_op(arg, param_count, local_count, applications, depth + 1, limits)?;
            }
        }
        OperationalOpV1::MemberProjection { target, member } => {
            if member.is_empty() || member.len() > limits.max_string_bytes {
                return Err(invalid("invalid member projection name"));
            }
            validate_op(target, param_count, local_count, applications, depth + 1, limits)?;
        }
        OperationalOpV1::If {
            condition,
            then_branch,
            else_branch,
        } => {
            validate_op(condition, param_count, local_count, applications, depth + 1, limits)?;
            validate_op(then_branch, param_count, local_count, applications, depth + 1, limits)?;
            if let Some(eb) = else_branch {
                validate_op(eb, param_count, local_count, applications, depth + 1, limits)?;
            }
        }
        OperationalOpV1::Sequence { elements, .. } => {
            if elements.len() > limits.max_semantic_entries_per_family {
                return Err(invalid("sequence element budget exceeded"));
            }
            for el in elements {
                validate_op(el, param_count, local_count, applications, depth + 1, limits)?;
            }
        }
        OperationalOpV1::Tuple { elements } => {
            if elements.len() > limits.max_semantic_entries_per_family {
                return Err(invalid("tuple element budget exceeded"));
            }
            for el in elements {
                validate_op(el, param_count, local_count, applications, depth + 1, limits)?;
            }
        }
        OperationalOpV1::Block {
            local_count: block_locals,
            statements,
            result,
        } => {
            if *block_locals > local_count {
                return Err(invalid("operational block local count exceeds its static-definition frame"));
            }
            for s in statements {
                validate_op(s, param_count, local_count, applications, depth + 1, limits)?;
            }
            if let Some(res) = result {
                validate_op(res, param_count, local_count, applications, depth + 1, limits)?;
            }
        }
        OperationalOpV1::Return { value } => {
            if let Some(val) = value {
                validate_op(val, param_count, local_count, applications, depth + 1, limits)?;
            }
        }
    }
    Ok(())
}

fn validate_module(module: &StableModuleRef, limits: &ValidationLimits) -> Result<(), ArtifactError> {
    crate::identity::validate_persistent_project_ref(&module.project, limits)?;
    let project_strings: Vec<&str> = match &module.project {
        StableProjectRef::Builtin { namespace, version } => vec![namespace, version],
        StableProjectRef::Package { package, version, .. } => vec![package, version],
        StableProjectRef::SourceArtifact { logical_uri, .. } => vec![logical_uri],
        StableProjectRef::Session { .. } => return Err(invalid("session project identity is not persistent")),
    };
    if project_strings.iter().any(|value| value.is_empty() || value.len() > limits.max_string_bytes)
        || module.path.is_empty()
        || module.path.len() > limits.max_semantic_entries_per_family
        || module.path.iter().any(|part| part.is_empty() || part.len() > limits.max_string_bytes)
    {
        return Err(invalid("invalid or oversized static constant module identity"));
    }
    Ok(())
}

fn validate_owner(value: &StableDeclarationRef, limits: &ValidationLimits) -> Result<(), ArtifactError> {
    crate::identity::validate_persistent_project_ref(&value.module.project, limits)?;
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
