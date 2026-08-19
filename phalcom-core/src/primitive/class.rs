//! Native primitives on `Class`.

use crate::error::PhResult;
use crate::error::RuntimeError;
use crate::heap::InstanceObject;
use crate::heap::Object;
use crate::heap::lookup_method_in_hierarchy;
use crate::method::{ArgumentView, CallOutcome};
use crate::primitive::expect_class;
use crate::value::Value;
use crate::vm::VM;

/// Signature: `Class::superclass` — returns the receiver's superclass, or
#[phalcom_native_macros::primitive(
    Behavior,
    "superclass",
    params = [],
    returns = "Option<Class>",
    types = "() -> Option<Class>",
    effects = pure
)]
pub fn class_superclass(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let class_id = expect_class(vm, receiver)?;
    match vm.heap.class(class_id).superclass {
        Some(superclass) => Ok(Value::obj(superclass)),
        None => Ok(vm.none_value()),
    }
}

/// Signature: `Class::superclass=(_)` — always an error; the tower is fixed here.
#[phalcom_native_macros::primitive(
    Behavior,
    "superclass=(put)",
    params = [Object],
    returns = Nothing,
    types = "(Object) -> Nothing",
    flow = never
)]
pub fn class_set_superclass(_vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Err(RuntimeError::InvalidSetSuper.into())
}

/// Signature: `Behavior::name` — the receiver class's OWN display name.
#[phalcom_native_macros::primitive(
    Behavior,
    "name",
    params = [],
    returns = String,
    types = "() -> String",
    effects = pure
)]
pub fn behavior_name(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let class_id = expect_class(vm, receiver)?;
    let name = vm.heap.class(class_id).name.clone();
    Ok(vm.alloc_string_value(name))
}

/// Signature: `Behavior::methods` — a fresh [`List`](crate::heap::ListObject)
#[phalcom_native_macros::primitive(
    Behavior,
    "methods",
    params = [],
    returns = List,
    types = "() -> List",
    effects = pure
)]
pub fn behavior_methods(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let class_id = expect_class(vm, receiver)?;
    let selectors: Vec<Value> = vm.heap.class(class_id).methods.keys().map(|selector| Value::symbol(*selector)).collect();
    Ok(Value::obj(vm.heap.alloc_list(selectors)))
}

/// Signature: `Behavior#>>(_)` — extracts either one effective exact `Method`
/// or an immutable `MethodFamily` snapshot from a selector-spec value.
///
/// This remains an ordinary polymorphic operator. `Int#>>(_)` keeps its
/// arithmetic meaning; only behavior receivers reach this reflection method.
///
/// # Errors
///
/// Returns [`RuntimeError::Arity`] for a missing selector-spec, a visibility
/// error for an inaccessible effective method, or [`RuntimeError::Type`] for a
/// non-`Symbol`/non-`SelectorPattern` spec.
pub fn behavior_extract(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let behavior = expect_class(vm, receiver)?;
    let rhs = args.first().ok_or(RuntimeError::Arity {
        signature: ">>",
        expected: 1,
        found: 0,
    })?;
    let caller_authority = (vm.current_access_class(), vm.current_has_internal_privilege());
    behavior_extract_as(vm, behavior, rhs, caller_authority)
}

/// Shape-aware gateway for `Behavior#>>(_)`. It preserves the authority of the
/// caller that initiated reflection instead of treating the Behavior primitive
/// itself as the caller.
pub fn behavior_extract_shape(vm: &mut VM, receiver: Value, args: ArgumentView) -> PhResult<CallOutcome> {
    let behavior = expect_class(vm, &receiver)?;
    let rhs = args.positional(vm, 0).ok_or_else(|| RuntimeError::Arity {
        signature: ">>",
        expected: 1,
        found: args.positional_count(),
    })?;
    let value = behavior_extract_as(vm, behavior, &rhs, args.caller_authority())?;
    Ok(CallOutcome::Returned(value))
}

fn behavior_extract_as(vm: &mut VM, behavior: crate::heap::ClassId, rhs: &Value, caller_authority: (Option<crate::heap::ClassId>, bool)) -> PhResult<Value> {
    if let Some(selector) = rhs.symbol_value() {
        match lookup_method_in_hierarchy(&vm.heap, behavior, selector) {
            Some(method) => {
                vm.authorize_method_access_as(method, caller_authority.0, caller_authority.1)?;
                Ok(Value::obj(method))
            }
            None => Ok(vm.none_value()),
        }
    } else if let Some(pattern) = rhs.as_obj() {
        if matches!(vm.heap.get(pattern), Object::SelectorPattern(_)) {
            let family = vm.capture_method_family(behavior, pattern, caller_authority)?;
            Ok(Value::obj(vm.heap.alloc(Object::MethodFamily(Box::new(family)))))
        } else {
            Err(RuntimeError::Type {
                expected: "Symbol or SelectorPattern",
                found: rhs.type_name(),
            }
            .into())
        }
    } else {
        Err(RuntimeError::Type {
            expected: "Symbol or SelectorPattern",
            found: rhs.type_name(),
        }
        .into())
    }
}

/// Signature: `Class::+(_)` — concatenates the two classes' names into a string.
#[phalcom_native_macros::primitive(
    Class,
    "+(_)",
    params = [Object],
    returns = String,
    types = "(Object) -> String",
    effects = pure
)]
pub fn class_add(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let this = expect_class(vm, receiver)?;
    let other = expect_class(vm, &args[0])?;
    let name = format!("{}{}", vm.heap.class(this).name, vm.heap.class(other).name);
    Ok(vm.alloc_string_value(name))
}

/// Signature: `Class::_$new()` — allocates a bare instance of the receiver class.
#[phalcom_native_macros::primitive(
    Class,
    "_$new()",
    params = [],
    returns = Object,
    types = "() -> Object",
    visibility = internal
)]
pub fn class_new_(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let class_id = expect_class(vm, receiver)?;
    let target_class = vm.heap.class(class_id);
    if target_class.is_abstract {
        let error_cls = vm.universe.classes.error_class;
        let field_count = vm.heap.class(error_cls).field_count;
        let mut inst = InstanceObject::new(error_cls, field_count);
        let msg = format!("cannot instantiate abstract class {}", target_class.name);
        inst.slots[0] = vm.alloc_string_value(msg.clone());
        let kind_sym = vm.get_or_intern("abstractClass");
        inst.slots[1] = Value::symbol(kind_sym);
        let err_obj = vm.heap.alloc(Object::Instance(inst));
        return Err(RuntimeError::Raise {
            error: Value::obj(err_obj),
            rendered: msg,
            traceback: None,
            help: None,
        }
        .into());
    }
    if target_class.native_repr {
        return Err(RuntimeError::Type {
            expected: "InstanceObject-backed class",
            found: "native representation class",
        }
        .into());
    }
    let field_count = target_class.field_count;
    let instance = InstanceObject::new(class_id, field_count);
    Ok(Value::obj(vm.heap.alloc(Object::Instance(instance))))
}
