//! General resolved-pattern × static-value matching engine (LANG007.C4.P6 T3).

use super::error::StaticEvalError;
use crate::db::{CancellationToken, QueryBudget};
use crate::identity::BindingId;
use crate::match_semantics::PatternResolution;
use crate::static_value::{StaticValueData, StaticValueRef, StaticValueStore};
use crate::types::outcome::BudgetReport;
use std::collections::BTreeMap;

/// Bindings established by matching a pattern against a static value.
pub(crate) type StaticPatternBindings = BTreeMap<BindingId, StaticValueRef>;

/// Outcome of matching one pattern against one static value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum StaticPatternMatchOutcome {
    Matched(StaticPatternBindings),
    NoMatch,
    Symbolic,
    Invalid(StaticEvalError),
    Cancelled,
    BudgetExceeded(BudgetReport),
}

impl StaticPatternMatchOutcome {
    #[allow(dead_code)]
    pub fn is_matched(&self) -> bool {
        matches!(self, Self::Matched(_))
    }
}

/// Matches a resolved pattern against an immutable static value reference.
pub(crate) fn match_pattern_against_value(
    pattern: &PatternResolution,
    value: StaticValueRef,
    store: &StaticValueStore,
    budget: &mut QueryBudget,
    cancellation: &CancellationToken,
) -> StaticPatternMatchOutcome {
    if cancellation.is_cancelled() {
        return StaticPatternMatchOutcome::Cancelled;
    }
    if let Err(report) = budget.charge_step() {
        return StaticPatternMatchOutcome::BudgetExceeded(report);
    }

    match pattern {
        PatternResolution::Wildcard => StaticPatternMatchOutcome::Matched(BTreeMap::new()),

        PatternResolution::Binding { binding, .. } => {
            let mut bindings = BTreeMap::new();
            bindings.insert(*binding, value);
            StaticPatternMatchOutcome::Matched(bindings)
        }

        PatternResolution::Char(expected) => match store.get(value) {
            Ok(StaticValueData::Char(c)) if *c == *expected => StaticPatternMatchOutcome::Matched(BTreeMap::new()),
            Ok(_) => StaticPatternMatchOutcome::NoMatch,
            Err(e) => StaticPatternMatchOutcome::Invalid(StaticEvalError::ValueStore(e.to_string())),
        },

        PatternResolution::Tuple(elements) => match store.get(value) {
            Ok(StaticValueData::Tuple(tuple_id)) => {
                let tuple_elements = match store.get_tuple(*tuple_id) {
                    Ok(elems) => elems,
                    Err(e) => return StaticPatternMatchOutcome::Invalid(StaticEvalError::ValueStore(e.to_string())),
                };
                if elements.len() != tuple_elements.len() {
                    return StaticPatternMatchOutcome::NoMatch;
                }
                let mut all_bindings = BTreeMap::new();
                for (pat, elem_val) in elements.iter().zip(tuple_elements.iter()) {
                    match match_pattern_against_value(pat, *elem_val, store, budget, cancellation) {
                        StaticPatternMatchOutcome::Matched(b) => all_bindings.extend(b),
                        StaticPatternMatchOutcome::NoMatch => return StaticPatternMatchOutcome::NoMatch,
                        terminal => return terminal,
                    }
                }
                StaticPatternMatchOutcome::Matched(all_bindings)
            }
            Ok(_) => StaticPatternMatchOutcome::NoMatch,
            Err(e) => StaticPatternMatchOutcome::Invalid(StaticEvalError::ValueStore(e.to_string())),
        },

        PatternResolution::Variant(variant_pat) => match store.get(value) {
            Ok(StaticValueData::Variant(v_id)) => {
                let variant_value = match store.get_variant(*v_id) {
                    Ok(v) => v,
                    Err(e) => return StaticPatternMatchOutcome::Invalid(StaticEvalError::ValueStore(e.to_string())),
                };
                let candidate = variant_pat.candidates.iter().find(|c| c.variant == variant_value.variant);
                let Some(candidate) = candidate else {
                    if variant_pat.blocked.is_some() {
                        return StaticPatternMatchOutcome::Symbolic;
                    }
                    return StaticPatternMatchOutcome::NoMatch;
                };

                let mut all_bindings = BTreeMap::new();
                for (i, field_pat) in candidate.fields.iter().enumerate() {
                    let field_val = variant_value
                        .fields
                        .iter()
                        .find(|(fid, _)| *fid == field_pat.field)
                        .map(|(_, val)| *val)
                        .or_else(|| variant_value.fields.get(i).map(|(_, val)| *val));

                    let Some(field_val) = field_val else {
                        return StaticPatternMatchOutcome::NoMatch;
                    };

                    match match_pattern_against_value(&field_pat.child, field_val, store, budget, cancellation) {
                        StaticPatternMatchOutcome::Matched(b) => all_bindings.extend(b),
                        StaticPatternMatchOutcome::NoMatch => return StaticPatternMatchOutcome::NoMatch,
                        terminal => return terminal,
                    }
                }
                StaticPatternMatchOutcome::Matched(all_bindings)
            }
            Ok(StaticValueData::Data(data_id)) => {
                let data_value = match store.get_data(*data_id) {
                    Ok(d) => d,
                    Err(e) => return StaticPatternMatchOutcome::Invalid(StaticEvalError::ValueStore(e.to_string())),
                };
                let candidate = variant_pat.candidates.iter().find(|c| c.variant.owner == data_value.owner);
                let Some(candidate) = candidate else {
                    if variant_pat.blocked.is_some() {
                        return StaticPatternMatchOutcome::Symbolic;
                    }
                    return StaticPatternMatchOutcome::NoMatch;
                };

                let mut all_bindings = BTreeMap::new();
                for (i, field_pat) in candidate.fields.iter().enumerate() {
                    let field_val = data_value.components.get(i).map(|(_, val)| *val);
                    let Some(field_val) = field_val else {
                        return StaticPatternMatchOutcome::NoMatch;
                    };
                    match match_pattern_against_value(&field_pat.child, field_val, store, budget, cancellation) {
                        StaticPatternMatchOutcome::Matched(b) => all_bindings.extend(b),
                        StaticPatternMatchOutcome::NoMatch => return StaticPatternMatchOutcome::NoMatch,
                        terminal => return terminal,
                    }
                }
                StaticPatternMatchOutcome::Matched(all_bindings)
            }
            Ok(_) => StaticPatternMatchOutcome::NoMatch,
            Err(e) => StaticPatternMatchOutcome::Invalid(StaticEvalError::ValueStore(e.to_string())),
        },

        PatternResolution::Or(or_pat) => {
            for alt in or_pat.alternatives.iter() {
                match match_pattern_against_value(alt, value, store, budget, cancellation) {
                    StaticPatternMatchOutcome::Matched(b) => return StaticPatternMatchOutcome::Matched(b),
                    StaticPatternMatchOutcome::NoMatch => continue,
                    terminal => return terminal,
                }
            }
            StaticPatternMatchOutcome::NoMatch
        }

        PatternResolution::Record(fields) => match store.get(value) {
            Ok(StaticValueData::Record(rec_id)) => {
                let record_fields = match store.get_record(*rec_id) {
                    Ok(f) => f,
                    Err(e) => return StaticPatternMatchOutcome::Invalid(StaticEvalError::ValueStore(e.to_string())),
                };
                let mut all_bindings = BTreeMap::new();
                for field_pat in fields.iter() {
                    let field_val = record_fields.iter().find(|f| match &f.key {
                        crate::field_key::FieldKey::Symbol(s) => s.as_ref() == &*field_pat.label,
                        crate::field_key::FieldKey::String(s) => s.as_ref() == &*field_pat.label,
                        _ => false,
                    }).map(|f| f.value);
                    let Some(field_val) = field_val else {
                        return StaticPatternMatchOutcome::NoMatch;
                    };
                    match match_pattern_against_value(&field_pat.child, field_val, store, budget, cancellation) {
                        StaticPatternMatchOutcome::Matched(b) => all_bindings.extend(b),
                        StaticPatternMatchOutcome::NoMatch => return StaticPatternMatchOutcome::NoMatch,
                        terminal => return terminal,
                    }
                }
                StaticPatternMatchOutcome::Matched(all_bindings)
            }
            Ok(_) => StaticPatternMatchOutcome::NoMatch,
            Err(e) => StaticPatternMatchOutcome::Invalid(StaticEvalError::ValueStore(e.to_string())),
        },

        PatternResolution::List(list_pat) => match store.get(value) {
            Ok(StaticValueData::Sequence(seq_id)) => {
                let seq_data = match store.get_sequence(*seq_id) {
                    Ok(s) => s,
                    Err(e) => return StaticPatternMatchOutcome::Invalid(StaticEvalError::ValueStore(e.to_string())),
                };
                let items = seq_data.elements();
                if items.len() < list_pat.prefix.len() {
                    return StaticPatternMatchOutcome::NoMatch;
                }
                if list_pat.rest.is_none() && items.len() != list_pat.prefix.len() {
                    return StaticPatternMatchOutcome::NoMatch;
                }
                let mut all_bindings = BTreeMap::new();
                for (pat, item_val) in list_pat.prefix.iter().zip(items.iter()) {
                    match match_pattern_against_value(pat, *item_val, store, budget, cancellation) {
                        StaticPatternMatchOutcome::Matched(b) => all_bindings.extend(b),
                        StaticPatternMatchOutcome::NoMatch => return StaticPatternMatchOutcome::NoMatch,
                        terminal => return terminal,
                    }
                }
                if let Some(rest_pat) = &list_pat.rest {
                    if matches!(**rest_pat, PatternResolution::Wildcard) {
                        // Wildcard matches remaining slice
                    } else {
                        return StaticPatternMatchOutcome::Invalid(StaticEvalError::UnsupportedExpression);
                    }
                }
                StaticPatternMatchOutcome::Matched(all_bindings)
            }
            Ok(_) => StaticPatternMatchOutcome::NoMatch,
            Err(e) => StaticPatternMatchOutcome::Invalid(StaticEvalError::ValueStore(e.to_string())),
        },

        PatternResolution::Map(_) => StaticPatternMatchOutcome::NoMatch,
    }
}
