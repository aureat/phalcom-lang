//! Internal product construction boundary and packed storage subsystem.

pub mod anonymous;
pub mod layout;
pub mod registry;
pub mod shape;
pub mod storage;
pub mod view;

pub use anonymous::{RuntimeAnonymousProductDescriptor, RuntimeAnonymousProductDescriptorId, RuntimeAnonymousProductDescriptorRegistry};
pub use layout::{ProductComponentLayout, ProductComponentSpec, ProductLayout, ProductLayoutId, ProductLayoutSpec, ProductSlotRepr};
pub use registry::ProductLayoutRegistry;
pub use shape::{AnonymousProductKind, ProductShape, ProductShapeId, ProductShapeRegistry, RecordProductShape, TupleProductShape};
pub use storage::ProductStorage;
pub use view::{RecordView, TupleView};


use crate::error::RuntimeError;
use crate::error::PhResult;
use crate::interner::Symbol;
use crate::value::Value;
use crate::vm::VM;
use crate::modules::semantic_lowering::{AnonymousProductConstructionKind, AnonymousProductConstructionLoweringSpec};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProductBuildError {
    DuplicateLabel(Symbol),
    Storage(&'static str),
    InvalidSpec,
}

pub(crate) fn runtime_error(vm: &VM, product: &'static str, error: ProductBuildError) -> RuntimeError {
    match error {
        ProductBuildError::DuplicateLabel(label) => RuntimeError::DuplicateProductLabel {
            product,
            label: vm.resolve_symbol(label).to_owned(),
        },
        ProductBuildError::Storage(message) => RuntimeError::Internal(format!("{product} storage construction failed: {message}")),
        ProductBuildError::InvalidSpec => RuntimeError::Internal(format!("{product} construction specification is invalid")),
    }
}

fn unique(entries: &[(Symbol, Value)]) -> Result<(), ProductBuildError> {
    let mut seen = HashSet::with_capacity(entries.len());
    for (label, _) in entries {
        if !seen.insert(*label) {
            return Err(ProductBuildError::DuplicateLabel(*label));
        }
    }
    Ok(())
}

/// Finalizes a Tuple at the product representation boundary.
///
/// The compiler normalizes source `()` directly to `Unit` as an allocation
/// optimization. This runtime check remains the invariant boundary for every
/// other construction route: no heap-allocated empty [`TupleObject`] may
/// exist. Bytecode passes positional values first, followed by labeled values;
/// `labels` describes that labeled suffix only.
pub(crate) fn finish_tuple(vm: &mut VM, mut positionals: Vec<Value>, labeled: Vec<(Symbol, Value)>) -> Result<Value, ProductBuildError> {
    unique(&labeled)?;
    if positionals.is_empty() && labeled.is_empty() {
        return Ok(Value::unit());
    }
    let mut labels = Vec::with_capacity(labeled.len());
    for (label, value) in labeled {
        labels.push(label);
        positionals.push(value);
    }
    let shape = ProductShape::Tuple(TupleProductShape::new(positionals.len() as u32 - labels.len() as u32, labels.into_boxed_slice()));
    let shape_id = vm.product_shapes.register(shape);
    let layout_spec = ProductLayoutSpec::new(
        (0..positionals.len())
            .map(|logical_index| ProductComponentSpec {
                logical_index: logical_index as u32,
                repr: ProductSlotRepr::Value,
            })
            .collect(),
    );
    let layout = layout_spec.build_layout().map_err(ProductBuildError::Storage)?;
    let layout_id = vm.heap.product_layouts.register(layout);
    let layout = vm.heap.product_layouts.get(layout_id).ok_or(ProductBuildError::Storage("missing registered layout"))?;
    let storage = ProductStorage::from_values(layout_id, layout, &positionals).map_err(ProductBuildError::Storage)?;
    let descriptor = vm
        .anonymous_product_descriptors
        .register(AnonymousProductKind::Tuple, shape_id, layout_id, None);
    Ok(Value::obj(vm.heap.alloc_tuple_nonempty(descriptor, storage)))
}

/// Public VM-facing tuple construction adapter for integration boundaries.
///
/// All construction still flows through the private product finalizer; this
/// adapter only translates its internal construction error into the normal
/// runtime result type for code outside `phalcom-core`.
pub fn finish_tuple_value(vm: &mut VM, positionals: Vec<Value>, labeled: Vec<(Symbol, Value)>) -> PhResult<Value> {
    finish_tuple(vm, positionals, labeled).map_err(|error| runtime_error(vm, "Tuple label", error).into())
}

/// Finalizes a Record at the product representation boundary.
///
/// The compiler normalizes source `#{}` directly to `Unit` as an allocation
/// optimization. This runtime check is still required for dynamic construction:
/// no heap-allocated empty [`RecordObject`] may exist.
pub(crate) fn finish_record(vm: &mut VM, fields: Vec<(Symbol, Value)>) -> Result<Value, ProductBuildError> {
    unique(&fields)?;
    if fields.is_empty() {
        return Ok(Value::unit());
    }
    let (labels, values): (Vec<_>, Vec<_>) = fields.into_iter().unzip();
    let shape_id = vm
        .product_shapes
        .register(ProductShape::Record(RecordProductShape::from_ordered_labels(labels.into_boxed_slice())));
    let layout_spec = ProductLayoutSpec::new(
        (0..values.len())
            .map(|logical_index| ProductComponentSpec {
                logical_index: logical_index as u32,
                repr: ProductSlotRepr::Value,
            })
            .collect(),
    );
    let layout = layout_spec.build_layout().map_err(ProductBuildError::Storage)?;
    let layout_id = vm.heap.product_layouts.register(layout);
    let layout = vm.heap.product_layouts.get(layout_id).ok_or(ProductBuildError::Storage("missing registered layout"))?;
    let storage = ProductStorage::from_values(layout_id, layout, &values).map_err(ProductBuildError::Storage)?;
    let descriptor = vm
        .anonymous_product_descriptors
        .register(AnonymousProductKind::Record, shape_id, layout_id, None);
    Ok(Value::obj(vm.heap.alloc_record_nonempty(descriptor, storage)))
}

/// Finalizes a statically-shaped Tuple from source-order component values.
pub(crate) fn finish_tuple_from_spec(
    vm: &mut VM,
    spec: &AnonymousProductConstructionLoweringSpec,
    source_values: Vec<Value>,
) -> Result<Value, ProductBuildError> {
    let AnonymousProductConstructionKind::Tuple { positional_len, labels } = &spec.kind else {
        return Err(ProductBuildError::InvalidSpec);
    };
    let total = usize::try_from(*positional_len).ok().and_then(|n| n.checked_add(labels.len())).ok_or(ProductBuildError::InvalidSpec)?;
    if total == 0 || source_values.len() != total || spec.layout.components.len() != total {
        return if total == 0 && source_values.is_empty() {
            Ok(Value::unit())
        } else {
            Err(ProductBuildError::InvalidSpec)
        };
    }
    let labels = labels.iter().map(|label| vm.interner.intern(label)).collect::<Vec<_>>().into_boxed_slice();
    let shape_id = vm.product_shapes.register(ProductShape::Tuple(TupleProductShape::new(*positional_len, labels)));
    let layout = spec.layout.build_layout().map_err(ProductBuildError::Storage)?;
    let layout_id = vm.heap.product_layouts.register(layout);
    let layout = vm.heap.product_layouts.get(layout_id).ok_or(ProductBuildError::Storage("missing registered layout"))?;
    let storage = ProductStorage::from_values(layout_id, layout, &source_values).map_err(ProductBuildError::Storage)?;
    let descriptor = vm.anonymous_product_descriptors.register(AnonymousProductKind::Tuple, shape_id, layout_id, None);
    Ok(Value::obj(vm.heap.alloc_tuple_nonempty(descriptor, storage)))
}

/// Finalizes a statically-shaped Record from source-order component values.
pub(crate) fn finish_record_from_spec(
    vm: &mut VM,
    spec: &AnonymousProductConstructionLoweringSpec,
    source_values: Vec<Value>,
) -> Result<Value, ProductBuildError> {
    let AnonymousProductConstructionKind::Record {
        presentation_labels,
        logical_labels,
        source_to_logical,
    } = &spec.kind
    else {
        return Err(ProductBuildError::InvalidSpec);
    };
    let total = presentation_labels.len();
    if total == 0 || source_values.len() != total || logical_labels.len() != total || source_to_logical.len() != total || spec.layout.components.len() != total {
        return if total == 0 && source_values.is_empty() {
            Ok(Value::unit())
        } else {
            Err(ProductBuildError::InvalidSpec)
        };
    }
    let presentation = presentation_labels.iter().map(|label| vm.interner.intern(label)).collect::<Vec<_>>().into_boxed_slice();
    let logical = logical_labels.iter().map(|label| vm.interner.intern(label)).collect::<Vec<_>>().into_boxed_slice();
    let shape_id = vm.product_shapes.register(ProductShape::Record(RecordProductShape::new(
        presentation,
        logical,
        source_to_logical.clone(),
    )));
    let layout = spec.layout.build_layout().map_err(ProductBuildError::Storage)?;
    let layout_id = vm.heap.product_layouts.register(layout);
    let layout = vm.heap.product_layouts.get(layout_id).ok_or(ProductBuildError::Storage("missing registered layout"))?;
    let mut logical_values = vec![Value::unit(); total];
    for (source_index, logical_index) in source_to_logical.iter().copied().enumerate() {
        let Some(slot) = logical_values.get_mut(logical_index as usize) else {
            return Err(ProductBuildError::InvalidSpec);
        };
        *slot = source_values[source_index];
    }
    let storage = ProductStorage::from_values(layout_id, layout, &logical_values).map_err(ProductBuildError::Storage)?;
    let descriptor = vm.anonymous_product_descriptors.register(AnonymousProductKind::Record, shape_id, layout_id, None);
    Ok(Value::obj(vm.heap.alloc_record_nonempty(descriptor, storage)))
}

#[cfg(test)]
mod tests {
    use super::layout::{ProductComponentLayout, ProductLayout, ProductSlotRepr};
    use super::registry::ProductLayoutRegistry;
    use super::storage::ProductStorage;
    use super::{finish_record, finish_record_from_spec, finish_tuple, finish_tuple_from_spec};
    use crate::modules::semantic_lowering::{AnonymousProductConstructionKind, AnonymousProductConstructionLoweringSpec};
    use crate::interner::Symbol;
    use crate::value::Value;
    use crate::vm::VM;

    #[test]
    fn empty_products_normalize_to_unit_without_heap_allocation() {
        std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(|| {
                let mut vm = VM::new_kernel();
                let before = vm.heap.live_count();

                assert_eq!(finish_tuple(&mut vm, Vec::new(), Vec::new()), Ok(Value::unit()));
                assert_eq!(finish_record(&mut vm, Vec::new()), Ok(Value::unit()));

                assert_eq!(vm.heap.live_count(), before);
            })
            .expect("spawn product test thread")
            .join()
            .expect("join product test thread");
    }

    #[test]
    fn product_layout_storage_round_trip() {
        let components = vec![
            ProductComponentLayout {
                logical_index: 0,
                word_offset: 0,
                repr: ProductSlotRepr::Int64,
            },
            ProductComponentLayout {
                logical_index: 1,
                word_offset: 1,
                repr: ProductSlotRepr::Float64,
            },
            ProductComponentLayout {
                logical_index: 2,
                word_offset: 2,
                repr: ProductSlotRepr::Bool,
            },
            ProductComponentLayout {
                logical_index: 3,
                word_offset: 3,
                repr: ProductSlotRepr::Symbol,
            },
            ProductComponentLayout {
                logical_index: 4,
                word_offset: 4,
                repr: ProductSlotRepr::Value,
            },
        ];
        let layout = ProductLayout::new(components).expect("valid layout");
        assert_eq!(layout.word_len, 6);
        assert_eq!(layout.value_slot_offsets(), &[4]);

        let mut registry = ProductLayoutRegistry::new();
        let layout_id = registry.register(layout.clone());

        let mut storage = ProductStorage::new(layout_id, &layout);
        storage.store_component(&layout, 0, Value::int(42)).unwrap();
        storage.store_component(&layout, 1, Value::float(3.125)).unwrap();
        storage.store_component(&layout, 2, Value::bool(true)).unwrap();
        storage.store_component(&layout, 3, Value::symbol(Symbol(7))).unwrap();
        storage.store_component(&layout, 4, Value::int(-100)).unwrap();

        assert_eq!(storage.load_component(&layout, 0).unwrap(), Value::int(42));
        assert_eq!(storage.load_component(&layout, 1).unwrap(), Value::float(3.125));
        assert_eq!(storage.load_component(&layout, 2).unwrap(), Value::bool(true));
        assert_eq!(storage.load_component(&layout, 3).unwrap(), Value::symbol(Symbol(7)));
        assert_eq!(storage.load_component(&layout, 4).unwrap(), Value::int(-100));
    }

    #[test]
    fn product_storage_layout_mismatch_is_caught() {
        let comp1 = vec![ProductComponentLayout {
            logical_index: 0,
            word_offset: 0,
            repr: ProductSlotRepr::Int64,
        }];
        let layout1 = ProductLayout::new(comp1).expect("layout1");

        let comp2 = vec![
            ProductComponentLayout {
                logical_index: 0,
                word_offset: 0,
                repr: ProductSlotRepr::Int64,
            },
            ProductComponentLayout {
                logical_index: 1,
                word_offset: 1,
                repr: ProductSlotRepr::Int64,
            },
        ];
        let layout2 = ProductLayout::new(comp2).expect("layout2");

        let mut registry = ProductLayoutRegistry::new();
        let id1 = registry.register(layout1.clone());

        let mut storage = ProductStorage::new(id1, &layout1);
        assert_eq!(storage.word_len(), 1);
        assert_eq!(storage.layout_id(), id1);

        // Accessing storage with a layout of different word size must fail cleanly
        assert!(storage.store_component(&layout2, 0, Value::int(1)).is_err());
        assert!(storage.load_component(&layout2, 0).is_err());
    }

    #[test]
    fn static_tuple_spec_uses_shared_packed_storage() {
        let mut vm = VM::new_kernel();
        let spec = AnonymousProductConstructionLoweringSpec {
            kind: AnonymousProductConstructionKind::Tuple {
                positional_len: 2,
                labels: Box::new([]),
            },
            layout: super::layout::ProductLayoutSpec::new(vec![
                super::layout::ProductComponentSpec { logical_index: 0, repr: ProductSlotRepr::Float64 },
                super::layout::ProductComponentSpec { logical_index: 1, repr: ProductSlotRepr::Bool },
            ]),
        };
        let value = finish_tuple_from_spec(&mut vm, &spec, vec![Value::float(1.5), Value::bool(true)]).expect("static tuple");
        let id = value.as_obj().expect("tuple object");
        assert_eq!(vm.heap.tuple(id).storage().word_len(), 2);
        assert_eq!(vm.tuple_view(id).expect("tuple view").values(), vec![Value::float(1.5), Value::bool(true)]);
    }

    #[test]
    fn static_record_spec_preserves_presentation_order_over_logical_storage() {
        let mut vm = VM::new_kernel();
        let spec = AnonymousProductConstructionLoweringSpec {
            kind: AnonymousProductConstructionKind::Record {
                presentation_labels: Box::new(["z".into(), "a".into()]),
                logical_labels: Box::new(["a".into(), "z".into()]),
                source_to_logical: Box::new([1, 0]),
            },
            layout: super::layout::ProductLayoutSpec::new(vec![
                super::layout::ProductComponentSpec { logical_index: 0, repr: ProductSlotRepr::Bool },
                super::layout::ProductComponentSpec { logical_index: 1, repr: ProductSlotRepr::Float64 },
            ]),
        };
        let value = finish_record_from_spec(&mut vm, &spec, vec![Value::float(2.5), Value::bool(false)]).expect("static record");
        let id = value.as_obj().expect("record object");
        assert_eq!(vm.heap.record(id).storage().word_len(), 2);
        let z = vm.interner.intern("z");
        let a = vm.interner.intern("a");
        let view = vm.record_view(id).expect("record view");
        assert_eq!(view.labels(), &[z, a]);
        assert_eq!(view.values(), vec![Value::float(2.5), Value::bool(false)]);
    }
}
