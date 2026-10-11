//! Owner-rooted, read-only views over contiguous collection storage.

use super::ObjRef;

/// Failure while constructing a normalized Slice view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliceError {
    /// The requested owner is stale or is not an Array, Vector, Buffer, or Slice.
    UnsupportedOwner,
    /// Range arithmetic overflowed.
    RangeOverflow,
    /// The requested range does not fit within the visible owner range.
    OutOfBounds,
}

/// A read-only range over a strong heap owner.
///
/// `owner` is always an Array, Vector, or Buffer. Constructing a Slice from
/// another Slice composes its offset so Slice chains never form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SliceObject {
    owner: ObjRef,
    start: usize,
    len: usize,
}

impl SliceObject {
    pub(crate) fn new(owner: ObjRef, start: usize, len: usize) -> Self {
        Self { owner, start, len }
    }

    /// Returns the strong backing owner handle.
    pub fn owner(&self) -> ObjRef {
        self.owner
    }

    /// Returns the offset within the backing owner.
    pub fn start(&self) -> usize {
        self.start
    }

    /// Returns the visible runtime length.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the visible range is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[cfg(test)]
mod tests {
    use super::{SliceError, SliceObject};
    use crate::heap::{BufferObject, Heap, Object};
    use crate::primitive::index::{NormalizedIndex, normalize_element_index};
    use crate::value::Value;

    #[test]
    fn array_views_compose_ranges_and_reuse_negative_index_normalization() {
        let mut heap = Heap::new();
        let owner = heap.alloc_array((0..6).map(Value::int).collect());
        let full = heap.alloc_slice_from_range(owner, 0, 6).expect("full view");
        let partial = heap.alloc_slice_from_range(full, 1, 5).expect("partial view");
        let nested = heap.alloc_slice_from_range(partial, 2, 2).expect("nested view");
        let nested_view = heap.slice(nested);

        assert_eq!(nested_view.owner(), owner);
        assert_eq!(nested_view.start(), 3);
        assert_eq!(nested_view.len(), 2);
        assert_eq!(heap.slice_element(nested, 0), Some(Value::int(3)));
        assert_eq!(heap.slice_element(nested, 1), Some(Value::int(4)));
        assert_eq!(heap.slice_element(nested, 2), None);
        assert!(matches!(
            normalize_element_index(&heap, &Value::int(-1), nested_view.len()).expect("negative index normalization"),
            NormalizedIndex::Valid(1)
        ));
        assert_eq!(heap.slice_element(nested, 1), Some(Value::int(4)));
    }

    #[test]
    fn vector_and_buffer_views_retain_the_original_owner() {
        let mut heap = Heap::new();
        let vector = heap.alloc_vector(vec![Value::int(7), Value::int(8)]);
        let vector_slice = heap.alloc_slice_from_range(vector, 1, 1).expect("Vector view");
        assert_eq!(heap.slice(vector_slice).owner(), vector);
        assert_eq!(heap.slice_element(vector_slice, 0), Some(Value::int(8)));

        let mut buffer = BufferObject::try_new(3).expect("Buffer reservation");
        buffer.push(Value::int(9)).expect("initialized value");
        buffer.push(Value::int(10)).expect("initialized value");
        let buffer = heap.alloc_buffer(buffer);
        let buffer_slice = heap.alloc_slice_from_range(buffer, 0, 2).expect("Buffer view");
        assert_eq!(heap.slice(buffer_slice).owner(), buffer);
        assert_eq!(heap.slice_element(buffer_slice, 1), Some(Value::int(10)));
    }

    #[test]
    fn empty_and_one_element_ranges_are_valid_including_the_end_boundary() {
        let mut heap = Heap::new();
        let owner = heap.alloc_array(vec![Value::int(1), Value::int(2)]);
        let empty = heap.alloc_slice_from_range(owner, 2, 0).expect("empty end view");
        assert!(heap.slice(empty).is_empty());
        assert_eq!(heap.slice(empty).start(), 2);
        let one = heap.alloc_slice_from_range(owner, 1, 1).expect("one element view");
        assert_eq!(heap.slice_element(one, 0), Some(Value::int(2)));
    }

    #[test]
    fn deep_nested_views_stay_flat_and_range_failures_are_checked() {
        let mut heap = Heap::new();
        let owner = heap.alloc_array((0..4).map(Value::int).collect());
        let mut view = heap.alloc_slice_from_range(owner, 1, 3).expect("initial view");
        for _ in 0..512 {
            view = heap.alloc_slice_from_range(view, 0, 3).expect("nested view");
        }
        assert_eq!(heap.slice(view).owner(), owner);
        assert_eq!(heap.slice(view).start(), 1);
        assert_eq!(heap.slice(view).len(), 3);
        assert_eq!(heap.alloc_slice_from_range(owner, usize::MAX, 1), Err(SliceError::RangeOverflow));
        assert_eq!(heap.alloc_slice_from_range(owner, 5, 0), Err(SliceError::OutOfBounds));
        assert_eq!(heap.alloc_slice_from_range(owner, 3, 2), Err(SliceError::OutOfBounds));
        let unsupported = heap.alloc_string("not a sequence".to_owned());
        assert_eq!(heap.alloc_slice_from_range(unsupported, 0, 0), Err(SliceError::UnsupportedOwner));
    }

    #[test]
    fn tracing_a_slice_adds_only_its_backing_owner_edge() {
        let mut heap = Heap::new();
        let owner = heap.alloc_array(vec![Value::int(1), Value::int(2)]);
        let slice = heap.alloc_slice_from_range(owner, 0, 2).expect("Slice view");
        let mut edges = Vec::new();
        crate::heap::trace_object(heap.get(slice), &heap.product_layouts, &mut |edge| edges.push(edge));
        assert_eq!(edges, vec![owner]);
        assert!(matches!(heap.get(slice), Object::Slice(SliceObject { .. })));
    }
}
