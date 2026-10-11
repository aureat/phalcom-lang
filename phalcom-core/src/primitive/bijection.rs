//! Native primitives for persistent bidirectional mapping (`Bijection<K, V>`).

use crate::error::{PhResult, RuntimeError};
use crate::primitive::option::wrap_some;
use crate::primitive::ordered_hash::keys_equal;
use crate::primitive::persistent_hash;
use crate::primitive::{expect_bijection, expect_class, is_mutable_collection_key, mutable_key_error};
use crate::value::Value;
use crate::vm::VM;

fn bijection_conflict_error(vm: &mut VM, message: String) -> RuntimeError {
    let class_name = vm.interner.intern("BijectionConflictError");
    let binding = vm
        .prelude_bindings
        .get(&class_name)
        .copied()
        .expect("BijectionConflictError is defined by Universe sources");
    let class_value = vm
        .heap
        .module(binding.module)
        .get_by_slot(binding.slot as usize)
        .expect("BijectionConflictError prelude binding is initialized");
    let class_id = expect_class(vm, &class_value).expect("BijectionConflictError prelude binding is a class");
    let field_count = vm.heap.class(class_id).field_count;
    let mut instance = crate::heap::InstanceObject::new(class_id, field_count);
    instance.slots[0] = vm.alloc_string_value(message.clone());
    let error = Value::obj(vm.heap.alloc(crate::heap::Object::Instance(instance)));
    RuntimeError::Raise {
        error,
        rendered: message,
        traceback: None,
        help: None,
    }
}

#[phalcom_native_macros::primitive(Bijection, "new()", side = class, types = "() -> Self", flow = receiver)]
pub fn bijection_class_new(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::obj(vm.heap.alloc_bijection()))
}

#[phalcom_native_macros::primitive(Bijection, "rawSize", visibility = internal)]
pub fn bijection_raw_size(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = expect_bijection(vm, receiver)?;
    let len = vm.heap.bijection(id).len();
    let count = i64::try_from(len)
        .map_err(|_| RuntimeError::NumericLimit("Bijection size exceeds Int range".to_string()))?;
    Ok(Value::int(count))
}

#[phalcom_native_macros::primitive(Bijection, "rawGet(_)", visibility = internal)]
pub fn bijection_raw_get(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_bijection(vm, receiver)?;
    let root = vm.heap.bijection(id).forward_root();
    match persistent_hash::lookup(vm, root, args[0])? {
        Some(val) => Ok(wrap_some(vm, val)?),
        None => Ok(vm.none_value()),
    }
}

#[phalcom_native_macros::primitive(Bijection, "rawAssociate(_,_)", visibility = internal)]
pub fn bijection_raw_associate(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_bijection(vm, receiver)?;
    let key = args[0];
    let val = args[1];

    if is_mutable_collection_key(vm, &key) {
        return Err(mutable_key_error(vm, "Bijection").into());
    }
    if is_mutable_collection_key(vm, &val) {
        return Err(mutable_key_error(vm, "Bijection").into());
    }

    let (forward_root, reverse_root, len) = {
        let b = vm.heap.bijection(id);
        (b.forward_root(), b.reverse_root(), b.len())
    };

    let fwd_plan = persistent_hash::prepare_insert(vm, forward_root, key, val)?;
    let rev_plan = persistent_hash::prepare_insert(vm, reverse_root, val, key)?;

    if let Some(existing_val) = fwd_plan.old_payload {
        if keys_equal(vm, existing_val, val)? {
            if let Some(existing_key) = rev_plan.old_payload {
                if keys_equal(vm, existing_key, key)? {
                    return Ok(*receiver);
                }
            }
        }
        let rendered_key = key.to_string(vm);
        return Err(bijection_conflict_error(
            vm,
            format!("Key '{rendered_key}' is already associated with another value in Bijection"),
        )
        .into());
    }

    if let Some(_existing_key) = rev_plan.old_payload {
        let rendered_val = val.to_string(vm);
        return Err(bijection_conflict_error(
            vm,
            format!("Value '{rendered_val}' is already associated with another key in Bijection"),
        )
        .into());
    }

    let (new_fwd, _) = persistent_hash::commit_insert(&mut vm.heap, fwd_plan);
    let (new_rev, _) = persistent_hash::commit_insert(&mut vm.heap, rev_plan);
    let new_len = len + 1;
    let new_id = vm.heap.alloc_bijection_with(Some(new_fwd), Some(new_rev), new_len);
    Ok(Value::obj(new_id))
}

#[phalcom_native_macros::primitive(Bijection, "rawRemoveKey(_)", visibility = internal)]
pub fn bijection_raw_remove_key(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let id = expect_bijection(vm, receiver)?;
    let key = args[0];
    if is_mutable_collection_key(vm, &key) {
        return Err(mutable_key_error(vm, "Bijection").into());
    }

    let (forward_root, reverse_root, len) = {
        let b = vm.heap.bijection(id);
        (b.forward_root(), b.reverse_root(), b.len())
    };

    let Some(root_ref) = forward_root else {
        return Ok(*receiver);
    };

    let Some(val) = persistent_hash::lookup(vm, Some(root_ref), key)? else {
        return Ok(*receiver);
    };

    let fwd_rem = persistent_hash::prepare_remove(vm, forward_root, key)?;
    let rev_rem = persistent_hash::prepare_remove(vm, reverse_root, val)?;

    let (new_fwd, _) = persistent_hash::commit_remove(&mut vm.heap, fwd_rem);
    let (new_rev, _) = persistent_hash::commit_remove(&mut vm.heap, rev_rem);
    let new_len = len.saturating_sub(1);
    let new_id = vm.heap.alloc_bijection_with(new_fwd, new_rev, new_len);
    Ok(Value::obj(new_id))
}

#[phalcom_native_macros::primitive(Bijection, "rawInverse", visibility = internal)]
pub fn bijection_raw_inverse(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = expect_bijection(vm, receiver)?;
    let (forward_root, reverse_root, len) = {
        let b = vm.heap.bijection(id);
        (b.forward_root(), b.reverse_root(), b.len())
    };
    let inv = vm.heap.alloc_bijection_with(reverse_root, forward_root, len);
    Ok(Value::obj(inv))
}

#[phalcom_native_macros::primitive(Bijection, "rawKeys", visibility = internal)]
pub fn bijection_raw_keys(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let id = expect_bijection(vm, receiver)?;
    let root = vm.heap.bijection(id).forward_root();
    let mut keys = Vec::new();
    persistent_hash::collect_keys(&vm.heap, root, &mut keys);
    let arr = vm.heap.alloc_array(keys);
    Ok(Value::obj(arr))
}
