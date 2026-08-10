//! Internal product construction boundary.

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
        return Ok(Value::Unit);
    }
    let mut labels = Vec::with_capacity(labeled.len());
    for (label, value) in labeled {
        labels.push(label);
        positionals.push(value);
    }
    Ok(Value::Obj(
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
        return Ok(Value::Unit);
    }
    let (labels, values): (Vec<_>, Vec<_>) = fields.into_iter().unzip();
    Ok(Value::Obj(vm.heap.alloc_record_nonempty(labels.into_boxed_slice(), values.into_boxed_slice())))
}

#[cfg(test)]
mod tests {
    use super::{finish_record, finish_tuple};
    use crate::value::Value;
    use crate::vm::VM;

    #[test]
    fn empty_products_normalize_to_unit_without_heap_allocation() {
        std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024)
            .spawn(|| {
                let mut vm = VM::new();
                let before = vm.heap.live_count();

                assert_eq!(finish_tuple(&mut vm, Vec::new(), Vec::new()), Ok(Value::Unit));
                assert_eq!(finish_record(&mut vm, Vec::new()), Ok(Value::Unit));

                assert_eq!(vm.heap.live_count(), before);
            })
            .expect("spawn product test thread")
            .join()
            .expect("join product test thread");
    }
}
