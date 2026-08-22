use crate::heap::BlockObject;
use crate::heap::BytesObject;
use crate::heap::ClassObject;
use crate::heap::ClosureObject;
use crate::heap::InstanceObject;
use crate::heap::ListObject;
use crate::heap::MapObject;
use crate::heap::ModuleObject;
use crate::heap::RangeObject;
use crate::heap::RecordObject;
use crate::heap::StringObject;
use crate::heap::TupleObject;
use crate::heap::Upvalue;
use crate::interner::Symbol;
use crate::method::MethodObject;
use crate::value::Value;
use indexmap::IndexMap;

use super::ArgumentPackBuilderObject;
use super::reflection::{
    ChildModuleTableObject, ExportObject, ExportTableObject, ModuleDependencyObject, ModuleIdentityObject, PackageAuthorObject, PackageIdentityObject,
    PackageInfoObject, PackageRequirementObject, ProjectIdentityObject, ProjectManifestObject, ProjectObject, ResolvedProjectDependencyObject, UriObject,
};
use super::{ClassId, FiberObject, ObjRef, RecordLiteralBuilderObject};

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
    /// Transitional home-frame wrapper for a first-class closure.
    ///
    /// This internal representation surfaces as `Closure`; Task Set 4 removes
    /// the wrapper while preserving non-local-return bookkeeping.
    Block(BlockObject),
    /// A method closed over a receiver — the result of `Method#bind(_)`.
    /// Its surface class is `BoundMethod`; it responds to the `Function` call
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
    /// Reached through [`Value::obj`](crate::value::Value::obj) exactly as a
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
    /// [`crate::value::NIL`] and unread by `Set`'s `.ph` protocol. A distinct heap
    /// variant (and distinct raw-primitive bindings) from `Map`, so
    /// `aSet.class == Set`, never `Map`.
    ///
    /// **Boxed** (72 B) — see [`Object::Class`].
    Set(Box<MapObject>),
    /// A native, fixed-length, mutable octet buffer ([`BytesObject`],
    /// [PDR-0011](../../../docs/decisions/0011-admit-bytes-native-octet-buffer.md)).
    /// `Tuple`'s backing shape (`Box<[u8]>`, length fixed at construction)
    /// with `List`'s mutability corner: contents mutable ⇒ identity
    /// `Object#hash`, not a valid `Map`/`Set` key (collection-protocol
    /// law 4). Reached through [`Value::obj`](crate::value::Value::obj); there is **no** `Value::Bytes`
    /// arm (ADR-0010 minimalism). Holds no [`Value`]s, so the tracer has
    /// nothing to visit, and its drop glue frees plain memory only — no OS
    /// handle lives here, so PDR-0005 §4's back-door-finalizer hazard does
    /// not apply to this arm.
    Bytes(BytesObject),
    /// A native, fixed-arity immutable product ([`TupleObject`],
    /// [ADR-0032](../../../docs/adr/accepted/0032-collections-representation-and-literals.md) §1,
    /// [ADR-0039](../../../docs/adr/accepted/0039-amend-floor-admit-collection-container-primitives.md)).
    /// Immutable ⇒ value-hashable and a valid `Map`/`Set` key (Q5,
    /// collection-protocol law 4) — the opposite corner of the mutability
    /// axis from [`Object::List`]/[`Object::Map`]/[`Object::Set`].
    Tuple(TupleObject),
    /// A native immutable labeled product. Boxed to preserve arena slot size.
    Record(Box<RecordObject>),
    /// A native range bounds descriptor ([`RangeObject`],
    /// [ADR-0032](../../../docs/adr/accepted/0032-collections-representation-and-literals.md) §1,
    /// [ADR-0039](../../../docs/adr/accepted/0039-amend-floor-admit-collection-container-primitives.md)).
    /// Omitted endpoints use the private `Value::Nil` sentinel, preserving the
    /// compact two-Value-plus-flag layout. Progression and equality semantics
    /// are deferred.
    Range(RangeObject),
    /// A bound `::` method reference — the callable **Family** value
    /// produced by `obj::name` (Open) or `obj::#name(...)` (Pinned)
    /// ([`FamilyObject`], selectors.md §3, U16-Open, U16-Pinned, [ADR-0047]).
    ///
    /// Bound forms only in this unit: `receiver` is always a concrete bound
    /// value. The spec is either an exact interned selector or an immutable
    /// selector-pattern object.
    ///
    /// [ADR-0047]: ../../../docs/adr/accepted/0047-amend-floor-admit-family-call-router.md
    Family(FamilyObject),
    /// An exact selector object materialized on demand from an interned symbol.
    Selector(Box<SelectorObject>),
    /// An immutable structural selector pattern compiled from a first-class
    /// selector-spec literal. It is boxed so selector metadata does not grow
    /// every arena slot.
    SelectorPattern(Box<SelectorPatternObject>),
    /// An immutable snapshot of the effective methods selected by a structural
    /// selector pattern. The pattern and captured method handles are retained so
    /// later method replacement cannot change this value's routing.
    MethodFamily(Box<MethodFamilyObject>),
    /// A captured `MethodFamily` closed over a receiver. It selects only from
    /// the immutable snapshot and never performs live receiver lookup.
    BoundMethodFamily(BoundMethodFamilyObject),
    /// An arbitrary-precision integer ([`num_bigint::BigInt`]).
    /// Normalization guarantees this is never representable as `i64`.
    LargeInt(num_bigint::BigInt),
    /// Private compiler/VM-only outgoing argument assembly state.
    PackBuilder(Box<ArgumentPackBuilderObject>),
    /// Private compiler/VM-only dynamic Record literal assembly state.
    RecordLiteralBuilder(Box<RecordLiteralBuilderObject>),
    /// A project development environment ([`ProjectObject`]).
    Project(Box<ProjectObject>),
    /// Validated project development manifest ([`ProjectManifestObject`]).
    ProjectManifest(Box<ProjectManifestObject>),
    /// Durable package information ([`PackageInfoObject`]).
    PackageInfo(Box<PackageInfoObject>),
    /// Package author descriptor ([`PackageAuthorObject`]).
    PackageAuthor(Box<PackageAuthorObject>),
    /// Durable package requirement ([`PackageRequirementObject`]).
    PackageRequirement(Box<PackageRequirementObject>),
    /// Resolved project dependency ([`ResolvedProjectDependencyObject`]).
    ResolvedProjectDependency(Box<ResolvedProjectDependencyObject>),
    /// Module runtime dependency ([`ModuleDependencyObject`]).
    ModuleDependency(Box<ModuleDependencyObject>),
    /// Reflective public export table ([`ExportTableObject`]).
    ExportTable(Box<ExportTableObject>),
    /// Individual reflected export ([`ExportObject`]).
    Export(Box<ExportObject>),
    /// Exposed child module table ([`ChildModuleTableObject`]).
    ChildModuleTable(Box<ChildModuleTableObject>),
    /// Opaque module identity ([`ModuleIdentityObject`]).
    ModuleIdentity(Box<ModuleIdentityObject>),
    /// Opaque package artifact identity ([`PackageIdentityObject`]).
    PackageIdentity(Box<PackageIdentityObject>),
    /// Opaque project identity ([`ProjectIdentityObject`]).
    ProjectIdentity(Box<ProjectIdentityObject>),
    /// Logical URI ([`UriObject`]).
    Uri(Box<UriObject>),
    /// Boxed typing context or descriptor object ([`super::typing::TypingObject`]).
    Typing(Box<super::typing::TypingObject>),
}

/// A bound `::` method reference (selectors.md §3, U16-Open, U16-Pinned).
///
/// Reached through [`Value::obj`](crate::value::Value::obj) exactly as an [`Object::List`] is — there
/// is no `Value::Family` arm (`Value` stays minimal, ADR-0010). All fields
/// are `Copy`, so the object itself never needs mutable accessors: a
/// `Family` is immutable once constructed.
#[derive(Debug, Clone, Copy)]
pub struct FamilyObject {
    /// The receiver this family is bound to — `obj` in `obj::name`, or the
    /// class object itself in `Type::name`.
    pub receiver: Value,
    /// Exact selectors remain compact interned symbols; patterns are heap
    /// objects so their structural predicate can be shared by all calls.
    pub spec: FamilySpec,
}

#[derive(Debug, Clone, Copy)]
pub enum FamilySpec {
    Exact(Symbol),
    Pattern(ObjRef),
}

pub use super::selector::SelectorObject;
pub use super::selector_pattern::SelectorPatternObject;

/// The immutable result of extracting a structural selector pattern from a
/// behavior. Exact bindings preserve declaration/inheritance order; rest
/// candidates preserve subclass-to-superclass fallback order.
#[derive(Debug, Clone)]
pub struct MethodFamilyObject {
    /// Behavior whose effective method dictionaries were scanned at capture.
    pub source_behavior: ClassId,
    /// Immutable selector-pattern object used to define the snapshot.
    pub pattern: ObjRef,
    /// Exact, non-rest methods keyed by canonical selector in capture order.
    pub exact_methods: IndexMap<Symbol, ObjRef>,
    /// Rest methods in subclass-to-superclass precedence order.
    pub rest_candidates: Box<[ObjRef]>,
}

/// A captured method-family snapshot closed over an explicit receiver.
#[derive(Debug, Clone, Copy)]
pub struct BoundMethodFamilyObject {
    /// Immutable MethodFamily snapshot handle.
    pub family: ObjRef,
    /// Receiver supplied by `MethodFamily#bind(_)`.
    pub receiver: Value,
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
