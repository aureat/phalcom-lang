//! Internal product construction boundary.

use crate::interner::Symbol;
use crate::value::Value;
use crate::vm::VM;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProductBuildError {
    DuplicateLabel(Symbol),
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

pub(crate) fn finish_tuple(
    vm: &mut VM,
    mut positionals: Vec<Value>,
    labeled: Vec<(Symbol, Value)>,
) -> Result<Value, ProductBuildError> {
    unique(&labeled)?;
    if positionals.is_empty() && labeled.is_empty() {
        return Ok(Value::Unit);
    }
    let mut labels = Vec::with_capacity(labeled.len());
    for (label, value) in labeled {
        labels.push(label);
        positionals.push(value);
    }
    Ok(Value::Obj(vm.heap.alloc_tuple_nonempty(positionals.into_boxed_slice(), labels.into_boxed_slice())))
}

pub(crate) fn finish_record(vm: &mut VM, fields: Vec<(Symbol, Value)>) -> Result<Value, ProductBuildError> {
    unique(&fields)?;
    if fields.is_empty() {
        return Ok(Value::Unit);
    }
    let (labels, values): (Vec<_>, Vec<_>) = fields.into_iter().unzip();
    Ok(Value::Obj(vm.heap.alloc_record_nonempty(labels.into_boxed_slice(), values.into_boxed_slice())))
}
