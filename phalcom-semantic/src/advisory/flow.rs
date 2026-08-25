//! One-pass advisory statement flow over compiler-owned source identities.

use std::cell::RefCell;
use std::collections::BTreeMap;

use phalcom_ast::ast::{Pattern, Statement};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::SelectorSlot;

use crate::identity::{CallableId, DeclarationId, DispatchSide, FieldId, SourceSiteId};
use crate::source_index::SourceScopeIndex;

use super::analyzer::{AdvisoryBuiltins, AdvisoryCallObservation, AdvisoryExpressionContext, analyze_expr};
use super::{AdvisoryConfidence, AdvisoryFact, AdvisoryOrigin, AdvisoryParameterSlot, CapturedMethodFamilyShape, ValueShape};
use phalcom_ast::ast::NormalizedSelectorSpec;

/// Static inputs shared by one advisory callable traversal.
pub struct AdvisoryFlowContext<'a> {
    pub scope_index: &'a SourceScopeIndex,
    pub fields: &'a BTreeMap<FieldId, AdvisoryFact>,
    pub callable_returns: &'a BTreeMap<CallableId, AdvisoryFact>,
    pub builtins: &'a AdvisoryBuiltins,
    pub current_owner: Option<&'a DeclarationId>,
    pub dispatch_side: DispatchSide,
    pub source_site_for_range: &'a dyn Fn(SourceRange) -> Option<SourceSiteId>,
    pub resolved_callable_for_range: &'a dyn Fn(SourceRange) -> Option<CallableId>,
    pub resolve_callable_for_shape: Option<&'a dyn Fn(&ValueShape, &str, &[phalcom_ast::ast::PackItem]) -> Option<CallableId>>,
    pub resolve_method_family: Option<&'a dyn Fn(&ValueShape, &NormalizedSelectorSpec) -> Option<CapturedMethodFamilyShape>>,
}

/// Published advisory facts collected by one callable traversal.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AdvisoryFlowProduct {
    /// Current advisory value for each canonical source binding declaration.
    pub bindings: BTreeMap<SourceSiteId, AdvisoryFact>,
    /// Expression facts keyed by canonical source occurrence site.
    pub expressions: BTreeMap<SourceSiteId, AdvisoryFact>,
    /// Advisory facts observed at normal return statements.
    pub returns: Vec<AdvisoryFact>,
    /// Parameter facts contributed by resolved call sites in this traversal.
    pub parameter_contributions: BTreeMap<AdvisoryParameterSlot, AdvisoryFact>,
    /// Tail expression fact for implicit callable return semantics.
    pub tail: Option<AdvisoryFact>,
}

impl AdvisoryFlowProduct {
    /// Joins all observed normal return facts.
    pub fn normal_return(&self) -> AdvisoryFact {
        self.returns
            .iter()
            .cloned()
            .chain(self.tail.iter().cloned())
            .reduce(|left, right| left.join(&right))
            .unwrap_or_else(AdvisoryFact::unknown)
    }
}

/// Analyzes statements once, sharing one mutable binding environment across
/// expression, declaration, and return collection.
pub fn analyze_statements(
    statements: &[Statement],
    context: &AdvisoryFlowContext<'_>,
    mut seed_bindings: BTreeMap<SourceSiteId, AdvisoryFact>,
) -> AdvisoryFlowProduct {
    let mut product = AdvisoryFlowProduct {
        bindings: std::mem::take(&mut seed_bindings),
        ..AdvisoryFlowProduct::default()
    };
    for (index, statement) in statements.iter().enumerate() {
        let value = analyze_statement(statement, context, &mut product);
        if index + 1 == statements.len() {
            product.tail = value;
        }
    }
    product
}

fn analyze_statement(statement: &Statement, context: &AdvisoryFlowContext<'_>, product: &mut AdvisoryFlowProduct) -> Option<AdvisoryFact> {
    match statement {
        Statement::Let(binding) => {
            let value = binding
                .value
                .as_ref()
                .map(|expr| analyze_expression(expr, context, product))
                .unwrap_or_else(AdvisoryFact::unknown);
            bind_pattern(&binding.pattern, &value, context, product);
            None
        }
        Statement::Return(return_statement) => {
            if let Some(expr) = &return_statement.value {
                let fact = analyze_expression(expr, context, product);
                product.returns.push(fact);
            }
            None
        }
        Statement::Expr { expr, .. } => Some(analyze_expression(expr, context, product)),
        Statement::Throw { expr, .. } => {
            let _ = analyze_expression(expr, context, product);
            None
        }
        Statement::For(for_statement) => {
            for lane in &for_statement.lanes {
                let iterable = analyze_expression(&lane.iter, context, product);
                bind_pattern(&lane.pattern, &iterable.element_fact(), context, product);
                if let Some(index) = &lane.index {
                    if let Some(binding) = context.scope_index.binding_for_declaration(index.range) {
                        let fact = AdvisoryFact::new(ValueShape::Unknown, AdvisoryConfidence::Flow)
                            .derive(AdvisoryConfidence::Flow, AdvisoryOrigin::Binding(binding.declaration_site.clone()));
                        product.bindings.insert(binding.declaration_site.clone(), fact);
                    }
                }
                for body_statement in &for_statement.body {
                    let _ = analyze_statement(body_statement, context, product);
                }
            }
            None
        }
        _ => None,
    }
}

fn analyze_expression(expr: &phalcom_ast::ast::Expr, context: &AdvisoryFlowContext<'_>, product: &mut AdvisoryFlowProduct) -> AdvisoryFact {
    let scope = context.scope_index.scope_at(expr.range().start);
    let calls = RefCell::new(Vec::new());
    let observe_call = |call: AdvisoryCallObservation| calls.borrow_mut().push(call);
    let expression_context = AdvisoryExpressionContext {
        scope_index: context.scope_index,
        scope,
        bindings: &product.bindings,
        fields: context.fields,
        callable_returns: context.callable_returns,
        builtins: context.builtins,
        current_owner: context.current_owner,
        dispatch_side: context.dispatch_side,
        source_site_for_range: context.source_site_for_range,
        resolved_callable_for_range: context.resolved_callable_for_range,
        resolve_callable_for_shape: context.resolve_callable_for_shape,
        resolve_method_family: context.resolve_method_family,
        call_observer: Some(&observe_call),
    };
    let fact = analyze_expr(expr, &expression_context);
    for call in calls.into_inner() {
        record_call_contributions(product, context, call);
    }
    if let Some(site) = (context.source_site_for_range)(expr.range()) {
        product.expressions.insert(site, fact.clone());
    }
    fact
}

fn record_call_contributions(product: &mut AdvisoryFlowProduct, context: &AdvisoryFlowContext<'_>, call: AdvisoryCallObservation) {
    let mut positional = 0;
    for argument in call.arguments {
        let index = if let Some(label) = argument.label.as_deref() {
            call.target
                .selector
                .slots
                .iter()
                .position(|slot| matches!(slot, SelectorSlot::Label(candidate) if candidate == label))
        } else {
            let index = call
                .target
                .selector
                .slots
                .iter()
                .enumerate()
                .filter(|(_, slot)| matches!(slot, SelectorSlot::Positional))
                .nth(positional)
                .map(|(index, _)| index);
            positional += 1;
            index
        };
        let Some(index) = index else { continue };
        if matches!(argument.fact.shape, ValueShape::Unknown) {
            continue;
        }
        let fact = if let Some(site) = (context.source_site_for_range)(call.range) {
            argument.fact.derive(AdvisoryConfidence::Interprocedural, AdvisoryOrigin::CallSite(site))
        } else {
            argument
                .fact
                .derive(AdvisoryConfidence::Interprocedural, AdvisoryOrigin::Callable(call.target.clone()))
        };
        let slot = AdvisoryParameterSlot::new(call.target.clone(), index as u32);
        product
            .parameter_contributions
            .entry(slot)
            .and_modify(|old| *old = old.join(&fact))
            .or_insert(fact);
    }
}

fn bind_pattern(pattern: &Pattern, fact: &AdvisoryFact, context: &AdvisoryFlowContext<'_>, product: &mut AdvisoryFlowProduct) {
    match pattern {
        Pattern::Name { range, .. } => {
            if let Some(binding) = context.scope_index.binding_for_declaration(*range) {
                let value = fact
                    .clone()
                    .derive(AdvisoryConfidence::Flow, AdvisoryOrigin::Binding(binding.declaration_site.clone()));
                product.bindings.insert(binding.declaration_site.clone(), value);
            }
        }
        Pattern::Tuple { elements, .. } => {
            for (index, element) in elements.iter().enumerate() {
                bind_pattern(element, &fact.at_index(index), context, product);
            }
        }
        Pattern::List { elements, rest, .. } => {
            for (index, element) in elements.iter().enumerate() {
                bind_pattern(element, &fact.at_index(index), context, product);
            }
            if let Some(rest) = rest {
                bind_pattern(rest, &fact.rest_shape(), context, product);
            }
        }
        Pattern::Variant { arguments, .. } => {
            for argument in arguments {
                bind_pattern(argument, &AdvisoryFact::unknown(), context, product);
            }
        }
        Pattern::Record { entries, .. } => {
            for entry in entries {
                bind_pattern(&entry.pattern, &AdvisoryFact::unknown(), context, product);
            }
        }
        Pattern::Map { entries, .. } => {
            for entry in entries {
                bind_pattern(&entry.pattern, &AdvisoryFact::unknown(), context, product);
            }
        }
    }
}

trait AdvisoryFactShapeExt {
    fn at_index(&self, index: usize) -> AdvisoryFact;
    fn rest_shape(&self) -> AdvisoryFact;
    fn element_fact(&self) -> AdvisoryFact;
}

impl AdvisoryFactShapeExt for AdvisoryFact {
    fn at_index(&self, index: usize) -> AdvisoryFact {
        let shape = match &self.shape {
            ValueShape::Tuple(elements) | ValueShape::ExactList(elements) => elements.get(index).cloned().unwrap_or(ValueShape::Unknown),
            _ => ValueShape::Unknown,
        };
        let mut fact = self.clone();
        fact.shape = shape;
        fact.confidence = fact.confidence.join(AdvisoryConfidence::Flow);
        fact
    }

    fn rest_shape(&self) -> AdvisoryFact {
        let mut fact = self.clone();
        fact.shape = ValueShape::List(Box::new(self.shape.element_shape()));
        fact.confidence = fact.confidence.join(AdvisoryConfidence::Flow);
        fact
    }

    fn element_fact(&self) -> AdvisoryFact {
        let mut fact = self.clone();
        fact.shape = self.shape.element_shape();
        fact.confidence = fact.confidence.join(AdvisoryConfidence::Flow);
        fact
    }
}
