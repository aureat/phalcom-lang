# Phalcom `DispatchKey`-in-`Value` Performance Audit

## Executive conclusion

The optimization is architecturally sound, fits the direction already anticipated by the repository, and attacks a real measured residual cost in Phalcom's current dispatch path.

It should be implemented, but with a narrower and more precise purpose than “put the class ID in `Value`.”

The right abstraction is:

> **`DispatchKey` is a compact, stable, VM-local identity for one runtime behavior class, used only to guard dispatch caches.**

It is not a replacement for `ClassId`. It is not a method-table version. It is not an object shape. It is not a truncated `ObjRef`. It should not replace `InstanceObject::class`, reflection identity, superclass relationships, ADT identities, or runtime type metadata.

The immediate optimization is straightforward:

```text
current warm Invoke
─────────────────────────────────────────
receiver Value
    ↓
Value::class(vm)
    ↓
possibly heap SlotMap lookup
    ↓
possibly Object enum discrimination
    ↓
possibly ADT/Data registry lookup
    ↓
ClassId
    ↓
compare InlineCache.class
    ↓
cached method


proposed warm Invoke
─────────────────────────────────────────
receiver Value
    ↓
extract DispatchKey from Value metadata
    ↓
compare InlineCache.dispatch_key
    ↓
cached method
```

Phalcom's monomorphic inline cache already eliminated the expensive hierarchy lookup from the common cache-hit path. The remaining problem is that the VM still computes `receiver.class(self)` before it can determine whether that cache hits. The current implementation explicitly does this before probing the call-site cache. 

That means `DispatchKey` is not speculative architecture looking for a use. It removes work that is demonstrably still on every ordinary warm message send.

There is also direct historical performance evidence. After U-IC landed, method lookup stopped being a major dispatch cost, but `Value::class` still accounted for roughly 4% of the sampled send-heavy profile; `call_method` was around 14%.  

So I would characterize the optimization as:

| Question | Assessment |
|---|---|
| Architecturally correct? | **Yes** |
| Compatible with current `Value` size? | **Yes** |
| Already anticipated by repository design? | **Yes** |
| Eliminates a real hot-path dependency? | **Yes** |
| Likely 20–30% whole-program improvement? | **No** |
| Plausible send-heavy improvement? | **Roughly low-single-digit whole-program**, potentially larger locally |
| Enables later dispatch optimizations? | **Strongly yes** |
| Should replace `ClassId`? | **No** |
| Should be bundled with PICs/per-class epochs/etc.? | **No** |

The strongest end-state is a 24-bit key occupying `meta` bits `40..=63`, with the lower metadata region continuing to encode Option/native unary-wrapper state. The key stored there should describe the **unwrapped/base value's runtime behavior**, while an outer `Some`, `Ok`, `Error`, or other native wrapper overrides the effective dispatch key without overwriting that base key.

That last point is critical.

---

# 1. Current repository architecture

## 1.1 `Value` already has exactly the right physical location

`Value` is currently exactly 16 bytes:

```rust
#[repr(C)]
pub struct Value {
    payload: u64,
    meta: u64,
}
```

Its current metadata allocation is:

```text
meta bits  0..=7   tag
meta bits  8..=39  Some nesting depth
meta bits 40..=63  reserved
```

The implementation comments and the authoritative VM specification both state that the upper 24 bits are currently reserved.  

This is unusually favorable for this optimization.

There is no need to grow `Value` from 16 bytes. Doing that would be unacceptable: `Value` is copied through operand stacks, fields, argument buffers, fibers, globals, tuple/data storage, constants, and native-call boundaries.

A previous runtime audit measured:

```text
Value               16 B
Bytecode              8 B
CallFrame            120 B
Cell<Option<IC>>      24 B
Cell<Option<GCache>>  24 B
```



So the metadata budget is precisely where this information belongs.

There is an even stronger repository signal: the current compact-native-`Result` implementation plan explicitly preserves bits `40..=63` while repurposing the current 32-bit `Some` region into native unary-wrapper cells, specifically leaving the 24-bit upper region available for future metadata. 

Another implementation plan explicitly lists a future "`Value` metadata allocation for `DispatchKey` or other future metadata" as deferred work. 

So this is not fighting the direction of the representation work. It is the obvious intended tenant of the remaining metadata budget.

---

# 2. What `Value::class` costs today

The current conceptual relationship is:

```text
Value
  │
  ├── immediate
  │     └── infer ClassId from tag/payload/core classes
  │
  ├── AdtSingleton
  │     └── runtime ADT descriptor
  │           └── behavior_class
  │
  ├── DataSingleton
  │     └── runtime data descriptor
  │           └── behavior_class
  │
  └── Obj
        └── Heap::get(ObjRef)
              └── Object variant
                    ├── Instance.class
                    ├── Class.class
                    ├── fixed native class
                    ├── ADT descriptor lookup
                    ├── Data descriptor lookup
                    └── ...
```

For ordinary heap objects, `Value::class()` first resolves the `ObjRef` through the heap and then switches over the large `Object` enum. For an `Instance`, it returns `instance.class`; for a `Class`, it returns the metaclass link; other native heap variants map to core classes; ADT and data objects may additionally require registry lookup. 

This is not intrinsically bad for reflection:

```phalcom
value.class
```

is allowed to be a comparatively rich semantic operation.

The problem is that ordinary cached dispatch currently pays this machinery merely to answer:

> “Is this receiver the same runtime behavior class that hit this call site last time?”

Those are different requirements.

The cache guard needs a compact equality token.

Reflection needs the actual `ClassId`.

Today both operations use `ClassId`, forcing the cache guard through the more expensive semantic path.

---

# 3. The existing inline cache makes `DispatchKey` especially useful

Phalcom already has a real monomorphic inline cache.

Its current structure is conceptually:

```rust
pub struct InlineCache {
    pub class: ClassId,
    pub method: ObjRef,
    pub version: u64,
}
```

and it is stored per `Bytecode::Invoke` instruction in `Chunk`. 

The send path effectively begins:

```rust
let receiver = self.stack[receiver_idx];
let receiver_class = receiver.class(self);

let cached = chunk.caches[cache_ip]
    .get()
    .filter(|slot| {
        slot.class == receiver_class
            && slot.version == self.world_version
    })
    .map(|slot| slot.method);
```



That order matters enormously.

Even on a perfect monomorphic cache hit:

```text
cache hit rate = 100%
hierarchy lookup = 0
method hash lookup = 0
```

Phalcom still performs:

```text
receiver.class(self)
```

every time.

The `DispatchKey` optimization therefore removes the final runtime-class derivation dependency from the warm IC path.

The new cache becomes:

```rust
pub struct InlineCache {
    pub dispatch_key: DispatchKey,
    pub method: ObjRef,
    pub version: u64,
}
```

and the hit becomes:

```rust
let receiver = self.stack[receiver_idx];
let key = receiver.dispatch_key();

let cached = chunk.caches[cache_ip]
    .get()
    .filter(|slot| {
        slot.dispatch_key == key
            && slot.version == self.world_version
    })
    .map(|slot| slot.method);
```

There should be no heap access in that guard.

No registry lookup.

No `Object` match.

No class hierarchy access.

No selector hash.

Just metadata extraction and integer comparison.

---

# 4. Why this matters at the CPU level

The optimization is primarily about removing a **dependent-load chain**.

Current heap-object fast path:

```text
load receiver Value
      │
      ▼
extract ObjRef
      │
      ▼
SlotMap lookup
      │
      ▼
load Object
      │
      ▼
inspect Object discriminant
      │
      ▼
load Instance.class / Class.class / descriptor
      │
      ▼
produce ClassId
      │
      ▼
load cache entry
      │
      ▼
compare
```

Proposed path:

```text
load receiver Value
      │
      ├─────────────┐
      ▼             ▼
extract key    load cache entry
      │             │
      └──────┬──────┘
             ▼
           compare
```

The second form exposes considerably more instruction-level parallelism. More importantly, it does not require the cache state of an unrelated heap slot merely to test the inline cache.

That distinction matters even where all data is in L1.

A chain of three dependent L1 loads is still latency that cannot be parallelized.

And when the referenced object or class metadata is not in L1, the difference gets substantially larger.

---

# 5. Existing performance data: expected magnitude

This optimization should not be oversold.

The repository already contains the relevant history.

After the monomorphic inline cache landed, the performance notes explicitly state that method lookup ceased to be the principal dispatch cost. 

A subsequent profile recorded approximately:

```text
global name resolution      ~13%
call_method mechanics       ~14%
Value::class                 ~4%
```



Another write-up summarizes the situation similarly: the post-U-IC `Value::class` contribution was roughly 4%, and the interpreter loop and method invocation machinery remained larger costs. 

Therefore:

> The historical upper bound for “delete all cost attributed to `Value::class`” on that particular send-heavy profile was about 4%.

A real implementation cannot delete 100% of that category because it replaces the operation with some instructions:

```rust
key = receiver.dispatch_key();
```

But those should be extremely cheap.

A realistic expectation before measurement would therefore be:

```text
send-heavy whole workload: perhaps ~1–4%
object-heavy monomorphic microbenchmark: potentially more
allocation-heavy workload: potentially near zero
hash-heavy workload: potentially near zero
fiber-heavy workload: likely modest
```

Those are expectations, not performance claims.

The optimization should ship only after current-HEAD benchmarking.

---

# 6. The semantic contract of `DispatchKey`

This is the most important design decision.

## 6.1 Do not define it as a truncated `ClassId`

`ClassId` is currently simply an intent alias for `ObjRef`:

```rust
pub type ClassId = ObjRef;
```



`ObjRef` is a generational SlotMap identity. It is deliberately wider and stronger than a compact array index.

Therefore this would be wrong:

```rust
DispatchKey((class_id.as_u64() & 0x00ff_ffff) as u32)
```

It introduces collision possibilities and generational-reuse hazards.

An inline-cache false positive is not a benign hash collision.

It can invoke a method belonging to the wrong class.

That is a correctness violation.

---

## 6.2 Required invariant

The invariant should be:

```text
Within one VM:

runtime ClassId A == runtime ClassId B
    ⇒ DispatchKey(A) == DispatchKey(B)

runtime ClassId A != runtime ClassId B
    ⇒ DispatchKey(A) != DispatchKey(B)
```

Or more concisely:

> `DispatchKey` is an injective VM-local mapping from live runtime behavior-class identity to a compact integer.

That means these should share keys:

```text
42                          ┐
Object::LargeInt(...)       ├── Int ClassId ── DK_INT
another immediate integer   ┘
```

Likewise:

```text
enum singleton representation ┐
heap ADT case representation   ├── same variant behavior class ── same key
                               ┘
```

and:

```text
nullary data immediate ┐
heap data object        ├── same data behavior class ── same key
                       ┘
```

This follows Phalcom's existing VM specification, which requires dispatch and reflection to use runtime behavior identity independently of physical storage representation. 

---

# 7. `DispatchKey` must not be method-table state

Suppose:

```phalcom
class User {
    name { ... }
}
```

and later `User#name` is redefined.

The `DispatchKey` of existing `User` values must not change.

It still means:

```text
runtime class = User
```

Method installation is already guarded separately by `VM::world_version`.

The current VM uses one global version counter and bumps it on method installation/redefinition; cache validity is therefore:

```text
same receiver class
AND
same method world
```



The two concepts should remain orthogonal:

```text
DispatchKey
    identity of behavior class
    stable

world_version
    identity of method-resolution world
    mutable
```

So the proper IC predicate is:

```rust
cache.dispatch_key == receiver.dispatch_key()
    && cache.world_version == vm.world_version
```

This is one reason not to call it a “shape version.”

It is not one.

---

# 8. Recommended physical representation

The natural layout is:

```text
Value.meta

63                                       40 39                 8 7        0
┌──────────────────────────────────────────┬────────────────────┬──────────┐
│              DispatchKey                 │ wrapper/depth state│ ValueTag │
│                24 bits                   │      32 bits       │  8 bits  │
└──────────────────────────────────────────┴────────────────────┴──────────┘
```

For the current representation:

```text
bits  0..=7   ValueTag
bits  8..=39  Some depth
bits 40..=63  DispatchKey
```

For the planned compact unary-wrapper representation:

```text
bits  0..=7   ValueTag
bits  8..=39  16 × 2-bit wrapper cells
bits 40..=63  DispatchKey
```

This matches the direction already recorded by the native-`Result` plan. 

Suggested representation code:

```rust
#[repr(transparent)]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
)]
pub struct DispatchKey(u32);

impl DispatchKey {
    pub const INVALID: Self = Self(0);
    pub const MAX_RAW: u32 = 0x00ff_ffff;

    #[inline]
    pub const fn from_raw(raw: u32) -> Option<Self> {
        if raw == 0 || raw > Self::MAX_RAW {
            None
        } else {
            Some(Self(raw))
        }
    }

    #[inline]
    pub const fn raw(self) -> u32 {
        self.0
    }
}
```

And:

```rust
const DISPATCH_SHIFT: u32 = 40;
const DISPATCH_MASK: u64 = 0x00ff_ffff_0000_0000;
```

This gives at most:

```text
2^24 - 1
= 16,777,215
```

nonzero dispatch identities.

That should be treated as a runtime resource bound.

Do not silently wrap.

Do not recycle keys.

---

# 9. Why keys should never be reused

Imagine:

```text
DK 4711 -> OldClass
```

A call-site cache holds:

```text
(DK 4711, foo-method)
```

Suppose `OldClass` is collected.

If key `4711` is then assigned to `NewClass`, a sufficiently stale cache becomes capable of matching the wrong class unless every possible cache and value lifetime is perfectly synchronized with reuse.

There is no reason to accept that complexity.

Use monotonic allocation:

```rust
next_dispatch_key += 1;
```

and never reuse keys within one VM.

The 24-bit namespace is large enough that exhaustion can simply be an explicit runtime resource error or fatal internal limit.

Correctness is more important than squeezing reuse into a namespace that already contains sixteen million identities.

---

# 10. The key should identify a behavior class, not an object

Every `User` instance:

```phalcom
User.new(...)
User.new(...)
User.new(...)
```

gets the same key.

Every distinct subclass gets another key:

```text
User      DK 172
Admin     DK 173
Guest     DK 174
```

Even if today they inherit identical methods.

Do not attempt this optimization:

```text
User and Admin currently dispatch identically
→ give them same DispatchKey
```

That would turn `DispatchKey` into a mutable equivalence-class identity.

It would become invalid the moment either class is reopened.

The current cache is guarded by runtime class identity. Preserve that model.

---

# 11. The crucial wrapper problem

Current Option representation makes the design slightly subtler.

`Some(x)` does not allocate another ordinary object. It retains the underlying `Value` representation and records wrapping in metadata.

That means:

```text
Some(user)
```

still physically contains the user's payload.

But its runtime class is `Some`, not `User`.

Therefore there are two different concepts:

```text
base dispatch key
effective dispatch key
```

For:

```phalcom
Some(user)
```

they are:

```text
base key       = DK_USER
effective key  = DK_SOME
```

After one unwrap:

```text
base key       = DK_USER
effective key  = DK_USER
```

This strongly argues against rewriting the upper metadata field whenever a wrapper is pushed.

Instead:

> The upper 24 bits should retain the dispatch identity of the underlying/base value.

Then:

```rust
fn dispatch_key(self) -> DispatchKey {
    if self.has_some_wrapper() {
        DK_SOME
    } else {
        self.base_dispatch_key()
    }
}
```

When the wrapper is removed, nothing has to recover the original class from the heap.

It was never destroyed.

This becomes even more important with the planned generalized wrapper representation:

```text
Some(Ok(Error(x)))
```

The metadata can contain:

```text
wrapper cells: Some → Ok → Error
base key:      DK_X
```

and effective dispatch is determined solely by the outermost wrapper.

Conceptually:

```rust
fn dispatch_key(self) -> DispatchKey {
    match self.outer_wrapper() {
        Some(OptionSome) => DK_SOME,
        Some(ResultOk)   => DK_RESULT_OK,
        Some(ResultError)=> DK_RESULT_ERROR,
        None             => self.base_dispatch_key(),
    }
}
```

This is a very good interaction between the two representation optimizations.

---

# 12. Immediate values: two implementation strategies

There is a design choice around immediates such as:

```text
Nil
Unit
Bool
Int
Float
Symbol
None
```

## Option A — embed keys in all Values

For example:

```rust
Value::int(42)
```

would produce:

```text
payload = 42
tag = Int
dispatch key = DK_INT
```

This gives the ideal API:

```rust
receiver.dispatch_key()
```

with no VM parameter.

It is especially attractive because compile-time constants can use fixed well-known core keys.

For example:

```rust
pub const DK_INT: DispatchKey = DispatchKey(5);
pub const DK_FLOAT: DispatchKey = DispatchKey(6);
pub const DK_SYMBOL: DispatchKey = DispatchKey(7);
```

The Universe bootstrap would guarantee that the `Int`, `Float`, and `Symbol` class rows receive those corresponding behavior keys.

Then the actual hot path is little more than:

```rust
(meta >> 40) & 0x00ff_ffff
```

plus wrapper handling.

This is the strongest end-state.

---

## Option B — hybrid derived keys for immediates

Alternatively, only dynamically classed representations need actual key bits.

An `Int` can derive:

```rust
ValueTag::Int => DK_INT
```

while an object uses its metadata field.

This means:

```rust
fn dispatch_key(self) -> DispatchKey {
    match self.tag() {
        ValueTag::Int   => DK_INT,
        ValueTag::Float => DK_FLOAT,
        ValueTag::Obj   => self.base_dispatch_key(),
        ...
    }
}
```

This avoids redundant bits for immediates but does not materially save representation space because the 24 bits already exist.

It does, however, make constructors slightly less coupled to core key constants.

### Recommendation

I prefer **Option A for the final design**.

The invariant becomes substantially stronger:

> Every unwrapped dispatchable `Value` contains its canonical base dispatch identity directly.

That simplifies debugging, testing, raw product storage, and future cache consumers.

The few well-known key constants are a reasonable VM ABI detail.

---

# 13. `Bool` demonstrates why keys must follow runtime classes

Phalcom's object model distinguishes:

```text
true.class  == True
false.class == False
```

rather than using one concrete `Bool` runtime class.

Therefore:

```text
true  → DK_TRUE
false → DK_FALSE
```

They must not share `DK_BOOL` merely because the physical tag is `Bool`.

This is exactly why the key should be defined from **runtime behavior identity**, not from storage representation.

---

# 14. `ClassObject` needs its own behavior key

The cleanest place to record the mapping from `ClassId` to `DispatchKey` is on the class row itself.

For example:

```rust
pub struct ClassObject {
    pub name: String,

    /// Runtime metaclass of this class object.
    pub class: ClassId,

    /// Superclass for instances governed by this row.
    pub superclass: Option<ClassId>,

    /// Compact identity for receivers whose runtime class is THIS row.
    dispatch_key: DispatchKey,

    ...
}
```

The name should make the direction clear. Something like:

```rust
instance_dispatch_key
```

may actually be less ambiguous than simply `dispatch_key`.

Because consider:

```phalcom
User
```

the class object itself.

The row `User` has:

```text
User.instance_dispatch_key = DK_USER
```

which is the key of `User` **instances**.

But the runtime class of the object `User` is its metaclass:

```text
User.class = User class
```

Therefore the `Value` representing the class object `User` must carry:

```text
heap.class(User.class).dispatch_key
```

not:

```text
heap.class(User).dispatch_key
```

That distinction must be made explicit in both implementation and tests.

---

# 15. Why putting the key on `ClassObject` is cheap

`ClassObject` is already one of the fat heap variants and is boxed.

The repository previously discovered that an inline `ClassObject` made every `SlotMap<Object>` entry pay for its size; the fat variants were subsequently boxed. 

Therefore adding four bytes of key metadata to the boxed `ClassObject` does not impose the same per-object arena-density penalty that adding a common field to every `Object` would.

This is one reason I would strongly reject the alternative:

```rust
struct HeapObjectHeader {
    class: ClassId,
    dispatch_key: DispatchKey,
    ...
}
```

for every heap object.

That still requires dereferencing the heap on every cache hit, so it fails to accomplish the principal optimization.

And it risks increasing the common heap footprint.

The key belongs in `Value`, with the authoritative class→key assignment stored on the class row.

---

# 16. `InstanceObject::class` should remain

Current instances contain:

```rust
pub struct InstanceObject {
    pub class: ClassId,
    pub slots: Box<[Value]>,
}
```



Do not remove the `class` field merely because `Value` gains a `DispatchKey`.

The two pieces of information serve different roles.

```text
DispatchKey
    cheap cache guard
    compact
    no hierarchy information
    no reflection object
    no superclass link

ClassId
    actual runtime class object
    reflection
    superclass traversal
    field-layout ownership
    method ownership
    runtime metadata
```

The optimization intentionally duplicates a small amount of identity information to make the common path cheaper.

Trying to eliminate that duplication would destroy most of the value of the optimization.

---

# 17. Class immutability makes this optimization much safer

A critical prerequisite is that an ordinary object's runtime class must not mutate underneath copied `Value`s.

Phalcom already has this property at the language level.

The runtime implementation of setting an object's class rejects the operation with `InvalidSetClass`. 

Likewise guest-level superclass mutation is rejected. 

So for an ordinary instance:

```text
object created as User
→ remains User for its lifetime
→ DK_USER stored in every Value pointing at it remains valid
```

That is exactly the lifecycle required for cached representation metadata.

---

# 18. Bootstrap is the main class-key complication

Classes themselves are constructed in stages.

For example, class rows can initially be allocated bare and then have their superclass/metaclass links patched during bootstrap. The class-creation API likewise performs allocation followed by wiring.  

This means we need to separate:

```text
key of instances governed by a class row
```

from:

```text
runtime class of the ClassObject itself
```

The first can be allocated immediately with the row.

The second depends on the metaclass link being finalized.

A safe bootstrap invariant should therefore be:

> A class row may receive its instance `DispatchKey` immediately, but it must not be published as a fully formed guest `Value` until its `.class` link is finalized.

This is already conceptually close to the existing allocate-then-patch bootstrap model.

---

# 19. Where the key allocator should live

The most mechanically convenient authority is probably the heap/class-allocation layer because `Heap::alloc_class` is already the point that creates `ClassId`s:

```rust
pub fn alloc_class(&mut self, class: ClassObject) -> ClassId
```



A reasonable model is:

```rust
pub struct Heap {
    objects: SlotMap<ObjRef, Object>,
    next_dispatch_key: u32,
    ...
}
```

and:

```rust
pub fn alloc_class(&mut self, mut class: ClassObject) -> ClassId {
    let key = self.allocate_dispatch_key();
    class.install_dispatch_key(key);
    self.insert(Object::Class(Box::new(class)))
}
```

But fixed keys for core immediate classes require one additional path.

For example:

```rust
pub(crate) fn alloc_core_class_with_key(
    &mut self,
    class: ClassObject,
    key: DispatchKey,
) -> ClassId
```

with checks that:

```text
key != INVALID
key not already assigned
key inside reserved well-known range
```

Ordinary dynamic class allocation then starts after the last well-known key.

---

# 20. Alternative: VM-level allocator

Another design is:

```rust
VM {
    next_dispatch_key: u32,
    ...
}
```

This makes semantic ownership clearer, but core Universe construction currently allocates class rows directly through heap-oriented bootstrap code.

Moving the allocator into `VM` would therefore force more bootstrap restructuring.

Unless there is a larger reason to move class allocation authority out of `Heap`, I would not do that as part of this optimization.

Keep this patch narrow.

---

# 21. Object `Value` construction is the largest implementation surface

Today, code can simply do:

```rust
Value::obj(obj_ref)
```

That constructor cannot know the object's runtime class.

Once an object-valued `Value` must contain a canonical dispatch key, this API becomes insufficient.

This is the largest implementation cost of the optimization.

The wrong fix is:

```rust
pub fn obj(obj: ObjRef, dispatch_key: DispatchKey)
```

and then allowing arbitrary callers to provide arbitrary pairs.

That makes this invalid state trivial:

```rust
Value::obj(user_ref, DK_STRING)
```

The pairing needs an authority.

---

# 22. Recommended construction API

Separate low-level encoding from runtime-authoritative construction.

For example:

```rust
impl Value {
    pub(crate) const fn obj_with_dispatch_key(
        obj: ObjRef,
        key: DispatchKey,
    ) -> Self {
        ...
    }
}
```

Then provide VM-owned factories:

```rust
impl VM {
    pub fn value_for_object(&self, object: ObjRef) -> Value {
        let class = self.class_of_object(object);
        let key = self.heap.class(class).dispatch_key();

        Value::obj_with_dispatch_key(object, key)
    }

    pub fn value_for_object_with_class(
        &self,
        object: ObjRef,
        class: ClassId,
    ) -> Value {
        let key = self.heap.class(class).dispatch_key();

        Value::obj_with_dispatch_key(object, key)
    }
}
```

The generic version may perform the same object-discrimination work as today's `Value::class`, but crucially:

> It performs it once when the `Value` is created, rather than on every message send.

Hot allocation sites should use the second form whenever they already know the class.

For example:

```rust
let instance_ref = heap.alloc(
    Object::Instance(
        InstanceObject::new(class, field_count)
    )
);

let value = vm.value_for_object_with_class(
    instance_ref,
    class,
);
```

The class ID is already present.

There is no reason to rediscover it.

---

# 23. Construction APIs should become progressively stricter

My preferred migration is:

### Stage 1

Introduce:

```rust
Value::obj_with_dispatch_key(...)
VM::value_for_object(...)
```

and migrate call sites.

### Stage 2

Make unkeyed:

```rust
Value::obj(...)
```

`pub(crate)` or test-only.

### Stage 3

Eventually remove the unkeyed constructor entirely from production code.

This gives the compiler an important invariant:

> Every guest-visible object `Value` is born with its correct dispatch identity.

That is much better than supporting a permanent:

```text
dispatch_key == 0
```

fallback.

A permanent unkeyed fallback would infect the hot path:

```rust
if key == INVALID {
    receiver.class(vm)
} else {
    key
}
```

and turn representation bugs into silent performance regressions.

Use such a fallback only during migration/debugging, if at all.

---

# 24. ADT values need keyed runtime descriptors

The runtime ADT descriptor already contains:

```rust
pub struct RuntimeVariantDescriptor {
    ...
    pub behavior_class: ClassId,
    pub singleton: Option<Value>,
}
```



This is a natural place to also record:

```rust
pub behavior_dispatch_key: DispatchKey,
```

At registration:

```rust
let key = vm.heap
    .class(behavior_class)
    .dispatch_key();
```

Then both representations:

```text
Value::AdtSingleton
Object::AdtCase
```

use the exact same behavior key.

This is important because the runtime already constructs singleton variants directly as immediate `Value`s. 

Instead of:

```rust
Value::adt_singleton(runtime_variant)
```

the low-level constructor should eventually become something like:

```rust
Value::adt_singleton_with_dispatch(
    runtime_variant,
    descriptor.behavior_dispatch_key,
)
```

Ideally outside code does not construct ADT singleton Values directly at all; the registry/VM should return the canonical prebuilt singleton.

---

# 25. Data values need the same treatment

`RuntimeDataDescriptor` similarly contains:

```rust
pub behavior_class: ClassId,
```

alongside the exact type and product layout. 

Add:

```rust
pub behavior_dispatch_key: DispatchKey,
```

and use it for both:

```text
Value::DataSingleton
Object::Data
```

Current nullary construction produces `Value::data_singleton(descriptor_id)`. 

That should become registry-owned keyed construction.

This lets:

```text
nullary Foo
heap-backed Foo(...)
```

share exactly the same dispatch identity without a descriptor lookup on every send.

---

# 26. GC interaction is favorable

The key is plain integer metadata.

It does not introduce a GC edge.

Therefore:

```text
DispatchKey
    NOT traced
```

The underlying object payload continues to be traced exactly as before.

ADT and data behavior classes are already rooted through their registries. The data registry exposes its behavior classes to GC rooting, and the ADT registry similarly enumerates class roots.  

The VM's root collector already includes these registries. 

That means a Phase-1 `DispatchKey` implementation needs no new reverse map and no new GC root category.

This is desirable.

---

# 27. Avoid a reverse `DispatchKey → ClassId` table initially

It may be tempting to add:

```rust
Vec<ClassId>
```

indexed by `DispatchKey`.

Do not do that for the initial optimization.

The common cache-hit path does not need it.

On a miss, the VM can continue using the current:

```rust
receiver.class(self)
```

and ordinary lookup machinery.

Thus:

```text
hit:
    Value -> DispatchKey -> cached method

miss:
    Value -> current ClassId derivation -> hierarchy lookup
```

This makes the optimization very safe.

No new class-root ownership problem.

No reverse-map lifetime rules.

No stale ClassId slots.

No need to redesign reflection.

If profiling later shows that cache misses themselves deserve optimization, the reverse mapping can be considered separately.

---

# 28. `===` / representation identity is an important interaction

Current `Value::same_bits` compares:

```rust
self.payload == other.payload
    && self.meta == other.meta
```



And `Value::same_as`, which participates in representation-level identity semantics, bottoms out in that representation comparison. 

Therefore adding `DispatchKey` to `meta` makes it part of exact bit identity.

That is safe only if:

> The key is canonical and deterministic for every valid runtime representation.

That is another reason it must not be mutable cache state.

Given:

```rust
let a = some_object
let b = a
```

both must contain the identical key.

Given two independently materialized representations of the same immediate value:

```rust
Value::int(42)
Value::int(42)
```

both must also contain `DK_INT`.

If that invariant holds, keeping `same_bits` unchanged is the best outcome.

I would not mask `DispatchKey` out of `===`.

That would subtly redefine “representation identity.”

The current native-Result work also explicitly aims to preserve the existing `same_bits` semantics rather than redefining it. 

---

# 29. Ordinary equality and hashing should ignore the key

Semantic equality should not depend on optimization metadata.

The current `Hash` implementation explicitly hashes semantic representation components such as the tag and wrapper/depth state rather than blindly hashing the entire metadata word. 

That should remain true.

Conceptually:

```text
==
hash()
semantic_same()
    key is irrelevant

===
same_bits()
    canonical key is part of the exact representation
```

This division is coherent as long as key assignment is canonical.

---

# 30. Packed product storage is an important hidden interaction

Phalcom's product representation stores components in different physical forms.

For a generic `Value` lane it persists both words:

```text
payload
meta
```

and reconstructs the `Value` with `Value::from_raw_words`. 

That path will preserve `DispatchKey` automatically.

But scalar lanes such as:

```text
Int64
Float64
Bool
Symbol
```

store only their raw scalar word and reconstruct a `Value` using:

```rust
Value::int(...)
Value::float(...)
Value::bool(...)
Value::symbol(...)
```

Therefore those constructors must recreate the correct canonical key.

This is another reason fixed well-known core keys are an excellent design.

For example:

```rust
Value::int(raw)
```

must always produce:

```text
DK_INT
```

regardless of whether the integer originated from:

```text
source literal
VM primitive
unpacked data field
tuple field
record field
native return
```

No packed-product format needs to grow.

That is a major benefit.

---

# 31. `from_raw_words` must change its validation contract

Currently the raw-word constructor expects the upper reserved bits to be zero. 

Once `DispatchKey` occupies them, that invariant becomes obsolete.

Instead of:

```rust
debug_assert!(meta & RESERVED_MASK == 0);
```

the representation layer should validate the new known fields.

Conceptually:

```rust
debug_assert!(
    decode_dispatch_key(meta).is_valid(),
    "Value contains invalid dispatch key"
);
```

or, during transitional bootstrap:

```rust
debug_assert!(
    key == INVALID || key.is_valid()
);
```

There would effectively be no unallocated `meta` bits once all 24 upper bits are assigned.

The authoritative VM specification must change accordingly.

---

# 32. The proposed warm send path

The implementation should preserve the current miss semantics and modify only the guard.

A good target is essentially:

```rust
fn invoke_at(
    &mut self,
    callable: &Callable,
    cache_ip: usize,
    arity: u8,
    selector_idx: u16,
) -> PhResult<()> {
    let arity = arity as usize;
    let receiver_idx = self.stack.len() - 1 - arity;
    let receiver = self.stack[receiver_idx];

    // NEW: no heap/class lookup.
    let dispatch_key = receiver.dispatch_key();

    let (cached_method, source_range) = {
        let chunk = &callable.chunk;

        let cached = chunk.caches[cache_ip]
            .get()
            .filter(|slot| {
                slot.dispatch_key == dispatch_key
                    && slot.version == self.world_version
            })
            .map(|slot| slot.method);

        (cached, chunk.spans[cache_ip])
    };

    if let Some(method) = cached_method {
        return self.call_method(
            &receiver,
            method,
            arity,
            source_range,
        );
    }

    // Existing slow path remains authoritative.
    let selector = ...;

    if let Some(method) = receiver.lookup_method(self, selector) {
        callable.chunk.caches[cache_ip].set(Some(
            InlineCache {
                dispatch_key,
                method,
                version: self.world_version,
            },
        ));

        return self.call_method(
            &receiver,
            method,
            arity,
            source_range,
        );
    }

    // Existing variadic / DNU behavior...
}
```

The important property is not the exact Rust spelling.

It is this:

```text
IC hit does not need ClassId.
```

---

# 33. Cache size will probably not shrink automatically

Today the measured cache cell is 24 bytes. 

Replacing:

```text
ClassId / ObjRef  8 B
```

with:

```text
DispatchKey       4 B
```

does not necessarily make the Rust struct smaller because:

```text
DispatchKey  4
padding      4
ObjRef       8
u64 version  8
──────────────
total       24
```

So do not justify this optimization primarily as an IC-density optimization.

Its first-order benefit is execution latency.

A later cache-layout optimization could reorganize data, but that deserves separate measurement.

---

# 34. Do not `repr(packed)` the inline cache

An obvious reaction would be:

```rust
#[repr(packed)]
struct InlineCache { ... }
```

to recover the padding.

That is the wrong priority.

Unaligned loads can complicate generated code and Rust references to packed fields.

This is a hot structure.

Optimize it based on assembly and benchmarks, not byte-count aesthetics.

---

# 35. Interaction with `world_version`

The current global-version design remains fully valid.

Suppose:

```text
DK_USER → User
```

and the inline cache is:

```text
DK_USER
method = old User#foo
world = 418
```

A method is redefined anywhere:

```text
VM.world_version = 419
```

The next call sees:

```text
DK_USER == DK_USER
419 != 418
```

and misses.

No existing `Value` has to be touched.

No `DispatchKey` changes.

This is exactly the desired separation of concerns.

---

# 36. Do not bundle per-class epochs into this patch

Once keys exist, this may look attractive:

```rust
method_epoch[dispatch_key]
```

instead of one global `world_version`.

There is potential there, but inheritance makes it substantially harder than it first appears.

Suppose:

```text
Object
  ↓
User
  ↓
Admin
```

and `User#foo` changes.

The cached resolution for an `Admin` receiver may also have changed because `Admin` inherited `foo`.

Therefore a per-class epoch would require some combination of:

```text
descendant invalidation
dependency tracking
hierarchy-version aggregation
resolution-owner epochs
```

The current global world version is coarse but correct.

The `DispatchKey` optimization does not require changing it.

Do not turn a safe hot-path representation patch into a dispatch invalidation redesign.

---

# 37. Interaction with method lookup tables

Class methods currently live in per-class hash/index maps and the slow path walks the superclass chain.

`DispatchKey` does not change this.

That is appropriate.

The fast path:

```text
key compare
→ cached Method
```

avoids the maps.

The slow path:

```text
ClassId
→ hierarchy traversal
→ selector lookup
```

remains fully semantic.

Later, if profiling identifies misses as important, the runtime could consider:

```text
dense selector IDs
per-class dispatch vectors
selector-indexed tables
global (DispatchKey, SelectorId) caches
```

but none are necessary to realize the initial benefit.

---

# 38. Interaction with superinstructions

Phalcom already fuses:

```text
GetLocal + Invoke
Constant + Invoke
```

into:

```text
InvokeLocal
InvokeConst
```

while preserving the original `Invoke` location for its inline-cache slot. 

The two optimizations are complementary.

Superinstructions attack:

```text
interpreter opcode-dispatch overhead
```

`DispatchKey` attacks:

```text
work inside the Invoke operation
```

Conceptually:

```text
Before:
GetLocal dispatch
Invoke dispatch
Value::class
IC probe

After fusion:
InvokeLocal dispatch
Value::class
IC probe

After fusion + DispatchKey:
InvokeLocal dispatch
metadata key extract
IC probe
```

The previous fusion work produced meaningful improvements on several dispatch-heavy benchmarks—for example around −8.1% `string_equals`, −5.1% `for`, −4.7% `variadic_send`, and −4.2% `bare_send` in that measurement campaign. 

That does not make `DispatchKey` obsolete.

It changes the remaining cost distribution and therefore means `DispatchKey` must be remeasured against current HEAD rather than against an old pre-fusion baseline.

---

# 39. Interaction with polymorphic inline caches

This is where `DispatchKey` becomes more strategically valuable.

Current cache:

```text
one class
one method
```

A call site receiving:

```text
User
Admin
User
Admin
User
Admin
```

will continually evict itself.

A small PIC could instead contain:

```text
DK_USER  → User#foo
DK_ADMIN → Admin#foo
```

Then its probe can become a few integer comparisons.

For example:

```rust
struct PicEntry {
    dispatch_key: DispatchKey,
    method: ObjRef,
}
```

and:

```rust
struct Pic {
    world_version: u64,
    len: u8,
    entries: [PicEntry; 4],
}
```

The compact key is materially better for this design than a full generational `ClassId`.

However:

> I would not implement PICs in the same patch.

First instrument monomorphic cache misses.

Determine how many call sites are:

```text
monomorphic
bimorphic
small polymorphic
megamorphic
```

before paying the memory cost of a larger cache at every site.

A 4-entry PIC can easily become roughly one cache line or more per send site depending on layout.

That is a distinct trade-off.

---

# 40. Better PIC layout becomes possible

The compact key enables structure-of-arrays layouts such as:

```rust
struct Pic4 {
    version: u64,
    methods: [ObjRef; 4],
    keys: [u32; 4],
    len: u8,
}
```

rather than:

```rust
[(ClassId, ObjRef); 4]
```

This can reduce key bandwidth and permit vectorized or branch-light comparisons later.

Again: valuable future option, not Phase 1.

---

# 41. Megamorphic dispatch caches

For a genuinely megamorphic site, a global or per-VM cache could key resolution by:

```text
(DispatchKey, SelectorId)
```

If:

```text
DispatchKey = 24 bits
Symbol       ≈ compact interned identity
```

then that pair can be extremely compact.

Conceptually:

```rust
struct DispatchCacheKey {
    class: DispatchKey,
    selector: Symbol,
}
```

which is much friendlier than a pair involving a generational heap handle.

Such a cache could eliminate repeated hierarchy walks at megamorphic sites.

It is another strong reason to make `DispatchKey` a first-class runtime identity rather than merely hiding a compact integer inside the current monomorphic cache implementation.

---

# 42. Interaction with primitive-call optimization

The historical profile says `call_method` was substantially more expensive than `Value::class` after U-IC—roughly 14% versus 4% in the cited send-heavy profile. 

So after `DispatchKey`, one of the more interesting remaining paths is likely:

```text
IC says Method ObjRef
    ↓
heap lookup MethodObject
    ↓
determine primitive vs interpreted
    ↓
argument mechanics
    ↓
invoke
```

A future cache could potentially remember a more directly executable target:

```rust
enum CachedCallTarget {
    Primitive(PrimitiveFn),
    Method(ObjRef),
}
```

or another compact callable handle.

That can potentially remove a second heap indirection after the IC succeeds.

However it has stronger invalidation and representation implications than a guard key.

It should be profiled and designed separately.

The correct priority is:

```text
1. make receiver guard cheap
2. measure remaining send body
3. optimize call target only with evidence
```

---

# 43. Interaction with global-variable caching

The post-U-IC profile found variable access itself to be a larger cost than remaining class resolution in some send-heavy programs.

That is consistent with why the repository later added dedicated global caches.

This matters strategically:

`DispatchKey` is a good optimization, but it is not the singular bottleneck of the VM.

It should be treated as one component of:

```text
faster bytecode execution
+
faster receiver classification
+
faster invocation mechanics
+
better cache locality
```

rather than expected to close the entire interpreter gap alone.

---

# 44. Interaction with 8-byte `Value` / NaN boxing

A possible objection is:

> “Should we avoid consuming these bits because we may later shrink `Value` to 8 bytes?”

The current repository already concludes that this is blocked by another representation issue: a NaN payload does not comfortably contain the full current 64-bit generational `ObjRef`. 

So reserving the existing 24 metadata bits indefinitely for a hypothetical immediate 8-byte conversion is not especially compelling.

If Phalcom eventually compresses `ObjRef` or changes heap addressing enough to make an 8-byte `Value` possible, the entire value representation will already require redesign.

At that point the location of `DispatchKey` will naturally be revisited.

For the current 16-byte architecture, the upper 24 bits are valuable runtime real estate and `DispatchKey` is a strong use for them.

---

# 45. Interaction with native `Result` representation

This is one of the highest-priority coordination points.

The in-progress `Result` representation plan changes:

```text
32-bit Some depth
```

into:

```text
16 × 2-bit native wrapper cells
```

while explicitly retaining the upper 24 bits. 

The two designs fit naturally if the rule is:

> Wrapper operations never modify the upper `DispatchKey` region.

Then:

```text
Value:
    tag
    native wrapper stack
    base dispatch key
```

For:

```phalcom
Some(Ok(user))
```

the representation can conceptually be:

```text
base payload        = user ObjRef
base tag            = Obj
wrapper[0]          = Some
wrapper[1]          = ResultOk
base DispatchKey    = DK_USER
```

The effective key is:

```text
DK_SOME
```

Pop one wrapper:

```text
effective key = DK_RESULT_OK
```

Pop another:

```text
effective key = DK_USER
```

No heap lookup is required during any of those transitions.

That is a very good compositional property.

---

# 46. Wrapper overflow/spill needs one explicit rule

The planned generalized native-wrapper representation anticipates spilling overly deep wrapper chains.

If a wrapper spill is physically represented by an internal heap object, it must not accidentally expose the spill object's physical runtime class to dispatch.

The rule must remain:

```text
logical outer wrapper
    determines effective dispatch identity
```

not:

```text
physical storage object's Object variant
    determines dispatch
```

This is the same principle already present in the VM specification: runtime behavior identity is not necessarily the same as storage representation. 

This deserves a joint regression test when both features exist.

---

# 47. Associated caches need a careful audit but not an immediate redesign

The executable semantic pool contains another cache form:

```rust
pub struct AssociatedTargetCache {
    pub receiver: Value,
    pub method: ObjRef,
    pub world_version: u64,
}
```



Because it stores a complete receiver `Value`, introducing canonical key bits changes those stored bits as well.

If the `DispatchKey` invariant is canonical, this is harmless.

However, do not automatically replace its full receiver with a `DispatchKey`.

Associated lookup may intentionally distinguish exact associated receiver identity, not merely runtime class identity.

This cache needs semantic review before class-based compression.

---

# 48. Super dispatch is different

A `super` send is not simply:

```text
dispatch based on receiver class
```

Its lookup start point depends on lexical/declaring-class context.

Therefore the ordinary receiver `DispatchKey` is not, by itself, sufficient to characterize a super-send resolution.

A super-send cache could potentially be guarded by:

```text
static lookup-start ClassId
world_version
```

and perhaps other lexical identity.

Do not incorrectly reuse ordinary receiver-key semantics for it.

This is another reason `DispatchKey` should remain a narrow primitive rather than becoming “the answer to every dispatch.”

---

# 49. Bilateral/operator dispatch likewise needs real class relationships

Any resolution process that needs questions such as:

```text
Is class A a strict subtype of class B?
Which class defined this candidate?
Which superclass introduced this method?
```

still requires `ClassId` and hierarchy metadata.

A key can guard a cache containing the answer.

It cannot answer the hierarchy question itself.

That is the correct architectural separation:

```text
ClassId
    semantic runtime structure

DispatchKey
    cheap equality guard for cached conclusions
```

---

# 50. Recommended `Value` API

I would aim for something structurally like this:

```rust
impl Value {
    #[inline]
    pub const fn base_dispatch_key(self) -> DispatchKey {
        DispatchKey(
            ((self.meta & DISPATCH_MASK) >> DISPATCH_SHIFT)
                as u32
        )
    }

    #[inline]
    const fn with_base_dispatch_key(
        mut self,
        key: DispatchKey,
    ) -> Self {
        debug_assert!(key.raw() <= DispatchKey::MAX_RAW);

        self.meta =
            (self.meta & !DISPATCH_MASK)
            | ((key.raw() as u64) << DISPATCH_SHIFT);

        self
    }

    #[inline]
    pub fn dispatch_key(self) -> DispatchKey {
        if let Some(wrapper) = self.outer_wrapper_kind() {
            return wrapper.dispatch_key();
        }

        self.base_dispatch_key()
    }
}
```

After the generalized wrapper representation lands:

```rust
impl NativeUnaryWrapperKind {
    #[inline]
    fn dispatch_key(self) -> DispatchKey {
        match self {
            Self::OptionSome  => DK_OPTION_SOME,
            Self::ResultOk    => DK_RESULT_OK,
            Self::ResultError => DK_RESULT_ERROR,
        }
    }
}
```

This gives the hot path an excellent abstraction:

```rust
receiver.dispatch_key()
```

No VM parameter.

No heap parameter.

No registry parameter.

---

# 51. Proposed well-known key namespace

I would reserve a small initial range for behavior classes that must be constructible without a VM context.

For illustration:

```text
0      INVALID

1      Nil
2      Unit
3      False
4      True
5      Int
6      Float
7      Symbol
8      None
9      Some
10     Result::Ok
11     Result::Error
...

256+   dynamically allocated class identities
```

The exact numbering is not important.

The design properties are:

1. fixed keys are documented implementation ABI;
2. zero is invalid;
3. dynamic IDs begin after a reserved range;
4. IDs never get recycled;
5. IDs have no semantic ordering or hierarchy meaning.

Do not attempt clever bit partitioning such as:

```text
top bits = object family
bottom bits = subclass
```

There is no need.

Dense opaque integers are better.

---

# 52. Class registration invariant

Every class row should satisfy:

```rust
heap.class(class_id).dispatch_key() != DispatchKey::INVALID
```

after allocation.

And globally:

```text
for all live class rows A, B:

A != B
⇒ A.dispatch_key != B.dispatch_key
```

A debug verification pass after Universe bootstrap should enumerate known classes and assert uniqueness.

This is the kind of invariant worth failing loudly.

A duplicated dispatch key is a wrong-method-execution bug waiting to happen.

---

# 53. Debug cross-check mode

During rollout, I would add a debug-only consistency check.

Conceptually:

```rust
#[cfg(debug_assertions)]
fn assert_dispatch_key_matches_class(
    &self,
    value: Value,
) {
    let actual_class = value.class(self);
    let expected_key =
        self.heap.class(actual_class).dispatch_key();

    assert_eq!(
        value.dispatch_key(),
        expected_key,
        "Value carries stale or incorrect DispatchKey"
    );
}
```

Do not call this in production hot paths.

But use it heavily in tests and possibly behind an opt-in runtime verification feature.

This will expose missing construction sites rapidly.

---

# 54. File-by-file implementation map

| Area | File / subsystem | Change |
|---|---|---|
| Key type | `phalcom-core/src/value/` or dedicated runtime identity module | Add `DispatchKey` newtype, bounds, constants |
| `Value` bits | `phalcom-core/src/value/repr.rs` | Allocate upper 24 bits, accessors, constructor integration |
| Option/wrappers | `phalcom-core/src/value/option.rs` and future wrapper layer | Preserve base key while push/pop; effective-key override |
| Class rows | `phalcom-core/src/heap/class.rs` | Add stable instance-behavior `DispatchKey` |
| Key allocation | `phalcom-core/src/heap/mod.rs` | Allocate unique non-reused keys with class rows |
| Universe | `phalcom-core/src/universe/core_classes.rs` | Assign well-known core keys |
| Class construction | `phalcom-core/src/vm/api.rs` | Ensure dynamic class/metaclass rows get keys |
| Object values | VM allocation/factory paths | Replace raw unkeyed `Value::obj` publication |
| ADTs | `phalcom-core/src/adt.rs`, `vm/adt.rs` | Cache behavior key in descriptors and Values |
| Data | `phalcom-core/src/data.rs`, `vm/data.rs` | Same |
| Products | `phalcom-core/src/product/storage.rs` | Ensure scalar constructors reconstruct canonical keys |
| Dispatch IC | `phalcom-core/src/chunk.rs` | `class` → `dispatch_key` |
| Invoke | `phalcom-core/src/vm/dispatch.rs` | Probe key before `Value::class` |
| Raw representation | product/trace/debug code | Permit and preserve key bits |
| Equality | `value/repr.rs`, `value/mod.rs` | Confirm `==`, `Hash`, `===` contracts |
| VM spec | `docs/specs/vm/values-and-objects.md` | Replace “reserved=zero” with normative key field |
| Performance | benchmark/perf-log infrastructure | A/B measurement and profile |

---

# 55. Implementation order

I would implement this in seven deliberately separated checkpoints.

## Checkpoint 1 — Freeze semantics and representation

Before code:

1. define `DispatchKey`;
2. declare bits `40..=63`;
3. specify VM-local uniqueness;
4. specify non-reuse;
5. specify base-vs-effective wrapper behavior;
6. specify that key is not a method version;
7. specify that `ClassId` remains authoritative for reflection;
8. coordinate explicitly with native unary wrappers.

Do this in the authoritative VM spec first.

---

## Checkpoint 2 — Add keys to class rows without changing dispatch

Implement:

```text
ClassObject.dispatch_key
key allocator
core fixed-key reservation
dynamic allocation
bootstrap uniqueness verification
```

Continue using old `Value::class` in sends.

This makes the identity substrate independently testable.

---

## Checkpoint 3 — Key the `Value` representation

Use bits `40..=63`.

Implement:

```text
base_dispatch_key()
with_base_dispatch_key()
dispatch_key()
```

Make all immediate constructors canonical.

Update raw-word round trips.

Do not change the inline cache yet.

This isolates representation correctness from dispatch behavior.

---

## Checkpoint 4 — Migrate every guest-visible object construction boundary

This is likely the largest patch.

Audit every occurrence of:

```rust
Value::obj(...)
```

Classify them by runtime class source:

```text
instance       → InstanceObject.class
class object   → ClassObject.class
string         → String
list           → List
fiber          → Fiber
module         → Module or Package
method         → Method
closure        → Closure or Block
ADT case       → descriptor.behavior_class
data object    → descriptor.behavior_class
typing object  → runtime descriptor class
reflection     → corresponding reflection class
...
```

Introduce VM factories where necessary.

At the end of this checkpoint, an unkeyed guest object `Value` should be impossible.

---

## Checkpoint 5 — Switch the IC guard

Only now change:

```rust
InlineCache {
    class: ClassId,
    ...
}
```

to:

```rust
InlineCache {
    dispatch_key: DispatchKey,
    ...
}
```

and move `Value::class` off the cache-hit path.

Leave the miss path unchanged.

This should be a remarkably small semantic change once the representation groundwork is complete.

---

## Checkpoint 6 — Full correctness matrix

Test:

```text
immediates
objects
classes/metaclasses
native object variants
ADTs
data
products
wrappers
GC
method redefinition
world-version invalidation
equality
hashing
===
```

No performance judgement yet.

---

## Checkpoint 7 — Benchmark and decide

Measure the current benchmark corpus.

Only then decide whether the complexity earned its place.

---

# 56. Required tests

The test surface should be much broader than a single `bare_send` benchmark.

## Representation tests

```rust
assert_eq!(size_of::<Value>(), 16);
```

must remain true.

Test:

```text
tag bits survive key insertion
wrapper bits survive key insertion
key bits survive wrapper push/pop
raw_words round-trip key exactly
```

---

## Identity tests

For each class pair:

```text
same ClassId → same key
different ClassId → different key
```

including:

```text
user classes
metaclasses
native classes
variant behavior classes
data behavior classes
```

---

## Cross-representation tests

These are especially important:

```text
small Int        key == large heap Int key
ADT singleton    key == heap ADT case key
data singleton   key == heap data-object key
```

where they share runtime behavior classes.

---

## Boolean tests

```text
true.dispatch_key()  != false.dispatch_key()
true key             == True class key
false key            == False class key
```

---

## Class-object tests

If:

```text
class Foo
```

then:

```text
Foo instance          → DK_FOO
Value representing Foo → DK_FOO_METACLASS
```

This will catch the most likely metaclass confusion.

---

## Option/wrapper tests

For:

```text
user
Some(user)
Some(Some(user))
```

assert:

```text
base key remains DK_USER
effective keys are:
    DK_USER
    DK_SOME
    DK_SOME
```

Then peel wrappers and confirm:

```text
DK_USER
```

returns without heap lookup.

After Result wrappers:

```text
Some(Ok(Error(user)))
```

should successively expose:

```text
DK_SOME
DK_RESULT_OK
DK_RESULT_ERROR
DK_USER
```

---

## Cache invalidation test

1. warm cache for `User#foo`;
2. record receiver key;
3. redefine `foo`;
4. assert receiver key unchanged;
5. assert `world_version` changed;
6. next send must miss and refill;
7. subsequent send hits new method.

This test proves the crucial separation between class identity and method-world identity.

---

## GC test

Create:

```text
Some(Some(object))
```

where that wrapped `Value` is the only root.

Force GC.

Assert the underlying `ObjRef` survives.

The presence of key bits must not alter tracing.

---

## Product-storage tests

Round-trip through:

```text
ProductSlotRepr::Value
ProductSlotRepr::Int64
ProductSlotRepr::Float64
ProductSlotRepr::Bool
ProductSlotRepr::Symbol
```

and verify the reconstructed `Value` has the correct dispatch identity.

The generic lane should preserve key bits.

The scalar lanes should recreate them through constructors.

---

## Equality tests

For every category:

```text
old == behavior unchanged
old hash behavior unchanged
old === behavior unchanged
```

Specifically verify that two independently constructed equal immediate Values receive the same canonical key and therefore remain bit-identical where they were before.

---

## Exhaustion test

The allocator must not:

```text
MAX → 0 → reuse
```

Test the boundary directly with a small/internal allocator constructor rather than allocating sixteen million classes.

---

# 57. Benchmark matrix

I would add focused dispatch benchmarks before comparing the whole corpus.

### Monomorphic user object

```phalcom
class Counter {
    value { 1 }
}

const c = Counter.new()

for _ in ... {
    c.value
}
```

This is the optimization's best-case intended target.

---

### Class-side/metaclass send

Repeated send to:

```phalcom
SomeClass.someClassMethod
```

This catches class-object/metaclass key handling.

---

### Immediate integer send

Repeated:

```phalcom
x + 1
```

to ensure well-known-key extraction is cheap.

---

### Heap-native object send

Use something such as String/List where class currently requires object discrimination.

This should show more benefit than pure immediate dispatch if the previous path involved heap access.

---

### ADT singleton

Repeated method/send on a singleton variant.

Tests elimination of descriptor lookup.

---

### ADT payload case

Same for heap-backed variant.

---

### Data singleton and data payload

Same rationale.

---

### Bimorphic site

Alternate:

```text
A
B
A
B
```

The initial `DispatchKey` patch should still thrash because the IC remains monomorphic.

That is useful baseline data for deciding whether a PIC is worth implementing.

---

### Variadic send

The perf history already calls this path out as important.

Measure it separately.

---

# 58. Whole-program performance corpus

At minimum retain the established relevant workloads:

```text
bare_send
arith_send
variadic_send
for
string_equals
fib
binary_trees
map_numeric
```

and the heavier allocation/concurrency workloads where relevant.

Measure:

```text
wall time
user time
system time
RSS
instructions
branches
branch misses
cache misses where available
```

and obtain a sampled profile.

Do not judge the optimization from one microbenchmark.

---

# 59. Instrument IC behavior before implementing PICs

A small instrumentation mode should count:

```text
Invoke count
IC hits
IC misses
site transitions:
    first class
    same class
    second distinct class
    third+
```

An even more useful histogram would report:

```text
sites seeing 1 DispatchKey
sites seeing 2 DispatchKeys
sites seeing 3–4 DispatchKeys
sites seeing 5+
```

That directly answers whether the next dispatch optimization should be:

```text
PIC
megamorphic cache
or neither
```

rather than guessing.

---

# 60. Performance acceptance criteria

I would use conservative criteria.

The patch is worthwhile if:

1. correctness suite is entirely unchanged;
2. `Value` remains 16 bytes;
3. no meaningful regression occurs on non-send-heavy workloads;
4. send-heavy monomorphic benchmarks improve reproducibly;
5. current profiles confirm reduced/eliminated `Value::class` samples;
6. construction-time key stamping does not merely move equivalent cost into a hotter allocation path;
7. code complexity remains localized around construction and dispatch identity.

I would not require an arbitrary 5% whole-suite win.

Based on the historical 4% `Value::class` profile contribution, that would be unrealistic.

Even a reproducible 2–3% improvement in a major send-heavy workload can be a good runtime optimization if the architecture becomes cleaner and the key becomes useful for PICs later.

---

# 61. Main risk: construction authority

The largest correctness risk is not the IC.

It is accidentally producing:

```text
ObjRef X + DispatchKey Y
```

where `Y` does not correspond to the object's runtime class.

That can cause a cache to execute the wrong method.

Therefore this optimization should be treated almost like adding a type-safety invariant to `Value`.

Do not expose arbitrary public constructors for keyed object Values.

Centralize construction.

Debug-verify aggressively.

---

# 62. Risk matrix

| Risk | Severity | Mitigation |
|---|---:|---|
| Wrong key attached to object | Critical | VM-owned constructors + debug cross-check |
| Key collision | Critical | Monotonic checked allocator |
| Key reuse after class death | Critical | Never reuse |
| Key changed on method redefine | High | Explicit identity/version separation |
| Class object gets its own instance key instead of metaclass key | Critical | Dedicated metaclass tests |
| Wrapper overwrites base key | High | Preserve key during wrapper push/pop |
| Product scalar reconstruction loses key | High | Canonical immediate constructors |
| `===` changes unexpectedly | High | Canonical keys + exact regression tests |
| Raw word decoding rejects new metadata | Medium | Update representation validator |
| IC gets larger | Low | Likely still 24 B; benchmark separately |
| GC starts treating key as pointer | Critical | Keep tracing payload/tag based |
| Key exhaustion wraps | Critical | Checked exhaustion |
| Result wrapper spill exposes physical class | High | Logical-wrapper dispatch rule |

---

# 63. Other optimization opportunities revealed by this audit

The repository suggests a sensible hierarchy of future runtime work.

## A. `DispatchKey` IC guard

Low conceptual risk after construction migration.

Target:

```text
~Value::class residual
```

Recommended now.

---

## B. Invocation-target caching

Historical potential larger than key extraction because `call_method` remained a much larger profile category.

Possible target:

```text
Method ObjRef
    ↓
MethodObject
    ↓
Primitive / interpreted target
```

Worth investigating after the key patch.

---

## C. Small PICs

Potentially strong if instrumentation shows many bimorphic/trimorphic sites.

`DispatchKey` is excellent infrastructure for this.

---

## D. Megamorphic `(DispatchKey, Selector)` cache

Useful only if highly polymorphic send sites matter.

Instrument first.

---

## E. Per-class method epochs

Potential reduction in unnecessary global invalidation.

Architecturally much harder because inherited resolutions depend on ancestors.

Not part of the key patch.

---

## F. Method-table representation

Could accelerate cold misses and reflective lookup.

Not useful to warm monomorphic hits once ICs work.

Lower priority unless miss rates warrant it.

---

## G. More superinstructions

Already shown to produce significant wins in the right workloads. 

Continue data-driven fusion.

---

## H. Interpreter-loop overhead

Historical profiles identified interpreter mechanics as substantial residual cost after earlier cache work. 

This likely remains strategically larger than any one class-identity micro-optimization.

---

## I. 8-byte Values

Potentially very large density win, but blocked by the full 64-bit `ObjRef` representation today. 

Treat as a future heap/handle architecture project, not a competing immediate optimization.

---

# 64. What I would explicitly reject

## Rejected: store `ClassId` itself in the metadata

It is 64-bit and does not fit.

Truncating it destroys identity guarantees.

---

## Rejected: derive key from `ObjRef` low bits

Generational handles make this unsafe.

A dispatch guard cannot tolerate collisions.

---

## Rejected: put key only in object headers

Still requires heap access on every IC hit.

Misses the main optimization.

---

## Rejected: mutate keys after method redefinition

Would require updating every copied `Value` in stacks, fields, closures, globals, products, and parked fibers.

Completely wrong abstraction.

---

## Rejected: key each individual object

Destroys monomorphic cache usefulness.

The identity is the behavior class.

---

## Rejected: merge classes with identical method tables

Open/redefinable classes make this unsafe and unnecessarily complex.

---

## Rejected: permanently permit unkeyed object Values

Creates dual fast/slow representation semantics and hides missing construction migrations.

---

## Rejected: remove `InstanceObject::class`

DispatchKey is intentionally not sufficient for reflection or hierarchy semantics.

---

## Rejected: build PICs simultaneously

It would obscure whether the base optimization itself works and substantially enlarge the patch/testing surface.

---

# 65. Recommended end-state architecture

The resulting runtime should look like this:

```text
                           ┌───────────────────┐
                           │     ClassObject   │
                           │                   │
                           │ ClassId           │
                           │ superclass        │
                           │ methods           │
                           │ DispatchKey       │
                           └─────────┬─────────┘
                                     │
                            authoritative mapping
                                     │
                                     ▼
┌──────────────────────────────────────────────────────────┐
│                         Value                            │
│                                                          │
│ payload                       meta                        │
│ ┌──────────────────┐         ┌─────────────────────────┐ │
│ │ immediate /      │         │ tag                     │ │
│ │ ObjRef / ID      │         │ wrapper metadata        │ │
│ │                  │         │ base DispatchKey        │ │
│ └──────────────────┘         └─────────────────────────┘ │
└──────────────────────────────┬───────────────────────────┘
                               │
                               │ dispatch_key()
                               ▼
                         DispatchKey
                               │
                               ▼
                  ┌─────────────────────────┐
                  │ per-call-site IC        │
                  │                         │
                  │ key                     │
                  │ method                  │
                  │ world_version           │
                  └───────────┬─────────────┘
                              │
                    hit       │      miss
                 ┌────────────┴────────────┐
                 ▼                         ▼
           call cached               derive ClassId
             method                  hierarchy lookup
                                         │
                                         ▼
                                      refill IC
```

The crucial division is:

```text
Value metadata answers:
    “What dispatch domain is this receiver in?”

ClassId answers:
    “What actual runtime behavior object is this?”

world_version answers:
    “Is this cached method resolution still current?”
```

Those are three separate questions.

Phalcom should represent them separately.

---

# 66. Why this improves the runtime architecture beyond raw speed

The strongest argument for the change is not actually the ~4% historical profile contribution.

It is that it removes heap representation from the dispatch-cache guard.

Today:

```text
IC hit
    depends on Value representation
    depends on heap object representation
    depends on Object enum layout
    depends on class derivation rules
```

Afterward:

```text
IC hit
    depends on Value metadata
    depends on IC metadata
```

That is a cleaner abstraction boundary.

It means Phalcom can later change:

```text
InstanceObject layout
native String representation
ADT storage
Data storage
Fiber internals
Module representation
LargeInt representation
```

without necessarily changing the common cache-hit path.

Each representation only has to obey one law:

> Produce a `Value` with the correct runtime dispatch identity.

That is a valuable VM invariant.

---

# 67. Priority relative to other optimizations

I would rank it as a **medium-sized architectural optimization with low-single-digit expected runtime return and high future leverage**.

Not the next “30% win.”

Not negligible either.

Relative priority:

```text
Tier A — measured large residual costs
    interpreter-loop mechanics
    invocation mechanics
    allocation/pathological copies where still present

Tier B — clean measured hot-path reductions
    DispatchKey-in-Value
    targeted superinstructions
    targeted cache improvements

Tier C — workload-dependent dispatch specialization
    PIC
    megamorphic cache
    method target specialization

Tier D — major representation projects
    ObjRef compression
    8-byte Value
    heap architecture changes
```

The reason I would still implement `DispatchKey` before PICs is that it establishes the correct primitive on which those more complicated caches should be built.

---

# 68. Concrete recommendation

I recommend proceeding with the optimization under these exact design rulings:

1. **Use meta bits `40..=63` for a 24-bit `DispatchKey`.**

2. **Define `DispatchKey` as a VM-local, injective identity of runtime behavior `ClassId`.**

3. **Never derive it by truncating `ObjRef`.**

4. **Never recycle it within a VM.**

5. **Reserve key `0` as invalid.**

6. **Reserve a small fixed range for core immediate/wrapper behavior classes.**

7. **Store the key for instances governed by each class row on `ClassObject`.**

8. **Keep `ClassId` everywhere it is semantically required.**

9. **Store the base/unwrapped key in `Value`; wrappers compute an effective outer key without destroying the base key.**

10. **Make every guest-visible object `Value` keyed at construction time.**

11. **Move raw object-Value creation behind VM/runtime-owned factories.**

12. **Cache behavior keys in ADT/Data runtime descriptors.**

13. **Change the monomorphic IC guard from `ClassId` to `DispatchKey`.**

14. **Keep the current full `Value::class`/hierarchy lookup on cache misses.**

15. **Keep `world_version` unchanged.**

16. **Do not introduce a reverse key→class table initially.**

17. **Do not bundle PICs, per-class epochs, or method-table redesign with this patch.**

18. **Preserve `Value` at exactly 16 bytes.**

19. **Preserve equality/hash/`===` semantics through canonical key construction.**

20. **Benchmark against current HEAD and treat the historical ~4% `Value::class` profile share as a ceiling clue, not as a promised result.**

---

# Final assessment

This optimization survives repository-level scrutiny.

More importantly, the audit changes the way I would describe it.

It is not:

> “cache the object's class inside `Value`.”

It is:

> **Introduce an explicit compact runtime dispatch identity, make it a canonical property of every `Value`, and use it to decouple warm dispatch from heap/class representation.**

That distinction is important.

`ClassId` is a semantic runtime object identity.

`DispatchKey` is a hot-path comparison identity.

`world_version` is cache-state identity.

Keeping those three concepts separate produces a much stronger runtime architecture.

The repository's current 16-byte `Value`, its intentionally untouched 24-bit metadata region, the existing monomorphic IC, the native-wrapper representation work, the non-moving stable-handle heap, and the current prohibition on guest class mutation all line up unusually well for this design.

I would therefore proceed with it—but as a carefully staged representation invariant first and a dispatch optimization second. The actual IC change should be almost the final and smallest step of the implementation, not the first.