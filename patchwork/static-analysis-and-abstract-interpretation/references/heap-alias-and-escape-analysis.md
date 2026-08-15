# Heap, Alias, and Escape Analysis

Lexical dataflow is comparatively easy because a `BindingId` can identify one source-level storage binding. Heap reasoning is harder because many expressions may designate the same runtime object, one abstract object can represent many concrete allocations, fields can be mutated through aliases, closures turn locals into heap-like cells, and native code can retain or mutate references outside the visible Phalcom body.

This reference gives the minimum theory and engineering discipline needed before Phalcom adds field-sensitive heap reasoning, escape-driven optimization, concurrency ownership facts, or proofs that rely on absence of aliases. Do not introduce a heavyweight points-to engine merely because such analyses exist; the current semantic engine should remain local/summary-oriented until a real consumer needs heap precision.

## 1. Concrete heap model

A simple concrete state can be written:

```text
σ = (ρ, H)

ρ : variable/binding -> concrete location or immediate value
H : object-location × field -> concrete value
```

Aliasing means two expressions evaluate to the same location:

```text
a = object
b = a
b._x = 2
```

If `a` and `b` denote location `ℓ`, then the concrete update is:

```text
H'(ℓ, _x) = 2
```

An analysis that independently stores `a._x` and `b._x` without a relation between `a` and `b` can preserve a stale `a._x` fact after `b._x = 2`. That is unsound for any correctness consumer.

## 2. Points-to abstraction

A common may-analysis maps references to sets of abstract locations:

```text
Pts# : ValueId -> P(AbstractLocation)
```

Example:

```text
Pts#(a) = {AllocSite#12}
Pts#(b) = {AllocSite#12}
```

Soundness requires:

```text
if concrete value v may point to concrete location ℓ
and ℓ is abstracted by L#
then L# ∈ Pts#(v)
```

A points-to set is a may property: adding possible aliases loses precision but is conservative; dropping a real alias is unsound.

## 3. Choosing abstract locations

### Allocation-site abstraction

```text
AbstractLocation = AllocationSite(SourceExprId)
```

Every object allocated at one source site maps to the same abstract location. This is simple and works well enough for many compiler analyses.

Loop example:

```text
for item in items {
    list.add(Node.new(item))
}
```

Every `Node` created at the same `new` site shares one abstract location, so field updates across different concrete nodes join.

### Context-sensitive allocation sites

Refine by calling context:

```text
Location = (AllocationSite, CallContext)
```

This separates objects created by the same helper for different callers, at higher memory/solver cost.

### Type/class abstraction

```text
Location = ClassId
```

Very coarse but sometimes enough for “may mutate any instance of C” effects.

### Singleton/static locations

Modules, class-side storage, interned singletons, and globals may deserve stable semantic locations distinct from ordinary allocation sites.

Do not pick one universal abstraction for every consumer. Effect summaries may only need field-class sets; scalar replacement needs much stronger object identity.

## 4. Abstract heap

A field-sensitive heap domain can be:

```text
Heap# : (AbstractLocation, FieldId) -> Value#
```

or, if fields are only class-known:

```text
Heap# : AbstractObjectId -> Map<FieldSlot, Value#>
```

A field-insensitive version merges all fields per object. It is cheaper but loses information quickly.

The abstract heap must have a join:

```text
(H1# ⊔ H2#)(l, f) = H1#(l, f) ⊔ H2#(l, f)
```

and a policy for locations absent from one predecessor. Absence cannot mean unreachable unless that invariant is explicit.

## 5. Strong versus weak updates

Suppose analysis executes:

```text
x._f = v
```

### Strong update

Replace the previous fact only when the abstract reference denotes exactly one concrete storage location under current assumptions:

```text
H#[(ℓ#, f) := v#]
```

A sufficient condition is often:

```text
Pts#(x) = {ℓ#}
and ℓ# is a must-singleton abstraction at this program point
```

Merely having a points-to set of cardinality one is **not always enough**. An allocation-site abstract location may represent many concrete objects allocated in a loop. This is the “singleton abstraction” distinction.

### Weak update

Otherwise join with old state:

```text
H'#(ℓ#, f) = H#(ℓ#, f) ⊔ v#
```

for every possible `ℓ# ∈ Pts#(x)`.

Weak update preserves all possible concrete objects represented by that abstract location.

## 6. Must-alias, may-alias, and no-alias

Different consumers need different relations:

```text
MayAlias(x, y)
    there exists a concrete execution where x and y designate same location

MustAlias(x, y)
    on all represented executions they designate same location

NoAlias(x, y)
    no represented execution aliases
```

A may-points-to analysis can prove `NoAlias` when sets are disjoint, under sound abstraction:

```text
Pts#(x) ∩ Pts#(y) = ∅  =>  NoAlias(x, y)
```

but overlapping sets only imply “may alias,” not “must alias.”

Optimizer transformations often need no-alias or must-singleton facts, not merely a small may-set.

## 7. Escape analysis

Escape is a may-property over an object's reachability beyond some boundary. Useful levels:

```text
NoEscapeExpression
NoEscapeCallable
EscapesToCaller
EscapesModule
EscapesFiber
EscapesFFI
GlobalEscape
UnknownEscape
```

The order can be modeled as increasing observation scope, or as independent flags when boundaries are not naturally nested.

An allocation escapes a callable when it can be:

- returned;
- stored into an object reachable by caller/global state;
- captured by an escaping closure;
- passed to a callee/native boundary that may retain it;
- scheduled into another fiber/task;
- inserted into a global/module/class-side container.

## 8. Escape constraints

A simple graph-based escape solver can generate edges:

```text
x = y                 => PointsTo(x) includes PointsTo(y)
x.f = y               => heap edge x --f--> y
return x              => Escape(x, Caller)
Global.g = x           => Escape(x, Global)
closure captures x    => Escape(x, ClosureLifetime)
ffi(x)                 => Escape(x, FFI) unless summary says NoRetain
```

Escape propagates through reachable heap edges:

```text
Escape(x, B) and x.f may point to y
    => Escape(y, B)
```

Solve monotonically with a worklist. Do not repeatedly traverse whole bodies once constraints are indexed.

## 9. Why escape analysis matters

Potential uses include:

- stack allocation of otherwise heap objects;
- scalar replacement of aggregates;
- eliding synchronization or interference havoc for local objects;
- proving an unknown call cannot mutate a fresh unpassed object;
- deciding whether captured cells need heap lifetime;
- narrowing FFI rooting/retention contracts;
- explaining whether a fiber closure shares mutable state.

Each optimization needs its own proof condition. “Does not escape” is not automatically enough for stack allocation if reflection exposes object identity or GC/finalization semantics make allocation observable.

## 10. Closures and captured storage

A mutable captured local is best modeled as a cell:

```text
Cell(BindingId, HomeCallableId)
```

Closures point to that cell. Two closures capturing the same binding alias the same cell.

```text
let n = 0
let inc = || { n = n + 1 }
let reset = || { n = 0 }
```

The points-to graph contains:

```text
inc   -> Cell(n)
reset -> Cell(n)
```

If either closure escapes, the cell escapes to at least the same lifetime/scope. If the closure crosses a fiber boundary, the cell becomes shared mutable state unless runtime semantics copy/isolate it—which must be verified, not assumed.

## 11. Receiver-local field syntax versus alias semantics

Phalcom may restrict how source code names fields, e.g. receiver-local access. Such syntax can simplify *which fields code can directly name*, but it does not automatically remove aliases to the receiver object. A method invoked on an alias still mutates the same object.

Therefore keep separate:

```text
access privilege / field visibility
alias identity / heap reachability
```

Reflection may further widen access. Any proof relying on field encapsulation must use the normative reflective privilege model.

## 12. Heap havoc for calls

Given call effect summary `W#`, mutate only reachable locations it may write:

```text
HavocHeap(H#, call, args#):
    targets = ReachableFrom(args#, globals#, captured#, receiver#)
    for location/field permitted by W#:
        H#(location, field) := TopValue or joined summary effect
```

An unknown call may require a broad write set. But local fresh objects not passed, captured, or globally reachable can often remain precise if escape/reachability analysis proves isolation.

This is one reason a modest escape analysis can pay for itself before a sophisticated points-to solver.

## 13. Abstract garbage collection

Detailed abstract interpretation can accumulate abstract locations that no longer matter. Abstract GC computes reachability from abstract roots and drops unreachable abstract heap entries:

```text
Roots# = live locals ∪ globals ∪ escaped references ∪ pending callbacks
Reach# = transitive closure through abstract heap
H# := H# restricted to Reach#
```

This can improve both precision and memory use because stale merged heap nodes stop contaminating joins. It is advanced; only add it when Phalcom has a heap abstraction rich enough to benefit.

## 14. Flow sensitivity and heap versions

A flow-sensitive heap analysis has different heap facts at different program points. SSA does not solve heap mutation by itself. Options include:

```text
explicit HeapState# in dataflow
MemorySSA-like versions
region/effect summaries
field-specific version counters
```

For current Phalcom semantic analysis, effect summaries plus conservative field facts are likely cheaper than full MemorySSA. Introduce a reusable heap IR only when optimizer/prover consumers repeatedly need the same alias-sensitive semantics.

## 15. Interprocedural points-to analysis

Classic choices include:

- Andersen-style inclusion constraints: scalable-ish, flow/context insensitive;
- Steensgaard-style unification: faster, coarser;
- flow-sensitive points-to: more precise, more expensive;
- object/context sensitivity: separates calling/allocation contexts.

Do not cargo-cult one algorithm. Ask which Phalcom property requires it. For example:

- completion rarely needs whole-program points-to;
- escape-driven optimizer might start with intraprocedural + summary escape;
- proving mutation footprints may need field-sensitive interprocedural regions;
- FFI safety may rely more on explicit ownership/effect contracts than inferred aliases.

## 16. FFI aliasing and retention

A Rust boundary needs declarations for at least:

```text
borrowed for call only?
retained after return?
mutated?
transitively mutated?
callback retained?
returns alias of an argument?
returns fresh object?
shares backing buffer/storage?
thread/fiber escape?
```

Without such a contract, passing a mutable Phalcom object can make it escape and invalidate heap facts reachable from it.

A native function returning an existing backing buffer alias must not be modeled as fresh merely because the wrapper constructs a new Phalcom-level object.

## 17. Runtime GC reachability is not static alias analysis

Do not conflate:

```text
GC tracing
    which concrete heap objects are reachable right now during execution

static points-to
    which abstract locations a source value may designate across executions

escape analysis
    whether an allocation may become reachable beyond a boundary
```

They may share concepts such as roots and reachability but solve different problems. Static analysis must not infer non-aliasing from the runtime GC handle representation alone.

## 18. Current Phalcom position

The inspected LSP semantic engine currently tracks source field evidence and lexical/captured-flow facts but does not expose a general points-to heap domain. That is reasonable for its present advisory use cases. **RECOMMENDATION:** preserve this separation until a concrete optimizer/prover/concurrency feature needs alias precision. When that time comes, bridge existing `BindingId`, `FieldId`, `CallableId`, source provenance, and effect summaries into a dedicated heap domain rather than storing pseudo-heap facts inside `ValueShape`.

## 19. Rust representation and scaling

Prefer typed IDs:

```rust
struct AbstractObjectId(u32);
struct RegionId(u32);
struct AllocationSiteId(u32);
```

and arena/indexed storage:

```text
points_to[value_id] -> interned bitset/set<AbstractObjectId>
heap[obj_id][field_id] -> ValueFactId
escape[obj_id] -> EscapeFact
reverse_users[obj_id] -> compact dependent set
```

Bitsets are attractive when abstract-object universes are bounded per analysis unit. Interned sorted small sets can work better for sparse sets. Measure allocation/hash cost before committing.

Canonicalization is essential: solver convergence and incremental equality must not depend on insertion order or freshly allocated set identity.

## 20. Failure modes

- Tracking `a._f` and `b._f` independently without alias reasoning.
- Strong-updating an allocation-site object just because `Pts#(x)` has one abstract element.
- Treating a returned object as non-escaping because it stays in the same module.
- Ignoring closure capture when computing escape.
- Treating “passed to FFI” as borrowed unless the native contract says so.
- Using GC object handles as proof of static uniqueness.
- Keeping heap facts unchanged across an unknown call that can reach them.
- Making every unknown call havoc the entire heap forever when escape analysis can safely preserve local fresh state.
- Adding whole-program Andersen analysis before a consumer justifies the complexity.

## 21. Testing obligations

1. direct aliases observe one field update;
2. disjoint allocation sites permit no-alias when justified;
3. loop allocation site uses weak update where one abstract node represents many concrete objects;
4. singleton fresh allocation can receive strong update under explicit singleton condition;
5. returned allocation escapes caller boundary;
6. storing into global/container propagates escape transitively;
7. escaping closure makes captured cell escape;
8. non-escaping closure does not automatically force global escape;
9. FFI `NoRetain` versus `MayRetain` produce distinct escape facts;
10. unknown call havocs only reachable mutable heap under the selected envelope;
11. yield preserves fiber-local objects but invalidates shared mutable objects when concurrency model requires it;
12. incremental body edit recomputes affected alias/escape frontier and matches clean analysis.

Property tests should exercise set union/order laws and monotonic escape propagation.

## 22. Review questions

1. What concrete locations does this abstract location represent?
2. Is a one-element points-to set actually a concrete singleton?
3. Why is this update strong rather than weak?
4. Through what paths can this object escape?
5. Can a closure/native call retain it after return?
6. Which aliases can observe the mutation?
7. Which heap facts survive an unknown call or fiber yield, and why?
8. Does the consumer truly need a heap analysis this precise?
9. How are abstract locations canonicalized and invalidated?
10. Are runtime GC reachability and static alias facts being kept conceptually separate?
