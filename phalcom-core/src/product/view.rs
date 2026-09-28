//! Registry-aware read views for materialized anonymous products.

use super::{ProductShape, ProductShapeId};
use crate::heap::ObjRef;
use crate::interner::Symbol;
use crate::value::Value;
use crate::vm::VM;

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

    pub fn labels(&self) -> &[Symbol] {
        self.shape.map_or(&[], |shape| shape.presentation_labels.as_ref())
    }

    pub fn values(&self) -> Vec<Value> {
        let (Some(object), Some(shape), Some(layout)) = (self.object, self.shape, self.layout) else {
            return Vec::new();
        };
        shape
            .presentation_to_logical
            .iter()
            .filter_map(|logical| object.storage().load_component(layout, *logical).ok())
            .collect()
    }

    pub fn get(&self, label: Symbol) -> Option<Value> {
        let (Some(object), Some(shape), Some(layout)) = (self.object, self.shape, self.layout) else {
            return None;
        };
        let logical = shape.logical_labels.iter().position(|candidate| *candidate == label)?;
        object.storage().load_component(layout, logical as u32).ok()
    }

    pub fn entries(&self) -> Vec<(Symbol, Value)> {
        self.labels().iter().copied().zip(self.values()).collect()
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
}
