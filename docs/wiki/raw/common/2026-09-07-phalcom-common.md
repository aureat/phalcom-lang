# Phalcom Common source snapshot

> Source: Local repository snapshot at `phalcom-common/`
> Collected: 2026-09-07
> Published: Unknown

## `phalcom-common/Cargo.toml`

~~~~toml
[package]
name = "phalcom-common"
version = "0.1.0"
edition = "2024"

[dependencies]
thiserror = { workspace = true }
~~~~

## `phalcom-common/src/lib.rs`

~~~~rust
//! Shared, compiler-stage-agnostic utilities for the Phalcom toolchain.
//!
//! This crate is intentionally small — it contains only primitives that are
//! stable across VM redesigns and safe to document and test independently.
//!
//! ## Modules
//!
//! - [`range`] — [`CopyRange<T>`](range::CopyRange) and the
//!   [`SourceRange`](range::SourceRange) alias used by every AST node to
//!   carry byte-offset source locations.
//!
//! ## What is NOT here
//!
//! The old `Rc<RefCell<T>>` aliases (`PhRef` / `PhWeakRef` / `MaybeWeak` /
//! `phref_new` / `phref_weak`) have been **retired** by the handle/arena heap
//! redesign ([ADR-0009](../../../docs/adr/accepted/0009-handle-arena-heap.md)). Every
//! heap reference is now a `Copy` generational handle,
//! `phalcom_core::heap::ObjRef`, resolved through the VM-owned
//! `phalcom_core::heap::Heap` — see `phalcom-core/src/heap.rs`. No shared-owner
//! reference type lives in this crate anymore.

pub mod range;
pub mod selector;
~~~~

## `phalcom-common/src/range.rs`

~~~~rust
//! Source-location span types for the Phalcom compiler.
//!
//! A [`CopyRange<T>`] is a half-open interval `[start, end)` over a generic
//! index type `T`.  The primary alias [`SourceRange`] specialises it to
//! `usize` byte offsets into a source string, making it cheap to [`Copy`] on
//! every AST node.
//!
//! # Design rationale
//!
//! [`std::ops::Range<T>`] is not [`Copy`] (because arbitrary `T: Clone` ranges
//! have no blanket `Copy` impl), so passing source spans through the AST would
//! require cloning.  [`CopyRange<T>`] wraps the same two-field layout with an
//! explicit `#[derive(Copy)]`, keeping spans cheap throughout the pipeline.

use std::ops::Range;

/// A half-open byte-offset interval `[start, end)` that is [`Copy`].
///
/// This is a [`Copy`]-enabled mirror of [`std::ops::Range<T>`].  It
/// represents the interval that *includes* the element at `start` and
/// *excludes* the element at `end` (i.e. `start <= index < end`).
///
/// The primary specialisation is [`SourceRange`] (`CopyRange<usize>`), which
/// carries byte offsets into a source string.
///
/// # Examples
///
/// ```
/// use phalcom_common::range::CopyRange;
///
/// let span = CopyRange { start: 4, end: 10 };
/// assert_eq!(span.len(), 6);
/// assert!(span.contains(5));
/// assert!(!span.contains(10));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CopyRange<T> {
    /// The inclusive lower bound of the interval.
    pub start: T,
    /// The exclusive upper bound of the interval.
    pub end: T,
}

impl<T: Copy + Ord> CopyRange<T> {
    /// Creates a new range from `start` (inclusive) to `end` (exclusive).
    ///
    /// # Panics
    ///
    /// Does **not** panic — the constructor itself is infallible.  However,
    /// passing `start > end` produces a logically invalid (empty or reversed)
    /// range; callers are responsible for ensuring `start <= end`.
    pub fn new(start: T, end: T) -> Self {
        Self { start, end }
    }

    /// Returns `true` if the range contains no elements, i.e. `start >= end`.
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Returns `true` if `index` is inside the half-open interval
    /// `[self.start, self.end)`.
    pub fn contains(&self, index: T) -> bool {
        index >= self.start && index < self.end
    }

    /// Returns `true` if `other` is entirely contained within `self`.
    ///
    /// A range is considered contained when `other.start >= self.start` and
    /// `other.end <= self.end`.  Two identical ranges contain each other.
    pub fn contains_range(&self, other: &Self) -> bool {
        other.start >= self.start && other.end <= self.end
    }

    /// Returns `true` if `self` and `other` share at least one index.
    ///
    /// An empty range (`start == end`) shares no indices with anything and
    /// always returns `false`.
    pub fn overlaps(&self, other: &Self) -> bool {
        !self.is_empty() && !other.is_empty() && self.start < other.end && other.start < self.end
    }

    /// Returns the smallest range that covers both `self` and `other`.
    ///
    /// This is the convex hull of the two ranges.  Works correctly even when
    /// `self` and `other` are disjoint.
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

impl CopyRange<usize> {
    /// Returns the number of elements in the range (`end - start`).
    ///
    /// Returns `0` when `end <= start` (empty or reversed range) rather than
    /// wrapping or panicking.
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }
}

impl From<Range<usize>> for CopyRange<usize> {
    /// Converts a [`std::ops::Range<usize>`] into a [`CopyRange<usize>`].
    ///
    /// The conversion is lossless: `start` and `end` are preserved verbatim.
    fn from(value: Range<usize>) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

/// A half-open byte-offset span into a Phalcom source string.
///
/// The most commonly used specialisation of [`CopyRange`].  Every AST node
/// carries a `SourceRange` so the compiler can attach error diagnostics to
/// the exact byte range in the original source text.
pub type SourceRange = CopyRange<usize>;

/// An empty [`SourceRange`] beginning at byte offset 0.
///
/// Useful as a placeholder for synthetic AST nodes that have no corresponding
/// source location (e.g. compiler-generated desugared nodes).
///
/// # Examples
///
/// ```
/// use phalcom_common::range::{EmptySourceRange, SourceRange};
///
/// assert!(EmptySourceRange.is_empty());
/// assert_eq!(EmptySourceRange, SourceRange::default());
/// ```
#[allow(non_upper_case_globals)]
pub const EmptySourceRange: SourceRange = SourceRange { start: 0, end: 0 };

#[cfg(test)]
mod tests {
    use super::*;

    // ── Construction ──────────────────────────────────────────────────────────

    #[test]
    fn new_stores_start_and_end() {
        let r = CopyRange::new(3usize, 8);
        assert_eq!(r.start, 3);
        assert_eq!(r.end, 8);
    }

    #[test]
    fn default_is_zero_zero() {
        let r: CopyRange<usize> = CopyRange::default();
        assert_eq!(r.start, 0);
        assert_eq!(r.end, 0);
    }

    #[test]
    fn empty_source_range_is_default() {
        assert_eq!(EmptySourceRange, CopyRange::default());
    }

    // ── From<Range<usize>> ────────────────────────────────────────────────────

    #[test]
    fn from_std_range_preserves_bounds() {
        let r: CopyRange<usize> = CopyRange::from(4..9);
        assert_eq!(r.start, 4);
        assert_eq!(r.end, 9);
    }

    #[test]
    fn from_empty_std_range() {
        let r: CopyRange<usize> = CopyRange::from(5..5);
        assert!(r.is_empty());
    }

    // ── is_empty ──────────────────────────────────────────────────────────────

    #[test]
    fn is_empty_when_start_equals_end() {
        assert!(CopyRange::new(7usize, 7).is_empty());
    }

    #[test]
    fn is_not_empty_when_start_less_than_end() {
        assert!(!CopyRange::new(0usize, 1).is_empty());
    }

    // ── len ───────────────────────────────────────────────────────────────────

    #[test]
    fn len_is_end_minus_start() {
        assert_eq!(CopyRange::new(3usize, 10).len(), 7);
    }

    #[test]
    fn len_of_empty_range_is_zero() {
        assert_eq!(CopyRange::new(5usize, 5).len(), 0);
    }

    #[test]
    fn len_of_reversed_range_saturates_to_zero() {
        // reversed (start > end) — saturating_sub prevents underflow
        assert_eq!(CopyRange::new(10usize, 3).len(), 0);
    }

    // ── contains ──────────────────────────────────────────────────────────────

    #[test]
    fn contains_index_inside_range() {
        let r = CopyRange::new(3usize, 8);
        for i in 3..8 {
            assert!(r.contains(i), "expected {i} to be contained in {r:?}");
        }
    }

    #[test]
    fn does_not_contain_end_index() {
        let r = CopyRange::new(3usize, 8);
        assert!(!r.contains(8));
    }

    #[test]
    fn does_not_contain_index_before_start() {
        let r = CopyRange::new(3usize, 8);
        assert!(!r.contains(2));
    }

    #[test]
    fn empty_range_contains_nothing() {
        let r = CopyRange::new(5usize, 5);
        assert!(!r.contains(5));
        assert!(!r.contains(4));
    }

    // ── contains_range ────────────────────────────────────────────────────────

    #[test]
    fn range_contains_itself() {
        let r = CopyRange::new(2usize, 9);
        assert!(r.contains_range(&r));
    }

    #[test]
    fn range_contains_sub_range() {
        let outer = CopyRange::new(0usize, 20);
        let inner = CopyRange::new(5usize, 15);
        assert!(outer.contains_range(&inner));
        assert!(!inner.contains_range(&outer));
    }

    #[test]
    fn non_overlapping_ranges_do_not_contain_each_other() {
        let a = CopyRange::new(0usize, 5);
        let b = CopyRange::new(10usize, 15);
        assert!(!a.contains_range(&b));
        assert!(!b.contains_range(&a));
    }

    // ── overlaps ──────────────────────────────────────────────────────────────

    #[test]
    fn adjacent_ranges_do_not_overlap() {
        // [0, 5) and [5, 10) share no index
        let a = CopyRange::new(0usize, 5);
        let b = CopyRange::new(5usize, 10);
        assert!(!a.overlaps(&b));
        assert!(!b.overlaps(&a));
    }

    #[test]
    fn overlapping_ranges_detect_overlap() {
        let a = CopyRange::new(0usize, 6);
        let b = CopyRange::new(4usize, 10);
        assert!(a.overlaps(&b));
        assert!(b.overlaps(&a));
    }

    #[test]
    fn empty_range_never_overlaps() {
        let empty = CopyRange::new(5usize, 5);
        let normal = CopyRange::new(3usize, 8);
        assert!(!empty.overlaps(&normal));
        assert!(!normal.overlaps(&empty));
    }

    // ── merge ─────────────────────────────────────────────────────────────────

    #[test]
    fn merge_of_adjacent_ranges_covers_both() {
        let a = CopyRange::new(0usize, 5);
        let b = CopyRange::new(5usize, 10);
        assert_eq!(a.merge(&b), CopyRange::new(0, 10));
    }

    #[test]
    fn merge_of_overlapping_ranges() {
        let a = CopyRange::new(1usize, 7);
        let b = CopyRange::new(4usize, 12);
        assert_eq!(a.merge(&b), CopyRange::new(1, 12));
    }

    #[test]
    fn merge_of_disjoint_ranges_spans_gap() {
        let a = CopyRange::new(0usize, 3);
        let b = CopyRange::new(10usize, 15);
        assert_eq!(a.merge(&b), CopyRange::new(0, 15));
    }

    #[test]
    fn merge_with_self_is_identity() {
        let r = CopyRange::new(3usize, 9);
        assert_eq!(r.merge(&r), r);
    }

    // ── ordering / equality ───────────────────────────────────────────────────

    #[test]
    fn equal_ranges_are_eq() {
        assert_eq!(CopyRange::new(1usize, 5), CopyRange::new(1, 5));
    }

    #[test]
    fn different_ranges_are_ne() {
        assert_ne!(CopyRange::new(1usize, 5), CopyRange::new(1, 6));
    }

    // ── Copy semantics ────────────────────────────────────────────────────────

    #[test]
    fn copy_is_independent() {
        let a = CopyRange::new(1usize, 5);
        let b = a; // implicit copy
        assert_eq!(a, b);
    }
}
~~~~

## `phalcom-common/src/selector.rs`

~~~~rust
//! Structural selector identity shared by the parser, compiler, VM, and LSP.
//!
//! Runtime dispatch still interns the encoded selector as a compact symbol. The
//! types in this module describe that identity without depending on any VM or
//! AST representation, and selector patterns remain predicates rather than
//! dispatch keys.

use std::fmt;

use thiserror::Error;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SelectorKind {
    Getter,
    Setter,
    Method,
    SubscriptGet,
    SubscriptSet,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SelectorBase {
    Named(String),
    Subscript,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SelectorSlot {
    Positional,
    Label(String),
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Selector {
    pub base: SelectorBase,
    pub kind: SelectorKind,
    pub slots: Box<[SelectorSlot]>,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SelectorKindPattern {
    AnyNamed,
    Exact(SelectorKind),
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SelectorPattern {
    pub base: SelectorBase,
    pub kind: SelectorKindPattern,
    pub prefix: Box<[SelectorSlot]>,
    pub suffix: Box<[SelectorSlot]>,
    pub has_gap: bool,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum SelectorError {
    #[error("selector base must not be empty")]
    EmptyBase,
    #[error("selector base contains invalid characters: {0:?}")]
    InvalidBase(String),
    #[error("selector has more than 255 slots")]
    TooManySlots,
    #[error("positional selector slots must precede labeled slots")]
    PositionalAfterLabel,
    #[error("selector kind {kind:?} is incompatible with base {base:?}")]
    IncompatibleKind { base: SelectorBase, kind: SelectorKind },
    #[error("selector pattern kind {kind:?} is incompatible with base {base:?}")]
    IncompatiblePatternKind { base: SelectorBase, kind: SelectorKindPattern },
    #[error("selector pattern must contain exactly one gap")]
    MissingGap,
    #[error("selector pattern has an impossible slot ordering")]
    InvalidPatternSlots,
    #[error("invalid selector syntax: {0}")]
    InvalidSyntax(String),
}

impl Selector {
    pub fn getter(name: impl Into<String>) -> Result<Self, SelectorError> {
        Self::new(SelectorBase::Named(name.into()), SelectorKind::Getter, Box::new([]))
    }

    pub fn setter(name: impl Into<String>) -> Result<Self, SelectorError> {
        Self::new(SelectorBase::Named(name.into()), SelectorKind::Setter, Box::new([]))
    }

    pub fn method(name: impl Into<String>, slots: impl Into<Box<[SelectorSlot]>>) -> Result<Self, SelectorError> {
        Self::new(SelectorBase::Named(name.into()), SelectorKind::Method, slots.into())
    }

    pub fn subscript_get(slots: impl Into<Box<[SelectorSlot]>>) -> Result<Self, SelectorError> {
        Self::new(SelectorBase::Subscript, SelectorKind::SubscriptGet, slots.into())
    }

    pub fn subscript_set(slots: impl Into<Box<[SelectorSlot]>>) -> Result<Self, SelectorError> {
        Self::new(SelectorBase::Subscript, SelectorKind::SubscriptSet, slots.into())
    }

    pub fn new(base: SelectorBase, kind: SelectorKind, slots: Box<[SelectorSlot]>) -> Result<Self, SelectorError> {
        validate_slots(&slots)?;
        if slots.len() > u8::MAX as usize {
            return Err(SelectorError::TooManySlots);
        }

        match (&base, kind) {
            (SelectorBase::Named(name), SelectorKind::Getter | SelectorKind::Setter | SelectorKind::Method) => {
                validate_named_base(name)?;
            }
            (SelectorBase::Named(_), SelectorKind::SubscriptGet | SelectorKind::SubscriptSet)
            | (SelectorBase::Subscript, SelectorKind::Getter | SelectorKind::Setter | SelectorKind::Method) => {
                return Err(SelectorError::IncompatibleKind { base, kind });
            }
            (SelectorBase::Subscript, SelectorKind::SubscriptGet | SelectorKind::SubscriptSet) => {}
        }

        if matches!(kind, SelectorKind::Getter | SelectorKind::Setter) && !slots.is_empty() {
            return Err(SelectorError::InvalidSyntax(format!("{kind:?} selector cannot have slots")));
        }

        Ok(Self { base, kind, slots })
    }

    /// Returns the number of parameter slots in this selector.
    pub fn parameter_count(&self) -> usize {
        self.slots.len()
    }

    /// Returns canonical comma-form selector text.
    pub fn encode(&self) -> String {
        let slots = encode_slots(&self.slots);
        match (&self.base, self.kind) {
            (SelectorBase::Named(name), SelectorKind::Getter) => name.clone(),
            (SelectorBase::Named(name), SelectorKind::Setter) => format!("{name}=(put)"),
            (SelectorBase::Named(name), SelectorKind::Method) => format!("{name}({slots})"),
            (SelectorBase::Subscript, SelectorKind::SubscriptGet) => format!("[{slots}]"),
            (SelectorBase::Subscript, SelectorKind::SubscriptSet) => format!("[{slots}]=(put)"),
            // Public fields permit a caller to construct an invalid pair. Keep
            // encoding total for reflection and diagnostics rather than panic.
            (SelectorBase::Named(name), SelectorKind::SubscriptGet | SelectorKind::SubscriptSet) => format!("{name}({slots})"),
            (SelectorBase::Subscript, SelectorKind::Getter | SelectorKind::Setter | SelectorKind::Method) => format!("[{slots}]"),
        }
    }

    /// Strictly decodes one canonical exact selector.
    pub fn try_decode_exact(text: &str) -> Result<Self, SelectorError> {
        if text.is_empty() {
            return Err(SelectorError::EmptyBase);
        }

        if let Some(rest) = text.strip_prefix('[') {
            if let Some(inner) = rest.strip_suffix("]=(put)") {
                let slots = parse_slots_strict(inner)?;
                return Self::subscript_set(slots.into_boxed_slice());
            }
            if let Some(inner) = rest.strip_suffix(']') {
                let slots = parse_slots_strict(inner)?;
                return Self::subscript_get(slots.into_boxed_slice());
            }
            return Err(SelectorError::InvalidSyntax(text.to_string()));
        }

        let Some(open) = text.find('(') else {
            return Self::getter(text);
        };
        if !text.ends_with(')') || open == 0 {
            return Err(SelectorError::InvalidSyntax(text.to_string()));
        }

        let head = &text[..open];
        let inner = &text[open + 1..text.len() - 1];
        if inner == "put" {
            let name = head.strip_suffix('=').ok_or_else(|| SelectorError::InvalidSyntax(text.to_string()))?;
            validate_named_base(name)?;
            return Self::setter(name);
        }

        let name = head.strip_prefix("init ").unwrap_or(head);
        let slots = parse_slots_strict(inner)?;
        Self::method(name, slots.into_boxed_slice())
    }

    /// Total runtime-originated decode. Malformed text is represented as an
    /// opaque getter-shaped selector so reflection and dNU cannot panic.
    pub fn decode(text: &str) -> Self {
        Self::try_decode_exact(text)
            .or_else(|_| decode_runtime_form(text).ok_or_else(|| SelectorError::InvalidSyntax(text.to_string())))
            .unwrap_or_else(|_| Self {
                base: SelectorBase::Named(text.to_string()),
                kind: SelectorKind::Getter,
                slots: Box::new([]),
            })
    }
}

pub fn is_exact_selector_syntax(text: &str) -> bool {
    !text.contains("...") && Selector::try_decode_exact(text).is_ok()
}

pub fn is_selector_pattern_syntax(text: &str) -> bool {
    SelectorPattern::try_decode_pattern(text).is_ok()
}

impl SelectorPattern {
    /// Returns canonical source-like selector-pattern text for diagnostics.
    pub fn encode(&self) -> String {
        let slots = encode_pattern_slots(&self.prefix, &self.suffix, self.has_gap);
        match (&self.base, &self.kind) {
            (SelectorBase::Named(name), SelectorKindPattern::AnyNamed) => {
                if self.prefix.is_empty() && self.suffix.is_empty() && self.has_gap {
                    format!("{name}...")
                } else {
                    format!("{name}({slots})")
                }
            }
            (SelectorBase::Named(name), SelectorKindPattern::Exact(SelectorKind::Getter)) => {
                if self.has_gap {
                    format!("{name}...")
                } else {
                    name.clone()
                }
            }
            (SelectorBase::Named(name), SelectorKindPattern::Exact(SelectorKind::Setter)) => {
                if self.has_gap {
                    format!("{name}=...")
                } else {
                    format!("{name}=(put)")
                }
            }
            (SelectorBase::Named(name), SelectorKindPattern::Exact(SelectorKind::Method)) => format!("{name}({slots})"),
            (SelectorBase::Subscript, SelectorKindPattern::Exact(SelectorKind::SubscriptGet)) => format!("[{slots}]"),
            (SelectorBase::Subscript, SelectorKindPattern::Exact(SelectorKind::SubscriptSet)) => format!("[{slots}]=(put)"),
            (SelectorBase::Named(name), SelectorKindPattern::Exact(SelectorKind::SubscriptGet | SelectorKind::SubscriptSet)) => {
                format!("{name}({slots})")
            }
            (
                SelectorBase::Subscript,
                SelectorKindPattern::AnyNamed | SelectorKindPattern::Exact(SelectorKind::Getter | SelectorKind::Setter | SelectorKind::Method),
            ) => {
                format!("[{slots}]")
            }
        }
    }

    pub fn new(
        base: SelectorBase,
        kind: SelectorKindPattern,
        prefix: impl Into<Box<[SelectorSlot]>>,
        suffix: impl Into<Box<[SelectorSlot]>>,
        has_gap: bool,
    ) -> Result<Self, SelectorError> {
        let prefix = prefix.into();
        let suffix = suffix.into();
        if !has_gap {
            return Err(SelectorError::MissingGap);
        }
        validate_slots(&prefix)?;
        validate_slots(&suffix)?;
        if prefix.iter().any(|slot| matches!(slot, SelectorSlot::Label(_))) && suffix.iter().any(|slot| matches!(slot, SelectorSlot::Positional)) {
            return Err(SelectorError::InvalidPatternSlots);
        }

        match (&base, &kind) {
            (
                SelectorBase::Named(name),
                SelectorKindPattern::AnyNamed | SelectorKindPattern::Exact(SelectorKind::Getter | SelectorKind::Setter | SelectorKind::Method),
            ) => {
                validate_named_base(name)?;
            }
            (SelectorBase::Subscript, SelectorKindPattern::Exact(SelectorKind::SubscriptGet | SelectorKind::SubscriptSet)) => {}
            _ => return Err(SelectorError::IncompatiblePatternKind { base, kind }),
        }

        if matches!(kind, SelectorKindPattern::Exact(SelectorKind::Getter | SelectorKind::Setter)) && (!prefix.is_empty() || !suffix.is_empty()) {
            return Err(SelectorError::InvalidPatternSlots);
        }

        Ok(Self {
            base,
            kind,
            prefix,
            suffix,
            has_gap,
        })
    }

    pub fn named(
        name: impl Into<String>,
        kind: SelectorKindPattern,
        prefix: impl Into<Box<[SelectorSlot]>>,
        suffix: impl Into<Box<[SelectorSlot]>>,
        has_gap: bool,
    ) -> Result<Self, SelectorError> {
        Self::new(SelectorBase::Named(name.into()), kind, prefix, suffix, has_gap)
    }

    pub fn named_method(
        name: impl Into<String>,
        prefix: impl Into<Box<[SelectorSlot]>>,
        suffix: impl Into<Box<[SelectorSlot]>>,
        has_gap: bool,
    ) -> Result<Self, SelectorError> {
        Self::named(name, SelectorKindPattern::Exact(SelectorKind::Method), prefix, suffix, has_gap)
    }

    pub fn matches(&self, selector: &Selector) -> bool {
        if self.base != selector.base {
            return false;
        }
        match self.kind {
            SelectorKindPattern::AnyNamed => {
                if !matches!(selector.kind, SelectorKind::Getter | SelectorKind::Setter | SelectorKind::Method) {
                    return false;
                }
            }
            SelectorKindPattern::Exact(kind) if kind != selector.kind => return false,
            SelectorKindPattern::Exact(_) => {}
        }

        let min_len = self.prefix.len() + self.suffix.len();
        if (self.has_gap && selector.slots.len() < min_len) || (!self.has_gap && selector.slots.len() != min_len) {
            return false;
        }
        if !selector.slots.starts_with(&self.prefix) {
            return false;
        }
        selector.slots.ends_with(&self.suffix)
    }

    /// Strictly decodes canonical selector-pattern text.
    pub fn try_decode_pattern(text: &str) -> Result<Self, SelectorError> {
        if text.is_empty() {
            return Err(SelectorError::EmptyBase);
        }

        if let Some(rest) = text.strip_prefix('[') {
            if let Some(inner) = rest.strip_suffix("]=(put)") {
                let (prefix, suffix) = parse_pattern_slots_strict(inner)?;
                return Self::new(
                    SelectorBase::Subscript,
                    SelectorKindPattern::Exact(SelectorKind::SubscriptSet),
                    prefix.into_boxed_slice(),
                    suffix.into_boxed_slice(),
                    true,
                );
            }
            if let Some(inner) = rest.strip_suffix(']') {
                let (prefix, suffix) = parse_pattern_slots_strict(inner)?;
                return Self::new(
                    SelectorBase::Subscript,
                    SelectorKindPattern::Exact(SelectorKind::SubscriptGet),
                    prefix.into_boxed_slice(),
                    suffix.into_boxed_slice(),
                    true,
                );
            }
            return Err(SelectorError::InvalidSyntax(text.to_string()));
        }

        if let Some(base) = text.strip_suffix("=...") {
            validate_named_base(base)?;
            return Self::new(
                SelectorBase::Named(base.to_string()),
                SelectorKindPattern::Exact(SelectorKind::Setter),
                Vec::<SelectorSlot>::new(),
                Vec::<SelectorSlot>::new(),
                true,
            );
        }

        if let Some(base) = text.strip_suffix("...") {
            if !base.is_empty() && !base.contains('(') && !base.contains(')') {
                validate_named_base(base)?;
                return Self::new(
                    SelectorBase::Named(base.to_string()),
                    SelectorKindPattern::AnyNamed,
                    Vec::<SelectorSlot>::new(),
                    Vec::<SelectorSlot>::new(),
                    true,
                );
            }
        }

        let Some(open) = text.find('(') else {
            return Err(SelectorError::MissingGap);
        };
        if !text.ends_with(')') || open == 0 {
            return Err(SelectorError::InvalidSyntax(text.to_string()));
        }

        let head = &text[..open];
        let inner = &text[open + 1..text.len() - 1];

        let name = head.strip_prefix("init ").unwrap_or(head);
        validate_named_base(name)?;
        let (prefix, suffix) = parse_pattern_slots_strict(inner)?;
        Self::new(
            SelectorBase::Named(name.to_string()),
            SelectorKindPattern::Exact(SelectorKind::Method),
            prefix.into_boxed_slice(),
            suffix.into_boxed_slice(),
            true,
        )
    }

    /// Total decode for selector-pattern values.
    pub fn decode(text: &str) -> Self {
        Self::try_decode_pattern(text).unwrap_or_else(|_| Self {
            base: SelectorBase::Named(text.to_string()),
            kind: SelectorKindPattern::AnyNamed,
            prefix: Box::new([]),
            suffix: Box::new([]),
            has_gap: true,
        })
    }
}

impl fmt::Display for SelectorPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.encode())
    }
}

fn encode_pattern_slots(prefix: &[SelectorSlot], suffix: &[SelectorSlot], has_gap: bool) -> String {
    let mut slots = prefix.iter().chain(suffix.iter()).map(encode_slot).collect::<Vec<_>>();
    if has_gap {
        let gap_at = prefix.len();
        slots.insert(gap_at, "...".to_string());
    }
    slots.join(", ")
}

fn encode_slot(slot: &SelectorSlot) -> String {
    match slot {
        SelectorSlot::Positional => "_".to_string(),
        SelectorSlot::Label(label) => encode_label_component(label),
    }
}

fn validate_named_base(name: &str) -> Result<(), SelectorError> {
    if name.is_empty() {
        return Err(SelectorError::EmptyBase);
    }
    if name.contains(['(', ')', '[', ']', ',']) {
        return Err(SelectorError::InvalidBase(name.to_string()));
    }
    Ok(())
}

fn validate_slots(slots: &[SelectorSlot]) -> Result<(), SelectorError> {
    let mut saw_label = false;
    for slot in slots {
        match slot {
            SelectorSlot::Positional if saw_label => return Err(SelectorError::PositionalAfterLabel),
            SelectorSlot::Positional => {}
            SelectorSlot::Label(label) => {
                if label.is_empty() {
                    return Err(SelectorError::InvalidSyntax("empty selector label".into()));
                }
                saw_label = true;
            }
        }
    }
    Ok(())
}

fn encode_slots(slots: &[SelectorSlot]) -> String {
    slots
        .iter()
        .map(|slot| match slot {
            SelectorSlot::Positional => "_".to_string(),
            SelectorSlot::Label(label) => encode_label_component(label),
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn parse_slot_strict(part: &str) -> Result<SelectorSlot, SelectorError> {
    let part = part.trim();
    if part.is_empty() {
        return Err(SelectorError::InvalidSyntax("empty selector slot".into()));
    }
    if matches!(part, "_" | "*" | "**" | "***") {
        return Ok(SelectorSlot::Positional);
    }
    Ok(SelectorSlot::Label(decode_label_component_strict(part)?))
}

fn parse_slots_strict(inner: &str) -> Result<Vec<SelectorSlot>, SelectorError> {
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }
    inner.split(',').map(parse_slot_strict).collect::<Result<Vec<_>, _>>().and_then(|slots| {
        validate_slots(&slots)?;
        Ok(slots)
    })
}

fn parse_pattern_slots_strict(inner: &str) -> Result<(Vec<SelectorSlot>, Vec<SelectorSlot>), SelectorError> {
    let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
    if parts.is_empty() || (parts.len() == 1 && parts[0].is_empty()) {
        return Err(SelectorError::MissingGap);
    }
    let gap_indices: Vec<usize> = parts
        .iter()
        .enumerate()
        .filter_map(|(idx, &part)| if part == "..." { Some(idx) } else { None })
        .collect();

    if gap_indices.is_empty() {
        return Err(SelectorError::MissingGap);
    }
    if gap_indices.len() > 1 {
        return Err(SelectorError::InvalidPatternSlots);
    }

    let gap_idx = gap_indices[0];
    let prefix_parts = &parts[..gap_idx];
    let suffix_parts = &parts[gap_idx + 1..];

    let mut prefix = Vec::with_capacity(prefix_parts.len());
    for &part in prefix_parts {
        if part.is_empty() {
            return Err(SelectorError::InvalidSyntax(inner.to_string()));
        }
        prefix.push(parse_slot_strict(part)?);
    }

    let mut suffix = Vec::with_capacity(suffix_parts.len());
    for &part in suffix_parts {
        if part.is_empty() {
            return Err(SelectorError::InvalidSyntax(inner.to_string()));
        }
        suffix.push(parse_slot_strict(part)?);
    }

    validate_slots(&prefix)?;
    validate_slots(&suffix)?;

    Ok((prefix, suffix))
}

/// Decodes the VM's total selector transport, including legacy rest-family
/// markers such as `name(fixed,**)`. These are not exact source selectors and
/// therefore intentionally bypass strict positional-before-label validation.
fn decode_runtime_form(text: &str) -> Option<Selector> {
    if let Some(rest) = text.strip_prefix('[') {
        if let Some(inner) = rest.strip_suffix("]=(put)") {
            return Some(Selector {
                base: SelectorBase::Subscript,
                kind: SelectorKind::SubscriptSet,
                slots: parse_slots_runtime(inner),
            });
        }
        if let Some(inner) = rest.strip_suffix(']') {
            return Some(Selector {
                base: SelectorBase::Subscript,
                kind: SelectorKind::SubscriptGet,
                slots: parse_slots_runtime(inner),
            });
        }
        return None;
    }

    let open = text.find('(')?;
    if !text.ends_with(')') || open == 0 {
        return None;
    }
    let head = &text[..open];
    let inner = &text[open + 1..text.len() - 1];
    if inner == "put" {
        let name = head.strip_suffix('=')?;
        if validate_named_base(name).is_ok() {
            return Some(Selector {
                base: SelectorBase::Named(name.to_string()),
                kind: SelectorKind::Setter,
                slots: Box::new([]),
            });
        }
    }
    let name = head.strip_prefix("init ").unwrap_or(head);
    if name.is_empty() {
        return None;
    }
    Some(Selector {
        base: SelectorBase::Named(name.to_string()),
        kind: SelectorKind::Method,
        slots: parse_slots_runtime(inner),
    })
}

fn parse_slots_runtime(inner: &str) -> Box<[SelectorSlot]> {
    if inner.is_empty() {
        return Box::new([]);
    }
    inner
        .split(',')
        .map(|part| match part {
            "_" | "*" | "**" | "***" => SelectorSlot::Positional,
            _ => SelectorSlot::Label(decode_label_component(part)),
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

pub fn encode_label_component(symbol_text: &str) -> String {
    let reserved = matches!(symbol_text, "_" | "*" | "**" | "***");
    let safe = !symbol_text.is_empty()
        && !symbol_text.starts_with('~')
        && !reserved
        && symbol_text.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b'_' | b'?' | b'!' | b'+' | b'-' | b'*' | b'/' | b'<' | b'>' | b'=' | b'&' | b'|' | b'^' | b'~' | b'%'
                )
        });
    if safe {
        return symbol_text.to_string();
    }
    let mut encoded = String::with_capacity(1 + symbol_text.len() * 2);
    encoded.push('~');
    for byte in symbol_text.bytes() {
        use std::fmt::Write;
        let _ = write!(&mut encoded, "{byte:02x}");
    }
    encoded
}

pub fn decode_label_component(component: &str) -> String {
    decode_label_component_strict(component).unwrap_or_else(|_| component.to_string())
}

fn decode_label_component_strict(component: &str) -> Result<String, SelectorError> {
    let Some(hex) = component.strip_prefix('~') else {
        return Ok(component.to_string());
    };
    if hex.is_empty() || hex.len() % 2 != 0 {
        return Err(SelectorError::InvalidSyntax(component.to_string()));
    }
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for pair in hex.as_bytes().chunks_exact(2) {
        let high = (pair[0] as char)
            .to_digit(16)
            .ok_or_else(|| SelectorError::InvalidSyntax(component.to_string()))?;
        let low = (pair[1] as char)
            .to_digit(16)
            .ok_or_else(|| SelectorError::InvalidSyntax(component.to_string()))?;
        bytes.push((high * 16 + low) as u8);
    }
    String::from_utf8(bytes).map_err(|_| SelectorError::InvalidSyntax(component.to_string()))
}

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_label_escape_is_total() {
        for raw in ["~", "~f", "~zz", "~ff"] {
            assert_eq!(decode_label_component(raw), raw);
        }
    }

    #[test]
    fn pattern_decode_round_trip() {
        let patterns = [
            "foo(...)",
            "foo(_, ..., duration)",
            "foo(..., duration)",
            "foo...",
            "foo=...",
            "[...]",
            "[...]=(put)",
            "[_, ...]",
            "[_, ...]=(put)",
            "+...",
            "+(...)",
        ];
        for text in patterns {
            let pat = SelectorPattern::try_decode_pattern(text).unwrap_or_else(|e| panic!("failed to decode {text}: {e}"));
            assert!(is_selector_pattern_syntax(text));
            assert!(!is_exact_selector_syntax(text));
            assert_eq!(pat.encode(), text);
        }
    }
}
~~~~


