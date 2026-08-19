//! The tagged [`Value`] — Phalcom's uniform in-register value representation.
//!
//! Realizes [ADR-0010](../../../docs/adr/accepted/0010-tagged-value-enum.md) and 16-byte representation.
//! `Value` is a 16-byte `Copy` struct: immediate tags carry their payload inline,
//! and every heap object is carried by an [`ObjRef`] handle ([ADR-0009](../../../docs/adr/accepted/0009-handle-arena-heap.md)).
//! [`crate::value::NIL`] is a **private** uninitialized-slot sentinel with no surface
//! class; user code can never produce or observe it (`values-and-absence.md`
//! Invariant 4). Because every value is `Copy`, values move freely on the VM stack
//! and in constant pools without cloning or reference counting.

mod boolean;
mod nil;
mod option;
mod render;
mod repr;

pub use boolean::{FALSE, TRUE};
pub use nil::NIL;
pub use option::MAX_OPTION_NESTING;
pub(crate) use option::OptionCase;
pub use repr::Value;

use crate::frame::CallContext;
use crate::heap::lookup_method_in_hierarchy;
use crate::heap::{ClassId, ObjRef, Object};
use crate::interner::Symbol;
use crate::vm::VM;
use num_traits::{FromPrimitive, Zero};

/// Normalizes a [`num_bigint::BigInt`] into a [`Value`].
/// Returns [`Value::int`] if representable as `i64`, or allocates a heap [`Object::LargeInt`] otherwise.
pub fn normalize_bigint(bigint: num_bigint::BigInt, heap: &mut crate::heap::Heap) -> Value {
    if let Ok(small) = i64::try_from(&bigint) {
        Value::int(small)
    } else {
        Value::obj(heap.alloc(Object::LargeInt(bigint)))
    }
}

impl Value {
    /// Returns a coarse, heap-free type tag for diagnostics.
    ///
    /// Immediate arms report their exact kind; every [`Value::obj`] reports
    /// `"object"` because discriminating its heap kind would require heap access
    /// (see [`Value::class`] for the precise class). Used by the `expect_value!`
    /// diagnostics macro.
    pub fn type_name(&self) -> &'static str {
        if self.is_option() {
            "option"
        } else if self.is_nil() {
            "nil"
        } else if self.is_unit() {
            "unit"
        } else if self.is_bool() {
            "bool"
        } else if self.is_int() {
            "int"
        } else if self.is_float() {
            "float"
        } else if self.is_symbol() {
            "symbol"
        } else if self.is_obj() {
            "object"
        } else {
            "unknown"
        }
    }

    /// Returns the [`ClassId`] of this value's class.
    ///
    /// Immediates map to their core class; a [`Value::obj`] resolves through the
    /// heap (an instance to its class, a class to its metaclass, a string to
    /// `String`, and so on). Realizes the "every value maps onto a class" rule
    /// (`object-model.md` §3).
    ///
    /// A [`Value::bool`] resolves by its payload to one of the two concrete
    /// singleton subclasses of the abstract `Bool` class — `true` to `True`,
    /// `false` to `False` — so `true.class == True` and `false.class == False`
    /// ([ADR-0004](../../../docs/adr/accepted/0004-boolean-as-abstract-bool-with-true-false.md)).
    /// The selection is a plain [`ClassId`] field read with no allocation, on
    /// the hot dispatch path.
    ///
    /// # Panics
    ///
    /// Panics if the value is a [`Value::obj`] whose handle is stale, or a bare
    /// closure handle (closures are never surface values).
    #[inline]
    pub fn class(&self, vm: &VM) -> ClassId {
        if self.is_none() {
            return vm.universe.classes.none_class;
        }
        if self.is_some() {
            return vm.universe.classes.some_class;
        }
        if self.is_nil() {
            return vm.universe.classes.nil_class;
        }
        if self.is_unit() {
            return vm.universe.classes.unit_class;
        }
        if let Some(b) = self.as_bool() {
            return if b { vm.universe.classes.true_class } else { vm.universe.classes.false_class };
        }
        if self.is_int() {
            return vm.universe.classes.int_class;
        }
        if self.is_float() {
            return vm.universe.classes.float_class;
        }
        if self.is_symbol() {
            return vm.universe.classes.symbol_class;
        }
        if let Some(id) = self.as_obj() {
            return match vm.heap.get(id) {
                Object::Instance(instance) => instance.class,
                Object::Class(class) => class.class,
                Object::Method(_) => vm.universe.classes.method_class,
                Object::Module(module) => match module.kind {
                    crate::heap::ModuleKind::Module => vm.universe.classes.module_class,
                    crate::heap::ModuleKind::Package => vm.universe.classes.package_class,
                },
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
                // through `Value::obj` exactly as `Object::List` is; no
                // `Value::Family` arm (ADR-0010 keeps `Value` minimal).
                Object::Family(_) => vm.universe.classes.family_class,
                Object::SelectorPattern(_) => vm.universe.classes.object_class,
                Object::MethodFamily(_) => vm.universe.classes.method_family_class,
                Object::BoundMethodFamily(_) => vm.universe.classes.bound_method_family_class,
                Object::LargeInt(_) => vm.universe.classes.int_class,
                Object::Project(_) => vm.universe.classes.project_class,
                Object::ProjectManifest(_) => vm.universe.classes.project_manifest_class,
                Object::PackageInfo(_) => vm.universe.classes.package_info_class,
                Object::PackageAuthor(_) => vm.universe.classes.package_author_class,
                Object::PackageRequirement(_) => vm.universe.classes.package_requirement_class,
                Object::ResolvedProjectDependency(_) => vm.universe.classes.resolved_project_dependency_class,
                Object::ModuleDependency(_) => vm.universe.classes.module_dependency_class,
                Object::ExportTable(_) => vm.universe.classes.export_table_class,
                Object::Export(_) => vm.universe.classes.export_class,
                Object::ChildModuleTable(_) => vm.universe.classes.child_module_table_class,
                Object::ModuleIdentity(_) => vm.universe.classes.module_identity_class,
                Object::PackageIdentity(_) => vm.universe.classes.package_identity_class,
                Object::ProjectIdentity(_) => vm.universe.classes.project_identity_class,
                Object::Uri(_) => vm.universe.classes.uri_class,
                Object::Upvalue(_) => panic!("upvalues are not surface values"),
                Object::PackBuilder(_) => panic!("pack builders are not surface values"),
                Object::RecordLiteralBuilder(_) => panic!("Record literal builders are not surface values"),
            };
        }
        panic!("invalid Value tag in class()")
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
        if let Some(id) = self.as_obj() {
            match heap.get(id) {
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
                | Object::LargeInt(_)
                | Object::BoundMethodFamily(_)
                | Object::Project(_)
                | Object::ProjectManifest(_)
                | Object::PackageInfo(_)
                | Object::PackageAuthor(_)
                | Object::PackageRequirement(_)
                | Object::ResolvedProjectDependency(_)
                | Object::ModuleDependency(_)
                | Object::ExportTable(_)
                | Object::Export(_)
                | Object::ChildModuleTable(_)
                | Object::ModuleIdentity(_)
                | Object::PackageIdentity(_)
                | Object::ProjectIdentity(_)
                | Object::Uri(_) => CallContext::Instance { instance: id },
                Object::PackBuilder(_) => panic!("pack builders are not surface receivers"),
                Object::RecordLiteralBuilder(_) => panic!("Record literal builders are not surface receivers"),
                Object::Class(_) => CallContext::Class { class: id },
                Object::Module(_) => CallContext::Module { module: id },
                Object::Upvalue(_) => panic!("upvalues are not surface receivers"),
            }
        } else {
            CallContext::Immediate { value: *self }
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
    /// - Two [`Value::obj`] strings compare by content; instances, classes and
    ///   methods compare by identity ([`ObjRef`] handle equality).
    /// - [`Value::symbol`] pairs compare by **interned identity**: two symbols
    ///   are equal if and only if they have the same interned `u32` id, which
    ///   happens when they name the same string. Two symbols for different strings
    ///   are never equal.
    /// - [`Object::Module`] pairs compare by identity ([`ObjRef`] handle equality).
    /// - `None` compares equal only to `None`; equal-depth `Some` values compare
    ///   by their payload, while different wrapper depths are unequal.
    /// - Every mismatched or otherwise unhandled pair is unequal.
    pub fn value_eq(&self, other: &Value, heap: &crate::heap::Heap) -> bool {
        use crate::heap::Object;

        if self.is_option() || other.is_option() {
            if self.option_depth() != other.option_depth() {
                return false;
            }
            if self.is_none() && other.is_none() {
                return true;
            }
            if self.is_some() && other.is_some() {
                return self.without_some_wrappers().value_eq(&other.without_some_wrappers(), heap);
            }
            return false;
        }

        if self.is_nil() && other.is_nil() {
            return true;
        }
        if self.is_unit() && other.is_unit() {
            return true;
        }
        if let (Some(a), Some(b)) = (self.as_bool(), other.as_bool()) {
            return a == b;
        }
        if let (Some(a), Some(b)) = (self.as_int(), other.as_int()) {
            return a == b;
        }
        if let (Some(a), Some(b)) = (self.as_float(), other.as_float()) {
            return if a.is_nan() || b.is_nan() { false } else { a == b };
        }
        if let (Some(a), Some(b)) = (self.as_int(), other.as_float()) {
            return if b.is_nan() || b.is_infinite() || b.fract() != 0.0 {
                false
            } else {
                num_bigint::BigInt::from_f64(b).map(|big| big == num_bigint::BigInt::from(a)).unwrap_or(false)
            };
        }
        if let (Some(a), Some(b)) = (self.as_float(), other.as_int()) {
            return if a.is_nan() || a.is_infinite() || a.fract() != 0.0 {
                false
            } else {
                num_bigint::BigInt::from_f64(a).map(|big| big == num_bigint::BigInt::from(b)).unwrap_or(false)
            };
        }
        if let (Some(a_ref), Some(b)) = (self.as_obj(), other.as_float()) {
            return if let Some(large_int) = heap.as_large_int(a_ref) {
                if b.is_nan() || b.is_infinite() || b.fract() != 0.0 {
                    false
                } else {
                    num_bigint::BigInt::from_f64(b).map(|big| &big == large_int).unwrap_or(false)
                }
            } else {
                false
            };
        }
        if let (Some(b), Some(a_ref)) = (self.as_float(), other.as_obj()) {
            return if let Some(large_int) = heap.as_large_int(a_ref) {
                if b.is_nan() || b.is_infinite() || b.fract() != 0.0 {
                    false
                } else {
                    num_bigint::BigInt::from_f64(b).map(|big| &big == large_int).unwrap_or(false)
                }
            } else {
                false
            };
        }
        if let (Some(a), Some(b)) = (self.symbol_value(), other.symbol_value()) {
            return a == b;
        }
        if let (Some(a), Some(b)) = (self.as_obj(), other.as_obj()) {
            // Strings compare by content, regardless of handle.
            match (heap.as_string(a), heap.as_string(b)) {
                (Some(x), Some(y)) => return x == y,
                (Some(_), None) | (None, Some(_)) => return false,
                (None, None) => {}
            }
            // LargeInts compare by BigInt content, regardless of handle.
            match (heap.as_large_int(a), heap.as_large_int(b)) {
                (Some(x), Some(y)) => return x == y,
                (Some(_), None) | (None, Some(_)) => return false,
                (None, None) => {}
            }
            // Modules were never equal under the pre-heap `PartialEq`
            // (they fell through to `_ => false`); preserve that.
            if matches!(heap.get(a), Object::Module(_)) || matches!(heap.get(b), Object::Module(_)) {
                return false;
            }
            // Instances, classes and methods compare by identity.
            return a == b;
        }

        false
    }
}

pub fn same_value_zero(a: Value, b: Value, heap: &crate::heap::Heap) -> bool {
    let a_num = is_numeric(a, heap);
    let b_num = is_numeric(b, heap);
    if a_num && b_num {
        let a_nan = a.as_float().map(|f| f.is_nan()).unwrap_or(false);
        let b_nan = b.as_float().map(|f| f.is_nan()).unwrap_or(false);
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
    if val.is_int() || val.is_float() {
        true
    } else if let Some(id) = val.as_obj() {
        heap.as_large_int(id).is_some()
    } else {
        false
    }
}

fn is_zero(val: Value, heap: &crate::heap::Heap) -> bool {
    if let Some(n) = val.as_int() {
        n == 0
    } else if let Some(f) = val.as_float() {
        f == 0.0
    } else if let Some(id) = val.as_obj() {
        heap.as_large_int(id).map(|big| big.is_zero()).unwrap_or(false)
    } else {
        false
    }
}

/// Surfaces the private [`crate::value::NIL`] sentinel as immediate `None`.
///
/// This is the **read-boundary surfacer** of U6's absence model: the private
/// `Value::nil()` sentinel ([ADR-0010](../../../docs/adr/accepted/0010-tagged-value-enum.md))
/// backs uninitialized slots internally but must never reach user code
/// (Invariant 4, `values-and-absence.md`). Every read boundary that can observe
/// an unwritten slot — an uninitialized `var` read, an unassigned field read, a
/// bare-`return` default, a method falling off its end — routes the value
/// through here so the sentinel is replaced by immediate `None`
/// ([ADR-0007](../../../docs/adr/accepted/0007-option-some-none.md)); any non-sentinel
/// value passes through unchanged.
#[inline]
#[must_use]
pub fn sentinel_to_option(value: Value) -> Value {
    if value.is_nil() { Value::none() } else { value }
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
