//! Structural selector construction from AST nodes over shared common selector semantics.
//!
//! The index (`crate::index`), go-to-definition, find-references, and
//! `workspace/symbol` all key on canonical selector strings or structural
//! [`Selector`] values. This module provides AST-to-Selector conversion
//! powered by `phalcom_common::selector`.

use phalcom_ast::ast::{ClassMember, FieldDef, GetterDef, IndexMethodDef, MethodDef, NormalizedSelectorSpec, PackItem, PackLabel, SelectorSpecSyntax, SetterDef};
pub use phalcom_common::selector::{
    Selector, SelectorBase, SelectorError, SelectorKind, SelectorKindPattern, SelectorPattern, SelectorSlot, decode_label_component, encode_label_component,
};

pub use phalcom_ast::selector::{
    comma_form, comma_form_from_labels, selector_from_field, selector_from_getter, selector_from_index, selector_from_member, selector_from_method,
    selector_from_setter,
};

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
