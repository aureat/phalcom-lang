//! Unified recursive expression analysis.

use std::collections::BTreeMap;
use std::sync::Arc;

use phalcom_ast::ast::{
    BinaryOp, Expr, ListLiteralElement, MapLiteralEntry, MapLiteralKey, MethodRefKind, NormalizedSelectorSpec, PackItem, PackLabel, ProductLabel,
    RecordLiteralEntry, RelationOp, SelectorPatternSyntax, SetLiteralEntry, SymbolLiteralKind, TupleLiteralEntry, UnaryOp,
};
pub use phalcom_common::selector::{Selector, SelectorBase, SelectorKind, SelectorKindPattern, SelectorPattern, SelectorSlot};

use super::NativeReturnShape;
use super::dispatch::{DispatchReceiver, ResolvedDispatch};
use super::facts::{CapturedMethodFamilyShape, InferredValue, LocalFacts, ValueShape};
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
    pub family_resolver: &'ctx dyn Fn(&DispatchReceiver, &SelectorPattern) -> CapturedMethodFamilyShape,
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
        Expr::Symbol(_) => exact(ValueShape::Instance(core_class("Symbol")), range),
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
                if operand.shape == ValueShape::Instance(core_class("Bool")) {
                    return flow(ValueShape::Instance(core_class("Bool")), range);
                }
            }
            let selector = unary_selector_name(&unary.op).to_string();
            analyze_send(&unary.expr, &operand, &selector, false, range, context)
        }
        Expr::Binary(binary) => {
            let left = analyze_expr(&binary.left, context);
            let right = analyze_expr(&binary.right, context);
            if matches!(binary.op, BinaryOp::ShiftRight) {
                if let ValueShape::ClassObject(class_id) = &left.shape {
                    match &right.shape {
                        ValueShape::Selector(selector) => {
                            let resolved = (context.resolver)(&DispatchReceiver::Instance(class_id.clone()), &selector.encode())
                                .or_else(|| (context.resolver)(&DispatchReceiver::ClassObject(class_id.clone()), &selector.encode()));
                            if let Some(resolved) = resolved {
                                return exact(ValueShape::Method(resolved.callable), range);
                            } else {
                                return flow(ValueShape::Unknown, range);
                            }
                        }
                        ValueShape::SelectorPattern(pattern) => {
                            let family = (context.family_resolver)(&DispatchReceiver::Instance(class_id.clone()), pattern);
                            return exact(ValueShape::MethodFamily(Arc::new(family)), range);
                        }
                        _ => {
                            if let Expr::Symbol(symbol) = &binary.right {
                                match &symbol.kind {
                                    SymbolLiteralKind::Pattern(syntax) => {
                                        if let Ok(pat) = syntax.normalize() {
                                            let family = (context.family_resolver)(&DispatchReceiver::Instance(class_id.clone()), &pat);
                                            return exact(ValueShape::MethodFamily(Arc::new(family)), range);
                                        }
                                    }
                                    _ => {
                                        if let Some(sym_text) = symbol_from_expr(&binary.right) {
                                            if let Ok(sel) = Selector::try_decode_exact(&sym_text) {
                                                let resolved = (context.resolver)(&DispatchReceiver::Instance(class_id.clone()), &sel.encode())
                                                    .or_else(|| (context.resolver)(&DispatchReceiver::ClassObject(class_id.clone()), &sel.encode()));
                                                if let Some(resolved) = resolved {
                                                    return exact(ValueShape::Method(resolved.callable), range);
                                                } else {
                                                    return flow(ValueShape::Unknown, range);
                                                }
                                            } else if let Ok(pat) = SelectorPattern::try_decode_pattern(&sym_text) {
                                                let family = (context.family_resolver)(&DispatchReceiver::Instance(class_id.clone()), &pat);
                                                return exact(ValueShape::MethodFamily(Arc::new(family)), range);
                                            }
                                        }
                                    }
                                }
                            } else if let Some(sym_text) = symbol_from_expr(&binary.right) {
                                if let Ok(sel) = Selector::try_decode_exact(&sym_text) {
                                    let resolved = (context.resolver)(&DispatchReceiver::Instance(class_id.clone()), &sel.encode())
                                        .or_else(|| (context.resolver)(&DispatchReceiver::ClassObject(class_id.clone()), &sel.encode()));
                                    if let Some(resolved) = resolved {
                                        return exact(ValueShape::Method(resolved.callable), range);
                                    } else {
                                        return flow(ValueShape::Unknown, range);
                                    }
                                } else if let Ok(pat) = SelectorPattern::try_decode_pattern(&sym_text) {
                                    let family = (context.family_resolver)(&DispatchReceiver::Instance(class_id.clone()), &pat);
                                    return exact(ValueShape::MethodFamily(Arc::new(family)), range);
                                }
                            }
                        }
                    }
                    return flow(ValueShape::Unknown, range);
                }
            }
            if matches!(binary.op, BinaryOp::Same) {
                return exact(ValueShape::Instance(core_class("Bool")), range);
            }
            if matches!(binary.op, BinaryOp::Compare) {
                let _ = analyze_send(&binary.left, &left, "compare(_)", false, range, context);
                let _ = analyze_send(&binary.right, &right, "compare(_)", false, range, context);
                return exact(ValueShape::Instance(core_class("Ordering")), range);
            }
            let selector = match binary.op {
                BinaryOp::And => Some("and(_)".to_string()),
                BinaryOp::Or => Some("or(_)".to_string()),
                _ => binary_selector_name(&binary.op).map(|name| format!("{name}(_)")),
            };
            let Some(selector) = selector else { return flow(ValueShape::Unknown, range) };
            let dynamic = matches!(binary.op, BinaryOp::And | BinaryOp::Or) && matches!(right.shape, ValueShape::Unknown);
            let direct = analyze_send(&binary.left, &left, &selector, dynamic, range, context);
            if matches!(
                binary.op,
                BinaryOp::Add
                    | BinaryOp::Subtract
                    | BinaryOp::Multiply
                    | BinaryOp::Divide
                    | BinaryOp::IntegerDivide
                    | BinaryOp::Power
                    | BinaryOp::Modulo
                    | BinaryOp::ShiftLeft
                    | BinaryOp::ShiftRight
                    | BinaryOp::BitAnd
                    | BinaryOp::BitXor
                    | BinaryOp::BitOr
            ) {
                let reflected_selector = selector.strip_suffix("(_)").map_or_else(|| selector.clone(), |name| format!("{name}(from)"));
                let reflected = analyze_send(&binary.right, &right, &reflected_selector, false, range, context);
                let known = [direct, reflected]
                    .into_iter()
                    .filter(|value| !matches!(value.shape, ValueShape::Unknown))
                    .collect::<Vec<_>>();
                return if known.is_empty() {
                    flow(ValueShape::Unknown, range)
                } else {
                    super::flow::join_values(known)
                };
            }
            direct
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
            if let ValueShape::ClassObject(class_id) = &receiver.shape {
                if class_id.name == "Selector" && (call.method == "call" || call.method == "from" || call.method == "new") && call.args.len() == 1 {
                    let arg_expr = match &call.args[0] {
                        PackItem::Positional { expr, .. } => Some(expr),
                        PackItem::Labeled { value, .. } => Some(value),
                        _ => None,
                    };
                    if let Some(expr) = arg_expr {
                        if let Some(sym_text) = symbol_from_expr(expr) {
                            if let Ok(sel) = Selector::try_decode_exact(&sym_text) {
                                return exact(ValueShape::Selector(sel), range);
                            }
                        }
                    }
                }
                if class_id.name == "SelectorPattern" && (call.method == "call" || call.method == "from" || call.method == "new") && call.args.len() == 1 {
                    let arg_expr = match &call.args[0] {
                        PackItem::Positional { expr, .. } => Some(expr),
                        PackItem::Labeled { value, .. } => Some(value),
                        _ => None,
                    };
                    if let Some(expr) = arg_expr {
                        if let Some(sym_text) = symbol_from_expr(expr) {
                            if let Ok(pat) = SelectorPattern::try_decode_pattern(&sym_text) {
                                return exact(ValueShape::SelectorPattern(pat), range);
                            }
                        }
                    }
                }
            }
            // Method & MethodFamily .bind(receiver)
            if call.method == "bind" && call.args.len() == 1 {
                let bound_recv = match &call.args[0] {
                    PackItem::Positional { expr, .. } => analyze_expr(expr, context),
                    PackItem::Labeled { value, .. } => analyze_expr(value, context),
                    _ => flow(ValueShape::Unknown, range),
                };
                if let ValueShape::Method(callable) = &receiver.shape {
                    return exact(
                        ValueShape::BoundMethod {
                            receiver: Box::new(bound_recv.shape),
                            method: callable.clone(),
                        },
                        range,
                    );
                }
                if let ValueShape::MethodFamily(family) = &receiver.shape {
                    return exact(
                        ValueShape::BoundMethodFamily {
                            receiver: Box::new(bound_recv.shape),
                            family: family.clone(),
                        },
                        range,
                    );
                }
            }
            // Family .get() and .set(_)
            if let ValueShape::Family { receiver: target_recv, spec } = &receiver.shape {
                if call.method == "get" && call.args.is_empty() {
                    let getter_sel = match spec {
                        NormalizedSelectorSpec::Exact(sel) if sel.kind == SelectorKind::Getter => Some(sel.encode()),
                        NormalizedSelectorSpec::Pattern(pat)
                            if matches!(pat.kind, SelectorKindPattern::AnyNamed | SelectorKindPattern::Exact(SelectorKind::Getter)) =>
                        {
                            match &pat.base {
                                SelectorBase::Named(name) => Some(name.clone()),
                                _ => None,
                            }
                        }
                        _ => None,
                    };
                    if let Some(sel) = getter_sel {
                        let targets = receiver_targets_for_shape(target_recv);
                        return analyze_resolved_targets(&targets, &sel, false, range, context);
                    }
                }
                if call.method == "set" && call.args.len() == 1 {
                    let setter_sel = match spec {
                        NormalizedSelectorSpec::Exact(sel) if sel.kind == SelectorKind::Setter => Some(sel.encode()),
                        NormalizedSelectorSpec::Pattern(pat)
                            if matches!(pat.kind, SelectorKindPattern::AnyNamed | SelectorKindPattern::Exact(SelectorKind::Setter)) =>
                        {
                            match &pat.base {
                                SelectorBase::Named(name) => Some(format!("{}=(put)", name)),
                                _ => None,
                            }
                        }
                        _ => None,
                    };
                    if let Some(sel) = setter_sel {
                        let targets = receiver_targets_for_shape(target_recv);
                        return analyze_resolved_targets(&targets, &sel, false, range, context);
                    }
                }
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
        Expr::RecordLiteral(record) => {
            let mut fields = Vec::new();
            let mut dynamic = false;
            for entry in &record.entries {
                match entry {
                    RecordLiteralEntry::Field(field) => {
                        insert_record_field(&mut fields, product_label(&field.label), analyze_expr(&field.value, context).shape);
                    }
                    RecordLiteralEntry::Expansion { expr, .. } => {
                        let shape = analyze_expr(expr, context).shape;
                        if let ValueShape::Record(expanded) = shape {
                            for (label, value) in expanded {
                                insert_record_field(&mut fields, label, value);
                            }
                        } else {
                            dynamic = true;
                        }
                    }
                }
            }
            if dynamic {
                exact(ValueShape::Unknown, range)
            } else {
                exact(ValueShape::Record(fields), range)
            }
        }
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
        Expr::ListLiteral(list) => {
            let mut elements = Vec::new();
            let mut has_expansion = false;
            for element in &list.elements {
                match element {
                    ListLiteralElement::Element { expr, .. } => elements.push(analyze_expr(expr, context).shape),
                    ListLiteralElement::Expansion { expr, .. } => {
                        has_expansion = true;
                        elements.push(analyze_expr(expr, context).shape.element_shape());
                    }
                }
            }
            if has_expansion {
                exact(ValueShape::List(Box::new(ValueShape::bounded_union(elements))), range)
            } else {
                exact(ValueShape::ExactList(elements), range)
            }
        }
        Expr::Membership(m) => {
            analyze_expr(&m.left, context);
            analyze_expr(&m.right, context);
            exact(ValueShape::Instance(core_class("Bool")), range)
        }
        Expr::IsMembership(m) => {
            analyze_expr(&m.left, context);
            analyze_expr(&m.candidates, context);
            exact(ValueShape::Instance(core_class("Bool")), range)
        }
        Expr::ComparisonChain(chain) => {
            let operands = chain.operands.iter().map(|operand| analyze_expr(operand, context)).collect::<Vec<_>>();
            for (index, relation) in chain.operators.iter().enumerate() {
                let left = &operands[index];
                let right = &operands[index + 1];
                match relation {
                    RelationOp::Matches => {
                        let _ = analyze_send(&chain.operands[index + 1], right, "matches(_)", false, range, context);
                    }
                    RelationOp::Understands => {
                        let _ = analyze_send(&chain.operands[index], left, "understands(_)", false, range, context);
                    }
                    RelationOp::Binary(_) => {}
                }
            }
            exact(ValueShape::Instance(core_class("Bool")), range)
        }
        Expr::IfLet(if_let) => {
            analyze_expr(&if_let.value, context);
            for statement in &if_let.then_body.body {
                analyze_statement(statement, context);
            }
            if let Some(else_body) = &if_let.else_body {
                for statement in &else_body.body {
                    analyze_statement(statement, context);
                }
            }
            flow(ValueShape::Unknown, range)
        }
        Expr::WhileLet(while_let) => {
            analyze_expr(&while_let.value, context);
            for statement in &while_let.body {
                analyze_statement(statement, context);
            }
            flow(ValueShape::Unknown, range)
        }
        Expr::Ellipsis { .. } => exact(ValueShape::Instance(core_class("Ellipsis")), range),
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

fn symbol_from_expr(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Symbol(s) => match &s.kind {
            SymbolLiteralKind::Name(name) => Some(name.clone()),
            SymbolLiteralKind::Selector { name, labels } => {
                let slots = labels
                    .iter()
                    .map(|l| match l {
                        Some(label) => SelectorSlot::Label(label.clone()),
                        None => SelectorSlot::Positional,
                    })
                    .collect::<Vec<_>>();
                Selector::method(name, slots).ok().map(|s| s.encode())
            }
            SymbolLiteralKind::Subscript { labels, setter } => {
                let slots = labels
                    .iter()
                    .map(|label| match label {
                        Some(label) => SelectorSlot::Label(label.clone()),
                        None => SelectorSlot::Positional,
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                if *setter {
                    Selector::subscript_set(slots).ok().map(|s| s.encode())
                } else {
                    Selector::subscript_get(slots).ok().map(|s| s.encode())
                }
            }
            SymbolLiteralKind::Pattern(syntax) => {
                syntax.normalize().ok().map(|p| p.encode())
            }
        },
        Expr::String { value, .. } => Some(value.clone()),
        _ => None,
    }
}

fn analyze_unqualified_call(name: &str, args: &[PackItem], range: phalcom_common::range::SourceRange, context: &AnalysisContext<'_>) -> InferredValue {
    for argument in args {
        analyze_pack(argument, context);
    }
    if name == "Selector" && args.len() == 1 {
        let arg_expr = match &args[0] {
            PackItem::Positional { expr, .. } => Some(expr),
            PackItem::Labeled { value, .. } => Some(value),
            _ => None,
        };
        if let Some(expr) = arg_expr {
            if let Some(sym_text) = symbol_from_expr(expr) {
                if let Ok(sel) = Selector::try_decode_exact(&sym_text) {
                    return exact(ValueShape::Selector(sel), range);
                }
            }
        }
    }
    if name == "SelectorPattern" && args.len() == 1 {
        let arg_expr = match &args[0] {
            PackItem::Positional { expr, .. } => Some(expr),
            PackItem::Labeled { value, .. } => Some(value),
            _ => None,
        };
        if let Some(expr) = arg_expr {
            if let Some(sym_text) = symbol_from_expr(expr) {
                if let Ok(pat) = SelectorPattern::try_decode_pattern(&sym_text) {
                    return exact(ValueShape::SelectorPattern(pat), range);
                }
            }
        }
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
        return analyze_callable_value_call(&binding.shape, args, range, context);
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

fn analyze_callable_value_call(
    shape: &ValueShape,
    args: &[PackItem],
    range: phalcom_common::range::SourceRange,
    context: &AnalysisContext<'_>,
) -> InferredValue {
    match shape {
        ValueShape::Callable(callable) | ValueShape::Method(callable) => {
            (context.callable_return)(callable).unwrap_or_else(|| flow(ValueShape::Unknown, range))
        }
        ValueShape::BoundMethod { method, .. } => (context.callable_return)(method).unwrap_or_else(|| flow(ValueShape::Unknown, range)),
        ValueShape::Family { receiver, spec } => match spec {
            NormalizedSelectorSpec::Exact(exact_sel) => {
                let call_slots = crate::selectors::static_call_slots(args);
                if call_slots.as_deref() == Some(&exact_sel.slots) {
                    let targets = receiver_targets_for_shape(receiver);
                    analyze_resolved_targets(&targets, &exact_sel.encode(), false, range, context)
                } else {
                    flow(ValueShape::Unknown, range)
                }
            }
            NormalizedSelectorSpec::Pattern(pattern) => {
                let base_name = match &pattern.base {
                    SelectorBase::Named(name) => name.as_str(),
                    SelectorBase::Subscript => "",
                };
                let call_selector_str = call_selector(base_name, args);
                if let Ok(derived_sel) = Selector::try_decode_exact(&call_selector_str) {
                    if pattern.matches(&derived_sel) {
                        let targets = receiver_targets_for_shape(receiver);
                        return analyze_resolved_targets(&targets, &call_selector_str, false, range, context);
                    }
                }
                flow(ValueShape::Unknown, range)
            }
        },
        ValueShape::BoundMethodFamily { family, .. } => {
            let base_name = match &family.pattern.base {
                SelectorBase::Named(name) => name.as_str(),
                SelectorBase::Subscript => "",
            };
            let call_selector_str = call_selector(base_name, args);
            if let Ok(derived_sel) = Selector::try_decode_exact(&call_selector_str) {
                if let Some(target) = family.resolve_call(&derived_sel) {
                    return (context.callable_return)(&target).unwrap_or_else(|| flow(ValueShape::Unknown, range));
                }
            }
            flow(ValueShape::Unknown, range)
        }
        ValueShape::Union(alternatives) => {
            let values = alternatives.iter().map(|alt| analyze_callable_value_call(alt, args, range, context));
            super::flow::join_values(values)
        }
        _ => flow(ValueShape::Unknown, range),
    }
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
    let spec = reference.spec.normalize().unwrap_or_else(|_| match &reference.kind {
        MethodRefKind::Open { name } => NormalizedSelectorSpec::Pattern(SelectorPattern::named_method(name, [], [], true).unwrap()),
        MethodRefKind::Pinned { name, labels } => {
            let slots = labels
                .iter()
                .map(|l| match l {
                    Some(label) => SelectorSlot::Label(label.clone()),
                    None => SelectorSlot::Positional,
                })
                .collect::<Vec<_>>();
            NormalizedSelectorSpec::Exact(Selector::method(name, slots).unwrap_or_else(|_| Selector::decode(name)))
        }
    });
    exact(
        ValueShape::Family {
            receiver: Box::new(receiver.shape),
            spec,
        },
        range,
    )
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
    receiver: &DispatchReceiver,
    resolved: &ResolvedDispatch,
    range: phalcom_common::range::SourceRange,
    context: &AnalysisContext<'_>,
) -> InferredValue {
    let surface = (context.member_surface)(&resolved.callable);
    if let Some(native_return) = surface.as_ref().and_then(|surface| surface.native_return) {
        match native_return {
            NativeReturnShape::Instance(class) => return InferredValue::flow(ValueShape::Instance(core_class(class)), range),
            NativeReturnShape::Receiver => {
                return match receiver {
                    DispatchReceiver::Instance(class) => InferredValue::flow(ValueShape::Instance(class.clone()), range),
                    DispatchReceiver::ClassObject(class) => InferredValue::flow(ValueShape::ClassObject(class.clone()), range),
                    DispatchReceiver::Super { lexical_class, side } => match side {
                        DispatchSide::Instance => InferredValue::flow(ValueShape::Instance(lexical_class.clone()), range),
                        DispatchSide::Class => InferredValue::flow(ValueShape::ClassObject(lexical_class.clone()), range),
                    },
                };
            }
            NativeReturnShape::ClassObject(class) => return InferredValue::flow(ValueShape::ClassObject(core_class(class)), range),
            NativeReturnShape::Unknown | NativeReturnShape::Argument(_) => {}
        }
    }
    (context.callable_return)(&resolved.callable).unwrap_or_else(|| flow(ValueShape::Unknown, range))
}

fn analyze_trusted_type_test(
    call: &phalcom_ast::ast::MethodCallExpr,
    receiver: &InferredValue,
    range: phalcom_common::range::SourceRange,
    context: &AnalysisContext<'_>,
) -> Option<InferredValue> {
    if matches!(call.method.as_str(), "is" | "is!") && call.args.len() == 1 {
        let PackItem::Positional { expr, .. } = &call.args[0] else {
            return Some(flow(ValueShape::Instance(core_class("Bool")), range));
        };
        let target = match analyze_expr(expr, context).shape {
            ValueShape::ClassObject(class) => class,
            _ => return Some(flow(ValueShape::Instance(core_class("Bool")), range)),
        };
        let result = match &receiver.shape {
            ValueShape::Instance(instance) => {
                if call.method == "is!" {
                    instance == &target
                } else {
                    (context.is_same_or_subclass)(instance, &target)
                }
            }
            _ => return Some(flow(ValueShape::Instance(core_class("Bool")), range)),
        };
        return Some(InferredValue::exact_boolean(result, range));
    }
    if !call.method.starts_with("is") || call.method.len() <= 2 || !call.args.is_empty() {
        return None;
    }
    let tested_name = &call.method[2..];
    if tested_name.is_empty() || !tested_name.chars().next()?.is_uppercase() {
        return None;
    }
    let target_class = (context.known_class)(tested_name)?;
    let result = match &receiver.shape {
        ValueShape::Instance(instance_class) => (context.is_same_or_subclass)(instance_class, &target_class),
        _ => return None,
    };
    Some(InferredValue::exact_boolean(result, range))
}

fn insert_record_field(fields: &mut Vec<(String, ValueShape)>, label: String, value: ValueShape) {
    if let Some((_, existing)) = fields.iter_mut().find(|(existing, _)| existing == &label) {
        *existing = value;
    } else {
        fields.push((label, value));
    }
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
            for lane in &for_statement.lanes {
                let _ = analyze_expr(&lane.iter, context);
            }
            for statement in &for_statement.body {
                analyze_statement(statement, context);
            }
        }
        phalcom_ast::ast::Statement::Class(_)
        | phalcom_ast::ast::Statement::Break { .. }
        | phalcom_ast::ast::Statement::Continue { .. }
        | phalcom_ast::ast::Statement::Export(_) => {}
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
        }
        | ProductLabel::Static {
            symbol: SymbolLiteralKind::Pattern(SelectorPatternSyntax { base: name, .. }),
            ..
        } => name.clone(),
        ProductLabel::Static {
            symbol: SymbolLiteralKind::Subscript { .. },
            ..
        } => "[]".to_string(),
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
        let family_resolver = |receiver: &DispatchReceiver, pattern: &SelectorPattern| resolver.capture_method_family(receiver, pattern);
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
            family_resolver: &family_resolver,
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
                .member("+(_)", DispatchSide::Instance)
                .is_some(),
            "members: {:?}",
            classes[&ClassId::new(module.clone(), "String")].members.keys()
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
        let family_resolver = |receiver: &DispatchReceiver, pattern: &SelectorPattern| resolver.capture_method_family(receiver, pattern);
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
            family_resolver: &family_resolver,
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
        let family_resolver = |receiver: &DispatchReceiver, pattern: &SelectorPattern| resolver.capture_method_family(receiver, pattern);
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
            family_resolver: &family_resolver,
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
        let family_resolver = |receiver: &DispatchReceiver, pattern: &SelectorPattern| resolver.capture_method_family(receiver, pattern);
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
            family_resolver: &family_resolver,
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
                ValueShape::Instance(core_class("Float")),
            ),
            (
                CallableId {
                    owner: child,
                    selector: "value".to_string(),
                    side: DispatchSide::Class,
                },
                ValueShape::Instance(core_class("String")),
            ),
        ]);
        let source = "class Parent { value { 1 } }\nclass Child is Parent { value { 1.0 } @class value { \"\" } }\n";
        assert_eq!(
            analyze_source_expression(source, "Child.new().value", None, BTreeMap::new(), returns.clone()).shape,
            ValueShape::Instance(core_class("Float"))
        );
        assert_eq!(
            analyze_source_expression(source, "Child.value", None, BTreeMap::new(), returns.clone()).shape,
            ValueShape::Instance(core_class("String"))
        );
        assert_eq!(
            analyze_source_expression(source, "super.value", Some(("Child", DispatchSide::Instance)), BTreeMap::new(), returns).shape,
            ValueShape::Instance(core_class("Int"))
        );
    }

    #[test]
    fn collection_spreads_and_record_expansions_transfer_shapes() {
        let int = ValueShape::Instance(core_class("Int"));
        let string = ValueShape::Instance(core_class("String"));
        let values = ValueShape::List(Box::new(int.clone()));
        let list = analyze_source_expression(
            "class Holder {}\n",
            "[0, *values, 4]",
            None,
            BTreeMap::from([("values".to_string(), InferredValue::flow(values, Default::default()))]),
            BTreeMap::new(),
        );
        assert_eq!(list.shape, ValueShape::List(Box::new(int.clone())));

        let record = analyze_source_expression(
            "class Holder {}\n",
            "#{**record, selected: false}",
            None,
            BTreeMap::from([(
                "record".to_string(),
                InferredValue::flow(
                    ValueShape::Record(vec![("selected".to_string(), int), ("kept".to_string(), string.clone())]),
                    Default::default(),
                ),
            )]),
            BTreeMap::new(),
        );
        assert_eq!(
            record.shape,
            ValueShape::Record(vec![
                ("selected".to_string(), ValueShape::Instance(core_class("Bool"))),
                ("kept".to_string(), string)
            ])
        );
    }

    #[test]
    fn type_tests_use_argument_class_and_preserve_boolean_shape() {
        let source = "class Number {}\n";
        let answer = InferredValue::flow(
            ValueShape::Instance(ClassId::new(ModuleId::new("file:///analyzer_cases.ph"), "Number")),
            Default::default(),
        );
        let exact = analyze_source_expression(
            source,
            "answer is Number",
            None,
            BTreeMap::from([(String::from("answer"), answer.clone())]),
            BTreeMap::new(),
        );
        assert_eq!(exact.shape, ValueShape::Instance(core_class("Bool")));
        assert_eq!(exact.known_boolean, Some(true));

        for expression in ["answer is! Number", "answer is not Number", "answer is! not Number"] {
            let result = analyze_source_expression(
                source,
                expression,
                None,
                BTreeMap::from([(String::from("answer"), answer.clone())]),
                BTreeMap::new(),
            );
            assert_eq!(result.shape, ValueShape::Instance(core_class("Bool")), "{expression}");
            assert_eq!(result.known_boolean, Some(!expression.contains("not")), "{expression}");
        }

        let unknown = analyze_source_expression(
            source,
            "answer is Number",
            None,
            BTreeMap::from([(String::from("answer"), InferredValue::flow(ValueShape::Unknown, Default::default()))]),
            BTreeMap::new(),
        );
        assert_eq!(unknown.shape, ValueShape::Instance(core_class("Bool")));
        assert_eq!(unknown.known_boolean, None);
    }

    #[test]
    fn core_syntax_inference_covers_literals_and_builtins() {
        let parsed = super::super::core_source::bundled_parse();
        assert!(parsed.errors.is_empty(), "unexpected core parse errors: {:?}", parsed.errors);
        let surface = super::super::core_source::build_core_surface(&parsed.program);
        let analyze_core = |expression: &str| {
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
            let environment = BTreeMap::new();
            let known_class = |name: &str| surface.classes.values().find(|class| class.id.name == name).map(|class| class.id.clone());
            let returns = |_: &CallableId| None;
            let fields = |_: &ClassId, _: &str, _: DispatchSide| None;
            let resolver = DispatchResolver::new(&surface.classes);
            let resolve_member = |receiver: &DispatchReceiver, selector: &str| resolver.resolve(receiver, selector);
            let family_resolver = |receiver: &DispatchReceiver, pattern: &SelectorPattern| resolver.capture_method_family(receiver, pattern);
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
                family_resolver: &family_resolver,
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
        let pinned = analyze_source_expression(source, "Box.new()::#value()", None, BTreeMap::new(), returns.clone());
        assert!(
            matches!(&pinned.shape, ValueShape::Family { receiver, spec: NormalizedSelectorSpec::Exact(sel) } if **receiver == ValueShape::Instance(box_id) && sel.encode() == "value()")
        );
        let env_pinned = BTreeMap::from([(String::from("pinned"), pinned)]);
        let invoked_pinned = analyze_source_expression(source, "pinned()", None, env_pinned, returns);
        assert_eq!(invoked_pinned.shape, ValueShape::Instance(core_class("Int")));
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
        let family = analyze_source_expression(source, "Box.new()::value(...)", None, BTreeMap::new(), returns.clone());
        assert!(
            matches!(&family.shape, ValueShape::Family { receiver, spec: NormalizedSelectorSpec::Pattern(pat) } if **receiver == ValueShape::Instance(box_id) && pat.base == SelectorBase::Named("value".to_string()))
        );

        let environment = BTreeMap::from([(String::from("family"), family)]);
        let invoked = analyze_source_expression(source, "family()", None, environment, returns);
        assert_eq!(invoked.shape, ValueShape::Instance(core_class("Int")));
    }

    #[test]
    fn method_family_capture_binding_and_bound_routing_adversarial() {
        let module = ModuleId::new("file:///analyzer_cases.ph");
        let a_id = ClassId::new(module.clone(), "A");
        let b_id = ClassId::new(module.clone(), "B");
        let a_foo = CallableId {
            owner: a_id.clone(),
            selector: "foo(_)".to_string(),
            side: DispatchSide::Instance,
        };
        let b_foo = CallableId {
            owner: b_id.clone(),
            selector: "foo(_)".to_string(),
            side: DispatchSide::Instance,
        };
        let source = "class A { foo(_ x) { 1 } }\nclass B { foo(_ x) { \"live\" } }\n";
        let returns = BTreeMap::from([
            (a_foo.clone(), ValueShape::Instance(core_class("Int"))),
            (b_foo.clone(), ValueShape::Instance(core_class("String"))),
        ]);

        // 1. C >> #foo(...) captures MethodFamily
        let captured = analyze_source_expression(source, "A >> #foo(...)", None, BTreeMap::new(), returns.clone());
        assert!(matches!(&captured.shape, ValueShape::MethodFamily(family) if family.source_behavior == a_id));

        // 2. captured.bind(B.new()) -> BoundMethodFamily with captured snapshot from A
        let env_captured = BTreeMap::from([(String::from("captured"), captured)]);
        let bound = analyze_source_expression(source, "captured.bind(B.new())", None, env_captured, returns.clone());
        assert!(
            matches!(&bound.shape, ValueShape::BoundMethodFamily { receiver, family } if **receiver == ValueShape::Instance(b_id) && family.source_behavior == a_id)
        );

        // 3. bound(3) -> resolves captured A#foo(_), returns Int (NOT String)
        let env_bound = BTreeMap::from([(String::from("bound"), bound)]);
        let bound_result = analyze_source_expression(source, "bound(3)", None, env_bound, returns.clone());
        assert_eq!(bound_result.shape, ValueShape::Instance(core_class("Int")));

        // 4. Dynamic counterpart: live = B.new()::foo(...)
        let live = analyze_source_expression(source, "B.new()::foo(...)", None, BTreeMap::new(), returns.clone());
        let env_live = BTreeMap::from([(String::from("live"), live)]);
        let live_result = analyze_source_expression(source, "live(3)", None, env_live, returns);
        assert_eq!(live_result.shape, ValueShape::Instance(core_class("String")));
    }

    #[test]
    fn dynamic_argument_expansion_keeps_dispatch_conservative() {
        let source = "class Box { [_ index] { 1 } }\n";
        let value = analyze_source_expression(source, "Box.new()[***indices]", None, BTreeMap::new(), BTreeMap::new());
        assert_eq!(value.shape, ValueShape::Unknown);
    }
}
