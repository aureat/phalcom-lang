//! Raw immutable `Record` observations.

use crate::error::{PhResult, RuntimeError};
use crate::primitive::expect_record;
use crate::primitive::tuple::expect_index;
use crate::value::Value;
use crate::vm::VM;

pub fn record_raw_size(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = expect_record(vm, receiver)?;
    Ok(Value::Int(vm.heap.record(id).len() as i64))
}

pub fn record_raw_label_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_record(vm, receiver)?;
    let index = expect_index(&args[0])?;
    Ok(vm.heap.record(id).labels().get(index).copied().map(Value::Symbol).unwrap_or_else(|| vm.none_value()))
}

pub fn record_raw_value_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_record(vm, receiver)?;
    let index = expect_index(&args[0])?;
    Ok(vm.heap.record(id).values().get(index).copied().unwrap_or_else(|| vm.none_value()))
}
