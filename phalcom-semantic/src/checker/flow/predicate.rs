//! Flow predicates for branch filtering and type refinement (Spec 04.5).

use crate::checker::context::CheckingContext;
use crate::identity::{BindingId, PredicateId};
use crate::types::id::TypeId;
use phalcom_ast::ast::{BinaryOp, Expr, PackItem, UnaryOp};

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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PredicateEntry {
    pub id: PredicateId,
    pub predicate: FlowPredicate,
}

/// Extracts a direct flow predicate from a conditional expression.
pub fn extract_predicate(ctx: &mut CheckingContext<'_>, expr: &Expr, truth: bool) -> Option<FlowPredicate> {
    match expr {
        Expr::Unary(unary) if matches!(unary.op, UnaryOp::Not) => extract_predicate(ctx, &unary.expr, !truth),
        Expr::MethodCall(call) => match call.method.as_str() {
            "is" | "is!" => {
                let Expr::Var { value: name, .. } = &call.object else { return None };
                let binding = ctx.lookup_binding_info(name)?.id;
                let target_ty = match call.args.first()? {
                    PackItem::Positional { expr: target_expr, .. } => match target_expr {
                        Expr::Var { value: type_name, .. } => {
                            let decl = ctx.resolver.resolve_type_name(&ctx.current_module, type_name, &[])?;
                            ctx.nominal_type_of(&decl)
                        }
                        _ => return None,
                    },
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
                                Some(FlowPredicate::NotNil { binding })
                            } else {
                                Some(FlowPredicate::IsNil { binding })
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
