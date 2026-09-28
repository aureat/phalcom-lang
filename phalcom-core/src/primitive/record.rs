//! Raw immutable `Record` observations.

use crate::error::PhResult;
use crate::primitive::expect_record;
use crate::primitive::tuple::expect_index;
use crate::value::Value;
use crate::vm::VM;

#[phalcom_native_macros::primitive(Record, "rawSize", visibility = internal)]
pub fn record_raw_size(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    if receiver.is_empty_record() {
        return Ok(Value::int(0));
    }
    let id = expect_record(vm, receiver)?;
    let len = vm
        .record_view(id)
        .ok_or_else(|| crate::error::RuntimeError::Internal("Record descriptor is unavailable".to_string()))?
        .len();
    Ok(Value::int(len as i64))
}

#[phalcom_native_macros::primitive(Record, "rawLabelAt(_)", visibility = internal)]
pub fn record_raw_label_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    if receiver.is_empty_record() {
        return Err(crate::error::RuntimeError::Internal("Record rawLabelAt called without a field".to_string()).into());
    }
    let id = expect_record(vm, receiver)?;
    let index = expect_index(&args[0])?;
    let label = vm
        .record_view(id)
        .ok_or_else(|| crate::error::RuntimeError::Internal("Record descriptor is unavailable".to_string()))?
        .labels()
        .get(index)
        .copied()
        .ok_or_else(|| crate::error::RuntimeError::Internal("Record rawLabelAt index is out of range".to_string()))?;
    Ok(Value::symbol(label))
}

#[phalcom_native_macros::primitive(Record, "rawValueAt(_)", visibility = internal)]
pub fn record_raw_value_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    if receiver.is_empty_record() {
        return Err(crate::error::RuntimeError::Internal("Record rawValueAt called without a field".to_string()).into());
    }
    let id = expect_record(vm, receiver)?;
    let index = expect_index(&args[0])?;
    vm.record_view(id)
        .ok_or_else(|| crate::error::RuntimeError::Internal("Record descriptor is unavailable".to_string()))?
        .values()
        .get(index)
        .copied()
        .ok_or_else(|| crate::error::RuntimeError::Internal("Record rawValueAt index is out of range".to_string()).into())
}
