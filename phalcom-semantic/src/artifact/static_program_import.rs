//! Typed P7 canonical static-program reconstruction for ARTF001 staging.
//!
//! Current-main no longer stores an Operational body variant in its live
//! AST-backed StaticDefinition. The certified operational program remains the
//! durable source-free execution authority, so this module reconstructs it as
//! an immutable staging product using current-main IDs and stores.

use super::identity::StableOwnerCatalog;
use super::static_import::ImportedStaticValueGraph;
use super::type_import::ReconstructedTypeGraph;
use crate::checker::analysis::SemanticDependency;
use crate::db::{InputFingerprint, ProductFingerprint};
use crate::identity::{DataConstructorId, DeclarationId, VariantId};
use crate::static_definition::{StaticApplicationId, StaticBodyId, StaticDefinitionSignature, StaticParameter};
use crate::static_domain::StaticValueDomain;
use crate::static_value::StaticValueRef;
use crate::types::id::{TypeId, TypeParameterId};
use crate::types::parameter::GenericParameterMode;
use phalcom_common::range::SourceRange;
use phalcom_compiler_artifact::semantic::operational::{
    OperationalOpV1, OperationalTargetV1, StaticBinaryOpV1, StaticUnaryOpV1,
};
use phalcom_compiler_artifact::semantic::type_store::ParameterModeV1;
use phalcom_compiler_artifact::semantic::world::{
    CanonicalCheckedStaticApplicationV1, CanonicalStaticApplicationArgumentV1,
    CanonicalStaticProgramV1, StaticConstantSymbolV1, StaticDefinitionSignatureV1,
    StaticValueDomainV1,
};
use phalcom_compiler_artifact::{ArtifactError, ValidationError, ValidationLimits};
use phalcom_modules::linker::SymbolId;
use phalcom_type_meta::allocation::AllocationBudget;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImportedStaticBinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Eq,
    NotEq,
    Less,
    LessEq,
    Greater,
    GreaterEq,
    And,
    Or,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImportedStaticUnaryOp {
    Neg,
    Not,
    BitNot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImportedOperationalTarget {
    Declaration(DeclarationId),
    DataConstructor(DataConstructorId),
    VariantConstructor(VariantId),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImportedOperationalNode {
    Constant(StaticValueRef),
    ReadStaticConstant(SymbolId),
    ReadParameter(u32),
    ReadLocal(u32),
    WriteLocal {
        slot: u32,
        value: Box<ImportedOperationalNode>,
    },
    TypeForm(TypeId),
    Binary {
        op: ImportedStaticBinaryOp,
        left: Box<ImportedOperationalNode>,
        right: Box<ImportedOperationalNode>,
    },
    Unary {
        op: ImportedStaticUnaryOp,
        operand: Box<ImportedOperationalNode>,
    },
    StaticApplication {
        application_ordinal: u32,
        arguments: Box<[ImportedOperationalNode]>,
    },
    StaticConstruction {
        target: ImportedOperationalTarget,
        result_type: TypeId,
        arguments: Box<[ImportedOperationalNode]>,
    },
    MemberProjection {
        target: Box<ImportedOperationalNode>,
        member: Box<str>,
    },
    If {
        condition: Box<ImportedOperationalNode>,
        then_branch: Box<ImportedOperationalNode>,
        else_branch: Option<Box<ImportedOperationalNode>>,
    },
    Sequence {
        result_type: TypeId,
        elements: Box<[ImportedOperationalNode]>,
    },
    Tuple {
        elements: Box<[ImportedOperationalNode]>,
    },
    Block {
        local_count: u32,
        statements: Box<[ImportedOperationalNode]>,
        result: Option<Box<ImportedOperationalNode>>,
    },
    Return(Option<Box<ImportedOperationalNode>>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImportedCanonicalStaticApplicationArgument {
    KindedTerm {
        parameter_index: u32,
        term: TypeId,
    },
    KindedExpression {
        parameter_index: u32,
        application_ordinal: u32,
    },
    Value {
        parameter_index: u32,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImportedCanonicalCheckedStaticApplication {
    pub id: StaticApplicationId,
    pub source_range: SourceRange,
    pub target: DeclarationId,
    pub signature: StaticDefinitionSignature,
    pub arguments: Box<[ImportedCanonicalStaticApplicationArgument]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImportedCanonicalStaticProgram {
    pub owner: StaticBodyId,
    pub parameter_count: u32,
    pub local_count: u32,
    pub root: ImportedOperationalNode,
    pub applications: Box<[ImportedCanonicalCheckedStaticApplication]>,
    pub kinded_parameters: Box<[u32]>,
    pub static_constant_reads: Box<[SymbolId]>,
    pub dependencies: Box<[SemanticDependency]>,
    pub result: StaticValueDomain,
    pub input_fingerprint: InputFingerprint,
    pub module_binding_fingerprint: Option<ProductFingerprint>,
}

pub(crate) fn reconstruct_static_definition_signature(
    signature: &StaticDefinitionSignatureV1,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<StaticDefinitionSignature, ArtifactError> {
    let mut parameters = allocations
        .vector::<StaticParameter>(signature.parameters.len())
        .map_err(ArtifactError::Allocation)?;
    for parameter in &signature.parameters {
        parameters.push(StaticParameter {
            name: allocations
                .string(&parameter.name)
                .map_err(ArtifactError::Allocation)?,
            domain: static_value_domain(&parameter.domain, types)?,
            mode: parameter_mode(&parameter.mode),
            index: parameter.index,
            type_parameter_id: parameter
                .type_parameter
                .map(|parameter| {
                    types
                        .parameters
                        .get(parameter.0 as usize)
                        .copied()
                        .ok_or_else(|| invalid("static parameter references an absent type parameter"))
                })
                .transpose()?,
            range: super::diagnostic_import::range(&parameter.source, "static parameter source")?,
        });
    }

    Ok(StaticDefinitionSignature {
        parameters: allocations
            .boxed_slice(parameters)
            .map_err(ArtifactError::Allocation)?,
        result_domain: static_value_domain(&signature.result_domain, types)?,
        generic_signature: signature
            .generic_signature
            .as_ref()
            .map(|signature| {
                super::semantic_import::reconstruct_signature(
                    signature,
                    types,
                    owners,
                    allocations,
                )
            })
            .transpose()?,
        // P7 transport carries the canonical generic signature/constraints, not
        // source syntax. Reconstructing a WhereClauseSyntax would invent source.
        where_clause: None,
    })
}

pub(crate) fn reconstruct_canonical_static_program(
    program: &CanonicalStaticProgramV1,
    static_values: &ImportedStaticValueGraph,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<ImportedCanonicalStaticProgram, ArtifactError> {
    let owner = StaticBodyId(owners.declaration(&program.owner)?);

    let mut applications = allocations
        .vector::<ImportedCanonicalCheckedStaticApplication>(program.applications.len())
        .map_err(ArtifactError::Allocation)?;
    for (index, application) in program.applications.iter().enumerate() {
        let ordinal = u32::try_from(index)
            .map_err(|_| invalid("static application ordinal exceeds u32"))?;
        if application.ordinal != ordinal {
            return Err(invalid("static application rows are not dense and ordinal-sorted"));
        }
        applications.push(reconstruct_application(
            application,
            &owner,
            types,
            owners,
            allocations,
        )?);
    }

    let mut kinded_parameters = allocations
        .vector::<u32>(program.kinded_parameters.len())
        .map_err(ArtifactError::Allocation)?;
    let mut previous = None;
    for &parameter in &program.kinded_parameters {
        if parameter >= program.parameter_count
            || previous.is_some_and(|previous| parameter <= previous)
        {
            return Err(invalid(
                "canonical static-program kinded parameters are not strictly ordered in range",
            ));
        }
        previous = Some(parameter);
        kinded_parameters.push(parameter);
    }

    let mut static_constant_reads = allocations
        .vector::<SymbolId>(program.static_constant_reads.len())
        .map_err(ArtifactError::Allocation)?;
    for read in &program.static_constant_reads {
        static_constant_reads.push(static_symbol(read, owners, allocations)?);
    }

    let mut dependencies = allocations
        .vector::<SemanticDependency>(program.dependencies.len())
        .map_err(ArtifactError::Allocation)?;
    for dependency in &program.dependencies {
        dependencies.push(super::static_dependency_import::reconstruct_static_dependency(
            dependency,
            types,
            owners,
            allocations,
        )?);
    }

    let root = reconstruct_op(
        &program.root,
        static_values,
        types,
        owners,
        program,
        0,
        limits,
        allocations,
    )?;

    Ok(ImportedCanonicalStaticProgram {
        owner,
        parameter_count: program.parameter_count,
        local_count: program.local_count,
        root,
        applications: allocations
            .boxed_slice(applications)
            .map_err(ArtifactError::Allocation)?,
        kinded_parameters: allocations
            .boxed_slice(kinded_parameters)
            .map_err(ArtifactError::Allocation)?,
        static_constant_reads: allocations
            .boxed_slice(static_constant_reads)
            .map_err(ArtifactError::Allocation)?,
        dependencies: allocations
            .boxed_slice(dependencies)
            .map_err(ArtifactError::Allocation)?,
        result: static_value_domain(&program.result, types)?,
        input_fingerprint: InputFingerprint::new(program.input_fingerprint),
        module_binding_fingerprint: program
            .module_binding_fingerprint
            .map(ProductFingerprint::new),
    })
}

fn reconstruct_application(
    application: &CanonicalCheckedStaticApplicationV1,
    owner: &StaticBodyId,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedCanonicalCheckedStaticApplication, ArtifactError> {
    let mut arguments = allocations
        .vector::<ImportedCanonicalStaticApplicationArgument>(application.arguments.len())
        .map_err(ArtifactError::Allocation)?;
    for (index, argument) in application.arguments.iter().enumerate() {
        let expected_index = u32::try_from(index)
            .map_err(|_| invalid("static application parameter index exceeds u32"))?;
        let imported = match argument {
            CanonicalStaticApplicationArgumentV1::KindedTerm {
                parameter_index,
                term,
            } => {
                if *parameter_index != expected_index {
                    return Err(invalid("static application argument order disagrees with parameter order"));
                }
                ImportedCanonicalStaticApplicationArgument::KindedTerm {
                    parameter_index: *parameter_index,
                    term: mapped_type(*term, types, "static application kinded term")?,
                }
            }
            CanonicalStaticApplicationArgumentV1::KindedExpression {
                parameter_index,
                application_ordinal,
            } => {
                if *parameter_index != expected_index {
                    return Err(invalid("static application argument order disagrees with parameter order"));
                }
                ImportedCanonicalStaticApplicationArgument::KindedExpression {
                    parameter_index: *parameter_index,
                    application_ordinal: *application_ordinal,
                }
            }
            CanonicalStaticApplicationArgumentV1::Value { parameter_index } => {
                if *parameter_index != expected_index {
                    return Err(invalid("static application argument order disagrees with parameter order"));
                }
                ImportedCanonicalStaticApplicationArgument::Value {
                    parameter_index: *parameter_index,
                }
            }
        };
        arguments.push(imported);
    }

    let signature = reconstruct_static_definition_signature(
        &application.signature,
        types,
        owners,
        allocations,
    )?;
    if signature.parameters.len() != arguments.len() {
        return Err(invalid(
            "static application argument count disagrees with target signature",
        ));
    }

    Ok(ImportedCanonicalCheckedStaticApplication {
        id: StaticApplicationId {
            body: owner.clone(),
            ordinal: application.ordinal,
        },
        source_range: super::diagnostic_import::range(
            &application.source,
            "static application source",
        )?,
        target: owners.declaration(&application.target)?,
        signature,
        arguments: allocations
            .boxed_slice(arguments)
            .map_err(ArtifactError::Allocation)?,
    })
}

fn reconstruct_op(
    op: &OperationalOpV1,
    static_values: &ImportedStaticValueGraph,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    program: &CanonicalStaticProgramV1,
    depth: usize,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<ImportedOperationalNode, ArtifactError> {
    if depth > limits.max_recursion_depth {
        return Err(invalid("operational template recursion depth exceeded"));
    }
    Ok(match op {
        OperationalOpV1::Constant { value } => ImportedOperationalNode::Constant(
            static_values
                .value(*value)
                .ok_or_else(|| invalid("operational constant references an absent static value"))?,
        ),
        OperationalOpV1::ReadStaticConstant { module, name } => {
            ImportedOperationalNode::ReadStaticConstant(SymbolId {
                module: owners.module(module)?,
                name: allocations.boxed_str(name).map_err(ArtifactError::Allocation)?,
            })
        }
        OperationalOpV1::ReadParameter { index } => {
            if *index >= program.parameter_count {
                return Err(invalid("operational parameter read is out of range"));
            }
            ImportedOperationalNode::ReadParameter(*index)
        }
        OperationalOpV1::ReadLocal { slot } => {
            if *slot >= program.local_count {
                return Err(invalid("operational local read is out of range"));
            }
            ImportedOperationalNode::ReadLocal(*slot)
        }
        OperationalOpV1::WriteLocal { slot, value } => {
            if *slot >= program.local_count {
                return Err(invalid("operational local write is out of range"));
            }
            ImportedOperationalNode::WriteLocal {
                slot: *slot,
                value: boxed_op(
                    value,
                    static_values,
                    types,
                    owners,
                    program,
                    depth + 1,
                    limits,
                    allocations,
                )?,
            }
        }
        OperationalOpV1::TypeForm { ty } => {
            ImportedOperationalNode::TypeForm(mapped_type(*ty, types, "operational type form")?)
        }
        OperationalOpV1::Binary { op, left, right } => ImportedOperationalNode::Binary {
            op: binary_op(op),
            left: boxed_op(
                left,
                static_values,
                types,
                owners,
                program,
                depth + 1,
                limits,
                allocations,
            )?,
            right: boxed_op(
                right,
                static_values,
                types,
                owners,
                program,
                depth + 1,
                limits,
                allocations,
            )?,
        },
        OperationalOpV1::Unary { op, operand } => ImportedOperationalNode::Unary {
            op: unary_op(op),
            operand: boxed_op(
                operand,
                static_values,
                types,
                owners,
                program,
                depth + 1,
                limits,
                allocations,
            )?,
        },
        OperationalOpV1::StaticApplication {
            application_ordinal,
            arguments,
        } => {
            let application = program
                .applications
                .get(*application_ordinal as usize)
                .filter(|row| row.ordinal == *application_ordinal)
                .ok_or_else(|| invalid("operational static application references an absent row"))?;
            if application.arguments.len() != arguments.len() {
                return Err(invalid(
                    "operational static application disagrees with its checked row",
                ));
            }
            ImportedOperationalNode::StaticApplication {
                application_ordinal: *application_ordinal,
                arguments: op_slice(
                    arguments,
                    static_values,
                    types,
                    owners,
                    program,
                    depth + 1,
                    limits,
                    allocations,
                )?,
            }
        }
        OperationalOpV1::StaticConstruction {
            target,
            result_type,
            arguments,
        } => ImportedOperationalNode::StaticConstruction {
            target: operational_target(target, owners)?,
            result_type: mapped_type(*result_type, types, "static construction result type")?,
            arguments: op_slice(
                arguments,
                static_values,
                types,
                owners,
                program,
                depth + 1,
                limits,
                allocations,
            )?,
        },
        OperationalOpV1::MemberProjection { target, member } => {
            ImportedOperationalNode::MemberProjection {
                target: boxed_op(
                    target,
                    static_values,
                    types,
                    owners,
                    program,
                    depth + 1,
                    limits,
                    allocations,
                )?,
                member: allocations.boxed_str(member).map_err(ArtifactError::Allocation)?,
            }
        }
        OperationalOpV1::If {
            condition,
            then_branch,
            else_branch,
        } => ImportedOperationalNode::If {
            condition: boxed_op(
                condition,
                static_values,
                types,
                owners,
                program,
                depth + 1,
                limits,
                allocations,
            )?,
            then_branch: boxed_op(
                then_branch,
                static_values,
                types,
                owners,
                program,
                depth + 1,
                limits,
                allocations,
            )?,
            else_branch: else_branch
                .as_ref()
                .map(|branch| {
                    boxed_op(
                        branch,
                        static_values,
                        types,
                        owners,
                        program,
                        depth + 1,
                        limits,
                        allocations,
                    )
                })
                .transpose()?,
        },
        OperationalOpV1::Sequence {
            result_type,
            elements,
        } => ImportedOperationalNode::Sequence {
            result_type: mapped_type(*result_type, types, "operational sequence result type")?,
            elements: op_slice(
                elements,
                static_values,
                types,
                owners,
                program,
                depth + 1,
                limits,
                allocations,
            )?,
        },
        OperationalOpV1::Tuple { elements } => ImportedOperationalNode::Tuple {
            elements: op_slice(
                elements,
                static_values,
                types,
                owners,
                program,
                depth + 1,
                limits,
                allocations,
            )?,
        },
        OperationalOpV1::Block {
            local_count,
            statements,
            result,
        } => ImportedOperationalNode::Block {
            local_count: *local_count,
            statements: op_slice(
                statements,
                static_values,
                types,
                owners,
                program,
                depth + 1,
                limits,
                allocations,
            )?,
            result: result
                .as_ref()
                .map(|result| {
                    boxed_op(
                        result,
                        static_values,
                        types,
                        owners,
                        program,
                        depth + 1,
                        limits,
                        allocations,
                    )
                })
                .transpose()?,
        },
        OperationalOpV1::Return { value } => ImportedOperationalNode::Return(
            value
                .as_ref()
                .map(|value| {
                    boxed_op(
                        value,
                        static_values,
                        types,
                        owners,
                        program,
                        depth + 1,
                        limits,
                        allocations,
                    )
                })
                .transpose()?,
        ),
    })
}

fn boxed_op(
    op: &OperationalOpV1,
    static_values: &ImportedStaticValueGraph,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    program: &CanonicalStaticProgramV1,
    depth: usize,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<Box<ImportedOperationalNode>, ArtifactError> {
    let node = reconstruct_op(
        op,
        static_values,
        types,
        owners,
        program,
        depth,
        limits,
        allocations,
    )?;
    allocations.boxed(node).map_err(ArtifactError::Allocation)
}

fn op_slice(
    ops: &[OperationalOpV1],
    static_values: &ImportedStaticValueGraph,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    program: &CanonicalStaticProgramV1,
    depth: usize,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<Box<[ImportedOperationalNode]>, ArtifactError> {
    let mut imported = allocations
        .vector::<ImportedOperationalNode>(ops.len())
        .map_err(ArtifactError::Allocation)?;
    for op in ops {
        imported.push(reconstruct_op(
            op,
            static_values,
            types,
            owners,
            program,
            depth,
            limits,
            allocations,
        )?);
    }
    allocations
        .boxed_slice(imported)
        .map_err(ArtifactError::Allocation)
}

fn operational_target(
    target: &OperationalTargetV1,
    owners: &StableOwnerCatalog<'_>,
) -> Result<ImportedOperationalTarget, ArtifactError> {
    Ok(match target {
        OperationalTargetV1::Declaration { target } => {
            ImportedOperationalTarget::Declaration(owners.declaration(target)?)
        }
        OperationalTargetV1::DataConstructor { owner } => {
            ImportedOperationalTarget::DataConstructor(DataConstructorId::new(
                owners.declaration(owner)?,
            ))
        }
        OperationalTargetV1::VariantConstructor { owner, selector } => {
            ImportedOperationalTarget::VariantConstructor(VariantId::new(
                owners.declaration(owner)?,
                super::identity::canonical_selector(selector)?,
            ))
        }
    })
}

fn static_value_domain(
    domain: &StaticValueDomainV1,
    types: &ReconstructedTypeGraph,
) -> Result<StaticValueDomain, ArtifactError> {
    Ok(match domain {
        StaticValueDomainV1::Kind { kind } => StaticValueDomain::Kind(
            *types
                .kinds
                .get(kind.0 as usize)
                .ok_or_else(|| invalid("static value domain references an absent kind"))?,
        ),
        StaticValueDomainV1::ValueType { ty } => {
            StaticValueDomain::ValueType(mapped_type(*ty, types, "static value domain type")?)
        }
    })
}

fn parameter_mode(mode: &ParameterModeV1) -> GenericParameterMode {
    match mode {
        ParameterModeV1::Single => GenericParameterMode::Single,
        ParameterModeV1::PositionalPack => GenericParameterMode::PositionalPack,
        ParameterModeV1::LabeledPack => GenericParameterMode::LabeledPack,
        ParameterModeV1::CompletePack => GenericParameterMode::CompletePack,
    }
}

fn static_symbol(
    symbol: &StaticConstantSymbolV1,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<SymbolId, ArtifactError> {
    Ok(SymbolId {
        module: owners.module(&symbol.module)?,
        name: allocations
            .boxed_str(&symbol.name)
            .map_err(ArtifactError::Allocation)?,
    })
}

fn mapped_type(
    id: phalcom_compiler_artifact::semantic::type_store::ArtifactTypeId,
    types: &ReconstructedTypeGraph,
    family: &'static str,
) -> Result<TypeId, ArtifactError> {
    types
        .types
        .get(id.0 as usize)
        .copied()
        .ok_or_else(|| invalid(format!("{family} references an absent type")))
}

fn binary_op(op: &StaticBinaryOpV1) -> ImportedStaticBinaryOp {
    match op {
        StaticBinaryOpV1::Add => ImportedStaticBinaryOp::Add,
        StaticBinaryOpV1::Sub => ImportedStaticBinaryOp::Sub,
        StaticBinaryOpV1::Mul => ImportedStaticBinaryOp::Mul,
        StaticBinaryOpV1::Div => ImportedStaticBinaryOp::Div,
        StaticBinaryOpV1::Mod => ImportedStaticBinaryOp::Mod,
        StaticBinaryOpV1::BitAnd => ImportedStaticBinaryOp::BitAnd,
        StaticBinaryOpV1::BitOr => ImportedStaticBinaryOp::BitOr,
        StaticBinaryOpV1::BitXor => ImportedStaticBinaryOp::BitXor,
        StaticBinaryOpV1::Shl => ImportedStaticBinaryOp::Shl,
        StaticBinaryOpV1::Shr => ImportedStaticBinaryOp::Shr,
        StaticBinaryOpV1::Eq => ImportedStaticBinaryOp::Eq,
        StaticBinaryOpV1::NotEq => ImportedStaticBinaryOp::NotEq,
        StaticBinaryOpV1::Less => ImportedStaticBinaryOp::Less,
        StaticBinaryOpV1::LessEq => ImportedStaticBinaryOp::LessEq,
        StaticBinaryOpV1::Greater => ImportedStaticBinaryOp::Greater,
        StaticBinaryOpV1::GreaterEq => ImportedStaticBinaryOp::GreaterEq,
        StaticBinaryOpV1::And => ImportedStaticBinaryOp::And,
        StaticBinaryOpV1::Or => ImportedStaticBinaryOp::Or,
    }
}

fn unary_op(op: &StaticUnaryOpV1) -> ImportedStaticUnaryOp {
    match op {
        StaticUnaryOpV1::Neg => ImportedStaticUnaryOp::Neg,
        StaticUnaryOpV1::Not => ImportedStaticUnaryOp::Not,
        StaticUnaryOpV1::BitNot => ImportedStaticUnaryOp::BitNot,
    }
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
