//! Native keyed and positional operations for OrderedSet.

use crate::error::{MapMutationError, PhResult, RuntimeError};
use crate::heap::ObjRef;
use crate::primitive::int::expect_int_big;
use crate::primitive::ordered_hash::{self, KeyedCollectionKind};
use crate::primitive::{expect_ordered_set, is_mutable_collection_key, mutable_key_error};
use crate::value::Value;
use crate::vm::VM;
use num_traits::ToPrimitive;

fn mutation_error(error: MapMutationError) -> RuntimeError {
    match error {
        MapMutationError::Locked => RuntimeError::ConcurrentMutation { collection: "OrderedSet" },
        MapMutationError::OutOfRange => RuntimeError::Internal("OrderedSet slot was out of range".to_string()),
    }
}

fn locate(vm: &mut VM, id: ObjRef, value: Value) -> PhResult<(i64, Option<usize>)> {
    ordered_hash::locate_key(vm, id, value, KeyedCollectionKind::OrderedSet)
}

fn slot_index(vm: &mut VM, value: &Value) -> PhResult<Option<usize>> {
    Ok(expect_int_big(value, vm)?.to_usize())
}

#[phalcom_native_macros::primitive(OrderedSet, "new()", side = class, types = "() -> Self", flow = receiver)]
pub fn ordered_set_class_new(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::obj(vm.heap.alloc_ordered_set()))
}

#[phalcom_native_macros::primitive(OrderedSet, "rawSize", visibility = internal)]
pub fn ordered_set_raw_size(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = expect_ordered_set(vm, receiver)?;
    let size = i64::try_from(vm.heap.ordered_set(id).len()).map_err(|_| RuntimeError::NumericLimit("OrderedSet size exceeds Int range".to_string()))?;
    Ok(Value::int(size))
}

#[phalcom_native_macros::primitive(OrderedSet, "rawAdd(_)" , visibility = internal)]
pub fn ordered_set_raw_add(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_ordered_set(vm, receiver)?;
    let value = args[0];
    if is_mutable_collection_key(vm, &value) {
        return Err(mutable_key_error(vm, "OrderedSet").into());
    }
    let (hash, slot) = locate(vm, id, value)?;
    if slot.is_none() {
        vm.heap.ordered_set_mut(id).insert_new(hash, value).map_err(mutation_error)?;
    }
    Ok(*receiver)
}

#[phalcom_native_macros::primitive(OrderedSet, "rawAt(_)" , visibility = internal)]
pub fn ordered_set_raw_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_ordered_set(vm, receiver)?;
    let index = slot_index(vm, &args[0])?;
    Ok(index.and_then(|index| vm.heap.ordered_set(id).value_at(index)).unwrap_or(Value::nil()))
}

#[phalcom_native_macros::primitive(OrderedSet, "rawIndexOf(_)" , visibility = internal)]
pub fn ordered_set_raw_index_of(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_ordered_set(vm, receiver)?;
    let (_, slot) = locate(vm, id, args[0])?;
    match slot {
        Some(slot) => {
            let index = i64::try_from(slot).expect("OrderedSet slot count fits Int");
            Ok(crate::primitive::option::wrap_some(vm, Value::int(index))?)
        }
        None => Ok(vm.none_value()),
    }
}

#[phalcom_native_macros::primitive(OrderedSet, "rawInsertAt(_,_)" , visibility = internal)]
pub fn ordered_set_raw_insert_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_ordered_set(vm, receiver)?;
    let value = args[0];
    let Some(index) = slot_index(vm, &args[1])? else {
        return Ok(Value::bool(false));
    };
    if index > vm.heap.ordered_set(id).len() {
        return Ok(Value::bool(false));
    }
    if is_mutable_collection_key(vm, &value) {
        return Err(mutable_key_error(vm, "OrderedSet").into());
    }
    let (hash, existing) = locate(vm, id, value)?;
    if existing.is_some() {
        return Ok(Value::bool(false));
    }
    vm.heap.ordered_set_mut(id).insert_at(index, hash, value).map_err(mutation_error)?;
    Ok(Value::bool(true))
}

#[phalcom_native_macros::primitive(OrderedSet, "rawReplaceAt(_,_)" , visibility = internal)]
pub fn ordered_set_raw_replace_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_ordered_set(vm, receiver)?;
    let value = args[0];
    let Some(index) = slot_index(vm, &args[1])? else {
        return Ok(Value::bool(false));
    };
    if index >= vm.heap.ordered_set(id).len() {
        return Ok(Value::bool(false));
    }
    if is_mutable_collection_key(vm, &value) {
        return Err(mutable_key_error(vm, "OrderedSet").into());
    }
    let (hash, existing) = locate(vm, id, value)?;
    if existing.is_some_and(|existing| existing != index) {
        return Ok(Value::bool(false));
    }
    if existing == Some(index) {
        return Ok(Value::bool(true));
    }
    vm.heap.ordered_set_mut(id).replace_at(index, hash, value).map_err(mutation_error)?;
    Ok(Value::bool(true))
}

#[phalcom_native_macros::primitive(OrderedSet, "rawMove(_,_)" , visibility = internal)]
pub fn ordered_set_raw_move(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_ordered_set(vm, receiver)?;
    let Some(from) = slot_index(vm, &args[0])? else {
        return Ok(Value::bool(false));
    };
    let Some(to) = slot_index(vm, &args[1])? else {
        return Ok(Value::bool(false));
    };
    match vm.heap.ordered_set_mut(id).move_at(from, to) {
        Ok(()) => Ok(Value::bool(true)),
        Err(MapMutationError::OutOfRange) => Ok(Value::bool(false)),
        Err(error) => Err(mutation_error(error).into()),
    }
}

#[phalcom_native_macros::primitive(OrderedSet, "rawRemoveAt(_)" , visibility = internal)]
pub fn ordered_set_raw_remove_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_ordered_set(vm, receiver)?;
    let Some(index) = slot_index(vm, &args[0])? else {
        return Ok(Value::bool(false));
    };
    match vm.heap.ordered_set_mut(id).remove_at(index) {
        Ok(_) => Ok(Value::bool(true)),
        Err(MapMutationError::OutOfRange) => Ok(Value::bool(false)),
        Err(error) => Err(mutation_error(error).into()),
    }
}

#[phalcom_native_macros::primitive(OrderedSet, "rawClear", visibility = internal)]
pub fn ordered_set_raw_clear(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = expect_ordered_set(vm, receiver)?;
    vm.heap.ordered_set_mut(id).clear().map_err(mutation_error)?;
    Ok(Value::nil())
}
