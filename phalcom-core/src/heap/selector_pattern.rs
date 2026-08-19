//! VM-native runtime representation and matching for selector patterns.

use phalcom_common::selector::{SelectorBase, SelectorKind, SelectorKindPattern, SelectorPattern, SelectorSlot};

use crate::interner::{Interner, Symbol};

/// The base component of a runtime selector pattern.
///
/// `Subscript` is encoded in the runtime representation to support future
/// subscript Family activation, but it is not currently handled by
/// `activate_family_with_kind`. Activating a subscript Family returns a
/// `RuntimeError::Message` until `FamilyInvocationKind::SubscriptGet` and
/// `FamilyInvocationKind::SubscriptSet` variants are added.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum RuntimeSelectorBase {
    Named(Symbol),
    Subscript,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum RuntimeSelectorKindPattern {
    AnyNamed,
    Exact(SelectorKind),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum RuntimeSelectorSlot {
    Positional,
    Label(Symbol),
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeSelectorPattern {
    pub(crate) base: RuntimeSelectorBase,
    pub(crate) kind: RuntimeSelectorKindPattern,
    pub(crate) prefix: Box<[RuntimeSelectorSlot]>,
    pub(crate) suffix: Box<[RuntimeSelectorSlot]>,
    pub(crate) has_gap: bool,
}

impl RuntimeSelectorPattern {
    pub(crate) fn compile(pattern: &SelectorPattern, interner: &mut Interner) -> Self {
        let base = match &pattern.base {
            SelectorBase::Named(name) => RuntimeSelectorBase::Named(interner.intern(name)),
            SelectorBase::Subscript => RuntimeSelectorBase::Subscript,
        };

        let kind = match pattern.kind {
            SelectorKindPattern::AnyNamed => RuntimeSelectorKindPattern::AnyNamed,
            SelectorKindPattern::Exact(k) => RuntimeSelectorKindPattern::Exact(k),
        };

        let prefix = pattern
            .prefix
            .iter()
            .map(|slot| match slot {
                SelectorSlot::Positional => RuntimeSelectorSlot::Positional,
                SelectorSlot::Label(l) => RuntimeSelectorSlot::Label(interner.intern(l)),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();

        let suffix = pattern
            .suffix
            .iter()
            .map(|slot| match slot {
                SelectorSlot::Positional => RuntimeSelectorSlot::Positional,
                SelectorSlot::Label(l) => RuntimeSelectorSlot::Label(interner.intern(l)),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();

        Self {
            base,
            kind,
            prefix,
            suffix,
            has_gap: pattern.has_gap,
        }
    }

    /// Checks whether a call with the given `kind`, `positional_count`, and
    /// `labels` matches this pattern.
    ///
    /// # Precondition
    ///
    /// This method does **not** compare selector bases. The caller is responsible
    /// for ensuring that the actual call's base already matches `self.base`
    /// before invoking this method. In `activate_family_with_kind`, the dispatch
    /// gateway always derives the actual base from `self.base`, so this
    /// precondition is satisfied by construction.
    ///
    /// To add an independent runtime check, add a `base: &str` parameter and
    /// compare it against `self.base` (for `RuntimeSelectorBase::Named(sym)`,
    /// resolve `sym` and compare strings).
    pub(crate) fn matches_call(&self, kind: SelectorKind, positional_count: usize, labels: &[Symbol]) -> bool {
        match self.kind {
            RuntimeSelectorKindPattern::AnyNamed => {
                if !matches!(kind, SelectorKind::Getter | SelectorKind::Setter | SelectorKind::Method) {
                    return false;
                }
            }
            RuntimeSelectorKindPattern::Exact(expected) => {
                if kind != expected {
                    return false;
                }
            }
        }

        let actual_len = positional_count + labels.len();
        let min_len = self.prefix.len() + self.suffix.len();

        if (self.has_gap && actual_len < min_len) || (!self.has_gap && actual_len != min_len) {
            return false;
        }

        // Compare prefix against slots [0 .. prefix.len]
        for (i, expected_slot) in self.prefix.iter().enumerate() {
            if call_slot_at(i, positional_count, labels) != Some(*expected_slot) {
                return false;
            }
        }

        // Compare suffix against slots [actual_len - suffix.len .. actual_len]
        let suffix_start = actual_len - self.suffix.len();
        for (i, expected_slot) in self.suffix.iter().enumerate() {
            if call_slot_at(suffix_start + i, positional_count, labels) != Some(*expected_slot) {
                return false;
            }
        }

        true
    }
}

#[inline]
fn call_slot_at(index: usize, positional_count: usize, labels: &[Symbol]) -> Option<RuntimeSelectorSlot> {
    if index < positional_count {
        Some(RuntimeSelectorSlot::Positional)
    } else {
        labels.get(index - positional_count).copied().map(RuntimeSelectorSlot::Label)
    }
}

/// Runtime payload for a structural selector pattern.
#[derive(Debug, Clone)]
pub struct SelectorPatternObject {
    /// Rich semantic representation retained for reflection and diagnostics.
    pub pattern: SelectorPattern,

    /// VM-oriented immutable matcher compiled once.
    pub(crate) runtime: RuntimeSelectorPattern,
}

impl SelectorPatternObject {
    /// Compiles a rich [`SelectorPattern`] into a [`SelectorPatternObject`].
    pub(crate) fn compile(pattern: SelectorPattern, interner: &mut Interner) -> Self {
        let runtime = RuntimeSelectorPattern::compile(&pattern, interner);
        Self { pattern, runtime }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_common::selector::Selector;

    #[test]
    fn test_exact_method_prefix_suffix() {
        let mut interner = Interner::with_capacity(32);
        // Pattern: move(_, ..., duration)
        let rich = SelectorPattern::named_method("move", vec![SelectorSlot::Positional], vec![SelectorSlot::Label("duration".into())], true).unwrap();

        let runtime = RuntimeSelectorPattern::compile(&rich, &mut interner);

        let duration_sym = interner.intern("duration");
        let to_sym = interner.intern("to");
        let easing_sym = interner.intern("easing");

        // match move(_, duration) -> pos: 1, labels: [duration]
        assert!(runtime.matches_call(SelectorKind::Method, 1, &[duration_sym]));

        // match move(_, to, duration) -> pos: 1, labels: [to, duration]
        assert!(runtime.matches_call(SelectorKind::Method, 1, &[to_sym, duration_sym]));

        // match move(_, to, easing, duration) -> pos: 1, labels: [to, easing, duration]
        assert!(runtime.matches_call(SelectorKind::Method, 1, &[to_sym, easing_sym, duration_sym]));

        // reject move(duration) -> pos: 0, labels: [duration] (missing pos prefix)
        assert!(!runtime.matches_call(SelectorKind::Method, 0, &[duration_sym]));

        // reject move(_, to) -> pos: 1, labels: [to] (missing duration suffix)
        assert!(!runtime.matches_call(SelectorKind::Method, 1, &[to_sym]));
    }

    #[test]
    fn test_equivalence_with_rich_selector_pattern() {
        let mut interner = Interner::with_capacity(32);

        let patterns = vec![
            // Gap with positional prefix: foo(_, _, ...)
            SelectorPattern::named_method("foo", vec![SelectorSlot::Positional, SelectorSlot::Positional], vec![], true).unwrap(),
            // Gap with label suffix: bar(..., y)
            SelectorPattern::named_method("bar", vec![], vec![SelectorSlot::Label("y".into())], true).unwrap(),
            // Prefix and suffix with gap: baz(x, ..., z)
            SelectorPattern::named_method("baz", vec![SelectorSlot::Label("x".into())], vec![SelectorSlot::Label("z".into())], true).unwrap(),
            // Empty pattern with gap: qux(...)
            SelectorPattern::named_method("qux", vec![], vec![], true).unwrap(),
        ];

        let call_scenarios = vec![
            (SelectorKind::Method, 2, vec![], vec![SelectorSlot::Positional, SelectorSlot::Positional]),
            (SelectorKind::Method, 0, vec!["y"], vec![SelectorSlot::Label("y".into())]),
            (
                SelectorKind::Method,
                1,
                vec!["y"],
                vec![SelectorSlot::Positional, SelectorSlot::Label("y".into())],
            ),
            (
                SelectorKind::Method,
                0,
                vec!["x", "z"],
                vec![SelectorSlot::Label("x".into()), SelectorSlot::Label("z".into())],
            ),
            (
                SelectorKind::Method,
                0,
                vec!["x", "m", "z"],
                vec![
                    SelectorSlot::Label("x".into()),
                    SelectorSlot::Label("m".into()),
                    SelectorSlot::Label("z".into()),
                ],
            ),
            (SelectorKind::Method, 0, vec![], vec![]),
            (SelectorKind::Getter, 0, vec![], vec![]),
        ];

        for pattern in &patterns {
            let runtime = RuntimeSelectorPattern::compile(pattern, &mut interner);

            for (kind, pos_count, label_strs, slots) in &call_scenarios {
                let sym_labels: Vec<Symbol> = label_strs.iter().map(|s| interner.intern(s)).collect();
                let name = match &pattern.base {
                    SelectorBase::Named(n) => n.clone(),
                    SelectorBase::Subscript => "subscript".to_string(),
                };

                let rich_selector = Selector::new(SelectorBase::Named(name), *kind, slots.clone().into());

                let rich_match = rich_selector.is_ok_and(|s| pattern.matches(&s));
                let runtime_match = runtime.matches_call(*kind, *pos_count, &sym_labels);

                assert_eq!(
                    rich_match, runtime_match,
                    "Mismatch for pattern {:?} against kind {:?}, pos_count {}, labels {:?}",
                    pattern, kind, pos_count, label_strs
                );
            }
        }
    }

    #[test]
    fn test_any_named_acceptance_and_subscript_rejection() {
        let mut interner = Interner::with_capacity(32);
        let rich = SelectorPattern::named("prop", SelectorKindPattern::AnyNamed, vec![], vec![], true).unwrap();

        let runtime = RuntimeSelectorPattern::compile(&rich, &mut interner);

        // AnyNamed accepts Getter, Setter, and Method
        assert!(runtime.matches_call(SelectorKind::Getter, 0, &[]));
        assert!(runtime.matches_call(SelectorKind::Setter, 0, &[]));
        assert!(runtime.matches_call(SelectorKind::Method, 0, &[]));

        // Rejects Subscript
        assert!(!runtime.matches_call(SelectorKind::SubscriptGet, 0, &[]));
        assert!(!runtime.matches_call(SelectorKind::SubscriptSet, 0, &[]));
    }

    #[test]
    fn test_exact_getter_setter_structural_arity() {
        let mut interner = Interner::with_capacity(32);
        let rich_getter = SelectorPattern::named("width", SelectorKindPattern::Exact(SelectorKind::Getter), vec![], vec![], true).unwrap();
        let runtime_getter = RuntimeSelectorPattern::compile(&rich_getter, &mut interner);

        let rich_setter = SelectorPattern::named("width", SelectorKindPattern::Exact(SelectorKind::Setter), vec![], vec![], true).unwrap();
        let runtime_setter = RuntimeSelectorPattern::compile(&rich_setter, &mut interner);

        // Getters and setters have structural arity 0 (positional = 0, labels = [])
        assert!(runtime_getter.matches_call(SelectorKind::Getter, 0, &[]));
        assert!(!runtime_getter.matches_call(SelectorKind::Setter, 0, &[]));

        assert!(runtime_setter.matches_call(SelectorKind::Setter, 0, &[]));
        assert!(!runtime_setter.matches_call(SelectorKind::Getter, 0, &[]));
    }

    #[test]
    fn test_escaped_selector_labels() {
        let mut interner = Interner::with_capacity(32);
        let escaped_label = "label with spaces:and;colons";

        let rich = SelectorPattern::named_method("custom", vec![], vec![SelectorSlot::Label(escaped_label.into())], true).unwrap();

        let runtime = RuntimeSelectorPattern::compile(&rich, &mut interner);
        let sym = interner.intern(escaped_label);

        assert!(runtime.matches_call(SelectorKind::Method, 0, &[sym]));
    }

    #[test]
    fn matches_call_does_not_check_base() {
        // Documents the precondition: matches_call ignores the caller's actual
        // base. This is acceptable only because the activation gateway always
        // fixes the base to self.base before calling matches_call.
        let mut interner = Interner::with_capacity(8);
        let pattern = RuntimeSelectorPattern::compile(&SelectorPattern::named_method("foo", vec![], vec![], true).unwrap(), &mut interner);
        // A call shaped like foo() — correct shape, wrong base from caller's perspective
        // is still accepted because matches_call doesn't see the base.
        assert!(pattern.matches_call(SelectorKind::Method, 0, &[]));
    }
}
