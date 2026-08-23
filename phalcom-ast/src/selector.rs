//! Structural selector construction from AST nodes over shared common selector semantics.

use crate::ast::{ClassMember, FieldDef, GetterDef, IndexAccessor, IndexMethodDef, MethodDef, ParameterDef, RestMode, SetterDef};
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
