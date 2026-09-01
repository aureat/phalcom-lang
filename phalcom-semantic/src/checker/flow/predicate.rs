use crate::checker::context::CheckingContext;
use crate::checker::typed_expr::TypedExpression;
use crate::identity::{BindingId, CallableId, PredicateId};
use crate::types::id::TypeId;
use phalcom_ast::ast::{BinaryOp, Expr, PackItem, UnaryOp};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PredicateAuthority {
    /// The condition itself supplies a runtime/compiler-trusted observation.
    AuthoritativeObservation,
    /// Refinement depends on existing formal knowledge and may not strengthen it.
    DerivedFilter,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustedFlowPredicate {
    pub predicate: FlowPredicate,
    pub authority: PredicateAuthority,
}

/// A formal predicate asserted on a control flow path.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FlowPredicate {
    /// Binding is known to be an instance of `target`.
    IsInstance { binding: BindingId, target: TypeId },
    /// Binding is known NOT to be an instance of `target`.
    IsNotInstance { binding: BindingId, target: TypeId },
    /// Binding is known to be nil/Unit.
    IsNil { binding: BindingId },
    /// Binding is known not to be nil.
    NotNil { binding: BindingId },
    /// Binding is known to equal a specific value/type.
    Equal { binding: BindingId, target: TypeId },
    /// Binding is known NOT to equal a specific value/type.
    NotEqual { binding: BindingId, target: TypeId },
    /// Binding equals a literal constant.
    EqualLiteral { binding: BindingId, literal: String },
    /// Binding does not equal a literal constant.
    NotEqualLiteral { binding: BindingId, literal: String },
    /// Ordered comparison predicate (e.g. `amount > 0`).
    OrderedPredicate { binding: BindingId, op: String, threshold: i64 },
    /// Condition is true on this branch.
    Truthy { binding: BindingId },
    /// Condition is false on this branch.
    Falsy { binding: BindingId },
}

impl FlowPredicate {
    /// Returns the lexical binding identity affected by this predicate, if any.
    pub fn binding(&self) -> Option<BindingId> {
        match self {
            FlowPredicate::IsInstance { binding, .. }
            | FlowPredicate::IsNotInstance { binding, .. }
            | FlowPredicate::IsNil { binding }
            | FlowPredicate::NotNil { binding }
            | FlowPredicate::Equal { binding, .. }
            | FlowPredicate::NotEqual { binding, .. }
            | FlowPredicate::EqualLiteral { binding, .. }
            | FlowPredicate::NotEqualLiteral { binding, .. }
            | FlowPredicate::OrderedPredicate { binding, .. }
            | FlowPredicate::Truthy { binding }
            | FlowPredicate::Falsy { binding } => Some(*binding),
        }
    }

    /// Logical inversion of this predicate on an alternate branch.
    pub fn invert(&self) -> Option<Self> {
        match self {
            FlowPredicate::IsInstance { binding, target } => Some(FlowPredicate::IsNotInstance {
                binding: *binding,
                target: *target,
            }),
            FlowPredicate::IsNotInstance { binding, target } => Some(FlowPredicate::IsInstance {
                binding: *binding,
                target: *target,
            }),
            FlowPredicate::IsNil { binding } => Some(FlowPredicate::NotNil { binding: *binding }),
            FlowPredicate::NotNil { binding } => Some(FlowPredicate::IsNil { binding: *binding }),
            FlowPredicate::Equal { binding, target } => Some(FlowPredicate::NotEqual {
                binding: *binding,
                target: *target,
            }),
            FlowPredicate::NotEqual { binding, target } => Some(FlowPredicate::Equal {
                binding: *binding,
                target: *target,
            }),
            FlowPredicate::EqualLiteral { binding, literal } => Some(FlowPredicate::NotEqualLiteral {
                binding: *binding,
                literal: literal.clone(),
            }),
            FlowPredicate::NotEqualLiteral { binding, literal } => Some(FlowPredicate::EqualLiteral {
                binding: *binding,
                literal: literal.clone(),
            }),
            FlowPredicate::Truthy { binding } => Some(FlowPredicate::Falsy { binding: *binding }),
            FlowPredicate::Falsy { binding } => Some(FlowPredicate::Truthy { binding: *binding }),
            FlowPredicate::OrderedPredicate { binding, op, threshold } => {
                let inv_op = match op.as_str() {
                    ">" => "<=",
                    "<" => ">=",
                    ">=" => "<",
                    "<=" => ">",
                    "==" => "!=",
                    "!=" => "==",
                    _ => return None,
                };
                Some(FlowPredicate::OrderedPredicate {
                    binding: *binding,
                    op: inv_op.into(),
                    threshold: *threshold,
                })
            }
        }
    }

    /// Wraps this predicate with authoritative runtime observation authority.
    pub fn authoritative(self) -> TrustedFlowPredicate {
        TrustedFlowPredicate {
            predicate: self,
            authority: PredicateAuthority::AuthoritativeObservation,
        }
    }

    /// Wraps this predicate with derived filtering authority.
    pub fn derived(self) -> TrustedFlowPredicate {
        TrustedFlowPredicate {
            predicate: self,
            authority: PredicateAuthority::DerivedFilter,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PredicateEntry {
    pub id: PredicateId,
    pub predicate: FlowPredicate,
}

/// Extracts a direct flow predicate from a conditional expression without authority validation.
pub fn extract_predicate_shape(ctx: &mut CheckingContext<'_>, expr: &Expr, truth: bool) -> Option<FlowPredicate> {
    match expr {
        Expr::Unary(unary) if matches!(unary.op, UnaryOp::Not) => extract_predicate_shape(ctx, &unary.expr, !truth),
        Expr::MethodCall(call) => match call.method.as_str() {
            "is" | "is!" => {
                let Expr::Var { value: name, .. } = &call.object else { return None };
                let binding = ctx.lookup_binding_info(name)?.id;
                let target_ty = match call.args.first()? {
                    PackItem::Positional {
                        expr: Expr::Var { value: type_name, .. },
                        ..
                    } => {
                        let decl = ctx.resolve_type_name(type_name)?;
                        ctx.nominal_type_of(&decl)?
                    }
                    _ => return None,
                };
                if truth {
                    Some(FlowPredicate::IsInstance { binding, target: target_ty })
                } else {
                    Some(FlowPredicate::IsNotInstance { binding, target: target_ty })
                }
            }
            "==" => {
                let Expr::Var { value: name, .. } = &call.object else { return None };
                let binding = ctx.lookup_binding_info(name)?.id;
                let arg = call.args.first()?;
                let PackItem::Positional { expr: rhs, .. } = arg else { return None };
                match rhs {
                    Expr::Var { value: rname, .. } if rname == "None" => {
                        if truth {
                            Some(FlowPredicate::IsNil { binding })
                        } else {
                            Some(FlowPredicate::NotNil { binding })
                        }
                    }
                    Expr::Int { digits, .. } => {
                        if truth {
                            Some(FlowPredicate::EqualLiteral {
                                binding,
                                literal: digits.clone(),
                            })
                        } else {
                            Some(FlowPredicate::NotEqualLiteral {
                                binding,
                                literal: digits.clone(),
                            })
                        }
                    }
                    _ => None,
                }
            }
            "!=" => {
                let Expr::Var { value: name, .. } = &call.object else { return None };
                let binding = ctx.lookup_binding_info(name)?.id;
                let arg = call.args.first()?;
                let PackItem::Positional { expr: rhs, .. } = arg else { return None };
                match rhs {
                    Expr::Var { value: rname, .. } if rname == "None" => {
                        if truth {
                            Some(FlowPredicate::NotNil { binding })
                        } else {
                            Some(FlowPredicate::IsNil { binding })
                        }
                    }
                    Expr::Int { digits, .. } => {
                        if truth {
                            Some(FlowPredicate::NotEqualLiteral {
                                binding,
                                literal: digits.clone(),
                            })
                        } else {
                            Some(FlowPredicate::EqualLiteral {
                                binding,
                                literal: digits.clone(),
                            })
                        }
                    }
                    _ => None,
                }
            }
            _ => None,
        },
        Expr::Binary(binary) => match binary.op {
            BinaryOp::Equal | BinaryOp::Same => {
                if let Expr::Var { value: name, .. } = &binary.left {
                    let binding = ctx.lookup_binding_info(name)?.id;
                    if let Expr::Var { value: rname, .. } = &binary.right {
                        if rname == "None" {
                            return if truth {
                                Some(FlowPredicate::IsNil { binding })
                            } else {
                                Some(FlowPredicate::NotNil { binding })
                            };
                        }
                    } else if let Expr::Int { digits, .. } = &binary.right {
                        return if truth {
                            Some(FlowPredicate::EqualLiteral {
                                binding,
                                literal: digits.clone(),
                            })
                        } else {
                            Some(FlowPredicate::NotEqualLiteral {
                                binding,
                                literal: digits.clone(),
                            })
                        };
                    }
                } else if let Expr::Var { value: name, .. } = &binary.right {
                    let binding = ctx.lookup_binding_info(name)?.id;
                    if let Expr::Var { value: lname, .. } = &binary.left {
                        if lname == "None" {
                            return if truth {
                                Some(FlowPredicate::IsNil { binding })
                            } else {
                                Some(FlowPredicate::NotNil { binding })
                            };
                        }
                    }
                }
                None
            }
            BinaryOp::NotEqual => {
                if let Expr::Var { value: name, .. } = &binary.left {
                    let binding = ctx.lookup_binding_info(name)?.id;
                    if let Expr::Var { value: rname, .. } = &binary.right {
                        if rname == "None" {
                            return if truth {
                                Some(FlowPredicate::NotNil { binding })
                            } else {
                                Some(FlowPredicate::IsNil { binding })
                            };
                        }
                    } else if let Expr::Int { digits, .. } = &binary.right {
                        return if truth {
                            Some(FlowPredicate::NotEqualLiteral {
                                binding,
                                literal: digits.clone(),
                            })
                        } else {
                            Some(FlowPredicate::EqualLiteral {
                                binding,
                                literal: digits.clone(),
                            })
                        };
                    }
                } else if let Expr::Var { value: name, .. } = &binary.right {
                    let binding = ctx.lookup_binding_info(name)?.id;
                    if let Expr::Var { value: lname, .. } = &binary.left {
                        if lname == "None" {
                            return if truth {
                                Some(FlowPredicate::IsNil { binding })
                            } else {
                                Some(FlowPredicate::NotNil { binding })
                            };
                        }
                    }
                }
                None
            }
            BinaryOp::GreaterThan | BinaryOp::GreaterThanOrEqual | BinaryOp::LessThan | BinaryOp::LessThanOrEqual => {
                if let Expr::Var { value: name, .. } = &binary.left {
                    let binding = ctx.lookup_binding_info(name)?.id;
                    if let Expr::Int { digits, .. } = &binary.right {
                        if let Ok(threshold) = digits.parse::<i64>() {
                            let op_str = match binary.op {
                                BinaryOp::GreaterThan => {
                                    if truth {
                                        ">"
                                    } else {
                                        "<="
                                    }
                                }
                                BinaryOp::GreaterThanOrEqual => {
                                    if truth {
                                        ">="
                                    } else {
                                        "<"
                                    }
                                }
                                BinaryOp::LessThan => {
                                    if truth {
                                        "<"
                                    } else {
                                        ">="
                                    }
                                }
                                BinaryOp::LessThanOrEqual => {
                                    if truth {
                                        "<="
                                    } else {
                                        ">"
                                    }
                                }
                                _ => unreachable!(),
                            };
                            return Some(FlowPredicate::OrderedPredicate {
                                binding,
                                op: op_str.into(),
                                threshold,
                            });
                        }
                    }
                }
                None
            }
            _ => None,
        },
        Expr::Var { value: name, .. } => {
            let binding = ctx.lookup_binding_info(name)?.id;
            if truth {
                Some(FlowPredicate::Truthy { binding })
            } else {
                Some(FlowPredicate::Falsy { binding })
            }
        }
        _ => None,
    }
}

/// Extracts a trusted flow predicate only when semantic identity authorizes formal proof.
pub fn extract_trusted_predicate(
    ctx: &mut CheckingContext<'_>,
    condition: &Expr,
    condition_typed: &TypedExpression,
    truth: bool,
) -> Option<TrustedFlowPredicate> {
    let predicate = extract_predicate_shape(ctx, condition, truth)?;
    let callable = resolve_predicate_callable(ctx, condition, condition_typed);
    match &predicate {
        FlowPredicate::IsInstance { .. } | FlowPredicate::IsNotInstance { .. } => {
            if is_canonical_type_test(ctx, callable.as_ref()) {
                Some(TrustedFlowPredicate {
                    predicate,
                    authority: PredicateAuthority::AuthoritativeObservation,
                })
            } else {
                None
            }
        }
        FlowPredicate::IsNil { .. }
        | FlowPredicate::NotNil { .. }
        | FlowPredicate::Equal { .. }
        | FlowPredicate::NotEqual { .. }
        | FlowPredicate::EqualLiteral { .. }
        | FlowPredicate::NotEqualLiteral { .. } => {
            if is_canonical_equality(ctx, callable.as_ref()) {
                Some(TrustedFlowPredicate {
                    predicate,
                    authority: PredicateAuthority::DerivedFilter,
                })
            } else {
                None
            }
        }
        FlowPredicate::OrderedPredicate { .. } => {
            if is_canonical_ordered_comparison(ctx, callable.as_ref()) {
                Some(TrustedFlowPredicate {
                    predicate,
                    authority: PredicateAuthority::DerivedFilter,
                })
            } else {
                None
            }
        }
        FlowPredicate::Truthy { .. } | FlowPredicate::Falsy { .. } => {
            if matches!(condition, Expr::Var { .. }) {
                Some(TrustedFlowPredicate {
                    predicate,
                    authority: PredicateAuthority::DerivedFilter,
                })
            } else {
                None
            }
        }
    }
}

fn resolve_predicate_callable(ctx: &mut CheckingContext<'_>, condition: &Expr, condition_typed: &TypedExpression) -> Option<CallableId> {
    if let Some(ref callable) = condition_typed.callable {
        if is_canonical_type_test(ctx, Some(callable)) {
            return Some(callable.clone());
        }
        if is_canonical_equality(ctx, Some(callable)) || is_canonical_ordered_comparison(ctx, Some(callable)) {
            return Some(callable.clone());
        }
    }
    match condition {
        Expr::Unary(unary) if matches!(unary.op, UnaryOp::Not) => match &unary.expr {
            Expr::MethodCall(call) => {
                let Expr::Var { value: name, .. } = &call.object else { return None };
                let binding = ctx.lookup_binding_info(name)?;
                let receiver_ty = ctx
                    .flow
                    .get_current_type(binding.id)
                    .and_then(|k| k.ty())
                    .or_else(|| ctx.core_type(&ctx.core_ids.object.clone()))?;
                let selector =
                    phalcom_common::selector::Selector::method(call.method.as_str(), vec![phalcom_common::selector::SelectorSlot::Positional]).ok()?;
                let target = ctx.resolve_dispatch_target(receiver_ty, &selector, crate::dispatch::DispatchLookup::Normal);
                if let crate::dispatch::ResolvedDispatchResult::Found(found) = target {
                    Some(found.callable)
                } else {
                    None
                }
            }
            Expr::Binary(binary) => {
                let op_name = match binary.op {
                    BinaryOp::Equal | BinaryOp::Same => "==",
                    BinaryOp::NotEqual => "!=",
                    BinaryOp::LessThan => "<",
                    BinaryOp::LessThanOrEqual => "<=",
                    BinaryOp::GreaterThan => ">",
                    BinaryOp::GreaterThanOrEqual => ">=",
                    _ => return None,
                };
                let name = if let Expr::Var { value: name, .. } = &binary.left {
                    name
                } else if let Expr::Var { value: name, .. } = &binary.right {
                    name
                } else {
                    return None;
                };
                let binding = ctx.lookup_binding_info(name)?;
                let receiver_ty = ctx
                    .flow
                    .get_current_type(binding.id)
                    .and_then(|k| k.ty())
                    .or_else(|| ctx.core_type(&ctx.core_ids.object.clone()))?;
                let selector = phalcom_common::selector::Selector::method(op_name, vec![phalcom_common::selector::SelectorSlot::Positional]).ok()?;
                let target = ctx.resolve_dispatch_target(receiver_ty, &selector, crate::dispatch::DispatchLookup::Normal);
                if let crate::dispatch::ResolvedDispatchResult::Found(found) = target {
                    Some(found.callable)
                } else {
                    None
                }
            }
            _ => None,
        },
        _ => condition_typed.callable.clone(),
    }
}

fn is_canonical_type_test(ctx: &CheckingContext<'_>, callable: Option<&CallableId>) -> bool {
    let Some(callable) = callable else { return false };
    let phalcom_common::selector::SelectorBase::Named(ref name) = callable.selector.base else {
        return false;
    };
    callable.owner == ctx.core_ids.object && matches!(name.as_str(), "is" | "is!")
}

fn is_canonical_equality(ctx: &CheckingContext<'_>, callable: Option<&CallableId>) -> bool {
    let Some(callable) = callable else { return false };
    let phalcom_common::selector::SelectorBase::Named(ref name) = callable.selector.base else {
        return false;
    };
    let owner = callable.declaration_owner();
    (owner == &ctx.core_ids.object
        || owner == &ctx.core_ids.bool_
        || owner == &ctx.core_ids.int
        || owner == &ctx.core_ids.float
        || owner == &ctx.core_ids.string
        || owner == &ctx.core_ids.symbol
        || owner == &ctx.core_ids.number)
        && matches!(name.as_str(), "==" | "!=" | "equals" | "same")
}

fn is_canonical_ordered_comparison(ctx: &CheckingContext<'_>, callable: Option<&CallableId>) -> bool {
    let Some(callable) = callable else { return false };
    let phalcom_common::selector::SelectorBase::Named(ref name) = callable.selector.base else {
        return false;
    };
    let owner = callable.declaration_owner();
    (owner == &ctx.core_ids.int || owner == &ctx.core_ids.float || owner == &ctx.core_ids.number) && matches!(name.as_str(), "<" | "<=" | ">" | ">=" | "<=>")
}
