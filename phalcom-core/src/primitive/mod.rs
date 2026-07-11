pub mod boolean;
pub mod block;
pub mod class;
pub mod list;
pub mod method;
pub mod module;
pub mod nil;
pub mod number;
pub mod object;
pub mod string;
pub mod symbol;
pub mod system;

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
    pub const name_set: &'static str = "name=(_)";
    pub const class: &'static str = "class";
    pub const class_set: &'static str = "class=(_)";
    pub const superclass: &'static str = "superclass";
    pub const superclass_set: &'static str = "superclass=(_)";

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
        let sig_str = crate::method::make_signature($base, $sig_kind);
        let symbol = $vm.get_or_intern(&sig_str);
        let method = MethodObject::new_primitive(symbol, $sig_kind, $func, $class);
        let method_id = $vm.heap.alloc(crate::heap::Object::Method(method));
        $vm.heap.class_mut($class).add_method(symbol, method_id);
    };
}

/// Installs a native *static* method `$func` on class `$class`.
///
/// Like [`primitive!`], but binds the method on `$class`'s metaclass (`$class`'s
/// `class`), where static methods live.
macro_rules! primitive_static {
    ($vm:expr, $class:expr, $base:expr, $sig_kind: expr, $func:expr) => {
        let sig_str = crate::method::make_signature($base, $sig_kind);
        let symbol = $vm.get_or_intern(&sig_str);
        let method = MethodObject::new_primitive(symbol, $sig_kind, $func, $class);
        let method_id = $vm.heap.alloc(crate::heap::Object::Method(method));
        let meta = $vm.heap.class($class).class;
        $vm.heap.class_mut(meta).add_method(symbol, method_id);
    };
}

pub(crate) use primitive;
pub(crate) use primitive_static;

use crate::error::{PhResult, RuntimeError};
use crate::heap::{ClassId, ObjRef};
use crate::value::Value;
use crate::vm::VM;

/// Extracts a class handle from a receiver value.
///
/// Replaces the heap-agnostic `expect_value!(_, Class)` arm: a class is now a
/// [`Value::Obj`] whose heap object is a [`ClassObject`](crate::class::ClassObject).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `value` is not a class.
pub(crate) fn expect_class(vm: &VM, value: &Value) -> PhResult<ClassId> {
    match value {
        Value::Obj(id) if vm.heap.as_class(*id).is_some() => Ok(*id),
        other => Err(RuntimeError::Type {
            expected: "Class",
            found: other.type_name(),
        }
        .into()),
    }
}

/// Extracts a string's contents from a receiver value.
///
/// Replaces the heap-agnostic `expect_value!(_, String)` arm: a string is now a
/// [`Value::Obj`] whose heap object is a [`StringObject`](crate::string::StringObject).
/// Returns an owned copy so callers can subsequently allocate without holding a
/// heap borrow.
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `value` is not a string.
pub(crate) fn expect_string(vm: &VM, value: &Value) -> PhResult<String> {
    match value {
        Value::Obj(id) => match vm.heap.as_string(*id) {
            Some(string) => Ok(string.value()),
            None => Err(RuntimeError::Type {
                expected: "String",
                found: value.type_name(),
            }
            .into()),
        },
        other => Err(RuntimeError::Type {
            expected: "String",
            found: other.type_name(),
        }
        .into()),
    }
}

/// Extracts a list's [`ObjRef`] handle from a receiver value.
///
/// Mirrors [`expect_string`]: a list is a [`Value::Obj`] whose heap object is
/// a [`crate::list::ListObject`]
/// ([ADR-0020](../../../docs/adr/0020-kernel-list-native-array-protocol.md)).
///
/// # Errors
///
/// Returns [`RuntimeError::Type`] if `value` is not a list.
pub(crate) fn expect_list(vm: &VM, value: &Value) -> PhResult<ObjRef> {
    match value {
        Value::Obj(id) if vm.heap.as_list(*id).is_some() => Ok(*id),
        other => Err(RuntimeError::Type {
            expected: "List",
            found: other.type_name(),
        }
        .into()),
    }
}
