pub mod attribute;
pub mod block;
pub mod boolean;
pub mod bytes;
pub mod class;
pub mod error;
pub mod family;
pub mod fiber;
pub mod float;
pub mod index;
pub mod int;
pub mod list;
pub mod map;
pub mod method;
pub mod method_family;
pub mod module;
pub mod nil;
pub mod number;
pub mod object;
pub mod range;
pub mod record;
pub mod reflection;
pub mod resource;
pub mod selector;
pub mod selector_pattern;
pub mod set;
pub mod string;
pub mod symbol;
pub mod system;
pub mod tuple;
pub mod typing;

#[non_exhaustive]
pub struct Sig;

#[allow(non_upper_case_globals)]
impl Sig {
    pub const ADD: &'static str = "+(_)";
    pub const SUB: &'static str = "-(_)";
    pub const MUL: &'static str = "*(_)";
    pub const DIV: &'static str = "/(_)";
    pub const EQ: &'static str = "==(_)";
    pub const LT: &'static str = "<(_)";
    pub const LE: &'static str = "<=(_)";
    pub const GT: &'static str = ">(_)";
    pub const GE: &'static str = ">=(_)";
    pub const AND: &'static str = "and(_)";
    pub const OR: &'static str = "or(_)";

    pub const NEG: &'static str = "-";
    pub const NOT: &'static str = "not";

    pub const name: &'static str = "name";
    pub const name_set: &'static str = "name=(put)";
    pub const class: &'static str = "class";
    pub const class_set: &'static str = "class=(put)";
    pub const superclass: &'static str = "superclass";
    pub const superclass_set: &'static str = "superclass=(put)";

    pub const toString: &'static str = "toString";
    pub const toNumber: &'static str = "toNumber";
    pub const toBool: &'static str = "toBool";
    pub const toDebug: &'static str = "toDebug";

    pub const new: &'static str = "new()";
    pub const new_1: &'static str = "new(_)";
    pub const new_2: &'static str = "new(_,_)";
}

#[non_exhaustive]
pub struct ClassName;

#[allow(non_upper_case_globals)]
impl ClassName {
    pub const Nil: &'static str = "Nil";
    pub const Bool: &'static str = "Bool";
    /// The `True` class — concrete singleton subclass of `Bool` and surface
    /// class of the `true` immediate ([ADR-0004]). Distinct from
    /// [`ObjectName::True`], which is the lowercase value spelling `"true"`.
    ///
    /// [ADR-0004]: ../../../docs/adr/accepted/0004-boolean-as-abstract-bool-with-true-false.md
    pub const True: &'static str = "True";
    /// The `False` class — concrete singleton subclass of `Bool` and surface
    /// class of the `false` immediate ([ADR-0004]). Distinct from
    /// [`ObjectName::False`], which is the lowercase value spelling `"false"`.
    ///
    /// [ADR-0004]: ../../../docs/adr/accepted/0004-boolean-as-abstract-bool-with-true-false.md
    pub const False: &'static str = "False";
    pub const Number: &'static str = "Number";
    pub const String: &'static str = "String";
    pub const Symbol: &'static str = "Symbol";
    pub const System: &'static str = "System";
    pub const Module: &'static str = "Module";
    pub const Object: &'static str = "Object";
    pub const Class: &'static str = "Class";
    pub const Metaclass: &'static str = "Metaclass";
    pub const Method: &'static str = "Method";
    pub const List: &'static str = "List";
    pub const Range: &'static str = "Range";
    pub const Map: &'static str = "Map";
    pub const Set: &'static str = "Set";
    pub const Tuple: &'static str = "Tuple";
    pub const Unit: &'static str = "Unit";
    pub const Record: &'static str = "Record";
    pub const Fiber: &'static str = "Fiber";
    pub const Future: &'static str = "Future";
}

#[non_exhaustive]
pub struct ObjectName;

#[allow(non_upper_case_globals)]
impl ObjectName {
    pub const Nil: &'static str = "nil";
    pub const True: &'static str = "true";
    pub const False: &'static str = "false";
}

/// Installs a native instance method `$func` on class `$class`.
///
/// Allocates a [`MethodObject`](crate::method::MethodObject) in the heap and
/// binds it under the encoded selector directly on `$class`.
macro_rules! primitive {
    ($vm:expr, $class:expr, $base:expr, $sig_kind: expr, $func:expr) => {
        debug_assert!(!$base.starts_with("_$"), "internal primitive selectors require `primitive_internal!`");
        let sig_str = crate::method::make_signature($base, $sig_kind);
        let symbol = $vm.get_or_intern(&sig_str);
        let method = MethodObject::new_primitive(symbol, $sig_kind, $func, $class);
        let method_id = $vm.heap.alloc(crate::heap::Object::Method(Box::new(method)));
        $vm.heap.class_mut($class).add_method(symbol, method_id);
        $vm.world_version += 1;
    };
}

/// Installs an exact-shape native primitive. Unlike `primitive!`, this keeps
/// the receiver/argument descriptor intact for a gateway that may enter a
/// bytecode frame without recursively driving the interpreter.
macro_rules! primitive_shape {
    ($vm:expr, $class:expr, $base:expr, $sig_kind:expr, $func:expr) => {
        debug_assert!(!$base.starts_with("_$"), "internal primitive selectors require `primitive_internal!`");
        let sig_str = crate::method::make_signature($base, $sig_kind);
        let symbol = $vm.get_or_intern(&sig_str);
        let method = MethodObject::new_shape_primitive(symbol, crate::method::Signature::new(symbol, $sig_kind), $func, $class);
        let method_id = $vm.heap.alloc(crate::heap::Object::Method(Box::new(method)));
        $vm.heap.class_mut($class).add_method(symbol, method_id);
        $vm.world_version += 1;
    };
}

/// Installs a shape-aware native rest primitive under its base-name family.
macro_rules! primitive_rest {
    ($vm:expr, $class:expr, $base:expr, $selector:expr, $sig_kind:expr, $arity:expr, $layout:expr, $func:expr) => {
        debug_assert!(!$base.starts_with("_$"), "internal primitive selectors require `primitive_internal!`");
        let symbol = $vm.get_or_intern($selector);
        let signature = crate::method::Signature::new_with_arity(symbol, $sig_kind, $arity, Some($layout));
        let method = MethodObject::new_shape_primitive(symbol, signature, $func, $class);
        let method_id = $vm.heap.alloc(crate::heap::Object::Method(Box::new(method)));
        $vm.heap.class_mut($class).add_method(symbol, method_id);
        let base_symbol = $vm.get_or_intern($base);
        $vm.heap.class_mut($class).add_rest_method(base_symbol, method_id);
        $vm.world_version += 1;
    };
}

/// Installs an implementation-only native instance method.
///
/// Internal selectors use the `_$` namespace and share the ordinary method
/// table with public methods. Their protection is therefore enforced by normal
/// dispatch visibility checks rather than by a parallel lookup mechanism.
macro_rules! primitive_internal {
    ($vm:expr, $class:expr, $base:expr, $sig_kind: expr, $func:expr) => {
        debug_assert!($base.starts_with("_$"), "internal primitive selectors must start with `_$`");
        let sig_str = crate::method::make_signature($base, $sig_kind);
        let symbol = $vm.get_or_intern(&sig_str);
        let mut method = MethodObject::new_primitive(symbol, $sig_kind, $func, $class);
        method.visibility = if $base == "_$attach" || $base == "_$freezeAttributes" || $base == "_$attributes" {
            crate::method::MemberVisibility::Public
        } else {
            crate::method::MemberVisibility::Internal
        };
        method.access_owner = Some($class);
        let method_id = $vm.heap.alloc(crate::heap::Object::Method(Box::new(method)));
        $vm.heap.class_mut($class).add_method(symbol, method_id);
        $vm.world_version += 1;
    };
}

/// Installs a native *static* method `$func` on class `$class`.
///
/// Like [`primitive!`], but binds the method on `$class`'s metaclass (`$class`'s
/// `class`), where static methods live.
macro_rules! primitive_static {
    ($vm:expr, $class:expr, $base:expr, $sig_kind: expr, $func:expr) => {
        debug_assert!(!$base.starts_with("_$"), "internal primitive selectors require `primitive_static_internal!`");
        let sig_str = crate::method::make_signature($base, $sig_kind);
        let symbol = $vm.get_or_intern(&sig_str);
        let method = MethodObject::new_primitive(symbol, $sig_kind, $func, $class);
        let method_id = $vm.heap.alloc(crate::heap::Object::Method(Box::new(method)));
        let meta = $vm.heap.class($class).class;
        $vm.heap.class_mut(meta).add_method(symbol, method_id);
        $vm.world_version += 1;
    };
}

/// Static counterpart to [`primitive_internal!`].
macro_rules! primitive_static_internal {
    ($vm:expr, $class:expr, $base:expr, $sig_kind: expr, $func:expr) => {
        debug_assert!($base.starts_with("_$"), "internal primitive selectors must start with `_$`");
        let sig_str = crate::method::make_signature($base, $sig_kind);
        let symbol = $vm.get_or_intern(&sig_str);
        let mut method = MethodObject::new_primitive(symbol, $sig_kind, $func, $class);
        method.visibility = crate::method::MemberVisibility::Internal;
        method.access_owner = Some($class);
        let method_id = $vm.heap.alloc(crate::heap::Object::Method(Box::new(method)));
        let meta = $vm.heap.class($class).class;
        $vm.heap.class_mut(meta).add_method(symbol, method_id);
        $vm.world_version += 1;
    };
}

pub(crate) use primitive;
pub(crate) use primitive_internal;
pub(crate) use primitive_rest;
pub(crate) use primitive_shape;
pub(crate) use primitive_static;
pub(crate) use primitive_static_internal;

use crate::error::{PhResult, RuntimeError};
use crate::heap::{ClassId, ObjRef};
use crate::value::Value;
use crate::vm::VM;

/// Folds a 64-bit digest into an exact integral [`Value::Number`] hash code.
///
/// Shared by every `hash` primitive ([`object_hash`](object::object_hash),
/// [`number_hash`](number::number_hash), [`string_hash`](string::string_hash),
/// [`symbol_hash`](symbol::symbol_hash), [`bool_hash`](boolean::bool_hash)) so
/// they all produce a comparable, `f64`-representable integer — the digest is
/// masked to 53 bits so the `as f64` cast is lossless and round-trips
/// (`object-model.md` §8; [ADR-0023](../../../docs/adr/accepted/0023-amend-floor-admit-hash-and-kernel-reflection.md)).
pub(crate) fn hash_code(bits: u64) -> Value {
    Value::int((bits & 0x1F_FFFF_FFFF_FFFF) as i64)
}

/// Extracts a class handle from a receiver value.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `value` is not a class.
pub(crate) fn expect_class(vm: &VM, value: &Value) -> PhResult<ClassId> {
    if let Some(id) = value.as_obj() {
        if vm.heap.as_class(id).is_some() {
            return Ok(id);
        }
    }
    Err(RuntimeError::Type {
        expected: "Class",
        found: value.type_name(),
    }
    .into())
}

/// Extracts a string's contents from a receiver value.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `value` is not a string.
pub(crate) fn expect_string(vm: &VM, value: &Value) -> PhResult<String> {
    if let Some(id) = value.as_obj() {
        if let Some(string) = vm.heap.as_string(id) {
            return Ok(string.value());
        }
    }
    Err(RuntimeError::Type {
        expected: "String",
        found: value.type_name(),
    }
    .into())
}

/// Extracts a method's [`ObjRef`] handle from a receiver value.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `value` is not a `Method`.
pub(crate) fn expect_method(vm: &VM, value: &Value) -> PhResult<ObjRef> {
    if let Some(id) = value.as_obj() {
        if matches!(vm.heap.get(id), crate::heap::Object::Method(_)) {
            return Ok(id);
        }
    }
    Err(RuntimeError::Type {
        expected: "Method",
        found: value.type_name(),
    }
    .into())
}

/// Extracts a bytes buffer's [`ObjRef`] handle from a receiver value.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `value` is not a `Bytes`.
pub(crate) fn expect_bytes(vm: &VM, value: &Value) -> PhResult<ObjRef> {
    if let Some(id) = value.as_obj() {
        if vm.heap.as_bytes(id).is_some() {
            return Ok(id);
        }
    }
    Err(RuntimeError::Type {
        expected: "Bytes",
        found: value.type_name(),
    }
    .into())
}

pub(crate) fn expect_list(vm: &VM, value: &Value) -> PhResult<ObjRef> {
    if let Some(id) = value.as_obj() {
        if vm.heap.as_list(id).is_some() {
            return Ok(id);
        }
    }
    Err(RuntimeError::Type {
        expected: "List",
        found: value.type_name(),
    }
    .into())
}

/// Extracts a map's [`ObjRef`] handle from a receiver value.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `value` is not a `Map`.
pub(crate) fn expect_map(vm: &VM, value: &Value) -> PhResult<ObjRef> {
    if let Some(id) = value.as_obj() {
        if vm.heap.as_map(id).is_some() {
            return Ok(id);
        }
    }
    Err(RuntimeError::Type {
        expected: "Map",
        found: value.type_name(),
    }
    .into())
}

/// Extracts a set's [`ObjRef`] handle from a receiver value.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `value` is not a `Set`.
pub(crate) fn expect_set(vm: &VM, value: &Value) -> PhResult<ObjRef> {
    if let Some(id) = value.as_obj() {
        if vm.heap.as_set(id).is_some() {
            return Ok(id);
        }
    }
    Err(RuntimeError::Type {
        expected: "Set",
        found: value.type_name(),
    }
    .into())
}

/// Extracts a tuple's [`ObjRef`] handle from a receiver value.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `value` is not a `Tuple`.
pub(crate) fn expect_tuple(vm: &VM, value: &Value) -> PhResult<ObjRef> {
    if let Some(id) = value.as_obj() {
        if vm.heap.as_tuple(id).is_some() {
            return Ok(id);
        }
    }
    Err(RuntimeError::Type {
        expected: "Tuple",
        found: value.type_name(),
    }
    .into())
}

/// Extracts a positive `Record` heap object.
pub(crate) fn expect_record(vm: &VM, value: &Value) -> PhResult<ObjRef> {
    if let Some(id) = value.as_obj() {
        if vm.heap.as_record(id).is_some() {
            return Ok(id);
        }
    }
    Err(RuntimeError::Type {
        expected: "Record",
        found: value.type_name(),
    }
    .into())
}

/// Extracts a range's [`ObjRef`] handle from a receiver value.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `value` is not a `Range`.
pub(crate) fn expect_range(vm: &VM, value: &Value) -> PhResult<ObjRef> {
    if let Some(id) = value.as_obj() {
        if vm.heap.as_range(id).is_some() {
            return Ok(id);
        }
    }
    Err(RuntimeError::Type {
        expected: "Range",
        found: value.type_name(),
    }
    .into())
}

/// Sends the nullary `hash` selector to `value` and truncates the result to
/// `i64` — the bucket key [`crate::primitive::map`]/[`crate::primitive::set`]'s
/// `locate` indexes by. A re-entrant [`VM::send_dynamic`] call (ADR-0039); the
/// caller must not hold a `&Heap` borrow across this call.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `hash` does not answer a `Number`, or
/// propagates any [`RuntimeError`] the `hash` send itself raises.
pub(crate) fn send_hash(vm: &mut VM, value: Value) -> PhResult<i64> {
    let sym = vm.get_or_intern("hash");
    let res = vm.send_dynamic(value, sym, &[])?;
    if let Some(n) = res.as_int() {
        Ok(n)
    } else {
        Err(RuntimeError::InvalidHash { actual_type: res.type_name() }.into())
    }
}

/// Sends the one-argument `==(_)` selector (`a == b`) and returns the `Bool`
/// result — the disambiguation half of the `Map`/`Set` key-lookup re-entrant
/// send pair (with [`send_hash`]). A re-entrant [`VM::send_dynamic`] call; the
/// caller must not hold a `&Heap` borrow across this call.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `==` does not answer a `Bool`, or
/// propagates any [`RuntimeError`] the `==` send itself raises.
pub(crate) fn send_eq(vm: &mut VM, a: Value, b: Value) -> PhResult<bool> {
    let sym = vm.get_or_intern("==(_)");
    let res = vm.send_dynamic(a, sym, &[b])?;
    if let Some(result) = res.as_bool() {
        Ok(result)
    } else {
        Err(RuntimeError::Type {
            expected: "Bool",
            found: res.type_name(),
        }
        .into())
    }
}

/// Returns `true` iff `value` is one of the three native **mutable**
/// collection arms (`List`/`Map`/`Set`) — the DEC-CT-C mutable-key rejection
/// test.
pub(crate) fn is_mutable_collection_key(vm: &VM, value: &Value) -> bool {
    if let Some(id) = value.as_obj() {
        matches!(
            vm.heap.get(id),
            crate::heap::Object::List(_) | crate::heap::Object::Map(_) | crate::heap::Object::Set(_) | crate::heap::Object::Bytes(_)
        )
    } else {
        false
    }
}

/// Builds a catchable `Error` announcing a mutable-collection `Map`/`Set` key
/// rejection (DEC-CT-C), ready to be raised via
/// [`RuntimeError::Raise`]/`.into()`.
pub(crate) fn mutable_key_error(vm: &mut VM, class_name: &str) -> RuntimeError {
    let rendered = format!(
        "{class_name} key must be immutable: a List/Map/Set key has identity hash, \
         which is inconsistent with structural == (collection-protocol.md law 4)"
    );
    let error_class = vm.universe.classes.error_class;
    let field_count = vm.heap.class(error_class).field_count;
    let mut inst = crate::heap::InstanceObject::new(error_class, field_count);
    inst.slots[0] = vm.alloc_string_value(rendered.clone());
    let error = Value::obj(vm.heap.alloc(crate::heap::Object::Instance(inst)));
    RuntimeError::Raise {
        error,
        rendered,
        traceback: None,
        help: None,
    }
}
