//! Raw immutable `Record` observations.

use crate::error::PhResult;
use crate::primitive::expect_record;
use crate::primitive::tuple::expect_index;
use crate::value::Value;
use crate::vm::VM;

#[phalcom_native_macros::primitive(Record, "_$size", visibility = internal)]
pub fn record_raw_size(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = expect_record(vm, receiver)?;
    Ok(Value::int(vm.heap.record(id).len() as i64))
}

#[phalcom_native_macros::primitive(Record, "_$labelAt(_)", visibility = internal)]
pub fn record_raw_label_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_record(vm, receiver)?;
    let index = expect_index(&args[0])?;
    Ok(vm
        .heap
        .record(id)
        .labels()
        .get(index)
        .copied()
        .map(Value::symbol)
        .unwrap_or_else(|| vm.none_value()))
}

#[phalcom_native_macros::primitive(Record, "_$valueAt(_)", visibility = internal)]
pub fn record_raw_value_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_record(vm, receiver)?;
    let index = expect_index(&args[0])?;
    Ok(vm.heap.record(id).values().get(index).copied().unwrap_or_else(|| vm.none_value()))
}
