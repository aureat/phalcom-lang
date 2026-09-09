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
        vm.write_output(text.as_bytes())?;
    }
    vm.write_output(b"\n")?;
    vm.flush_output()?;
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
    vm.enqueue_unowned_fiber(fiber_ref)?;
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
    match vm.pop_public_scheduled() {
        Some(fiber_ref) => Ok(wrap_some(vm, Value::obj(fiber_ref))?),
        None => Ok(vm.none_value()),
    }
}

/// Internal scheduler dequeue. Unlike the legacy public getter, this keeps
/// the `Queued` reservation until the scheduler resume primitive consumes it.
#[phalcom_native_macros::primitive(System, "_$nextScheduled", side = class, visibility = internal)]
pub fn system_next_scheduled_internal(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    match vm.pop_next_queued() {
        Some(fiber_ref) => Ok(wrap_some(vm, Value::obj(fiber_ref))?),
        None => Ok(vm.none_value()),
    }
}

/// Internal Future wake. Only an exact `Parked(generation)` state can be
/// admitted, so stale and duplicate waiter entries are harmless no-ops.
#[phalcom_native_macros::primitive(System, "_$wake(_,_)", side = class, visibility = internal)]
pub fn system_wake(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let fiber_ref = args
        .first()
        .and_then(Value::as_obj)
        .filter(|id| vm.heap.as_fiber(*id).is_some())
        .ok_or_else(|| RuntimeError::Type {
            expected: "Fiber",
            found: args.first().map_or("missing", Value::type_name),
        })?;
    let generation = args.get(1).and_then(|value| value.as_int()).ok_or_else(|| RuntimeError::Type {
        expected: "Int",
        found: args.get(1).map_or("missing", Value::type_name),
    })?;
    Ok(Value::bool(vm.wake_parked_fiber(fiber_ref, generation)))
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
    vm.write_output(s.as_bytes())?;
    Ok(vm.unit_value())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::PhError;
    use crate::vm::{BufferedOutput, RuntimeOutput, VM};
    use std::io;

    struct FailingOutput;

    impl RuntimeOutput for FailingOutput {
        fn write(&mut self, _bytes: &[u8]) -> io::Result<()> {
            Err(io::Error::other("synthetic output failure"))
        }
    }

    #[test]
    fn raw_write_and_print_share_vm_output_sink() {
        let sink = BufferedOutput::new();
        let handle = sink.handle();
        let mut vm = VM::new_native_with_output(Box::new(sink));
        let receiver = Value::obj(vm.universe.classes.system_class);
        let raw = vm.alloc_string_value("raw".to_owned());

        system_raw_write(&mut vm, &receiver, &[raw]).expect("raw write succeeds");
        system_class_print(&mut vm, &receiver, &[Value::int(1)]).expect("print succeeds");

        assert_eq!(handle.bytes(), b"raw1\n");
    }

    #[test]
    fn output_sink_failure_returns_io_error() {
        let mut vm = VM::new_native_with_output(Box::new(FailingOutput));
        let receiver = Value::obj(vm.universe.classes.system_class);

        let error = system_class_print(&mut vm, &receiver, &[Value::int(1)]).expect_err("output failure must be returned");
        assert!(matches!(error, PhError::Io(_)));
    }
}
