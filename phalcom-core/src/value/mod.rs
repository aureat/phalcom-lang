//! The tagged [`Value`] — Phalcom's uniform in-register value representation.
//!
//! Realizes [ADR-0010](../../../docs/adr/accepted/0010-tagged-value-enum.md). `Value` is a
//! small `Copy` tagged `enum`: immediate arms ([`Value::Int`], [`Value::Float`],
//! [`Value::Bool`], [`Value::Symbol`], and bounded `Option` variants) carry their payload inline, and every heap object is carried
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
mod option;
mod render;

pub use boolean::{FALSE, TRUE};
pub use nil::NIL;
pub(crate) use option::OptionCase;
pub use option::{MAX_OPTION_NESTING, OptionPayload};

use crate::frame::CallContext;
use crate::heap::lookup_method_in_hierarchy;
use crate::heap::{ClassId, ObjRef, Object};
use crate::interner::Symbol;
use crate::vm::VM;
use num_traits::{FromPrimitive, Zero};
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
    /// The canonical zero-arity product. Unlike `Nil`, this is surface-visible.
    Unit,
    /// A boolean. One `Bool` class; `True`/`False` are a later dispatch
    /// refinement, not a `Value` arm ([ADR-0004](../../../docs/adr/accepted/0004-boolean-as-abstract-bool-with-true-false.md)).
    Bool(bool),
    /// A small signed integer payload (canonical for representable `i64`).
    Int(i64),
    /// A 64-bit IEEE-754 binary floating-point payload.
    Float(f64),
    /// An interned identifier or selector ([`Symbol`]).
    Symbol(Symbol),
    /// Any heap object — instance, string, class, method, module or large int — by
    /// [`ObjRef`] handle into the [`crate::heap::Heap`].
    Obj(ObjRef),
    /// Immediate absence. This is distinct from the private [`Value::Nil`]
    /// uninitialized-slot sentinel.
    None,
    /// One through seven flattened immediate `Some` layers. The payload is
    /// non-recursive so nested Options remain allocation-free.
    Some1(OptionPayload),
    Some2(OptionPayload),
    Some3(OptionPayload),
    Some4(OptionPayload),
    Some5(OptionPayload),
    Some6(OptionPayload),
    Some7(OptionPayload),
}

/// Normalizes a [`num_bigint::BigInt`] into a [`Value`].
/// Returns [`Value::Int(i64)`] if representable as `i64`, or allocates a heap [`Object::LargeInt`] otherwise.
pub fn normalize_bigint(bigint: num_bigint::BigInt, heap: &mut crate::heap::Heap) -> Value {
    if let Ok(small) = i64::try_from(&bigint) {
        Value::Int(small)
    } else {
        Value::Obj(heap.alloc(Object::LargeInt(bigint)))
    }
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
    /// This ordinary object query exposes only a direct `Value::Obj`; it does
    /// not see through an immediate `Some` payload. Symbols live in the
    /// interner rather than the heap and are never collected
    /// ([memory-management.md §2.2](../../../docs/spec/v0.2/memory-management.md)).
    pub fn as_obj(&self) -> Option<ObjRef> {
        match self {
            Value::Obj(id) => Some(*id),
            Value::Nil
            | Value::Unit
            | Value::Bool(_)
            | Value::Int(_)
            | Value::Float(_)
            | Value::Symbol(_)
            | Value::None
            | Value::Some1(_)
            | Value::Some2(_)
            | Value::Some3(_)
            | Value::Some4(_)
            | Value::Some5(_)
            | Value::Some6(_)
            | Value::Some7(_) => None,
        }
    }

    /// Returns the one heap child carried by this value for precise GC tracing.
    /// Unlike [`Self::as_obj`], this intentionally sees through an immediate
    /// `Some` wrapper without changing the meaning of ordinary object queries.
    pub fn gc_obj_ref(&self) -> Option<ObjRef> {
        match self {
            Value::Obj(id) => Some(*id),
            Value::Some1(payload)
            | Value::Some2(payload)
            | Value::Some3(payload)
            | Value::Some4(payload)
            | Value::Some5(payload)
            | Value::Some6(payload)
            | Value::Some7(payload) => payload.gc_obj_ref(),
            Value::Nil | Value::Unit | Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::Symbol(_) | Value::None => None,
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
            Value::Unit => "unit",
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Symbol(_) => "symbol",
            Value::Obj(_) => "object",
            Value::None | Value::Some1(_) | Value::Some2(_) | Value::Some3(_) | Value::Some4(_) | Value::Some5(_) | Value::Some6(_) | Value::Some7(_) => {
                "option"
            }
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
            Value::Unit => vm.universe.classes.unit_class,
            Value::Bool(b) => {
                if *b {
                    vm.universe.classes.true_class
                } else {
                    vm.universe.classes.false_class
                }
            }
            Value::Int(_) => vm.universe.classes.int_class,
            Value::Float(_) => vm.universe.classes.float_class,
            Value::Symbol(_) => vm.universe.classes.symbol_class,
            Value::None => vm.universe.classes.none_class,
            Value::Some1(_) | Value::Some2(_) | Value::Some3(_) | Value::Some4(_) | Value::Some5(_) | Value::Some6(_) | Value::Some7(_) => {
                vm.universe.classes.some_class
            }
            Value::Obj(id) => match vm.heap.get(*id) {
                Object::Instance(instance) => instance.class,
                Object::Class(class) => class.class,
                Object::Method(_) => vm.universe.classes.method_class,
                Object::Module(_) => vm.universe.classes.module_class,
                Object::Str(_) => vm.universe.classes.string_class,
                Object::Closure(_) => vm.universe.classes.closure_class,
                // Transitional home-frame wrapper. Public closure values
                // surface as `Closure` until Task Set 4 removes this wrapper.
                Object::Block(_) => vm.universe.classes.closure_class,
                Object::BoundMethod(_) => vm.universe.classes.bound_method_class,
                Object::List(_) => vm.universe.classes.list_class,
                Object::Bytes(_) => vm.universe.classes.bytes_class,
                Object::Fiber(_) => vm.universe.classes.fiber_class,
                Object::Map(_) => vm.universe.classes.map_class,
                Object::Set(_) => vm.universe.classes.set_class,
                Object::Tuple(_) => vm.universe.classes.tuple_class,
                Object::Record(_) => vm.universe.classes.record_class,
                Object::Range(_) => vm.universe.classes.range_class,
                // `::` method reference (selectors.md §3, U16-Open) — reached
                // through `Value::Obj` exactly as `Object::List` is; no
                // `Value::Family` arm (ADR-0010 keeps `Value` minimal).
                Object::Family(_) => vm.universe.classes.family_class,
                Object::SelectorPattern(_) => vm.universe.classes.object_class,
                Object::MethodFamily(_) => vm.universe.classes.object_class,
                Object::LargeInt(_) => vm.universe.classes.int_class,
                Object::Upvalue(_) => panic!("upvalues are not surface values"),
                Object::PackBuilder(_) => panic!("pack builders are not surface values"),
                Object::RecordLiteralBuilder(_) => panic!("Record literal builders are not surface values"),
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
    /// An immediate receiver (`Bool`/`Int`/`Float`/`Symbol`/`Option`/the private `Nil`
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
                | Object::Bytes(_)
                | Object::Fiber(_)
                | Object::Map(_)
                | Object::Set(_)
                | Object::Tuple(_)
                | Object::Record(_)
                | Object::Range(_)
                | Object::Family(_)
                | Object::SelectorPattern(_)
                | Object::MethodFamily(_)
                | Object::LargeInt(_) => CallContext::Instance { instance: *id },
                Object::PackBuilder(_) => panic!("pack builders are not surface receivers"),
                Object::RecordLiteralBuilder(_) => panic!("Record literal builders are not surface receivers"),
                Object::Class(_) => CallContext::Class { class: *id },
                Object::Module(_) => CallContext::Module { module: *id },
                Object::Upvalue(_) => panic!("upvalues are not surface receivers"),
            },
            Value::Nil
            | Value::Unit
            | Value::Bool(_)
            | Value::Int(_)
            | Value::Float(_)
            | Value::Symbol(_)
            | Value::None
            | Value::Some1(_)
            | Value::Some2(_)
            | Value::Some3(_)
            | Value::Some4(_)
            | Value::Some5(_)
            | Value::Some6(_)
            | Value::Some7(_) => CallContext::Immediate { value: *self },
        }
    }

    /// Tests Phalcom value equality (the `==` operator).
    ///
    /// This reproduces, exactly, the observable semantics of the pre-heap
    /// hand-written `impl PartialEq for Value`, so `==`/`!=` behaviour is
    /// preserved across the handle-arena migration
    /// ([ADR-0009](../../../docs/adr/accepted/0009-handle-arena-heap.md)):
    ///
    /// - `Nil`, `Bool`, `Int`, `Float` compare by value.
    /// - Two [`Value::Obj`] strings compare by content; instances, classes and
    ///   methods compare by identity ([`ObjRef`] handle equality).
    /// - [`Value::Symbol`] pairs and [`Object::Module`] pairs are **never**
    ///   equal — they routed to the `_ => false` arm before the migration and
    ///   must keep doing so.
    /// - `None` compares equal only to `None`; equal-depth `Some` values compare
    ///   by their payload, while different wrapper depths are unequal.
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
            (Value::Unit, Value::Unit) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => {
                if a.is_nan() || b.is_nan() {
                    false
                } else {
                    a == b
                }
            }
            (Value::Int(a), Value::Float(b)) => {
                if b.is_nan() || b.is_infinite() || b.fract() != 0.0 {
                    false
                } else {
                    num_bigint::BigInt::from_f64(*b).map(|big| big == num_bigint::BigInt::from(*a)).unwrap_or(false)
                }
            }
            (Value::Float(a), Value::Int(b)) => {
                if a.is_nan() || a.is_infinite() || a.fract() != 0.0 {
                    false
                } else {
                    num_bigint::BigInt::from_f64(*a).map(|big| big == num_bigint::BigInt::from(*b)).unwrap_or(false)
                }
            }
            (Value::Obj(a_ref), Value::Float(b)) | (Value::Float(b), Value::Obj(a_ref)) => {
                if let Some(large_int) = heap.as_large_int(*a_ref) {
                    if b.is_nan() || b.is_infinite() || b.fract() != 0.0 {
                        false
                    } else {
                        num_bigint::BigInt::from_f64(*b).map(|big| &big == large_int).unwrap_or(false)
                    }
                } else {
                    false
                }
            }
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
                // LargeInts compare by BigInt content, regardless of handle.
                match (heap.as_large_int(*a), heap.as_large_int(*b)) {
                    (Some(x), Some(y)) => return x == y,
                    (Some(_), None) | (None, Some(_)) => return false,
                    (None, None) => {}
                }
                // Modules were never equal under the pre-heap `PartialEq`
                // (they fell through to `_ => false`); preserve that.
                if matches!(heap.get(*a), Object::Module(_)) || matches!(heap.get(*b), Object::Module(_)) {
                    return false;
                }
                // Instances, classes and methods compare by identity.
                a == b
            }
            (Value::None, Value::None) => true,
            (Value::Some1(a), Value::Some1(b))
            | (Value::Some2(a), Value::Some2(b))
            | (Value::Some3(a), Value::Some3(b))
            | (Value::Some4(a), Value::Some4(b))
            | (Value::Some5(a), Value::Some5(b))
            | (Value::Some6(a), Value::Some6(b))
            | (Value::Some7(a), Value::Some7(b)) => a.into_value().value_eq(&b.into_value(), heap),
            // Every mismatched pair is unequal.
            _ => false,
        }
    }
}

pub fn same_value_zero(a: Value, b: Value, heap: &crate::heap::Heap) -> bool {
    let a_num = is_numeric(a, heap);
    let b_num = is_numeric(b, heap);
    if a_num && b_num {
        let a_nan = match a {
            Value::Float(f) => f.is_nan(),
            _ => false,
        };
        let b_nan = match b {
            Value::Float(f) => f.is_nan(),
            _ => false,
        };
        if a_nan || b_nan {
            return a_nan && b_nan;
        }

        let a_zero = is_zero(a, heap);
        let b_zero = is_zero(b, heap);
        if a_zero || b_zero {
            return a_zero && b_zero;
        }

        a.value_eq(&b, heap)
    } else {
        a.value_eq(&b, heap)
    }
}

fn is_numeric(val: Value, heap: &crate::heap::Heap) -> bool {
    match val {
        Value::Int(_) | Value::Float(_) => true,
        Value::Obj(id) => heap.as_large_int(id).is_some(),
        _ => false,
    }
}

fn is_zero(val: Value, heap: &crate::heap::Heap) -> bool {
    match val {
        Value::Int(n) => n == 0,
        Value::Float(f) => f == 0.0,
        Value::Obj(id) => heap.as_large_int(id).map(|big| big.is_zero()).unwrap_or(false),
        _ => false,
    }
}

/// Surfaces the private [`Value::Nil`] sentinel as immediate `None`.
///
/// This is the **read-boundary surfacer** of U6's absence model: the private
/// `Value::Nil` sentinel ([ADR-0010](../../../docs/adr/accepted/0010-tagged-value-enum.md))
/// backs uninitialized slots internally but must never reach user code
/// (Invariant 4, `values-and-absence.md`). Every read boundary that can observe
/// an unwritten slot — an uninitialized `var` read, an unassigned field read, a
/// bare-`return` default, a method falling off its end — routes the value
/// through here so the sentinel is replaced by immediate `None`
/// ([ADR-0007](../../../docs/adr/accepted/0007-option-some-none.md)); any non-sentinel
/// value passes through unchanged.
/// There is intentionally **no** public constructor that yields [`Value::Nil`]:
/// surfacing is one-directional (sentinel → `None`), never the reverse, so
/// `None` can never be confused with the private storage sentinel.
#[inline]
#[must_use]
pub fn sentinel_to_option(value: Value) -> Value {
    match value {
        Value::Nil => Value::None,
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
            Value::Unit => 1u8.hash(state),
            Value::Bool(b) => b.hash(state),
            Value::Int(i) => i.hash(state),
            Value::Float(n) => hash_f64(*n, state),
            Value::Symbol(s) => s.0.hash(state),
            Value::Obj(id) => id.hash(state),
            Value::None => 2u8.hash(state),
            Value::Some1(payload) => {
                3u8.hash(state);
                payload.hash(state);
            }
            Value::Some2(payload) => {
                4u8.hash(state);
                payload.hash(state);
            }
            Value::Some3(payload) => {
                5u8.hash(state);
                payload.hash(state);
            }
            Value::Some4(payload) => {
                6u8.hash(state);
                payload.hash(state);
            }
            Value::Some5(payload) => {
                7u8.hash(state);
                payload.hash(state);
            }
            Value::Some6(payload) => {
                8u8.hash(state);
                payload.hash(state);
            }
            Value::Some7(payload) => {
                9u8.hash(state);
                payload.hash(state);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumericPolicy {
    pub max_source_numeric_digits: Option<usize>,
    pub max_text_conversion_digits: Option<usize>,
    pub max_integer_bits: Option<usize>,
    pub max_numeric_allocation_bytes: Option<usize>,
}

impl NumericPolicy {
    pub const fn standard() -> Self {
        Self {
            max_source_numeric_digits: Some(100_000),
            max_text_conversion_digits: Some(100_000),
            max_integer_bits: Some(8_388_608),
            max_numeric_allocation_bytes: Some(2_097_152),
        }
    }

    pub const fn sandbox() -> Self {
        Self {
            max_source_numeric_digits: Some(4_096),
            max_text_conversion_digits: Some(4_096),
            max_integer_bits: Some(262_144),
            max_numeric_allocation_bytes: Some(65_536),
        }
    }
}
