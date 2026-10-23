//! Artifact export, import, and lowering for checked operational static templates (`PERF003.C2.P7.T7.B`).

use super::import::{
    import_generic_signature_with_allocation_budget, import_stable_declaration_with_allocation_budget, import_stable_module_with_allocation_budget,
    import_stable_selector_with_allocation_budget,
};
use crate::identity::VariantId;
use crate::metadata::stable_identity::{to_stable_declaration_with_context, to_stable_module_with_context};
use crate::static_definition::{
    CanonicalCheckedStaticApplication, CanonicalStaticApplicationArgument, CanonicalStaticProgram, CheckedStaticApplication, CheckedStaticApplicationArgument,
    CheckedStaticBody, OperationalNode, OperationalTarget, OperationalTemplate, StaticBinaryOp, StaticBodyId, StaticDefinitionBodyRepr,
    StaticDefinitionSignature, StaticUnaryOp,
};
use crate::static_domain::StaticValueDomain;
use crate::static_eval::machine::canonical_symbol_spelling;
use crate::static_value::{StaticExportContext, StaticFloat, StaticRelocation, StaticValueStore};
use crate::types::store::TypeStore;
use crate::types::store::artifact::{TypeExportContext, TypeRelocation};
use num_bigint::BigInt;
use phalcom_ast::ast::{BinaryOp, BracketLiteralElement, Expr, IfElseBody, LexicalBlock, Pattern, Statement, TupleLiteralEntry, UnaryOp};
use phalcom_compiler_artifact::semantic::operational::*;
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_type_meta::allocation::AllocationBudget;

fn referential(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}

impl CanonicalStaticProgram {
    pub fn from_checked_body(body: &CheckedStaticBody, values: &mut StaticValueStore, types: &TypeStore) -> Result<Self, ArtifactError> {
        let parameter_count = body
            .parameter_bindings
            .keys()
            .chain(body.kinded_parameters.keys())
            .max()
            .map_or(0, |idx| *idx + 1);
        let local_count = body.local_slots.len() as u32;
        let root = match &body.root {
            StaticDefinitionBodyRepr::Operational(program) => return Ok((**program).clone()),
            StaticDefinitionBodyRepr::Expression(expr) => lower_expr(expr, body, values, types)?,
            StaticDefinitionBodyRepr::Block(block) => lower_block(block, body, values, types)?,
            StaticDefinitionBodyRepr::TypeForm(annotation) => lower_expr(&Expr::TypeForm(Box::new(annotation.clone())), body, values, types)?,
        };

        let mut ordered_rows = body.static_applications.iter().collect::<Vec<_>>();
        ordered_rows.sort_unstable_by_key(|application| application.id.ordinal);
        let mut applications = Vec::with_capacity(ordered_rows.len());
        for (index, application) in ordered_rows.into_iter().enumerate() {
            let expected = u32::try_from(index).map_err(|_| referential("static application ordinal overflow"))?;
            if application.id.body != body.owner || application.id.ordinal != expected {
                return Err(referential("checked static application rows are not canonical for their body"));
            }
            let mut canonical_arguments = Vec::with_capacity(application.arguments.len());
            for argument in application.arguments.iter() {
                canonical_arguments.push(match argument {
                    CheckedStaticApplicationArgument::Kinded { parameter_index, term } => CanonicalStaticApplicationArgument::KindedTerm {
                        parameter_index: *parameter_index,
                        term: *term,
                    },
                    CheckedStaticApplicationArgument::KindedExpression { parameter_index, expression } => {
                        let nested = unique_application_for_expression(body, *expression)?;
                        CanonicalStaticApplicationArgument::KindedExpression {
                            parameter_index: *parameter_index,
                            application_ordinal: nested.id.ordinal,
                        }
                    }
                    CheckedStaticApplicationArgument::Value { parameter_index, .. } => CanonicalStaticApplicationArgument::Value {
                        parameter_index: *parameter_index,
                    },
                });
            }
            if application.signature.parameters.len() != canonical_arguments.len()
                || canonical_arguments.iter().enumerate().any(|(index, argument)| {
                    let parameter_index = match argument {
                        CanonicalStaticApplicationArgument::KindedTerm { parameter_index, .. }
                        | CanonicalStaticApplicationArgument::KindedExpression { parameter_index, .. }
                        | CanonicalStaticApplicationArgument::Value { parameter_index } => *parameter_index,
                    };
                    parameter_index as usize != index
                })
            {
                return Err(referential("checked static application arguments disagree with the target signature"));
            }
            for (parameter, argument) in application.signature.parameters.iter().zip(&canonical_arguments) {
                match (parameter.domain, argument) {
                    (StaticValueDomain::Kind(expected), CanonicalStaticApplicationArgument::KindedTerm { term, .. }) if types.kind_of(*term) == expected => {}
                    (StaticValueDomain::Kind(expected), CanonicalStaticApplicationArgument::KindedExpression { application_ordinal, .. }) => {
                        let nested = body
                            .static_applications
                            .iter()
                            .find(|candidate| candidate.id.ordinal == *application_ordinal)
                            .ok_or_else(|| referential("kinded-expression child application is missing"))?;
                        if nested.signature.result_domain.as_kind() != Some(expected) {
                            return Err(referential("kinded-expression child result kind disagrees with its parameter"));
                        }
                    }
                    (StaticValueDomain::ValueType(_), CanonicalStaticApplicationArgument::Value { .. }) => {}
                    _ => return Err(referential("checked static application argument category disagrees with its parameter")),
                }
            }
            applications.push(CanonicalCheckedStaticApplication {
                id: application.id.clone(),
                source_range: application.source_range,
                target: application.target.clone(),
                signature: application.signature.clone(),
                arguments: canonical_arguments.into_boxed_slice(),
            });
        }

        Ok(Self {
            owner: body.owner.clone(),
            parameter_count,
            local_count,
            root,
            applications: applications.into(),
            kinded_parameters: body.kinded_parameters.keys().copied().collect::<Vec<_>>().into(),
            static_constant_reads: body.static_constant_reads.clone(),
            dependencies: body.dependencies.clone(),
            result: body.result,
            input_fingerprint: body.input_fingerprint,
            module_binding_fingerprint: body.module_binding_fingerprint,
        })
    }
}

fn unique_application_for_expression(body: &CheckedStaticBody, expression: crate::identity::ExpressionId) -> Result<&CheckedStaticApplication, ArtifactError> {
    let mut matches = body.static_applications.iter().filter(|application| application.expression == expression);
    let Some(application) = matches.next() else {
        return Err(referential("checked static application reference has no row"));
    };
    if matches.next().is_some() {
        return Err(referential("checked static application reference is ambiguous"));
    }
    Ok(application)
}

fn checked_application_for_range(
    body: &CheckedStaticBody,
    range: phalcom_common::range::SourceRange,
) -> Result<Option<&CheckedStaticApplication>, ArtifactError> {
    let mut expressions = body.expressions.iter().filter(|(_, analysis)| analysis.range == range).map(|(id, _)| *id);
    let Some(expression) = expressions.next() else {
        return Ok(None);
    };
    if expressions.next().is_some() {
        return Err(referential("static application expression range is ambiguous"));
    }
    let mut applications = body.static_applications.iter().filter(|application| application.expression == expression);
    let Some(application) = applications.next() else {
        return Ok(None);
    };
    if applications.next().is_some() {
        return Err(referential("static application expression maps to multiple checked rows"));
    }
    Ok(Some(application))
}

fn lower_application_node(
    body: &CheckedStaticBody,
    application: &CheckedStaticApplication,
    path: &mut std::collections::BTreeSet<u32>,
) -> Result<OperationalNode, ArtifactError> {
    if !path.insert(application.id.ordinal) {
        return Err(referential("checked static application graph contains a cycle"));
    }
    let mut arguments = Vec::with_capacity(application.arguments.len());
    for argument in application.arguments.iter() {
        arguments.push(match argument {
            CheckedStaticApplicationArgument::Kinded { term, .. } => OperationalNode::TypeForm(*term),
            CheckedStaticApplicationArgument::KindedExpression { expression, .. } => {
                let nested = unique_application_for_expression(body, *expression)?;
                lower_application_node(body, nested, path)?
            }
            CheckedStaticApplicationArgument::Value { expression, .. } => {
                let binding = body.expressions.get(expression).and_then(|analysis| analysis.binding_read);
                let Some(binding) = binding else {
                    return Err(referential("checked static value argument has no canonical binding"));
                };
                if let Some((&index, _)) = body.parameter_bindings.iter().find(|(_, candidate)| **candidate == binding) {
                    OperationalNode::ReadParameter(index)
                } else if let Some(&slot) = body.local_slots.get(&binding) {
                    OperationalNode::ReadLocal(slot)
                } else if let Some(symbol) = body.static_constant_bindings.get(&binding) {
                    OperationalNode::ReadStaticConstant(symbol.clone())
                } else {
                    return Err(referential("checked static value argument binding is not local to its body"));
                }
            }
        });
    }
    path.remove(&application.id.ordinal);
    Ok(OperationalNode::StaticApplication {
        application_ordinal: application.id.ordinal,
        arguments,
    })
}

fn lower_expr(expr: &Expr, body: &CheckedStaticBody, values: &mut StaticValueStore, types: &TypeStore) -> Result<OperationalNode, ArtifactError> {
    match expr {
        Expr::Int { digits, radix, .. } => {
            let integer = BigInt::parse_bytes(digits.as_bytes(), *radix).ok_or_else(|| referential("invalid integer literal"))?;
            let val = values.intern_int(integer);
            Ok(OperationalNode::Constant(val))
        }
        Expr::Float { value, .. } => {
            let val = values.intern_float(StaticFloat::from_f64(*value));
            Ok(OperationalNode::Constant(val))
        }
        Expr::String { value, .. } => {
            let val = values.intern_string(value);
            Ok(OperationalNode::Constant(val))
        }
        Expr::Boolean { value, .. } => {
            let val = values.intern_bool(*value);
            Ok(OperationalNode::Constant(val))
        }
        Expr::Char { value, .. } => {
            let val = values.intern_char(*value);
            Ok(OperationalNode::Constant(val))
        }
        Expr::Symbol(symbol) => {
            let spelling = canonical_symbol_spelling(&symbol.kind).ok_or_else(|| referential("unsupported symbol literal"))?;
            let val = values.intern_symbol(spelling);
            Ok(OperationalNode::Constant(val))
        }
        Expr::Var { .. } => {
            let (_, analysis) = body
                .expressions
                .iter()
                .find(|(_, a)| a.range == expr.range())
                .ok_or_else(|| referential("missing expression analysis for var"))?;
            if let Some(crate::types::denotation::SemanticDenotation::TypeForm(ty) | crate::types::denotation::SemanticDenotation::StaticTerm(ty)) =
                analysis.denotation.as_ref()
            {
                Ok(OperationalNode::TypeForm(*ty))
            } else if let Some(binding) = analysis.binding_read {
                if let Some((&index, _)) = body.parameter_bindings.iter().find(|(_, b)| **b == binding) {
                    Ok(OperationalNode::ReadParameter(index))
                } else if let Some(&slot) = body.local_slots.get(&binding) {
                    Ok(OperationalNode::ReadLocal(slot))
                } else if let Some(symbol) = body.static_constant_bindings.get(&binding) {
                    Ok(OperationalNode::ReadStaticConstant(symbol.clone()))
                } else {
                    Err(referential(format!("unbound binding {:?}", binding)))
                }
            } else {
                Err(referential("var has neither type denotation nor binding read"))
            }
        }
        Expr::TypeForm(_) => {
            if let Some(application) = checked_application_for_range(body, expr.range())? {
                return lower_application_node(body, application, &mut std::collections::BTreeSet::new());
            }
            let mut analyses = body.expressions.iter().filter(|(_, analysis)| analysis.range == expr.range());
            let Some((_, analysis)) = analyses.next() else {
                return Err(referential("unresolved type form expression"));
            };
            if analyses.next().is_some() {
                return Err(referential("static type-form expression range is ambiguous"));
            }
            if let Some(crate::types::denotation::SemanticDenotation::TypeForm(ty) | crate::types::denotation::SemanticDenotation::StaticTerm(ty)) =
                analysis.denotation.as_ref()
            {
                Ok(OperationalNode::TypeForm(*ty))
            } else if matches!(analysis.denotation, Some(crate::types::denotation::SemanticDenotation::StaticApplication(_))) {
                Err(referential("static application denotation has no checked row"))
            } else {
                Err(referential("unresolved type form expression"))
            }
        }
        Expr::Binary(b) => {
            let op = match b.op {
                BinaryOp::Add => StaticBinaryOp::Add,
                BinaryOp::Subtract => StaticBinaryOp::Sub,
                BinaryOp::Multiply => StaticBinaryOp::Mul,
                BinaryOp::Divide => StaticBinaryOp::Div,
                BinaryOp::Modulo => StaticBinaryOp::Mod,
                BinaryOp::BitAnd => StaticBinaryOp::BitAnd,
                BinaryOp::BitOr => StaticBinaryOp::BitOr,
                BinaryOp::BitXor => StaticBinaryOp::BitXor,
                BinaryOp::ShiftLeft => StaticBinaryOp::Shl,
                BinaryOp::ShiftRight => StaticBinaryOp::Shr,
                BinaryOp::Equal => StaticBinaryOp::Eq,
                BinaryOp::NotEqual => StaticBinaryOp::NotEq,
                BinaryOp::LessThan => StaticBinaryOp::Less,
                BinaryOp::LessThanOrEqual => StaticBinaryOp::LessEq,
                BinaryOp::GreaterThan => StaticBinaryOp::Greater,
                BinaryOp::GreaterThanOrEqual => StaticBinaryOp::GreaterEq,
                BinaryOp::And => StaticBinaryOp::And,
                BinaryOp::Or => StaticBinaryOp::Or,
                _ => return Err(referential(format!("unsupported binary op {:?}", b.op))),
            };
            let left = lower_expr(&b.left, body, values, types)?;
            let right = lower_expr(&b.right, body, values, types)?;
            Ok(OperationalNode::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            })
        }
        Expr::Unary(u) => {
            let op = match u.op {
                UnaryOp::Minus => StaticUnaryOp::Neg,
                UnaryOp::Not => StaticUnaryOp::Not,
                UnaryOp::BitNot => StaticUnaryOp::BitNot,
                _ => return Err(referential(format!("unsupported unary op {:?}", u.op))),
            };
            let operand = lower_expr(&u.expr, body, values, types)?;
            Ok(OperationalNode::Unary {
                op,
                operand: Box::new(operand),
            })
        }
        Expr::If(if_expr) => {
            let condition = lower_expr(&if_expr.condition, body, values, types)?;
            let then_branch = lower_block(&if_expr.then_body, body, values, types)?;
            let else_branch = match &if_expr.else_body {
                Some(IfElseBody::Block(block)) => Some(Box::new(lower_block(block, body, values, types)?)),
                Some(IfElseBody::ElseIf(else_if)) => Some(Box::new(lower_expr(else_if, body, values, types)?)),
                None => None,
            };
            Ok(OperationalNode::If {
                condition: Box::new(condition),
                then_branch: Box::new(then_branch),
                else_branch,
            })
        }
        Expr::Assignment(assignment) => {
            let (_, analysis) = body
                .expressions
                .iter()
                .find(|(_, a)| a.range == expr.range())
                .ok_or_else(|| referential("missing expression analysis for assignment"))?;
            let binding = analysis.binding_write.ok_or_else(|| referential("missing binding_write on assignment"))?;
            let slot = body
                .local_slots
                .get(&binding)
                .copied()
                .ok_or_else(|| referential("missing local slot for binding_write"))?;
            let value = lower_expr(&assignment.value, body, values, types)?;
            Ok(OperationalNode::WriteLocal(slot, Box::new(value)))
        }
        Expr::TupleLiteral(tuple) => {
            let mut elements = Vec::with_capacity(tuple.entries.len());
            for entry in &tuple.entries {
                match entry {
                    TupleLiteralEntry::Positional { expr, .. } => {
                        elements.push(lower_expr(expr, body, values, types)?);
                    }
                    _ => return Err(referential("unsupported labeled/expand tuple literal in static body")),
                }
            }
            Ok(OperationalNode::Tuple { elements })
        }
        Expr::BracketLiteral(bracket) => {
            let (id, _) = body
                .expressions
                .iter()
                .find(|(_, a)| a.range == expr.range())
                .ok_or_else(|| referential("missing expression analysis for bracket literal"))?;
            let resolution = body
                .sequence_literals
                .get(id)
                .ok_or_else(|| referential("missing sequence literal resolution"))?;
            let mut elements = Vec::with_capacity(bracket.elements.len());
            for el in &bracket.elements {
                match el {
                    BracketLiteralElement::Element { expr: e, .. } => {
                        elements.push(lower_expr(e, body, values, types)?);
                    }
                    _ => return Err(referential("unsupported expansion in static sequence literal")),
                }
            }
            Ok(OperationalNode::Sequence {
                result_type: resolution.result_type,
                elements,
            })
        }
        _ => {
            if let Some((id, _)) = body.expressions.iter().find(|(_, a)| a.range == expr.range()) {
                if let Some(construction) = body.static_constructions.get(id) {
                    let target = match &construction.target {
                        crate::identity::InvocationTargetId::DataConstructor(c) => OperationalTarget::DataConstructor(c.owner.clone()),
                        crate::identity::InvocationTargetId::VariantConstructor(v) => OperationalTarget::VariantConstructor(v.variant.clone()),
                        _ => return Err(referential("unsupported invocation target for static construction")),
                    };
                    let mut arguments = Vec::with_capacity(construction.arguments.len());
                    for arg in construction.arguments.iter() {
                        let binding = body.expressions.get(&arg.expression).and_then(|e| e.binding_read);
                        if let Some(binding) = binding {
                            if let Some((&idx, _)) = body.parameter_bindings.iter().find(|(_, b)| **b == binding) {
                                arguments.push(OperationalNode::ReadParameter(idx));
                            } else if let Some(&slot) = body.local_slots.get(&binding) {
                                arguments.push(OperationalNode::ReadLocal(slot));
                            } else {
                                return Err(referential("unbound static construction argument"));
                            }
                        } else {
                            return Err(referential("missing binding read on construction argument"));
                        }
                    }
                    return Ok(OperationalNode::StaticConstruction {
                        target,
                        result_type: construction.result_type,
                        arguments,
                    });
                }
            }
            Err(referential(format!("unsupported expression in static body: {:?}", expr)))
        }
    }
}

fn lower_block(block: &LexicalBlock, body: &CheckedStaticBody, values: &mut StaticValueStore, types: &TypeStore) -> Result<OperationalNode, ArtifactError> {
    let mut statements = Vec::with_capacity(block.body.len());
    for statement in &block.body {
        match statement {
            Statement::Expr { expr, .. } => {
                statements.push(lower_expr(expr, body, values, types)?);
            }
            Statement::Let(binding) => {
                if let Some(init) = &binding.value {
                    let Pattern::Name { name, range } = &binding.pattern else {
                        return Err(referential("unsupported destructuring pattern in static block let"));
                    };
                    let binding_id = body
                        .bindings
                        .iter()
                        .find(|(_, s)| s.name == *name && s.range == *range && body.local_slots.contains_key(&s.binding))
                        .map(|(id, _)| *id)
                        .ok_or_else(|| referential("missing let binding analysis"))?;
                    let slot = body
                        .local_slots
                        .get(&binding_id)
                        .copied()
                        .ok_or_else(|| referential("missing local slot for let binding"))?;
                    let val = lower_expr(init, body, values, types)?;
                    statements.push(OperationalNode::WriteLocal(slot, Box::new(val)));
                }
            }
            Statement::Return(ret) => {
                let val = match &ret.value {
                    Some(e) => Some(Box::new(lower_expr(e, body, values, types)?)),
                    None => None,
                };
                statements.push(OperationalNode::Return(val));
            }
            _ => return Err(referential("unsupported statement in static block")),
        }
    }
    let local_count = body.local_slots.len() as u32;
    Ok(OperationalNode::Block {
        local_count,
        statements,
        result: None,
    })
}

pub fn register_operational_template_roots(
    template: &OperationalTemplate,
    static_context: &mut StaticExportContext,
    type_context: &mut TypeExportContext,
) -> Result<u32, ArtifactError> {
    fn walk(
        node: &OperationalNode,
        static_context: &mut StaticExportContext,
        type_context: &mut TypeExportContext,
        owner: &phalcom_compiler_artifact::semantic::type_store::StableOwnerRef,
        ordinal: &mut u32,
    ) -> Result<(), ArtifactError> {
        use phalcom_compiler_artifact::semantic::type_store::{CanonicalRootFamilyTag, CanonicalTypeRootKey};
        let key = CanonicalTypeRootKey {
            family_tag: CanonicalRootFamilyTag::StaticDefAndBinding,
            owner: owner.clone(),
            product_subkey: *ordinal,
        };
        *ordinal = ordinal.checked_add(1).ok_or_else(|| referential("operational root ordinal overflow"))?;
        match node {
            OperationalNode::Constant(val) => {
                static_context.register_root(key, *val)?;
            }
            OperationalNode::TypeForm(ty) => {
                type_context.register_type_root(type_context.expected_lineage(), key, *ty)?;
            }
            OperationalNode::ReadParameter(_) | OperationalNode::ReadLocal(_) | OperationalNode::ReadStaticConstant(_) => {}
            OperationalNode::WriteLocal(_, val) => {
                walk(val, static_context, type_context, owner, ordinal)?;
            }
            OperationalNode::Binary { left, right, .. } => {
                walk(left, static_context, type_context, owner, ordinal)?;
                walk(right, static_context, type_context, owner, ordinal)?;
            }
            OperationalNode::Unary { operand, .. } => {
                walk(operand, static_context, type_context, owner, ordinal)?;
            }
            OperationalNode::If {
                condition,
                then_branch,
                else_branch,
            } => {
                walk(condition, static_context, type_context, owner, ordinal)?;
                walk(then_branch, static_context, type_context, owner, ordinal)?;
                if let Some(eb) = else_branch {
                    walk(eb, static_context, type_context, owner, ordinal)?;
                }
            }
            OperationalNode::Tuple { elements } => {
                for el in elements {
                    walk(el, static_context, type_context, owner, ordinal)?;
                }
            }
            OperationalNode::Sequence { result_type, elements } => {
                type_context.register_type_root(type_context.expected_lineage(), key, *result_type)?;
                for el in elements {
                    walk(el, static_context, type_context, owner, ordinal)?;
                }
            }
            OperationalNode::StaticConstruction { result_type, arguments, .. } => {
                type_context.register_type_root(type_context.expected_lineage(), key, *result_type)?;
                for arg in arguments {
                    walk(arg, static_context, type_context, owner, ordinal)?;
                }
            }
            OperationalNode::MemberProjection { target, .. } => {
                walk(target, static_context, type_context, owner, ordinal)?;
            }
            OperationalNode::StaticApplication { arguments, .. } => {
                for arg in arguments {
                    walk(arg, static_context, type_context, owner, ordinal)?;
                }
            }
            OperationalNode::Block { statements, result, .. } => {
                for s in statements {
                    walk(s, static_context, type_context, owner, ordinal)?;
                }
                if let Some(res) = result {
                    walk(res, static_context, type_context, owner, ordinal)?;
                }
            }
            OperationalNode::Return(val) => {
                if let Some(v) = val {
                    walk(v, static_context, type_context, owner, ordinal)?;
                }
            }
        }
        Ok(())
    }
    let owner = phalcom_compiler_artifact::semantic::type_store::StableOwnerRef::Declaration(to_stable_declaration_with_context(
        &template.owner.0,
        type_context.identity_context(),
    ));
    let mut ordinal = 0;
    walk(&template.root, static_context, type_context, &owner, &mut ordinal)?;
    Ok(ordinal)
}

fn export_binary_op(op: StaticBinaryOp) -> StaticBinaryOpV1 {
    match op {
        StaticBinaryOp::Add => StaticBinaryOpV1::Add,
        StaticBinaryOp::Sub => StaticBinaryOpV1::Sub,
        StaticBinaryOp::Mul => StaticBinaryOpV1::Mul,
        StaticBinaryOp::Div => StaticBinaryOpV1::Div,
        StaticBinaryOp::Mod => StaticBinaryOpV1::Mod,
        StaticBinaryOp::BitAnd => StaticBinaryOpV1::BitAnd,
        StaticBinaryOp::BitOr => StaticBinaryOpV1::BitOr,
        StaticBinaryOp::BitXor => StaticBinaryOpV1::BitXor,
        StaticBinaryOp::Shl => StaticBinaryOpV1::Shl,
        StaticBinaryOp::Shr => StaticBinaryOpV1::Shr,
        StaticBinaryOp::Eq => StaticBinaryOpV1::Eq,
        StaticBinaryOp::NotEq => StaticBinaryOpV1::NotEq,
        StaticBinaryOp::Less => StaticBinaryOpV1::Less,
        StaticBinaryOp::LessEq => StaticBinaryOpV1::LessEq,
        StaticBinaryOp::Greater => StaticBinaryOpV1::Greater,
        StaticBinaryOp::GreaterEq => StaticBinaryOpV1::GreaterEq,
        StaticBinaryOp::And => StaticBinaryOpV1::And,
        StaticBinaryOp::Or => StaticBinaryOpV1::Or,
    }
}

fn export_unary_op(op: StaticUnaryOp) -> StaticUnaryOpV1 {
    match op {
        StaticUnaryOp::Neg => StaticUnaryOpV1::Neg,
        StaticUnaryOp::Not => StaticUnaryOpV1::Not,
        StaticUnaryOp::BitNot => StaticUnaryOpV1::BitNot,
    }
}

fn export_target(target: &OperationalTarget, type_context: &TypeExportContext<'_>) -> OperationalTargetV1 {
    match target {
        OperationalTarget::Declaration(decl) => OperationalTargetV1::Declaration {
            target: to_stable_declaration_with_context(decl, type_context.identity_context()),
        },
        OperationalTarget::DataConstructor(owner) => OperationalTargetV1::DataConstructor {
            owner: to_stable_declaration_with_context(owner, type_context.identity_context()),
        },
        OperationalTarget::VariantConstructor(variant) => OperationalTargetV1::VariantConstructor {
            owner: to_stable_declaration_with_context(&variant.owner, type_context.identity_context()),
            selector: variant.selector.encode().into_boxed_str(),
        },
    }
}

fn export_op(
    node: &OperationalNode,
    static_context: &StaticExportContext,
    type_context: &TypeExportContext<'_>,
) -> Result<OperationalOpV1, ArtifactError> {
    Ok(match node {
        OperationalNode::Constant(val) => OperationalOpV1::Constant {
            value: static_context.value(*val)?,
        },
        OperationalNode::ReadStaticConstant(symbol) => OperationalOpV1::ReadStaticConstant {
            module: to_stable_module_with_context(&symbol.module, type_context.identity_context()),
            name: symbol.name.to_string(),
        },
        OperationalNode::ReadParameter(index) => OperationalOpV1::ReadParameter { index: *index },
        OperationalNode::ReadLocal(slot) => OperationalOpV1::ReadLocal { slot: *slot },
        OperationalNode::WriteLocal(slot, val) => OperationalOpV1::WriteLocal {
            slot: *slot,
            value: Box::new(export_op(val, static_context, type_context)?),
        },
        OperationalNode::TypeForm(ty) => OperationalOpV1::TypeForm { ty: type_context.ty(*ty)? },
        OperationalNode::Binary { op, left, right } => OperationalOpV1::Binary {
            op: export_binary_op(*op),
            left: Box::new(export_op(left, static_context, type_context)?),
            right: Box::new(export_op(right, static_context, type_context)?),
        },
        OperationalNode::Unary { op, operand } => OperationalOpV1::Unary {
            op: export_unary_op(*op),
            operand: Box::new(export_op(operand, static_context, type_context)?),
        },
        OperationalNode::If {
            condition,
            then_branch,
            else_branch,
        } => OperationalOpV1::If {
            condition: Box::new(export_op(condition, static_context, type_context)?),
            then_branch: Box::new(export_op(then_branch, static_context, type_context)?),
            else_branch: else_branch
                .as_ref()
                .map(|b| export_op(b, static_context, type_context).map(Box::new))
                .transpose()?,
        },
        OperationalNode::Tuple { elements } => OperationalOpV1::Tuple {
            elements: elements
                .iter()
                .map(|e| export_op(e, static_context, type_context))
                .collect::<Result<Vec<_>, _>>()?,
        },
        OperationalNode::Sequence { result_type, elements } => OperationalOpV1::Sequence {
            result_type: type_context.ty(*result_type)?,
            elements: elements
                .iter()
                .map(|e| export_op(e, static_context, type_context))
                .collect::<Result<Vec<_>, _>>()?,
        },
        OperationalNode::StaticConstruction {
            target,
            result_type,
            arguments,
        } => OperationalOpV1::StaticConstruction {
            target: export_target(target, type_context),
            result_type: type_context.ty(*result_type)?,
            arguments: arguments
                .iter()
                .map(|a| export_op(a, static_context, type_context))
                .collect::<Result<Vec<_>, _>>()?,
        },
        OperationalNode::MemberProjection { target, member } => OperationalOpV1::MemberProjection {
            target: Box::new(export_op(target, static_context, type_context)?),
            member: member.clone(),
        },
        OperationalNode::StaticApplication {
            application_ordinal,
            arguments,
        } => OperationalOpV1::StaticApplication {
            application_ordinal: *application_ordinal,
            arguments: arguments
                .iter()
                .map(|a| export_op(a, static_context, type_context))
                .collect::<Result<Vec<_>, _>>()?,
        },
        OperationalNode::Block {
            local_count,
            statements,
            result,
        } => OperationalOpV1::Block {
            local_count: *local_count,
            statements: statements
                .iter()
                .map(|s| export_op(s, static_context, type_context))
                .collect::<Result<Vec<_>, _>>()?,
            result: result.as_ref().map(|r| export_op(r, static_context, type_context).map(Box::new)).transpose()?,
        },
        OperationalNode::Return(val) => OperationalOpV1::Return {
            value: val.as_ref().map(|v| export_op(v, static_context, type_context).map(Box::new)).transpose()?,
        },
    })
}

fn import_binary_op(op: &StaticBinaryOpV1) -> StaticBinaryOp {
    match op {
        StaticBinaryOpV1::Add => StaticBinaryOp::Add,
        StaticBinaryOpV1::Sub => StaticBinaryOp::Sub,
        StaticBinaryOpV1::Mul => StaticBinaryOp::Mul,
        StaticBinaryOpV1::Div => StaticBinaryOp::Div,
        StaticBinaryOpV1::Mod => StaticBinaryOp::Mod,
        StaticBinaryOpV1::BitAnd => StaticBinaryOp::BitAnd,
        StaticBinaryOpV1::BitOr => StaticBinaryOp::BitOr,
        StaticBinaryOpV1::BitXor => StaticBinaryOp::BitXor,
        StaticBinaryOpV1::Shl => StaticBinaryOp::Shl,
        StaticBinaryOpV1::Shr => StaticBinaryOp::Shr,
        StaticBinaryOpV1::Eq => StaticBinaryOp::Eq,
        StaticBinaryOpV1::NotEq => StaticBinaryOp::NotEq,
        StaticBinaryOpV1::Less => StaticBinaryOp::Less,
        StaticBinaryOpV1::LessEq => StaticBinaryOp::LessEq,
        StaticBinaryOpV1::Greater => StaticBinaryOp::Greater,
        StaticBinaryOpV1::GreaterEq => StaticBinaryOp::GreaterEq,
        StaticBinaryOpV1::And => StaticBinaryOp::And,
        StaticBinaryOpV1::Or => StaticBinaryOp::Or,
    }
}

fn import_unary_op(op: &StaticUnaryOpV1) -> StaticUnaryOp {
    match op {
        StaticUnaryOpV1::Neg => StaticUnaryOp::Neg,
        StaticUnaryOpV1::Not => StaticUnaryOp::Not,
        StaticUnaryOpV1::BitNot => StaticUnaryOp::BitNot,
    }
}

fn import_target(target: &OperationalTargetV1, allocations: &mut AllocationBudget) -> Result<OperationalTarget, ArtifactError> {
    Ok(match target {
        OperationalTargetV1::Declaration { target } => OperationalTarget::Declaration(import_stable_declaration_with_allocation_budget(target, allocations)?),
        OperationalTargetV1::DataConstructor { owner } => {
            OperationalTarget::DataConstructor(import_stable_declaration_with_allocation_budget(owner, allocations)?)
        }
        OperationalTargetV1::VariantConstructor { owner, selector } => {
            let decl = import_stable_declaration_with_allocation_budget(owner, allocations)?;
            let v_selector = import_stable_selector_with_allocation_budget(selector, allocations)?;
            OperationalTarget::VariantConstructor(VariantId::new(decl, v_selector))
        }
    })
}

fn import_op(
    op: &OperationalOpV1,
    static_relocation: &StaticRelocation,
    type_relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<OperationalNode, ArtifactError> {
    Ok(match op {
        OperationalOpV1::Constant { value } => OperationalNode::Constant(static_relocation.value(*value)?),
        OperationalOpV1::ReadStaticConstant { module, name } => OperationalNode::ReadStaticConstant(phalcom_modules::linker::SymbolId {
            module: import_stable_module_with_allocation_budget(module, allocations)?,
            name: allocations.boxed_str(name)?,
        }),
        OperationalOpV1::ReadParameter { index } => OperationalNode::ReadParameter(*index),
        OperationalOpV1::ReadLocal { slot } => OperationalNode::ReadLocal(*slot),
        OperationalOpV1::WriteLocal { slot, value } => {
            OperationalNode::WriteLocal(*slot, import_boxed_op(value, static_relocation, type_relocation, allocations)?)
        }
        OperationalOpV1::TypeForm { ty } => OperationalNode::TypeForm(type_relocation.ty(*ty)?),
        OperationalOpV1::Binary { op, left, right } => OperationalNode::Binary {
            op: import_binary_op(op),
            left: import_boxed_op(left, static_relocation, type_relocation, allocations)?,
            right: import_boxed_op(right, static_relocation, type_relocation, allocations)?,
        },
        OperationalOpV1::Unary { op, operand } => OperationalNode::Unary {
            op: import_unary_op(op),
            operand: import_boxed_op(operand, static_relocation, type_relocation, allocations)?,
        },
        OperationalOpV1::If {
            condition,
            then_branch,
            else_branch,
        } => OperationalNode::If {
            condition: import_boxed_op(condition, static_relocation, type_relocation, allocations)?,
            then_branch: import_boxed_op(then_branch, static_relocation, type_relocation, allocations)?,
            else_branch: else_branch
                .as_ref()
                .map(|b| import_boxed_op(b, static_relocation, type_relocation, allocations))
                .transpose()?,
        },
        OperationalOpV1::Tuple { elements } => OperationalNode::Tuple {
            elements: import_ops(elements, static_relocation, type_relocation, allocations)?,
        },
        OperationalOpV1::Sequence { result_type, elements } => OperationalNode::Sequence {
            result_type: type_relocation.ty(*result_type)?,
            elements: import_ops(elements, static_relocation, type_relocation, allocations)?,
        },
        OperationalOpV1::StaticConstruction {
            target,
            result_type,
            arguments,
        } => OperationalNode::StaticConstruction {
            target: import_target(target, allocations)?,
            result_type: type_relocation.ty(*result_type)?,
            arguments: import_ops(arguments, static_relocation, type_relocation, allocations)?,
        },
        OperationalOpV1::MemberProjection { target, member } => OperationalNode::MemberProjection {
            target: import_boxed_op(target, static_relocation, type_relocation, allocations)?,
            member: allocations.boxed_str(member)?,
        },
        OperationalOpV1::StaticApplication {
            application_ordinal,
            arguments,
        } => OperationalNode::StaticApplication {
            application_ordinal: *application_ordinal,
            arguments: import_ops(arguments, static_relocation, type_relocation, allocations)?,
        },
        OperationalOpV1::Block {
            local_count,
            statements,
            result,
        } => OperationalNode::Block {
            local_count: *local_count,
            statements: import_ops(statements, static_relocation, type_relocation, allocations)?,
            result: result
                .as_ref()
                .map(|r| import_boxed_op(r, static_relocation, type_relocation, allocations))
                .transpose()?,
        },
        OperationalOpV1::Return { value } => OperationalNode::Return(
            value
                .as_ref()
                .map(|v| import_boxed_op(v, static_relocation, type_relocation, allocations))
                .transpose()?,
        ),
    })
}

fn import_boxed_op(
    op: &OperationalOpV1,
    static_relocation: &StaticRelocation,
    type_relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<Box<OperationalNode>, ArtifactError> {
    let node = import_op(op, static_relocation, type_relocation, allocations)?;
    Ok(allocations.boxed(node)?)
}

fn import_ops(
    ops: &[OperationalOpV1],
    static_relocation: &StaticRelocation,
    type_relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<Vec<OperationalNode>, ArtifactError> {
    let mut nodes = allocations.vector(ops.len())?;
    for op in ops {
        nodes.push(import_op(op, static_relocation, type_relocation, allocations)?);
    }
    Ok(nodes)
}

fn export_source_range(range: phalcom_common::range::SourceRange) -> Result<phalcom_compiler_artifact::module_world::SpanV1, ArtifactError> {
    Ok(phalcom_compiler_artifact::module_world::SpanV1 {
        start: u64::try_from(range.start).map_err(|_| referential("static definition source start exceeds portable range"))?,
        end: u64::try_from(range.end).map_err(|_| referential("static definition source end exceeds portable range"))?,
    })
}

fn import_source_range(range: &phalcom_compiler_artifact::module_world::SpanV1) -> Result<phalcom_common::range::SourceRange, ArtifactError> {
    if range.start > range.end {
        return Err(referential("static definition source range is inverted"));
    }
    Ok(phalcom_common::range::SourceRange {
        start: usize::try_from(range.start).map_err(|_| referential("static definition source start exceeds address space"))?,
        end: usize::try_from(range.end).map_err(|_| referential("static definition source end exceeds address space"))?,
    })
}

pub(crate) fn export_static_value_domain(
    domain: StaticValueDomain,
    types: &TypeExportContext,
) -> Result<phalcom_compiler_artifact::semantic::world::StaticValueDomainV1, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::StaticValueDomainV1 as Portable;
    Ok(match domain {
        StaticValueDomain::Kind(kind) => Portable::Kind { kind: types.kind(kind)? },
        StaticValueDomain::ValueType(ty) => Portable::ValueType { ty: types.ty(ty)? },
    })
}

pub(crate) fn import_static_value_domain(
    domain: &phalcom_compiler_artifact::semantic::world::StaticValueDomainV1,
    types: &TypeRelocation,
) -> Result<StaticValueDomain, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::StaticValueDomainV1 as Portable;
    Ok(match domain {
        Portable::Kind { kind } => StaticValueDomain::Kind(types.kind(*kind)?),
        Portable::ValueType { ty } => StaticValueDomain::ValueType(types.ty(*ty)?),
    })
}

pub(crate) fn export_static_definition_signature(
    signature: &StaticDefinitionSignature,
    types: &TypeExportContext,
) -> Result<phalcom_compiler_artifact::semantic::world::StaticDefinitionSignatureV1, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::{StaticDefinitionSignatureV1, StaticParameterV1};
    let parameters = signature
        .parameters
        .iter()
        .map(|parameter| {
            Ok(StaticParameterV1 {
                name: parameter.name.clone().into_boxed_str(),
                domain: export_static_value_domain(parameter.domain, types)?,
                mode: crate::artifact::export::export_mode(parameter.mode),
                index: parameter.index,
                type_parameter: parameter.type_parameter_id.map(|id| types.parameter(id)).transpose()?,
                source: export_source_range(parameter.range)?,
            })
        })
        .collect::<Result<Vec<_>, ArtifactError>>()?;
    Ok(StaticDefinitionSignatureV1 {
        parameters,
        result_domain: export_static_value_domain(signature.result_domain, types)?,
        generic_signature: signature
            .generic_signature
            .as_ref()
            .map(|signature| crate::artifact::export::export_generic_signature(signature, types))
            .transpose()?,
    })
}

pub(crate) fn import_static_definition_signature(
    signature: &phalcom_compiler_artifact::semantic::world::StaticDefinitionSignatureV1,
    types: &TypeRelocation,
    store: &TypeStore,
    allocations: &mut AllocationBudget,
) -> Result<StaticDefinitionSignature, ArtifactError> {
    let mut parameters = allocations.vector(signature.parameters.len())?;
    for parameter in &signature.parameters {
        parameters.push(crate::static_definition::StaticParameter {
            name: allocations.string(&parameter.name)?,
            domain: import_static_value_domain(&parameter.domain, types)?,
            mode: crate::artifact::import::import_mode(&parameter.mode),
            index: parameter.index,
            type_parameter_id: parameter.type_parameter.map(|id| types.parameter(id)).transpose()?,
            range: import_source_range(&parameter.source)?,
        });
    }
    Ok(StaticDefinitionSignature {
        parameters: allocations.boxed_slice(parameters)?,
        result_domain: import_static_value_domain(&signature.result_domain, types)?,
        generic_signature: signature
            .generic_signature
            .as_ref()
            .map(|signature| import_generic_signature_with_allocation_budget(signature, types, store, allocations))
            .transpose()?,
        where_clause: None,
    })
}

pub(crate) fn export_canonical_static_program(
    program: &CanonicalStaticProgram,
    static_context: &StaticExportContext,
    type_context: &TypeExportContext,
    dependencies: Vec<phalcom_compiler_artifact::semantic::world::StaticSemanticDependencyV1>,
) -> Result<phalcom_compiler_artifact::semantic::world::CanonicalStaticProgramV1, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::{
        CanonicalCheckedStaticApplicationV1, CanonicalStaticApplicationArgumentV1 as Argument, CanonicalStaticProgramV1, StaticConstantSymbolV1,
    };
    let applications = program
        .applications
        .iter()
        .map(|application| {
            let arguments = application
                .arguments
                .iter()
                .map(|argument| {
                    Ok(match argument {
                        CanonicalStaticApplicationArgument::KindedTerm { parameter_index, term } => Argument::KindedTerm {
                            parameter_index: *parameter_index,
                            term: type_context.ty(*term)?,
                        },
                        CanonicalStaticApplicationArgument::KindedExpression {
                            parameter_index,
                            application_ordinal,
                        } => Argument::KindedExpression {
                            parameter_index: *parameter_index,
                            application_ordinal: *application_ordinal,
                        },
                        CanonicalStaticApplicationArgument::Value { parameter_index } => Argument::Value {
                            parameter_index: *parameter_index,
                        },
                    })
                })
                .collect::<Result<Vec<_>, ArtifactError>>()?;
            Ok(CanonicalCheckedStaticApplicationV1 {
                ordinal: application.id.ordinal,
                source: export_source_range(application.source_range)?,
                target: to_stable_declaration_with_context(&application.target, type_context.identity_context()),
                signature: export_static_definition_signature(&application.signature, type_context)?,
                arguments,
            })
        })
        .collect::<Result<Vec<_>, ArtifactError>>()?;
    let static_constant_reads = program
        .static_constant_reads
        .iter()
        .map(|symbol| StaticConstantSymbolV1 {
            module: to_stable_module_with_context(&symbol.module, type_context.identity_context()),
            name: symbol.name.clone(),
        })
        .collect();
    Ok(CanonicalStaticProgramV1 {
        owner: to_stable_declaration_with_context(&program.owner.0, type_context.identity_context()),
        parameter_count: program.parameter_count,
        local_count: program.local_count,
        root: export_op(&program.root, static_context, type_context)?,
        applications,
        kinded_parameters: program.kinded_parameters.to_vec(),
        static_constant_reads,
        dependencies,
        result: export_static_value_domain(program.result, type_context)?,
        input_fingerprint: program.input_fingerprint.0,
        module_binding_fingerprint: program.module_binding_fingerprint.map(|fingerprint| fingerprint.0),
    })
}

pub(crate) fn import_canonical_static_program(
    program: &phalcom_compiler_artifact::semantic::world::CanonicalStaticProgramV1,
    static_relocation: &StaticRelocation,
    type_relocation: &TypeRelocation,
    type_store: &TypeStore,
    dependencies: Vec<crate::checker::analysis::SemanticDependency>,
    allocations: &mut AllocationBudget,
) -> Result<CanonicalStaticProgram, ArtifactError> {
    use phalcom_compiler_artifact::semantic::world::CanonicalStaticApplicationArgumentV1 as Argument;
    let owner = StaticBodyId(import_stable_declaration_with_allocation_budget(&program.owner, allocations)?);
    let mut applications = allocations.vector(program.applications.len())?;
    for application in &program.applications {
        let mut arguments = allocations.vector(application.arguments.len())?;
        for argument in &application.arguments {
            arguments.push(match argument {
                Argument::KindedTerm { parameter_index, term } => CanonicalStaticApplicationArgument::KindedTerm {
                    parameter_index: *parameter_index,
                    term: type_relocation.ty(*term)?,
                },
                Argument::KindedExpression {
                    parameter_index,
                    application_ordinal,
                } => CanonicalStaticApplicationArgument::KindedExpression {
                    parameter_index: *parameter_index,
                    application_ordinal: *application_ordinal,
                },
                Argument::Value { parameter_index } => CanonicalStaticApplicationArgument::Value {
                    parameter_index: *parameter_index,
                },
            });
        }
        applications.push(CanonicalCheckedStaticApplication {
            id: crate::static_definition::StaticApplicationId {
                body: StaticBodyId(owner.0.clone_with_allocation_budget(allocations)?),
                ordinal: application.ordinal,
            },
            source_range: import_source_range(&application.source)?,
            target: import_stable_declaration_with_allocation_budget(&application.target, allocations)?,
            signature: import_static_definition_signature(&application.signature, type_relocation, type_store, allocations)?,
            arguments: allocations.boxed_slice(arguments)?,
        });
    }
    let mut static_constant_reads = allocations.vector(program.static_constant_reads.len())?;
    for symbol in &program.static_constant_reads {
        static_constant_reads.push(phalcom_modules::linker::SymbolId {
            module: import_stable_module_with_allocation_budget(&symbol.module, allocations)?,
            name: allocations.boxed_str(&symbol.name)?,
        });
    }
    let mut kinded_parameters = allocations.vector(program.kinded_parameters.len())?;
    kinded_parameters.extend_from_slice(&program.kinded_parameters);
    Ok(CanonicalStaticProgram {
        owner,
        parameter_count: program.parameter_count,
        local_count: program.local_count,
        root: import_op(&program.root, static_relocation, type_relocation, allocations)?,
        applications: allocations.arc_slice(applications)?,
        kinded_parameters: allocations.arc_slice(kinded_parameters)?,
        static_constant_reads: allocations.arc_slice(static_constant_reads)?,
        dependencies: allocations.arc_slice(dependencies)?,
        result: import_static_value_domain(&program.result, type_relocation)?,
        input_fingerprint: crate::db::key::InputFingerprint(program.input_fingerprint),
        module_binding_fingerprint: program.module_binding_fingerprint.map(crate::db::ProductFingerprint),
    })
}
