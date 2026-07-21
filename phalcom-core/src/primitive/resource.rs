use crate::error::{PhResult, RuntimeError};
use crate::primitive::expect_string;
use crate::resource::{ResourceHandle, ResourceKind};
use crate::value::Value;
use crate::vm::VM;

/// `Resource.register_(_)` static native primitive.
pub fn resource_register(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let kind_str = expect_string(vm, &args[0])?;
    let site = None;
    let handle = vm.resources.open(ResourceKind::Custom(kind_str), site);
    let packed = ResourceHandle::pack(handle.index, handle.generation);
    Ok(Value::Number(packed))
}

/// Helper to extract ResourceHandle from an Instance instance slot 0.
fn extract_handle(vm: &VM, instance_val: &Value) -> PhResult<ResourceHandle> {
    if let Value::Obj(obj_ref) = *instance_val {
        let heap_obj = vm.heap.get(obj_ref);
        if let crate::heap::Object::Instance(inst) = heap_obj {
            if let Some(val) = inst.slots.first() {
                if let Value::Number(n) = val {
                    return Ok(ResourceHandle::unpack(*n));
                }
            }
        }
    }
    Err(RuntimeError::Type {
        expected: "Resource instance",
        found: instance_val.type_name(),
    }
    .into())
}

/// Helper to raise a UseAfterCloseError surface error.
fn raise_use_after_close(vm: &mut VM, message: &str) -> PhResult<Value> {
    let msg_obj = vm.heap.alloc_string(message.to_string());
    let uace_cls = vm.universe.classes.use_after_close_error_class;
    let mut inst = crate::heap::InstanceObject::new(uace_cls, 4);
    inst.slots[0] = Value::Obj(msg_obj);
    let err_obj = vm.heap.alloc(crate::heap::Object::Instance(inst));
    let err_val = Value::Obj(err_obj);
    Err(RuntimeError::Raise {
        error: err_val,
        rendered: message.to_string(),
        traceback: None,
        help: None,
    }
    .into())
}

/// `Resource#close_` primitive.
pub fn resource_raw_close(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let handle = extract_handle(vm, receiver)?;
    match vm.resources.close(handle) {
        Ok(()) => Ok(Value::Obj(vm.universe.classes.none_singleton)),
        Err(crate::resource::ResourceError::AlreadyClosed) => Ok(Value::Obj(vm.universe.classes.none_singleton)),
        Err(crate::resource::ResourceError::StaleHandle) => {
            raise_use_after_close(vm, "Resource already closed or stale")
        }
    }
}

/// `Resource#isClosed_` primitive.
pub fn resource_raw_is_closed(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let handle = extract_handle(vm, receiver)?;
    let closed = vm.resources.is_closed(handle);
    Ok(Value::Bool(closed))
}

/// `System.leakReport_` primitive.
pub fn system_leak_report(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let leaks = vm.resources.leaks_detail();
    let mut str_vals = Vec::new();
    for (_idx, kind, site) in leaks {
        let site_str = site
            .map(|s| format!("{}:{}", s.start, s.end))
            .unwrap_or_else(|| "unknown".to_string());
        let line = format!("Unclosed resource kind: {} opened at {}", kind, site_str);
        let str_obj = vm.heap.alloc_string(line);
        str_vals.push(Value::Obj(str_obj));
    }
    let list_obj = vm.heap.alloc_list(str_vals);
    Ok(Value::Obj(list_obj))
}

/// `System.strictResources_(_)` primitive.
pub fn system_strict_resources(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let flag = match &args[0] {
        Value::Bool(b) => *b,
        other => return Err(RuntimeError::Type { expected: "Bool", found: other.type_name() }.into()),
    };
    vm.strict_resources = flag;
    Ok(Value::Obj(vm.universe.classes.none_singleton))
}
