use crate::ast::{
    ClassMember, EnumBehaviorMember, EnumMember, FieldDef, GetterDef, IndexAccessor, IndexMethodDef, MethodDef, ParameterDef, RestMode, SetterDef, VariantDecl,
};
pub use phalcom_common::selector::{
    Selector, SelectorBase, SelectorError, SelectorKind, SelectorKindPattern, SelectorPattern, SelectorSlot, decode_label_component, encode_label_component,
};

fn selector_slot_from_parameter(param: &ParameterDef) -> SelectorSlot {
    match (&param.rest_mode, &param.label) {
        (RestMode::None, Some(label)) => SelectorSlot::Label(label.clone()),
        _ => SelectorSlot::Positional,
    }
}

/// Constructs a structural [`Selector`] for a method declaration.
pub fn selector_from_method(m: &MethodDef) -> Selector {
    let slots = m.params.iter().map(selector_slot_from_parameter).collect::<Vec<_>>();
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

/// Constructs a structural [`Selector`] for an enum [`VariantDecl`].
pub fn selector_from_variant(variant: &VariantDecl) -> Selector {
    match &variant.payload {
        None => Selector::getter(&variant.name).unwrap_or_else(|_| Selector {
            base: SelectorBase::Named(variant.name.clone()),
            kind: SelectorKind::Getter,
            slots: Box::new([]),
        }),
        Some(payload) => {
            let slots = payload.parameters.iter().map(selector_slot_from_parameter).collect::<Vec<_>>();
            Selector::method(&variant.name, slots).unwrap_or_else(|_| Selector {
                base: SelectorBase::Named(variant.name.clone()),
                kind: SelectorKind::Method,
                slots: Box::new([]),
            })
        }
    }
}

/// Constructs a structural [`Selector`] for an [`EnumBehaviorMember`].
pub fn selector_from_enum_behavior_member(member: &EnumBehaviorMember) -> Selector {
    match member {
        EnumBehaviorMember::Method(m) => selector_from_method(m),
        EnumBehaviorMember::Getter(g) => selector_from_getter(g),
        EnumBehaviorMember::Setter(s) => selector_from_setter(s),
        EnumBehaviorMember::Index(ix) => selector_from_index(ix),
    }
}

/// Constructs a structural [`Selector`] for an [`EnumMember`].
pub fn selector_from_enum_member(member: &EnumMember) -> Selector {
    match member {
        EnumMember::Variant(v) => selector_from_variant(v),
        EnumMember::Behavior(b) => selector_from_enum_behavior_member(b),
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

fn slot_from_variant_pattern_arg(arg: &crate::ast::VariantPatternArgument) -> SelectorSlot {
    match &arg.label {
        Some(label) => SelectorSlot::Label(label.clone()),
        None => SelectorSlot::Positional,
    }
}

/// Constructs a structural exact [`Selector`] for a [`VariantPattern`] with `Singleton` or `ExactCall` mode.
pub fn selector_from_exact_variant_pattern(pattern: &crate::ast::VariantPattern) -> Result<Selector, SelectorError> {
    match &pattern.mode {
        crate::ast::VariantPatternMode::Singleton => Selector::getter(&pattern.base),
        crate::ast::VariantPatternMode::ExactCall { arguments } => {
            let slots = arguments.iter().map(slot_from_variant_pattern_arg).collect::<Vec<_>>();
            Selector::method(&pattern.base, slots)
        }
        _ => Err(SelectorError::InvalidPatternSlots),
    }
}

/// Constructs a structural [`SelectorPattern`] for a [`VariantPattern`] with `CallablePattern` mode.
pub fn selector_pattern_from_variant_pattern(pattern: &crate::ast::VariantPattern) -> Result<SelectorPattern, SelectorError> {
    match &pattern.mode {
        crate::ast::VariantPatternMode::CallablePattern { prefix, suffix, .. } => {
            let prefix_slots = prefix.iter().map(slot_from_variant_pattern_arg).collect::<Vec<_>>();
            let suffix_slots = suffix.iter().map(slot_from_variant_pattern_arg).collect::<Vec<_>>();
            SelectorPattern::new(
                SelectorBase::Named(pattern.base.clone()),
                SelectorKindPattern::Exact(SelectorKind::Method),
                prefix_slots.into_boxed_slice(),
                suffix_slots.into_boxed_slice(),
                true,
            )
        }
        _ => Err(SelectorError::MissingGap),
    }
}

