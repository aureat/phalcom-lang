//! Registry-aware read views for materialized anonymous products.

use super::{ProductShape, ProductShapeId};
use crate::heap::ObjRef;
use crate::interner::Symbol;
use crate::value::Value;
use crate::vm::VM;

/// Read-only view over a materialized Tuple and its shared metadata.
pub struct TupleView<'a> {
    object: &'a crate::heap::TupleObject,
    shape: &'a super::TupleProductShape,
    layout: &'a super::ProductLayout,
}

impl<'a> TupleView<'a> {
    fn load(&self, index: usize) -> Option<Value> {
        self.object.storage().load_component(self.layout, index as u32).ok()
    }

    pub fn len(&self) -> usize {
        self.shape.total_len() as usize
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn positional_len(&self) -> usize {
        self.shape.positional_len as usize
    }

    pub fn labeled_len(&self) -> usize {
        self.shape.labels.len()
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
        &self.shape.labels
    }

    pub fn get_label(&self, label: Symbol) -> Option<Value> {
        self.shape
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
    object: &'a crate::heap::RecordObject,
    shape: &'a super::RecordProductShape,
    layout: &'a super::ProductLayout,
}

impl<'a> RecordView<'a> {
    pub fn len(&self) -> usize {
        self.shape.len()
    }

    pub fn is_empty(&self) -> bool {
        self.shape.is_empty()
    }

    pub fn labels(&self) -> &[Symbol] {
        &self.shape.presentation_labels
    }

    pub fn values(&self) -> Vec<Value> {
        self.shape
            .presentation_to_logical
            .iter()
            .filter_map(|logical| self.object.storage().load_component(self.layout, *logical).ok())
            .collect()
    }

    pub fn get(&self, label: Symbol) -> Option<Value> {
        let logical = self.shape.logical_labels.iter().position(|candidate| *candidate == label)?;
        self.object.storage().load_component(self.layout, logical as u32).ok()
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
        Some(TupleView { object, shape, layout })
    }

    pub fn record_view(&self, id: ObjRef) -> Option<RecordView<'_>> {
        let object = self.heap.as_record(id)?;
        let (shape_id, layout) = product_metadata(self, object.descriptor())?;
        let Some(ProductShape::Record(shape)) = self.product_shapes.get(shape_id) else {
            return None;
        };
        Some(RecordView { object, shape, layout })
    }
}
