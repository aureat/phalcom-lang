//! Compiler-owned advisory expression analysis.
//!
//! This module intentionally consumes canonical source scopes, formal call
//! targets, and compiler-owned field/return products. It does not perform
//! formal checking and does not create a second dispatch or identity system.

use std::collections::BTreeMap;
use std::sync::Arc;

use phalcom_ast::ast::{
    AssociatedMemberSyntax, AssociatedNamedMode, Expr, ListLiteralElement, MapLiteralEntry, MapLiteralKey, NormalizedSelectorSpec, PackItem, ProductLabel,
    RecordLiteralEntry, SetLiteralEntry, Statement, SymbolLiteralKind, TupleLiteralEntry,
};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::{SelectorKindPattern, SelectorPattern, SelectorSlot};

use crate::declarations::DeclarationTypeTable;
use crate::identity::{CallableId, DeclarationId, DispatchSide, FieldId, SourceSiteId};
use crate::source_index::{SourceNameResolution, SourceScopeId, SourceScopeIndex};

use super::{AdvisoryConfidence, AdvisoryFact, AdvisoryOrigin, CapturedMethodFamilyShape, ValueShape};

pub(crate) type CallableForShapeResolver<'a> = &'a dyn Fn(&ValueShape, &str, &[PackItem]) -> Option<CallableId>;
pub(crate) type FormalCallResultResolver<'a> = &'a dyn Fn(&CallableId, Option<&ValueShape>) -> Option<AdvisoryFact>;
pub(crate) type ModuleMemberResolver<'a> = &'a dyn Fn(&ValueShape, &str) -> Option<ValueShape>;
pub(crate) type MethodFamilyResolver<'a> = &'a dyn Fn(&ValueShape, &NormalizedSelectorSpec) -> Option<CapturedMethodFamilyShape>;

/// Canonical builtin declarations used for literal facts.
///
/// Missing entries remain unknown. Advisory code never fabricates a class
/// identity from a spelling such as `Int` or `Bool`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AdvisoryBuiltins {
    pub int: Option<DeclarationId>,
    pub float: Option<DeclarationId>,
    pub string: Option<DeclarationId>,
    pub boolean: Option<DeclarationId>,
    pub symbol: Option<DeclarationId>,
    pub ordering: Option<DeclarationId>,
}

impl AdvisoryBuiltins {
    /// Resolves builtin identities only from the supplied canonical
    /// declaration table. Missing declarations remain absent and therefore
    /// produce advisory `Unknown` facts.
    pub fn from_declarations(declarations: &DeclarationTypeTable) -> Self {
        fn lookup(declarations: &DeclarationTypeTable, name: &str) -> Option<DeclarationId> {
            let key = match name {
                "Int" => phalcom_native_meta::UniverseKey::Int,
                "Float" => phalcom_native_meta::UniverseKey::Float,
                "String" => phalcom_native_meta::UniverseKey::String,
                "Bool" => phalcom_native_meta::UniverseKey::Bool,
                "Symbol" => phalcom_native_meta::UniverseKey::Symbol,
                "Ordering" => phalcom_native_meta::UniverseKey::Ordering,
                _ => return None,
            };
            let declaration = crate::core_surface::universe_declaration(key);
            declarations.get(&declaration).map(|_| declaration)
        }

        Self {
            int: lookup(declarations, "Int"),
            float: lookup(declarations, "Float"),
            string: lookup(declarations, "String"),
            boolean: lookup(declarations, "Bool"),
            symbol: lookup(declarations, "Symbol"),
            ordering: lookup(declarations, "Ordering"),
        }
    }
}

/// Inputs for one advisory expression evaluation.
pub struct AdvisoryExpressionContext<'a> {
    pub scope_index: &'a SourceScopeIndex,
    pub scope: SourceScopeId,
    pub bindings: &'a BTreeMap<SourceSiteId, AdvisoryFact>,
    pub fields: &'a BTreeMap<FieldId, AdvisoryFact>,
    pub callable_returns: &'a BTreeMap<CallableId, AdvisoryFact>,
    pub builtins: &'a AdvisoryBuiltins,
    pub current_owner: Option<&'a DeclarationId>,
    pub dispatch_side: DispatchSide,
    /// Maps an AST range to its canonical source-site identity.
    pub source_site_for_range: &'a dyn Fn(SourceRange) -> Option<SourceSiteId>,
    /// Formal expression products provide exact targets when available.
    pub resolved_callable_for_range: &'a dyn Fn(SourceRange) -> Option<CallableId>,
    /// Resolves a method call from an advisory receiver shape through the
    /// compiler dispatch adapter when no formal call attachment is available.
    pub resolve_callable_for_shape: Option<CallableForShapeResolver<'a>>,
    /// Projects a canonical callable's formal result against the concrete receiver.
    pub resolve_formal_call_result: Option<FormalCallResultResolver<'a>>,
    /// Maps a public callable identity to the compiler-owned advisory transfer/summary identity.
    pub advisory_transfer_target: Option<&'a dyn Fn(&CallableId) -> CallableId>,
    /// Resolves a member of a compiler-linked module shape. This is kept
    /// separate from method dispatch because top-level module exports have no
    /// callable formal attachment of their own.
    pub resolve_module_member: Option<ModuleMemberResolver<'a>>,
    /// Canonical dispatch adapter for method-family references.
    pub resolve_method_family: Option<MethodFamilyResolver<'a>>,
    /// Observes resolved call arguments for compiler-owned parameter transfer.
    pub call_observer: Option<&'a dyn Fn(AdvisoryCallObservation)>,
    /// Observes every nested expression fact for source-site publication.
    pub expression_observer: Option<&'a dyn Fn(SourceRange, AdvisoryFact)>,
    /// Observes implicit field writes so the workspace can publish field facts.
    pub field_observer: Option<&'a dyn Fn(FieldId, AdvisoryFact)>,
}

/// One advisory argument observed at a resolved call site.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvisoryCallArgument {
    /// Static external label, when the argument used one.
    pub label: Option<String>,
    /// Advisory fact evaluated for the argument expression.
    pub fact: AdvisoryFact,
}

/// Canonical call-site evidence for advisory parameter propagation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdvisoryCallObservation {
    /// Resolved canonical callable target.
    pub target: CallableId,
    /// Compiler-owned advisory parameter/summary transfer target.
    pub transfer_target: CallableId,
    /// Exact source range of the written selector/name token, when present.
    pub target_range: Option<SourceRange>,
    /// Exact call expression range.
    pub range: SourceRange,
    /// Arguments evaluated in source order.
    pub arguments: Vec<AdvisoryCallArgument>,
}

/// Evaluates one AST expression into an advisory fact.
pub fn analyze_expr(expr: &Expr, context: &AdvisoryExpressionContext<'_>) -> AdvisoryFact {
    let fact = analyze_expr_inner(expr, context);
    if let Some(observer) = context.expression_observer {
        observer(expr.range(), fact.clone());
    }
    fact
}

fn analyze_expr_inner(expr: &Expr, context: &AdvisoryExpressionContext<'_>) -> AdvisoryFact {
    let range = expr.range();
    match expr {
        Expr::Int { .. } => literal(context, context.builtins.int.clone(), range),
        Expr::Float { .. } => literal(context, context.builtins.float.clone(), range),
        Expr::String { .. } => literal(context, context.builtins.string.clone(), range),
        Expr::Boolean { value, .. } => {
            let mut fact = literal(context, context.builtins.boolean.clone(), range);
            fact.literal = Some(super::AdvisoryLiteral::Bool(*value));
            fact
        }
        Expr::Symbol(_) => literal(context, context.builtins.symbol.clone(), range),
        Expr::Var { value, .. } => analyze_var(value, range, context),
        Expr::Field { value, .. } => context
            .current_owner
            .map(|owner| FieldId::new(owner.clone(), value.clone(), context.dispatch_side))
            .and_then(|field| context.fields.get(&field).cloned().map(|fact| (field, fact)))
            .map(|(field, fact)| fact.derive(AdvisoryConfidence::Flow, AdvisoryOrigin::Field(field)))
            .unwrap_or_else(AdvisoryFact::unknown),
        Expr::SelfVar { .. } => context
            .current_owner
            .map(|owner| {
                let shape = match context.dispatch_side {
                    DispatchSide::Instance => ValueShape::Instance(owner.clone()),
                    DispatchSide::Class => ValueShape::ClassObject(owner.clone()),
                };
                syntax_fact(context, shape, range)
            })
            .unwrap_or_else(AdvisoryFact::unknown),
        Expr::Assignment(assignment) => {
            let fact = analyze_expr(&assignment.value, context);
            if let Expr::Field { value, .. } = &*assignment.name
                && let Some(owner) = context.current_owner
                && let Some(observer) = context.field_observer
            {
                observer(FieldId::new(owner.clone(), value.clone(), context.dispatch_side), fact.clone());
            }
            fact
        }
        Expr::Range(range_expr) => {
            let lower = range_expr.lower.as_ref().map(|expr| analyze_expr(expr, context).shape);
            let upper = range_expr.upper.as_ref().map(|expr| analyze_expr(expr, context).shape);
            syntax_fact(
                context,
                ValueShape::Range(Box::new(ValueShape::bounded_union(lower.into_iter().chain(upper)))),
                range,
            )
        }
        Expr::TupleLiteral(tuple) => syntax_fact(
            context,
            ValueShape::Tuple(tuple.entries.iter().map(|entry| tuple_entry_shape(entry, context)).collect::<Vec<_>>().into()),
            range,
        ),
        Expr::ListLiteral(list) => {
            let mut elements = Vec::new();
            let mut expanded = false;
            for entry in &list.elements {
                match entry {
                    ListLiteralElement::Element { expr, .. } => elements.push(analyze_expr(expr, context).shape),
                    ListLiteralElement::Expansion { expr, .. } => {
                        expanded = true;
                        elements.push(analyze_expr(expr, context).shape.element_shape());
                    }
                }
            }
            let shape = if expanded {
                ValueShape::List(Box::new(ValueShape::bounded_union(elements)))
            } else {
                ValueShape::ExactList(elements.into())
            };
            syntax_fact(context, shape, range)
        }
        Expr::RecordLiteral(record) => {
            let mut fields = Vec::new();
            let mut dynamic = false;
            for entry in &record.entries {
                match entry {
                    RecordLiteralEntry::Field(field) => fields.push((product_label(&field.label), analyze_expr(&field.value, context).shape)),
                    RecordLiteralEntry::Expansion { expr, .. } => {
                        if let ValueShape::Record(expanded) = analyze_expr(expr, context).shape {
                            fields.extend(expanded.iter().map(|(label, value)| (label.to_string(), value.clone())));
                        } else {
                            dynamic = true;
                        }
                    }
                }
            }
            if dynamic {
                unknown_at(context, range)
            } else {
                syntax_fact(context, ValueShape::record(fields), range)
            }
        }
        Expr::MapLiteral(map) => {
            let mut keys = Vec::new();
            let mut values = Vec::new();
            for entry in &map.entries {
                let MapLiteralEntry::Association { key, value, .. } = entry else { continue };
                keys.push(match key {
                    MapLiteralKey::BareSymbol { .. } => context.builtins.symbol.clone().map(ValueShape::Instance).unwrap_or(ValueShape::Unknown),
                    MapLiteralKey::Computed { expr, .. } => analyze_expr(expr, context).shape,
                });
                values.push(analyze_expr(value, context).shape);
            }
            syntax_fact(
                context,
                ValueShape::Map {
                    key: Box::new(ValueShape::bounded_union(keys)),
                    value: Box::new(ValueShape::bounded_union(values)),
                },
                range,
            )
        }
        Expr::SetLiteral(set) => syntax_fact(
            context,
            ValueShape::Set(Box::new(ValueShape::bounded_union(set.entries.iter().map(|entry| match entry {
                SetLiteralEntry::Element { expr, .. } | SetLiteralEntry::Expansion { expr, .. } => analyze_expr(expr, context).shape,
            })))),
            range,
        ),
        Expr::AssociatedLookup(lookup) => {
            let receiver = analyze_expr(&lookup.receiver, context);
            if let AssociatedMemberSyntax::Named(named) = &lookup.member {
                let spec = match &named.mode {
                    AssociatedNamedMode::Getter { .. } | AssociatedNamedMode::Family { .. } => SelectorPattern::named(
                        named.base.clone(),
                        SelectorKindPattern::AnyNamed,
                        Vec::<SelectorSlot>::new(),
                        Vec::<SelectorSlot>::new(),
                        true,
                    )
                    .ok()
                    .map(NormalizedSelectorSpec::Pattern),
                    AssociatedNamedMode::Exact { .. } => None,
                };
                if let Some(spec) = spec
                    && let Some(resolve) = context.resolve_method_family
                    && let Some(family) = resolve(&receiver.shape, &spec)
                {
                    return syntax_fact(context, ValueShape::MethodFamily(Arc::new(family)), range);
                }
            }
            unknown_at(context, range)
        }
        Expr::AssociatedInvoke(invoke) => {
            let _ = analyze_expr(&invoke.receiver, context);
            for arg in &invoke.args {
                match arg {
                    phalcom_ast::ast::PackItem::Positional { expr: e, .. }
                    | phalcom_ast::ast::PackItem::Expand { expr: e, .. }
                    | phalcom_ast::ast::PackItem::Labeled { value: e, .. } => {
                        let _ = analyze_expr(e, context);
                    }
                }
            }
            unknown_at(context, range)
        }
        Expr::MethodCall(call) => {
            let receiver = analyze_expr(&call.object, context);
            let arguments = call.args.iter().map(|arg| analyze_pack(arg, context)).collect::<Vec<_>>();
            resolved_call_or_unknown_with_shape(call.range, call.method_range, &receiver.shape, &call.method, &call.args, &arguments, context)
        }
        Expr::UnqualifiedCall(call) => {
            let arguments = call.args.iter().map(|arg| analyze_pack(arg, context)).collect::<Vec<_>>();
            resolved_call_or_unknown_with_arguments_at(call.range, call.name_range, &arguments, context)
        }
        Expr::GetProperty(property) => {
            let object = analyze_expr(&property.object, context);
            if let Some(resolve) = context.resolve_module_member
                && let Some(shape) = resolve(&object.shape, &property.property)
            {
                return compiler_shape_fact(context, shape, property.range);
            }
            resolved_call_or_unknown(property.range, context)
        }
        Expr::SetProperty(property) => {
            let _ = analyze_expr(&property.object, context);
            let _ = analyze_expr(&property.value, context);
            resolved_call_or_unknown(property.range, context)
        }
        Expr::Index(index) => {
            let _ = analyze_expr(&index.object, context);
            let arguments = index.args.iter().map(|arg| analyze_pack(arg, context)).collect::<Vec<_>>();
            resolved_call_or_unknown_with_arguments(index.range, &arguments, context)
        }
        Expr::SetIndex(index) => {
            let _ = analyze_expr(&index.object, context);
            let arguments = index.args.iter().map(|arg| analyze_pack(arg, context)).collect::<Vec<_>>();
            let _ = analyze_expr(&index.value, context);
            resolved_call_or_unknown_with_arguments(index.range, &arguments, context)
        }
        Expr::Unary(unary) => {
            let operand = analyze_expr(&unary.expr, context);
            if matches!(unary.op, phalcom_ast::ast::UnaryOp::Not) {
                if let Some(value) = operand.literal.map(|literal| match literal {
                    super::AdvisoryLiteral::Bool(value) => value,
                }) {
                    let mut fact = literal(context, context.builtins.boolean.clone(), range);
                    fact.literal = Some(super::AdvisoryLiteral::Bool(!value));
                    return fact;
                }
            }
            resolved_call_or_unknown(range, context)
        }
        Expr::Binary(binary) => {
            let _ = analyze_expr(&binary.left, context);
            let _ = analyze_expr(&binary.right, context);
            if matches!(binary.op, phalcom_ast::ast::BinaryOp::Same | phalcom_ast::ast::BinaryOp::Compare) {
                let shape = if matches!(binary.op, phalcom_ast::ast::BinaryOp::Compare) {
                    context.builtins.ordering.clone().map(ValueShape::Instance)
                } else {
                    context.builtins.boolean.clone().map(ValueShape::Instance)
                };
                return literal(
                    context,
                    shape.map(|shape| match shape {
                        ValueShape::Instance(id) => id,
                        _ => unreachable!(),
                    }),
                    range,
                );
            }
            resolved_call_or_unknown(range, context)
        }
        Expr::Membership(membership) => {
            let _ = analyze_expr(&membership.left, context);
            let _ = analyze_expr(&membership.right, context);
            literal(context, context.builtins.boolean.clone(), range)
        }
        Expr::IsMembership(membership) => {
            let _ = analyze_expr(&membership.left, context);
            let _ = analyze_expr(&membership.candidates, context);
            literal(context, context.builtins.boolean.clone(), range)
        }
        Expr::ComparisonChain(chain) => {
            for operand in &chain.operands {
                let _ = analyze_expr(operand, context);
            }
            literal(context, context.builtins.boolean.clone(), range)
        }
        Expr::Block(block) => {
            for statement in &block.body {
                let _ = analyze_statement(statement, context);
            }
            unknown_at(context, range)
        }
        Expr::IfLet(if_let) => {
            let _ = analyze_expr(&if_let.value, context);
            literal(context, context.builtins.boolean.clone(), range)
        }
        Expr::WhileLet(while_let) => {
            let _ = analyze_expr(&while_let.value, context);
            literal(context, context.builtins.boolean.clone(), range)
        }
        Expr::Match(match_expr) => {
            let _ = analyze_expr(&match_expr.value, context);
            for arm in &match_expr.arms {
                let _ = analyze_expr(&arm.branch, context);
            }
            unknown_at(context, range)
        }
        Expr::SuperVar { .. } | Expr::ImplementationSelector { .. } | Expr::Ellipsis { .. } | Expr::TypeForm(_) => unknown_at(context, range),
    }
}

fn analyze_statement(statement: &Statement, context: &AdvisoryExpressionContext<'_>) -> AdvisoryFact {
    match statement {
        Statement::Expr { expr, .. } => analyze_expr(expr, context),
        Statement::Return(return_statement) => return_statement
            .value
            .as_ref()
            .map(|expr| analyze_expr(expr, context))
            .unwrap_or_else(AdvisoryFact::unknown),
        Statement::Throw { expr, .. } => analyze_expr(expr, context),
        _ => AdvisoryFact::unknown(),
    }
}

fn analyze_var(name: &str, range: SourceRange, context: &AdvisoryExpressionContext<'_>) -> AdvisoryFact {
    let resolution = context.scope_index.resolve_name(context.scope, name, range.start);
    match resolution {
        SourceNameResolution::Binding(site) => {
            if let Some(fact) = context.bindings.get(&site) {
                return fact.clone().derive(AdvisoryConfidence::Flow, AdvisoryOrigin::Binding(site));
            }
            if let Some(origin) = context.scope_index.import_origin(&site) {
                return match &origin.remote_target {
                    crate::identity::SemanticTargetId::Declaration(declaration) => syntax_fact(context, ValueShape::ClassObject(declaration.clone()), range),
                    crate::identity::SemanticTargetId::Module(module) => syntax_fact(context, ValueShape::Module(module.clone()), range),
                    _ => unknown_at(context, range),
                };
            }
            match context.scope_index.target_for(&site) {
                Some(crate::identity::SemanticTargetId::Declaration(declaration)) => syntax_fact(context, ValueShape::ClassObject(declaration.clone()), range),
                Some(crate::identity::SemanticTargetId::Module(module)) => syntax_fact(context, ValueShape::Module(module.clone()), range),
                _ => unknown_at(context, range),
            }
        }
        SourceNameResolution::Target(crate::identity::SemanticTargetId::Declaration(declaration)) => {
            syntax_fact(context, ValueShape::ClassObject(declaration), range)
        }
        SourceNameResolution::Target(crate::identity::SemanticTargetId::Module(module)) => syntax_fact(context, ValueShape::Module(module), range),
        SourceNameResolution::Target(_) | SourceNameResolution::ImplicitSelf | SourceNameResolution::Unresolved => unknown_at(context, range),
    }
}

fn analyze_pack(item: &PackItem, context: &AdvisoryExpressionContext<'_>) -> AdvisoryCallArgument {
    match item {
        PackItem::Positional { expr, .. } => AdvisoryCallArgument {
            label: None,
            fact: analyze_expr(expr, context),
        },
        PackItem::Expand { expr, .. } => AdvisoryCallArgument {
            label: None,
            fact: analyze_expr(expr, context),
        },
        PackItem::Labeled { value, label, .. } => {
            let fact = analyze_expr(value, context);
            if let phalcom_ast::ast::PackLabel::Computed { expr, .. } = label {
                let _ = analyze_expr(expr, context);
            }
            AdvisoryCallArgument {
                label: match label {
                    phalcom_ast::ast::PackLabel::Static { text, .. } => Some(text.clone()),
                    phalcom_ast::ast::PackLabel::Computed { .. } => None,
                },
                fact,
            }
        }
    }
}

fn tuple_entry_shape(entry: &TupleLiteralEntry, context: &AdvisoryExpressionContext<'_>) -> ValueShape {
    match entry {
        TupleLiteralEntry::Positional { expr, .. } | TupleLiteralEntry::Labeled { value: expr, .. } | TupleLiteralEntry::Expand { expr, .. } => {
            analyze_expr(expr, context).shape
        }
    }
}

fn resolved_call_or_unknown(range: SourceRange, context: &AdvisoryExpressionContext<'_>) -> AdvisoryFact {
    resolved_call_or_unknown_with_arguments(range, &[], context)
}

fn resolved_call_or_unknown_with_arguments(range: SourceRange, arguments: &[AdvisoryCallArgument], context: &AdvisoryExpressionContext<'_>) -> AdvisoryFact {
    resolved_call_or_unknown_with_arguments_at(range, None, arguments, context)
}

fn resolved_call_or_unknown_with_arguments_at(
    range: SourceRange,
    target_range: Option<SourceRange>,
    arguments: &[AdvisoryCallArgument],
    context: &AdvisoryExpressionContext<'_>,
) -> AdvisoryFact {
    let Some(callable) = (context.resolved_callable_for_range)(range) else {
        return unknown_at(context, range);
    };
    resolved_callable_fact(callable, None, range, target_range, arguments, context)
}

fn resolved_call_or_unknown_with_shape(
    range: SourceRange,
    target_range: Option<SourceRange>,
    receiver: &ValueShape,
    name: &str,
    args: &[PackItem],
    arguments: &[AdvisoryCallArgument],
    context: &AdvisoryExpressionContext<'_>,
) -> AdvisoryFact {
    let callable = if let Some(callable) = (context.resolved_callable_for_range)(range) {
        callable
    } else {
        let Some(resolve) = context.resolve_callable_for_shape else {
            return unknown_at(context, range);
        };
        let Some(callable) = resolve(receiver, name, args) else {
            return unknown_at(context, range);
        };
        callable
    };
    resolved_callable_fact(callable, Some(receiver), range, target_range, arguments, context)
}

fn resolved_callable_fact(
    callable: CallableId,
    receiver: Option<&ValueShape>,
    range: SourceRange,
    target_range: Option<SourceRange>,
    arguments: &[AdvisoryCallArgument],
    context: &AdvisoryExpressionContext<'_>,
) -> AdvisoryFact {
    let transfer_target = context
        .advisory_transfer_target
        .map(|resolve| resolve(&callable))
        .unwrap_or_else(|| callable.clone());
    observe_call(callable.clone(), transfer_target.clone(), range, target_range, arguments, context);

    if let Some(resolve) = context.resolve_formal_call_result
        && let Some(fact) = resolve(&callable, receiver)
    {
        return fact.derive(AdvisoryConfidence::Interprocedural, AdvisoryOrigin::Callable(callable));
    }

    context
        .callable_returns
        .get(&callable)
        .or_else(|| context.callable_returns.get(&transfer_target))
        .cloned()
        .map(|fact| fact.derive(AdvisoryConfidence::Interprocedural, AdvisoryOrigin::Callable(callable)))
        .unwrap_or_else(AdvisoryFact::unknown)
}

fn observe_call(
    target: CallableId,
    transfer_target: CallableId,
    range: SourceRange,
    target_range: Option<SourceRange>,
    arguments: &[AdvisoryCallArgument],
    context: &AdvisoryExpressionContext<'_>,
) {
    if let Some(observer) = context.call_observer {
        observer(AdvisoryCallObservation {
            target,
            transfer_target,
            target_range,
            range,
            arguments: arguments.to_vec(),
        });
    }
}

fn literal(context: &AdvisoryExpressionContext<'_>, declaration: Option<DeclarationId>, range: SourceRange) -> AdvisoryFact {
    let Some(declaration) = declaration else {
        return unknown_at(context, range);
    };
    syntax_fact(context, ValueShape::Instance(declaration), range)
}

fn syntax_fact(context: &AdvisoryExpressionContext<'_>, shape: ValueShape, range: SourceRange) -> AdvisoryFact {
    let fact = AdvisoryFact::new(shape, AdvisoryConfidence::Exact);
    let Some(site) = (context.source_site_for_range)(range) else {
        return fact;
    };
    fact.derive(AdvisoryConfidence::Exact, AdvisoryOrigin::Syntax(site))
}

fn compiler_shape_fact(context: &AdvisoryExpressionContext<'_>, shape: ValueShape, range: SourceRange) -> AdvisoryFact {
    let fact = AdvisoryFact::new(shape, AdvisoryConfidence::Exact);
    let Some(site) = (context.source_site_for_range)(range) else {
        return fact;
    };
    fact.derive(AdvisoryConfidence::Exact, AdvisoryOrigin::Constraint(site))
}

fn unknown_at(context: &AdvisoryExpressionContext<'_>, range: SourceRange) -> AdvisoryFact {
    let fact = AdvisoryFact::unknown();
    let Some(site) = (context.source_site_for_range)(range) else {
        return fact;
    };
    fact.derive(AdvisoryConfidence::Heuristic, AdvisoryOrigin::Syntax(site))
}

fn product_label(label: &ProductLabel) -> String {
    match label {
        ProductLabel::Static {
            symbol: SymbolLiteralKind::Name(name),
            ..
        }
        | ProductLabel::Static {
            symbol: SymbolLiteralKind::Selector { name, .. },
            ..
        }
        | ProductLabel::Static {
            symbol: SymbolLiteralKind::Pattern(phalcom_ast::ast::SelectorPatternSyntax { base: name, .. }),
            ..
        } => name.clone(),
        ProductLabel::Static {
            symbol: SymbolLiteralKind::Subscript { .. },
            ..
        } => "[]".to_string(),
        ProductLabel::Computed { .. } => "?".to_string(),
    }
}
