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
