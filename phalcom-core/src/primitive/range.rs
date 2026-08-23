//! Native primitives on `Range`.
//!
//! Realizes the [ADR-0039](../../../docs/adr/accepted/0039-amend-floor-admit-collection-container-primitives.md)
//! floor for [ADR-0032](../../../docs/adr/accepted/0032-collections-representation-and-literals.md)'s
//! native `Range`: three raw bound observations. Syntax constructs Range
//! directly; each optional observation distinguishes omission from `None`.

use crate::error::PhResult;
use crate::heap::ObjRef;
use crate::primitive::expect_range;
use crate::value::Value;
use crate::vm::VM;

/// Signature: `Range::lower_` — `None` for omission, otherwise `Some(value)`.
#[phalcom_native_macros::primitive(Range, "_$lower", visibility = internal)]
pub fn range_raw_lower(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_range(vm, receiver)?;
    let lower = vm.heap.range(id).lower();
    Ok(match lower {
        Some(value) => crate::primitive::nil::wrap_some(vm, value)?,
        None => vm.none_value(),
    })
}

/// Signature: `Range::upper_` — `None` for omission, otherwise `Some(value)`.
#[phalcom_native_macros::primitive(Range, "_$upper", visibility = internal)]
pub fn range_raw_upper(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_range(vm, receiver)?;
    let upper = vm.heap.range(id).upper();
    Ok(match upper {
        Some(value) => crate::primitive::nil::wrap_some(vm, value)?,
        None => vm.none_value(),
    })
}

/// Signature: `Range::upperInclusive_`.
#[phalcom_native_macros::primitive(Range, "_$upperInclusive", visibility = internal)]
pub fn range_raw_upper_inclusive(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id: ObjRef = expect_range(vm, receiver)?;
    Ok(Value::bool(vm.heap.range(id).upper_inclusive()))
}
