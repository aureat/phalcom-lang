//! Unified recursive expression analysis.

use std::collections::BTreeMap;

use phalcom_ast::ast::{
    BinaryOp, Expr, ListLiteralElement, MapLiteralEntry, MapLiteralKey, MethodRefKind, PackItem, PackLabel, ProductLabel, RecordLiteralEntry, SetLiteralEntry,
    SymbolLiteralKind, TupleLiteralEntry, UnaryOp,
};

use super::NativeReturnShape;
use super::dispatch::{DispatchReceiver, ResolvedDispatch};
use super::facts::{InferredValue, LocalFacts, ValueShape};
use super::ids::{CORE_MODULE_URI, CallableId, ClassId, DispatchSide, ModuleId};
use super::scope::{BindingId, ScopeGraph};
use super::surface::MemberSurface;
use crate::selectors::{binary_selector_name, call_selector, index_selector_from_labels, unary_selector_name};

/// Inputs shared by every recursive expression-analysis arm.
pub(crate) struct AnalysisContext<'ctx> {
    pub current_class: Option<&'ctx ClassId>,
    pub dispatch_side: Option<DispatchSide>,
    pub query_offset: usize,
    pub environment: &'ctx BTreeMap<String, InferredValue>,
    pub local_facts: Option<&'ctx LocalFacts>,
    /// Binding-identity state used by structured flow analysis.
    pub binding_values: Option<&'ctx BTreeMap<BindingId, InferredValue>>,
    /// Lexical graph used to resolve each variable occurrence independently.
    pub scopes: Option<&'ctx ScopeGraph>,
    pub known_class: &'ctx dyn Fn(&str) -> Option<ClassId>,
    pub callable_return: &'ctx dyn Fn(&CallableId) -> Option<InferredValue>,
    pub field_value: &'ctx dyn Fn(&ClassId, &str, DispatchSide) -> Option<InferredValue>,
    pub resolver: &'ctx dyn Fn(&DispatchReceiver, &str) -> Option<ResolvedDispatch>,
    pub member_surface: &'ctx dyn Fn(&CallableId) -> Option<MemberSurface>,
    pub contains_class: &'ctx dyn Fn(&ClassId) -> bool,
    pub is_same_or_subclass: &'ctx dyn Fn(&ClassId, &ClassId) -> bool,
}

/// Recursively analyzes every current AST expression variant.
pub(crate) fn analyze_expr(expr: &Expr, context: &AnalysisContext<'_>) -> InferredValue {
    let range = expr.range();
    match expr {
        Expr::Int { .. } => exact(ValueShape::Instance(core_class("Int")), range),
        Expr::Float { .. } => exact(ValueShape::Instance(core_class("Float")), range),
        Expr::String { .. } => exact(ValueShape::Instance(core_class("String")), range),
        Expr::Boolean { value, .. } => InferredValue::exact_boolean(*value, range),
        Expr::Symbol { .. } => exact(ValueShape::Instance(core_class("Symbol")), range),
        Expr::Var { value, .. } => analyze_var(value, range, context),
        Expr::Field { value, .. } => context
            .current_class
            .and_then(|class| (context.field_value)(class, value, context.dispatch_side.unwrap_or(DispatchSide::Instance)))
            .unwrap_or_else(|| flow(ValueShape::Unknown, range)),
        Expr::SelfVar { .. } => match (context.current_class, context.dispatch_side) {
            (Some(class), Some(DispatchSide::Class)) => exact(ValueShape::ClassObject(class.clone()), range),
            (Some(class), _) => exact(ValueShape::Instance(class.clone()), range),
            _ => flow(ValueShape::Unknown, range),
        },
        Expr::SuperVar { .. } => flow(ValueShape::Unknown, range),
        Expr::Assignment(assignment) => {
            let _ = analyze_expr(&assignment.name, context);
            analyze_expr(&assignment.value, context)
        }
        Expr::Range(range_expr) => {
            let bounds = range_expr
                .lower
                .iter()
                .chain(range_expr.upper.iter())
                .map(|bound| analyze_expr(bound, context).shape);
            exact(ValueShape::Range(Box::new(ValueShape::bounded_union(bounds))), range)
        }
        Expr::Unary(unary) => {
            let operand = analyze_expr(&unary.expr, context);
            if matches!(unary.op, UnaryOp::Not) {
                if let Some(value) = operand.known_boolean {
                    return InferredValue::exact_boolean(!value, range);
                }
            }
            let selector = unary_selector_name(&unary.op).to_string() + "()";
            analyze_send(&unary.expr, &operand, &selector, false, range, context)
        }
        Expr::Binary(binary) => {
            let left = analyze_expr(&binary.left, context);
            let right = analyze_expr(&binary.right, context);
            let selector = match binary.op {
                BinaryOp::And => Some("and(_)".to_string()),
                BinaryOp::Or => Some("or(_)".to_string()),
                _ => binary_selector_name(&binary.op).map(|name| format!("{name}(_)")),
            };
            let Some(selector) = selector else { return flow(ValueShape::Unknown, range) };
            let dynamic = matches!(binary.op, BinaryOp::And | BinaryOp::Or) && matches!(right.shape, ValueShape::Unknown);
            analyze_send(&binary.left, &left, &selector, dynamic, range, context)
        }
        Expr::UnqualifiedCall(call) => analyze_unqualified_call(call.name.as_str(), &call.args, range, context),
        Expr::MethodCall(call) => {
            let receiver = analyze_expr(&call.object, context);
            for argument in &call.args {
                analyze_pack(argument, context);
            }
            if let Some(value) = analyze_trusted_type_test(call, &receiver, range, context) {
                return value;
            }
            let selector = call_selector(&call.method, &call.args);
            analyze_send(&call.object, &receiver, &selector, has_dynamic_pack(&call.args), range, context)
        }
        Expr::ImplementationSelector { .. } => flow(ValueShape::Unknown, range),
        Expr::GetProperty(property) => analyze_get_property(property, range, context),
        Expr::SetProperty(property) => {
            let receiver = analyze_expr(&property.object, context);
            let _ = analyze_expr(&property.value, context);
            let selector = crate::selectors::setter_selector_from_name(&property.property);
            analyze_send(&property.object, &receiver, &selector, false, range, context)
        }
        Expr::Index(index) => {
            let receiver = analyze_expr(&index.object, context);
            for argument in &index.args {
                analyze_pack(argument, context);
            }
            let labels = static_labels(&index.args);
            let selector = index_selector_from_labels(&labels, false);
            analyze_send(&index.object, &receiver, &selector, has_dynamic_pack(&index.args), range, context)
        }
        Expr::SetIndex(index) => {
            let receiver = analyze_expr(&index.object, context);
            for argument in &index.args {
                analyze_pack(argument, context);
            }
            let value = analyze_expr(&index.value, context);
            let labels = static_labels(&index.args);
            let selector = index_selector_from_labels(&labels, true);
            let _ = analyze_send(&index.object, &receiver, &selector, has_dynamic_pack(&index.args), range, context);
            value
        }
        Expr::Block(block) => {
            for statement in &block.body {
                analyze_statement(statement, context);
            }
            flow(ValueShape::Unknown, range)
        }
        Expr::MethodRef(reference) => analyze_method_ref(reference, range, context),
        Expr::TupleLiteral(tuple) => exact(
            ValueShape::Tuple(
                tuple
                    .entries
                    .iter()
                    .map(|entry| match entry {
                        TupleLiteralEntry::Positional { expr, .. }
                        | TupleLiteralEntry::Labeled { value: expr, .. }
                        | TupleLiteralEntry::Expand { expr, .. } => analyze_expr(expr, context).shape,
                    })
                    .collect(),
            ),
            range,
        ),
        Expr::RecordLiteral(record) => exact(
            ValueShape::Record(
                record
                    .entries
                    .iter()
                    .filter_map(|entry| match entry {
                        RecordLiteralEntry::Field(field) => Some((product_label(&field.label), analyze_expr(&field.value, context).shape)),
                        RecordLiteralEntry::Expansion { expr, .. } => {
                            let _ = analyze_expr(expr, context);
                            None
                        }
                    })
                    .collect(),
            ),
            range,
        ),
        Expr::MapLiteral(map) => {
            let mut keys = Vec::new();
            let mut values = Vec::new();
            for entry in &map.entries {
                let MapLiteralEntry::Association { key, value, .. } = entry else { continue };
                keys.push(match key {
                    MapLiteralKey::BareSymbol { .. } => ValueShape::Instance(core_class("Symbol")),
                    MapLiteralKey::Computed { expr, .. } => analyze_expr(expr, context).shape,
                });
                values.push(analyze_expr(value, context).shape);
            }
            exact(
                ValueShape::Map {
                    key: Box::new(ValueShape::bounded_union(keys)),
                    value: Box::new(ValueShape::bounded_union(values)),
                },
                range,
            )
        }
        Expr::SetLiteral(set) => exact(
            ValueShape::Set(Box::new(ValueShape::bounded_union(set.entries.iter().map(|entry| match entry {
                SetLiteralEntry::Element { expr, .. } | SetLiteralEntry::Expansion { expr, .. } => analyze_expr(expr, context).shape,
            })))),
            range,
        ),
        Expr::ListLiteral(list) => exact(
            ValueShape::List(Box::new(ValueShape::bounded_union(list.elements.iter().map(|element| match element {
                ListLiteralElement::Element { expr, .. } | ListLiteralElement::Expansion { expr, .. } => analyze_expr(expr, context).shape,
            })))),
            range,
        ),
    }
}

fn analyze_var(name: &str, range: phalcom_common::range::SourceRange, context: &AnalysisContext<'_>) -> InferredValue {
    if let Some(value) = binding_value(name, range.start, context) {
        return value;
    }
    if let Some(value) = context.environment.get(name) {
        return value.clone();
    }
    if let Some(value) = context.local_facts.and_then(|facts| {
        binding_id(name, range.start, context).and_then(|binding| {
            facts
                .value_before(binding, range.start)
                .or_else(|| facts.value_before(binding, context.query_offset))
        })
    }) {
        return value.clone();
    }
    (context.known_class)(name)
        .map(|class| exact(ValueShape::ClassObject(class), range))
        .unwrap_or_else(|| flow(ValueShape::Unknown, range))
}

fn analyze_unqualified_call(name: &str, args: &[PackItem], range: phalcom_common::range::SourceRange, context: &AnalysisContext<'_>) -> InferredValue {
    for argument in args {
        analyze_pack(argument, context);
    }
    if let Some(binding) = context.environment.get(name).or_else(|| {
        context.local_facts.and_then(|facts| {
            binding_id(name, range.start, context).and_then(|binding| {
                facts
                    .value_before(binding, range.start)
                    .or_else(|| facts.value_before(binding, context.query_offset))
            })
        })
    }) {
        return match &binding.shape {
            ValueShape::Callable(callable) => (context.callable_return)(callable).unwrap_or_else(|| flow(ValueShape::Unknown, range)),
            ValueShape::Family { receiver, base } => {
                let selector = call_selector(base, args);
                let targets = receiver_targets_for_shape(receiver);
                analyze_resolved_targets(&targets, &selector, false, range, context)
            }
            _ => flow(ValueShape::Unknown, range),
        };
    }
    let Some(class) = context.current_class else {
        return flow(ValueShape::Unknown, range);
    };
    let receiver = match context.dispatch_side {
        Some(DispatchSide::Class) => DispatchReceiver::ClassObject(class.clone()),
        _ => DispatchReceiver::Instance(class.clone()),
    };
    let selector = call_selector(name, args);
    analyze_resolved_targets(std::slice::from_ref(&receiver), &selector, false, range, context)
}

fn analyze_get_property(
    property: &phalcom_ast::ast::GetPropertyExpr,
    range: phalcom_common::range::SourceRange,
    context: &AnalysisContext<'_>,
) -> InferredValue {
    if let Expr::Var {
        value: binding,
        range: binding_range,
    } = &property.object
        && !is_bound(context, binding, binding_range.start)
        && let Some(class) = (context.known_class)(&format!("{binding}.{}", property.property))
    {
        return exact(ValueShape::ClassObject(class), range);
    }
    let receiver = analyze_expr(&property.object, context);
    analyze_send(&property.object, &receiver, &property.property, false, range, context)
}

fn analyze_method_ref(reference: &phalcom_ast::ast::MethodRefExpr, range: phalcom_common::range::SourceRange, context: &AnalysisContext<'_>) -> InferredValue {
    let receiver = analyze_expr(&reference.receiver, context);
    match &reference.kind {
        MethodRefKind::Open { name } => exact(
            ValueShape::Family {
                receiver: Box::new(receiver.shape),
                base: name.clone(),
            },
            range,
        ),
        MethodRefKind::Pinned { name, labels } => {
            let selector = crate::selectors::comma_form_from_labels(name, labels);
            let receivers = receiver_targets(&reference.receiver, &receiver.shape, context);
            let callables = receivers
                .into_iter()
                .filter_map(|target| (context.resolver)(&target, &selector).map(|resolved| resolved.callable));
            let shapes = callables.map(ValueShape::Callable).collect::<Vec<_>>();
            exact(ValueShape::bounded_union(shapes), range)
        }
    }
}

fn analyze_send(
    object: &Expr,
    receiver: &InferredValue,
    selector: &str,
    dynamic: bool,
    range: phalcom_common::range::SourceRange,
    context: &AnalysisContext<'_>,
) -> InferredValue {
    if dynamic {
        return flow(ValueShape::Unknown, range);
    }
    let targets = receiver_targets(object, &receiver.shape, context);
    analyze_resolved_targets(&targets, selector, false, range, context)
}

fn analyze_resolved_targets(
    targets: &[DispatchReceiver],
    selector: &str,
    dynamic: bool,
    range: phalcom_common::range::SourceRange,
    context: &AnalysisContext<'_>,
) -> InferredValue {
    if dynamic {
        return flow(ValueShape::Unknown, range);
    }
    let values = targets.iter().map(|target| {
        let resolved = (context.resolver)(target, selector);
        match target {
            DispatchReceiver::ClassObject(class) if selector == "new()" && (context.contains_class)(class) => {
                InferredValue::flow(ValueShape::Instance(class.clone()), range)
            }
            DispatchReceiver::ClassObject(class) => {
                let is_constructor = resolved
                    .as_ref()
                    .and_then(|r| (context.member_surface)(&r.callable))
                    .is_some_and(|m| m.is_constructor);
                if is_constructor {
                    InferredValue::flow(ValueShape::Instance(class.clone()), range)
                } else {
                    resolved
                        .map(|member| resolved_return_value(target, &member, range, context))
                        .unwrap_or_else(|| flow(ValueShape::Unknown, range))
                }
            }
            _ => resolved
                .map(|member| resolved_return_value(target, &member, range, context))
                .unwrap_or_else(|| flow(ValueShape::Unknown, range)),
        }
    });
    super::flow::join_values(values)
}

fn resolved_return_value(
    target: &DispatchReceiver,
    resolved: &ResolvedDispatch,
    range: phalcom_common::range::SourceRange,
    context: &AnalysisContext<'_>,
) -> InferredValue {
    if let Some(value) = (context.callable_return)(&resolved.callable) {
        return value;
    }
    let Some(member) = (context.member_surface)(&resolved.callable) else {
        return flow(ValueShape::Unknown, range);
    };
    let Some(native) = member.native_return else {
        return flow(ValueShape::Unknown, range);
    };
    let shape = match native {
        NativeReturnShape::Unknown | NativeReturnShape::Argument(_) => ValueShape::Unknown,
        NativeReturnShape::Instance(name) => ValueShape::Instance(core_class(name)),
        NativeReturnShape::ClassObject(name) => ValueShape::ClassObject(core_class(name)),
        NativeReturnShape::Receiver => match target {
            DispatchReceiver::Instance(class) => ValueShape::Instance(class.clone()),
            DispatchReceiver::ClassObject(class) => ValueShape::ClassObject(class.clone()),
            DispatchReceiver::Super { side, .. } if *side == DispatchSide::Class => ValueShape::ClassObject(resolved.receiver_class.clone()),
            DispatchReceiver::Super { .. } => ValueShape::Instance(resolved.receiver_class.clone()),
        },
    };
    flow(shape, range)
}

fn analyze_trusted_type_test(
    call: &phalcom_ast::ast::MethodCallExpr,
    receiver: &InferredValue,
    range: phalcom_common::range::SourceRange,
    context: &AnalysisContext<'_>,
) -> Option<InferredValue> {
    // `is` and `isExactly` are sealed core predicates: source syntax may
    // desugar to sends, but user classes cannot replace their semantics.
    if !matches!(call.method.as_str(), "is" | "isExactly") || call.args.len() != 1 {
        return None;
    }
    let PackItem::Positional { expr, .. } = &call.args[0] else { return None };
    let ValueShape::ClassObject(target) = analyze_expr(expr, context).shape else {
        return None;
    };
    let is_exact = call.method == "isExactly";
    let result = match &receiver.shape {
        ValueShape::Instance(actual) => Some(if is_exact {
            actual == &target
        } else {
            (context.is_same_or_subclass)(actual, &target)
        }),
        ValueShape::Union(shapes) => {
            let results = shapes.iter().filter_map(|shape| match shape {
                ValueShape::Instance(actual) => Some(if is_exact {
                    actual == &target
                } else {
                    (context.is_same_or_subclass)(actual, &target)
                }),
                _ => None,
            });
            let results = results.collect::<Vec<_>>();
            (!results.is_empty() && results.iter().all(|result| *result == results[0])).then_some(results[0])
        }
        _ => None,
    }?;
    Some(InferredValue::exact_boolean(result, range))
}

fn receiver_targets(expr: &Expr, shape: &ValueShape, context: &AnalysisContext<'_>) -> Vec<DispatchReceiver> {
    if matches!(expr, Expr::SuperVar { .. }) {
        return context
            .current_class
            .map(|class| {
                vec![DispatchReceiver::Super {
                    lexical_class: class.clone(),
                    side: context.dispatch_side.unwrap_or(DispatchSide::Instance),
                }]
            })
            .unwrap_or_default();
    }
    receiver_targets_for_shape(shape)
}

fn receiver_targets_for_shape(shape: &ValueShape) -> Vec<DispatchReceiver> {
    match shape {
        ValueShape::Instance(class) => vec![DispatchReceiver::Instance(class.clone())],
        ValueShape::ClassObject(class) => vec![DispatchReceiver::ClassObject(class.clone())],
        ValueShape::Union(shapes) => shapes.iter().flat_map(receiver_targets_for_shape).collect(),
        _ => Vec::new(),
    }
}

fn analyze_pack(argument: &PackItem, context: &AnalysisContext<'_>) {
    match argument {
        PackItem::Positional { expr, .. } | PackItem::Expand { expr, .. } | PackItem::Labeled { value: expr, .. } => {
            let _ = analyze_expr(expr, context);
        }
    }
}

fn analyze_statement(statement: &phalcom_ast::ast::Statement, context: &AnalysisContext<'_>) {
    match statement {
        phalcom_ast::ast::Statement::Let(binding) => {
            if let Some(value) = &binding.value {
                let _ = analyze_expr(value, context);
            }
        }
        phalcom_ast::ast::Statement::Return(value) => {
            if let Some(expr) = &value.value {
                let _ = analyze_expr(expr, context);
            }
        }
        phalcom_ast::ast::Statement::Expr { expr, .. } | phalcom_ast::ast::Statement::Throw { expr, .. } => {
            let _ = analyze_expr(expr, context);
        }
        phalcom_ast::ast::Statement::For(for_statement) => {
            let _ = analyze_expr(&for_statement.iter, context);
            for statement in &for_statement.body {
                analyze_statement(statement, context);
            }
        }
        phalcom_ast::ast::Statement::Class(_)
        | phalcom_ast::ast::Statement::Break { .. }
        | phalcom_ast::ast::Statement::Continue { .. }
        | phalcom_ast::ast::Statement::Import(_) => {}
    }
}

fn is_bound(context: &AnalysisContext<'_>, name: &str, offset: usize) -> bool {
    binding_value(name, offset, context).is_some()
        || context.environment.contains_key(name)
        || context.local_facts.is_some_and(|facts| {
            binding_id(name, offset, context)
                .and_then(|binding| {
                    facts
                        .value_before(binding, offset)
                        .or_else(|| facts.value_before(binding, context.query_offset))
                })
                .is_some()
        })
}

fn binding_id(name: &str, offset: usize, context: &AnalysisContext<'_>) -> Option<BindingId> {
    let scopes = context.scopes?;
    let resolve_at = |position| match scopes.resolve(scopes.scope_at(position), name, position) {
        super::scope::NameResolution::Binding(binding) => Some(binding),
        _ => None,
    };
    resolve_at(offset).or_else(|| resolve_at(context.query_offset))
}

fn binding_value(name: &str, offset: usize, context: &AnalysisContext<'_>) -> Option<InferredValue> {
    let binding = binding_id(name, offset, context)?;
    context.binding_values?.get(&binding).cloned()
}

fn has_dynamic_pack(args: &[PackItem]) -> bool {
    args.iter().any(|argument| match argument {
        PackItem::Expand { .. } => true,
        PackItem::Labeled {
            label: PackLabel::Computed { .. },
            ..
        } => true,
        PackItem::Positional { .. }
        | PackItem::Labeled {
            label: PackLabel::Static { .. },
            ..
        } => false,
    })
}

fn static_labels(args: &[PackItem]) -> Vec<Option<String>> {
    args.iter()
        .map(|argument| match argument {
            PackItem::Positional { .. } => None,
            PackItem::Labeled {
                label: PackLabel::Static { text, .. },
                ..
            } => Some(text.clone()),
            PackItem::Expand { .. }
            | PackItem::Labeled {
                label: PackLabel::Computed { .. },
                ..
            } => None,
        })
        .collect()
}

fn exact(shape: ValueShape, range: phalcom_common::range::SourceRange) -> InferredValue {
    InferredValue::exact(shape, range)
}

fn flow(shape: ValueShape, range: phalcom_common::range::SourceRange) -> InferredValue {
    InferredValue::flow(shape, range)
}

fn core_class(name: &str) -> ClassId {
    ClassId::new(ModuleId::new(CORE_MODULE_URI), name)
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
        } => name.clone(),
        ProductLabel::Computed { .. } => "?".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::dispatch::DispatchResolver;
    use crate::semantic::surface::build_module_surface;
    use phalcom_ast::parser::parse;

    #[test]
    fn inherited_dispatch_uses_resolved_declaration_summary() {
        let source = "class Parent { value() { } }\nclass Child is Parent { }\n";
        let module = ModuleId::new("file:///analyzer.ph");
        let parsed = parse(source, 0);
        let classes = build_module_surface(module.clone(), &parsed.program).classes;
        let child = ClassId::new(module.clone(), "Child");
        let target = CallableId {
            owner: ClassId::new(module, "Parent"),
            selector: "value()".to_string(),
            side: DispatchSide::Instance,
        };
        let environment = BTreeMap::from([("child".to_string(), InferredValue::flow(ValueShape::Instance(child), Default::default()))]);
        let known_class = |_: &str| None;
        let returns = |id: &CallableId| (id == &target).then(|| InferredValue::flow(ValueShape::Instance(core_class("String")), Default::default()));
        let fields = |_: &ClassId, _: &str, _: DispatchSide| None;
        let resolver = DispatchResolver::new(&classes);
        let resolve_member = |receiver: &DispatchReceiver, selector: &str| resolver.resolve(receiver, selector);
        let contains_class = |class: &ClassId| resolver.contains_class(class);
        let is_same_or_subclass = |child: &ClassId, ancestor: &ClassId| child == ancestor;
        let expression_parse = parse("child.value()", 0);
        let phalcom_ast::ast::Statement::Expr {
            expr: Expr::MethodCall(call), ..
        } = &expression_parse.program.statements[0]
        else {
            panic!("expected call")
        };
        let context = AnalysisContext {
            current_class: None,
            dispatch_side: None,
            query_offset: 0,
            environment: &environment,
            local_facts: None,
            binding_values: None,
            scopes: None,
            known_class: &known_class,
            callable_return: &returns,
            field_value: &fields,
            resolver: &resolve_member,
            member_surface: &|id: &CallableId| resolver.member(id).cloned(),
            contains_class: &contains_class,
            is_same_or_subclass: &is_same_or_subclass,
        };

        assert_eq!(
            analyze_expr(&Expr::MethodCall(call.clone()), &context).shape,
            ValueShape::Instance(core_class("String"))
        );
    }

    #[test]
    fn binary_add_uses_string_return_contract_when_surface_declares_it() {
        let module = ModuleId::new("file:///analyzer.ph");
        let parsed = parse("class String { +(_ other) { } }\n", 0);
        assert!(parsed.errors.is_empty(), "unexpected source errors: {:?}", parsed.errors);
        let classes = build_module_surface(module.clone(), &parsed.program).classes;
        assert!(
            classes[&ClassId::new(module.clone(), "String")]
                .members_by_side
                .contains_key(&("+(_)".to_string(), DispatchSide::Instance)),
            "members: {:?}",
            classes[&ClassId::new(module.clone(), "String")].members_by_side.keys()
        );
        let target = CallableId {
            owner: ClassId::new(module.clone(), "String"),
            selector: "+(_)".to_string(),
            side: DispatchSide::Instance,
        };
        let environment = BTreeMap::from([(
            "left".to_string(),
            InferredValue::flow(ValueShape::Instance(ClassId::new(module, "String")), Default::default()),
        )]);
        let known_class = |_: &str| None;
        let returns = |id: &CallableId| (id == &target).then(|| InferredValue::flow(ValueShape::Instance(core_class("String")), Default::default()));
        let fields = |_: &ClassId, _: &str, _: DispatchSide| None;
        let resolver = DispatchResolver::new(&classes);
        let resolve_member = |receiver: &DispatchReceiver, selector: &str| resolver.resolve(receiver, selector);
        let contains_class = |class: &ClassId| resolver.contains_class(class);
        let is_same_or_subclass = |child: &ClassId, ancestor: &ClassId| child == ancestor;
        let expression_parse = parse("left + \"!\"", 0);
        let phalcom_ast::ast::Statement::Expr { expr, .. } = &expression_parse.program.statements[0] else {
            panic!("expected expression")
        };
        let context = AnalysisContext {
            current_class: None,
            dispatch_side: None,
            query_offset: 0,
            environment: &environment,
            local_facts: None,
            binding_values: None,
            scopes: None,
            known_class: &known_class,
            callable_return: &returns,
            field_value: &fields,
            resolver: &resolve_member,
            member_surface: &|id: &CallableId| resolver.member(id).cloned(),
            contains_class: &contains_class,
            is_same_or_subclass: &is_same_or_subclass,
        };

        assert_eq!(analyze_expr(expr, &context).shape, ValueShape::Instance(core_class("String")));
    }

    #[test]
    fn binary_add_uses_canonical_native_return_contract() {
        let parsed = super::super::core_source::bundled_parse();
        assert!(parsed.errors.is_empty(), "unexpected core parse errors: {:?}", parsed.errors);
        let surface = super::super::core_source::build_core_surface(&parsed.program);
        let string = core_class("String");
        let environment = BTreeMap::from([("left".to_string(), InferredValue::flow(ValueShape::Instance(string), Default::default()))]);
        let known_class = |_: &str| None;
        let returns = |_: &CallableId| None;
        let fields = |_: &ClassId, _: &str, _: DispatchSide| None;
        let resolver = DispatchResolver::new(&surface.classes);
        let resolve_member = |receiver: &DispatchReceiver, selector: &str| resolver.resolve(receiver, selector);
        let contains_class = |class: &ClassId| resolver.contains_class(class);
        let is_same_or_subclass = |child: &ClassId, ancestor: &ClassId| child == ancestor;
        let expression_parse = parse("left + \"!\"", 0);
        let phalcom_ast::ast::Statement::Expr { expr, .. } = &expression_parse.program.statements[0] else {
            panic!("expected expression")
        };
        let context = AnalysisContext {
            current_class: None,
            dispatch_side: None,
            query_offset: 0,
            environment: &environment,
            local_facts: None,
            binding_values: None,
            scopes: None,
            known_class: &known_class,
            callable_return: &returns,
            field_value: &fields,
            resolver: &resolve_member,
            member_surface: &|id: &CallableId| resolver.member(id).cloned(),
            contains_class: &contains_class,
            is_same_or_subclass: &is_same_or_subclass,
        };

        assert_eq!(analyze_expr(expr, &context).shape, ValueShape::Instance(core_class("String")));
    }

    fn analyze_source_expression(
        source: &str,
        expression: &str,
        current_class: Option<(&str, DispatchSide)>,
        environment: BTreeMap<String, InferredValue>,
        returns: BTreeMap<CallableId, ValueShape>,
    ) -> InferredValue {
        let module = ModuleId::new("file:///analyzer_cases.ph");
        let parsed = parse(source, 0);
        assert!(parsed.errors.is_empty(), "unexpected source errors: {:?}", parsed.errors);
        let classes = build_module_surface(module.clone(), &parsed.program).classes;
        let expression_parse = parse(expression, 0);
        assert!(
            expression_parse.errors.is_empty(),
            "unexpected expression errors: {:?}",
            expression_parse.errors
        );
        let expression = expression_parse
            .program
            .statements
            .into_iter()
            .find_map(|statement| match statement {
                phalcom_ast::ast::Statement::Expr { expr, .. } => Some(expr),
                _ => None,
            })
            .expect("expression statement");
        let current_id = current_class.map(|(name, _)| ClassId::new(module.clone(), name));
        let dispatch_side = current_class.map(|(_, side)| side);
        let known_class = |name: &str| classes.values().find(|class| class.id.name == name).map(|class| class.id.clone());
        let callable_return = |id: &CallableId| returns.get(id).cloned().map(|shape| InferredValue::flow(shape, Default::default()));
        let fields = |_: &ClassId, _: &str, _: DispatchSide| None;
        let resolver = DispatchResolver::new(&classes);
        let resolve_member = |receiver: &DispatchReceiver, selector: &str| resolver.resolve(receiver, selector);
        let contains_class = |class: &ClassId| resolver.contains_class(class);
        let is_same_or_subclass = |child: &ClassId, ancestor: &ClassId| child == ancestor;
        let context = AnalysisContext {
            current_class: current_id.as_ref(),
            dispatch_side,
            query_offset: 0,
            environment: &environment,
            local_facts: None,
            binding_values: None,
            scopes: None,
            known_class: &known_class,
            callable_return: &callable_return,
            field_value: &fields,
            resolver: &resolve_member,
            member_surface: &|id: &CallableId| resolver.member(id).cloned(),
            contains_class: &contains_class,
            is_same_or_subclass: &is_same_or_subclass,
        };
        analyze_expr(&expression, &context)
    }

    #[test]
    fn getter_super_and_class_instance_collision_use_dispatch_side() {
        let module = ModuleId::new("file:///analyzer_cases.ph");
        let parent = ClassId::new(module.clone(), "Parent");
        let child = ClassId::new(module.clone(), "Child");
        let returns = BTreeMap::from([
            (
                CallableId {
                    owner: parent.clone(),
                    selector: "value".to_string(),
                    side: DispatchSide::Instance,
                },
                ValueShape::Instance(core_class("Int")),
            ),
            (
                CallableId {
                    owner: child.clone(),
                    selector: "value".to_string(),
                    side: DispatchSide::Instance,
                },
                ValueShape::Instance(core_class("String")),
            ),
            (
                CallableId {
                    owner: child,
                    selector: "value".to_string(),
                    side: DispatchSide::Class,
                },
                ValueShape::Instance(core_class("Bool")),
            ),
        ]);
        let source = "class Parent { value { 1 } }\nclass Child is Parent { value { \"child\" } @class value { true } }\n";
        assert_eq!(
            analyze_source_expression(source, "super.value", Some(("Child", DispatchSide::Instance)), BTreeMap::new(), returns.clone()).shape,
            ValueShape::Instance(core_class("Int"))
        );
        assert_eq!(
            analyze_source_expression(source, "self.value", Some(("Child", DispatchSide::Instance)), BTreeMap::new(), returns.clone()).shape,
            ValueShape::Instance(core_class("String"))
        );
        assert_eq!(
            analyze_source_expression(source, "self.value", Some(("Child", DispatchSide::Class)), BTreeMap::new(), returns).shape,
            ValueShape::Instance(core_class("Bool"))
        );
    }

    #[test]
    fn constructors_operators_and_interpolation_keep_exact_shapes() {
        let source = "class Child {}\n";
        let child = ClassId::new(ModuleId::new("file:///analyzer_cases.ph"), "Child");
        assert_eq!(
            analyze_source_expression(source, "Child.new()", None, BTreeMap::new(), BTreeMap::new()).shape,
            ValueShape::Instance(child)
        );
        let inherited_source = "class Parent { @constructor new() { } }\nclass Child is Parent { }\n";
        let inherited_child = ClassId::new(ModuleId::new("file:///analyzer_cases.ph"), "Child");
        assert_eq!(
            analyze_source_expression(inherited_source, "Child.new()", None, BTreeMap::new(), BTreeMap::new()).shape,
            ValueShape::Instance(inherited_child)
        );

        let parsed = super::super::core_source::bundled_parse();
        assert!(parsed.errors.is_empty(), "unexpected core parse errors: {:?}", parsed.errors);
        let surface = super::super::core_source::build_core_surface(&parsed.program);
        let analyze_core = |expression: &str| {
            let parsed = parse(expression, 0);
            assert!(parsed.errors.is_empty(), "unexpected expression errors: {:?}", parsed.errors);
            let expression = parsed
                .program
                .statements
                .into_iter()
                .find_map(|statement| match statement {
                    phalcom_ast::ast::Statement::Expr { expr, .. } => Some(expr),
                    _ => None,
                })
                .expect("expression statement");
            let known_class = |name: &str| surface.classes.values().find(|class| class.id.name == name).map(|class| class.id.clone());
            let returns = |_: &CallableId| None;
            let fields = |_: &ClassId, _: &str, _: DispatchSide| None;
            let environment = BTreeMap::new();
            let resolver = DispatchResolver::new(&surface.classes);
            let resolve_member = |receiver: &DispatchReceiver, selector: &str| resolver.resolve(receiver, selector);
            let contains_class = |class: &ClassId| resolver.contains_class(class);
            let is_same_or_subclass = |child: &ClassId, ancestor: &ClassId| child == ancestor;
            let context = AnalysisContext {
                current_class: None,
                dispatch_side: None,
                query_offset: 0,
                environment: &environment,
                local_facts: None,
                binding_values: None,
                scopes: None,
                known_class: &known_class,
                callable_return: &returns,
                field_value: &fields,
                resolver: &resolve_member,
                member_surface: &|id: &CallableId| resolver.member(id).cloned(),
                contains_class: &contains_class,
                is_same_or_subclass: &is_same_or_subclass,
            };
            analyze_expr(&expression, &context).shape
        };
        assert_eq!(analyze_core("1 < 2"), ValueShape::Instance(core_class("Bool")));
        assert_eq!(analyze_core("\"value: \\(1)\""), ValueShape::Instance(core_class("String")));
        assert_eq!(analyze_core("true.toString"), ValueShape::Unknown);
    }

    #[test]
    fn subscripts_lexical_bindings_and_pinned_references_are_resolved() {
        let module = ModuleId::new("file:///analyzer_cases.ph");
        let box_id = ClassId::new(module.clone(), "Box");
        let value_callable = CallableId {
            owner: box_id.clone(),
            selector: "value()".to_string(),
            side: DispatchSide::Instance,
        };
        let returns = BTreeMap::from([
            (
                CallableId {
                    owner: box_id.clone(),
                    selector: "[_]".to_string(),
                    side: DispatchSide::Instance,
                },
                ValueShape::Instance(core_class("Int")),
            ),
            (value_callable.clone(), ValueShape::Instance(core_class("Int"))),
        ]);
        let source = "class Box { [_ index] { 1 } [_ index]=(put value) { value } value() { 1 } }\n";
        assert_eq!(
            analyze_source_expression(source, "Box.new()[0]", None, BTreeMap::new(), returns.clone()).shape,
            ValueShape::Instance(core_class("Int"))
        );
        let environment = BTreeMap::from([(
            "value".to_string(),
            InferredValue::exact(ValueShape::Callable(value_callable.clone()), Default::default()),
        )]);
        assert_eq!(
            analyze_source_expression(source, "value()", Some(("Box", DispatchSide::Instance)), environment, returns.clone()).shape,
            ValueShape::Instance(core_class("Int"))
        );
        let pinned = analyze_source_expression(source, "Box.new()::#value()", None, BTreeMap::new(), returns);
        assert!(matches!(pinned.shape, ValueShape::Callable(callable) if callable == value_callable));
    }

    #[test]
    fn open_references_preserve_family_and_dispatch_at_call_site() {
        let module = ModuleId::new("file:///analyzer_cases.ph");
        let box_id = ClassId::new(module.clone(), "Box");
        let value_callable = CallableId {
            owner: box_id.clone(),
            selector: "value()".to_string(),
            side: DispatchSide::Instance,
        };
        let source = "class Box { value() { 1 } }\n";
        let returns = BTreeMap::from([(value_callable.clone(), ValueShape::Instance(core_class("Int")))]);
        let family = analyze_source_expression(source, "Box.new()::value", None, BTreeMap::new(), returns.clone());
        assert!(matches!(&family.shape, ValueShape::Family { receiver, base } if **receiver == ValueShape::Instance(box_id) && base == "value"));

        let environment = BTreeMap::from([(String::from("family"), family)]);
        let invoked = analyze_source_expression(source, "family()", None, environment, returns);
        assert_eq!(invoked.shape, ValueShape::Instance(core_class("Int")));
    }

    #[test]
    fn dynamic_argument_expansion_keeps_dispatch_conservative() {
        let source = "class Box { [_ index] { 1 } }\n";
        let value = analyze_source_expression(source, "Box.new()[***indices]", None, BTreeMap::new(), BTreeMap::new());
        assert_eq!(value.shape, ValueShape::Unknown);
    }
}
