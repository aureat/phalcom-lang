//! Registry-aware read views for materialized anonymous products.

use super::{ProductShape, ProductShapeId};
use crate::heap::ObjRef;
use crate::interner::Symbol;
use crate::value::Value;
use crate::vm::VM;
use phalcom_semantic::field_key::FieldKey;

/// Read-only view over a materialized Tuple and its shared metadata.
pub struct TupleView<'a> {
    object: Option<&'a crate::heap::TupleObject>,
    shape: Option<&'a super::TupleProductShape>,
    layout: Option<&'a super::ProductLayout>,
}

impl<'a> TupleView<'a> {
    fn empty() -> Self {
        Self {
            object: None,
            shape: None,
            layout: None,
        }
    }

    fn load(&self, index: usize) -> Option<Value> {
        self.object?.storage().load_component(self.layout?, index as u32).ok()
    }

    pub fn len(&self) -> usize {
        self.shape.map_or(0, |shape| shape.total_len() as usize)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn positional_len(&self) -> usize {
        self.shape.map_or(0, |shape| shape.positional_len as usize)
    }

    pub fn labeled_len(&self) -> usize {
        self.shape.map_or(0, |shape| shape.labels.len())
    }

    pub fn get(&self, index: usize) -> Option<Value> {
        (index < self.len()).then(|| self.load(index)).flatten()
    }

    pub fn values(&self) -> Vec<Value> {
        (0..self.len()).filter_map(|index| self.get(index)).collect()
    }

    pub fn positionals(&self) -> Vec<Value> {
        (0..self.positional_len()).filter_map(|index| self.get(index)).collect()
    }

    pub fn labeled_values(&self) -> Vec<Value> {
        (self.positional_len()..self.len()).filter_map(|index| self.get(index)).collect()
    }

    pub fn labels(&self) -> &[Symbol] {
        self.shape.map_or(&[], |shape| shape.labels.as_ref())
    }

    pub fn get_label(&self, label: Symbol) -> Option<Value> {
        self.shape?
            .labels
            .iter()
            .position(|candidate| *candidate == label)
            .and_then(|index| self.get(self.positional_len() + index))
    }

    pub fn labeled_entries(&self) -> Vec<(Symbol, Value)> {
        self.labels().iter().copied().zip(self.labeled_values()).collect()
    }
}

/// Read-only view over a materialized Record and its shared metadata.
pub struct RecordView<'a> {
    object: Option<&'a crate::heap::RecordObject>,
    shape: Option<&'a super::RecordProductShape>,
    layout: Option<&'a super::ProductLayout>,
}

impl<'a> RecordView<'a> {
    fn empty() -> Self {
        Self {
            object: None,
            shape: None,
            layout: None,
        }
    }

    pub fn len(&self) -> usize {
        self.shape.map_or(0, super::RecordProductShape::len)
    }

    pub fn is_empty(&self) -> bool {
        self.shape.map_or(true, super::RecordProductShape::is_empty)
    }

    pub fn logical_keys(&self) -> &[FieldKey] {
        self.shape.map_or(&[], |shape| shape.logical_keys.as_ref())
    }

    pub fn presentation_key(&self, index: usize) -> Option<&FieldKey> {
        self.shape?.presentation_key(index)
    }

    pub fn logical_value_at(&self, index: usize) -> Option<Value> {
        let (Some(object), Some(shape), Some(layout)) = (self.object, self.shape, self.layout) else {
            return None;
        };
        (index < shape.logical_keys.len())
            .then(|| object.storage().load_component(layout, index as u32).ok())
            .flatten()
    }

    pub fn values(&self) -> Vec<Value> {
        (0..self.len())
            .filter_map(|index| {
                let logical = *self.shape?.presentation_to_logical.get(index)? as usize;
                self.logical_value_at(logical)
            })
            .collect()
    }

    pub fn get(&self, key: &FieldKey) -> Option<Value> {
        let shape = self.shape?;
        let logical = shape.lookup_logical_index_by(|candidate| candidate.canonical_cmp(key))?;
        self.logical_value_at(logical as usize)
    }

    pub fn get_by_value(&self, vm: &VM, key: Value) -> Result<Option<Value>, crate::error::RuntimeError> {
        crate::product::validate_field_key_value(vm, key)?;
        let Some(shape) = self.shape else {
            return Ok(None);
        };
        let mut low = 0;
        let mut high = shape.logical_keys.len();
        while low < high {
            let mid = low + (high - low) / 2;
            match crate::product::compare_field_key_to_value(vm, &shape.logical_keys[mid], key)? {
                std::cmp::Ordering::Less => low = mid + 1,
                std::cmp::Ordering::Greater => high = mid,
                std::cmp::Ordering::Equal => return Ok(self.logical_value_at(mid)),
            }
        }
        Ok(None)
    }

    pub fn entries(&self) -> Vec<(FieldKey, Value)> {
        let values = self.values();
        (0..self.len())
            .filter_map(|index| Some((self.presentation_key(index)?.clone(), values.get(index).copied()?)))
            .collect()
    }
}

fn product_metadata(vm: &VM, descriptor_id: super::RuntimeAnonymousProductDescriptorId) -> Option<(ProductShapeId, &super::ProductLayout)> {
    let descriptor = vm.anonymous_product_descriptors.get(descriptor_id)?;
    let layout = vm.heap.product_layouts.get(descriptor.layout)?;
    Some((descriptor.shape, layout))
}

impl VM {
    pub fn tuple_view(&self, id: ObjRef) -> Option<TupleView<'_>> {
        let object = self.heap.as_tuple(id)?;
        let (shape_id, layout) = product_metadata(self, object.descriptor())?;
        let Some(ProductShape::Tuple(shape)) = self.product_shapes.get(shape_id) else {
            return None;
        };
        Some(TupleView {
            object: Some(object),
            shape: Some(shape),
            layout: Some(layout),
        })
    }

    /// Canonical value-level Tuple view. The immediate Unit is the zero-length
    /// Tuple and is represented directly without allocating a heap object.
    pub fn tuple_view_value(&self, value: Value) -> Option<TupleView<'_>> {
        if value.is_unit() {
            return Some(TupleView::empty());
        }
        self.tuple_view(value.as_obj()?)
    }

    pub fn record_view(&self, id: ObjRef) -> Option<RecordView<'_>> {
        let object = self.heap.as_record(id)?;
        let (shape_id, layout) = product_metadata(self, object.descriptor())?;
        let Some(ProductShape::Record(shape)) = self.product_shapes.get(shape_id) else {
            return None;
        };
        Some(RecordView {
            object: Some(object),
            shape: Some(shape),
            layout: Some(layout),
        })
    }

    /// Canonical value-level Record view. EmptyRecord is the zero-field Record
    /// and is represented directly without allocating a heap object.
    pub fn record_view_value(&self, value: Value) -> Option<RecordView<'_>> {
        if value.is_empty_record() {
            return Some(RecordView::empty());
        }
        self.record_view(value.as_obj()?)
    }

    pub fn pack_view(&self, id: ObjRef) -> Option<PackView<'_>> {
        let object = self.heap.as_pack(id)?;
        let (root_id, storage, descriptor_id, pos_start, pos_len, lab_start, lab_len) = match object.storage() {
            crate::heap::pack::PackStorage::Root { descriptor, storage } => (
                id,
                storage,
                *descriptor,
                object.positional_start(),
                object.positional_len(),
                object.labeled_start(),
                object.labeled_len(),
            ),
            crate::heap::pack::PackStorage::Slice { root } => {
                let root_obj = self.heap.as_pack(*root)?;
                let (descriptor, storage) = match root_obj.storage() {
                    crate::heap::pack::PackStorage::Root { descriptor, storage } => (*descriptor, storage),
                    crate::heap::pack::PackStorage::Slice { .. } => return None,
                };
                (
                    *root,
                    storage,
                    descriptor,
                    object.positional_start(),
                    object.positional_len(),
                    object.labeled_start(),
                    object.labeled_len(),
                )
            }
        };
        let (shape_id, layout) = product_metadata(self, descriptor_id)?;
        let Some(ProductShape::Pack(shape)) = self.product_shapes.get(shape_id) else {
            return None;
        };
        Some(PackView {
            root: Some(root_id),
            storage: Some(storage),
            shape: Some(shape),
            layout: Some(layout),
            pos_start,
            pos_len,
            lab_start,
            lab_len,
        })
    }

    /// Canonical value-level Pack view. EmptyPack is the zero-lane Pack
    /// and is represented directly without allocating a heap object.
    pub fn pack_view_value(&self, value: Value) -> Option<PackView<'_>> {
        if value.is_empty_pack() {
            return Some(PackView::empty());
        }
        self.pack_view(value.as_obj()?)
    }
}

/// Read-only view over a materialized Pack (root or slice) and its shared metadata.
pub struct PackView<'a> {
    root: Option<ObjRef>,
    storage: Option<&'a super::ProductStorage>,
    shape: Option<&'a super::PackProductShape>,
    layout: Option<&'a super::ProductLayout>,
    pos_start: u32,
    pos_len: u32,
    lab_start: u32,
    lab_len: u32,
}

impl<'a> PackView<'a> {
    pub fn empty() -> Self {
        Self {
            root: None,
            storage: None,
            shape: None,
            layout: None,
            pos_start: 0,
            pos_len: 0,
            lab_start: 0,
            lab_len: 0,
        }
    }

    pub fn positional_len(&self) -> usize {
        self.pos_len as usize
    }

    pub fn labeled_len(&self) -> usize {
        self.lab_len as usize
    }

    pub fn len(&self) -> usize {
        self.positional_len() + self.labeled_len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn root_obj(&self) -> Option<ObjRef> {
        self.root
    }

    pub fn positional_start(&self) -> u32 {
        self.pos_start
    }

    pub fn labeled_start(&self) -> u32 {
        self.lab_start
    }

    /// O(1) load of visible positional element at index.
    pub fn get_positional(&self, index: usize) -> Option<Value> {
        if index >= self.positional_len() {
            return None;
        }
        let logical_index = self.pos_start as usize + index;
        self.storage?.load_component(self.layout?, logical_index as u32).ok()
    }

    /// O(1) lookup of visible label symbol at index.
    pub fn get_label(&self, index: usize) -> Option<Symbol> {
        if index >= self.labeled_len() {
            return None;
        }
        let root_label_index = self.lab_start as usize + index;
        self.shape?.labels.get(root_label_index).copied()
    }

    /// O(1) load of visible labeled element at index.
    pub fn get_labeled_value(&self, index: usize) -> Option<Value> {
        if index >= self.labeled_len() {
            return None;
        }
        let root_pos_len = self.shape?.positional_len as usize;
        let logical_index = root_pos_len + self.lab_start as usize + index;
        self.storage?.load_component(self.layout?, logical_index as u32).ok()
    }

    /// Lookup labeled value by label spelling/Symbol.
    pub fn get_by_label(&self, label: Symbol) -> Option<Value> {
        let shape = self.shape?;
        let visible_labels = &shape.labels[self.lab_start as usize..(self.lab_start + self.lab_len) as usize];
        let index = visible_labels.iter().position(|l| *l == label)?;
        self.get_labeled_value(index)
    }

    pub fn positionals(&self) -> Vec<Value> {
        (0..self.positional_len()).filter_map(|i| self.get_positional(i)).collect()
    }

    pub fn labels(&self) -> Vec<Symbol> {
        (0..self.labeled_len()).filter_map(|i| self.get_label(i)).collect()
    }

    pub fn labeled_values(&self) -> Vec<Value> {
        (0..self.labeled_len()).filter_map(|i| self.get_labeled_value(i)).collect()
    }

    pub fn labeled_entries(&self) -> Vec<(Symbol, Value)> {
        self.labels().into_iter().zip(self.labeled_values()).collect()
    }
}
