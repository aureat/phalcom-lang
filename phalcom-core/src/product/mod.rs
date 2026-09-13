//! Internal product construction boundary and packed storage subsystem.

pub mod layout;
pub mod registry;
pub mod storage;

pub use layout::{ProductComponentLayout, ProductComponentSpec, ProductLayout, ProductLayoutId, ProductLayoutSpec, ProductSlotRepr};
pub use registry::ProductLayoutRegistry;
pub use storage::ProductStorage;

use crate::error::RuntimeError;
use crate::interner::Symbol;
use crate::value::Value;
use crate::vm::VM;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProductBuildError {
    DuplicateLabel(Symbol),
}

pub(crate) fn runtime_error(vm: &VM, product: &'static str, error: ProductBuildError) -> RuntimeError {
    match error {
        ProductBuildError::DuplicateLabel(label) => RuntimeError::DuplicateProductLabel {
            product,
            label: vm.resolve_symbol(label).to_owned(),
        },
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
    Ok(Value::obj(
        vm.heap.alloc_tuple_nonempty(positionals.into_boxed_slice(), labels.into_boxed_slice()),
    ))
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
    Ok(Value::obj(vm.heap.alloc_record_nonempty(labels.into_boxed_slice(), values.into_boxed_slice())))
}

#[cfg(test)]
mod tests {
    use super::{finish_record, finish_tuple};
    use super::layout::{ProductComponentLayout, ProductLayout, ProductSlotRepr};
    use super::registry::ProductLayoutRegistry;
    use super::storage::ProductStorage;
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
            ProductComponentLayout { logical_index: 0, word_offset: 0, repr: ProductSlotRepr::Int64 },
            ProductComponentLayout { logical_index: 1, word_offset: 1, repr: ProductSlotRepr::Float64 },
            ProductComponentLayout { logical_index: 2, word_offset: 2, repr: ProductSlotRepr::Bool },
            ProductComponentLayout { logical_index: 3, word_offset: 3, repr: ProductSlotRepr::Symbol },
            ProductComponentLayout { logical_index: 4, word_offset: 4, repr: ProductSlotRepr::Value },
        ];
        let layout = ProductLayout::new(components).expect("valid layout");
        assert_eq!(layout.word_len, 6);
        assert_eq!(layout.value_slot_offsets(), &[4]);

        let mut registry = ProductLayoutRegistry::new();
        let layout_id = registry.register(layout.clone());

        let mut storage = ProductStorage::new(layout_id, &layout);
        storage.store_component(&layout, 0, Value::int(42)).unwrap();
        storage.store_component(&layout, 1, Value::float(3.14)).unwrap();
        storage.store_component(&layout, 2, Value::bool(true)).unwrap();
        storage.store_component(&layout, 3, Value::symbol(Symbol(7))).unwrap();
        storage.store_component(&layout, 4, Value::int(-100)).unwrap();

        assert_eq!(storage.load_component(&layout, 0).unwrap(), Value::int(42));
        assert_eq!(storage.load_component(&layout, 1).unwrap(), Value::float(3.14));
        assert_eq!(storage.load_component(&layout, 2).unwrap(), Value::bool(true));
        assert_eq!(storage.load_component(&layout, 3).unwrap(), Value::symbol(Symbol(7)));
        assert_eq!(storage.load_component(&layout, 4).unwrap(), Value::int(-100));
    }
}
