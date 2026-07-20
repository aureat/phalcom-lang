//! The tagged [`Value`] — Phalcom's uniform in-register value representation.
//!
//! Realizes [ADR-0010](../../../docs/adr/accepted/0010-tagged-value-enum.md). `Value` is a
//! small `Copy` tagged `enum`: immediate arms ([`Value::Number`], [`Value::Bool`],
//! [`Value::Symbol`]) carry their payload inline, and every heap object is carried
//! by an [`ObjRef`] handle in [`Value::Obj`] ([ADR-0009](../../../docs/adr/accepted/0009-handle-arena-heap.md)).
//! [`Value::Nil`] is a **private** uninitialized-slot sentinel with no surface
//! class; user code can never produce or observe it (`values-and-absence.md`
//! Invariant 4). Because every arm is `Copy`, values move freely on the VM stack
//! and in constant pools without cloning or reference counting.
//!
//! NaN-boxing all of this into a single tagged `f64` word is a deferred
//! optimization *behind this same enum API*
//! ([ADR-0010](../../../docs/adr/accepted/0010-tagged-value-enum.md)); see
//! `docs/forge/DEFERRED.md`.

mod boolean;
mod nil;
mod render;

pub use boolean::{FALSE, TRUE};
pub use nil::NIL;

use crate::heap::lookup_method_in_hierarchy;
use crate::frame::CallContext;
use crate::heap::{ClassId, Object, ObjRef};
use crate::interner::Symbol;
use crate::vm::VM;
use std::hash::{Hash, Hasher};

/// A uniform Phalcom value: an immediate or a handle to a heap [`Object`].
///
/// Realizes [ADR-0010](../../../docs/adr/accepted/0010-tagged-value-enum.md). `Value` is
/// `Copy`; comparing two [`Value::Obj`]s tests object identity, while string
/// content-equality is provided separately by [`Value::value_eq`].
#[derive(Clone, Copy, PartialEq)]
pub enum Value {
    /// Private uninitialized-slot sentinel. Not surface-visible: it has no class
    /// row and user code can never produce or observe it (Invariant 4).
    Nil,
    /// A boolean. One `Bool` class; `True`/`False` are a later dispatch
    /// refinement, not a `Value` arm ([ADR-0004](../../../docs/adr/accepted/0004-boolean-as-abstract-bool-with-true-false.md)).
    Bool(bool),
    /// A flat 64-bit float ([ADR-0005](../../../docs/adr/retired/0005-number-as-flat-f64.md)).
    Number(f64),
    /// An interned identifier or selector ([`Symbol`]).
    Symbol(Symbol),
    /// Any heap object — instance, string, class, method or module — by
    /// [`ObjRef`] handle into the [`crate::heap::Heap`].
    Obj(ObjRef),
}

impl Value {
    /// Returns the [`ObjRef`] this value points at, or `None` for an immediate.
    ///
    /// **The garbage collector's sole seam onto `Value`.** The tracer
    /// ([`crate::heap::trace_object`]) reaches a value's heap child *only*
    /// through this accessor and never by matching `Value`'s arms, so a future
    /// `Value` representation change — NaN-boxing, per
    /// [ADR-0010](../../../docs/adr/accepted/0010-tagged-value-enum.md) — rewrites this one
    /// function and leaves the mark/sweep untouched
    /// ([ADR-0050](../../../docs/adr/accepted/0050-non-moving-mark-sweep-collector.md) §3,
    /// [memory-management.md §2.3](../../../docs/spec/v0.2/memory-management.md)).
    ///
    /// Immediates (`nil`, `Bool`, `Number`, `Symbol`) hold no handle: symbols live
    /// in the interner rather than the heap and are never collected
    /// ([memory-management.md §2.2](../../../docs/spec/v0.2/memory-management.md)).
    pub fn as_obj(&self) -> Option<ObjRef> {
        match self {
            Value::Obj(id) => Some(*id),
            Value::Nil | Value::Bool(_) | Value::Number(_) | Value::Symbol(_) => None,
        }
    }

    /// Returns the interned [`Symbol`] this value carries.
    ///
    /// # Errors
    ///
    /// Returns a type-error message string if the value is not a [`Value::Symbol`].
    pub fn as_symbol(&self) -> Result<Symbol, String> {
        match self {
            Value::Symbol(s) => Ok(*s),
            _ => Err("Type Error: Expected a Symbol.".to_string()),
        }
    }

    /// Returns a coarse, heap-free type tag for diagnostics.
    ///
    /// Immediate arms report their exact kind; every [`Value::Obj`] reports
    /// `"object"` because discriminating its heap kind would require heap access
    /// (see [`Value::class`] for the precise class). Used by the `expect_value!`
    /// diagnostics macro.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Nil => "nil",
            Value::Bool(_) => "bool",
            Value::Number(_) => "number",
            Value::Symbol(_) => "symbol",
            Value::Obj(_) => "object",
        }
    }

    /// Returns the [`ClassId`] of this value's class.
    ///
    /// Immediates map to their core class; a [`Value::Obj`] resolves through the
    /// heap (an instance to its class, a class to its metaclass, a string to
    /// `String`, and so on). Realizes the "every value maps onto a class" rule
    /// (`object-model.md` §3).
    ///
    /// A [`Value::Bool`] resolves by its payload to one of the two concrete
    /// singleton subclasses of the abstract `Bool` class — `true` to `True`,
    /// `false` to `False` — so `true.class == True` and `false.class == False`
    /// ([ADR-0004](../../../docs/adr/accepted/0004-boolean-as-abstract-bool-with-true-false.md)).
    /// The selection is a plain [`ClassId`] field read with no allocation, on
    /// the hot dispatch path.
    ///
    /// # Panics
    ///
    /// Panics if the value is a [`Value::Obj`] whose handle is stale, or a bare
    /// closure handle (closures are never surface values).
    #[inline]
    pub fn class(&self, vm: &VM) -> ClassId {
        match self {
            Value::Nil => vm.universe.classes.nil_class,
            Value::Bool(b) => {
                if *b {
                    vm.universe.classes.true_class
                } else {
                    vm.universe.classes.false_class
                }
            }
            Value::Number(_) => vm.universe.classes.number_class,
            Value::Symbol(_) => vm.universe.classes.symbol_class,
            Value::Obj(id) => match vm.heap.get(*id) {
                Object::Instance(instance) => instance.class,
                Object::Class(class) => class.class,
                Object::Method(_) => vm.universe.classes.method_class,
                Object::Module(_) => vm.universe.classes.module_class,
                Object::Str(_) => vm.universe.classes.string_class,
                Object::Closure(_) => vm.universe.classes.block_class,
                Object::Block(_) => vm.universe.classes.block_class,
                // `bind(_)`'s result surfaces as a `Block` (ADR-0006, U-CORE-3
                // / ADR-0028) so `bound.call(...)` is ordinary `Function` call
                // protocol from the surface's point of view.
                Object::BoundMethod(_) => vm.universe.classes.block_class,
                Object::List(_) => vm.universe.classes.list_class,
                Object::Fiber(_) => vm.universe.classes.fiber_class,
                Object::Map(_) => vm.universe.classes.map_class,
                Object::Set(_) => vm.universe.classes.set_class,
                Object::Tuple(_) => vm.universe.classes.tuple_class,
                Object::Range(_) => vm.universe.classes.range_class,
                // `::` method reference (selectors.md §3, U16-Open) — reached
                // through `Value::Obj` exactly as `Object::List` is; no
                // `Value::Family` arm (ADR-0010 keeps `Value` minimal).
                Object::Family(_) => vm.universe.classes.family_class,
                Object::Upvalue(_) => panic!("upvalues are not surface values"),
            },
        }
    }

    /// Looks up `selector` on this value's class hierarchy.
    ///
    /// Returns the resolved [`MethodObject`](crate::method::MethodObject) handle,
    /// or `None` if no class in the chain defines it.
    ///
    /// A class receiver needs no constructor-specific fallback here:
    /// constructors install on the metaclass under the ordinary selector their
    /// call sites encode, so the plain hierarchy walk resolves `Foo.new()` to
    /// `Foo`'s constructor — shadowing the bare allocator `Class >> new()` at
    /// the tower root — exactly as it resolves any other class-side method.
    pub fn lookup_method(&self, vm: &VM, selector: Symbol) -> Option<ObjRef> {
        let class = self.class(vm);
        lookup_method_in_hierarchy(&vm.heap, class, selector)
    }

    /// Builds the [`CallContext`] a closure-backed method call against `self`
    /// runs with.
    ///
    /// An immediate receiver (`Bool`/`Number`/`Symbol`/the private `Nil`
    /// sentinel) yields [`CallContext::Immediate`] rather than panicking —
    /// U5 (ADR-0018) needs this so a user-reopened sacred selector on the
    /// kernel `Bool` class (a closure method, unlike the primitive it
    /// shadows) is actually callable on a real `true`/`false` receiver, not
    /// just a heap object.
    pub fn to_context(&self, heap: &crate::heap::Heap) -> CallContext {
        match self {
            Value::Obj(id) => match heap.get(*id) {
                Object::Instance(_)
                | Object::Block(_)
                | Object::Closure(_)
                | Object::Str(_)
                | Object::Method(_)
                | Object::BoundMethod(_)
                | Object::List(_)
                | Object::Fiber(_)
                | Object::Map(_)
                | Object::Set(_)
                | Object::Tuple(_)
                | Object::Range(_)
                | Object::Family(_) => CallContext::Instance { instance: *id },
                Object::Class(_) => CallContext::Class { class: *id },
                Object::Module(_) => CallContext::Module { module: *id },
                Object::Upvalue(_) => panic!("upvalues are not surface receivers"),
            },
            Value::Nil | Value::Bool(_) | Value::Number(_) | Value::Symbol(_) => CallContext::Immediate { value: *self },
        }
    }

    /// Tests Phalcom value equality (the `==` operator).
    ///
    /// This reproduces, exactly, the observable semantics of the pre-heap
    /// hand-written `impl PartialEq for Value`, so `==`/`!=` behaviour is
    /// preserved across the handle-arena migration
    /// ([ADR-0009](../../../docs/adr/accepted/0009-handle-arena-heap.md)):
    ///
    /// - `Nil`, `Bool`, `Number` compare by value.
    /// - Two [`Value::Obj`] strings compare by content; instances, classes and
    ///   methods compare by identity ([`ObjRef`] handle equality).
    /// - [`Value::Symbol`] pairs and [`Object::Module`] pairs are **never**
    ///   equal — they routed to the `_ => false` arm before the migration and
    ///   must keep doing so.
    /// - Every mismatched or otherwise unhandled pair is unequal.
    ///
    /// Equality is matched explicitly here rather than delegating to the
    /// [`PartialEq`] derive: the derive compares [`Value::Symbol`]s
    /// structurally (equal interned symbols would be equal) and same-handle
    /// modules as equal, both of which would silently change `==` semantics.
    pub fn value_eq(&self, other: &Value, heap: &crate::heap::Heap) -> bool {
        use crate::heap::Object;
        match (self, other) {
            (Value::Nil, Value::Nil) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            // Symbols compare by interned identity, not handle — two
            // independently-interned occurrences of the same text share one
            // `Symbol` (the interner is content-addressed), so `a == b` here
            // is content equality (U-LEX-HASH coupled fix; was previously
            // absent, falling through to the generic `_ => false` below and
            // making `Symbol.new("a") == Symbol.new("a")` false).
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            (Value::Obj(a), Value::Obj(b)) => {
                // Strings compare by content, regardless of handle.
                match (heap.as_string(*a), heap.as_string(*b)) {
                    (Some(x), Some(y)) => return x == y,
                    (Some(_), None) | (None, Some(_)) => return false,
                    (None, None) => {}
                }
                // Modules were never equal under the pre-heap `PartialEq`
                // (they fell through to `_ => false`); preserve that.
                if matches!(heap.get(*a), Object::Module(_))
                    || matches!(heap.get(*b), Object::Module(_))
                {
                    return false;
                }
                // Instances, classes and methods compare by identity.
                a == b
            }
            // Every mismatched pair is unequal.
            _ => false,
        }
    }
}

/// Surfaces the private [`Value::Nil`] sentinel as the `None` singleton.
///
/// This is the **read-boundary surfacer** of U6's absence model: the private
/// `Value::Nil` sentinel ([ADR-0010](../../../docs/adr/accepted/0010-tagged-value-enum.md))
/// backs uninitialized slots internally but must never reach user code
/// (Invariant 4, `values-and-absence.md`). Every read boundary that can observe
/// an unwritten slot — an uninitialized `var` read, an unassigned field read, a
/// bare-`return` default, a method falling off its end — routes the value
/// through here so the sentinel is replaced by the shared `None` object
/// ([ADR-0007](../../../docs/adr/accepted/0007-option-some-none.md)); any non-sentinel
/// value passes through unchanged.
///
/// `none_singleton` is the handle to the process-wide `None` instance
/// ([`crate::universe::CoreClasses::none_singleton`]); passing the same handle
/// everywhere keeps `None` identity-comparable and zero-allocation
/// ([ADR-0004](../../../docs/adr/accepted/0004-boolean-as-abstract-bool-with-true-false.md)
/// mirror). There is intentionally **no** public constructor that yields
/// [`Value::Nil`]: surfacing is one-directional (sentinel → `None`), never the
/// reverse, so `None` can never end up inside a `Some`.
#[inline]
#[must_use]
pub fn sentinel_to_option(value: Value, none_singleton: ObjRef) -> Value {
    match value {
        Value::Nil => Value::Obj(none_singleton),
        other => other,
    }
}

/// Hashes an [`f64`] by its raw bits so `NaN`/`-0.0` hash deterministically.
fn hash_f64<H: Hasher>(num: f64, state: &mut H) {
    num.to_bits().hash(state);
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Nil => 0u8.hash(state),
            Value::Bool(b) => b.hash(state),
            Value::Number(n) => hash_f64(*n, state),
            Value::Symbol(s) => s.0.hash(state),
            Value::Obj(id) => id.hash(state),
        }
    }
}
