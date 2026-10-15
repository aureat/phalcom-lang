//! Native primitives for persistent immutable multiset (`Bag<T>`).

use crate::error::{PhResult, RuntimeError};
use crate::primitive::int::expect_int_big;
use crate::primitive::persistent_hash;
use crate::primitive::{expect_bag, is_mutable_collection_key, mutable_key_error};
use crate::value::{Value, normalize_bigint};
use crate::vm::VM;
use num_bigint::BigInt;
use num_traits::{Signed, Zero};

fn check_count_policy(vm: &mut VM, count: &BigInt) -> PhResult<()> {
    let limit = vm.numeric_policy.max_integer_bits.unwrap_or(8_388_608);
    if count.bits() as usize > limit {
        return Err(vm.raise_numeric_error(RuntimeError::NumericLimit("Bag count exceeds configured bit limit".to_string())));
    }
    Ok(())
}

#[phalcom_native_macros::primitive(Bag, "new()", side = class, types = "() -> Self", flow = receiver)]
pub fn bag_class_new(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::obj(vm.heap.alloc_bag()))
}

#[phalcom_native_macros::primitive(Bag, "rawSize", visibility = internal)]
pub fn bag_raw_size(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = expect_bag(vm, receiver)?;
    let total = vm.heap.bag(id).total_count().clone();
    Ok(normalize_bigint(total, &mut vm.heap))
}

#[phalcom_native_macros::primitive(Bag, "rawDistinctCount", visibility = internal)]
pub fn bag_raw_distinct_count(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = expect_bag(vm, receiver)?;
    let count = i64::try_from(vm.heap.bag(id).distinct_count()).map_err(|_| RuntimeError::NumericLimit("Bag distinct count exceeds Int range".to_string()))?;
    Ok(Value::int(count))
}

#[phalcom_native_macros::primitive(Bag, "rawCount(_)", visibility = internal)]
pub fn bag_raw_count(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_bag(vm, receiver)?;
    let root = vm.heap.bag(id).root();
    match persistent_hash::lookup(vm, root, args[0])? {
        Some(val) => Ok(val),
        None => Ok(Value::int(0)),
    }
}

#[phalcom_native_macros::primitive(Bag, "rawAdding(_,_)", visibility = internal)]
pub fn bag_raw_adding(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let bag_id = expect_bag(vm, receiver)?;
    let key = args[0];
    if is_mutable_collection_key(vm, &key) {
        return Err(mutable_key_error(vm, "Bag").into());
    }
    let count = expect_int_big(&args[1], vm)?;
    if count.is_negative() {
        return Err(RuntimeError::ArgumentError("count cannot be negative".to_string()).into());
    }
    if count.is_zero() {
        return Ok(*receiver);
    }
    let (root, total_count, distinct_count) = {
        let b = vm.heap.bag(bag_id);
        (b.root(), b.total_count().clone(), b.distinct_count())
    };
    check_count_policy(vm, &(&total_count + &count))?;

    let mut plan = persistent_hash::prepare_insert(vm, root, key, Value::nil())?;
    let existing_count = match plan.old_payload {
        Some(v) => expect_int_big(&v, vm)?,
        None => BigInt::zero(),
    };
    let new_count = &existing_count + &count;
    check_count_policy(vm, &new_count)?;
    plan.payload = normalize_bigint(new_count, &mut vm.heap);

    let (new_root, _) = persistent_hash::commit_insert(&mut vm.heap, plan);
    let new_distinct = if existing_count.is_zero() { distinct_count + 1 } else { distinct_count };
    let new_total = total_count + count;
    let new_bag = vm.heap.alloc_bag_with(Some(new_root), new_total, new_distinct);
    Ok(Value::obj(new_bag))
}

#[phalcom_native_macros::primitive(Bag, "rawRemoving(_,_)", visibility = internal)]
pub fn bag_raw_removing(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let bag_id = expect_bag(vm, receiver)?;
    let key = args[0];
    if is_mutable_collection_key(vm, &key) {
        return Err(mutable_key_error(vm, "Bag").into());
    }
    let count = expect_int_big(&args[1], vm)?;
    if count.is_negative() {
        return Err(RuntimeError::ArgumentError("count cannot be negative".to_string()).into());
    }
    if count.is_zero() {
        return Ok(*receiver);
    }
    let (root, total_count, distinct_count) = {
        let b = vm.heap.bag(bag_id);
        (b.root(), b.total_count().clone(), b.distinct_count())
    };
    let Some(root_ref) = root else {
        return Ok(*receiver);
    };

    let rem_plan = persistent_hash::prepare_remove(vm, Some(root_ref), key)?;
    let Some(old_val) = rem_plan.removed_payload else {
        return Ok(*receiver);
    };
    let existing_count = expect_int_big(&old_val, vm)?;
    if count >= existing_count {
        let (new_root, _) = persistent_hash::commit_remove(&mut vm.heap, rem_plan);
        let new_distinct = distinct_count.saturating_sub(1);
        let new_total = total_count - &existing_count;
        let new_bag = vm.heap.alloc_bag_with(new_root, new_total, new_distinct);
        Ok(Value::obj(new_bag))
    } else {
        let new_count = &existing_count - &count;
        let new_payload = normalize_bigint(new_count, &mut vm.heap);
        let new_root = persistent_hash::commit_replace_payload(&mut vm.heap, rem_plan, new_payload);
        let new_total = total_count - count;
        let new_bag = vm.heap.alloc_bag_with(new_root, new_total, distinct_count);
        Ok(Value::obj(new_bag))
    }
}

#[phalcom_native_macros::primitive(Bag, "rawKeys", visibility = internal)]
pub fn bag_raw_keys(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = expect_bag(vm, receiver)?;
    let root = vm.heap.bag(id).root();
    let mut keys = Vec::new();
    persistent_hash::collect_keys(&vm.heap, root, &mut keys);
    let arr = vm.heap.alloc_array(keys);
    Ok(Value::obj(arr))
}
