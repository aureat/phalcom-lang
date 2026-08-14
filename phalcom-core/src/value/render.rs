use crate::error::PhResult;
use crate::heap::Object;
use crate::vm::VM;
use std::fmt::{self, Debug, Display};

use super::{OptionPayload, Value};

impl Value {
    /// Renders this value the way `System.print` and `toString` present it.
    ///
    /// Strings render as their raw content (no quotes); numbers, booleans and the
    /// `nil` sentinel render as their literals; a symbol renders as `Symbol("…")`;
    /// a `List` renders as `[e1, e2, …]` (each element recursively via this same
    /// renderer); immediate `None` renders as `"None"` and an immediate `Some`
    /// value as `"Some(v)"` (`v` rendered recursively) — kept in agreement
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
            Value::None => "None".to_string(),
            value @ (Value::Some1(_) | Value::Some2(_) | Value::Some3(_) | Value::Some4(_) | Value::Some5(_) | Value::Some6(_) | Value::Some7(_)) => {
                render_option(*value, vm, false)
            }
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
            Value::None => "None".to_string(),
            value @ (Value::Some1(_) | Value::Some2(_) | Value::Some3(_) | Value::Some4(_) | Value::Some5(_) | Value::Some6(_) | Value::Some7(_)) => {
                render_option(*value, vm, true)
            }
            Value::Obj(id) => match vm.heap.get(*id) {
                Object::LargeInt(bigint) => bigint.to_string(),
                Object::Str(string) => string.value(),
                Object::Instance(instance) => instance.to_debug(&vm.heap),
                Object::Class(class) => class.to_debug(),
                Object::Method(method) => method.to_debug(vm),
                Object::Module(module) => module.to_debug(),
                Object::Closure(_) => "<closure>".to_string(),
                Object::Block(_) => "<closure>".to_string(),
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
                Object::SelectorPattern(_) => "<selector pattern>".to_string(),
                Object::MethodFamily(_) => "<method family>".to_string(),
                Object::Upvalue(_) => "<upvalue>".to_string(),
                Object::PackBuilder(_) => "<internal pack builder>".to_string(),
                Object::RecordLiteralBuilder(_) => "<internal Record literal builder>".to_string(),
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
            Self::None => write!(f, "None"),
            Self::Some1(payload) => fmt_option(f, 1, *payload, true),
            Self::Some2(payload) => fmt_option(f, 2, *payload, true),
            Self::Some3(payload) => fmt_option(f, 3, *payload, true),
            Self::Some4(payload) => fmt_option(f, 4, *payload, true),
            Self::Some5(payload) => fmt_option(f, 5, *payload, true),
            Self::Some6(payload) => fmt_option(f, 6, *payload, true),
            Self::Some7(payload) => fmt_option(f, 7, *payload, true),
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
            Self::None => write!(f, "None"),
            Self::Some1(payload) => fmt_option(f, 1, *payload, false),
            Self::Some2(payload) => fmt_option(f, 2, *payload, false),
            Self::Some3(payload) => fmt_option(f, 3, *payload, false),
            Self::Some4(payload) => fmt_option(f, 4, *payload, false),
            Self::Some5(payload) => fmt_option(f, 5, *payload, false),
            Self::Some6(payload) => fmt_option(f, 6, *payload, false),
            Self::Some7(payload) => fmt_option(f, 7, *payload, false),
        }
    }
}

fn option_parts(value: Value) -> (u8, OptionPayload) {
    match value {
        Value::Some1(payload) => (1, payload),
        Value::Some2(payload) => (2, payload),
        Value::Some3(payload) => (3, payload),
        Value::Some4(payload) => (4, payload),
        Value::Some5(payload) => (5, payload),
        Value::Some6(payload) => (6, payload),
        Value::Some7(payload) => (7, payload),
        _ => unreachable!("option_parts called for non-Some value"),
    }
}

fn render_option(value: Value, vm: &VM, debug: bool) -> String {
    let (depth, payload) = option_parts(value);
    let mut rendered = if debug {
        payload.into_value().to_debug(vm)
    } else {
        payload.into_value().to_string(vm)
    };
    for _ in 0..depth {
        rendered = format!("Some({rendered})");
    }
    rendered
}

fn fmt_option(f: &mut fmt::Formatter<'_>, depth: u8, payload: OptionPayload, debug: bool) -> fmt::Result {
    write!(f, "Some(")?;
    if depth == 1 {
        if debug {
            write!(f, "{:?}", payload.into_value())?;
        } else {
            write!(f, "{}", payload.into_value())?;
        }
    } else {
        fmt_option(f, depth - 1, payload, debug)?;
    }
    write!(f, ")")
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
