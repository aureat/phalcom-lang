//! Shared VM-owned anonymous product shape metadata.
//!
//! Realizes LANG005.C1.P3 Task 5: Shape metadata for Tuple and Record products
//! is interned once per unique coordinate/presentation structure and shared
//! across all instances rather than copied per value.

use crate::interner::Symbol;
use phalcom_semantic::field_key::FieldKey;
use std::cmp::Ordering;
use std::collections::HashMap;

/// Identifies an interned concrete product shape in [`ProductShapeRegistry`].
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProductShapeId(pub u32);

/// Discriminates Tuple vs Record vs Pack anonymous product kinds.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AnonymousProductKind {
    Tuple,
    Record,
    Pack,
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
        Self { positional_len, labels }
    }

    pub fn total_len(&self) -> u32 {
        self.positional_len + self.labels.len() as u32
    }
}

/// Coordinate shape for a Pack product.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PackProductShape {
    pub positional_len: u32,
    pub labels: Box<[Symbol]>,
}

impl PackProductShape {
    pub fn new(positional_len: u32, labels: Box<[Symbol]>) -> Self {
        // Assert label uniqueness
        for (i, label) in labels.iter().enumerate() {
            assert!(!labels[..i].contains(label), "Pack shape labels must be unique");
        }
        Self { positional_len, labels }
    }

    pub fn total_len(&self) -> u32 {
        self.positional_len + self.labels.len() as u32
    }
}

/// Coordinate and presentation shape for a Record product.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RecordProductShape {
    /// Canonical logical/storage coordinate labels.
    pub logical_keys: Box<[FieldKey]>,
    /// presentation index -> logical/storage index.
    pub presentation_to_logical: Box<[u32]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordProductShapeError {
    DuplicateKey(FieldKey),
    InvalidCanonicalKeys,
    InvalidPermutation,
}

impl RecordProductShape {
    /// Builds a shape from canonical keys and a checked presentation permutation.
    pub fn new(logical_keys: Box<[FieldKey]>, presentation_to_logical: Box<[u32]>) -> Result<Self, RecordProductShapeError> {
        if logical_keys.len() != presentation_to_logical.len() {
            return Err(RecordProductShapeError::InvalidPermutation);
        }
        if logical_keys.windows(2).any(|pair| pair[0].canonical_cmp(&pair[1]) != Ordering::Less) {
            return Err(RecordProductShapeError::InvalidCanonicalKeys);
        }
        let mut mapping_seen = vec![false; presentation_to_logical.len()];
        for logical_index in presentation_to_logical.iter().copied() {
            let index = usize::try_from(logical_index).map_err(|_| RecordProductShapeError::InvalidPermutation)?;
            if index >= mapping_seen.len() || mapping_seen[index] {
                return Err(RecordProductShapeError::InvalidPermutation);
            }
            mapping_seen[index] = true;
        }
        if mapping_seen.iter().any(|seen| !seen) {
            return Err(RecordProductShapeError::InvalidPermutation);
        }
        Ok(Self {
            logical_keys,
            presentation_to_logical,
        })
    }

    /// Constructs canonical logical/storage coordinates from source-order keys.
    pub fn from_presentation_keys(presentation_keys: Vec<FieldKey>) -> Result<Self, RecordProductShapeError> {
        let mut logical_with_source = presentation_keys
            .into_iter()
            .enumerate()
            .map(|(source_index, key)| (key, source_index))
            .collect::<Vec<_>>();
        logical_with_source.sort_by(|(left, _), (right, _)| left.canonical_cmp(right));
        for pair in logical_with_source.windows(2) {
            if pair[0].0.canonical_cmp(&pair[1].0) == Ordering::Equal {
                return Err(RecordProductShapeError::DuplicateKey(pair[0].0.clone()));
            }
        }
        let mut logical_keys = Vec::with_capacity(logical_with_source.len());
        let mut presentation_to_logical = vec![0; logical_with_source.len()];
        for (logical_index, (key, source_index)) in logical_with_source.into_iter().enumerate() {
            logical_keys.push(key);
            presentation_to_logical[source_index] = u32::try_from(logical_index).map_err(|_| RecordProductShapeError::InvalidPermutation)?;
        }
        Self::new(logical_keys.into_boxed_slice(), presentation_to_logical.into_boxed_slice())
    }

    pub fn len(&self) -> usize {
        self.logical_keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.logical_keys.is_empty()
    }

    pub fn presentation_key(&self, index: usize) -> Option<&FieldKey> {
        let logical = *self.presentation_to_logical.get(index)? as usize;
        self.logical_keys.get(logical)
    }

    pub fn lookup_logical_index_by<F>(&self, mut compare: F) -> Option<u32>
    where
        F: FnMut(&FieldKey) -> Ordering,
    {
        self.logical_keys
            .binary_search_by(|key| compare(key))
            .ok()
            .and_then(|index| u32::try_from(index).ok())
    }
}

/// Unified product shape enumeration.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ProductShape {
    Tuple(TupleProductShape),
    Record(RecordProductShape),
    Pack(PackProductShape),
}

/// VM-level registry managing interned product shapes.
#[derive(Clone, Debug, Default)]
pub struct ProductShapeRegistry {
    shapes: Vec<ProductShape>,
    shape_by_key: HashMap<ProductShape, ProductShapeId>,
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

    pub fn get_pack(&self, id: ProductShapeId) -> Option<&PackProductShape> {
        match self.get(id) {
            Some(ProductShape::Pack(p)) => Some(p),
            _ => None,
        }
    }

    /// Binary-searches the canonical key vector of a registered Record shape.
    pub fn lookup_record_logical_index(&self, id: ProductShapeId, key: &FieldKey) -> Option<u32> {
        self.get_record(id)?.lookup_logical_index_by(|candidate| candidate.canonical_cmp(key))
    }
}
