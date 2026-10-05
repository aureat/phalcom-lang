//! Native primitives on `Pack`.

use crate::error::{PhResult, RuntimeError};
use crate::primitive::expect_pack_view;
use crate::primitive::tuple::expect_index;
use crate::product::finish_pack_slice_value;
use crate::value::Value;
use crate::vm::VM;

#[phalcom_native_macros::primitive(Pack, "rawPositionalSize", visibility = internal)]
pub fn pack_raw_positional_size(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let len = expect_pack_view(vm, receiver)?.positional_len();
    Ok(Value::int(len as i64))
}

#[phalcom_native_macros::primitive(Pack, "rawLabeledSize", visibility = internal)]
pub fn pack_raw_labeled_size(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let len = expect_pack_view(vm, receiver)?.labeled_len();
    Ok(Value::int(len as i64))
}

#[phalcom_native_macros::primitive(Pack, "rawPositionalAt(_)", visibility = internal)]
pub fn pack_raw_positional_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let view = expect_pack_view(vm, receiver)?;
    let index = expect_index(&args[0])?;
    view.get_positional(index).ok_or_else(|| RuntimeError::Internal("Pack rawPositionalAt index is out of range".into()).into())
}

#[phalcom_native_macros::primitive(Pack, "rawLabelAt(_)", visibility = internal)]
pub fn pack_raw_label_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let view = expect_pack_view(vm, receiver)?;
    let index = expect_index(&args[0])?;
    let sym = view.get_label(index).ok_or_else(|| RuntimeError::Internal("Pack rawLabelAt index is out of range".into()))?;
    Ok(Value::symbol(sym))
}

#[phalcom_native_macros::primitive(Pack, "rawLabeledValueAt(_)", visibility = internal)]
pub fn pack_raw_labeled_value_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let view = expect_pack_view(vm, receiver)?;
    let index = expect_index(&args[0])?;
    view.get_labeled_value(index).ok_or_else(|| RuntimeError::Internal("Pack rawLabeledValueAt index is out of range".into()).into())
}

#[phalcom_native_macros::primitive(Pack, "rawPositionalRestSlice", visibility = internal)]
pub fn pack_raw_positional_rest_slice(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let view = expect_pack_view(vm, receiver)?;
    let pos_len = view.positional_len() as u32;
    if pos_len == 0 {
        return Ok(Value::empty_pack());
    }
    finish_pack_slice_value(vm, *receiver, 1, pos_len - 1, 0, 0)
}

#[phalcom_native_macros::primitive(Pack, "rawLabeledRestSlice", visibility = internal)]
pub fn pack_raw_labeled_rest_slice(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let view = expect_pack_view(vm, receiver)?;
    let lab_len = view.labeled_len() as u32;
    if lab_len == 0 {
        return Ok(Value::empty_pack());
    }
    finish_pack_slice_value(vm, *receiver, 0, 0, 1, lab_len - 1)
}
