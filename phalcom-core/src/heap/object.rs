use crate::heap::BlockObject;
use crate::heap::ClassObject;
use crate::heap::ClosureObject;
use crate::heap::InstanceObject;
use crate::interner::Symbol;
use crate::heap::ListObject;
use crate::heap::MapObject;
use crate::method::MethodObject;
use crate::heap::ModuleObject;
use crate::heap::RangeObject;
use crate::heap::StringObject;
use crate::heap::TupleObject;
use crate::heap::Upvalue;
use crate::value::Value;

use super::{FiberObject, ObjRef};

/// The tagged payload stored at each live [`ObjRef`] in the [`super::Heap`].
///
/// Every heap-allocated Phalcom object is one of these variants. Immediate
/// values (`nil`, booleans, numbers, interned symbols) are *not* here — they
/// live inline in [`crate::value::Value`] per
/// [ADR-0010](../../../docs/adr/accepted/0010-tagged-value-enum.md).
pub enum Object {
    /// A user-defined object with per-instance fields ([`InstanceObject`]).
    Instance(InstanceObject),
    /// A class or metaclass row in the tower ([`ClassObject`]).
    ///
    /// **Boxed.** `ClassObject` is the fattest payload (280 B measured); the
    /// `SlotMap` sizes every slot to the fattest variant, so leaving it inline
    /// would tax every `Str`/`Tuple`/`Instance` on the hot `Heap::get` path. See
    /// [memory-management.md §7](../../../docs/spec/v0.2/memory-management.md).
    Class(Box<ClassObject>),
    /// A method — primitive or bytecode closure ([`MethodObject`]).
    ///
    /// **Boxed** (88 B) — see [`Object::Class`].
    Method(Box<MethodObject>),
    /// A loaded module and its global slots ([`ModuleObject`]).
    ///
    /// **Boxed** (168 B) — see [`Object::Class`].
    Module(Box<ModuleObject>),
    /// A compiled closure over a [`crate::callable::Callable`] ([`ClosureObject`]).
    ///
    /// **Boxed** (160 B) — see [`Object::Class`].
    Closure(Box<ClosureObject>),
    /// An immutable interned-by-content string ([`StringObject`]).
    Str(StringObject),
    /// A first-class block closure ([`BlockObject`]).
    Block(BlockObject),
    /// A method closed over a receiver — the result of `Method#bind(_)`
    /// ([ADR-0006](../../../docs/adr/accepted/0006-function-as-abstract-callable-root.md)).
    /// Its surface class is `Block`; it responds to the `Function` call
    /// protocol by delegating to [`crate::vm::VM::invoke_method_object`]
    /// (U-CORE-3, [ADR-0028](../../../docs/adr/accepted/0028-amend-floor-admit-method-reflection.md)).
    BoundMethod(BoundMethodObject),
    /// A heap-allocated upvalue cell ([`Upvalue`]).
    Upvalue(Upvalue),
    /// A native array-backed list ([`ListObject`],
    /// [ADR-0020](../../../docs/adr/accepted/0020-kernel-list-native-array-protocol.md)).
    List(ListObject),
    /// A cooperative fiber — the sole concurrency primitive
    /// ([`FiberObject`], [ADR-0030](../../../docs/adr/accepted/0030-fibers-and-futures-cooperative-concurrency.md) §2).
    ///
    /// Reached through [`Value::Obj`] exactly as a
    /// [`Object::List`] is — there is **no** `Value::Fiber` arm (ADR-0030 §2,
    /// forward-compat §7 D2). It owns its own value + call stacks so it can be
    /// suspended and resumed by an O(1) pointer swap of `vm.current`.
    ///
    /// **Boxed** (176 B) — see [`Object::Class`].
    Fiber(Box<FiberObject>),
    /// A native, insertion-ordered hash map ([`MapObject`],
    /// [ADR-0032](../../../docs/adr/accepted/0032-collections-representation-and-literals.md) §1,
    /// [ADR-0039](../../../docs/adr/accepted/0039-amend-floor-admit-collection-container-primitives.md)).
    /// Keyed by **Phalcom** `hash`+`==` (not Rust identity) — see
    /// the `heap::map` module doc. Mutable ⇒ inherits identity `Object#hash`, not a valid
    /// `Map`/`Set` key (Q5, collection-protocol law 4).
    ///
    /// **Boxed** (72 B) — see [`Object::Class`].
    Map(Box<MapObject>),
    /// A native hash set — a keys-only [`MapObject`] (DEC-CT-B): every
    /// [`Object::Set`] value shares [`MapObject`]'s backing struct with
    /// [`Object::Map`], the `.1` (value) slot of each entry always
    /// [`Value::Nil`] and unread by `Set`'s `.ph` protocol. A distinct heap
    /// variant (and distinct raw-primitive bindings) from `Map`, so
    /// `aSet.class == Set`, never `Map`.
    ///
    /// **Boxed** (72 B) — see [`Object::Class`].
    Set(Box<MapObject>),
    /// A native, fixed-arity immutable product ([`TupleObject`],
    /// [ADR-0032](../../../docs/adr/accepted/0032-collections-representation-and-literals.md) §1,
    /// [ADR-0039](../../../docs/adr/accepted/0039-amend-floor-admit-collection-container-primitives.md)).
    /// Immutable ⇒ value-hashable and a valid `Map`/`Set` key (Q5,
    /// collection-protocol law 4) — the opposite corner of the mutability
    /// axis from [`Object::List`]/[`Object::Map`]/[`Object::Set`].
    Tuple(TupleObject),
    /// A native, lazy numeric interval ([`RangeObject`],
    /// [ADR-0032](../../../docs/adr/accepted/0032-collections-representation-and-literals.md) §1,
    /// [ADR-0039](../../../docs/adr/accepted/0039-amend-floor-admit-collection-container-primitives.md)).
    /// Three bound fields, **no element storage** (RG-2 laziness,
    /// `docs/spec/v0.2/core/tuple-and-range.md` §2) — `each`/`toList`
    /// generate elements in `.ph` over the raw getters. Immutable ⇒
    /// value-hashable, a valid `Map`/`Set` key (Q5).
    Range(RangeObject),
    /// A bound `::` method reference — the callable **Family** value
    /// produced by `obj::name` (Open) or `obj::#name(...)` (Pinned)
    /// ([`FamilyObject`], selectors.md §3, U16-Open, U16-Pinned, [ADR-0047]).
    ///
    /// Bound forms only in this unit: `recv` is always a concrete bound
    /// value (no unbound/first-argument form). [`FamilyObject::open`]
    /// discriminates the two reference shapes; see its doc for what
    /// [`FamilyObject::selector`] holds in each.
    ///
    /// [ADR-0047]: ../../../docs/adr/accepted/0047-amend-floor-admit-family-call-router.md
    Family(FamilyObject),
}

/// A bound `::` method reference (selectors.md §3, U16-Open, U16-Pinned).
///
/// Reached through [`Value::Obj`] exactly as an [`Object::List`] is — there
/// is no `Value::Family` arm (`Value` stays minimal, ADR-0010). All fields
/// are `Copy`, so the object itself never needs mutable accessors: a
/// `Family` is immutable once constructed.
#[derive(Debug, Clone, Copy)]
pub struct FamilyObject {
    /// The receiver this family is bound to — `obj` in `obj::name`, or the
    /// class object itself in `Type::name`.
    pub recv: Value,
    /// The symbol carried by this family — its meaning depends on
    /// [`open`](Self::open):
    ///
    /// - **Open** (`open == true`): a bare base method name, not a full
    ///   selector. The selector actually sent is rebuilt at *call* time from
    ///   this name plus the call site's argument labels (selectors.md §3
    ///   "Open families resolve at call time").
    /// - **Pinned** (`open == false`): the complete target selector, already
    ///   interned through [`crate::method::encode_selector`] — the exact
    ///   identity dispatched on every call, independent of the call site's
    ///   own labels (selectors.md §3 "Pinned families have their selector
    ///   fully known at compile time").
    pub selector: Symbol,
    /// Discriminates the two `::` reference shapes this family was built
    /// from — `true` for an Open reference (`obj::name`, U16-Open), `false`
    /// for a Pinned reference (`obj::#name(...)`, U16-Pinned). Drives both
    /// [`crate::vm::VM`]'s reference-time empty-family check and
    /// [`crate::primitive::family::family_does_not_understand`]'s call
    /// router branch.
    pub open: bool,
}

/// The payload of an [`Object::BoundMethod`] — a reified [`MethodObject`]
/// closed over an explicit receiver, the runtime value
/// [`crate::primitive::method::method_bind`] (`Method#bind(_)`, U-CORE-3)
/// constructs.
///
/// Unlike [`BlockObject`], a `BoundMethodObject` carries no closure or
/// home-frame token: it must work for **primitive** methods too (which have
/// no [`ClosureObject`]), and it is not itself a lexical block, so it has no
/// non-local return and introduces no frame-indexing
/// ([ADR-0028](../../../docs/adr/accepted/0028-amend-floor-admit-method-reflection.md)
/// forward-compat §1). Calling it (`bound.call(args)`) and
/// `method.invokeOn(receiver, args)` funnel through the same
/// [`crate::vm::VM::invoke_method_object`] workhorse, so the two are
/// identical by construction (R-INV-3.3).
#[derive(Debug, Clone, Copy)]
pub struct BoundMethodObject {
    /// The wrapped [`Object::Method`] handle.
    pub method: ObjRef,
    /// The receiver this method is closed over.
    pub receiver: Value,
}
