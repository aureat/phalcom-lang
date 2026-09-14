//! Shared VM-owned anonymous product shape metadata.
//!
//! Realizes LANG005.C1.P3 Task 5: Shape metadata for Tuple and Record products
//! is interned once per unique coordinate/presentation structure and shared
//! across all instances rather than copied per value.

use crate::interner::Symbol;
use std::cmp::Ordering;
use std::collections::HashMap;

/// Identifies an interned concrete product shape in [`ProductShapeRegistry`].
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProductShapeId(pub u32);

/// Discriminates Tuple vs Record anonymous product kinds.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AnonymousProductKind {
    Tuple,
    Record,
}

/// Coordinate shape for a Tuple product.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TupleProductShape {
    pub positional_len: u32,
    pub labels: Box<[Symbol]>,
}

impl TupleProductShape {
    pub fn new(positional_len: u32, labels: Box<[Symbol]>) -> Self {
        // Assert label uniqueness
        for (i, label) in labels.iter().enumerate() {
            assert!(!labels[..i].contains(label), "Tuple shape labels must be unique");
        }
        Self {
            positional_len,
            labels,
        }
    }

    pub fn total_len(&self) -> u32 {
        self.positional_len + self.labels.len() as u32
    }
}

/// Coordinate and presentation shape for a Record product.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RecordProductShape {
    /// Observable encounter/presentation order.
    pub presentation_labels: Box<[Symbol]>,
    /// Canonical logical/storage coordinate labels.
    pub logical_labels: Box<[Symbol]>,
    /// presentation index -> logical/storage index.
    pub presentation_to_logical: Box<[u32]>,
}

impl RecordProductShape {
    pub fn new(
        presentation_labels: Box<[Symbol]>,
        logical_labels: Box<[Symbol]>,
        presentation_to_logical: Box<[u32]>,
    ) -> Self {
        assert_eq!(
            presentation_labels.len(),
            logical_labels.len(),
            "presentation and logical label count must match"
        );
        assert_eq!(
            presentation_labels.len(),
            presentation_to_logical.len(),
            "presentation label and mapping count must match"
        );
        for (i, label) in presentation_labels.iter().enumerate() {
            assert!(
                !presentation_labels[..i].contains(label),
                "presentation labels must be unique"
            );
        }
        for (i, label) in logical_labels.iter().enumerate() {
            assert!(!logical_labels[..i].contains(label), "logical labels must be unique");
        }
        let mut mapping_seen = vec![false; presentation_to_logical.len()];
        for logical_index in presentation_to_logical.iter().copied() {
            let index = usize::try_from(logical_index).expect("record shape logical index must fit usize");
            assert!(index < mapping_seen.len(), "record shape logical index out of bounds");
            assert!(!mapping_seen[index], "record shape presentation mapping must be a permutation");
            mapping_seen[index] = true;
        }
        assert!(mapping_seen.iter().all(|seen| *seen), "record shape presentation mapping must be complete");
        Self {
            presentation_labels,
            logical_labels,
            presentation_to_logical,
        }
    }

    /// Helper to construct a shape where presentation order is identical to logical order.
    pub fn from_ordered_labels(labels: Box<[Symbol]>) -> Self {
        let len = labels.len() as u32;
        let mapping: Box<[u32]> = (0..len).collect();
        Self::new(labels.clone(), labels, mapping)
    }

    /// Constructs canonical logical/storage coordinates from presentation labels.
    pub fn from_presentation_labels<F>(presentation_labels: Box<[Symbol]>, mut compare: F) -> Self
    where
        F: FnMut(Symbol, Symbol) -> Ordering,
    {
        let mut logical_with_source = presentation_labels
            .iter()
            .copied()
            .enumerate()
            .collect::<Vec<_>>();
        logical_with_source.sort_by(|(_, left), (_, right)| compare(*left, *right));
        let mut logical_labels = Vec::with_capacity(logical_with_source.len());
        let mut presentation_to_logical = vec![0; presentation_labels.len()];
        for (logical_index, (source_index, label)) in logical_with_source.into_iter().enumerate() {
            logical_labels.push(label);
            presentation_to_logical[source_index] = logical_index as u32;
        }
        Self::new(
            presentation_labels,
            logical_labels.into_boxed_slice(),
            presentation_to_logical.into_boxed_slice(),
        )
    }

    pub fn len(&self) -> usize {
        self.presentation_labels.len()
    }

    pub fn is_empty(&self) -> bool {
        self.presentation_labels.is_empty()
    }
}

/// Unified product shape enumeration.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ProductShape {
    Tuple(TupleProductShape),
    Record(RecordProductShape),
}

/// VM-level registry managing interned product shapes.
#[derive(Clone, Debug, Default)]
pub struct ProductShapeRegistry {
    shapes: Vec<ProductShape>,
    shape_by_key: HashMap<ProductShape, ProductShapeId>,
    /// Non-key label-to-logical lookup cache for record shapes.
    record_lookup_cache: Vec<HashMap<Symbol, u32>>,
}

impl ProductShapeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Interns a product shape.
    pub fn register(&mut self, shape: ProductShape) -> ProductShapeId {
        if let Some(&existing) = self.shape_by_key.get(&shape) {
            return existing;
        }

        let id = ProductShapeId(self.shapes.len() as u32);
        let mut lookup_map = HashMap::new();
        if let ProductShape::Record(rec) = &shape {
            for (logical_idx, label) in rec.logical_labels.iter().enumerate() {
                lookup_map.insert(*label, logical_idx as u32);
            }
        }
        self.record_lookup_cache.push(lookup_map);
        self.shapes.push(shape.clone());
        self.shape_by_key.insert(shape, id);
        id
    }

    pub fn get(&self, id: ProductShapeId) -> Option<&ProductShape> {
        self.shapes.get(id.0 as usize)
    }

    pub fn get_tuple(&self, id: ProductShapeId) -> Option<&TupleProductShape> {
        match self.get(id) {
            Some(ProductShape::Tuple(t)) => Some(t),
            _ => None,
        }
    }

    pub fn get_record(&self, id: ProductShapeId) -> Option<&RecordProductShape> {
        match self.get(id) {
            Some(ProductShape::Record(r)) => Some(r),
            _ => None,
        }
    }

    /// Fast logical index lookup by label for a registered Record shape.
    pub fn lookup_record_logical_index(&self, id: ProductShapeId, label: Symbol) -> Option<u32> {
        self.record_lookup_cache
            .get(id.0 as usize)
            .and_then(|map| map.get(&label).copied())
    }
}
