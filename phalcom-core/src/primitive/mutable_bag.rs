//! Native storage and exact-count operations for MutableBag.

use crate::error::{MapMutationError, PhResult, RuntimeError};
use crate::heap::ObjRef;
use crate::primitive::int::expect_int_big;
use crate::primitive::ordered_hash::{self, KeyedCollectionKind};
use crate::primitive::{expect_mutable_bag, is_mutable_collection_key, mutable_key_error};
use crate::value::{Value, normalize_bigint};
use crate::vm::VM;
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use num_traits::{Signed, Zero};

fn mutation_error(error: MapMutationError) -> RuntimeError {
    match error {
        MapMutationError::Locked => RuntimeError::ConcurrentMutation { collection: "MutableBag" },
        MapMutationError::OutOfRange => RuntimeError::Internal("MutableBag slot was out of range".to_string()),
    }
}

fn check_count_policy(vm: &mut VM, count: &BigInt) -> PhResult<()> {
    let limit = vm.numeric_policy.max_integer_bits.unwrap_or(8_388_608);
    if count.bits() as usize > limit {
        return Err(vm.raise_numeric_error(RuntimeError::NumericLimit("MutableBag count exceeds configured bit limit".to_string())));
    }
    Ok(())
}

fn locate(vm: &mut VM, id: ObjRef, value: Value) -> PhResult<(i64, Option<usize>)> {
    ordered_hash::locate_key(vm, id, value, KeyedCollectionKind::MutableBag)
}

#[phalcom_native_macros::primitive(MutableBag, "new()", side = class, types = "() -> Self", flow = receiver)]
pub fn mutable_bag_class_new(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::obj(vm.heap.alloc_mutable_bag()))
}

#[phalcom_native_macros::primitive(MutableBag, "rawSize", visibility = internal)]
pub fn mutable_bag_raw_size(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = expect_mutable_bag(vm, receiver)?;
    let total = vm.heap.mutable_bag(id).total_count().clone();
    Ok(normalize_bigint(total, &mut vm.heap))
}

#[phalcom_native_macros::primitive(MutableBag, "rawDistinctCount", visibility = internal)]
pub fn mutable_bag_raw_distinct_count(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = expect_mutable_bag(vm, receiver)?;
    let count =
        i64::try_from(vm.heap.mutable_bag(id).len()).map_err(|_| RuntimeError::NumericLimit("MutableBag distinct count exceeds Int range".to_string()))?;
    Ok(Value::int(count))
}

#[phalcom_native_macros::primitive(MutableBag, "rawCount(_)" , visibility = internal)]
pub fn mutable_bag_raw_count(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_mutable_bag(vm, receiver)?;
    let (_, slot) = locate(vm, id, args[0])?;
    let count = slot.and_then(|slot| vm.heap.mutable_bag(id).count_at(slot)).unwrap_or_else(BigInt::zero);
    Ok(normalize_bigint(count, &mut vm.heap))
}

#[phalcom_native_macros::primitive(MutableBag, "rawAdd(_,_)" , visibility = internal)]
pub fn mutable_bag_raw_add(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_mutable_bag(vm, receiver)?;
    let key = args[0];
    let amount = expect_int_big(&args[1], vm)?;
    if amount.is_negative() {
        return Err(RuntimeError::ArgumentError("MutableBag count must be nonnegative".to_string()).into());
    }
    if amount.is_zero() {
        return Ok(*receiver);
    }
    if is_mutable_collection_key(vm, &key) {
        return Err(mutable_key_error(vm, "MutableBag").into());
    }
    check_count_policy(vm, &amount)?;
    let (bucket, slot) = locate(vm, id, key)?;
    let old_count = slot.and_then(|slot| vm.heap.mutable_bag(id).count_at(slot)).unwrap_or_else(BigInt::zero);
    let count = old_count + &amount;
    let total = vm.heap.mutable_bag(id).total_count() + &amount;
    check_count_policy(vm, &count)?;
    check_count_policy(vm, &total)?;
    match slot {
        Some(slot) => vm.heap.mutable_bag_mut(id).set_count(slot, count, total).map_err(mutation_error)?,
        None => vm.heap.mutable_bag_mut(id).insert_new(bucket, key, amount, total).map_err(mutation_error)?,
    }
    Ok(*receiver)
}

#[phalcom_native_macros::primitive(MutableBag, "rawRemove(_,_)" , visibility = internal)]
pub fn mutable_bag_raw_remove(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_mutable_bag(vm, receiver)?;
    let amount = expect_int_big(&args[1], vm)?;
    if amount.is_negative() {
        return Err(RuntimeError::ArgumentError("MutableBag count must be nonnegative".to_string()).into());
    }
    if amount.is_zero() {
        return Ok(*receiver);
    }
    let (_, slot) = locate(vm, id, args[0])?;
    if let Some(slot) = slot {
        let old_count = vm.heap.mutable_bag(id).count_at(slot).expect("located MutableBag slot exists");
        let removed = if amount < old_count { amount } else { old_count.clone() };
        let remaining = &old_count - &removed;
        let total = vm.heap.mutable_bag(id).total_count() - &removed;
        if remaining.is_zero() {
            vm.heap.mutable_bag_mut(id).remove_at(slot, total).map_err(mutation_error)?;
        } else {
            vm.heap.mutable_bag_mut(id).set_count(slot, remaining, total).map_err(mutation_error)?;
        }
    }
    Ok(*receiver)
}

#[phalcom_native_macros::primitive(MutableBag, "rawAt(_)" , visibility = internal)]
pub fn mutable_bag_raw_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_mutable_bag(vm, receiver)?;
    let index = expect_int_big(&args[0], vm)?.to_usize();
    Ok(index.and_then(|index| vm.heap.mutable_bag(id).key_at(index)).unwrap_or(Value::nil()))
}

#[phalcom_native_macros::primitive(MutableBag, "rawCountAt(_)" , visibility = internal)]
pub fn mutable_bag_raw_count_at(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_mutable_bag(vm, receiver)?;
    let index = expect_int_big(&args[0], vm)?.to_usize();
    let count = index.and_then(|index| vm.heap.mutable_bag(id).count_at(index)).unwrap_or_else(BigInt::zero);
    Ok(normalize_bigint(count, &mut vm.heap))
}

#[phalcom_native_macros::primitive(MutableBag, "rawClear", visibility = internal)]
pub fn mutable_bag_raw_clear(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = expect_mutable_bag(vm, receiver)?;
    vm.heap.mutable_bag_mut(id).clear().map_err(mutation_error)?;
    Ok(Value::nil())
}
