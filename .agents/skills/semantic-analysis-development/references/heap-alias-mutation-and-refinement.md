# Heap, Aliasing, Mutation, and Refinement Stability

This reference owns a problem that local AST inference tends to postpone until it becomes a correctness bug: a Phalcom value may be reachable through aliases, fields, closures, reflective state, or concurrent/fiber-visible state. A refinement is valid only while the proposition it describes remains stable.

## 1. Binding facts and heap facts are different

Consider:

```phalcom
let user = currentUser()
if user.name != None {
    use(user.name)
}
```

The local binding `user` may be immutable while the object referenced by `user` is mutable. Therefore:

```text
binding_stable(user)  ≠  heap_property_stable(user.name)
```

A flow engine that keeps refinements solely because the local variable was not reassigned can become unsound once calls, aliases, setters, reflection, or suspension may mutate the object.

## 2. A minimal concrete heap model

For reasoning, use:

```text
ρ : BindingId -> AbstractLocationSet
H : AbstractLocation -> Object/Field State
```

At runtime, locations are concrete object identities. Static analysis normally uses abstract locations such as:

```text
AllocationSite(site_id)
Parameter(callable, index)
SelfObject(class/callable context)
GlobalObject(id)
UnknownEscaped
```

You do not need a full points-to analysis for every editor query. But every heap-sensitive fact implicitly assumes some alias model, even if that model is “all mutable heap aliases everything.” Make that assumption explicit.

## 3. Strong versus weak update

A **strong update** replaces prior knowledge because analysis knows the assignment targets exactly one abstract location:

```text
H'[ℓ.f] = v
```

A **weak update** joins because the target may represent multiple runtime locations:

```text
H'[ℓ.f] = H[ℓ.f] ⊔ v
```

Strong update is justified only if uniqueness is established for the abstract location at that point. A class-wide field summary such as `(ClassId, field)` aggregates many instances and therefore normally requires weak-update/may semantics.

Current Phalcom `FieldFacts` are evidence summaries of field values, not a flow-sensitive per-object heap. Do not accidentally interpret them as proving the current value of `self.field` after arbitrary mutation.

## 4. Kill sets for refinements

Suppose branch analysis establishes:

```text
R = { place(user.name) excludes None }
```

A statement kills `R` if it may mutate the place or invalidate the identity assumption used by the refinement.

Define conceptually:

```text
kill(stmt, R) = { r ∈ R | may_invalidate(stmt, r.place, r.assumptions) }
```

Transfers become:

```text
R_out = (R_in - kill(stmt, R_in)) ∪ gen(stmt)
```

For a call, `may_invalidate` consults effect/alias information. If an unknown call may mutate any reachable heap state, heap refinements should be killed conservatively while immutable lexical-value refinements can survive.

## 5. Place identity

To refine structured state, identify what is being refined. A future `Place` representation could distinguish:

```rust
pub enum Place {
    Binding(BindingId),
    Field { base: PlaceBase, field: FieldId },
    TupleElem { base: PlaceBase, index: u32 },
    Index { base: PlaceBase, key: AbstractKey },
}
```

This is a conceptual sketch, not a requirement to implement this enum immediately. The important property is that refinement keys are semantic places, not source text such as `"user.name"`.

Text keys fail under shadowing, aliasing, rename, and semantically equivalent expressions.

## 6. Stability classes

A practical staged implementation can classify refinements:

```text
StableLexical
    immutable local/parameter value; no rebinding

LocallyMutable
    local can be reassigned; kill on writes/captured writes

HeapDependent
    field/property/index state; kill on relevant may-mutate effects

ExternallyMutable
    globals, FFI-visible memory, shared objects; conservative across calls/yields

DynamicDependent
    reflection/metaprogramming can change interpretation itself
```

This is more useful than one `is_mutable` bit because it connects facts to invalidation behavior.

## 7. Closures and captured mutation

Example:

```phalcom
let x = value
let bump = || { x = other }
if x is String {
    bump()
    use(x)
}
```

If the language permits captured reassignment, invocation of `bump` kills the refinement of `x`. If the block escapes to an unknown caller, later analysis may need to assume the write can occur at any point allowed by the escape semantics.

Capture analysis therefore needs more than “binding is captured”:

```text
CapturedRead
CapturedWrite
Escapes
InvokedSynchronously
InvokedDeferred
```

Add precision only when consumers need it; but do not preserve a refinement in the presence of a known captured write.

## 8. Setter sends and user code

A surface syntax assignment can invoke user code depending on Phalcom semantics. Distinguish:

```text
lexical variable assignment
raw VM/internal field store
language-level setter/message send
subscript assignment
reflective field mutation
```

If a setter is dispatched as a message, its effects include everything its method body or dynamic fallback may do. A syntactically assignment-like operation is not automatically a simple store.

## 9. Unknown calls and effect footprints

An effect system can represent mutation at increasing precision:

```text
MayMutateAnything
MayMutateReceiver
MayMutateArgument(i)
MayWriteField(FieldId)
MayWriteGlobal(GlobalId)
NoHeapMutation   // only when justified/proven/trusted
```

Do not start with the most precise scheme unless it has consumers. A sound staged approach is:

```text
CURRENT/near-term:
  unknown/dynamic calls -> MayMutateReachableHeap
  known source call     -> summary says whether it mutates
  known pure primitive  -> trusted no-mutation metadata

FUTURE:
  field/region footprints + alias-aware kills
```

The key is that “no recorded mutation effect” must not mean “pure” unless completeness of effect analysis is guaranteed.

## 10. Fibers and suspension

A cooperative fiber has no parallel execution during one uninterrupted run segment, but a suspension point permits other scheduled work to mutate shared state before execution resumes.

Thus:

```text
before yield: refine shared_obj.state = Ready
fiber yields
other fiber mutates shared_obj.state
resume: refinement may no longer hold
```

A `may_yield` effect should therefore act as a stability barrier for facts depending on externally mutable/shared heap state.

Lexical immutable values whose referenced identity/value cannot be changed may survive. Heap facts require a concurrency/ownership guarantee to survive.

## 11. Futures and callback reentrancy

Even without OS-thread parallelism, user callbacks can create reentrancy:

```text
method establishes invariant-like temporary state
method invokes callback/user hook
callback re-enters same object and mutates state
method resumes with stale assumption
```

Static analysis and optimizer reasoning should classify “invokes user code” as an invalidation event where appropriate. This is especially important for collection iteration, finalizers/hooks, coercions, reflection, and native callbacks.

## 12. Reflection

If Phalcom reflection can mutate class method dictionaries or fields, separate two categories:

```text
object-state mutation      -> invalidates heap/property facts
class/dispatch mutation    -> invalidates lookup/dispatch assumptions and caches
```

The latter is not ordinary dataflow mutation. A compiler inline cache may need class-version/shape guards; the semantic engine may need conservative dispatch results under open-world modification.

Do not respond by making all semantic identities unstable. Source declaration identity and runtime method-dictionary version are distinct concepts.

## 13. Escape analysis as a precision tool

Escape facts can justify stronger reasoning:

```text
NoEscape          // allocation remains local to region/callable
ReturnEscape
ArgumentEscape
GlobalEscape
ClosureEscape
UnknownEscape
```

If a freshly allocated object does not escape and no aliasing operation is introduced, field updates may be strongly modeled within a callable. Once escaped, join to a conservative heap abstraction.

Escape analysis is optional precision. Soundness must not depend on it unless the implementation is complete for the relevant operations.

## 14. Interprocedural summaries for heap effects

A callable summary may eventually carry:

```text
reads:   EffectFootprint
writes:  EffectFootprint
escapes: EscapeSummary
invokes_user_code: bool/may
may_yield: bool
```

At call sites:

```text
post_refinements = pre_refinements - killed_by(summary, aliases)
```

For unresolved dispatch, use the conservative summary for the dynamic boundary.

If dispatch resolves to a bounded set of callees, join their may-effects. For must-guarantees, retain only guarantees shared by every possible callee.

## 15. Field inference versus field validity

Current field evidence can answer questions such as:

> What shapes have source analysis observed being written to this class field?

It generally cannot answer:

> At this exact program point, what is the current value of this particular object's field?

Those require different abstractions. Keep separate APIs/types if both are implemented:

```text
FieldEvidenceSummary(FieldId)       // global/class-level advisory observations
HeapFlowFact(AbstractLocation, FieldId, point) // point-sensitive state
```

Collapsing them produces misleading hover/checker results.

## 16. Worked refinement example

Consider:

```phalcom
if self.token != None {
    log(self.token)
    consume(self.token)
}
```

Assume `log` is known not to mutate `self.token`, but `consume` may invoke arbitrary user code.

Entry:

```text
R0 = {}
```

True branch generation:

```text
R1 = { self.token != None }
```

After `log`:

```text
kill(log, R1) = {}
R2 = R1
```

After `consume`:

```text
kill(consume, R2) = { self.token != None }
R3 = {}
```

A second access after `consume` cannot use the earlier non-None proof unless a stronger effect/ownership contract establishes stability.

## 17. Rust representation guidance

Prefer compact semantic IDs and immutable effect summaries:

```rust
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RegionId(u32);

#[derive(Clone, Eq, PartialEq)]
pub struct EffectSummary {
    pub may_invoke_user_code: bool,
    pub may_yield: bool,
    pub mutation: MutationFootprint,
}
```

Avoid storing references into mutable arenas inside published facts. Snapshot-owned IDs should resolve through the snapshot/generation that created them.

Mutation footprints should be canonicalized so summary equality is deterministic and fixed points converge.

## 18. Tests

Required tests when heap-sensitive refinements are introduced:

- immutable local refinement survives unrelated local operations;
- direct reassignment kills local refinement;
- captured write kills refinement after closure invocation;
- non-invoked captured closure does not kill merely by construction;
- known no-mutation call preserves eligible refinement;
- unknown/dynamic call kills heap-dependent refinement;
- setter call kills affected field facts;
- unrelated field write preserves a field-specific refinement if footprint precision claims that;
- yield kills shared/external heap refinement;
- incremental change from pure -> mutating callee invalidates callers;
- clean full analysis equals incremental final facts.

## 19. Unsound shortcuts

Reject:

- “immutable binding means immutable object”;
- “no assignment syntax means no mutation”;
- “cooperative fibers imply refinements survive yield”;
- “captured means mutated” and its opposite “capture never matters”;
- “field facts are per-instance current-state facts”;
- “unknown call is pure until proven otherwise”;
- “reflection only affects IDE navigation, not semantic assumptions”;
- strong-updating class-wide aggregate state.

## 20. Review questions

- What semantic place is refined?
- Can two expressions alias that place?
- Is the fact about binding identity, object identity, or field state?
- Which writes are strong versus weak updates?
- Can the next call invoke user code?
- Can it yield, re-enter, reflect, or cross FFI?
- What effect summary kills the fact?
- Is the effect summary itself complete or advisory?
- How does a callee effect change invalidate caller refinements incrementally?
- Does the optimizer rely on a stronger stability guarantee than the LSP analysis can provide?
