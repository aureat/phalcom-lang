# Phalcom VM Values and Objects

**Status:** Draft 0.1 — authoritative implementation-internals specification  
**Specification domain:** `spec/vm/`  
**Implementation:** Phalcom reference runtime (`phalcom-core`)  
**Companion specifications:** `bytecode.md`, `execution-model.md`

---

## 1. Scope

This document specifies the runtime representation of values and objects in the Phalcom reference virtual machine.

It defines the implementation boundary between:

- the uniform `Value` carried by the operand stack, locals, fields, constants, and native interfaces;
- immediate values encoded entirely inside `Value`;
- heap-reference values encoded by `ObjRef`;
- objects stored in the VM's central `Heap`;
- runtime class identity;
- representation normalization;
- object identity;
- representation equality and exact sameness;
- value-semantic product/data representations;
- garbage-collector visibility of references embedded in values and objects.

This is an **implementation-internals specification**. It intentionally names concrete Rust types and representation choices where those choices are architectural contracts of the reference runtime.

The central rule of this document is:

> **`Value` is the VM's universal carrier; `Object` is one possible referent of that carrier; runtime class and semantic identity are deliberately independent of either physical representation.**

A Phalcom runtime value is therefore not synonymous with a heap object.

An immediate integer is a value but not a heap object. A class is both a value and a heap object. A payload-bearing data value is a heap object, while a nullary value of the same data declaration may be represented without allocation. `Some(x)` is normally represented by metadata on the existing `Value` rather than by a wrapper object.

This distinction is foundational to the VM.

---

## 2. Normative language

The terms **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, and **MAY** express requirements on the reference implementation.

This document distinguishes three kinds of statement.

### 2.1 Representation invariant

A representation invariant is a property on which VM subsystems may rely.

Examples include:

- `Value` is exactly 16 bytes;
- an `ObjRef` is a generational arena handle, not a Rust pointer;
- the private `Nil` sentinel never crosses a guest-visible read boundary;
- `Some` metadata does not hide an underlying heap reference from GC;
- an internal Rust `Box<T>` is not Phalcom object identity.

Changing a representation invariant requires coordinated review of every dependent VM subsystem.

### 2.2 Current representation mechanism

A current representation mechanism is the concrete implementation of an invariant.

Examples include:

- `SlotMap<ObjRef, Object>` as the heap arena;
- two `u64` words for `Value`;
- a `u32` `Some` nesting field inside the metadata word;
- `Box<[Value]>` for ordinary instance fields;
- `ProductStorage` as a packed word buffer.

Such mechanisms are authoritative descriptions of the current implementation, but they may be replaced if the replacement preserves or deliberately amends the architectural contract.

### 2.3 Optimization

An optimization changes physical cost without changing the semantic value/object model.

Examples include:

- boxing large `Object` variants so they do not inflate every arena slot;
- packing scalar product components into one 64-bit word;
- representing nullary ADT/data values immediately;
- representing small integers inline while allocating only large integers.

Optimizations MUST preserve the representation-independent contracts explicitly defined here.

---

# Part I — Representation strata

## 3. Runtime value versus heap object

Every guest-visible runtime value is carried as a `Value`.

Only values whose base tag is `Obj` contain an `ObjRef` pointing into the heap.

Conceptually:

```text
                              Value
                                │
                  ┌─────────────┴─────────────┐
                  │                           │
             immediate                    heap-backed
                  │                           │
                  │                       base tag Obj
                  │                           │
                  │                         ObjRef
                  │                           │
                  │                          Heap
                  │                           │
                  │                         Object
                  │
                  ├── Unit
                  ├── Bool
                  ├── Int
                  ├── Float
                  ├── Symbol
                  ├── None
                  ├── Some metadata
                  ├── ADT singleton
                  └── data singleton
```

There is deliberately no dedicated `Value` tag for every runtime object category.

For example, there is no `Value::Fiber`, `Value::List`, `Value::Class`, `Value::Tuple`, or `Value::Method` representation. These are `Value`s whose base tag is `Obj` and whose `ObjRef` resolves to the corresponding `Object` payload.

---

## 4. Physical representation and runtime class are different axes

The runtime MUST NOT infer behavioral class identity merely from the physical representation category.

Examples include:

```text
immediate Int ─────────────┐
                           ├── class Int
Object::LargeInt ──────────┘

Value::AdtSingleton ───────┐
                           ├── variant behavior class
Object::AdtCase ───────────┘

Value::DataSingleton ──────┐
                           ├── data behavior class
Object::Data ──────────────┘

Bool(true)  ─────────────────── class True
Bool(false) ─────────────────── class False
```

A physical representation is an implementation strategy.

A runtime class is a behavioral identity.

The mapping between them is specified by `Value::class` and the runtime registries.

---

# Part II — The uniform `Value`

## 5. `Value` is exactly two machine-independent 64-bit words

The current representation is:

```rust
#[repr(C)]
pub struct Value {
    payload: u64,
    meta: u64,
}
```

The representation size is exactly:

```text
16 bytes
```

On supported 64-bit targets its current alignment is 8 bytes.

`Value` is `Copy`.

Copying a `Value` copies the two representation words. It does not clone or retain a heap object through reference counting.

If the copied value contains an `ObjRef`, both copies name the same heap object.

---

## 6. Metadata word layout

The `meta` word is currently partitioned as follows:

```text
bit 63                                      bit 0
┌──────────────────────┬────────────────────┬──────────┐
│ reserved             │ Some depth         │ tag      │
│ bits 40..=63         │ bits 8..=39        │ 0..=7    │
│ 24 bits              │ 32 bits            │ 8 bits   │
└──────────────────────┴────────────────────┴──────────┘
```

Equivalently:

```text
tag         = meta bits 0..=7
some_depth  = meta bits 8..=39
reserved    = meta bits 40..=63
```

The reserved bits MUST be zero in every currently valid `Value`.

No subsystem may use reserved bits as private metadata without amending this specification.

---

## 7. Current value tags

The current tag set is:

| Numeric tag | `ValueTag` | Base payload meaning |
|---:|---|---|
| 0 | `Nil` | private VM sentinel |
| 1 | `Unit` | no payload |
| 2 | `Bool` | `0` false, nonzero true; constructors emit 0/1 |
| 3 | `Int` | signed `i64` bit pattern |
| 4 | `Float` | exact IEEE-754 `f64` bit pattern |
| 5 | `Symbol` | interned `Symbol` id |
| 6 | `Obj` | opaque encoded `ObjRef` |
| 7 | `None` | surface Option absence |
| 8 | `AdtSingleton` | `RuntimeVariantId` |
| 9 | `DataSingleton` | `RuntimeDataDescriptorId` |

Adding a new `ValueTag` requires review of:

- constructors and checked accessors;
- `Value::class`;
- representation equality/hash;
- rendering;
- `Option` wrapping;
- GC edge extraction;
- serialization/debugging seams, if any;
- this specification.

---

## 8. Base value and Option wrapping

The tag and payload describe the **base value**.

The `Some` depth describes zero or more immediate Option wrappers around that base.

Thus one physical `Value` can represent:

```text
x
Some(x)
Some(Some(x))
...
```

without allocating wrapper objects.

The base tag alone therefore does not always identify the surface runtime category.

---

## 9. Exact raw representation

The representation-level exact relation compares:

```text
payload equality
AND
meta equality
```

The implementation exposes this through `same_bits` and `Value::same_as`.

This relation is referred to in this document as **raw representation sameness**.

It is not, by itself, the complete language-level `===` relation. See Part XV.

---

# Part III — Immediate values

## 10. Unit

Unit is represented as:

```text
tag        = Unit
payload    = 0
some_depth = 0
```

The empty tuple `()` normalizes to this immediate representation.

The VM MUST NOT allocate a `TupleObject` merely to represent the empty tuple.

This establishes:

```text
() ≡ Unit immediate representation
```

at the runtime representation level.

---

## 11. Boolean

A Boolean is represented as:

```text
tag        = Bool
payload    = 0 for false
             1 for true
some_depth = 0
```

`true` and `false` are immediate values.

Their runtime class mapping is payload-sensitive:

```text
true.class  → True
false.class → False
```

`Bool` is not the direct concrete class returned for either Boolean payload.

---

## 12. Small integer

An integer representable by `i64` is:

```text
tag        = Int
payload    = value interpreted as i64 bits
some_depth = 0
```

This is the canonical representation of every `i64`-representable Phalcom integer.

The runtime MUST normalize a mathematically equal arbitrary-precision integer back to this representation whenever it fits in `i64`.

---

## 13. Float

A floating-point value is:

```text
tag        = Float
payload    = f64::to_bits(value)
some_depth = 0
```

The VM preserves the exact IEEE-754 bit pattern when storing and loading the representation.

Distinct NaN payloads therefore survive a representation round-trip.

Language equality and hashing do not necessarily use bitwise float equivalence; see Part XV.

---

## 14. Symbol

A Symbol is:

```text
tag        = Symbol
payload    = interned Symbol identifier
some_depth = 0
```

The payload is an interner identity, not an `ObjRef`.

Symbols are therefore not heap objects solely by virtue of being runtime values.

The interner owns the corresponding symbol/name storage.

---

## 15. `None`

The surface absence value is:

```text
tag        = None
payload    = 0
some_depth = 0
```

`None` is a guest-visible runtime value.

It is distinct from the private `Nil` sentinel.

---

## 16. Immediate ADT singleton

A nullary enum variant may be represented as:

```text
tag        = AdtSingleton
payload    = RuntimeVariantId
some_depth = 0
```

The payload identifies the runtime variant descriptor, not a heap object.

The runtime variant descriptor determines the behavior class of the value.

---

## 17. Immediate data singleton

A nullary `data` value may be represented as:

```text
tag        = DataSingleton
payload    = RuntimeDataDescriptorId
some_depth = 0
```

The payload identifies the runtime data descriptor.

The descriptor determines the value's behavior class.

This representation is distinct from `AdtSingleton`; the two registries and semantic identities are not interchangeable.

---

# Part IV — Immediate `Option`

## 18. Representation

`Option` uses the 32-bit `some_depth` field in every `Value`.

A value with:

```text
some_depth > 0
```

is a `Some` value regardless of its base tag.

A value with:

```text
tag == None
some_depth == 0
```

is exactly `None`.

---

## 19. Examples

For an integer:

```text
42
    tag        = Int
    payload    = 42
    some_depth = 0

Some(42)
    tag        = Int
    payload    = 42
    some_depth = 1

Some(Some(42))
    tag        = Int
    payload    = 42
    some_depth = 2
```

For `None`:

```text
None
    tag        = None
    payload    = 0
    some_depth = 0

Some(None)
    tag        = None
    payload    = 0
    some_depth = 1

Some(Some(None))
    tag        = None
    payload    = 0
    some_depth = 2
```

`None` and `Some(None)` are therefore physically and semantically distinct values.

---

## 20. Wrapper limit

The current maximum representable `Some` nesting depth is:

```text
u32::MAX
```

Adding one wrapper past that depth fails with the runtime Option nesting-limit error.

This is a representation limit.

---

## 21. Peeling one wrapper

Peeling one `Some` layer decrements `some_depth` by exactly one while preserving the base payload and tag.

A depth-one value therefore yields its base value.

A depth-N value yields depth N-1.

`None` is handled as its own case and is not a wrapped payload.

---

## 22. Private `Nil` cannot be wrapped

The private VM `Nil` sentinel MUST NOT be converted into `Some(Nil)`.

`wrap_some(Value::nil())` is an internal runtime error.

This preserves the invariant that `Nil` has no surface-language representation.

---

## 23. Ordinary object access versus GC object access

A wrapped object demonstrates a critical distinction.

Suppose:

```text
x = Value::obj(object)
y = Some(x)
```

Then `y` has:

```text
base tag   = Obj
payload    = object ObjRef
some_depth = 1
```

For ordinary surface access:

```text
y.as_obj() == None
```

because the surface value is `Some`, not the raw object.

For garbage collection:

```text
y.gc_obj_ref() == Some(object)
```

because the underlying heap edge remains live.

The implementation MUST preserve this distinction.

Option wrapping may hide the base representation from normal value accessors, but it MUST NOT hide the underlying heap reference from tracing.

---

## 24. Option runtime class

Class mapping tests Option wrapping before inspecting the base representation:

```text
None     → class None
Some(_)  → class Some
```

Thus:

```text
Some(42).class
Some(object).class
Some(None).class
```

all resolve through the `Some` behavior class rather than through the class of the wrapped base value.

---

# Part V — Private `Nil` and surface absence

## 25. `Nil` is an implementation sentinel

The private sentinel is:

```text
tag        = Nil
payload    = 0
some_depth = 0
```

It has no source literal and is not a guest-visible absence value.

It is used where the runtime needs an efficient "not initialized / not present in storage" marker.

Examples include:

- newly allocated ordinary instance fields;
- uninitialized VM/local storage;
- selected compact internal payload fields;
- default internal return state before surfacing.

---

## 26. `Nil` versus `None`

The two representations have different purposes:

```text
Nil
    private implementation state

None
    surface-language Option absence
```

They MUST NOT be conflated.

---

## 27. Read-boundary surfacing

A VM read boundary that can expose an unwritten internal slot MUST transform:

```text
Nil → None
```

before returning the value to guest code.

The central helper is:

```text
sentinel_to_option(value)
```

which returns immediate `None` only when `value` is the private sentinel.

All other values pass through unchanged.

---

## 28. Surface-absence invariant

The implementation MUST preserve:

> A guest-visible execution result, field/local read, ordinary return value, or other surface value MUST NOT be the private `Nil` representation.

The execution model depends on this rule for:

- bare return;
- falling off the end of a method;
- reads of uninitialized fields/locals;
- frame-floor completion.

---

## 29. Internal class mapping of `Nil`

The runtime contains a kernel `nil_class` mapping for internal machinery.

That mapping does not make `Nil` a legal guest value.

The surface non-escape invariant takes precedence over the existence of internal behavior metadata.

---

# Part VI — Heap references

## 30. `Value::Obj`

An unwrapped heap-reference value is:

```text
tag        = Obj
payload    = opaque ObjRef bits
some_depth = 0
```

The payload does not contain a Rust memory address.

It encodes a generational key into the VM's central heap arena.

---

## 31. `ObjRef`

`ObjRef` is a `Copy` generational handle.

Conceptually:

```text
ObjRef
    ├── arena slot identity
    └── generation identity
```

The exact `slotmap` private bit encoding is not a language/runtime ABI.

The architectural properties are:

- it is cheap to copy;
- it is cheap to hash and compare;
- live-handle equality is heap object identity;
- dereference goes through `Heap`;
- stale handles cannot silently identify a replacement object in a reused slot.

---

## 32. `ClassId`

`ClassId` is currently:

```rust
pub type ClassId = ObjRef;
```

It is an intent/documentation alias rather than a second handle mechanism.

Therefore:

```text
InstanceObject.class
ClassObject.class
ClassObject.superclass
```

all use the same generational object-reference model as other heap edges.

---

## 33. Heap identity is not pointer identity

A Phalcom heap object's identity is its live `ObjRef`.

If an `Object` variant stores its Rust payload in a `Box<T>`, the address of that `Box<T>` is not a second identity and MUST NOT be used as language-visible or runtime-stable object identity.

The following:

```text
ObjRef → Object::Class(Box<ClassObject>)
```

contains one Phalcom object identity: the `ObjRef`.

---

## 34. Stale handles

The generational arena gives stale handles defined non-aliasing behavior.

A handle to an object that has been reclaimed MUST NOT resolve to an unrelated later object that reuses the same physical arena slot.

Trusted internal accessors may panic when given a stale handle because a stale reference is an invariant violation on that path.

Staleness-sensitive paths may use checked lookup and receive no referent.

The memory-management specification owns reclamation timing.

---

# Part VII — The central heap

## 35. One heap per VM

A `VM` owns exactly one `Heap`.

The heap owns all `Object` payloads allocated into its object arena.

The core storage is conceptually:

```text
SlotMap<ObjRef, Object>
```

Objects refer to other objects using handles.

They do not establish Rust ownership graphs among one another.

---

## 36. Cyclic runtime graphs

Because object edges use `ObjRef`, cyclic language/runtime graphs are ordinary.

This includes class/metaclass bootstrap cycles.

For example, the metaclass tower may contain a self-cycle without requiring `Rc`, `Weak`, or per-object `RefCell` ownership tricks.

The arena owns the payloads; handles express graph edges.

---

## 37. Allocation

Heap allocation:

1. inserts a new `Object`;
2. receives a fresh live `ObjRef`;
3. may latch that garbage collection is due;
4. does not itself arbitrarily run collection in the middle of an opcode.

GC scheduling is specified by `execution-model.md` and the memory-management specification.

---

# Part VIII — `Object` taxonomy

## 38. `Object` is the tagged heap payload

Each live heap slot contains exactly one `Object` variant.

Immediate values are not represented by an `Object` merely because they are first-class Phalcom values.

The current object taxonomy is exhaustive below.

---

## 39. Ordinary runtime, callable, collection, and execution objects

| `Object` variant | Physical payload | Surface category | Mutability / role |
|---|---|---|---|
| `Instance` | inline `InstanceObject` | ordinary class instance | field slots mutable |
| `Class` | boxed `ClassObject` | class/metaclass object | runtime behavior/layout metadata |
| `Method` | boxed `MethodObject` | reflected Method | callable metadata |
| `Module` | boxed `ModuleObject` | Module or Package | globals/linkage state |
| `Closure` | boxed `ClosureObject` | Closure | bytecode callable + captures |
| `Str` | inline `StringObject` | String | immutable value |
| `Block` | inline `BlockObject` | currently surfaces as Closure | transitional home-frame wrapper |
| `BoundMethod` | inline `BoundMethodObject` | BoundMethod | method + receiver capability |
| `Upvalue` | inline `Upvalue` | **internal-only** | mutable capture cell |
| `List` | inline `ListObject` | List | mutable sequence |
| `Fiber` | boxed `FiberObject` | Fiber | mutable execution object |
| `Map` | boxed `MapObject` | Map | mutable associative collection |
| `Set` | boxed `MapObject` | Set | mutable set representation |
| `Bytes` | inline `BytesObject` | Bytes | mutable octet buffer |
| `Tuple` | inline `TupleObject` | Tuple | immutable product |
| `Record` | boxed `RecordObject` | Record | immutable labeled product |
| `Range` | inline `RangeObject` | Range | bounds descriptor |
| `Family` | inline `FamilyObject` | Family | bound callable-family capability |
| `Selector` | boxed `SelectorObject` | Selector | immutable descriptor |
| `SelectorPattern` | boxed `SelectorPatternObject` | SelectorPattern | immutable descriptor |
| `MethodFamily` | boxed `MethodFamilyObject` | MethodFamily | immutable captured method snapshot |
| `BoundMethodFamily` | inline `BoundMethodFamilyObject` | BoundMethodFamily | family + receiver |
| `LargeInt` | inline `BigInt` | Int | immutable arbitrary-precision integer |
| `PackBuilder` | boxed `ArgumentPackBuilderObject` | **internal-only** | dynamic invocation construction |
| `RecordLiteralBuilder` | boxed builder | **internal-only** | dynamic record construction |

---

## 40. Project, module-reflection, and descriptor objects

| `Object` variant | Surface class / role |
|---|---|
| `Project` | project development object |
| `ProjectManifest` | validated project manifest |
| `PackageInfo` | durable package information |
| `PackageAuthor` | package author descriptor |
| `PackageRequirement` | package requirement descriptor |
| `ResolvedProjectDependency` | resolved project dependency |
| `ModuleDependency` | runtime module dependency descriptor |
| `ExportTable` | reflective export table |
| `Export` | reflected export |
| `ChildModuleTable` | reflected child-module table |
| `ModuleIdentity` | opaque module identity |
| `PackageIdentity` | opaque package identity |
| `ProjectIdentity` | opaque project identity |
| `Uri` | logical URI |
| `Typing` | runtime typing context/descriptor |

These are ordinary arena objects when surfaced. Their APIs and semantic meanings belong to the module/reflection/typing specifications.

---

## 41. Algebraic/data/capability objects

| `Object` variant | Surface role |
|---|---|
| `AdtCase` | payload-bearing enum variant value |
| `Data` | materialized positive-arity data value |
| `AssociatedFamily` | reified associated callable-family capability |

`AdtCase` and `Data` use packed product storage.

---

## 42. Surface object versus internal object

Heap allocation does not imply surface objecthood.

The current explicitly non-surface object variants include at least:

```text
Upvalue
PackBuilder
RecordLiteralBuilder
```

Passing one of these through the generic surface-class/receiver boundary is an implementation error.

The runtime therefore treats class lookup or receiver conversion for these variants as unreachable/panic conditions.

---

## 43. Internal-object non-escape invariant

Every heap object that can legally become a guest-visible `Value::Obj` MUST have a defined surface runtime class.

Every heap object without a defined surface class MUST be prevented from escaping VM-internal construction/execution paths.

This rule applies equally to future temporary heap objects.

---

# Part IX — Boxing inside the arena

## 44. Why some `Object` variants contain `Box<T>`

The backing `SlotMap` stores one `Object` enum per slot.

An enum slot must accommodate its largest inline variant.

Large payloads are therefore boxed when leaving them inline would increase the physical size of every heap slot.

Current examples include class, method, module, closure, fiber, map/set, record, many reflection descriptors, ADT/data payload objects, and other relatively large structures.

---

## 45. Boxing is not semantic allocation

The internal `Box<T>`:

- is an implementation storage optimization;
- is owned by its containing `Object` arena slot;
- does not create another Phalcom object;
- has no separately observable object identity.

Only the arena's `ObjRef` is the object identity.

---

# Part X — Runtime class mapping

## 46. `classOf`

The runtime bridge from representation to behavior is conceptually:

```text
classOf : Value × VM → ClassId
```

implemented by `Value::class`.

Class mapping is allowed to inspect:

- the immediate tag/payload;
- Option depth;
- the referent of an `ObjRef`;
- ADT/data runtime descriptor registries.

---

## 47. Immediate class mapping

| Value form | Runtime class |
|---|---|
| `None` | `None` class |
| any `Some(...)` | `Some` class |
| `AdtSingleton(rid)` | registered variant behavior class |
| `DataSingleton(did)` | registered data behavior class |
| private `Nil` | internal nil class; MUST NOT surface |
| `Unit` | Unit class |
| `Bool(true)` | True class |
| `Bool(false)` | False class |
| `Int` | Int class |
| `Float` | Float class |
| `Symbol` | Symbol class |

If an ADT/data runtime descriptor is unexpectedly unavailable, the current implementation falls back to the root Object class. A properly registered executable value SHOULD have its descriptor available.

---

## 48. Heap-object class mapping

Current heap mappings include:

| `Object` | Runtime class source |
|---|---|
| `Instance` | `InstanceObject.class` |
| `Class` | `ClassObject.class` (the metaclass) |
| `Method` | kernel Method class |
| `Module` | Module or Package according to `ModuleKind` |
| `Str` | String |
| `Closure` | Closure |
| transitional `Block` | Closure |
| `BoundMethod` | BoundMethod |
| `List` | List |
| `Bytes` | Bytes |
| `Fiber` | Fiber |
| `Map` | Map |
| `Set` | Set |
| `Tuple` | Tuple |
| `Record` | Record |
| `Range` | Range |
| `Family` | Family |
| `Selector` | Selector |
| `SelectorPattern` | SelectorPattern |
| `MethodFamily` | MethodFamily |
| `BoundMethodFamily` | BoundMethodFamily |
| `LargeInt` | Int |
| project/package/reflection variants | corresponding kernel reflection class |
| `Typing` | class stored by the `TypingObject` |
| `AdtCase` | registered variant behavior class |
| `Data` | registered data behavior class |
| `AssociatedFamily` | Family |

Internal-only `Upvalue`, `PackBuilder`, and `RecordLiteralBuilder` do not have a valid surface class path.

---

## 49. Representation-independent behavior identity

The following pairs illustrate one behavior identity with multiple physical representations:

```text
Int immediate
LargeInt object
    → Int behavior

AdtSingleton
AdtCase object
    → one variant behavior class for the corresponding runtime variant

DataSingleton
Data object
    → one data behavior class for the corresponding runtime descriptor
```

Dispatch and reflection MUST be based on runtime behavior identity rather than on assumptions that each class has exactly one storage form.

---

# Part XI — Ordinary instances

## 50. `InstanceObject`

A general user-defined instance is:

```rust
InstanceObject {
    class: ClassId,
    slots: Box<[Value]>,
}
```

Its class handle and fields are stored directly in the heap payload.

---

## 51. Fixed field slots

Instance fields are represented by physical slots.

Field names are not repeated in each instance.

The class/runtime layout maps field names to `u16` offsets, and each instance owns only the fixed-size value array required by that layout.

This avoids per-instance hash-table field lookup.

---

## 52. Instance initialization

A fresh ordinary instance allocates exactly `field_count` slots.

Every slot begins as:

```text
Value::Nil
```

The private sentinel represents "not assigned yet" in storage.

A guest-visible field read MUST surface it as `None`.

---

## 53. Class link

`InstanceObject.class` is the runtime behavior identity used by `Value::class`.

It is a `ClassId`, and therefore an `ObjRef`.

The instance keeps its class alive through the normal heap tracing edge.

---

# Part XII — Class objects as heap objects

## 54. `ClassObject`

A class or metaclass is itself an `Object::Class`.

Representation-critical fields currently include:

```text
name
class
superclass
methods
rest_methods
field_slots
field_count
static_slots
base_names
attributes
attributes_frozen
native_repr
is_abstract
```

This document specifies their role as stored runtime representation.

Detailed lookup and metaclass semantics belong to later specifications.

---

## 55. Metaclass link

`ClassObject.class` points to the class object's own runtime class—its metaclass.

Because it is a normal `ClassId`, cycles in the class/metaclass graph require no special ownership mechanism.

---

## 56. Superclass link

`ClassObject.superclass` is either:

```text
Some(ClassId)
```

or `None` at the root of the relevant hierarchy.

Method lookup may traverse this link, but the traversal algorithm belongs to `dispatch.md`.

---

## 57. Instance layout metadata

`field_slots` maps field symbols to offsets.

`field_count` defines the physical instance slot count.

These are layout metadata for `InstanceObject`.

The VM/compiler class-layout infrastructure MUST agree with the finalized class payload.

---

## 58. Class-side stored fields

`static_slots` is a fixed-size `Box<[Value]>` owned by the class object.

Class-side stored values therefore participate in object tracing as child values of the class object.

---

## 59. Native-representation and abstract flags

`native_repr` and `is_abstract` constrain allocation behavior.

A class may be a valid behavioral class without being representable through the generic `InstanceObject::new` allocator.

This is another reason runtime class identity must not imply one universal physical instance layout.

---

# Part XIII — Normalized multi-form values

## 60. Integer normalization

Phalcom integers have two physical forms.

### 60.1 Small form

If mathematically representable as `i64`:

```text
ValueTag::Int
```

is canonical.

### 60.2 Large form

Otherwise:

```text
Value::Obj
    ↓
Object::LargeInt(BigInt)
```

is used.

`normalize_bigint` enforces this split.

An arbitrary-precision result that falls back into the `i64` range MUST normalize to immediate `Int`.

---

## 61. Unit versus Tuple

The zero-arity tuple is normalized to immediate `Unit`.

Positive-arity tuple values use the Tuple product representation.

Thus:

```text
()            → Value::Unit
(a, ...)      → Object::Tuple
```

The exact tuple shape/descriptor model is described below.

---

## 62. Enum values

Enum values have two major physical forms.

### 62.1 Nullary variant

```text
Value::AdtSingleton(RuntimeVariantId)
```

No heap allocation is required for the value itself.

### 62.2 Payload-bearing variant

```text
Value::Obj
    ↓
Object::AdtCase {
    variant: RuntimeVariantId,
    storage: ProductStorage,
}
```

Both forms resolve behavior through the runtime variant registry.

---

## 63. Data values

Data values likewise have two forms.

### 63.1 Nullary data value

```text
Value::DataSingleton(RuntimeDataDescriptorId)
```

### 63.2 Positive-arity data value

```text
Value::Obj
    ↓
Object::Data {
    descriptor: RuntimeDataDescriptorId,
    storage: ProductStorage,
}
```

Both forms resolve behavior through the runtime data registry.

---

## 64. Semantic, runtime, and physical identities are separate

The implementation MUST distinguish at least:

```text
static semantic declaration/variant identity
runtime descriptor identity
physical representation identity
behavior ClassId
```

These identities may be related but are not interchangeable.

An optimization that changes the physical representation MUST NOT silently collapse semantic or runtime descriptor identities.

---

# Part XIV — Product storage

## 65. Shared packed representation

Tuple, Record, positive-arity Data, and payload-bearing ADT cases use the product representation infrastructure.

The core packed payload is:

```rust
ProductStorage {
    layout: ProductLayoutId,
    words: Box<[u64]>,
}
```

The layout authority is the heap's `ProductLayoutRegistry`.

---

## 66. Logical components versus physical words

A product has logical components.

Each logical component is assigned:

```text
logical_index
word_offset
ProductSlotRepr
```

The current physical slot representations are:

```text
Int64
Float64
Bool
Symbol
Value
```

---

## 67. Word widths

The representations occupy:

| `ProductSlotRepr` | 64-bit words |
|---|---:|
| `Int64` | 1 |
| `Float64` | 1 |
| `Bool` | 1 |
| `Symbol` | 1 |
| `Value` | 2 |

`Value` slots store the exact two representation words.

Scalar-specialized slots avoid storing the tag/metadata word when the layout already proves the component representation.

---

## 68. Layout validity

A valid `ProductLayout` requires:

- dense logical indexes `0..N`;
- non-overlapping word ranges;
- offsets that do not overflow;
- a total word length sufficient for every component.

The layout records the offsets of all `Value` slots separately for precise tracing.

---

## 69. Loading restores uniform values

Regardless of physical packed form, loading a product component reconstructs a uniform `Value`.

For example:

```text
Int64   → Value::int(...)
Float64 → Value::float(...)
Bool    → Value::bool(...)
Symbol  → Value::symbol(...)
Value   → Value::from_raw_words(payload, meta)
```

Code outside the packed-storage layer should reason in terms of logical `Value`s rather than interpreting raw product words ad hoc.

---

## 70. GC and packed products

Only `ProductSlotRepr::Value` can contain a managed `ObjRef` under the current layout model.

The layout therefore exposes `value_slot_offsets()`.

Precise object tracing reconstructs the `Value` at each such offset and routes it through `Value::gc_obj_ref`.

Scalar-specialized product slots are not heap edges.

If a future `ProductSlotRepr` can contain a managed reference, the layout and tracer MUST be amended together.

---

## 71. Tuple representation

A positive-arity tuple is:

```rust
TupleObject {
    descriptor: RuntimeAnonymousProductDescriptorId,
    storage: ProductStorage,
}
```

The tuple is immutable at the representation boundary.

It exposes no mutation accessor for its product storage.

---

## 72. Record representation

A positive-arity record is:

```rust
RecordObject {
    descriptor: RuntimeAnonymousProductDescriptorId,
    storage: ProductStorage,
}
```

Records are immutable product values.

---

## 73. Data representation

A materialized positive-arity data value is:

```rust
DataObject {
    descriptor: RuntimeDataDescriptorId,
    storage: ProductStorage,
}
```

The descriptor is part of semantic runtime identity.

Two product payloads with equal bits but different data descriptors are different data values for exact semantic-sameness purposes.

---

## 74. ADT case representation

A payload-bearing enum case is:

```rust
AdtCaseObject {
    variant: RuntimeVariantId,
    storage: ProductStorage,
}
```

The runtime variant identity and payload storage are distinct.

---

# Part XV — Identity, equality, and sameness

## 75. There is no single "equality" relation inside the VM

The implementation contains multiple relations serving different purposes.

At minimum:

```text
ObjRef identity
Rust Value PartialEq
raw representation sameness
VM semantic sameness (language ===)
default value equality substrate
dispatchable language ==
collection key comparison rules
```

These MUST NOT be conflated.

---

## 76. Heap object identity

For live heap objects:

```text
ObjRef A == ObjRef B
```

means the two handles identify the same heap object.

This relation does not inspect object contents.

---

## 77. Rust `Value::PartialEq`

Rust `PartialEq<Value>` is a heap-free representation-level operation.

It first requires equal base tags and equal `Some` depth.

It then compares tag-specific immediate payloads.

For `Obj`, it compares encoded object handles rather than dereferencing heap content.

This relation exists for runtime infrastructure.

It is not the complete language `==`.

---

## 78. Float behavior in representation equality

For `Value::PartialEq`:

- `+0.0` and `-0.0` compare equal under ordinary `f64` equality;
- NaN compares unequal, including to itself.

The `Hash<Value>` implementation normalizes zero hashing and uses a canonical NaN hash path so values that compare equal have coherent hashes.

This infrastructure behavior MUST NOT be mistaken for the complete collection protocol.

---

## 79. Raw representation sameness

`Value::same_as` compares the two physical representation words exactly.

Consequences include:

- no numeric coercion;
- no string content comparison;
- no object dereference;
- no Option unwrapping;
- exact float bits participate;
- heap values are the same only when their encoded `ObjRef` bits are the same.

This is the lowest-level sameness relation.

---

## 80. Language-level exact sameness (`===`)

The current language-level exact-sameness operation is:

```text
VM::semantic_same(lhs, rhs)
```

The `Same` bytecode and the `Object` sameness primitive use this VM-level relation.

`semantic_same` begins with raw representation sameness and then provides representation-independent exact sameness for selected immutable semantic value forms.

---

## 81. Tuple exact sameness

Two distinct `TupleObject`s may be semantically same when:

- both are tuples;
- positional arity matches;
- label sequence/shape matches;
- corresponding logical component values are recursively `semantic_same`.

Tuple `===` therefore does not reduce to `ObjRef` identity.

---

## 82. Record exact sameness

Two distinct `RecordObject`s may be semantically same when:

- both are records;
- field counts match;
- they have the same labels as a logical labeled product;
- corresponding values are recursively `semantic_same`.

The comparison is expressed through decoded logical record views, not raw arena identity.

---

## 83. Data exact sameness

Data exact sameness is descriptor-sensitive and representation-independent.

Current cases include:

```text
DataSingleton(d) === DataSingleton(d)

DataObject(d, components A) === DataObject(d, components B)
    if every logical component is recursively semantic_same

DataSingleton(d) === a materialized nullary DataObject with descriptor d
```

Different runtime data descriptor identities are not semantically same even when physical payloads happen to match.

---

## 84. Other values under `semantic_same`

Unless explicitly special-cased, values retain raw representation-sameness behavior.

In particular, this document does not infer structural `===` for an object category merely because it is immutable.

Any new representation-independent `===` category MUST be added deliberately to `semantic_same` and to this specification.

---

## 85. Default value equality substrate

`Value::value_eq` implements the heap-aware default value-equality substrate used by core equality behavior.

It includes representation-sensitive and content-sensitive cases such as:

- equal Option depth and recursively equal wrapped bases;
- ADT singleton runtime-id equality;
- data descriptor/component equality;
- Unit and Bool value equality;
- numeric cross-representation equality;
- symbol interned-id equality;
- String content equality;
- LargeInt mathematical equality;
- identity comparison for many ordinary heap objects.

It is not itself a statement that `==` can never be overridden.

---

## 86. Numeric equality crosses physical representations

Default numeric equality may compare mathematically across:

```text
Int
Float
LargeInt
```

when the floating value is finite/integral as required by the implementation.

Thus physical representation does not define mathematical equality.

Hashing/collection semantics must preserve the corresponding equality contract.

---

## 87. Module default equality caveat

The current default `Value::value_eq` deliberately does not treat module handles as equal merely because the `ObjRef`s match; module equality retains its historical behavior.

This is an example of why `ObjRef` identity and language `==` are separate relations.

---

## 88. Dispatchable equality

Language `==` remains part of runtime behavior/dispatch.

The object primitive may layer specialized semantics before or around `value_eq`, and classes may provide behavior according to the language's method model.

This document specifies the representation substrate, not the entire operator-dispatch protocol.

---

# Part XVI — Mutability and semantic categories

## 89. Representation categories

For runtime reasoning, object/value forms should be classified by both storage and semantic mutability.

A useful implementation taxonomy is:

### Immediate/value-semantic forms

```text
Unit
Bool
Int / LargeInt mathematical integer value
Float
Symbol
Option
ADT singleton
Data singleton
```

### Immutable heap value forms

```text
String
Tuple
Record
Data
AdtCase payload
selected descriptor values
```

### Mutable identity-bearing objects

```text
ordinary Instance
List
Map
Set
Bytes
Fiber
Module
Class
```

### Callable/descriptor/capability objects

```text
Method
Closure
BoundMethod
Family
Selector
SelectorPattern
MethodFamily
BoundMethodFamily
AssociatedFamily
reflection/typing descriptors
```

### VM-private objects

```text
Upvalue
PackBuilder
RecordLiteralBuilder
```

This classification is implementation guidance. Exact equality/hash behavior remains type-specific.

---

## 90. Mutability is a representation property

Where immutability is enforced by the storage API, it is stronger than a missing surface mutator.

For example, Tuple and Record expose immutable product storage.

By contrast, List, Map, Set, Bytes, ordinary Instance fields, Fiber state, Module globals, and Class runtime metadata are mutable runtime structures.

Optimizations relying on value stability MUST respect this distinction.

---

# Part XVII — GC visibility

## 91. `Value::gc_obj_ref` is the tracing seam

GC extraction from a `Value` MUST go through the representation-aware heap-edge seam.

Under the current representation:

```text
base tag Obj → return underlying ObjRef
otherwise    → no heap edge
```

Crucially, this test observes the base tag even when `some_depth > 0`.

Thus `Some(heapObject)` remains a GC edge.

---

## 92. Tracing MUST NOT duplicate Value representation knowledge

The object tracer should not manually reproduce the `ValueTag` switch.

It should call `gc_obj_ref`.

This keeps precise tracing coupled to one representation seam and allows the `Value` encoding to evolve without duplicating tag logic throughout the collector.

---

## 93. Frame and stack values

`execution-model.md` specifies where active and parked values live.

Any such `Value` is traced through `gc_obj_ref`.

The current running fiber's live stack is rooted directly by `VM`.

Parked fiber stacks are traced through `Object::Fiber`.

---

## 94. Object-edge tracing is exhaustive by variant

`trace_object` performs an exhaustive match over `Object` without a wildcard arm.

A new `Object` variant MUST therefore be explicitly classified by the tracer.

This is an architectural discipline:

> Adding an object representation and adding its managed-edge classification are one change.

---

## 95. Fields on existing object variants

Exhaustiveness over variants cannot detect a newly added reference-bearing field on an existing payload struct.

Therefore adding an `ObjRef`, `Value`, collection of handles/values, or equivalent managed edge to an existing object payload MUST include a tracing audit.

---

## 96. Ordinary instances

Tracing an `InstanceObject` marks:

- its class handle;
- every heap edge contained in its field `Value`s.

---

## 97. Classes

Tracing a `ClassObject` marks at least:

- metaclass;
- superclass;
- method handles;
- rest-method handles;
- heap references in static slots;
- heap references in attached attributes.

Symbol-keyed maps and Rust strings are not heap edges.

---

## 98. Closures and upvalues

A closure traces:

- its owning module;
- its upvalue cells;
- any heap edges in the callable constant pool.

An open upvalue traces its owning fiber.

A closed upvalue traces the closed `Value`.

Detailed capture lifetime is specified by the calls/closures VM specification.

---

## 99. Collections

The current tracing categories include:

- List: every element `Value`;
- Map/Set: keys and values through `Value` tracing;
- Bytes: no managed edges;
- Tuple/Record: only packed `Value` component slots;
- Range: optional endpoint values.

---

## 100. Fibers

A parked `FiberObject` traces its parked execution state and resident links, including:

- parked value stack;
- parked frames;
- open-upvalue cells;
- consumer;
- completion observer;
- result;
- entry callable;
- invariant-check receiver set;
- control stack roots.

The running fiber's corresponding execution buffers are empty and its live execution roots reside in the VM mirror.

---

## 101. Product-backed data and ADTs

`Object::Data` and `Object::AdtCase` trace product storage through the associated layout's `Value` slot offsets.

Packed `Int64`, `Float64`, `Bool`, and `Symbol` slots do not represent heap edges.

---

# Part XVIII — Surface receiver conversion

## 102. Call context

A surface `Value` can be converted to the execution receiver context required by a closure-backed method.

Heap object categories become one of:

```text
Instance context
Class context
Module context
```

Immediate values become:

```text
Immediate { value }
```

This permits user-defined/reopened bytecode behavior to execute against immediate values without manufacturing wrapper objects.

---

## 103. Immediate receivers

An immediate receiver preserves the actual `Value` in the frame's `CallContext`.

The VM MUST NOT assume that `self` always has an `ObjRef`.

This rule is specified in more detail by `execution-model.md`.

---

## 104. Internal non-receivers

VM-private heap objects such as upvalues and literal/argument builders are invalid surface receivers.

Attempting to convert one into an ordinary call context is an implementation invariant failure.

---

# Part XIX — Strings, collections, and other native representations

## 105. Dedicated native object forms

Not every class instance uses `InstanceObject`.

The heap contains dedicated native representations for runtime types where a specialized payload is required or materially more efficient.

Examples include:

```text
String
List
Map
Set
Bytes
Tuple
Record
Range
Fiber
LargeInt
```

Such values still map onto ordinary runtime classes.

---

## 106. Native representation classes

A class may be marked `native_repr`.

Such a class is behavioral metadata for values whose storage is not a generic `InstanceObject`.

Generic instance allocation MUST refuse classes whose runtime representation requires a dedicated heap/immediate form.

---

## 107. Representation-specific primitive floor

Native representations may expose low-level primitives implemented directly against their payload structs.

That fact does not alter the universal `Value`/`ObjRef` surface through which those values participate in ordinary calls and stack storage.

---

# Part XX — Descriptor and capability objects

## 108. Runtime descriptors are heap values

Selectors, methods, callable families, reflection descriptors, typing descriptors, module/project identities, and related runtime metadata may be reified as normal heap objects.

They therefore have:

- an `ObjRef` identity;
- a surface runtime class where applicable;
- ordinary GC edges;
- ordinary `Value::Obj` transport.

The entity they describe is not necessarily itself represented by the descriptor's `ObjRef`.

---

## 109. Bound method

A `BoundMethodObject` contains:

```text
method: ObjRef
receiver: Value
```

Its object identity is the bound-method object's own `ObjRef`.

The referenced method and receiver are child edges.

The object introduces no lexical block home-frame identity merely by being bound.

---

## 110. Family values

A `FamilyObject` contains:

```text
receiver: Value
spec: Exact(Symbol) | Pattern(ObjRef)
conditional entries
```

It is represented as an ordinary heap object even though it is callable capability metadata.

No dedicated `ValueTag::Family` exists.

---

## 111. Associated family

An `AssociatedFamilyObject` is also a heap object and surfaces through the Family behavior class.

The optional bound owner is a `Value` and therefore participates in GC through the normal value edge seam.

---

# Part XXI — Transitional representations

## 112. `Object::Block`

The current heap still contains a `BlockObject` wrapper used for home-frame/non-local-return bookkeeping.

It is explicitly transitional.

At the surface representation boundary, both:

```text
Object::Closure
Object::Block
```

currently map to the `Closure` runtime class.

The existence of the wrapper MUST NOT be interpreted as requiring a permanent separate source-level/runtime class distinction.

When the wrapper is removed, this document must be amended while preserving the required closure/non-local-return semantics.

---

# Part XXII — Performance-sensitive representation notes

## 113. Sixteen-byte value size

A fixed-size 16-byte `Value` makes stack and field storage uniform.

The VM can copy values without dynamic dispatch, ownership cloning, or reference counting.

Any future representation change MUST audit:

- operand stack density;
- `CallFrame`/argument interfaces;
- constant pools;
- instance slots;
- class static slots;
- collection payloads;
- product `Value` slots;
- native ABI by-value passing.

---

## 114. No NaN boxing in the current representation

The current VM uses an explicit tagged representation.

NaN boxing is not the current architecture.

Future adoption of another physical `Value` encoding may preserve the logical contracts in this document, but the size/layout sections would require amendment.

---

## 115. Packed product scalars

Packing proven Int/Float/Bool/Symbol product components into one word reduces storage relative to a full two-word `Value`.

The optimization is layout-driven.

Code outside `ProductStorage` MUST NOT assume that every logical product component occupies two raw words.

---

## 116. Large object payload boxing

Boxing large `Object` variants prevents one large payload from increasing every arena slot.

The optimization is behind the stable `ObjRef → Object` boundary.

---

## 117. Immediate singleton optimization

Nullary ADT and data values avoid heap allocation.

The corresponding runtime descriptor IDs preserve their semantic runtime identity.

Behavior and exact semantic value relations MUST remain valid if a future implementation materializes equivalent nullary objects for some internal path.

---

# Part XXIII — Representation invariants

## 118. Value-size invariant

`Value` is exactly 16 bytes in the current VM representation.

---

## 119. Reserved-bit invariant

Bits 40..=63 of `Value::meta` are zero in every currently valid representation.

---

## 120. Copy invariant

Copying a `Value` copies representation bits only.

It does not clone, retain, or otherwise duplicate the heap object named by an `ObjRef`.

---

## 121. Heap-handle invariant

A heap reference is a generational `ObjRef`, not a raw pointer.

---

## 122. Stale-handle invariant

A stale `ObjRef` MUST NOT resolve to an unrelated replacement object.

---

## 123. Object-identity invariant

The identity of a heap object is its live `ObjRef`.

Internal Rust payload pointers are not object identity.

---

## 124. Surface-Nil invariant

Private `Nil` MUST NOT escape through a guest-visible value boundary.

---

## 125. Option-depth invariant

`Some` nesting is represented by the `some_depth` metadata field without changing the base payload/tag.

---

## 126. Option-GC invariant

Option wrapping MUST NOT hide an underlying `ObjRef` from the garbage collector.

---

## 127. Option-Nil invariant

The private `Nil` sentinel cannot be wrapped as `Some`.

---

## 128. Class-mapping invariant

Every surface runtime value has one defined runtime behavior `ClassId`.

---

## 129. Representation-independence invariant

A runtime behavior class may have more than one physical value representation.

Dispatch MUST NOT assume one storage form per class.

---

## 130. Integer-normalization invariant

Every integer representable as `i64` uses the immediate `Int` form after normalization.

---

## 131. Unit-normalization invariant

The empty tuple uses immediate Unit rather than a heap `TupleObject`.

---

## 132. ADT singleton invariant

A nullary enum value may use `AdtSingleton`; a payload-bearing enum value uses an `AdtCase` product object.

Both preserve runtime variant identity.

---

## 133. Data singleton invariant

A nullary data value may use `DataSingleton`; a positive-arity data value uses a `DataObject`.

Both preserve runtime data descriptor identity.

---

## 134. Internal-object invariant

An object kind without a defined surface-class path MUST NOT escape as a guest-visible object.

---

## 135. Instance-layout invariant

Ordinary instances store a fixed slot array whose shape is established by their class layout.

---

## 136. Class-ID invariant

`ClassId` uses the same generational heap-handle mechanism as other object references.

---

## 137. Equality-layer invariant

The implementation MUST distinguish:

```text
ObjRef identity
Value representation equality
raw representation sameness
semantic language sameness
default value equality
dispatchable language equality
```

Using one relation where another is required is a VM correctness defect.

---

## 138. Product-layout invariant

Packed product storage is interpreted only through its associated `ProductLayout`.

Logical components MUST NOT be decoded from raw words without the layout.

---

## 139. Product-GC invariant

Every product slot representation capable of containing a managed reference MUST be included in precise tracing metadata.

---

## 140. Boxing-identity invariant

Boxing a Rust payload inside an `Object` variant does not create or change Phalcom object identity.

---

# Part XXIV — Required subsystem boundaries

## 141. Bytecode

`bytecode.md` owns:

- value-producing/consuming instruction effects;
- constant loading;
- `WrapSome`;
- product/data/ADT construction opcodes;
- field/local instruction behavior.

This document owns the runtime representations those instructions manipulate.

---

## 142. Execution model

`execution-model.md` owns:

- where `Value`s live during execution;
- operand/frame windows;
- `CallContext`;
- fiber live/parked value storage;
- return/unwind lifetime.

This document owns what a `Value` or object reference physically means.

---

## 143. Calls, closures, and frames

A later calls/closures specification should own:

- capture descriptors;
- block/closure construction;
- open versus closed upvalue lifecycle in full;
- callable parameter binding;
- non-local-return lexical semantics.

This document owns the `Upvalue` object category and its representation boundary.

---

## 144. Dispatch and classes

`dispatch.md` / class runtime specifications should own:

- selector lookup;
- superclass traversal;
- rest/family selection;
- cache invalidation;
- method visibility;
- metaclass tower semantics in full.

This document owns:

- class objects as heap payloads;
- class IDs;
- instance layout storage;
- the representation-to-class mapping.

---

## 145. Memory management

The memory-management specification owns:

- root discovery in full;
- mark/sweep algorithm;
- collection thresholds;
- reclamation;
- heap-growth policy;
- weak semantics.

This document owns:

- `ObjRef`;
- object-child representation;
- `Value::gc_obj_ref`;
- product slot traceability;
- the rule that every new representation must expose its managed edges.

---

## 146. Data/product representation

More specialized documents may specify:

- semantic product shape identity;
- descriptor registries;
- product layout selection;
- static lowering;
- optimized construction;
- component reflection.

This document fixes the common VM representation substrate.

---

# Part XXV — Implementation source map

## 147. Primary implementation anchors

| Concern | Primary source |
|---|---|
| uniform `Value` API | `phalcom-core/src/value/mod.rs` |
| exact two-word representation | `phalcom-core/src/value/repr.rs` |
| Option depth encoding | `phalcom-core/src/value/option.rs` |
| private Nil | `phalcom-core/src/value/nil.rs` |
| value rendering | `phalcom-core/src/value/render.rs` |
| central heap / `ObjRef` | `phalcom-core/src/heap/mod.rs` |
| exhaustive object taxonomy | `phalcom-core/src/heap/object.rs` |
| ordinary instance payload | `phalcom-core/src/heap/instance.rs` |
| class payload | `phalcom-core/src/heap/class.rs` |
| tuple payload | `phalcom-core/src/heap/tuple.rs` |
| record payload | `phalcom-core/src/heap/record.rs` |
| data payload | `phalcom-core/src/heap/data.rs` |
| ADT case payload | `phalcom-core/src/heap/adt.rs` |
| product layout | `phalcom-core/src/product/layout.rs` |
| product storage | `phalcom-core/src/product/storage.rs` |
| precise object tracing | `phalcom-core/src/heap/trace.rs` |
| semantic `===` | `phalcom-core/src/vm/mod.rs::semantic_same` |
| default object equality | `phalcom-core/src/primitive/object.rs` |
| execution roots | `phalcom-core/src/vm/gc.rs` |

A representation-changing patch to these sources requires review of this specification.

---

# Appendix A — Physical `Value` diagram

```text
16-byte Value

word 0: payload
┌──────────────────────────────────────────────────────────────┐
│                         u64 payload                          │
└──────────────────────────────────────────────────────────────┘

word 1: metadata
63                    40 39                          8 7       0
┌──────────────────────┬─────────────────────────────┬─────────┐
│ reserved = 0         │ Some nesting depth (u32)    │ tag u8  │
└──────────────────────┴─────────────────────────────┴─────────┘
```

---

# Appendix B — Representation decision tree

```text
runtime Value
    │
    ├── some_depth > 0
    │       └── surface class Some
    │           base tag/payload retained underneath
    │
    ├── tag None
    │       └── surface class None
    │
    ├── tag Unit
    │       └── Unit
    │
    ├── tag Bool
    │       ├── true  → True
    │       └── false → False
    │
    ├── tag Int
    │       └── Int
    │
    ├── tag Float
    │       └── Float
    │
    ├── tag Symbol
    │       └── Symbol
    │
    ├── tag AdtSingleton
    │       └── variant registry → behavior class
    │
    ├── tag DataSingleton
    │       └── data registry → behavior class
    │
    └── tag Obj
            │
            ▼
          ObjRef
            │
            ▼
           Heap
            │
            ▼
          Object
            │
            └── variant-specific class mapping
```

---

# Appendix C — Immediate Option examples

```text
Value::int(7)

payload = 7
tag = Int
depth = 0


Some(7)

payload = 7
tag = Int
depth = 1


Some(Some(7))

payload = 7
tag = Int
depth = 2


None

payload = 0
tag = None
depth = 0


Some(None)

payload = 0
tag = None
depth = 1
```

No Option wrapper object is allocated for these forms.

---

# Appendix D — Handle arena

```text
Value::Obj
    │
    │ payload encodes
    ▼
 ObjRef(slot, generation)
    │
    │ checked through arena
    ▼
┌─────────────────────────────┐
│ Heap                        │
│ SlotMap<ObjRef, Object>     │
│                             │
│ slot A → Object::Instance   │
│ slot B → Object::Class      │
│ slot C → Object::Tuple      │
│ ...                         │
└─────────────────────────────┘
```

A recycled physical slot with a different generation is not the same `ObjRef`.

---

# Appendix E — Ordinary object layout

```text
Value::Obj(instance_ref)
          │
          ▼
      Object::Instance
          │
          ▼
┌───────────────────────────────┐
│ InstanceObject                │
│                               │
│ class ────────────────────────┼────► ClassObject
│ slots[0] : Value              │       │
│ slots[1] : Value              │       ├─ field_slots
│ ...                           │       ├─ field_count
└───────────────────────────────┘       ├─ superclass
                                        └─ class ─► metaclass
```

---

# Appendix F — Multi-form semantic values

```text
INTEGER
small                              large
Value::Int                         Value::Obj
                                      │
                                      ▼
                                  LargeInt
        └──────────── both map to Int ────────────┘


ENUM VARIANT
nullary                            payload-bearing
AdtSingleton(rid)                  Value::Obj
                                      │
                                      ▼
                                  AdtCase(rid, storage)
        └──── both map through runtime variant rid ────┘


DATA
nullary                            positive-arity
DataSingleton(did)                 Value::Obj
                                      │
                                      ▼
                                  Data(did, storage)
        └──── both map through runtime data did ───────┘
```

---

# Appendix G — Product storage

```text
logical components
    │
    ▼
ProductLayout
    │
    ├── component 0 → Int64   @ word 0
    ├── component 1 → Value   @ words 1..2
    ├── component 2 → Bool    @ word 3
    └── component 3 → Symbol  @ word 4
    │
    ▼
ProductStorage
┌────────┬────────┬────────┬────────┬────────┐
│ word 0 │ word 1 │ word 2 │ word 3 │ word 4 │
│ Int64  │ Value payload/meta │ Bool │ Symbol │
└────────┴────────┴────────┴────────┴────────┘

GC value-slot offsets = [1]
```

Only slots declared capable of carrying a full `Value` are scanned for object references.

---

# Appendix H — Equality/sameness layers

```text
ObjRef identity
    "same heap object?"

Value::PartialEq
    "same representation category/payload according to Rust infrastructure?"

Value::same_as / same_bits
    "same two 64-bit Value words?"

VM::semantic_same
    "language exact sameness (===), including selected
     representation-independent value forms?"

Value::value_eq
    "heap-aware default value equality substrate?"

language ==
    "dispatchable equality protocol using the appropriate runtime behavior"
```

No implementation code should use one layer merely because it is convenient when another layer is semantically required.

---

# Appendix I — Representation review checklist

A patch changing values or objects should answer all of the following.

1. Does it add or alter a `ValueTag`?
2. Does it consume any currently reserved metadata bits?
3. Does it change the 16-byte `Value` size or alignment?
4. Can the new representation contain an `ObjRef`?
5. If wrapped in `Some`, will that reference remain visible to GC?
6. Does it add an `Object` variant?
7. Does it add a managed-reference field to an existing object payload?
8. What runtime class does the new surface representation map to?
9. Is the object surface-visible or VM-private?
10. What is its object/value identity relation?
11. What does raw `Value::PartialEq` mean for it?
12. What does `===` mean for it?
13. What does default `==` mean for it?
14. What hash contract follows from that equality?
15. Is it mutable?
16. Is it legal as a Map/Set key under the collection protocol?
17. Does it have a normalized immediate form?
18. Does it require a heap form as well?
19. Can two physical forms represent the same semantic value?
20. If so, does `semantic_same` need representation-independent handling?
21. Does it require a new `ProductSlotRepr`?
22. If packed storage can hold references, is precise tracing updated?
23. Does boxing/unboxing alter only physical storage rather than object identity?
24. Can a private implementation value escape through `Value::class`, `to_context`, return, field access, or reflection?
25. Do bytecode, execution, GC, dispatch, and reflection specs need corresponding amendments?

A representation patch is incomplete until every applicable question has a deliberate answer.
