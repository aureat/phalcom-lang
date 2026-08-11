//! Deterministic syntax and local-flow inference.

use std::collections::BTreeMap;

use super::facts::{InferredValue, LocalFacts, ValueShape};
use super::ids::{CORE_MODULE_URI, ClassId, ModuleId};
use phalcom_ast::ast::{
    Expr, ListLiteralElement, MapLiteralEntry, MapLiteralKey, PackItem, PackLabel, Pattern, ProductLabel, RecordLiteralEntry, SetLiteralEntry, Statement,
    SymbolLiteralKind, TupleLiteralEntry,
};

/// Collects exact and local-flow facts from a recovered program.
pub fn collect_local_facts(
    program: &phalcom_ast::ast::Program,
    module: &ModuleId,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
) -> LocalFacts {
    let mut facts = LocalFacts::default();
    let mut environment = BTreeMap::new();
    collect_statements(&program.statements, module, None, known_class, is_constructor, &mut environment, &mut facts);
    facts
}

/// Infers one expression using an existing local environment.
pub fn infer_expr(
    expr: &Expr,
    module: &ModuleId,
    current_class: Option<&ClassId>,
    environment: &BTreeMap<String, InferredValue>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
) -> InferredValue {
    let range = expr.range();
    let shape = match expr {
        Expr::Int { .. } => ValueShape::Instance(core_class("Int")),
        Expr::Float { .. } => ValueShape::Instance(core_class("Float")),
        Expr::String { .. } => ValueShape::Instance(core_class("String")),
        Expr::Boolean { .. } => ValueShape::Instance(core_class("Bool")),
        Expr::Symbol { .. } => ValueShape::Instance(core_class("Symbol")),
        Expr::Var { value, .. } => environment
            .get(value)
            .map(|value| value.shape.clone())
            .or_else(|| known_class(value).map(ValueShape::ClassObject))
            .unwrap_or(ValueShape::Unknown),
        Expr::SelfVar { .. } => current_class.map(|class| ValueShape::Instance(class.clone())).unwrap_or(ValueShape::Unknown),
        Expr::SuperVar { .. } => ValueShape::Unknown,
        Expr::TupleLiteral(tuple) => ValueShape::Tuple(
            tuple
                .entries
                .iter()
                .map(|entry| match entry {
                    TupleLiteralEntry::Positional { expr, .. } | TupleLiteralEntry::Labeled { value: expr, .. } | TupleLiteralEntry::Expand { expr, .. } => {
                        infer_expr(expr, module, current_class, environment, known_class, is_constructor).shape
                    }
                })
                .collect(),
        ),
        Expr::RecordLiteral(record) => ValueShape::Record(
            record
                .entries
                .iter()
                .filter_map(|entry| match entry {
                    RecordLiteralEntry::Field(field) => Some((
                        product_label(&field.label),
                        infer_expr(&field.value, module, current_class, environment, known_class, is_constructor).shape,
                    )),
                    RecordLiteralEntry::Expansion { .. } => None,
                })
                .collect(),
        ),
        Expr::ListLiteral(list) => ValueShape::List(Box::new(ValueShape::bounded_union(list.elements.iter().map(|element| match element {
            ListLiteralElement::Element { expr, .. } | ListLiteralElement::Expansion { expr, .. } => {
                infer_expr(expr, module, current_class, environment, known_class, is_constructor).shape
            }
        })))),
        Expr::SetLiteral(set) => ValueShape::Set(Box::new(ValueShape::bounded_union(set.entries.iter().map(|entry| match entry {
            SetLiteralEntry::Element { expr, .. } | SetLiteralEntry::Expansion { expr, .. } => {
                infer_expr(expr, module, current_class, environment, known_class, is_constructor).shape
            }
        })))),
        Expr::MapLiteral(map) => {
            let mut keys = Vec::new();
            let mut values = Vec::new();
            for entry in &map.entries {
                if let MapLiteralEntry::Association { key, value, .. } = entry {
                    keys.push(match key {
                        MapLiteralKey::BareSymbol { .. } => ValueShape::Instance(core_class("Symbol")),
                        MapLiteralKey::Computed { expr, .. } => infer_expr(expr, module, current_class, environment, known_class, is_constructor).shape,
                    });
                    values.push(infer_expr(value, module, current_class, environment, known_class, is_constructor).shape);
                }
            }
            ValueShape::Map {
                key: Box::new(ValueShape::bounded_union(keys)),
                value: Box::new(ValueShape::bounded_union(values)),
            }
        }
        Expr::Range(range) => {
            let bounds = range
                .lower
                .iter()
                .chain(range.upper.iter())
                .map(|bound| infer_expr(bound, module, current_class, environment, known_class, is_constructor).shape);
            ValueShape::Range(Box::new(ValueShape::bounded_union(bounds)))
        }
        Expr::Assignment(assignment) => infer_expr(&assignment.value, module, current_class, environment, known_class, is_constructor).shape,
        Expr::MethodCall(call) => {
            let receiver = infer_expr(&call.object, module, current_class, environment, known_class, is_constructor);
            let selector = call_selector(&call.method, &call.args);
            match receiver.shape {
                ValueShape::ClassObject(class) if is_constructor(&class, &selector) => ValueShape::Instance(class),
                _ => ValueShape::Unknown,
            }
        }
        _ => ValueShape::Unknown,
    };
    InferredValue::exact(shape, range)
}

fn collect_statements(
    statements: &[Statement],
    module: &ModuleId,
    current_class: Option<&ClassId>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    environment: &mut BTreeMap<String, InferredValue>,
    facts: &mut LocalFacts,
) {
    for statement in statements {
        match statement {
            Statement::Let(binding) => {
                let value = binding
                    .value
                    .as_ref()
                    .map(|expr| infer_expr(expr, module, current_class, environment, known_class, is_constructor))
                    .unwrap_or_else(|| InferredValue::flow(ValueShape::Unknown, binding.range));
                bind_pattern(&binding.pattern, &value, module, current_class, known_class, environment, facts);
            }
            Statement::For(for_statement) => {
                let iterable = infer_expr(&for_statement.iter, module, current_class, environment, known_class, is_constructor);
                let value = InferredValue::flow(iterable.shape.element_shape(), for_statement.range);
                facts.record(for_statement.binding.clone(), for_statement.range, value.clone());
                environment.insert(for_statement.binding.clone(), value);
                collect_statements(&for_statement.body, module, current_class, known_class, is_constructor, environment, facts);
            }
            Statement::Expr {
                expr: Expr::Assignment(assignment),
                ..
            } => {
                let value = infer_expr(&assignment.value, module, current_class, environment, known_class, is_constructor);
                if let Expr::Var { value: name, range } = assignment.name.as_ref() {
                    let value = InferredValue::flow(value.shape.clone(), *range);
                    facts.record(name.clone(), *range, value.clone());
                    environment.insert(name.clone(), value);
                }
            }
            Statement::Class(class) => {
                let id = ClassId::new(module.clone(), class.name.clone());
                for member in &class.members {
                    let body = match member {
                        phalcom_ast::ast::ClassMember::Method(item) => &item.body,
                        phalcom_ast::ast::ClassMember::Getter(item) => &item.body,
                        phalcom_ast::ast::ClassMember::Setter(item) => &item.body,
                        phalcom_ast::ast::ClassMember::Index(item) => &item.body,
                        phalcom_ast::ast::ClassMember::Field(_) | phalcom_ast::ast::ClassMember::Variant(_) => continue,
                    };
                    let mut member_environment = BTreeMap::new();
                    collect_statements(body, module, Some(&id), known_class, is_constructor, &mut member_environment, facts);
                }
            }
            _ => {}
        }
    }
}

fn bind_pattern(
    pattern: &Pattern,
    value: &InferredValue,
    module: &ModuleId,
    current_class: Option<&ClassId>,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    environment: &mut BTreeMap<String, InferredValue>,
    facts: &mut LocalFacts,
) {
    match pattern {
        Pattern::Name { name, range } => {
            let fact = InferredValue::flow(value.shape.clone(), *range);
            facts.record(name.clone(), *range, fact.clone());
            environment.insert(name.clone(), fact);
        }
        Pattern::Tuple { elements, .. } | Pattern::List { elements, .. } => {
            for (index, element) in elements.iter().enumerate() {
                let projected = match &value.shape {
                    ValueShape::Tuple(values) => values.get(index).cloned().unwrap_or(ValueShape::Unknown),
                    ValueShape::List(element) => (**element).clone(),
                    _ => ValueShape::Unknown,
                };
                bind_pattern(
                    element,
                    &InferredValue::flow(projected, element.range()),
                    module,
                    current_class,
                    known_class,
                    environment,
                    facts,
                );
            }
            if let Pattern::List { rest: Some(rest), .. } = pattern {
                bind_pattern(
                    rest,
                    &InferredValue::flow(ValueShape::List(Box::new(ValueShape::Unknown)), rest.range()),
                    module,
                    current_class,
                    known_class,
                    environment,
                    facts,
                );
            }
        }
    }
}

fn core_class(name: &str) -> ClassId {
    ClassId::new(ModuleId::new(CORE_MODULE_URI), name)
}

fn call_selector(method: &str, args: &[PackItem]) -> String {
    let labels = args
        .iter()
        .map(|arg| match arg {
            PackItem::Positional { .. } | PackItem::Expand { .. } => None,
            PackItem::Labeled {
                label: PackLabel::Static { text, .. },
                ..
            } => Some(text.clone()),
            PackItem::Labeled { .. } => None,
        })
        .collect::<Vec<_>>();
    crate::selectors::comma_form_from_labels(method, &labels)
}

fn product_label(label: &ProductLabel) -> String {
    match label {
        ProductLabel::Static {
            symbol: SymbolLiteralKind::Name(name),
            ..
        } => name.clone(),
        ProductLabel::Static {
            symbol: SymbolLiteralKind::Selector { name, .. },
            ..
        } => name.clone(),
        ProductLabel::Computed { .. } => "?".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_ast::parser::parse;

    fn no_classes(_: &str) -> Option<ClassId> {
        None
    }

    #[test]
    fn literals_and_reassignment_are_queryable() {
        let program = parse("let value = 1\nvalue = \"ok\"\n", 0).program;
        let module = ModuleId::new("file:///main.ph");
        let facts = collect_local_facts(&program, &module, no_classes, |_, _| false);
        assert_eq!(facts.binding_at("value", 10).unwrap().shape, ValueShape::Instance(core_class("Int")));
        assert_eq!(facts.binding_at("value", 30).unwrap().shape, ValueShape::Instance(core_class("String")));
    }

    #[test]
    fn union_widens_after_nine_distinct_shapes() {
        let module = ModuleId::new("file:///main.ph");
        let mut shape = ValueShape::Instance(ClassId::new(module.clone(), "C0"));
        for index in 1..=7 {
            shape = shape.join(&ValueShape::Instance(ClassId::new(module.clone(), format!("C{index}"))));
        }
        assert!(matches!(shape, ValueShape::Union(_)));
        shape = shape.join(&ValueShape::Instance(ClassId::new(module, "C8")));
        assert_eq!(shape, ValueShape::Unknown);
    }
}
