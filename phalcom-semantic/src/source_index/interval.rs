//! Bounded deterministic source-range interval lookup.

use phalcom_common::range::SourceRange;

/// One range-index entry with deterministic tie-breaking priority.
#[derive(Clone, Copy, Debug)]
pub struct RangeEntry<T: Copy> {
    pub range: SourceRange,
    pub value: T,
    pub priority: u8,
    ordinal: usize,
}

impl<T: Copy> RangeEntry<T> {
    /// Creates an entry. Lower priority values win equal-length matches.
    pub fn new(range: SourceRange, value: T, priority: u8) -> Self {
        Self {
            range,
            value,
            priority,
            ordinal: 0,
        }
    }
}

/// Immutable range index using sorted starts and prefix maximum ends.
///
/// A point query first binary-searches by start and then walks backwards only
/// while the prefix maximum proves that an earlier interval can still contain
/// the point. It therefore avoids a whole-module linear scan in the common
/// case while retaining deterministic nested-interval selection.
#[derive(Clone, Debug, Default)]
pub struct RangeIndex<T: Copy> {
    entries: Vec<RangeEntry<T>>,
    max_end_prefix: Vec<usize>,
}

impl<T: Copy> RangeIndex<T> {
    /// Builds an index with stable source-order and priority ordering.
    pub fn new(entries: impl IntoIterator<Item = RangeEntry<T>>) -> Self {
        let mut entries = entries
            .into_iter()
            .enumerate()
            .map(|(ordinal, mut entry)| {
                entry.ordinal = ordinal;
                entry
            })
            .collect::<Vec<_>>();
        entries.sort_by_key(|entry| (entry.range.start, entry.range.len(), entry.priority, entry.ordinal));
        let mut max_end_prefix = Vec::with_capacity(entries.len());
        let mut max_end = 0;
        for entry in &entries {
            max_end = max_end.max(entry.range.end);
            max_end_prefix.push(max_end);
        }
        Self { entries, max_end_prefix }
    }

    /// Number of indexed ranges.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether no ranges are indexed.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the selected entry index for `offset`.
    pub fn index_at(&self, offset: usize) -> Option<usize> {
        let mut low = 0;
        let mut high = self.entries.len();
        while low < high {
            let middle = (low + high) / 2;
            if self.entries[middle].range.start <= offset {
                low = middle + 1;
            } else {
                high = middle;
            }
        }

        let mut best: Option<usize> = None;
        let mut index = low;
        while index > 0 {
            index -= 1;
            if self.max_end_prefix[index] <= offset {
                break;
            }
            let entry = &self.entries[index];
            if !entry.range.contains(offset) {
                continue;
            }
            let candidate_key = (entry.range.len(), entry.priority, entry.range.start, entry.ordinal);
            let replaces = best.is_none_or(|best_index| {
                let best_entry = &self.entries[best_index];
                candidate_key < (best_entry.range.len(), best_entry.priority, best_entry.range.start, best_entry.ordinal)
            });
            if replaces {
                best = Some(index);
            }
        }
        best
    }

    /// Returns selected value for `offset`.
    pub fn value_at(&self, offset: usize) -> Option<T> {
        self.index_at(offset).map(|index| self.entries[index].value)
    }
}
