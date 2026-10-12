---
id: COLL006.C1.P7
category: COLL
program: COLL006
checkpoint: COLL006.C1
kind: implementation-plan
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
depends_on:
  - COLL006.C1.P6
follows: COLL006.C1.P6
supersedes: null
repository: aureat/phalcom-lang
branch: feat/telos-collection-upgrade
prepared_against_remote_revision: ffa2ab96578acf2f176aa2ddbd31e655590c7d29
prepared_against_checkpoint_snapshot: 2026-10-11 user-supplied COLL006.C1-CHECKPOINT.md
---

# COLL006.C1.P7 — Persistent Hash Family: `Bag<T>` and `Bijection<K,V>`

## 0. Executor contract

This plan is designed for GPT-5.6 Luna High/Extra High execution under higher-end architectural supervision. It is a patch-grade implementation contract, not permission to redesign the collection model.

The implementer:

- may adapt private names, file decomposition, imports, and equivalent mechanical APIs to the live repository;
- must preserve the architecture, ownership boundaries, public semantics, GC model, key relation, and persistent invariants defined below;
- must not infer a public selector/result/error contract that has not been explicitly ratified;
- must not add native bindings outside an accepted TDR-0017 floor amendment;
- must not hide unbounded persistent `Value`-bearing graphs behind `Arc`, `Rc`, or another untracked native arena;
- must not hold `Heap`/object borrows across re-entrant Telos `hash`/`==` sends;
- must follow the testing budget instead of running broad suites after every edit;
- must classify and defer unrelated failures rather than opportunistically repairing them;
- must STOP AND CONSULT when an escalation trigger fires;
- must keep `COLL006.C1-CHECKPOINT.md` current at durable task/gate boundaries;
- must produce `COLL006.C1.P7-walkthrough.md` and `COLL006.C1.P7-handoff.md` at completion.

Testing is evidence gathering, not ritual. BUILD mode uses the smallest discriminating command that can falsify the current hypothesis. Workspace/release certification remains P9-owned unless evidence requires a narrower cross-crate check.

**Do not start production P7 edits while P6 is still materially incomplete.** Planning may proceed now; execution begins only after T0 proves that the live worktree contains a coherent P6 takeover state.

---

## 1. Goal

Implement one reusable, structurally shared, GC-visible persistent hash substrate and use it to realize immutable `Bag<T>` and `Bijection<K,V>` collection families with old-root immutability, precise tracing, Telos-defined hashing/equality, collision correctness, failure atomicity, and path-copy update complexity.

---

## 2. Checkpoint acceptance objective

`COLL006.C1.P7` is one numbered plan inside `COLL006.C1`; it does **not** close the checkpoint.

It advances the C1 acceptance objective that:

```text
persistent hash storage exists as a reusable mechanism
Bag<T> is a persistent immutable multiset
Bijection<K,V> is a persistent immutable one-to-one bidirectional mapping
persistent updates preserve old roots and structurally share unaffected nodes
persistent node graphs participate in precise GC reachability and allocation pressure
Telos hash/equality laws and callback safety remain consistent with the existing hash family
```

P8 remains responsible for persistent **ordered** storage and `RangeMap<K,V>`. P9 remains responsible for whole-C1 API/conversion convergence, `Entry<K,V>` migration, broad property/performance certification, documentation convergence, and final residual classification.

P7 is therefore a **foundation + feature** plan: it establishes a private persistent hash engine and then implements the two C1 public families that depend on it.

---

## 3. Repository grounding

### 3.1 Prepared-against state

```text
repository:             aureat/phalcom-lang
branch:                 feat/telos-collection-upgrade
remote branch revision: ffa2ab96578acf2f176aa2ddbd31e655590c7d29
remote commit message:  fix(COLL006): close P5 audit follow-up
checkpoint snapshot:    user-supplied COLL006.C1-CHECKPOINT.md, newer than remote branch
predecessor:            COLL006.C1.P6, currently active/partial in checkpoint snapshot
```

The uploaded checkpoint is the newer execution authority. It records P6 T2 public API approval, TDR-0092 acceptance for exactly 29 P6 bindings, and partially implemented uncommitted P6 work. The remote branch at `ffa2ab...` does not contain `docs/decisions/accepted/0092-ordered-mutable-hash-collection-native-floor.md` or `docs/specs/collection-families.md` yet. P7 execution must therefore reconcile the actual live worktree before editing; never overwrite or reconstruct newer P6 work from the older remote snapshot.

### 3.2 Verified live/remote repository facts

The following facts were verified against `feat/telos-collection-upgrade` at `ffa2ab96578acf2f176aa2ddbd31e655590c7d29` and against the supplied checkpoint/handoff records:

1. `phalcom-core/src/primitive/ordered_hash.rs` is the committed shared Map/Set key engine. It centralizes Telos `hash`, candidate `==`, numeric SameValueZero, and per-send re-entry bracketing.
2. `ordered_hash::KeyedCollectionKind` currently contains `Map` and `Set` only on the remote branch; P6 is expected to extend/reuse this authority for its new mutable families.
3. `primitive/mod.rs::send_hash` re-enters the VM and requires an immediate `Int`; `send_eq` re-enters the VM and requires `Bool`.
4. `primitive/mod.rs::is_mutable_collection_key` is the central built-in key-admission classifier; the committed remote version includes List/Array/Buffer/Slice/Map/Set/Bytes. P6 must add its mutable collection families before P7 relies on final policy.
5. `value::same_value_zero` equates numeric `+0.0/-0.0`, treats numeric NaNs as equal for key purposes, and otherwise uses Telos value equality for numeric cross-representation comparisons.
6. `Number#hash` has representation-sensitive branches for immediate Int, Float, and heap-backed LargeInt. Source inspection exposes a potential hash-law mismatch for SameValueZero-equal values such as large integral Float versus LargeInt/Int, and for NaN payload variants. P7 must test this before claiming hash-law correctness.
7. The VM heap is a `SlotMap<ObjRef,Object>` with non-moving precise mark/sweep. `Heap::alloc` latches GC pending; it does not collect synchronously.
8. Re-entering the interpreter can reach a GC safepoint. `VM::push_temp_root`, `temp_root_depth`, and `truncate_temp_roots` exist specifically for native values/handles held only in Rust locals across re-entrant sends.
9. `heap/trace.rs::trace_object` is exhaustive. Every new `Object` arm must explicitly enumerate its outgoing `ObjRef`/`Value` edges.
10. P5 measured `Value`/`Object` at 16/48 bytes in its final worktree. P7 must record the live P7-start sizes and must not increase `Object` slot size as a side effect of persistent nodes/public wrappers.
11. Existing large `Object` payloads are boxed to protect arena slot size. New Bag/Bijection/node arms should follow that established pattern when needed.
12. The existing `Collection` nominal root and canonical `Iterator`/`Iterable` traits are already present in Universe source. P7 reuses them; no parallel protocol hierarchy is allowed.
13. Universe/native identity is explicit across `phalcom-native-meta`, semantic `CoreDeclarationIds`, `core_classes`, builtin source registration, native primitive installation, `Value::get_class`, rendering, reflection/native-surface generation, and tests.
14. Current collection source modules live under `phalcom-core/core/universe/src/collections/` and are exposed by `collections/package.ph`; builtin source is enumerated in `phalcom-modules/src/builtin.rs`.
15. `BijectionConflictError` is currently only proposed in C1 implementation records; no source class exists on the remote branch.
16. `docs/specs/README.md` makes `docs/specs/` normative over implementation records and TDR history. The remote branch does not yet contain the checkpoint-referenced `docs/specs/collection-families.md`; T0 must verify the newer worktree version before P7 uses it.
17. `docs/decisions/README.md` makes TDRs historical decision evidence rather than the normative language spec. New floor authority must still be reflected in the owning spec.
18. `phalcom-repl/src/snapshot.rs` has an explicit core-class list containing only a subset of classes; P7 should touch it only if live reflection/repl tests establish that Bag/Bijection belong there. Do not expand it merely because a new class exists.
19. `vm/dispatch.rs` contains special Map expansion paths. Bag/Bijection do not participate in `**`/record expansion by default; do not add special dispatch paths without explicit semantic authority.
20. No implemented `PersistentHashNode`, Bag runtime, or Bijection runtime exists in the inspected committed repository.

### 3.3 Repository-state warning

`STATUS.md` and `PROGRAM.md` can lag the checkpoint. The uploaded `STATUS.md` still says P6 is next, while the newer checkpoint records P6 as active with TDR-0092 accepted. The checkpoint is the single durable C1 state authority. T0 must reconcile the live documents without treating stale summary text as semantic authority.

> Re-read every named live path before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

---

## 4. Required reads before implementation

Read only the material needed for P7. Do not restart a broad repository audit if the P6 handoff and live tree agree.

### Workflow and state

```text
AGENTS.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/shared-consultation-escalation-protocol.md

docs/implementation/COLL006/PROGRAM.md
docs/implementation/COLL006/STATUS.md
docs/implementation/COLL006/COLL006.C1/COLL006.C1-CHECKPOINT.md
```

### Predecessor records

```text
docs/implementation/COLL006/COLL006.C1/COLL006.C1.P6-detailed-implementation-plan.md
docs/implementation/COLL006/COLL006.C1/COLL006.C1.P6-walkthrough.md
docs/implementation/COLL006/COLL006.C1/COLL006.C1.P6-handoff.md
```

The walkthrough/handoff may not exist at planning time. They are **P7 execution preconditions**. If P6 is still partial, stop T0 after recording the state; do not implement P7 on top of an unstable P6 substrate.

### Normative/specification authority

```text
docs/specs/README.md
docs/specs/collection-families.md              # checkpoint says P6 created it; verify live path
docs/specs/universe.md
docs/specs/vm/values-and-objects.md
```

If `collection-families.md` has been renamed or decomposed, follow `docs/specs/README.md` to the current owner.

### Accepted decisions

```text
docs/decisions/accepted/0017-freeze-vm-blessed-primitive-floor.md
docs/decisions/accepted/0028-collections-representation-and-literals.md
docs/decisions/accepted/0033-amend-floor-admit-collection-container-primitives.md
docs/decisions/accepted/0047-non-moving-mark-sweep-collector.md
docs/decisions/accepted/0063-admit-bytes-native-octet-buffer.md
docs/decisions/accepted/0092-ordered-mutable-hash-collection-native-floor.md  # verify live after P6
```

### Runtime/hash/GC owners

```text
phalcom-core/src/primitive/ordered_hash.rs
phalcom-core/src/primitive/mod.rs
phalcom-core/src/primitive/map.rs
phalcom-core/src/primitive/set.rs
phalcom-core/src/primitive/number.rs
phalcom-core/src/value/mod.rs
phalcom-core/src/vm/gc.rs
phalcom-core/src/heap/mod.rs
phalcom-core/src/heap/object.rs
phalcom-core/src/heap/accessors.rs
phalcom-core/src/heap/trace.rs
```

### Universe/native surface

```text
phalcom-native-meta/src/universe.rs
phalcom-semantic/src/core_surface/identity.rs
phalcom-core/src/universe/core_classes.rs
phalcom-core/src/universe/primitives.rs
phalcom-native-surface/src/generated.rs
phalcom-native-surface-gen/src/main.rs
phalcom-modules/src/builtin.rs
phalcom-core/core/universe/src/collections/package.ph
phalcom-core/core/universe/src/collections/iterator_laws.ph
phalcom-core/core/universe/src/errors/argument.ph
```

### Test conventions

```text
phalcom-core/tests/README.md
phalcom-core/tests/core/mod.rs
phalcom-core/tests/core/collections/contract.rs
phalcom-core/tests/core/memory/gc.rs
phalcom-core/tests/core/object_model/invariants.rs
```

---

## 5. Normative authority

### 5.1 Current normative behavior

The P7 implementation must preserve the following C1 requirements:

```text
Bag<T>
    persistent immutable multiset
    logical key -> positive exact multiplicity
    equality by multiplicity, independent of representation/traversal order

Bijection<K,V>
    persistent immutable finite partial bijection
    forward[k] = v iff reverse[v] = k
    no duplicate values
    no silent displacement

Persistent hash storage
    structurally shared
    old public roots immutable
    collision-correct
    Telos hash/== semantics
    precise GC traversal
    allocation pressure visible to VM GC
    no Heap borrow across callbacks
    failed operation publishes no partial public result
```

The C1 checkpoint explicitly fixes old-root immutability and fully GC-visible persistent graphs. Those requirements may not be weakened for implementation convenience.

### 5.2 Existing hash authority

Existing Map/Set key semantics remain authoritative unless separately amended:

- Telos `hash` selects the hash path.
- Equal-hash candidate disambiguation uses Telos `==`, with numeric SameValueZero as the established numeric key relation.
- Hash/equality callbacks are re-entrant VM sends.
- No live heap borrow crosses those sends.
- Built-in mutable/content-unstable collection keys are rejected by the central key-admission policy.

P7 may extract a lower-level shared key helper from P6's final engine, but must not introduce a second key-equivalence definition.

### 5.3 Runtime/GC authority

TDR-0047 and the live VM establish:

```text
non-moving precise mark/sweep
ObjRef handle stability
exhaustive Object tracing
GC only at safepoints
allocation latches collection rather than collecting immediately
VM temp roots protect Rust-local handles across interpreter re-entry
```

P7 extends this model. It does not create a node arena, reference-counted island, finalizer, write barrier, or second collector.

### 5.4 Native floor authority

TDR-0017 freezes the native floor. TDR-0028/TDR-0033/TDR-0063 establish precedents for native collection storage and callback-free bulk operations. TDR-0092 is P6-specific and does **not** authorize P7 Bag/Bijection bindings.

P7 therefore requires a separate scoped floor amendment before its native methods are finalized/registered.

### 5.5 Unresolved public contract

The C1 records fix Bag/Bijection data models but do **not** ratify every selector spelling, return channel, constructor behavior, Bag algebra spelling, or P7 floor inventory.

This plan supplies a concrete recommendation at T4, but those source-visible choices remain **PROPOSED** until explicitly approved and reflected in `docs/specs/`.

---

## 6. Takeover state

### 6.1 Stable interface map

| Concept | Current symbol/path | Owner | P7 invariant |
|---|---|---|---|
| Canonical key callback dispatch | `primitive/mod.rs::{send_hash,send_eq}` | runtime primitive layer | Reuse; do not clone semantics |
| Mutable hash lookup orchestration | `primitive/ordered_hash.rs` | P6 hash mechanism | Final P6 API is prerequisite; extract only a narrower shared key relation if needed |
| Numeric key equivalence | `value::same_value_zero` | runtime value semantics | Persistent collections must agree with Map/Set |
| Built-in mutable-key rejection | `primitive/mod.rs::is_mutable_collection_key` | runtime primitive layer | Consume final P6 classifier; Bag/Bijection remain immutable candidates |
| Heap handles | `heap::ObjRef` | heap/runtime | Persistent edges use stable `ObjRef` |
| Heap allocation/pressure | `Heap::insert/alloc` | heap/runtime | Every persistent node is counted by heap allocation |
| GC roots | `VM::collect_roots` | VM | Public receiver/intermediate callback-live handles must remain rooted |
| Native temp roots | `VM::{push_temp_root,temp_root_depth,truncate_temp_roots}` | VM | Use across re-entrant calls when Rust locals otherwise own the only edge |
| Object tracing | `heap/trace.rs::trace_object` | heap/GC | New public objects and private nodes enumerate exact outgoing edges |
| Universe identity | `phalcom-native-meta/src/universe.rs` | native metadata | Bag/Bijection receive canonical identities; private nodes do not |
| Runtime class table | `universe/core_classes.rs` | VM bootstrap | Bag/Bijection derive from `Collection` |
| Source module provider | `phalcom-modules/src/builtin.rs` | modules | Add modules only after files exist |
| Collection package | `core/universe/src/collections/package.ph` | Universe source | Expose Bag/Bijection |
| Iterable protocol | `collections/iterator_laws.ph` | Universe source | Reuse existing `Iterator`/`Iterable` |
| Error vocabulary | `errors/argument.ph` | Universe source | Add Bijection conflict error only if approved |
| Native descriptor generator | `phalcom-native-surface-gen` | native-surface infra | Exact approved bindings must regenerate cleanly |
| Legacy floor audit | `tests/core/object_model/invariants.rs` | core tests | Add P7 owner/selectors; do not claim partial count is global census |

### 6.2 P6 handoff dependency

At P7 start, the P6 handoff must establish at minimum:

```text
Map/Set regression state
final shared ordered-hash/key helper boundary
final mutable-key classifier including P6 families
MutableBag/IndexedDict/OrderedSet class/object arms
TDR-0092 exact 29-binding floor installed
native-surface catalog current
P6 focused tests and remaining baseline issues
```

If the handoff does not establish these, P7 must not silently guess the final predecessor API.

---

## 7. Architecture

### 7.1 Architectural summary

Use one private immutable persistent hash map engine shared by Bag and both Bijection directions:

```text
Bag<T>
    public Object::Bag
        root: Option<ObjRef>
        total_count: exact Telos Int Value
        distinct_count: physical entry count
            |
            v
    PersistentHashNode graph

Bijection<K,V>
    public Object::Bijection
        forward_root: Option<ObjRef>  // K -> V
        reverse_root: Option<ObjRef>  // V -> K
        len: physical association count
            |                 |
            +-------+---------+
                    v
          PersistentHashNode graph
```

The private node graph is GC-visible and immutable after publication.

No public `PersistentHashMap` type is introduced.

### 7.2 Private node model

Preferred v1 representation:

```rust
enum PersistentHashNode {
    Leaf {
        hash: i64,
        key: Value,
        payload: Value,
    },

    Collision {
        hash: i64,
        entries: Box<[(Value, Value)]>,
    },

    Branch {
        bitmap: u32,
        children: Box<[ObjRef]>,
    },
}
```

This is a representation contract, not necessarily the exact final Rust syntax. Equivalent private wrappers are allowed if they preserve every invariant below.

Recommended `Object` arms:

```rust
Object::Bag(Box<BagObject>)
Object::Bijection(Box<BijectionObject>)
Object::PersistentHashNode(Box<PersistentHashNode>)
```

Box public/node payloads when necessary to preserve the P7-start `Object` slot size.

### 7.3 Trie branching/indexing

Use a 32-way bitmap-indexed trie (5 hash bits per level) unless live measurements or code constraints require an equivalent branch factor.

Conceptual helpers:

```rust
const BITS: u32 = 5;
const MASK: u64 = 0x1f;
const MAX_SHIFT: u32 = 60;

fn fragment(hash: i64, shift: u32) -> u32 {
    (((hash as u64) >> shift) & MASK) as u32
}

fn child_bit(fragment: u32) -> u32 {
    1u32 << fragment
}

fn compact_index(bitmap: u32, bit: u32) -> usize {
    (bitmap & (bit - 1)).count_ones() as usize
}
```

A signed `i64` hash is interpreted by bit pattern (`as u64`) rather than arithmetic signed shifting.

At `shift == 60`, the final fragment contains the remaining high bits. Equal full hashes route to a Collision node when keys differ.

### 7.4 Branch normalization

Keep normalization deliberately conservative for Luna implementation:

```text
Collision with 0 entries -> absent
Collision with 1 entry   -> Leaf
Collision with 2+        -> Collision
Branch with 0 children   -> absent
Branch with 1+ children  -> Branch
```

Do **not** implement aggressive singleton-branch promotion in v1 unless the node stores enough depth/prefix metadata to prove it safe. Promoting a Branch child across a level can change which hash fragment it interprets.

The bounded trie depth makes retaining a singleton branch acceptable.

### 7.5 Structural invariants

Every Branch satisfies:

```text
bitmap.count_ones() == children.len()
```

Every Leaf satisfies:

```text
stored hash == canonical hash used when association was constructed
```

Every Collision satisfies:

```text
all entries share the Collision hash
no two entries are equal under the canonical key relation
```

Published nodes are immutable. No method may mutate a node reachable from an existing public Bag/Bijection root.

### 7.6 Shared key relation

Factor/reuse one callback-safe key relation beneath mutable/persistent storage mechanics:

```text
hash_key(vm, value) -> PhResult<i64>
keys_equal(vm, candidate, query) -> PhResult<bool>
```

The exact helper names are flexible.

`keys_equal` must preserve the existing P6 semantics:

```text
if both values are numeric:
    SameValueZero
else:
    Telos == send
```

`hash_key` must use the canonical Telos `hash` send and existing hash-result validation.

Mutable collection re-entry locking remains owned by P6's mutable engine. Persistent tries do not need a structural-mutation lock because published nodes never mutate, but callback rooting and heap-borrow discipline remain mandatory.

### 7.7 Callback-safe operation phases

Every persistent operation that can invoke user Telos code is divided into two phases.

#### Phase A — inspect / preflight

May invoke `hash`/`==`.

```text
root callback-live receiver/arguments as necessary
compute canonical key hash
walk immutable nodes
copy candidate Value(s) before callback
resolve equality/collision candidate
validate semantic operation
compute exact Bag arithmetic where needed
record a structural update plan referencing old immutable nodes
```

No new public root is published.

#### Phase B — build / commit

Must invoke **no user code**.

```text
allocate replacement terminal node
path-copy only affected ancestors
reuse unaffected child ObjRefs
allocate new public Bag/Bijection wrapper
return the new public value
```

Because `Heap::alloc` only latches GC and does not collect immediately, fresh Phase-B handles can safely survive additional allocation as Rust locals **provided no interpreter re-entry occurs during Phase B**.

If implementation evidence requires a callback during build/commit, STOP AND CONSULT rather than smuggling a new rooting model into the algorithm.

### 7.8 Update-plan shape

A practical internal update plan can retain immutable path frames such as:

```rust
struct PathFrame {
    node: ObjRef,
    bit: u32,
    compact_index: usize,
}
```

Terminal result conceptually distinguishes:

```text
Absent
Present { existing_key, existing_payload, terminal_node }
Collision { ... }
```

The commit walks frames from leaf to root, allocating copied branches.

Do not store `&PersistentHashNode`, slices borrowed from Heap, or other live heap borrows in the plan across callbacks.

### 7.9 Lookup

Lookup algorithm:

```text
hash query key
start shift = 0
node = root

while Branch:
    frag = fragment(hash, shift)
    bit = 1 << frag
    if bitmap lacks bit: absent
    child = children[compact_index(bitmap, bit)]
    shift += 5

Leaf:
    if stored hash != query hash: absent
    else compare stored key with query key

Collision:
    if collision hash != query hash: absent
    else compare candidate keys in encounter order until equal/complete
```

Candidate keys must be copied out as `Value` before any Telos `==` send.

### 7.10 Insertion/replacement

Persistent generic map insertion supports:

```text
Absent -> insert Leaf
Equal existing key -> replace payload with a new Leaf/Collision, old terminal unchanged
Different hash -> build the minimum branch chain at differing hash fragment
Same hash, unequal key -> Collision
```

No generic trie operation decides Bag or Bijection conflict policy. The caller decides whether replacement is legal before commit.

### 7.11 Removal

Persistent removal supports:

```text
absent -> unchanged root
Leaf equal -> remove leaf
Collision -> copy without one entry; 1 entry normalizes to Leaf
Branch -> rebuild affected path; empty child disappears
```

Do not copy unrelated branches or entries.

### 7.12 Bag representation

Conceptual public root:

```rust
struct BagObject {
    root: Option<ObjRef>,
    total_count: Value,   // exact Telos Int, immediate or LargeInt-backed
    distinct_count: usize,
}
```

`total_count` is a semantic exact Int. It must use the existing BigInt/normalization/numeric policy; it is not host `usize`/`i64` state.

`distinct_count` is a physical entry count and may use `usize` internally. Public projection converts through established collection size conventions.

Each trie payload is an exact positive Telos Int `Value`.

### 7.13 Bag updates

Conceptually:

```text
adding(value, n):
    require n >= 0
    n == 0 -> unchanged Bag
    old = count(value)
    new = old + n
    total' = total + n
    distinct' = distinct + (old == 0 ? 1 : 0)
    path-copy with payload new

removing(value, n):
    require n >= 0
    n == 0 -> unchanged Bag
    old = count(value)
    removed = min(old, n)
    new = old - removed
    total' = total - removed
    if new == 0: remove trie key
    distinct' = distinct - (old > 0 && new == 0 ? 1 : 0)
```

No zero/negative multiplicity is stored.

### 7.14 Bag algebra

**Proposed public semantics for T4 approval:**

```text
(A + B)(x)             = A(x) + B(x)
union(A,B)(x)           = max(A(x), B(x))
intersection(A,B)(x)    = min(A(x), B(x))
difference(A,B)(x)      = max(0, A(x) - B(x))
```

These operations may remain source-level loops over persistent operations. They need not be native merely for speed.

### 7.15 Bijection representation

Conceptual public root:

```rust
struct BijectionObject {
    forward_root: Option<ObjRef>,  // K -> V
    reverse_root: Option<ObjRef>,  // V -> K
    len: usize,
}
```

Invariant:

```text
forward[k] = v iff reverse[v] = k
forward.size == reverse.size == len
```

### 7.16 Bijection atomic association

Strict association algorithm:

```text
Phase A:
    validate K is key-admissible
    validate V is key-admissible (V becomes a reverse key)
    compute hash(K)
    compute hash(V)
    inspect forward K association
    inspect reverse V association

    exact existing k -> v:
        idempotent success; may return self

    existing k -> otherV:
        conflict

    existing otherK -> v:
        conflict

    neither:
        prepare both path-copy update plans

Phase B:
    build new forward root
    build new reverse root
    allocate one Bijection wrapper containing both roots
    return
```

No public half-state is ever observable.

### 7.17 Bijection removal

Removal by key:

```text
lookup k in forward
absent -> unchanged self
obtain v
verify reverse[v] resolves to k (debug/internal invariant assertion)
prepare forward removal plan
prepare reverse removal plan
commit both roots
publish one wrapper
```

Removal by value should be source-derivable through `inverse` + removal-by-key + `inverse` unless the approved API/floor rules otherwise.

### 7.18 O(1) inverse

`inverse` should allocate at most the public wrapper:

```rust
BijectionObject {
    forward_root: old.reverse_root,
    reverse_root: old.forward_root,
    len: old.len,
}
```

No trie node allocation, traversal, hashing, or equality callback is required.

### 7.19 Iteration strategy

Avoid ordinal HAMT access (`rawAt(i)` implemented as a root traversal for each index), which risks O(n²) iteration.

Recommended v1 mechanism: one callback-free native snapshot of distinct keys into an `Array<Dynamic>` (or an equivalent existing stable sequence), then source-level iterators use that snapshot.

```text
Bag iterator:
    snapshot distinct keys once
    for current key, obtain multiplicity once
    yield current key lazily multiplicity times

Bijection iterator:
    snapshot forward keys once
    for each key, perform forward lookup and yield labeled (key:, value:) tuple
```

Bag iteration must never materialize one element per total occurrence. A Bag with an enormous exact count remains iterable lazily.

If a source-safe key snapshot cannot be expressed without an unsuitable native surface or excessive conversion machinery, STOP AND CONSULT before adding a private cursor class or repeated O(n) ordinal scans.

### 7.20 Equality

Keep equality in Telos source unless an approved floor decision says otherwise.

Bag equality:

```text
same class/family
same total count
same distinct count
for every distinct key in self:
    self.count(k) == other.count(k)
```

Traversal order is irrelevant.

Bijection equality:

```text
same class/family
same size
for every forward (k,v) in self:
    other.get(k) == Some(v)
```

The reverse root is an invariant, not an additional equality dimension.

Because public values are immutable, P6's mutable snapshot-equality policy is not needed here; ordinary rooted iterators/snapshots are sufficient.

### 7.21 Value hashing

Bag and Bijection are intended to be value-hashable when constituent values obey stable Telos hash laws.

Hash must be independent of trie shape, insertion history, and traversal order.

A source implementation can safely keep the final result inside immediate-Int range using the existing Vector pattern of modular arithmetic.

Recommended pattern (constants may change mechanically):

```text
P = 999999937

Bag entry contribution:
    mix(hash(key), hash(count)) mod P
Bag hash:
    (seed + totalCount.hash + distinctCount + sum(entry contributions)) mod P

Bijection pair contribution:
    directional_mix(hash(key), hash(value)) mod P
Bijection hash:
    (seed + size + sum(pair contributions)) mod P
```

Use a **commutative outer aggregate** but a **directional pair mix** for Bijection, so pair role is preserved while association order is ignored.

Required law:

```text
A == B => A.hash == B.hash
```

This depends on the shared child hash law. The numeric hash-law gate in T1 must pass or receive an owning-layer decision before P7 hashability is certified.

### 7.22 Key admission

Bag elements are trie keys and therefore use the canonical Map/Set key-admission policy.

Bijection uses both type parameters as trie keys:

```text
K is key in forward trie
V is key in reverse trie
```

Thus insertion must reject known mutable/content-unstable built-in collection values in **either** position.

Bag and Bijection themselves are immutable and should not be added to the mutable-key rejection list merely because they are collections.

### 7.23 Public/private type identity

Add canonical public identities:

```text
UniverseKey::Bag
UniverseKey::Bijection
```

with generic forms:

```text
Bag       : Type -> Type
Bijection : Type × Type -> Type
```

`PersistentHashNode` receives no Universe class/type identity and never becomes a source-visible `Value` product.

---

## 8. Ownership boundaries

### 8.1 Owns

P7 owns:

- the private GC-visible persistent hash node representation;
- callback-safe persistent lookup/update/remove orchestration;
- collision-node and bitmap-branch correctness;
- path-copy structural sharing and old-root immutability;
- Bag runtime/native/source implementation after public API approval;
- Bijection runtime/native/source implementation after public API approval;
- exact Bag multiplicity/total invariants;
- Bijection dual-root atomicity/inverse invariants;
- P7-specific native-floor consultation/amendment;
- P7-specific GC, re-entry, property/model, allocation-shape, and native-surface evidence;
- P7 lifecycle records.

### 8.2 Does not own

P7 does **not** own:

- P6 completion or redesign of MutableBag/IndexedDict/OrderedSet;
- changes to accepted Map/Set public semantics;
- a new hashability/ordering trait system;
- a new GC, node arena, ownership checker, or write barrier;
- persistent List/Vector/Deque/Queue/Stack;
- RangeMap persistent ordered tree storage (P8);
- whole-C1 `Entry<K,V>` migration or conversion convergence (P9);
- general constructor syntax redesign;
- sequence literal semantics;
- compiler/static-evaluation work unrelated to collection class identity;
- unrelated P4 D1/D2/D4 or P5 associated-family residual repair;
- broad release/workspace certification.

### 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| P7 public semantics | owning `docs/specs/` collection spec after T4 approval | Universe source/tests/runtime | implementation-plan-only semantics |
| Key hash/equality relation | final P6 shared key helper + Number/Value semantics | Map/Set/P6/P7 hash collections | P7-local equality/hash relation |
| Key admissibility | central `is_mutable_collection_key` policy | all hash insertion paths | class-name checks in Bag/Bijection only |
| Persistent node reachability | `Object` graph + `trace_object` | GC | hidden Arc/Rc graph |
| Bag total | `BagObject.total_count` + trie counts | source API | recomputed/cache copy that can diverge |
| Bag distinct count | `BagObject.distinct_count` + trie entry cardinality | source API | separate source-maintained count |
| Bijection association truth | forward/reverse roots in one `BijectionObject` | lookup/inverse/equality | two independently published collection objects |
| Native floor | accepted P7 TDR amendment | native registration/generator/audit | “internal means exempt” assumption |
| Plan execution state | `COLL006.C1-CHECKPOINT.md` | P7 walkthrough/handoff | new competing status file |

---

## 9. Global invariants

Use these IDs in tasks/tests.

```text
INV-01 — P7 consumes one canonical Telos key relation; it does not define a second hash/equality law.
INV-02 — No live Heap/object borrow crosses a re-entrant Telos hash or equality send.
INV-03 — Every callback-live Rust-local heap handle/value is reachable through VM roots/temp roots.
INV-04 — Published PersistentHashNode values are immutable.
INV-05 — A persistent update never mutates any node reachable from an older public root.
INV-06 — Unaffected subtrees are shared by ObjRef across versions.
INV-07 — Ordinary update allocation is bounded by trie depth/collision size, not collection cardinality.
INV-08 — Hash collision does not imply key equality; equal-hash candidates are compared by the canonical key relation.
INV-09 — Branch bitmap popcount equals child count.
INV-10 — Collision nodes contain only one full hash and no duplicate logical keys.
INV-11 — Persistent node graphs are fully visible to precise tracing and heap allocation pressure.
INV-12 — P7 does not increase the verified P7-start size_of::<Object>(); box new variants as needed.
INV-13 — Callback failure publishes no new public collection value representing a partial operation.
INV-14 — Build/commit phase invokes no user code.
INV-15 — Bag stores no zero or negative multiplicity.
INV-16 — Bag multiplicities and totalCount are exact Telos Int, not host-width counters.
INV-17 — Bag totalCount equals the mathematical sum of stored multiplicities.
INV-18 — Bag distinctCount equals trie logical-key cardinality.
INV-19 — Bag equality is multiplicity-based and traversal/representation-order independent.
INV-20 — Bag iteration expands occurrences lazily and does not allocate O(totalCount) storage.
INV-21 — Bijection forward[k] = v iff reverse[v] = k.
INV-22 — Bijection forward size, reverse size, and public size agree.
INV-23 — Failed Bijection association/removal never publishes only one updated root.
INV-24 — Bijection inverse swaps roots without rehashing/copying trie nodes.
INV-25 — Bijection equality is extensional forward-mapping equality and independent of trie shape/history.
INV-26 — Both Bijection K and V obey the canonical key-admission policy because each is a trie key in one direction.
INV-27 — Bag/Bijection value hash is representation-order independent and satisfies equality => same hash.
INV-28 — No unapproved native binding is registered; internal visibility does not bypass TDR-0017.
INV-29 — Existing Map/Set/P6 hash-collection behavior does not regress from shared-helper changes.
INV-30 — Broad unrelated baseline failures are recorded, not repaired or hidden.
```

---

## 10. Non-goals

Do not:

1. introduce a public `PersistentHashMap`;
2. implement Bag by cloning a mutable Map on every update;
3. implement Bijection as two mutable Maps with separately visible updates;
4. hide nodes behind `Arc`, `Rc`, reference counting, or a private arena outside GC accounting;
5. introduce a second `Collection`, `Iterable`, `Iterator`, `Hashable`, `Eq`, `Ord`, or ordering hierarchy;
6. redesign Map/Set semantics for reuse convenience;
7. add Bijection literal syntax;
8. add Bag bracket-literal contextualization;
9. generalize `Range` or implement RangeMap;
10. add persistent sequence/deque/queue/stack families;
11. redesign constructor syntax;
12. introduce a new `Value` public representation arm for Bag/Bijection beyond ordinary `Value::obj` handles;
13. add private nodes to Universe type identity/reflection;
14. add special compiler bytecodes for Bag/Bijection absent measured need;
15. perform P9 `Entry<K,V>` migration;
16. repair P4/P5 residuals opportunistically;
17. run broad workspace/release gates after each task;
18. weaken a key, GC, persistence, or atomicity invariant to make tests pass.

---

## 11. Expected impact map

### 11.1 Expected source areas

| Path | Expected P7 reason |
|---|---|
| `phalcom-core/src/heap/persistent_hash.rs` | new private immutable trie node/storage helpers |
| `phalcom-core/src/heap/bag.rs` | new Bag public-root object |
| `phalcom-core/src/heap/bijection.rs` | new Bijection dual-root object |
| `phalcom-core/src/heap/object.rs` | new public/private object variants |
| `phalcom-core/src/heap/mod.rs` | module exports/allocation/test kind handling |
| `phalcom-core/src/heap/accessors.rs` | typed public/node accessors |
| `phalcom-core/src/heap/trace.rs` | exact Bag/Bijection/node GC edges |
| `phalcom-core/src/primitive/persistent_hash.rs` | callback-safe lookup/update planning and commit |
| `phalcom-core/src/primitive/bag.rs` | approved Bag primitive mechanisms |
| `phalcom-core/src/primitive/bijection.rs` | approved Bijection primitive mechanisms |
| `phalcom-core/src/primitive/mod.rs` | modules, shared key relation extraction/admission helpers if needed |
| `phalcom-core/src/primitive/ordered_hash.rs` | narrow refactor only if P7 needs reusable key relation below mutable storage |
| `phalcom-core/src/value/mod.rs` | class mapping/private-node exhaustive handling; possibly numeric-law owner only after consultation |
| `phalcom-core/src/value/render.rs` | Bag/Bijection debug rendering/private-node exhaustive handling |
| `phalcom-native-meta/src/universe.rs` | Bag/Bijection canonical identities/type forms |
| `phalcom-semantic/src/core_surface/identity.rs` | canonical declaration IDs where current pattern requires |
| `phalcom-core/src/universe/core_classes.rs` | Bag/Bijection runtime class rows |
| `phalcom-core/src/universe/primitives.rs` | approved P7 registrations/validator coverage |
| `phalcom-core/core/universe/src/collections/bag.ph` | public Bag source API/Iterable/equality/hash/algebra |
| `phalcom-core/core/universe/src/collections/bijection.ph` | public Bijection API/Iterable/equality/hash/inverse wrappers |
| `phalcom-core/core/universe/src/collections/package.ph` | expose Bag/Bijection |
| `phalcom-core/core/universe/src/errors/argument.ph` | approved Bijection conflict error |
| `phalcom-modules/src/builtin.rs` | builtin module source registration |
| `phalcom-native-surface/src/generated.rs` | regenerated artifact, not hand-edited |

### 11.2 Expected tests

| Path | Purpose |
|---|---|
| `phalcom-core/src/heap/persistent_hash.rs` unit tests | branch/collision/path-copy structural invariants |
| `phalcom-core/tests/core/collections/persistent_hash.rs` | VM callback/collision/root-sharing behavior |
| `phalcom-core/tests/core/collections/bag.rs` | Bag source/runtime/exact arithmetic/algebra |
| `phalcom-core/tests/core/collections/bijection.rs` | Bijection source/runtime/conflict/inverse invariants |
| `phalcom-core/tests/core/collections/persistent_gc.rs` | forced GC sharing/reclamation/rooting |
| `phalcom-core/tests/core/collections/contract.rs` | only shared Map/Set/P6 regression additions when necessary |
| `phalcom-core/tests/core/object_model/invariants.rs` | approved floor/runtime owner census |
| native-meta tests | generic type forms/identity |

Mechanical test filenames may be consolidated if existing conventions prefer fewer modules.

### 11.3 Expected documentation/state

```text
docs/specs/collection-families.md                    # extend approved Bag/Bijection contract
docs/decisions/proposed/...                          # P7 floor amendment until accepted
docs/decisions/accepted/...                          # accepted P7 floor record, assigned canonical number on acceptance
docs/implementation/COLL006/COLL006.C1/COLL006.C1-CHECKPOINT.md
docs/implementation/COLL006/COLL006.C1/COLL006.C1.P7-walkthrough.md
docs/implementation/COLL006/COLL006.C1/COLL006.C1.P7-handoff.md
```

### 11.4 Unexpected-touch rule

Touch adjacent helpers/tests when mechanically required by exhaustive `Object`/Universe matches.

STOP AND CONSULT before entering materially different subsystems such as semantic inference, compiler bytecode design, static evaluation/materialization, GC policy, new typeclass/capability design, or a new memory arena.

---

## 12. Implementer decision authority

### 12.1 FIXED

The implementer must not change:

- one reusable private persistent hash substrate for both Bag and Bijection;
- GC-visible node allocation through the normal heap;
- immutable published nodes and old-root immutability;
- structural sharing/path-copy instead of whole-map cloning;
- Telos key relation shared with existing hash collections;
- no Heap borrow across callbacks;
- callback/preflight before callback-free build/commit;
- exact Bag Int multiplicities/total;
- Bijection dual-root exact inverse invariant;
- no silent Bijection displacement;
- O(1) root-swapping inverse aside from wrapper allocation;
- existing `Collection`/`Iterator`/`Iterable` reuse;
- no unapproved native floor expansion;
- P9 ownership of final conversion/Entry/release convergence.

### 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- helper/module names;
- private struct field names;
- whether public/root object payloads use small wrappers or direct fields;
- exact 32-way node child container type (`Box<[ObjRef]>`, equivalent immutable compact vector);
- private update-plan structs;
- test helper decomposition;
- whether Bag/Bijection tests share one persistent-hash fixture module;
- source iterator helper class names;
- exact commutative hash-mix constants while preserving the required laws and immediate-Int range;
- equivalent current-repository APIs discovered during execution.

### 12.3 VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| P6 is complete enough for takeover | P6 checkpoint/walkthrough/handoff + `git status` | do not start P7 production edits |
| TDR-0092/spec files exist in live worktree | decision/spec paths from checkpoint | adapt path mechanically if renamed; consult if authority missing |
| P6 exposes reusable key semantics | `primitive/ordered_hash.rs`, P6 handoff | extract narrow helper; consult if semantics must change |
| P6 classifier rejects all new mutable families | `is_mutable_collection_key` + P6 tests | P6-owned completion before P7 integration |
| `Heap::alloc` still never collects synchronously | `heap/mod.rs`, `vm/gc.rs` | STOP; build-phase rooting model changes |
| `Object` start size is stable | focused representation test | use measured baseline; STOP if P7 necessarily grows it |
| generic Bag/Bijection metadata follows Map/Set patterns | native meta/core classes | adapt mechanically |
| internal native return typing can hide raw Dynamic behind typed source wrapper | existing collection `.ph` patterns | consult only if a new typing mechanism is required |
| `Array<Dynamic>` key snapshots are practical | Array allocation/native helper APIs | if not, consult before cursor/ordinal workaround |
| source modular hash arithmetic remains immediate Int | Vector hash/source arithmetic | use same bounded modulus pattern |
| SameValueZero/hash law holds for numeric keys | T1 regression | STOP/CONSULT before changing Number semantics |

---

## 13. Global STOP / CONSULT triggers

The implementer must stop editing and build a consultation packet if any of the following occurs:

1. P6 takeover state is materially incomplete or contradicts the checkpoint.
2. The final P6 key engine cannot expose/reuse canonical hash/equality semantics without changing Map/Set behavior.
3. SameValueZero-equal numeric keys demonstrably produce different canonical hashes and correction requires changing `Number#hash` semantics.
4. A persistent operation requires a live Heap borrow across a Telos callback.
5. A fresh unpublished node/root must survive a callback and the planned preflight/build split cannot avoid it.
6. Persistent nodes cannot be represented as ordinary GC-visible Objects without a new GC ownership architecture.
7. Correct representation would increase `size_of::<Object>()` beyond the P7-start baseline and boxing/equivalent indirection cannot avoid it.
8. Path-copy update cannot meet depth-bounded allocation without changing public semantics.
9. Collision correctness requires a different key relation than Map/Set/P6.
10. Bag exact counts cannot use existing Telos Int/BigInt normalization and numeric policy.
11. Bijection strict conflict behavior/public error channel is not ratified when public implementation is reached.
12. A native selector outside the accepted P7 floor becomes necessary.
13. Iteration can only be implemented by O(n²) ordinal trie scans or an unapproved native cursor architecture.
14. Value hashing would require an unapproved new generic hash-mixing runtime facility.
15. Public API/spec, checkpoint, TDR, and test oracle materially disagree.
16. Tests can pass only by weakening old-root immutability, collision, GC, atomicity, or hash/equality invariants.
17. The same nontrivial semantic failure persists after one serious corrective attempt.
18. An unexpected compiler/semantic/static-evaluation subsystem becomes necessary.
19. P7 starts accumulating P8/P9 conversion or Entry migration work.
20. A hidden Arc/Rc/arena workaround starts to emerge.

A triggered consultation cannot be self-waived.

---

## 14. Debugging budget

### 14.1 Mechanical failures

Allow up to three coherent correction cycles while evidence shows progress.

A cycle is:

```text
inspect exact compile/test failure
identify concrete cause
make one coherent mechanical correction
rerun the smallest discriminating command
```

Examples: missing exhaustive match arm, import/module registration, private signature drift, borrow checker cleanup after correct callback boundaries.

### 14.2 Semantic failures

Before changing code, record in the checkpoint/task notes:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious corrective attempt for the same underlying semantic failure.

If it persists: STOP AND CONSULT.

### 14.3 Architectural failures

Zero speculative architecture-fix attempts.

Examples requiring immediate consultation:

```text
GC node arena proposal
new stable identity model
new callback-rooting model
new hash relation
public API reinterpretation
floor expansion beyond approved inventory
whole-trie copy justified as “temporary” production design
```

### 14.4 Test rerun discipline

Never rerun an unchanged failing test unless something relevant to that failure changed.

Do not rerun `cargo test --workspace`, Clippy, or release suites during BUILD merely to “see where things stand.”

---

## 15. Testing surface analysis

| Coverage ID | Dimension | Invariant(s) | Required case |
|---|---|---|---|
| CV-01 | empty trie | INV-04/09 | lookup/remove on empty root |
| CV-02 | one leaf | INV-04/05 | insert one pair, old root empty |
| CV-03 | first-level branch | INV-06/09 | two hashes diverge at first fragment |
| CV-04 | deep branch | INV-06/07 | hashes diverge after several equal fragments |
| CV-05 | full-hash collision | INV-08/10 | distinct custom keys, same hash |
| CV-06 | collision replace | INV-05/08/10 | equal logical key replaces payload without duplicate |
| CV-07 | collision remove/collapse | INV-10 | 2-entry collision -> Leaf |
| CV-08 | branch remove | INV-05/09 | remove one child, old branch unchanged |
| CV-09 | negative hash bits | INV-08/09 | custom negative i64 hash routes correctly |
| CV-10 | max-depth fragments | INV-09 | differing high hash bits reach final level |
| CV-11 | structural sharing | INV-05/06/07 | unaffected child ObjRefs identical across versions |
| CV-12 | allocation shape | INV-07/11 | update allocation bounded by depth, not n |
| CV-13 | old-root immutability | INV-05 | multi-version lookup after later updates/removes |
| CV-14 | callback hash error | INV-02/03/13 | throwing hash leaves old root valid |
| CV-15 | callback equality error | INV-02/03/13 | throwing == leaves old root valid |
| CV-16 | reentrant read | INV-02/03 | hash/== callback can read immutable collection safely |
| CV-17 | forced GC in callback | INV-03/11 | callback invokes GC; receiver/query/candidate survive |
| CV-18 | shared-node GC | INV-11 | old/new roots share nodes; drop one version, shared survives |
| CV-19 | unique-node reclamation | INV-11 | drop old version, unique path becomes collectible |
| CV-20 | complete reclamation | INV-11 | drop all roots; persistent nodes collected |
| CV-21 | object slot size | INV-12 | P7 start/end `size_of::<Object>()` unchanged |
| CV-22 | numeric key law | INV-01/27/29 | SameValueZero-equal numbers have equal hash |
| CV-23 | mutable-key rejection Bag | INV-01/26 | known mutable collection cannot become Bag key |
| CV-24 | mutable-key rejection Bijection K | INV-26 | mutable K rejected |
| CV-25 | mutable-key rejection Bijection V | INV-26 | mutable V rejected because reverse key |
| CV-26 | Bag exact counts | INV-15/16/17 | multiplicity/total above i64 where policy permits |
| CV-27 | Bag zero/negative | INV-15 | zero no-op, negative public count rejected |
| CV-28 | Bag saturation | INV-15/17/18 | remove more than count deletes key and exact total delta |
| CV-29 | Bag distinct count | INV-18 | first add/delete-last occurrence changes distinct only once |
| CV-30 | Bag versions | INV-05/17 | old Bag counts unchanged after new versions |
| CV-31 | Bag equality order | INV-19 | different insertion/update histories equal |
| CV-32 | Bag algebra | INV-17/19 | addition/union/intersection/difference model laws |
| CV-33 | Bag iterator | INV-20 | repeated occurrences lazy; large count does not preallocate occurrences |
| CV-34 | Bag hash law | INV-27 | equal Bags via different histories have equal hash |
| CV-35 | Bijection insertion | INV-21/22 | new k/v appears both directions |
| CV-36 | exact association idempotence | INV-21/23 | k->v again succeeds without structural corruption |
| CV-37 | key conflict | INV-21/23 | k->v2 rejected, old roots unchanged |
| CV-38 | value conflict | INV-21/23/26 | k2->v rejected, old roots unchanged |
| CV-39 | remove by key | INV-21/22/23 | association disappears from both roots |
| CV-40 | remove by value | INV-21/22/23 | source-derived inverse/removal path exact |
| CV-41 | inverse O(1) | INV-24 | wrapper roots swap; trie node handles identical |
| CV-42 | inverse involution | INV-21/24 | `b.inverse.inverse == b` |
| CV-43 | Bijection equality | INV-25 | histories/trie shapes do not affect equality |
| CV-44 | Bijection hash law | INV-27 | equal bijections hash equally |
| CV-45 | Bijection GC dual roots | INV-11/21 | both forward/reverse graphs traced |
| CV-46 | failure recovery | INV-13/23 | operation after callback/conflict failure still succeeds |
| CV-47 | Map/Set regression | INV-01/29 | existing collision/reentry/numeric keys unchanged |
| CV-48 | P6 family regression | INV-01/29 | final P6 collection focused suite remains green |
| CV-49 | native metadata | INV-28 | Bag/Bijection generic forms and registered selectors exact |
| CV-50 | native floor | INV-28 | generated surface delta equals approved P7 inventory only |

Do not add a test solely to observe private helper names. Tests should establish semantic or architectural invariants.

---

## 16. Verification execution budget

### 16.1 Modes

#### BUILD MODE

Sparse, discriminating feedback only. Compile/test the smallest directly affected unit.

#### STABILIZE MODE

Run P7 focused runtime/source/GC/native-surface suites plus predecessor hash-family regressions.

#### CERTIFY MODE

P7 does **not** automatically enter release certification. P9 owns broad workspace/release evidence. A targeted `cargo check --workspace --all-targets` may be run once at P7 closeout if cross-crate Universe/native metadata changes warrant it, but it is compile evidence rather than behavioral certification.

### 16.2 Verification ladder

```text
T0 — exact reproducer/new regression
T1 — directly affected feature tests
T2 — owning subsystem/module suite
T3 — adjacent cross-layer integration
T4 — broad crate/package verification
T5 — workspace/release verification
```

### 16.3 Mandatory during BUILD

- New persistent-hash heap unit filter after T2/T3 structural edits.
- Exact numeric hash-law discriminator after T1 test creation.
- Exact Bag focused filter during T5.
- Exact Bijection focused filter during T6.
- Native-surface generator only after native declarations change coherently, not after each selector edit.

### 16.4 Mandatory at named gates

- G0: predecessor/baseline state.
- G1: shared key-law/regression gate.
- G2: node GC/slot-size gate.
- G3: persistent trie semantic/property gate.
- G4: approved API/floor decision gate (documentation/authority, not broad tests).
- G5: Bag focused acceptance.
- G6: Bijection focused acceptance.
- G7: cross-family/native-surface/GC stabilization.
- G8: lifecycle closeout and final focused matrix.

### 16.5 Mandatory during STABILIZE

Representative commands; confirm exact filters with `-- --list` first:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib persistent_hash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core persistent_hash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_bag
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_bijection
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_collections
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core memory_gc
RUSTFLAGS='' RUSTC_WRAPPER='' cargo run -p phalcom-native-surface-gen -- --root . --check
```

Use actual discovered test names, not fictional filters, in execution records.

### 16.6 Only if evidence demands

- `cargo test -p phalcom-native-meta <bag/bijection filter>` if canonical metadata tests are not exercised by core identity tests.
- focused semantic reflection tests if generic identity/reification fails.
- `cargo check -p phalcom-core --all-targets` for cross-module compile failures.
- one `cargo check --workspace --all-targets` at closeout if new Universe/native metadata touches downstream crates.

### 16.7 Explicitly deferred / DO NOT RUN during BUILD

```text
cargo test --workspace
cargo clippy --workspace --all-targets
release packaging/certification
broad language-corpus unrelated filters
P4 static_constants aggregate
P5 modules_parse_package full residual certification
```

Reason: P9 owns broad C1 certification and existing baseline residual classification. P7 should not burn implementation time rediscovering D1/D2/D4 or the associated-family parser blocker.

---

## 17. Baseline / unrelated failure policy

Classify failures:

```text
A — definitely caused by P7
B — probably caused by P7
C — unclear
D — clearly unrelated/baseline
```

Rules:

- A/B: active responsibility.
- C: one bounded classification pass; if nonblocking and still unclear, record in checkpoint and continue.
- D: record and continue immediately.
- Never weaken/skip an assertion to turn a failing broad gate green.
- Do not use adviser time on a D-class issue unless it blocks a P7 acceptance gate.

Inherited known residuals include at least:

```text
P4 D1 metadata residual
P4 D2 reflection/module residual
P4 D4 computed Record-key static_constant residual
P5 R-PARSE-ASSOCIATED-INVOKE
```

At T0 import the final P6 residual ledger rather than assuming the remote snapshot's state is current.

---

# 18. Tasks

## T0 — Reconcile P6 takeover and establish the P7 baseline

### Purpose

Start P7 from the actual live worktree, not from the older remote branch snapshot or stale program/status summary.

### Preconditions

- User has authorized P7 execution.
- P6 has completed or has an explicit handoff declaring a stable substrate suitable for P7.

### Consumes

- C1 checkpoint.
- P6 plan/walkthrough/handoff.
- Final P6 accepted spec and TDR-0092.
- Live `git` state.

### Produces

- Exact starting branch/SHA/tree status.
- Final P6 takeover map.
- Baseline native-surface count.
- Baseline `Value`/`Object` sizes.
- Baseline focused hash-family test results.
- Explicit inherited residual list.

### Files and symbols

**Read:**

```text
COLL006.C1-CHECKPOINT.md
COLL006.C1.P6-detailed-implementation-plan.md
COLL006.C1.P6-walkthrough.md
COLL006.C1.P6-handoff.md
docs/specs/collection-families.md
TDR-0092
primitive/ordered_hash.rs
primitive/mod.rs
heap/object.rs
heap/trace.rs
universe/primitives.rs
native-surface generated artifact
```

**Modify:** checkpoint only if plan activation is authorized and predecessor state is complete.

**Tests:** no new test files.

### Required implementation shape

Run/record:

```sh
git status --short
git rev-parse HEAD
git diff --check
```

Discover the focused core collection tests before running them:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_collections -- --list
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_collections
```

Run the live native-surface check once to establish the baseline count:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo run -p phalcom-native-surface-gen -- --root . --check
```

Record P7-start `size_of::<Value>()` and `size_of::<Object>()` through an existing or narrowly added test helper if P6 does not already expose the measurement.

### Forbidden approaches

- Do not reset/clean/stash unrelated work.
- Do not rebuild P6 implementation from remote `ffa2ab` if the local worktree is newer.
- Do not start P7 with P6 native surface/generator in a partial dirty state unless P6 handoff explicitly says that state is stable and owned.
- Do not fix inherited failures.

### Test changes required

None.

### Tests to run now

Only the baseline commands above.

### Tests explicitly deferred

All P7 runtime/property tests; no P7 code exists yet.

### Acceptance

```text
P6 takeover is coherent and documented.
P7 start revision/worktree state is recorded.
Focused predecessor collection suite result is known.
Native-surface baseline count is known.
Object/Value size baseline is known.
No P7 production edit has begun from an ambiguous predecessor state.
```

### Local STOP / CONSULT triggers

- P6 remains materially incomplete.
- P6 handoff conflicts with live source.
- TDR-0092/spec authority referenced by checkpoint cannot be found in live worktree.
- Native surface is already inconsistent at the predecessor boundary.

### Checkpoint update

Mark P7 `IN_PROGRESS / NOT_STARTED-or-PARTIAL / UNVERIFIED` only when execution actually begins. Record exact starting revision and predecessor evidence.

---

## T1 — Prove/reuse the canonical key law before persistent integration

### Purpose

Ensure P7 will not duplicate or silently inherit a broken key relation, especially numeric SameValueZero/hash coherence.

### Preconditions

- G0 takeover pass.

### Consumes

- Final P6 key helper boundary.
- `send_hash`, `send_eq`, SameValueZero.
- existing Number hash semantics.

### Produces

- A narrow reusable key-semantic helper usable by mutable and persistent storage, or proof that the final P6 helper already provides it.
- Focused numeric hash-law regression.
- Map/Set/P6 compatibility evidence after any helper refactor.

### Files and symbols

**Read:**

```text
phalcom-core/src/primitive/ordered_hash.rs
phalcom-core/src/primitive/mod.rs
phalcom-core/src/primitive/number.rs
phalcom-core/src/value/mod.rs
final P6 primitive modules
```

**Modify only if needed:**

```text
primitive/ordered_hash.rs
primitive/mod.rs
focused collection test module
```

Do **not** modify `Number#hash` under this task without an architecture ruling if the new regression fails.

### Required implementation shape

Prefer a small shared layer such as:

```rust
pub(crate) fn hash_key(vm: &mut VM, value: Value) -> PhResult<i64>;
pub(crate) fn key_eq(vm: &mut VM, candidate: Value, query: Value) -> PhResult<bool>;
```

The mutable P6 engine may wrap these with receiver-specific re-entry locking; P7 calls the relation without mutable receiver locks.

Add a focused regression establishing:

```text
SameValueZero(a,b) => hash(a) == hash(b)
```

for at least:

```text
1 and 1.0
0 and -0.0
integral Float at/above 2^53 when exactly representable
heap-backed LargeInt and equal integral Float where representable
multiple NaN payloads
```

The test should compare actual Telos `hash` results and actual key behavior, not private Rust helper internals only.

### Forbidden approaches

- No P7-specific numeric hash normalization.
- No Rust `Hash`/identity substitute for Telos keys.
- No changed Map/Set equality semantics.
- No “skip NaN/large Float” test weakening if the law fails.

### Test changes required

- CV-22 numeric key law.
- CV-47/CV-48 predecessor hash-family regression if helper extraction changes code.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core <numeric-key-law-test> -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_collections
```

### Tests explicitly deferred

Persistent trie/Bag/Bijection tests.

### Acceptance

Either:

1. numeric/hash law passes and one canonical key helper is ready for P7; or
2. an explicit consultation blocks public P7 hash integration while T2 private GC representation work may proceed only if the consultation permits.

### Local STOP / CONSULT triggers

The numeric law regression fails and fixing it would alter `Number#hash` or cross-family key semantics.

### Checkpoint update

Record `R-P7-NUMERIC-HASH-LAW` only if reproduced. Do not bury it in a test comment.

---

## T2 — Add GC-visible immutable persistent-node storage

### Purpose

Establish the node lifetime/accounting model before implementing persistent lookup/update.

### Preconditions

- G0 pass.
- T1 may be pending consultation only if no public hash semantics are being integrated yet.

### Consumes

- Heap allocation model.
- exhaustive object tracing.
- P7-start object-size baseline.

### Produces

- `PersistentHashNode` private object representation.
- heap allocation/accessor APIs.
- exact trace edges.
- unit-level structural invariants.
- no public Bag/Bijection API yet.

### Files and symbols

**Create (preferred):**

```text
phalcom-core/src/heap/persistent_hash.rs
```

**Modify:**

```text
phalcom-core/src/heap/object.rs
phalcom-core/src/heap/mod.rs
phalcom-core/src/heap/accessors.rs
phalcom-core/src/heap/trace.rs
phalcom-core/src/value/mod.rs          # exhaustive private-node handling only
phalcom-core/src/value/render.rs       # exhaustive private-node handling only
```

**Tests:** heap module unit tests + focused GC integration test registration.

### Required implementation shape

Implement immutable constructors/accessors for:

```text
Leaf
Collision
Branch
```

Do not expose mutable node accessors unless construction itself mechanically requires a private pre-publication builder. Prefer fully constructed immutable node values passed to `Heap::alloc`.

Trace:

```text
Leaf        -> key Value, payload Value
Collision   -> every key Value, every payload Value
Branch      -> every child ObjRef
```

Do not give private nodes a Universe class. If exhaustive `Value::get_class` needs a result for impossible accidental exposure, follow the live precedent for internal-only objects; do not create a public classifier.

Add representation-size assertion relative to T0 baseline.

### Forbidden approaches

- No `Arc<Node>`/`Rc<Node>`.
- No untracked arena.
- No recursive Rust `Drop`/tracing graph.
- No inline fat node variant that enlarges `Object` slots.
- No public Node class.

### Test changes required

- CV-01/02 basic node reachability.
- CV-18/19/20 GC shared/reclamation skeleton.
- CV-21 object slot size.

At this task, GC tests may construct nodes directly from Rust test scaffolding before public collections exist.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib heap::persistent_hash::tests
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core <persistent-node-gc-test> -- --nocapture
```

### Tests explicitly deferred

Source-language Bag/Bijection, native surface, algebra/conflict tests.

### Acceptance

```text
private nodes allocate through Heap
exact tracing is implemented
shared nodes survive when one root remains
unreachable nodes are reclaimable
Object size does not increase from T0 baseline
no public identity/surface exists for nodes
```

### Local STOP / CONSULT triggers

- tracing requires a new root category or collector change;
- normal heap allocation cannot account for nodes;
- object-size ceiling cannot be preserved by boxing/equivalent indirection.

### Checkpoint update

Record the durable GC ownership model and G2 result; this is a checkpoint-worthy invariant.

---

## T3 — Implement persistent hash lookup/path-copy/remove

### Purpose

Build and prove the reusable persistent hash engine before either public collection depends on it.

### Preconditions

- T2/G2 node lifetime model passes.
- canonical key helper from T1 is available for callback-integrated tests, or nonnumeric relation work is explicitly allowed while a numeric consultation remains open.

### Consumes

- `PersistentHashNode` allocation/tracing.
- canonical key helper.
- VM temp-root facility.

### Produces

- lookup.
- insertion/replacement planning.
- removal planning.
- collision handling.
- path-copy commit.
- deterministic traversal/key snapshot helper.
- structural-sharing and model tests.

### Files and symbols

**Create (preferred):**

```text
phalcom-core/src/primitive/persistent_hash.rs
```

**Modify:**

```text
phalcom-core/src/primitive/mod.rs
heap/persistent_hash.rs
focused tests
```

### Required implementation shape

Use two layers:

```text
heap/persistent_hash.rs
    node representation
    pure bitmap/index helpers
    immutable node inspection

primitive/persistent_hash.rs
    VM callback/rooting orchestration
    lookup preparation
    update/remove plans
    callback-free commit allocation
    traversal snapshot helpers
```

Public collection semantics must not leak into the generic engine.

The engine should traffic in generic `(key,payload)` associations.

#### Callback rooting discipline

At the start of any operation that sends Telos code, record temp-root depth and protect every Rust-local object that is not otherwise provably rooted for the entire callback interval.

Use one cleanup path (or an RAII-equivalent helper if already available) that always restores the original depth on `Ok` and all `Err` paths.

Never retain `&Heap`, `&Object`, `&PersistentHashNode`, or borrowed child/entry slices across the send.

#### Build phase

Once the final callback returns:

```text
no send_dynamic
no block call
no user equality/hash
only immutable old-node reads + allocation/copy
```

### Forbidden approaches

- Full trie-to-Vec clone and rebuild per update.
- Mutating old nodes “temporarily then restoring”.
- Using MapObject as persistent storage.
- Rehashing all existing entries on every update.
- Recursive node ownership outside the GC.
- Aggressive unsafe singleton-branch compression.

### Test changes required

CV-01 through CV-20, excluding public Bag/Bijection-specific cases.

Add deterministic randomized/model tests. A simple Rust `HashMap`/Vec reference model is acceptable **only in tests**; production must remain persistent trie storage.

Collision keys should be real Telos objects whose `hash` intentionally collides and whose `==` distinguishes logical identity.

### Tests to run now

During BUILD:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --lib persistent_hash
```

At G3:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core persistent_hash -- --nocapture
```

If GC-stress fixtures exist for the new path:

```sh
PHALCOM_GC_STRESS=1 RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core <persistent-hash-reentry-gc-test> -- --nocapture
```

### Tests explicitly deferred

Bag algebra/Bijection conflict/native surface.

### Acceptance

```text
lookup/update/remove agree with reference model
collisions correct
old roots unchanged
unaffected subtree ObjRefs shared
allocation bounded by path depth/collision size
callback errors publish nothing
forced GC during callbacks does not stale live values
```

### Local STOP / CONSULT triggers

- callback/build phase cannot be separated;
- structural update requires whole-collection copy;
- temp-root requirements imply a new VM rooting API rather than local use of existing one;
- persistent key relation diverges from P6.

### Checkpoint update

Record G3 and the final internal substrate interface used by T5/T6.

---

## T4 — Ratify P7 public API and native primitive floor

### Purpose

Resolve every source-visible and floor-governance decision before Luna publishes Bag/Bijection APIs or registers new native bindings.

### Preconditions

- Persistent substrate representation and irreducible operations are understood from T2/T3.
- No public P7 API has been committed as de facto authority.

### Consumes

- C1 Bag/Bijection semantic model.
- `docs/specs/collection-families.md` current P6 contract.
- TDR-0017/0028/0033/0063/0092 precedents.
- actual T3 mechanism needs.

### Produces

- `COLL006.C1.P7-api-and-floor-consultation.md`.
- user/architect ruling.
- updated owning `docs/specs/` contract.
- accepted scoped P7 TDR floor amendment.
- exact selector/error/result inventory for T5/T6.

### Proposed public contract for consultation

#### Bag

Recommended semantics:

```text
Bag<T>.new()                  -> empty Bag
size                          -> total occurrence count
distinctCount                 -> unique logical key count
count(value)                  -> exact Int, zero if absent
contains(value)               -> count(value) > 0
adding(value)                 -> new Bag with +1
adding(value, count: n)       -> new Bag with +n, n >= 0
removing(value)               -> saturating -1
removing(value, count: n)     -> saturating -n, n >= 0
+                             -> additive multiset combination
union                         -> max multiplicities
intersection                  -> min multiplicities
difference                    -> truncated subtraction
== / !=                       -> multiplicity equality
hash                          -> representation-order-independent value hash
iterator                      -> occurrences lazily
```

Recommended invalid count behavior: negative `n` raises existing `ArgumentError`; zero is identity/no-op.

#### Bijection

Recommended semantics:

```text
Bijection<K,V>.new()          -> empty
size                          -> association count
get(key)                      -> Option<V>
[_ key]                       -> V or KeyError, if strict subscript is desired consistently with Map
keyFor(value)                 -> Option<K>
containsKey(key)              -> Bool
containsValue(value)          -> Bool
associating(key, value)       -> strict persistent association
removingKey(key)              -> new Bijection
removingValue(value)          -> new Bijection
inverse                       -> Bijection<V,K>, O(1) root swap
iterator                      -> labeled (key: K, value: V) tuples
== / !=                       -> extensional mapping equality
hash                          -> order-independent directional-pair hash
```

Conflict recommendation:

```text
exact existing k -> v       idempotent success
existing k -> otherV        BijectionConflictError
existing otherK -> v        BijectionConflictError
no silent displacement
```

The consultation must choose the error channel: thrown `BijectionConflictError` versus `Result<...,BijectionConflictError>` or another existing convention. Do not infer it.

### Proposed minimal native-floor candidate

This is a recommendation to audit, **not authorization**.

#### Bag candidate — 7 bindings

```text
new()
rawSize                  # total count
rawDistinctCount
rawCount(_)
rawAdding(_,_)
rawRemoving(_,_)
rawKeys                  # callback-free distinct-key snapshot
```

#### Bijection candidate — 7 bindings

```text
new()
rawSize
rawGet(_)
rawAssociate(_,_)
rawRemoveKey(_)
rawInverse
rawKeys                  # callback-free forward-key snapshot
```

Total candidate: **14 bindings**.

Source derivations should cover `contains*`, reverse lookup via inverse, remove-by-value via inverse/remove/inverse, equality, value hash, and iteration.

The consultation must explicitly audit whether `rawKeys` is the right callback-free bulk mechanism. If the actual typed source layer cannot use it coherently, provide a narrower/more appropriate approved mechanism rather than adding repeated ordinal scans.

### Files and symbols

**Create/modify:** consultation, owning spec, proposed/accepted decision, checkpoint/plan amendment.

No production native registration until accepted.

### Required implementation shape

The accepted ruling must include an exact selector table, not merely “Bag/Bijection storage primitives are allowed.”

The native-surface expected count is:

```text
verified P7-start generated count + exact approved P7 bindings
```

Do not use 429/458 as a hard-coded P7 baseline; those are historical P5/P6 planning numbers and the live P6 finish count must be measured at T0.

### Forbidden approaches

- “internal” visibility as a floor exemption.
- Implementer-chosen conflict result type.
- Native block-taking combinators.
- Native Bag algebra purely for convenience.
- Native equality purely for convenience.
- New error class per convenience operation.

### Test changes required

None until the ruling is accepted.

### Tests to run now

None; this is an architecture/authority gate.

### Acceptance

```text
observable Bag/Bijection API is explicit
conflict/count/error behavior is explicit
iteration item shape is explicit
exact native selector inventory is accepted
owning docs/specs contract is updated
accepted TDR amendment exists
```

### Local STOP / CONSULT triggers

Any required selector/error/constructor/hash behavior remains ambiguous.

### Checkpoint update

Record the ruling/amendment and exact approved floor before T5/T6 registrations.

---

## T5 — Implement `Bag<T>` over the persistent substrate

### Purpose

Implement the approved immutable multiset with exact counts, structural sharing, source-level algebra/equality/hash, and lazy iteration.

### Preconditions

- G3 persistent substrate pass.
- G4 API/floor approval.
- Numeric hash-law blocker resolved or explicitly dispositioned in a way that permits P7 value hashing.

### Consumes

- persistent map engine.
- final key admission helper.
- approved Bag API/floor.
- exact Int helpers (`expect_int_big`/normalization or current equivalents).

### Produces

- canonical Bag Universe class/type identity.
- `BagObject` heap root.
- approved native mechanisms.
- typed `.ph` public API.
- source iterator/algebra/equality/hash.
- focused exact-count/model/GC tests.

### Files and symbols

**Create (preferred):**

```text
phalcom-core/src/heap/bag.rs
phalcom-core/src/primitive/bag.rs
phalcom-core/core/universe/src/collections/bag.ph
phalcom-core/tests/core/collections/bag.rs
```

**Modify as required:**

```text
heap/object.rs
heap/mod.rs
heap/accessors.rs
heap/trace.rs
value/mod.rs
value/render.rs
phalcom-native-meta/src/universe.rs
phalcom-semantic/src/core_surface/identity.rs
universe/core_classes.rs
universe/primitives.rs
collections/package.ph
phalcom-modules/src/builtin.rs
primitive/mod.rs
native-surface generated artifact via generator
```

### Required implementation shape

`BagObject` stores root + exact total + distinct count.

Native add/remove performs exactly one canonical lookup planning phase for the target key, exact count arithmetic, then callback-free path-copy commit and wrapper allocation.

Do not implement `adding` as:

```text
count(key) callback
then second independent insert callback
```

because repeated user hash/equality callbacks can have effects and create inconsistent operation boundaries. One native mechanism should own the atomic logical update if the approved floor follows the candidate design.

Source `contains` derives from count. Source algebra loops over distinct keys using persistent updates.

Bag iterator snapshots distinct keys once and lazily emits repetitions; it does not build a total-occurrence Array/List.

### Forbidden approaches

- counts stored as i64/usize semantic values;
- zero-count trie entries;
- mutable in-place Bag root update;
- full trie copy per occurrence change;
- iteration materializing total occurrences;
- order-sensitive equality/hash.

### Test changes required

CV-23, CV-26 through CV-34, plus relevant CV-14..20 GC/reentry cases through Bag's public path.

Reference model should use a simple test-only logical map from canonical test keys to BigInt/nonnegative count.

### Tests to run now

BUILD:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core <single-bag-discriminator> -- --nocapture
```

G5:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_bag -- --nocapture
```

Use the actual registered filter name.

### Tests explicitly deferred

Bijection and full cross-family/native-surface closeout until T6/T7.

### Acceptance

```text
all stored counts positive exact Int
totalCount invariant exact
distinctCount invariant exact
old Bags unchanged
unaffected nodes shared
algebra matches reference model
equality/hash laws pass
iterator is lazy in total multiplicity
GC/reentry tests pass
```

### Local STOP / CONSULT triggers

- source API cannot represent approved exact count/result types;
- native mechanism needs an extra unapproved selector;
- BigInt arithmetic needs new numeric semantics rather than existing helpers.

### Checkpoint update

Record G5 and any finalized Bag-specific runtime interface that Bijection/P9 should inherit.

---

## T6 — Implement `Bijection<K,V>` with atomic dual roots

### Purpose

Implement strict persistent one-to-one mapping using two roots over the same persistent substrate, with O(1) inverse and no half-published state.

### Preconditions

- G3 substrate pass.
- G4 public/floor approval.
- final key admission policy available.

### Consumes

- generic persistent map engine.
- approved conflict channel/error.
- approved native floor.

### Produces

- canonical Bijection Universe class/type identity.
- `BijectionObject` dual-root heap representation.
- strict association/removal/inverse primitives.
- source lookup/contains/remove-value/equality/hash/iterator APIs.
- model/property/GC/conflict tests.

### Files and symbols

**Create (preferred):**

```text
phalcom-core/src/heap/bijection.rs
phalcom-core/src/primitive/bijection.rs
phalcom-core/core/universe/src/collections/bijection.ph
phalcom-core/tests/core/collections/bijection.rs
```

**Modify:** shared identity/bootstrap/registration/source package/error files established in T5.

### Required implementation shape

#### Strict association

All hash/equality callbacks for both directions occur before any new root is allocated.

Pseudo-flow:

```text
root receiver + k + v for callbacks
prepare forward lookup/update plan for k -> v
prepare reverse lookup/update plan for v -> k
validate exact/idempotent/conflict cases
restore callback temp roots only after no further send is needed

if conflict: return approved error; allocate/publish no public result
if exact pair: return self or equivalent approved identity result

commit forward path
commit reverse path
allocate one Bijection wrapper with both
```

If a plan structure retains ObjRefs to old nodes after temp roots are released, ensure the receiver remains VM-rooted or keep the receiver temp root until the final wrapper exists. Favor the simplest provable lifetime.

#### Removal

Prepare both root removals before commit. The reverse association found from the forward payload must agree with the key; disagreement is an internal invariant failure, not a recoverable user conflict.

#### Inverse

Swap roots and preserve `len`. Test exact node-handle reuse.

### Forbidden approaches

- two mutable Maps;
- updating/publishing forward root before reverse success;
- silent displacement;
- rebuilding reverse root by iterating forward entries on each update;
- inverse by copying/rehashing entries;
- treating V as an ordinary non-key value for key-admission purposes.

### Test changes required

CV-24/25, CV-35 through CV-46, plus GC stress over dual roots.

Randomized model:

```text
reference forward map
reference reverse map
sequence of associate/removeKey/removeValue/inverse/lookups
verify invariant after every successful operation
verify original version after every failed operation
```

### Tests to run now

BUILD:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core <single-bijection-discriminator> -- --nocapture
```

G6:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_bijection -- --nocapture
```

### Tests explicitly deferred

Full P7 cross-family/native surface to T7.

### Acceptance

```text
forward/reverse exact inverse invariant holds after every success
strict conflicts match approved contract
failed updates preserve old roots
inverse swaps existing roots without node copy/rehash
K and V mutable-key restrictions both enforced
source equality/hash/iterator contracts pass
```

### Local STOP / CONSULT triggers

- exact conflict channel cannot be implemented without changing approved API;
- dual-root preflight cannot avoid callback after first root allocation;
- inverse requires new generic runtime typing architecture.

### Checkpoint update

Record G6 and exact Bijection invariant evidence.

---

## T7 — Cross-family integration, native surface, GC, and hash-law stabilization

### Purpose

Prove P7 integrates with P6/Map/Set/native metadata/GC as one coherent hash family without broad P9 certification.

### Preconditions

- G5/G6 focused public families pass.

### Consumes

- final Bag/Bijection code.
- approved floor.
- final P6 tests.

### Produces

- exact native-surface regenerated catalog.
- runtime registration/floor audit updates.
- cross-family key-admission tests.
- forced-GC persistent sharing tests through public source/runtime paths.
- predecessor regression evidence.

### Files and symbols

**Modify as needed:**

```text
universe/primitives.rs native validator owner list
native-surface generated artifact
object_model/invariants.rs floor audit
primitive/mod.rs mutable-key policy tests
core/mod.rs test registration
```

Do not broaden unrelated legacy floor-census repair unless P7 selectors cannot be audited accurately without it. Record wider omissions separately.

### Required implementation shape

Verify:

```text
Bag/Bijection classes and generic forms reflect correctly
all and only approved P7 bindings are generated/installed
private node has no public native surface
Bag/Bijection are accepted as immutable Map/Set keys only when their constituent hashes are stable by behavioral law
known mutable built-ins/P6 families are rejected when used as Bag key or either Bijection direction key
Map/Set/P6 regressions remain green
persistent nodes survive/reclaim under forced GC
```

When testing Bag/Bijection as keys in Map/Set, use constituent immutable keys with known stable hash laws. Do not claim static proof for arbitrary user-defined mutable internals.

### Forbidden approaches

- adding Bag/Bijection to mutable-key rejection merely to avoid implementing value hash;
- hiding native-surface mismatch by hand-editing generated file;
- claiming legacy partial floor count is global census;
- fixing unrelated floor-audit omissions as a prerequisite unless they prevent exact P7 accounting.

### Test changes required

CV-47 through CV-50 and final public-path CV-17..21.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_collections
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_bag
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core core_bijection
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core persistent_hash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core memory_gc
RUSTFLAGS='' RUSTC_WRAPPER='' cargo run -p phalcom-native-surface-gen -- --root . --check
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core floor_census_matches_installed_bindings
```

Use discovered filter names if different.

### Tests explicitly deferred

Workspace tests, Clippy, release gates, P4/P5 residual reruns.

### Acceptance

```text
P7 native surface exact and current
runtime registration matches descriptors
public GC/rooting tests pass
Map/Set/P6 focused regressions pass or only preclassified baselines remain
key admission is consistent
numeric hash-law gate is resolved
```

### Local STOP / CONSULT triggers

- generated/installed selector inventory differs from accepted floor for architectural reasons;
- Bag/Bijection value hash violates equality law with valid built-in keys;
- P6 behavior regresses from the shared key helper.

### Checkpoint update

Record G7 exact commands/results and any surviving classified residual.

---

## T8 — Stabilize, document, and close P7

### Purpose

Convert the implemented code into a truthful, resumable plan state with focused evidence and a clean P8/P9 handoff.

### Preconditions

- G5/G6/G7 acceptance met.

### Consumes

- all P7 implementation/test results.

### Produces

```text
COLL006.C1.P7-walkthrough.md
COLL006.C1.P7-handoff.md
updated COLL006.C1-CHECKPOINT.md
updated PROGRAM/STATUS summaries only if repository convention requires synchronization
```

### Files and symbols

Lifecycle/docs only, plus scoped formatting fixes in changed files.

### Required implementation shape

Run scoped formatting/diff hygiene:

```sh
rustfmt --check --edition 2024 <changed Rust files or repository-standard scoped equivalent>
git diff --check
```

If cross-crate metadata changes warrant one compile gate:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace --all-targets
```

Run this at most once at coherent closeout, not repeatedly during BUILD.

Do not run `cargo test --workspace`, full Clippy, or release certification unless a P7-owned issue specifically requires them.

### Forbidden approaches

- Claiming `RELEASE_COMPLETE` from focused evidence.
- Omitting failed commands from walkthrough.
- Marking baseline-blocked aggregate suites as PASS.
- Closing P7 while API/floor authority remains proposed rather than accepted.

### Test changes required

No new behavior tests unless stabilization exposes an uncovered P7 invariant.

### Tests to run now

Final focused gate matrix only; do not mechanically rerun every intermediate discriminator if a stronger same-tree gate already covers it.

### Acceptance

P7 can finish as:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

or, if an independently classified baseline blocks a required aggregate filter:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: BASELINE_BLOCKED
```

Never infer release completion.

### Local STOP / CONSULT triggers

Any unclassified failure still directly threatens a P7 invariant.

### Checkpoint update

Set P7 final ledger state, persist residuals, and set the next action. P8 may proceed independently where its prerequisites are satisfied; P9 remains final C1 convergence/certification owner.

---

# 19. Verification gates

## G0 — P6 takeover gate

**Purpose:** prove P7 starts from a coherent predecessor, not stale remote or partial P6 state.

Run the T0 commands.

Expected:

```text
P6 is complete/stable for takeover
live branch/SHA/dirty state recorded
final P6 focused collection gate known
native-surface baseline known/current
Object/Value size baseline known
```

On failure, classify before editing. Do not broaden.

## G1 — Canonical key-law gate

**Purpose:** prove one shared key relation is reusable and numeric SameValueZero/hash laws are coherent.

Run exact numeric-law test + `core_collections`.

Expected:

```text
SameValueZero-equal tested numeric pairs have equal canonical hash
Map/Set/P6 behavior unchanged after helper reuse/extraction
```

A numeric-law failure is a consultation trigger, not permission for P7-local normalization.

## G2 — Persistent-node GC ownership gate

**Purpose:** prove nodes are ordinary traced heap objects and do not enlarge arena slots.

Expected:

```text
node edges exact
shared child survives
unique unreachable child reclaimed
Object slot size unchanged from P7 start
```

## G3 — Persistent trie correctness gate

**Purpose:** prove the reusable engine before public collection semantics.

Expected:

```text
lookup/update/remove reference-model agreement
collision cases pass
path sharing proven by ObjRef identity
old roots unchanged
callback failures recover
forced GC during callback safe
no O(n) whole-copy update shape
```

## G4 — Public API/native floor decision gate

**Purpose:** ensure Luna does not invent observable semantics or floor authority.

Expected:

```text
Bag API ratified
Bijection API/conflict channel ratified
iteration item/snapshot behavior ratified
exact P7 native bindings accepted in a TDR-0017 amendment
owning docs/specs updated
```

No Bag/Bijection native registration before PASS.

## G5 — Bag coherent behavior gate

**Purpose:** prove exact persistent multiset semantics.

Expected:

```text
positive-count invariant
total/distinct invariants
exact BigInt cases
saturating removal
old-root immutability
algebra/reference-model agreement
order-independent equality/hash
lazy occurrence iteration
GC/reentry safety
```

## G6 — Bijection coherent behavior gate

**Purpose:** prove strict atomic dual-root semantics.

Expected:

```text
forward/reverse exact inverses
key/value conflict behavior correct
failed operation publishes nothing
remove both directions correct
inverse swaps roots with no trie copy
extensional equality/hash laws
GC/reentry safety
```

## G7 — Cross-family/native integration gate

**Purpose:** prove P7 composes with the existing hash family and native/runtime metadata.

Expected:

```text
Map/Set/P6 focused regressions clean except preclassified baselines
key admission consistent
native descriptor/installation inventory exact
floor audit updated
persistent public-path GC tests pass
```

## G8 — P7 closeout gate

**Purpose:** prove plan records match actual implementation/evidence and are safe for takeover.

Expected:

```text
all required CV obligations mapped to tests/results
checkpoint current
walkthrough complete
handoff complete
scoped diff/format hygiene recorded
broad deferred gates explicitly named
```

---

## 20. Final focused acceptance

At completion, the walkthrough must include a table mapping every required coverage obligation to an actual test/result.

Minimum grouping:

| Coverage group | Required evidence |
|---|---|
| CV-01..13 | persistent structural unit/model tests |
| CV-14..17 | callback/error/reentrant/GC tests |
| CV-18..21 | GC sharing/reclamation/slot-size tests |
| CV-22 | numeric key-law regression |
| CV-23,26..34 | Bag focused source/runtime/property tests |
| CV-24..25,35..46 | Bijection focused source/runtime/property tests |
| CV-47..48 | Map/Set/P6 regression tests |
| CV-49..50 | native metadata/surface/floor tests |

Do not leave a coverage row marked PASS without an executed same-tree test.

A stronger gate may subsume an earlier discriminator; document the relationship instead of rerunning needlessly.

---

## 21. Performance/resource evidence

P7 has real asymptotic/allocation obligations; provide structural evidence, not unstable microbenchmark theater.

### 21.1 Required complexity evidence

| Operation | Required architecture |
|---|---|
| Persistent lookup | O(trie depth + collision scan) |
| Persistent insert/update | O(trie depth + collision-local copy) |
| Persistent remove | O(trie depth + collision-local copy) |
| Unaffected subtree | O(1) ObjRef reuse |
| Bijection inverse | O(1) roots + wrapper allocation |
| Full traversal/key snapshot | O(n) |
| Bag occurrence iteration memory | O(distinct snapshot + iterator state), not O(total multiplicity) |

### 21.2 Allocation-shape test

For increasing collection sizes with noncolliding well-distributed test hashes, record heap live/allocation delta for one update while retaining old/new roots.

The delta may vary with differing hash paths, but it must remain bounded by trie depth + public wrapper, not linearly proportional to entry count.

Do not require nanosecond timing thresholds.

### 21.3 GC-pressure proof

Insert enough persistent nodes to cross ordinary heap pressure in a controlled test or use forced GC to prove nodes are counted/reclaimed through the normal heap.

Do not create a special persistent-node memory counter.

### 21.4 Collision worst case

Document that a full-hash collision node has O(c) candidate equality/copy cost for c colliding logical keys. This is an accepted adversarial behavior of the chosen representation, not evidence to replace Telos hashing with Rust identity.

---

## 22. Checkpoint bookkeeping

### 22.1 At plan start

Update `COLL006.C1-CHECKPOINT.md` with:

```text
P7 active
exact starting branch/SHA/worktree state
P6 takeover revision/handoff
baseline native-surface count
baseline Object/Value size
known inherited residuals
```

Do not rewrite old evidence.

### 22.2 During plan

Update only for durable events:

- G1 numeric/hash-law disposition;
- G2 persistent GC ownership model;
- G3 persistent substrate interface;
- G4 public API/floor consultation result;
- G5/G6 coherent family gates;
- new deferred failures or consultations;
- G7 integration/native-surface result.

Do not append a diary entry for every compile error.

### 22.3 At plan completion

Update:

- P7 plans-ledger row;
- status/completion/verification axes;
- final persistent invariants;
- exact native-floor decision and delta;
- executed verification evidence;
- deliberately deferred tests;
- residual ledger;
- next action.

Synchronize `PROGRAM.md`/`STATUS.md` summary only according to repository convention; do not let them compete with the checkpoint.

---

## 23. Walkthrough deliverable

Create:

```text
docs/implementation/COLL006/COLL006.C1/COLL006.C1.P7-walkthrough.md
```

Must contain:

1. final result and lifecycle state;
2. exact start/final revision/worktree status;
3. persistent node architecture actually implemented;
4. key callback/rooting boundary;
5. Bag implementation and approved public semantics;
6. Bijection implementation and approved conflict semantics;
7. native-floor amendment identity and exact binding count;
8. important code paths/symbols;
9. deviations from this plan and why they were mechanical/consulted;
10. consultations and resulting plan amendments;
11. tests added;
12. tests actually run, exact command/filter/count/result;
13. tests deliberately deferred;
14. performance/allocation-shape evidence;
15. verification classification;
16. remaining residual risks.

Do not paste the plan back into the walkthrough. Record what actually happened.

---

## 24. Handoff deliverable

Create:

```text
docs/implementation/COLL006/COLL006.C1/COLL006.C1.P7-handoff.md
```

Must contain:

- current branch/revision/worktree state;
- final stable persistent-hash internal interface;
- Bag/Bijection public contracts;
- exact native-floor authority;
- GC/rooting invariants;
- key-law/numeric disposition;
- cross-family key-admission behavior;
- tests that are reliable takeover gates;
- next C1 objective(s);
- must-read files;
- deferred failures;
- first recommended commands;
- things P8/P9 must not redesign or rediscover.

The handoff should be the smallest complete operational context for the next implementer, not a second walkthrough.

---

## 25. Completion truth table

Do not conflate:

```text
private persistent node code written
persistent unit tests added
persistent unit tests passed
Bag source written
Bag focused gate passed
Bijection source written
Bijection focused gate passed
native floor accepted
native surface regenerated/current
cross-family integration passed
P7 COMPLETE
C1 COMPLETE
release certified
```

Truthful P7 end states include:

### Normal successful P7

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

### P7 implemented but a required aggregate is independently baseline-blocked

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: BASELINE_BLOCKED
```

### Public/floor architecture unresolved

```yaml
status: BLOCKED
completion: PARTIAL
verification: FOCUSED_TESTED  # only if completed private substrate evidence exists
```

Never mark `RELEASE_COMPLETE` unless actual broad release evidence exists.

---

## 26. Plan self-review

### Architecture ownership

- [x] Persistent representation has one private owner.
- [x] Key semantics have one shared authority.
- [x] GC ownership is explicit.
- [x] Bag and Bijection semantic responsibilities are separated from generic trie mechanics.
- [x] Bijection dual-root publication is atomic by construction.
- [x] Public API and native floor are not delegated to implementer invention.

### Invariants and coverage

- [x] Old-root immutability has direct tests.
- [x] Structural sharing has ObjRef-identity tests.
- [x] Collision handling has explicit same-hash distinct-key tests.
- [x] Callback failure/recovery has direct tests.
- [x] Forced-GC callback lifetime has direct tests.
- [x] Node reclamation/allocation pressure has direct tests.
- [x] Exact Bag BigInt multiplicity has coverage.
- [x] Bag algebra has model/property coverage.
- [x] Bijection inverse/conflicts have model/property coverage.
- [x] Numeric SameValueZero/hash coherence has a prerequisite regression.
- [x] Native-floor exactness has a gate.

### Luna execution quality

- [x] Tasks are coherent slices rather than edit-sized microsteps.
- [x] Each task states consumes/produces and local stop conditions.
- [x] Debugging loops are bounded.
- [x] BUILD mode avoids broad testing.
- [x] Broad P9-owned failures remain deferred.
- [x] Mechanical drift is adaptable.
- [x] Architectural drift is an objective consultation trigger.
- [x] No placeholder selectors are treated as approved.
- [x] No hidden full-map copy fallback exists.
- [x] No hidden GC arena/refcount fallback exists.

### Final architect check before execution

Before handing this plan to Luna for implementation, verify:

```text
P6 is actually complete and its handoff exists.
The live collection-families spec contains the accepted P6 contract.
TDR-0092 is present in the live decision registry.
The starting native-surface count is current.
The P6 final key helper/classifier names are updated in this plan if they drifted.
No later user ruling has already settled Bag/Bijection API or P7 floor; if it has, fold that ruling into §5/§7/T4 and mark G4 pre-satisfied.
```

If those conditions hold, the task dependency order is valid:

```text
T0 predecessor truth
  -> T1 key-law/shared semantics
  -> T2 node lifetime
  -> T3 persistent engine
  -> T4 public/floor authority
  -> T5 Bag
  -> T6 Bijection
  -> T7 integration
  -> T8 closeout
```

The only intentionally parallelizable planning activity is T4 consultation preparation while T2/T3 private substrate work proceeds. Production Bag/Bijection surfaces remain blocked until G4 is accepted.

---

# Appendix A — Concrete algorithm notes for Luna

## A.1 Building a branch for two different hashes

Given two Leaf-like associations with different full hashes at `shift`:

```text
fa = fragment(hashA, shift)
fb = fragment(hashB, shift)
```

If `fa != fb`:

```text
bitmap = bit(fa) | bit(fb)
children ordered by fragment position
```

If `fa == fb`:

```text
child = recursively build divergence at shift + 5
parent = Branch(bitmap = bit(fa), children = [child])
```

This recursion is bounded by the fixed hash width. If full hashes are identical, build Collision rather than recursing.

## A.2 Path-copy branch replacement

Given old Branch `(bitmap, children)`, selected `bit`, and replacement child:

```text
idx = compact_index(bitmap, bit)
new_children = children.to_vec()
new_children[idx] = replacement
allocate new Branch(bitmap, new_children)
```

Insertion into absent bit:

```text
idx = compact_index(bitmap, bit)
new_bitmap = bitmap | bit
new_children.insert(idx, new_child)
```

Removal:

```text
idx = compact_index(bitmap, bit)
new_bitmap = bitmap & !bit
new_children.remove(idx)
if new_bitmap == 0 -> absent
else Branch(new_bitmap, new_children)
```

Do not mutate `children` in the old node.

## A.3 Collision equality

For a Collision with matching full hash:

```text
for each copied candidate key:
    if key_eq(candidate, query):
        present
absent
```

Never call `key_eq` while holding a borrowed Collision entries slice from Heap.

A safe implementation can copy the candidate `Value`s (and indices) into a temporary small Vec before sends, or borrow/copy one candidate at a time and end the borrow lexically before the send. Prefer minimal allocation if borrow scopes remain obvious and tested.

## A.4 Temp-root scope

Conceptual discipline:

```rust
let depth = vm.temp_root_depth();
vm.push_temp_root(receiver);
vm.push_temp_root(query_key);
// push query value too when Bijection reverse preflight needs it

let result = (|| {
    // all re-entrant hash/equality sends
})();

vm.truncate_temp_roots(depth);
result
```

Use existing repository helpers if P6 introduced an RAII/scope abstraction. Do not invent unsafe guards.

## A.5 Bijection dual-plan lifetime

Keep receiver rooted across **both** forward and reverse preflight planning. Do not truncate after the forward plan while path ObjRefs/candidate Values are still needed and a reverse callback can collect.

Preferred ordering:

```text
root receiver/k/v
prepare forward
prepare reverse
validate conflict
commit forward
commit reverse
allocate wrapper
truncate temp-root scope when no callback and returned Value is safely rooted by call return machinery
```

If the runtime's primitive return path requires a different exact rooting lifetime, follow the live GC contract and add a focused forced-GC test.

---

# Appendix B — Proposed Bag algebra property oracle

For test-only reference model `M: logical_key -> BigInt`:

```text
additive(A,B)[k]      = A[k] + B[k]
union(A,B)[k]         = max(A[k], B[k])
intersection(A,B)[k]  = min(A[k], B[k])
difference(A,B)[k]    = max(0, A[k] - B[k])
```

Properties:

```text
A + empty == A
A + B == B + A
(A + B) + C == A + (B + C)

A union empty == A
A union B == B union A
A union A == A

A intersection B == B intersection A
A intersection A == A
A intersection empty == empty

A difference empty == A
A difference A == empty

count(A + B, k) == count(A,k) + count(B,k)
count(A union B,k) == max(count(A,k), count(B,k))
count(A intersection B,k) == min(count(A,k), count(B,k))
count(A difference B,k) == max(0, count(A,k)-count(B,k))
```

Use bounded randomized counts for broad property sequences plus dedicated LargeInt cases for exactness.

---

# Appendix C — Proposed Bijection reference-model oracle

Test-only model:

```text
forward: ordinary host/test map K -> V
reverse: ordinary host/test map V -> K
```

For each generated operation:

```text
associate(k,v)
removeKey(k)
removeValue(v)
inverse
lookupKey(k)
lookupValue(v)
```

After every successful mutation:

```text
forward.len == reverse.len
for (k,v) in forward: reverse[v] == k
for (v,k) in reverse: forward[k] == v
runtime Bijection agrees with both maps
all previously retained runtime Bijection versions still agree with their recorded model snapshots
```

After every conflict/callback failure:

```text
current runtime value unchanged
all retained old versions unchanged
next valid operation still succeeds
```

Use user-defined colliding keys/values in a separate deterministic lane so host-map hash behavior does not accidentally substitute for Telos collision semantics.

---

# Appendix D — Repository references used to prepare this plan

Remote branch paths verified at `ffa2ab96578acf2f176aa2ddbd31e655590c7d29`:

```text
AGENTS.md
docs/specs/README.md
docs/decisions/README.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/implementation/COLL006/COLL006.C1/COLL006.C1.P6-detailed-implementation-plan.md
phalcom-core/src/primitive/ordered_hash.rs
phalcom-core/src/primitive/mod.rs
phalcom-core/src/primitive/number.rs
phalcom-core/src/value/mod.rs
phalcom-core/src/vm/gc.rs
phalcom-core/src/heap/object.rs
phalcom-core/src/heap/mod.rs
phalcom-core/src/heap/accessors.rs
phalcom-core/src/heap/trace.rs
phalcom-core/src/universe/core_classes.rs
phalcom-core/src/universe/primitives.rs
phalcom-native-meta/src/universe.rs
phalcom-semantic/src/core_surface/identity.rs
phalcom-modules/src/builtin.rs
phalcom-core/core/universe/src/collections/package.ph
phalcom-core/core/universe/src/collections/iterator_laws.ph
phalcom-core/core/universe/src/errors/argument.ph
phalcom-core/tests/README.md
phalcom-core/tests/core/mod.rs
phalcom-core/tests/core/memory/gc.rs
phalcom-core/tests/core/object_model/invariants.rs
```

User-supplied execution records used as newer state authority:

```text
COLL006.C1-CHECKPOINT.md
PROGRAM.md
STATUS.md
COLL006.C1.P5-walkthrough.md
COLL006.C1.P5-handoff.md
luna-patch-grade-plan-schema.md
```

The remote branch does not yet expose the checkpoint-recorded P6 `collection-families.md`/TDR-0092 files. T0 intentionally treats that as repository-state drift to reconcile, not as evidence that the checkpoint ruling is invalid.
