//! Raw immutable `Record` observations.

use crate::error::PhResult;
use crate::primitive::expect_record;
use crate::primitive::tuple::expect_index;
use crate::value::Value;
use crate::vm::VM;
use std::hash::{DefaultHasher, Hash, Hasher};

#[phalcom_native_macros::primitive(Record, "rawSize", visibility = internal)]
pub fn record_raw_size(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let len = expect_record(vm, receiver)?.len();
    Ok(Value::int(len as i64))
}

#[phalcom_native_macros::primitive(Record, "rawLabelAt(_)", visibility = internal)]
pub fn record_raw_label_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let index = expect_index(&args[0])?;
    let record = expect_record(vm, receiver)?;
    let key = record
        .presentation_key(index)
        .ok_or_else(|| crate::error::RuntimeError::Internal("Record rawLabelAt index is out of range".to_string()))?;
    let phalcom_semantic::field_key::FieldKey::Symbol(text) = key else {
        return Err(crate::error::RuntimeError::RecordLabelNotSymbol { key: key.to_string() }.into());
    };
    let text = text.clone();
    Ok(Value::symbol(vm.interner.intern(&text)))
}

#[phalcom_native_macros::primitive(Record, "rawKeyAt(_)", visibility = internal)]
pub fn record_raw_key_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let index = expect_index(&args[0])?;
    let key = expect_record(vm, receiver)?
        .presentation_key(index)
        .ok_or_else(|| crate::error::RuntimeError::Internal("Record rawKeyAt index is out of range".to_string()))?
        .clone();
    Ok(crate::product::field_key_to_value(vm, &key))
}

#[phalcom_native_macros::primitive(Record, "rawValueAt(_)", visibility = internal)]
pub fn record_raw_value_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let index = expect_index(&args[0])?;
    expect_record(vm, receiver)?
        .values()
        .get(index)
        .copied()
        .ok_or_else(|| crate::error::RuntimeError::Internal("Record rawValueAt index is out of range".to_string()).into())
}

#[phalcom_native_macros::primitive(Record, "rawLogicalValueAt(_)", visibility = internal)]
pub fn record_raw_logical_value_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let index = expect_index(&args[0])?;
    expect_record(vm, receiver)?
        .logical_value_at(index)
        .ok_or_else(|| crate::error::RuntimeError::Internal("Record rawLogicalValueAt index is out of range".to_string()).into())
}

#[phalcom_native_macros::primitive(Record, "rawSameKeys(_)", visibility = internal)]
pub fn record_raw_same_keys(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let left = expect_record(vm, receiver)?;
    let right = expect_record(vm, &args[0])?;
    Ok(Value::bool(left.logical_keys() == right.logical_keys()))
}

#[phalcom_native_macros::primitive(Record, "rawKeyHashAt(_)", visibility = internal)]
pub fn record_raw_key_hash_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let index = expect_index(&args[0])?;
    let record = expect_record(vm, receiver)?;
    let key = record
        .logical_keys()
        .get(index)
        .ok_or_else(|| crate::error::RuntimeError::Internal("Record rawKeyHashAt index is out of range".to_string()))?;
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    Ok(Value::int((hasher.finish() % 999_999_937) as i64))
}

#[phalcom_native_macros::primitive(Record, "rawLookup(_)", visibility = internal)]
pub fn record_raw_lookup(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let record = expect_record(vm, receiver)?;
    match record.get_by_value(vm, args[0])? {
        Some(value) => Ok(value.wrap_some()?),
        None => Ok(Value::none()),
    }
}
