//! Native primitives on `System`.

use crate::error::{PhResult, RuntimeError};
use crate::primitive::option::wrap_some;
use crate::value::Value;
use crate::vm::VM;

/// Signature: `System.class::print(_)` — prints its arguments, then a newline,
/// and returns the canonical `Unit` value.
#[phalcom_native_macros::primitive(
    System,
    "print(_)",
    params = [Object],
    returns = Unit,
    types = "(Object) -> Unit",
    side = class
)]
pub fn system_class_print(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    for arg in args {
        let text = arg.to_display_string(vm)?;
        print!("{text}");
    }
    println!();
    Ok(vm.unit_value())
}

/// Signature: `System.class::new()` — always an error; `System` is not instantiable.
#[phalcom_native_macros::primitive(
    System,
    "new()",
    params = [],
    returns = Never,
    types = "() -> Never",
    side = class,
    flow = never
)]
pub fn system_class_new(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Err(RuntimeError::NotAllowed("System instances cannot be created".to_string()).into())
}

/// Signature: `System::schedule(_)` — wraps `args[0]` (a `Function`) as a
#[phalcom_native_macros::primitive(
    System,
    "schedule(_)",
    params = [Object],
    returns = Fiber,
    types = "(Object) -> Fiber",
    side = class
)]
pub fn system_schedule(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let fiber_ref = if let Some(id) = args[0].as_obj() {
        if matches!(vm.heap.get(id), crate::heap::Object::Fiber(_)) {
            id
        } else {
            crate::primitive::fiber::new_fiber_ref(vm, args[0])?
        }
    } else {
        crate::primitive::fiber::new_fiber_ref(vm, args[0])?
    };
    vm.ready_queue.push_back(fiber_ref);
    Ok(Value::obj(fiber_ref))
}

/// Signature: `System::nextScheduled` — pops and returns the next queued
#[phalcom_native_macros::primitive(
    System,
    "nextScheduled",
    params = [],
    returns = "Option<Fiber>",
    types = "() -> Option<Fiber>",
    side = class
)]
pub fn system_next_scheduled(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    match vm.ready_queue.pop_front() {
        Some(fiber_ref) => Ok(wrap_some(vm, Value::obj(fiber_ref))?),
        None => Ok(vm.none_value()),
    }
}

/// Signature: `System.gc` — forces one full mark-sweep and returns `Unit`.
#[phalcom_native_macros::primitive(
    System,
    "gc",
    params = [],
    returns = Unit,
    types = "() -> Unit",
    side = class
)]
pub fn system_gc(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    vm.force_gc();
    Ok(vm.unit_value())
}

/// Signature: `System._$write(_)` — raw stdout write of an already-formed `String`.
#[phalcom_native_macros::primitive(
    System,
    "_$write(_)",
    params = [String],
    returns = Unit,
    types = "(String) -> Unit",
    side = class,
    visibility = internal
)]
pub fn system_raw_write(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let s = if let Some(id) = args[0].as_obj() {
        if vm.heap.as_string(id).is_some() {
            vm.heap.string(id).as_str().to_string()
        } else {
            return Err(RuntimeError::Type {
                expected: "String",
                found: args[0].type_name(),
            }
            .into());
        }
    } else {
        return Err(RuntimeError::Type {
            expected: "String",
            found: args[0].type_name(),
        }
        .into());
    };
    print!("{s}");
    Ok(vm.unit_value())
}
