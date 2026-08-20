//! Structural selector construction from AST nodes over shared common selector semantics.
//!
//! The index (`crate::index`), go-to-definition, find-references, and
//! `workspace/symbol` all key on canonical selector strings or structural
//! [`Selector`] values. This module provides AST-to-Selector conversion
//! powered by `phalcom_common::selector`.

use phalcom_ast::ast::{
    BinaryOp, ClassMember, FieldDef, GetterDef, IndexAccessor, IndexMethodDef, MethodDef, NormalizedSelectorSpec, PackItem, PackLabel, ParameterDef, RestMode,
    SelectorSpecSyntax, SetterDef, UnaryOp,
};
pub use phalcom_common::selector::{
    Selector, SelectorBase, SelectorError, SelectorKind, SelectorKindPattern, SelectorPattern, SelectorSlot, decode_label_component, encode_label_component,
};

/// Constructs a structural [`Selector`] for a method declaration.
pub fn selector_from_method(m: &MethodDef) -> Selector {
    let slots = m
        .params
        .iter()
        .map(|param| match (&param.rest_mode, &param.label) {
            (RestMode::None, Some(label)) => SelectorSlot::Label(label.clone()),
            _ => SelectorSlot::Positional,
        })
        .collect::<Vec<_>>();
    Selector::method(&m.name, slots).unwrap_or_else(|_| Selector {
        base: SelectorBase::Named(m.name.clone()),
        kind: SelectorKind::Method,
        slots: Box::new([]),
    })
}

/// Constructs a structural [`Selector`] for a getter declaration.
pub fn selector_from_getter(g: &GetterDef) -> Selector {
    Selector::getter(&g.name).unwrap_or_else(|_| Selector {
        base: SelectorBase::Named(g.name.clone()),
        kind: SelectorKind::Getter,
        slots: Box::new([]),
    })
}

/// Constructs a structural [`Selector`] for a setter declaration.
pub fn selector_from_setter(s: &SetterDef) -> Selector {
    Selector::setter(&s.name).unwrap_or_else(|_| Selector {
        base: SelectorBase::Named(s.name.clone()),
        kind: SelectorKind::Setter,
        slots: Box::new([]),
    })
}

/// Constructs a structural [`Selector`] for a field declaration (read access).
pub fn selector_from_field(f: &FieldDef) -> Selector {
    Selector::getter(&f.name).unwrap_or_else(|_| Selector {
        base: SelectorBase::Named(f.name.clone()),
        kind: SelectorKind::Getter,
        slots: Box::new([]),
    })
}

/// Constructs a structural [`Selector`] for a subscript method declaration.
pub fn selector_from_index(ix: &IndexMethodDef) -> Selector {
    let slots = ix
        .params
        .iter()
        .map(|param| {
            if let Some(label) = &param.label {
                SelectorSlot::Label(label.clone())
            } else {
                SelectorSlot::Positional
            }
        })
        .collect::<Vec<_>>();
    let kind = match &ix.accessor {
        IndexAccessor::Get => SelectorKind::SubscriptGet,
        IndexAccessor::Set { .. } => SelectorKind::SubscriptSet,
    };
    Selector::new(SelectorBase::Subscript, kind, slots.into_boxed_slice()).unwrap_or_else(|_| Selector {
        base: SelectorBase::Subscript,
        kind,
        slots: Box::new([]),
    })
}

/// Constructs a structural [`Selector`] for any [`ClassMember`].
pub fn selector_from_member(member: &ClassMember) -> Selector {
    match member {
        ClassMember::Method(m) => selector_from_method(m),
        ClassMember::Getter(g) => selector_from_getter(g),
        ClassMember::Setter(s) => selector_from_setter(s),
        ClassMember::Field(f) => selector_from_field(f),
        ClassMember::Variant(v) => Selector::getter(&v.name).unwrap_or_else(|_| Selector {
            base: SelectorBase::Named(v.name.clone()),
            kind: SelectorKind::Getter,
            slots: Box::new([]),
        }),
        ClassMember::Index(ix) => selector_from_index(ix),
    }
}

/// Extracts static argument slots from call pack items. Returns `None` if dynamic/expand pack is present.
pub fn static_call_slots(args: &[PackItem]) -> Option<Vec<SelectorSlot>> {
    let mut slots = Vec::with_capacity(args.len());
    for arg in args {
        match arg {
            PackItem::Positional { .. } => slots.push(SelectorSlot::Positional),
            PackItem::Labeled {
                label: PackLabel::Static { text, .. },
                ..
            } => slots.push(SelectorSlot::Label(text.clone())),
            PackItem::Labeled {
                label: PackLabel::Computed { .. },
                ..
            }
            | PackItem::Expand { .. } => return None,
        }
    }
    Some(slots)
}

/// Builds a call-site structural [`Selector`] when all argument slots are statically known.
///
/// Returns `None` if computed/dynamic packs or expansions prevent exact static reconstruction.
pub fn selector_from_call(name: &str, args: &[PackItem]) -> Option<Selector> {
    let slots = static_call_slots(args)?;
    Selector::method(name, slots).ok()
}

/// Normalizes an AST selector spec into common structural selector forms.
pub fn selector_spec_from_ast(spec: &SelectorSpecSyntax) -> Result<NormalizedSelectorSpec, SelectorError> {
    spec.normalize()
}

/// Builds the canonical comma-form selector string from a method name and its parameter list.
pub fn comma_form(name: &str, params: &[ParameterDef]) -> String {
    let slots = params
        .iter()
        .map(|param| match (&param.rest_mode, &param.label) {
            (RestMode::None, Some(label)) => SelectorSlot::Label(label.clone()),
            _ => SelectorSlot::Positional,
        })
        .collect::<Vec<_>>();
    Selector::method(name, slots).map(|s| s.encode()).unwrap_or_else(|_| format!("{name}()"))
}

/// Builds the canonical comma-form selector string from a method name and argument labels.
pub fn comma_form_from_labels(name: &str, labels: &[Option<String>]) -> String {
    let slots = labels
        .iter()
        .map(|label| {
            if let Some(text) = label {
                SelectorSlot::Label(text.clone())
            } else {
                SelectorSlot::Positional
            }
        })
        .collect::<Vec<_>>();
    Selector::method(name, slots).map(|s| s.encode()).unwrap_or_else(|_| format!("{name}()"))
}

/// Builds a call-site selector string from argument packs.
pub(crate) fn call_selector(name: &str, args: &[PackItem]) -> String {
    if let Some(sel) = selector_from_call(name, args) {
        sel.encode()
    } else {
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
        comma_form_from_labels(name, &labels)
    }
}

/// Maps a binary operator to the selector emitted by the compiler.
pub(crate) fn binary_selector_name(op: &BinaryOp) -> Option<&'static str> {
    Some(match op {
        BinaryOp::Add => "+",
        BinaryOp::Subtract => "-",
        BinaryOp::Multiply => "*",
        BinaryOp::Divide => "/",
        BinaryOp::IntegerDivide => "~/",
        BinaryOp::Power => "**",
        BinaryOp::Modulo => "%",
        BinaryOp::ShiftLeft => "<<",
        BinaryOp::ShiftRight => ">>",
        BinaryOp::BitAnd => "&",
        BinaryOp::BitXor => "^",
        BinaryOp::BitOr => "|",
        BinaryOp::Equal => "==",
        BinaryOp::NotEqual => "!=",
        BinaryOp::LessThan => "<",
        BinaryOp::LessThanOrEqual => "<=",
        BinaryOp::GreaterThan => ">",
        BinaryOp::GreaterThanOrEqual => ">=",
        BinaryOp::And | BinaryOp::Or => return None,
    })
}

/// Maps a unary operator to the bare getter selector emitted by the compiler.
pub(crate) fn unary_selector_name(op: &UnaryOp) -> &'static str {
    match op {
        UnaryOp::Plus => "+",
        UnaryOp::Minus => "-",
        UnaryOp::Not => "not",
        UnaryOp::BitNot => "~",
    }
}

/// The comma-form selector a getter's bare-name access resolves to.
pub fn getter_selector(g: &GetterDef) -> String {
    selector_from_getter(g).encode()
}

/// The comma-form selector a `name=(put)` write resolves to, given just the bare property name.
pub fn setter_selector_from_name(name: &str) -> String {
    Selector::setter(name).map(|s| s.encode()).unwrap_or_else(|_| format!("{name}=(put)"))
}

/// The comma-form selector a setter's write resolves to.
pub fn setter_selector(s: &SetterDef) -> String {
    selector_from_setter(s).encode()
}

/// The comma-form selector a method declaration defines.
pub fn method_selector(m: &MethodDef) -> String {
    selector_from_method(m).encode()
}

/// The comma-form selector a declared class field's bare-name read resolves to.
pub fn field_selector(f: &FieldDef) -> String {
    selector_from_field(f).encode()
}

/// The bracket-form selector a bracket subscript method declaration defines.
pub fn index_selector(ix: &IndexMethodDef) -> String {
    selector_from_index(ix).encode()
}

/// Builds a bracket selector from call-site argument labels.
pub fn index_selector_from_labels(labels: &[Option<String>], setter: bool) -> String {
    let slots = labels
        .iter()
        .map(|label| {
            if let Some(text) = label {
                SelectorSlot::Label(text.clone())
            } else {
                SelectorSlot::Positional
            }
        })
        .collect::<Vec<_>>();
    let kind = if setter { SelectorKind::SubscriptSet } else { SelectorKind::SubscriptGet };
    Selector::new(SelectorBase::Subscript, kind, slots.into_boxed_slice())
        .map(|s| s.encode())
        .unwrap_or_else(|_| if setter { "[_]=(put)".into() } else { "[_]".into() })
}

/// The comma-form selector any [`ClassMember`] declaration defines.
pub fn class_member_selector(member: &ClassMember) -> String {
    selector_from_member(member).encode()
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_ast::ast::Statement;
    use phalcom_ast::parser::parse;

    fn parse_class(src: &str) -> phalcom_ast::ast::ClassDef {
        let parsed = parse(src, 0);
        assert!(parsed.errors.is_empty(), "unexpected parse errors: {:?}", parsed.errors);
        for statement in parsed.program.statements {
            if let Statement::Class(class_def) = statement {
                return class_def;
            }
        }
        panic!("no class found in {src:?}");
    }

    #[test]
    fn method_with_positional_and_labeled_params() {
        let class_def = parse_class("class Point {\n  move(_ x, to, duration) { }\n}\n");
        let member = &class_def.members[0];
        let selector = selector_from_member(member);
        assert_eq!(selector.kind, SelectorKind::Method);
        assert_eq!(selector.encode(), "move(_,to,duration)");
        assert_eq!(class_member_selector(member), "move(_,to,duration)");
    }

    #[test]
    fn zero_arity_method_is_not_bare_name() {
        let class_def = parse_class("class Point {\n  reset() { }\n}\n");
        let member = &class_def.members[0];
        let selector = selector_from_member(member);
        assert_eq!(selector.kind, SelectorKind::Method);
        assert_eq!(selector.encode(), "reset()");
    }

    #[test]
    fn getter_has_no_parens_and_never_aliases_zero_arity_method() {
        let class_def = parse_class("class Point {\n  y { }\n  x() { }\n}\n");
        let getter_member = &class_def.members[0];
        let method_member = &class_def.members[1];
        let getter_sel = selector_from_member(getter_member);
        let method_sel = selector_from_member(method_member);
        assert_eq!(getter_sel.kind, SelectorKind::Getter);
        assert_eq!(method_sel.kind, SelectorKind::Method);
        assert_eq!(getter_sel.encode(), "y");
        assert_eq!(method_sel.encode(), "x()");
        assert_ne!(getter_sel, method_sel);
    }

    #[test]
    fn setter_is_literal_single_slot() {
        let class_def = parse_class("class Point {\n  x=(put v) { }\n}\n");
        let member = &class_def.members[0];
        let selector = selector_from_member(member);
        assert_eq!(selector.kind, SelectorKind::Setter);
        assert_eq!(selector.encode(), "x=(put)");
    }

    #[test]
    fn construct_is_comma_form() {
        let class_def = parse_class("class Point {\n  @constructor\n  new(_ x, y) { }\n}\n");
        let member = &class_def.members[0];
        let selector = selector_from_member(member);
        assert_eq!(selector.encode(), "new(_,y)");
    }

    #[test]
    fn call_site_labels_match_declaration_selector() {
        assert_eq!(
            comma_form_from_labels("move", &[None, Some("to".to_string()), Some("duration".to_string())]),
            "move(_,to,duration)"
        );
        assert_eq!(comma_form_from_labels("reset", &[]), "reset()");
    }

    #[test]
    fn subscript_get_and_set() {
        let class_def = parse_class("class Arr {\n  [_ idx] { }\n  [_ idx]=(put value) { }\n}\n");
        let get_member = &class_def.members[0];
        let set_member = &class_def.members[1];
        let get_sel = selector_from_member(get_member);
        let set_sel = selector_from_member(set_member);
        assert_eq!(get_sel.kind, SelectorKind::SubscriptGet);
        assert_eq!(set_sel.kind, SelectorKind::SubscriptSet);
        assert_eq!(get_sel.encode(), "[_]");
        assert_eq!(set_sel.encode(), "[_]=(put)");
    }
}
