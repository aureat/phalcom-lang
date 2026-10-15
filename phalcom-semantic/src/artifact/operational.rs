//! Artifact export, import, and lowering for checked operational static templates (`PERF003.C2.P7.T7.B`).

use crate::identity::{BindingId, VariantId};
use crate::metadata::stable_identity::{from_stable_declaration, to_stable_declaration};
use crate::static_definition::{
    CheckedStaticApplicationArgument, CheckedStaticBody, OperationalNode, OperationalTarget,
    OperationalTemplate, StaticBinaryOp, StaticDefinition, StaticDefinitionBodyRepr,
    StaticDefinitionProduct, StaticDefinitionSignature, StaticUnaryOp,
};
use crate::static_domain::StaticValueDomain;
use crate::static_eval::machine::canonical_symbol_spelling;
use crate::static_value::{StaticExportContext, StaticFloat, StaticRelocation, StaticValueStore};
use crate::types::store::artifact::{TypeExportContext, TypeRelocation};
use crate::types::store::TypeStore;
use num_bigint::BigInt;
use phalcom_ast::ast::{
    BinaryOp, BracketLiteralElement, Expr, IfElseBody, LexicalBlock, Pattern, Statement, TupleLiteralEntry, UnaryOp,
};
use phalcom_compiler_artifact::semantic::operational::*;
use phalcom_compiler_artifact::{ArtifactError, ValidationError, ValidationLimits};
use std::collections::BTreeMap;
use std::sync::Arc;

fn referential(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}

impl OperationalTemplate {
    pub fn from_checked_body(
        body: &CheckedStaticBody,
        values: &mut StaticValueStore,
        types: &TypeStore,
    ) -> Result<Self, ArtifactError> {
        let parameter_count = body
            .parameter_bindings
            .keys()
            .chain(body.kinded_parameters.keys())
            .max()
            .map_or(0, |idx| *idx + 1);
        let local_count = body.local_slots.len() as u32;
        let owner = body.owner.0.clone();

        let root = match &body.root {
            StaticDefinitionBodyRepr::Operational(template) => return Ok((**template).clone()),
            StaticDefinitionBodyRepr::Expression(expr) => lower_expr(expr, body, values, types)?,
            StaticDefinitionBodyRepr::Block(block) => lower_block(block, body, values, types)?,
            StaticDefinitionBodyRepr::TypeForm(_) => {
                if let Some(app) = body.static_applications.first() {
                    let mut arguments = Vec::with_capacity(app.arguments.len());
                    for arg in app.arguments.iter() {
                        match arg {
                            CheckedStaticApplicationArgument::Kinded { term, .. } => {
                                arguments.push(OperationalNode::TypeForm(*term));
                            }
                            CheckedStaticApplicationArgument::Value { expression, .. } => {
                                let binding = body.expressions.get(expression).and_then(|e| e.binding_read);
                                if let Some(binding) = binding {
                                    if let Some((&idx, _)) = body.parameter_bindings.iter().find(|(_, b)| **b == binding) {
                                        arguments.push(OperationalNode::ReadParameter(idx));
                                    } else if let Some(&slot) = body.local_slots.get(&binding) {
                                        arguments.push(OperationalNode::ReadLocal(slot));
                                    } else {
                                        return Err(referential("unbound static arg binding"));
                                    }
                                } else {
                                    return Err(referential("missing binding for static argument"));
                                }
                            }
                            CheckedStaticApplicationArgument::KindedExpression { .. } => {
                                return Err(referential("unsupported kinded expression in static application"));
                            }
                        }
                    }
                    OperationalNode::StaticApplication {
                        target: app.target.clone(),
                        arguments,
                    }
                } else if let StaticValueDomain::Kind(_) = body.result {
                    if let Some((_, analysis)) = body.expressions.iter().next() {
                        if let Some(
                            crate::types::denotation::SemanticDenotation::TypeForm(ty)
                            | crate::types::denotation::SemanticDenotation::StaticTerm(ty),
                        ) = analysis.denotation.as_ref() {
                            OperationalNode::TypeForm(*ty)
                        } else {
                            return Err(referential("missing type denotation on static type form"));
                        }
                    } else {
                        return Err(referential("empty expressions on static type form"));
                    }
                } else {
                    return Err(referential("unsupported static type form body"));
                }
            }
        };

        Ok(OperationalTemplate {
            owner,
            parameter_count,
            local_count,
            root,
        })
    }
}

fn lower_expr(
    expr: &Expr,
    body: &CheckedStaticBody,
    values: &mut StaticValueStore,
    types: &TypeStore,
) -> Result<OperationalNode, ArtifactError> {
    match expr {
        Expr::Int { digits, radix, .. } => {
            let integer = BigInt::parse_bytes(digits.as_bytes(), *radix)
                .ok_or_else(|| referential("invalid integer literal"))?;
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
            let spelling = canonical_symbol_spelling(&symbol.kind)
                .ok_or_else(|| referential("unsupported symbol literal"))?;
            let val = values.intern_symbol(spelling);
            Ok(OperationalNode::Constant(val))
        }
        Expr::Var { .. } => {
            let (_, analysis) = body
                .expressions
                .iter()
                .find(|(_, a)| a.range == expr.range())
                .ok_or_else(|| referential("missing expression analysis for var"))?;
            if let Some(
                crate::types::denotation::SemanticDenotation::TypeForm(ty)
                | crate::types::denotation::SemanticDenotation::StaticTerm(ty),
            ) = analysis.denotation.as_ref()
            {
                Ok(OperationalNode::TypeForm(*ty))
            } else if let Some(binding) = analysis.binding_read {
                if let Some((&index, _)) = body.parameter_bindings.iter().find(|(_, b)| **b == binding) {
                    Ok(OperationalNode::ReadParameter(index))
                } else if let Some(&slot) = body.local_slots.get(&binding) {
                    Ok(OperationalNode::ReadLocal(slot))
                } else {
                    Err(referential(format!("unbound binding {:?}", binding)))
                }
            } else {
                Err(referential("var has neither type denotation nor binding read"))
            }
        }
        Expr::TypeForm(_) => {
            let expr_id = body
                .expressions
                .iter()
                .find(|(_, a)| a.range == expr.range())
                .map(|(id, _)| *id);
            if let Some(id) = expr_id {
                if let Some(app) = body.static_applications.iter().find(|a| a.expression == id) {
                    let mut arguments = Vec::with_capacity(app.arguments.len());
                    for arg in app.arguments.iter() {
                        match arg {
                            CheckedStaticApplicationArgument::Kinded { term, .. } => {
                                arguments.push(OperationalNode::TypeForm(*term));
                            }
                            CheckedStaticApplicationArgument::Value { expression, .. } => {
                                let binding = body.expressions.get(expression).and_then(|e| e.binding_read);
                                if let Some(binding) = binding {
                                    if let Some((&idx, _)) = body.parameter_bindings.iter().find(|(_, b)| **b == binding) {
                                        arguments.push(OperationalNode::ReadParameter(idx));
                                    } else if let Some(&slot) = body.local_slots.get(&binding) {
                                        arguments.push(OperationalNode::ReadLocal(slot));
                                    } else {
                                        return Err(referential("unbound static arg binding"));
                                    }
                                } else {
                                    return Err(referential("missing binding for static argument"));
                                }
                            }
                            CheckedStaticApplicationArgument::KindedExpression { .. } => {
                                return Err(referential("unsupported kinded expression in static application"));
                            }
                        }
                    }
                    return Ok(OperationalNode::StaticApplication {
                        target: app.target.clone(),
                        arguments,
                    });
                }
                if let Some(analysis) = body.expressions.get(&id) {
                    if let Some(
                        crate::types::denotation::SemanticDenotation::TypeForm(ty)
                        | crate::types::denotation::SemanticDenotation::StaticTerm(ty),
                    ) = analysis.denotation.as_ref()
                    {
                        return Ok(OperationalNode::TypeForm(*ty));
                    }
                }
            }
            Err(referential("unresolved type form expression"))
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
            let binding = analysis
                .binding_write
                .ok_or_else(|| referential("missing binding_write on assignment"))?;
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
                        crate::identity::InvocationTargetId::DataConstructor(c) => {
                            OperationalTarget::DataConstructor(c.owner.clone())
                        }
                        crate::identity::InvocationTargetId::VariantConstructor(v) => {
                            OperationalTarget::VariantConstructor(v.variant.clone())
                        }
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

fn lower_block(
    block: &LexicalBlock,
    body: &CheckedStaticBody,
    values: &mut StaticValueStore,
    types: &TypeStore,
) -> Result<OperationalNode, ArtifactError> {
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
        use phalcom_compiler_artifact::semantic::type_store::{CanonicalTypeRootKey, CanonicalRootFamilyTag};
        let key = CanonicalTypeRootKey { family_tag: CanonicalRootFamilyTag::StaticDefAndBinding, owner: owner.clone(), product_subkey: *ordinal };
        *ordinal = ordinal.checked_add(1).ok_or_else(|| referential("operational root ordinal overflow"))?;
        match node {
            OperationalNode::Constant(val) => {
                static_context.register_root(key, *val)?;
            }
            OperationalNode::TypeForm(ty) => {
                type_context.register_type_root(type_context.expected_lineage(), key, *ty)?;
            }
            OperationalNode::ReadParameter(_) | OperationalNode::ReadLocal(_) => {}
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
            OperationalNode::If { condition, then_branch, else_branch } => {
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
    let owner = phalcom_compiler_artifact::semantic::type_store::StableOwnerRef::Declaration(to_stable_declaration(&template.owner));
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

fn export_target(target: &OperationalTarget) -> OperationalTargetV1 {
    match target {
        OperationalTarget::Declaration(decl) => OperationalTargetV1::Declaration {
            target: to_stable_declaration(decl),
        },
        OperationalTarget::DataConstructor(owner) => OperationalTargetV1::DataConstructor {
            owner: to_stable_declaration(owner),
        },
        OperationalTarget::VariantConstructor(variant) => OperationalTargetV1::VariantConstructor {
            owner: to_stable_declaration(&variant.owner),
            selector: variant.selector.encode().into_boxed_str(),
        },
    }
}

fn export_op(
    node: &OperationalNode,
    static_context: &StaticExportContext,
    type_context: &TypeExportContext,
) -> Result<OperationalOpV1, ArtifactError> {
    Ok(match node {
        OperationalNode::Constant(val) => OperationalOpV1::Constant {
            value: static_context.value(*val)?,
        },
        OperationalNode::ReadParameter(index) => OperationalOpV1::ReadParameter { index: *index },
        OperationalNode::ReadLocal(slot) => OperationalOpV1::ReadLocal { slot: *slot },
        OperationalNode::WriteLocal(slot, val) => OperationalOpV1::WriteLocal {
            slot: *slot,
            value: Box::new(export_op(val, static_context, type_context)?),
        },
        OperationalNode::TypeForm(ty) => OperationalOpV1::TypeForm {
            ty: type_context.ty(*ty)?,
        },
        OperationalNode::Binary { op, left, right } => OperationalOpV1::Binary {
            op: export_binary_op(*op),
            left: Box::new(export_op(left, static_context, type_context)?),
            right: Box::new(export_op(right, static_context, type_context)?),
        },
        OperationalNode::Unary { op, operand } => OperationalOpV1::Unary {
            op: export_unary_op(*op),
            operand: Box::new(export_op(operand, static_context, type_context)?),
        },
        OperationalNode::If { condition, then_branch, else_branch } => OperationalOpV1::If {
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
        OperationalNode::StaticConstruction { target, result_type, arguments } => {
            OperationalOpV1::StaticConstruction {
                target: export_target(target),
                result_type: type_context.ty(*result_type)?,
                arguments: arguments
                    .iter()
                    .map(|a| export_op(a, static_context, type_context))
                    .collect::<Result<Vec<_>, _>>()?,
            }
        }
        OperationalNode::MemberProjection { target, member } => OperationalOpV1::MemberProjection {
            target: Box::new(export_op(target, static_context, type_context)?),
            member: member.clone(),
        },
        OperationalNode::StaticApplication { target, arguments } => {
            OperationalOpV1::StaticApplication {
                target: to_stable_declaration(target),
                arguments: arguments
                    .iter()
                    .map(|a| export_op(a, static_context, type_context))
                    .collect::<Result<Vec<_>, _>>()?,
            }
        }
        OperationalNode::Block { local_count, statements, result } => OperationalOpV1::Block {
            local_count: *local_count,
            statements: statements
                .iter()
                .map(|s| export_op(s, static_context, type_context))
                .collect::<Result<Vec<_>, _>>()?,
            result: result
                .as_ref()
                .map(|r| export_op(r, static_context, type_context).map(Box::new))
                .transpose()?,
        },
        OperationalNode::Return(val) => OperationalOpV1::Return {
            value: val
                .as_ref()
                .map(|v| export_op(v, static_context, type_context).map(Box::new))
                .transpose()?,
        },
    })
}

pub fn export_checked_static_definitions(
    templates: &[OperationalTemplate],
    static_context: &StaticExportContext,
    type_context: &TypeExportContext,
    limits: &ValidationLimits,
) -> Result<CheckedStaticDefinitionsV1, ArtifactError> {
    let mut definitions = Vec::with_capacity(templates.len());
    for template in templates {
        let owner = to_stable_declaration(&template.owner);
        let root = export_op(&template.root, static_context, type_context)?;
        definitions.push(OperationalTemplateV1 {
            owner,
            parameter_count: template.parameter_count,
            local_count: template.local_count,
            root,
        });
    }
    let product = CheckedStaticDefinitionsV1 { definitions };
    product.validate(limits)?;
    Ok(product)
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

fn import_target(target: &OperationalTargetV1) -> Result<OperationalTarget, ArtifactError> {
    Ok(match target {
        OperationalTargetV1::Declaration { target } => OperationalTarget::Declaration(
            from_stable_declaration(target).map_err(referential)?,
        ),
        OperationalTargetV1::DataConstructor { owner } => OperationalTarget::DataConstructor(
            from_stable_declaration(owner).map_err(referential)?,
        ),
        OperationalTargetV1::VariantConstructor { owner, selector } => {
            let decl = from_stable_declaration(owner).map_err(referential)?;
            let v_selector = phalcom_common::selector::Selector::decode(selector);
            OperationalTarget::VariantConstructor(VariantId::new(decl, v_selector))
        }
    })
}

fn import_op(
    op: &OperationalOpV1,
    static_relocation: &StaticRelocation,
    type_relocation: &TypeRelocation,
) -> Result<OperationalNode, ArtifactError> {
    Ok(match op {
        OperationalOpV1::Constant { value } => {
            OperationalNode::Constant(static_relocation.value(*value)?)
        }
        OperationalOpV1::ReadParameter { index } => OperationalNode::ReadParameter(*index),
        OperationalOpV1::ReadLocal { slot } => OperationalNode::ReadLocal(*slot),
        OperationalOpV1::WriteLocal { slot, value } => OperationalNode::WriteLocal(
            *slot,
            Box::new(import_op(value, static_relocation, type_relocation)?),
        ),
        OperationalOpV1::TypeForm { ty } => {
            OperationalNode::TypeForm(type_relocation.ty(*ty)?)
        }
        OperationalOpV1::Binary { op, left, right } => OperationalNode::Binary {
            op: import_binary_op(op),
            left: Box::new(import_op(left, static_relocation, type_relocation)?),
            right: Box::new(import_op(right, static_relocation, type_relocation)?),
        },
        OperationalOpV1::Unary { op, operand } => OperationalNode::Unary {
            op: import_unary_op(op),
            operand: Box::new(import_op(operand, static_relocation, type_relocation)?),
        },
        OperationalOpV1::If { condition, then_branch, else_branch } => OperationalNode::If {
            condition: Box::new(import_op(condition, static_relocation, type_relocation)?),
            then_branch: Box::new(import_op(then_branch, static_relocation, type_relocation)?),
            else_branch: else_branch
                .as_ref()
                .map(|b| import_op(b, static_relocation, type_relocation).map(Box::new))
                .transpose()?,
        },
        OperationalOpV1::Tuple { elements } => OperationalNode::Tuple {
            elements: elements
                .iter()
                .map(|e| import_op(e, static_relocation, type_relocation))
                .collect::<Result<Vec<_>, _>>()?,
        },
        OperationalOpV1::Sequence { result_type, elements } => OperationalNode::Sequence {
            result_type: type_relocation.ty(*result_type)?,
            elements: elements
                .iter()
                .map(|e| import_op(e, static_relocation, type_relocation))
                .collect::<Result<Vec<_>, _>>()?,
        },
        OperationalOpV1::StaticConstruction { target, result_type, arguments } => {
            OperationalNode::StaticConstruction {
                target: import_target(target)?,
                result_type: type_relocation.ty(*result_type)?,
                arguments: arguments
                    .iter()
                    .map(|a| import_op(a, static_relocation, type_relocation))
                    .collect::<Result<Vec<_>, _>>()?,
            }
        }
        OperationalOpV1::MemberProjection { target, member } => {
            OperationalNode::MemberProjection {
                target: Box::new(import_op(target, static_relocation, type_relocation)?),
                member: member.clone(),
            }
        }
        OperationalOpV1::StaticApplication { target, arguments } => {
            OperationalNode::StaticApplication {
                target: from_stable_declaration(target).map_err(referential)?,
                arguments: arguments
                    .iter()
                    .map(|a| import_op(a, static_relocation, type_relocation))
                    .collect::<Result<Vec<_>, _>>()?,
            }
        }
        OperationalOpV1::Block { local_count, statements, result } => OperationalNode::Block {
            local_count: *local_count,
            statements: statements
                .iter()
                .map(|s| import_op(s, static_relocation, type_relocation))
                .collect::<Result<Vec<_>, _>>()?,
            result: result
                .as_ref()
                .map(|r| import_op(r, static_relocation, type_relocation).map(Box::new))
                .transpose()?,
        },
        OperationalOpV1::Return { value } => OperationalNode::Return(
            value
                .as_ref()
                .map(|v| import_op(v, static_relocation, type_relocation).map(Box::new))
                .transpose()?,
        ),
    })
}

pub fn import_checked_static_definitions(
    definitions: &CheckedStaticDefinitionsV1,
    static_relocation: &StaticRelocation,
    type_relocation: &TypeRelocation,
    limits: &ValidationLimits,
) -> Result<Vec<OperationalTemplate>, ArtifactError> {
    definitions.validate(limits)?;
    let mut templates = Vec::with_capacity(definitions.definitions.len());
    for def in &definitions.definitions {
        let owner = from_stable_declaration(&def.owner).map_err(referential)?;
        let root = import_op(&def.root, static_relocation, type_relocation)?;
        templates.push(OperationalTemplate {
            owner,
            parameter_count: def.parameter_count,
            local_count: def.local_count,
            root,
        });
    }
    Ok(templates)
}

pub fn template_to_static_definition(
    template: OperationalTemplate,
    signature: StaticDefinitionSignature,
) -> StaticDefinitionProduct {
    let owner_decl = template.owner.clone();
    let body_repr = StaticDefinitionBodyRepr::Operational(Arc::new(template.clone()));
    let definition = Arc::new(StaticDefinition {
        id: owner_decl.clone(),
        signature: signature.clone(),
        body: body_repr.clone(),
        source_range: phalcom_common::range::SourceRange::default(),
    });

    let mut parameter_bindings = BTreeMap::new();
    for i in 0..template.parameter_count {
        parameter_bindings.insert(i, BindingId(i));
    }
    let mut local_slots = BTreeMap::new();
    for slot in 0..template.local_count {
        local_slots.insert(BindingId(slot), slot);
    }

    let checked_body = Arc::new(CheckedStaticBody {
        owner: crate::static_definition::StaticBodyId(owner_decl),
        root: body_repr,
        expressions: Arc::new(Default::default()),
        bindings: Arc::new(Default::default()),
        static_operations: Arc::new(Default::default()),
        static_applications: Arc::new([]),
        static_constructions: Arc::new(Default::default()),
        member_projections: Arc::new(Default::default()),
        associated_resolutions: Arc::new(Default::default()),
        sequence_literals: Arc::new(Default::default()),
        match_resolutions: Arc::new(Default::default()),
        pattern_site_resolutions: Arc::new(Default::default()),
        static_iteration_lanes: Arc::new(Default::default()),
        proof_predicates: Arc::new(Default::default()),
        parameter_bindings: Arc::new(parameter_bindings),
        kinded_parameters: Arc::new(Default::default()),
        local_slots: Arc::new(local_slots),
        static_constant_bindings: Arc::new(Default::default()),
        static_constant_reads: Arc::new([]),
        module_binding_fingerprint: None,
        input_fingerprint: crate::db::key::InputFingerprint(0),
        dependencies: Arc::new([]),
        result: signature.result_domain,
        diagnostics: Arc::new([]),
    });

    StaticDefinitionProduct {
        definition,
        diagnostics: Arc::new([]),
        alias_info: None,
        checked_body: Some(checked_body),
    }
}
