//! Conservative static boundedness facts for collection exhaustion.
//!
//! This is compiler metadata only.  `Unbounded` is deliberately emitted only
//! for expression shapes whose non-exhaustion is established without running
//! user code; every other shape falls back to `Unknown`.

use std::collections::HashMap;

use phalcom_ast::ast::{Expr, MethodCallExpr, PackItem, PackLabel};
use phalcom_common::range::SourceRange;

use crate::method::{SignatureKind, encode_selector};

use super::lib::CompilerError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Boundedness {
    Bounded,
    Unbounded,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IterationMode {
    Concrete,
    LazyPipeline,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SourceFacts {
    pub(crate) mode: IterationMode,
    pub(crate) boundedness: Boundedness,
}

const UNKNOWN: SourceFacts = SourceFacts {
    mode: IterationMode::Unknown,
    boundedness: Boundedness::Unknown,
};

const fn concrete(boundedness: Boundedness) -> SourceFacts {
    SourceFacts {
        mode: IterationMode::Concrete,
        boundedness,
    }
}

const fn lazy(boundedness: Boundedness) -> SourceFacts {
    SourceFacts {
        mode: IterationMode::LazyPipeline,
        boundedness,
    }
}

/// Infers facts from only syntax and immutable binding facts.  The binding map
/// uses source names so this module stays independent from compiler/VM state.
pub(crate) fn infer_source_facts(expr: &Expr, const_env: &HashMap<String, SourceFacts>) -> SourceFacts {
    match expr {
        Expr::Range(range) => match (range.lower.is_some(), range.upper.is_some()) {
            (true, true) => concrete(Boundedness::Bounded),
            (true, false) => concrete(Boundedness::Unbounded),
            (false, _) => concrete(Boundedness::Unknown),
        },
        Expr::ListLiteral(_) | Expr::TupleLiteral(_) | Expr::MapLiteral(_) | Expr::SetLiteral(_) => concrete(Boundedness::Bounded),
        Expr::Var { value, .. } => const_env.get(value).copied().unwrap_or(UNKNOWN),
        Expr::GetProperty(property) if property.property == "iter" => {
            let source = infer_source_facts(&property.object, const_env);
            match source.mode {
                IterationMode::Concrete | IterationMode::LazyPipeline => lazy(source.boundedness),
                IterationMode::Unknown => UNKNOWN,
            }
        }
        Expr::MethodCall(call) => infer_method_call_facts(call, const_env),
        _ => UNKNOWN,
    }
}

/// Rejects a full-source operation only when its receiver is provably
/// unbounded.  Unknown is always legal.
pub(crate) fn check_exhaustor(
    operation: &str,
    receiver_expr: &Expr,
    call_range: SourceRange,
    const_env: &HashMap<String, SourceFacts>,
) -> Result<(), CompilerError> {
    if infer_source_facts(receiver_expr, const_env).boundedness == Boundedness::Unbounded {
        return Err(CompilerError::ProvablyUnboundedExhaustion {
            operation: operation.to_owned(),
            span: call_range,
        });
    }
    Ok(())
}

/// Spec F entry point for positional expansion.
#[expect(dead_code, reason = "Spec F consumes this compiler seam")]
pub(crate) fn require_exhaustible(source_expr: &Expr, expansion_range: SourceRange, const_env: &HashMap<String, SourceFacts>) -> Result<(), CompilerError> {
    check_exhaustor("expansion", source_expr, expansion_range, const_env)
}

/// Applies the currently implemented collection catalog to a normal send.
pub(crate) fn check_method_call(call: &MethodCallExpr, const_env: &HashMap<String, SourceFacts>) -> Result<(), CompilerError> {
    let selector = canonical_method_selector(call);
    let receiver = infer_source_facts(&call.object, const_env);
    if is_eager_method_selector(&selector, receiver.mode) {
        check_exhaustor(&call.method, &call.object, call.range, const_env)?;
    }
    Ok(())
}

/// Applies the getter catalog (`toList`, `toSet`, `toMap`) to a property send.
pub(crate) fn check_property(property: &str, receiver: &Expr, range: SourceRange, const_env: &HashMap<String, SourceFacts>) -> Result<(), CompilerError> {
    let facts = infer_source_facts(receiver, const_env);
    if is_eager_getter_selector(property, facts.mode) {
        check_exhaustor(property, receiver, range, const_env)?;
    }
    Ok(())
}

fn infer_method_call_facts(call: &MethodCallExpr, const_env: &HashMap<String, SourceFacts>) -> SourceFacts {
    let receiver = infer_source_facts(&call.object, const_env);
    let selector = canonical_method_selector(call);

    if receiver.mode == IterationMode::LazyPipeline {
        return match selector.as_str() {
            "map(_)" | "filter(_)" | "skip(_)" => lazy(receiver.boundedness),
            "take(_)" => lazy(Boundedness::Bounded),
            "takeWhile(_)" => lazy(match receiver.boundedness {
                Boundedness::Bounded => Boundedness::Bounded,
                Boundedness::Unbounded | Boundedness::Unknown => Boundedness::Unknown,
            }),
            "flatMap(_)" => lazy(match receiver.boundedness {
                Boundedness::Unbounded => Boundedness::Unbounded,
                Boundedness::Bounded | Boundedness::Unknown => Boundedness::Unknown,
            }),
            _ if is_eager_method_selector(&selector, receiver.mode) => concrete(Boundedness::Bounded),
            _ => UNKNOWN,
        };
    }

    if receiver.mode == IterationMode::Concrete && is_eager_method_selector(&selector, receiver.mode) {
        return concrete(Boundedness::Bounded);
    }
    UNKNOWN
}

fn canonical_method_selector(call: &MethodCallExpr) -> String {
    // Dynamic pack forms are rejected by lowering; keep them distinct here so
    // boundedness never treats them as a known ordinary selector.
    let labels = call
        .args
        .iter()
        .map(|arg| match arg {
            PackItem::Positional { .. } => None,
            PackItem::Labeled {
                label: PackLabel::Static { text, .. },
                ..
            } => Some(text.clone()),
            PackItem::Labeled {
                label: PackLabel::Computed { .. },
                ..
            }
            | PackItem::Expand { .. } => Some("~dynamic-pack".into()),
        })
        .collect::<Vec<_>>();
    encode_selector(&call.method, &labels, SignatureKind::Method(call.args.len() as u8))
}

fn is_eager_getter_selector(selector: &str, mode: IterationMode) -> bool {
    matches!(mode, IterationMode::Concrete | IterationMode::LazyPipeline) && matches!(selector, "toList" | "toSet" | "toMap")
}

fn is_eager_method_selector(selector: &str, mode: IterationMode) -> bool {
    let common = matches!(
        selector,
        "fold(_,_)" | "fold(_,using)" | "reduce(_,_)" | "count(_)" | "each(_)" | "group(by)" | "partition(where)" | "toMap(merging)"
    );
    let concrete_only = matches!(selector, "map(_)" | "filter(_)" | "flatMap(_)");
    common || (mode == IterationMode::Concrete && concrete_only)
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_ast::ast::{GetPropertyExpr, RangeExpr};
    use phalcom_common::range::EmptySourceRange;

    fn range(lower: bool, upper: bool) -> Expr {
        Expr::Range(Box::new(RangeExpr {
            lower: lower.then(|| Expr::Int {
                digits: "0".into(),
                radix: 10,
                range: EmptySourceRange,
            }),
            upper: upper.then(|| Expr::Int {
                digits: "10".into(),
                radix: 10,
                range: EmptySourceRange,
            }),
            upper_inclusive: false,
            range: EmptySourceRange,
        }))
    }

    fn iter(expr: Expr) -> Expr {
        Expr::GetProperty(Box::new(GetPropertyExpr {
            object: expr,
            property: "iter".into(),
            range: EmptySourceRange,
        }))
    }

    fn call(object: Expr, method: &str) -> Expr {
        Expr::MethodCall(Box::new(MethodCallExpr {
            object,
            method: method.into(),
            args: vec![PackItem::Positional {
                expr: Expr::Block(Box::new(phalcom_ast::ast::BlockExpr {
                    params: vec![],
                    body: vec![],
                    expr_body: false,
                    range: EmptySourceRange,
                })),
                range: EmptySourceRange,
            }],
            range: EmptySourceRange,
        }))
    }

    #[test]
    fn infers_ranges_and_lazy_stages_conservatively() {
        let env = HashMap::new();
        assert_eq!(infer_source_facts(&range(true, true), &env), concrete(Boundedness::Bounded));
        assert_eq!(infer_source_facts(&range(true, false), &env), concrete(Boundedness::Unbounded));
        assert_eq!(infer_source_facts(&range(false, true), &env), concrete(Boundedness::Unknown));
        assert_eq!(infer_source_facts(&iter(range(true, false)), &env), lazy(Boundedness::Unbounded));
        assert_eq!(infer_source_facts(&call(iter(range(true, false)), "map"), &env), lazy(Boundedness::Unbounded));
        assert_eq!(
            infer_source_facts(&call(iter(range(true, false)), "filter"), &env),
            lazy(Boundedness::Unbounded)
        );
        assert_eq!(infer_source_facts(&call(iter(range(true, false)), "skip"), &env), lazy(Boundedness::Unbounded));
        assert_eq!(infer_source_facts(&call(iter(range(true, false)), "take"), &env), lazy(Boundedness::Bounded));
        assert_eq!(
            infer_source_facts(&call(iter(range(true, false)), "takeWhile"), &env),
            lazy(Boundedness::Unknown)
        );
        assert_eq!(
            infer_source_facts(&call(iter(range(true, false)), "flatMap"), &env),
            lazy(Boundedness::Unbounded)
        );
        assert_eq!(infer_source_facts(&call(iter(range(true, true)), "flatMap"), &env), lazy(Boundedness::Unknown));
        assert_eq!(
            infer_source_facts(
                &Expr::Var {
                    value: "unknown".into(),
                    range: EmptySourceRange,
                },
                &env,
            ),
            UNKNOWN
        );
    }
}
