use crate::error::PhResult;
use crate::heap::Object;
use crate::vm::VM;
use std::fmt::{self, Debug, Display};

use super::Value;

impl Value {
    /// Renders this value the way `System.print` and `toString` present it.
    ///
    /// Strings render as their raw content (no quotes); numbers, booleans and the
    /// `nil` sentinel render as their literals; a symbol renders as `Symbol("…")`;
    /// a `List` renders as `[e1, e2, …]` (each element recursively via this same
    /// renderer); the shared `None` singleton renders as `"None"` and a `Some`
    /// instance as `"Some(v)"` (`v` rendered recursively) — kept in agreement
    /// with the `.ph`-message `Option#toString` (U-CORE-4, R-INV-4.1). Any other
    /// heap object renders via its debug form. Returns an owned [`String`]
    /// rather than allocating a heap object.
    pub fn to_string(&self, vm: &VM) -> String {
        match self {
            Value::Nil => "nil".to_string(),
            Value::Unit => "()".to_string(),
            Value::Bool(b) => bool_literal(*b).to_string(),
            Value::Int(n) => n.to_string(),
            Value::Float(n) => render_float(*n),
            Value::Symbol(s) => s.to_string(vm),
            Value::Obj(id) => match vm.heap.get(*id) {
                Object::LargeInt(bigint) => bigint.to_string(),
                Object::Str(string) => string.value(),
                Object::List(list) => {
                    let parts: Vec<String> = list.elements().iter().map(|v| v.to_string(vm)).collect();
                    format!("[{}]", parts.join(", "))
                }
                // The `.ph` `Bytes#toString` (bytes.md §4) is the surface
                // spelling; this is the same debug form for the raw-render
                // path (echo of a receiver with no user override yet).
                Object::Bytes(bytes) => format!("Bytes({})", bytes.len()),
                Object::Instance(inst) if inst.class == vm.universe.classes.none_class => "None".to_string(),
                Object::Instance(inst) if inst.class == vm.universe.classes.some_class => {
                    format!("Some({})", inst.slots[0].to_string(vm))
                }
                Object::Map(map) => {
                    let parts: Vec<String> = map.entries().map(|(k, v)| format!("{}: {}", k.to_string(vm), v.to_string(vm))).collect();
                    format!("{{{}}}", parts.join(", "))
                }
                Object::Set(set) => {
                    let parts: Vec<String> = set.entries().map(|(k, _)| k.to_string(vm)).collect();
                    format!("Set({})", parts.join(", "))
                }
                Object::Tuple(tuple) => {
                    let parts: Vec<String> = tuple.values().iter().map(|v| v.to_string(vm)).collect();
                    format!("({})", parts.join(", "))
                }
                Object::Range(range) => {
                    let lower = range.lower().map_or_else(String::new, |value| value.to_string(vm));
                    let upper = range.upper().map_or_else(String::new, |value| value.to_string(vm));
                    let sep = if range.upper_inclusive() { "..=" } else { ".." };
                    format!("{lower}{sep}{upper}")
                }
                Object::Record(_) => "<record>".to_string(),
                _ => self.to_debug(vm),
            },
        }
    }

    /// Renders this value for display (`System.print`) by sending it a
    /// `toString` message, unconditionally.
    pub fn to_display_string(&self, vm: &mut VM) -> PhResult<String> {
        if let Value::Int(_) = self {
            if vm.universe.int_tostring_pristine {
                return Ok(self.to_string(vm));
            }
        } else if let Value::Float(_) = self {
            if vm.universe.float_tostring_pristine {
                return Ok(self.to_string(vm));
            }
        } else if let Value::Symbol(_) = self {
            if vm.universe.symbol_tostring_pristine {
                return Ok(self.to_string(vm));
            }
        } else if let Value::Obj(id) = self {
            if vm.universe.str_tostring_pristine && matches!(vm.heap.get(*id), Object::Str(_)) {
                return Ok(self.to_string(vm));
            }
            if vm.universe.int_tostring_pristine && matches!(vm.heap.get(*id), Object::LargeInt(_)) {
                return Ok(self.to_string(vm));
            }
        }
        let selector = vm.get_or_intern("toString");
        let rendered = vm.send_dynamic(*self, selector, &[])?;
        Ok(rendered.to_string(vm))
    }

    /// Renders this value's debug form (used by error messages and diagnostics).
    ///
    /// Identical to [`Value::to_string`] except that a symbol renders as
    /// `<symbol N>`.
    pub fn to_debug(&self, vm: &VM) -> String {
        match self {
            Value::Nil => "nil".to_string(),
            Value::Unit => "()".to_string(),
            Value::Bool(b) => bool_literal(*b).to_string(),
            Value::Int(n) => n.to_string(),
            Value::Float(n) => render_float(*n),
            Value::Symbol(s) => s.to_debug(),
            Value::Obj(id) => match vm.heap.get(*id) {
                Object::LargeInt(bigint) => bigint.to_string(),
                Object::Str(string) => string.value(),
                Object::Instance(instance) => instance.to_debug(&vm.heap),
                Object::Class(class) => class.to_debug(),
                Object::Method(method) => method.to_debug(vm),
                Object::Module(module) => module.to_debug(),
                Object::Closure(_) => "<block>".to_string(),
                Object::Block(_) => "<block>".to_string(),
                Object::BoundMethod(_) => "<bound method>".to_string(),
                Object::List(_) => "<list>".to_string(),
                Object::Bytes(_) => "<bytes>".to_string(),
                Object::Fiber(_) => "<fiber>".to_string(),
                Object::Map(_) => "<map>".to_string(),
                Object::Set(_) => "<set>".to_string(),
                Object::Tuple(_) => "<tuple>".to_string(),
                Object::Record(_) => "<record>".to_string(),
                Object::Range(_) => "<range>".to_string(),
                Object::Family(_) => "<family>".to_string(),
                Object::Upvalue(_) => "<upvalue>".to_string(),
            },
        }
    }
}

/// Returns the surface literal (`"true"` / `"false"`) for a boolean.
fn bool_literal(b: bool) -> &'static str {
    if b { "true" } else { "false" }
}

impl Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Unit => write!(f, "()"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Int(i) => write!(f, "{i}"),
            Self::Float(n) => write!(f, "{}", render_float(*n)),
            Self::Symbol(s) => write!(f, "Symbol({})", s.0),
            Self::Obj(id) => write!(f, "<obj {id:?}>"),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Unit => write!(f, "()"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Int(i) => write!(f, "{i}"),
            Self::Float(n) => write!(f, "{}", render_float(*n)),
            Self::Symbol(s) => write!(f, "Symbol({})", s.0),
            Self::Obj(id) => write!(f, "<obj {id:?}>"),
        }
    }
}

/// Canonical shortest-roundtrip rendering for an `f64` value.
///
/// Matches the FLOAT-TEXT grammar from the spec:
/// - `NaN` for any IEEE 754 NaN.
/// - `Infinity` / `-Infinity` for positive/negative infinity.
/// - Shortest decimal that round-trips for all finite values (via `ryu`).
pub(crate) fn render_float(n: f64) -> String {
    if n.is_nan() {
        "NaN".to_string()
    } else if n.is_infinite() {
        if n.is_sign_negative() {
            "-Infinity".to_string()
        } else {
            "Infinity".to_string()
        }
    } else {
        let mut buf = ryu::Buffer::new();
        buf.format(n).to_string()
    }
}
