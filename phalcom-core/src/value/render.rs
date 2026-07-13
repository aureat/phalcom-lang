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
            Value::Bool(b) => bool_literal(*b).to_string(),
            Value::Number(n) => n.to_string(),
            Value::Symbol(s) => s.to_string(vm),
            Value::Obj(id) => match vm.heap.get(*id) {
                Object::Str(string) => string.value(),
                Object::List(list) => {
                    let parts: Vec<String> = list.elements().iter().map(|v| v.to_string(vm)).collect();
                    format!("[{}]", parts.join(", "))
                }
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
                    let parts: Vec<String> = tuple.elements().iter().map(|v| v.to_string(vm)).collect();
                    format!("({})", parts.join(", "))
                }
                Object::Range(range) => {
                    let sep = if range.inclusive() { ".." } else { "..." };
                    format!("{}{sep}{}", range.start().to_string(vm), range.end().to_string(vm))
                }
                _ => self.to_debug(vm),
            },
        }
    }

    /// Renders this value for display (`System.print`), sending `toString`
    /// to any heap object [`Value::to_string`] has no bespoke native
    /// renderer for.
    ///
    /// [`Value::to_string`] hardcodes formatting for `Str`/`List`/`Map`/
    /// `Set`/`Tuple`/`Range` and the shared `None`/`Some` singletons/wrapper
    /// — all cases where the native rendering already agrees with what a
    /// `.ph` `toString` override would produce. Every other heap object
    /// (a plain instance, a class, a metaclass, …) instead falls through
    /// [`Value::to_string`] to [`Value::to_debug`], which disagrees with the
    /// object's own `Object#toString` message
    /// ([ADR-0015](../../../docs/adr/0015-object-default-tostring.md)) — e.g.
    /// a bare `Point` instance printed `<Point instance>` via
    /// `System.print` but `<Point>` via `.toString`. This method closes that
    /// gap by sending `toString` for exactly those cases, so `System.print`
    /// and an explicit `.toString` send always agree (U-ERR-FIX
    /// PRINT-TOSTRING).
    ///
    /// # Errors
    ///
    /// Propagates any error the `toString` send itself raises (e.g. a user
    /// override that throws).
    pub fn to_display_string(&self, vm: &mut VM) -> PhResult<String> {
        if let Value::Obj(id) = *self {
            let handled_natively = match vm.heap.get(id) {
                Object::Str(_) | Object::List(_) | Object::Map(_) | Object::Set(_) | Object::Tuple(_) | Object::Range(_) => true,
                Object::Instance(inst) => inst.class == vm.universe.classes.none_class || inst.class == vm.universe.classes.some_class,
                _ => false,
            };
            if !handled_natively {
                let selector = vm.get_or_intern("toString");
                let rendered = vm.send_dynamic(*self, selector, &[])?;
                return Ok(rendered.to_string(vm));
            }
        }
        Ok(self.to_string(vm))
    }

    /// Renders this value's debug form (used by error messages and diagnostics).
    ///
    /// Identical to [`Value::to_string`] except that a symbol renders as
    /// `<symbol N>`.
    pub fn to_debug(&self, vm: &VM) -> String {
        match self {
            Value::Nil => "nil".to_string(),
            Value::Bool(b) => bool_literal(*b).to_string(),
            Value::Number(n) => n.to_string(),
            Value::Symbol(s) => s.to_debug(),
            Value::Obj(id) => match vm.heap.get(*id) {
                Object::Str(string) => string.value(),
                Object::Instance(instance) => instance.to_debug(&vm.heap),
                Object::Class(class) => class.to_debug(),
                Object::Method(method) => method.to_debug(vm),
                Object::Module(module) => module.to_debug(),
                Object::Closure(_) => "<block>".to_string(),
                Object::Block(_) => "<block>".to_string(),
                Object::BoundMethod(_) => "<bound method>".to_string(),
                Object::List(_) => "<list>".to_string(),
                Object::Fiber(_) => "<fiber>".to_string(),
                Object::Map(_) => "<map>".to_string(),
                Object::Set(_) => "<set>".to_string(),
                Object::Tuple(_) => "<tuple>".to_string(),
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
            Self::Bool(b) => write!(f, "{b}"),
            Self::Number(n) => write!(f, "{n}"),
            Self::Symbol(s) => write!(f, "Symbol({})", s.0),
            Self::Obj(id) => write!(f, "<obj {id:?}>"),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Number(n) => write!(f, "{n}"),
            Self::Symbol(s) => write!(f, "Symbol({})", s.0),
            Self::Obj(id) => write!(f, "<obj {id:?}>"),
        }
    }
}
