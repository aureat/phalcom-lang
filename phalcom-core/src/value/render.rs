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

    /// Renders this value for display (`System.print`) by sending it a
    /// `toString` message, unconditionally.
    ///
    /// Every value — immediates and heap objects alike — gets a real
    /// `toString` message send: `Str` resolves to `self` (core.ph), `Number`
    /// to [`crate::primitive::number::number_to_string`], `List` to
    /// [`crate::primitive::list::list_to_string`], `Map`/`Set`/`Tuple`/
    /// `Range` to their `.ph` `toString` overrides, a plain instance to
    /// `Object#toString`
    /// ([ADR-0015](../../../docs/adr/accepted/0015-object-default-tostring.md)),
    /// and a user class that overrides `toString` to that override. This is
    /// what makes `System.print` and an explicit `.toString` send agree at
    /// every depth: a container's native renderer (e.g. `list_to_string`)
    /// itself calls this method per element, so a user `toString` override
    /// on a nested value is honored no matter how deep it's nested.
    ///
    /// Earlier versions of this method special-cased `Str`/`List`/`Map`/
    /// `Set`/`Tuple`/`Range` as "already handled by [`Value::to_string`]'s
    /// native formatting" and skipped the send for them. That was wrong:
    /// once those types grew `.ph` `toString` overrides (or could be
    /// re-opened by user code), the special case silently bypassed
    /// dispatch and disagreed with `\(…)` string interpolation on the same
    /// value (CB-6). Sending unconditionally is what dispatch requires.
    ///
    /// # Leaf fast path
    ///
    /// `Number`/`Symbol`/`Str` are the exception, and only while their
    /// override-epoch flag ([`crate::universe::Universe::number_tostring_pristine`]/
    /// [`crate::universe::Universe::symbol_tostring_pristine`]/
    /// [`crate::universe::Universe::str_tostring_pristine`]) reads `true`: for these three
    /// leaf types [`Value::to_string`]'s native formatting is *provably*
    /// identical to their real `toString` body, because none of the three
    /// recurses into another value's `toString` — there is nothing for a
    /// nested override to bypass. This lets `System.print` on a `Number`/
    /// `Symbol`/`Str` (and, transitively, each element a container's
    /// `toString` sends this method for) skip the `toString` send and its
    /// throwaway `Str` allocation entirely.
    ///
    /// This is deliberately **not** widened to any other type. `Bool`/`Nil`
    /// have no override-epoch flag of their own here and always send
    /// (`Bool` has a `.ph` `toString` in `core.ph`, watched flags for it
    /// would also need to track `Object#toString` reopens transitively,
    /// which is unnecessary complexity for two receivers). Containers
    /// (`List`/`Map`/`Set`/`Tuple`/`Range`) and plain instances must
    /// *always* send: their real `toString` recurses per element/slot via
    /// this exact method, so a nested override several levels deep is only
    /// honored if every level actually sends — fast-pathing a container
    /// would silently reintroduce CB-6.
    ///
    /// # Errors
    ///
    /// Propagates any error the `toString` send itself raises (e.g. a user
    /// override that throws).
    pub fn to_display_string(&self, vm: &mut VM) -> PhResult<String> {
        if let Value::Number(_) = self {
            if vm.universe.number_tostring_pristine {
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
