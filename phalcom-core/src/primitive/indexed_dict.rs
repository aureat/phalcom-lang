//! Native keyed and positional operations for IndexedDict.

use crate::error::{MapMutationError, PhResult, RuntimeError};
use crate::heap::ObjRef;
use crate::primitive::int::expect_int_big;
use crate::primitive::option::wrap_some;
use crate::primitive::ordered_hash::{self, KeyedCollectionKind};
use crate::primitive::{expect_indexed_dict, is_mutable_collection_key, mutable_key_error};
use crate::value::Value;
use crate::vm::VM;
use num_traits::ToPrimitive;

fn mutation_error(error: MapMutationError) -> RuntimeError {
    match error {
        MapMutationError::Locked => RuntimeError::ConcurrentMutation { collection: "IndexedDict" },
        MapMutationError::OutOfRange => RuntimeError::Internal("IndexedDict slot was out of range".to_string()),
    }
}

fn locate(vm: &mut VM, id: ObjRef, key: Value) -> PhResult<(i64, Option<usize>)> {
    ordered_hash::locate_key(vm, id, key, KeyedCollectionKind::IndexedDict)
}

fn slot_index(vm: &mut VM, value: &Value) -> PhResult<Option<usize>> {
    Ok(expect_int_big(value, vm)?.to_usize())
}

#[phalcom_native_macros::primitive(IndexedDict, "new()", side = class, types = "() -> Self", flow = receiver)]
pub fn indexed_dict_class_new(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::obj(vm.heap.alloc_indexed_dict()))
}

#[phalcom_native_macros::primitive(IndexedDict, "rawSize", visibility = internal)]
pub fn indexed_dict_raw_size(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = expect_indexed_dict(vm, receiver)?;
    let size = i64::try_from(vm.heap.indexed_dict(id).len()).map_err(|_| RuntimeError::NumericLimit("IndexedDict size exceeds Int range".to_string()))?;
    Ok(Value::int(size))
}

#[phalcom_native_macros::primitive(IndexedDict, "rawPut(_,_)" , visibility = internal)]
pub fn indexed_dict_raw_put(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_indexed_dict(vm, receiver)?;
    let key = args[0];
    let value = args[1];
    if is_mutable_collection_key(vm, &key) {
        return Err(mutable_key_error(vm, "IndexedDict").into());
    }
    let (hash, slot) = locate(vm, id, key)?;
    match slot {
        Some(slot) => {
            // Read at the outer commit point: a permitted reentrant value-only
            // overwrite becomes the returned previous value, then this write wins.
            let previous = vm.heap.indexed_dict(id).value_at(slot).expect("located IndexedDict slot exists");
            let result = wrap_some(vm, previous)?;
            vm.heap
                .indexed_dict_mut(id)
                .set_value_at(slot, value)
                .ok_or_else(|| RuntimeError::Internal("IndexedDict slot from lookup was out of range".to_string()))?;
            Ok(result)
        }
        None => {
            vm.heap.indexed_dict_mut(id).insert_new(hash, key, value).map_err(mutation_error)?;
            Ok(vm.none_value())
        }
    }
}

#[phalcom_native_macros::primitive(IndexedDict, "rawKeyAt(_)" , visibility = internal)]
pub fn indexed_dict_raw_key_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_indexed_dict(vm, receiver)?;
    let index = slot_index(vm, &args[0])?;
    Ok(index.and_then(|index| vm.heap.indexed_dict(id).key_at(index)).unwrap_or(Value::nil()))
}

#[phalcom_native_macros::primitive(IndexedDict, "rawValueAt(_)" , visibility = internal)]
pub fn indexed_dict_raw_value_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_indexed_dict(vm, receiver)?;
    let index = slot_index(vm, &args[0])?;
    Ok(index.and_then(|index| vm.heap.indexed_dict(id).value_at(index)).unwrap_or(Value::nil()))
}

#[phalcom_native_macros::primitive(IndexedDict, "rawInsertAt(_,_,_)" , visibility = internal)]
pub fn indexed_dict_raw_insert_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_indexed_dict(vm, receiver)?;
    let key = args[0];
    let value = args[1];
    let Some(index) = slot_index(vm, &args[2])? else {
        return Ok(Value::bool(false));
    };
    if index > vm.heap.indexed_dict(id).len() {
        return Ok(Value::bool(false));
    }
    if is_mutable_collection_key(vm, &key) {
        return Err(mutable_key_error(vm, "IndexedDict").into());
    }
    let (hash, existing) = locate(vm, id, key)?;
    if existing.is_some() {
        return Ok(Value::bool(false));
    }
    vm.heap.indexed_dict_mut(id).insert_at(index, hash, key, value).map_err(mutation_error)?;
    Ok(Value::bool(true))
}

#[phalcom_native_macros::primitive(IndexedDict, "rawMove(_,_)" , visibility = internal)]
pub fn indexed_dict_raw_move(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_indexed_dict(vm, receiver)?;
    let Some(from) = slot_index(vm, &args[0])? else {
        return Ok(Value::bool(false));
    };
    let Some(to) = slot_index(vm, &args[1])? else {
        return Ok(Value::bool(false));
    };
    match vm.heap.indexed_dict_mut(id).move_at(from, to) {
        Ok(()) => Ok(Value::bool(true)),
        Err(MapMutationError::OutOfRange) => Ok(Value::bool(false)),
        Err(error) => Err(mutation_error(error).into()),
    }
}

#[phalcom_native_macros::primitive(IndexedDict, "rawRemoveAt(_)" , visibility = internal)]
pub fn indexed_dict_raw_remove_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_indexed_dict(vm, receiver)?;
    let Some(index) = slot_index(vm, &args[0])? else {
        return Ok(Value::bool(false));
    };
    match vm.heap.indexed_dict_mut(id).remove_at(index) {
        Ok(_) => Ok(Value::bool(true)),
        Err(MapMutationError::OutOfRange) => Ok(Value::bool(false)),
        Err(error) => Err(mutation_error(error).into()),
    }
}

#[phalcom_native_macros::primitive(IndexedDict, "rawIndexOfKey(_)" , visibility = internal)]
pub fn indexed_dict_raw_index_of_key(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_indexed_dict(vm, receiver)?;
    let (_, slot) = locate(vm, id, args[0])?;
    match slot {
        Some(slot) => Ok(wrap_some(vm, Value::int(i64::try_from(slot).expect("slot count fits Int")))?),
        None => Ok(vm.none_value()),
    }
}

#[phalcom_native_macros::primitive(IndexedDict, "rawClear", visibility = internal)]
pub fn indexed_dict_raw_clear(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = expect_indexed_dict(vm, receiver)?;
    vm.heap.indexed_dict_mut(id).clear().map_err(mutation_error)?;
    Ok(Value::nil())
}
