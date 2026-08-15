# Phalcom Analysis Domain Map

Phalcom needs one coherent semantic truth and multiple analysis abstractions over that truth. This document is the map that prevents those abstractions from collapsing into one giant `Unknown`-heavy model or, worse, from silently redefining language semantics. It identifies what is CURRENT in the inspected repository, what belongs to future correctness machinery, and the explicit bridges agents should design between domains.

Repository-specific statements in this reference are grounded in the inspected `main` baseline dated 2026-08-14. Re-inspect source before implementation because the semantic engine is active work.

## 1. The five-layer distinction

Never collapse these layers:

```text
1. Dynamic language semantics
   What actual Phalcom execution means.

2. Runtime representation
   Objects, classes, handles, bytecode, VM storage, method tables, GC.

3. Analysis abstraction
   Finite approximation used to answer a static question.

4. Language type / proof fact
   Normative correctness relation or proposition.

5. Optimization fact
   Assumption strong enough to justify a transformation, often with guards.
```

A runtime class object can provide evidence to several layers without being identical to any of them.

Example:

```text
runtime value has concrete class String
    ↓ bridge
advisory ValueShape = Instance(String)
    ↓ possible bridge
nominal type evidence: String
    ↓ if trusted under checker rules
send constraint may be discharged
    ↓ if world/effect assumptions also hold
optimizer may consider guarded devirtualization
```

Every arrow has conditions. Do not replace the diagram with equality signs.

## 2. CURRENT: semantic identities and source products

The current LSP semantic subsystem already has stable semantic concepts including module-, class-, callable-, field-, and binding-level identities, source surfaces/scopes, occurrence indexing, callable lookup, module dependencies, and immutable published snapshots.

Use those identities as the spine for future analyses. Do not create checker-specific string keys for the same declaration when the semantic model already has an identity.

Important distinctions:

```text
ModuleId       semantic module identity
ClassId        module-qualified class identity
CallableId     owner + selector + dispatch side identity
FieldId        owner + field name + storage/dispatch side
BindingId      lexical declaration identity
SourceRange    source location, not semantic identity
FileRevision   source revision stamp
SemanticGeneration published coherent analysis generation
```

A source edit can change ranges without changing declaration identity. Current invalidation code deliberately fingerprints declaration semantics without letting source ranges define equality.

## 3. CURRENT: advisory `ValueShape`

The current `ValueShape` domain explicitly describes advisory runtime value knowledge and explicitly says it is not a language type. At the inspected baseline it contains:

```text
Unknown
Instance(ClassId)
ClassObject(ClassId)
Module(ModuleId)
Tuple(Vec<ValueShape>)
Record(Vec<(String, ValueShape)>)
List(Box<ValueShape>)
Set(Box<ValueShape>)
Map { key, value }
Range(Box<ValueShape>)
Callable(CallableId)
Family { receiver, base }
Union(Vec<ValueShape>)
```

The union is bounded by `MAX_SHAPE_UNION = 8`; overflow widens to `Unknown`. Structural collection joins preserve element/lane facts where compatible.

This is excellent for editor inference and receiver-aware dispatch. It is **not** automatically suitable for a future checker because:

- `Unknown` is an advisory loss of runtime-shape precision, not a formal `Dynamic` type;
- bounded union overflow is an engineering widening, not a language union-type rule;
- shape equality follows runtime categories, not necessarily type equivalence/subtyping;
- no type variables, generic substitution, variance, protocols, intersections, or formal dynamic boundaries are represented.

## 4. CURRENT: fact quality and provenance

`InferredValue` currently carries:

```text
shape
known_boolean
confidence
bounded provenance
```

with confidence categories:

```text
Exact
Flow
Interprocedural
Heuristic
```

and origins such as syntax, binding, callable, call site, and constraint.

This is an important foundation for explaining facts. Future correctness machinery should preserve provenance but may need a more explicit trust model. “Interprocedural” says how evidence was derived, not by itself whether the derivation is sound enough for a prover or optimizer.

A future fact envelope might separate:

```text
Fact<T> {
    value: T,
    trust: TrustClass,
    precision: PrecisionStatus,
    provenance: ProvenanceId,
    dependencies: DependencySetId,
}
```

so derivation source, soundness status, and precision-loss reason do not collapse into one ordinal confidence.

## 5. CURRENT: local flow domain

The current structured-flow state is approximately:

```text
FlowState = BTreeMap<BindingId, InferredValue>
```

and statement analysis produces a product of normal and abrupt outcomes:

```text
normal: Option<FlowState>
returns
breaks
continues
throws
tail_value
```

This already distinguishes reachability (`normal = None`) from an unknown value. Loops use bounded fixed-point iteration and widening. Branches join states and trusted class-test conditions can refine them.

This is a sound *shape-flow architecture pattern*, but its consumer contract remains advisory. Future correctness analyses can reuse the control-flow machinery while substituting or product-composing domains with stronger semantics.

## 6. CURRENT: callable summaries and contribution facts

The semantic engine currently summarizes source callables with:

```text
params: Vec<InferredValue>
returns: InferredValue
dependencies: Vec<CallableId>
effects: SummaryEffects
revision: SemanticGeneration
```

`SummaryEffects` currently contains:

```text
dynamic_send
invokes_parameters
```

Parameter evidence is contribution-indexed by source and parameter slot. Replacing a caller removes that caller's previous contribution before recomputing the join. This solves an important incremental-analysis problem: monotone joins alone cannot retract stale evidence after source deletion/change.

Future domains should reuse this contribution/dependency pattern when facts are aggregated from many source owners.

## 7. CURRENT: incremental/publication domain

The semantic worker keeps mutable analysis state but publishes coherent immutable `SemanticSnapshot`s backed by `Arc`-shared products. Source changes are classified into body/import/declaration/file/core categories; body-only edits can seed an exact callable frontier, while broader surface changes expand through module dependencies.

Cancellation is transactional: analysis applies to a candidate engine state and publishes only if the worker epoch is still current. This is a strong architecture to preserve for future type/proof analyses.

A stale internally consistent fact is still incorrect. Generation coherence is part of semantic correctness for live tooling.

## 8. FUTURE: formal language type domain

Do not extend `ValueShape` until it “looks like a type system.” A future type representation may require concepts such as:

```text
Type =
    NominalInstance(ClassId, Args)
  | ClassObjectType(Type)
  | Protocol(ProtocolId, Args)
  | Union(Vec<Type>)
  | Intersection(Vec<Type>)
  | Tuple(...)
  | Record(...)
  | Callable(...)
  | Applied(TypeCtor, Args)
  | TypeVar(TypeVarId)
  | SelfType(...)
  | Dynamic             # only if ratified
  | UnknownInferenceVar # implementation state, not language Dynamic
  | Top
  | Bottom
```

Exact shape depends on the ratified typing specification. The type skill owns subtyping, assignability, generic inference, substitution, variance, normalization, and kinds.

This static-analysis skill owns only how those facts participate in flow, fixed points, effects, interprocedural summaries, incrementality, and bridges.

## 9. FUTURE: path/proposition domain

A path/proof domain should retain propositions not naturally represented as types:

```text
x isSome
tag(x) == VariantA
x < 10
x == y
field(self, ready) == true
contract assumption P
not DynamicMutationSince(epoch)
```

Possible implementation tiers:

```text
Tier 0: recognized finite refinements only
Tier 1: boolean/presence/tag propositions
Tier 2: intervals/difference constraints
Tier 3: symbolic formula/SMT-facing representation for prover
```

Do not make the everyday LSP carry solver formulas for every expression. Share proposition identities and provenance while allowing consumer-specific abstraction strength.

## 10. FUTURE: effect domain

The current `dynamic_send` + `invokes_parameters` summary is deliberately compact. Correctness/optimization may need:

```text
throws
reads/writes fields/globals/captured cells
allocates
IO/process/network
may_yield
blocks_thread
spawns/deferred callback
reflection reads/mutation
dynamic invocation
FFI/native boundary
callback invocation timing/cardinality
```

Effect facts compose with value/type facts but are not types. A callable can return exact `Int` while having unknown heap effects.

## 11. FUTURE: heap/alias/escape domain

Only introduce this when a consumer needs it. Candidate facts:

```text
PointsTo(ValueId) -> Set<AbstractObjectId>
Heap(AbstractObjectId, FieldId) -> ValueFact
Escape(AbstractObjectId) -> EscapeFact
RegionOwnership -> Local/Shared/FFI/Global
```

The LSP does not need a whole-program points-to engine merely for completion. Optimizer/prover/concurrency work may.

Keep heap domain separate from runtime GC reachability and from source field evidence.

## 12. FUTURE: proof domain

A prover may require:

```text
symbolic terms
path conditions
verification conditions
loop invariants
heap/frame predicates
contract assumptions
solver result = Proved | Disproved(model) | Unknown(reason)
```

Do not encode `Proved` as “ValueShape is exact” or “no runtime counterexample was seen.” Abstract interpretation can generate invariants and discharge simple facts, but proof has a separate obligation semantics.

## 13. FUTURE: optimization assumption domain

Optimization requires facts strong enough to preserve observable execution. Examples:

```text
UniqueTarget(callsite)
ReceiverClassExact(C)
MethodTableEpoch(E)
NoAlias(a,b)
NoThrow(expr)
NoYield(expr)
NoObservableAllocation(expr)
ConstantValue(v)
RangeWithin(bounds)
```

These may be derived from static analysis, language invariants, runtime profiling + guards, or combinations. Keep provenance/validity:

```text
OptimizationFact {
    proposition,
    proof_source,
    runtime_guards,
    invalidation_condition,
}
```

A high-confidence LSP heuristic is not an optimization fact.

## 14. Bridge: runtime shape -> nominal type evidence

A possible bridge:

```text
shape_to_type_evidence(Instance(C)) = NominalInstance(C)
```

is sound only for what it claims: “the represented executions have runtime class C” can support nominal type evidence if type semantics classify that runtime class accordingly.

But:

```text
ValueShape::Unknown
```

must not map blindly to `Dynamic`. It might mean union cap, missing dependency, analysis budget, malformed source, unresolved call, or genuinely dynamic behavior.

Use:

```text
ShapeKnowledge::Unavailable(reason)
```

or equivalent provenance to choose the right checker behavior.

## 15. Bridge: declared type -> shape constraint

A declared type may constrain receiver candidates:

```text
x : String
```

could seed static receiver knowledge, but only according to the typing/runtime enforcement profile:

- statically checked strict code may trust the declaration under checker assumptions;
- typed runtime mode may enforce it at boundaries;
- untrusted dynamic interop may require a guard;
- editor inference can show it as declared evidence rather than observed shape.

Do not erase whether the fact was declared, inferred, runtime-checked, or proven.

## 16. Bridge: Option/type fact -> path refinement

```text
Type(x) = Option<T>
condition = isSome(x)
```

can produce:

```text
true path proposition:  tag(x) = Some
true type refinement:    x : Some<T> / payload T
false path proposition: tag(x) = None
```

The type checker owns the formal refinement rule; the flow engine owns edge propagation and merge.

## 17. Bridge: path fact -> type narrowing

A proof/path fact can eliminate alternatives:

```text
Type(x) = A | B
PathFact = runtimeClass(x) == A
=> refined Type(x) = A
```

This is a reduction between domains. It should be centralized and tested for soundness, not reimplemented ad hoc in hover, checker, and prover.

## 18. Bridge: callable type + runtime dispatch

A typed send has two distinct questions:

```text
1. Dynamic semantics:
   Which method implementation(s) can runtime dispatch select?

2. Type correctness:
   Are receiver/arguments/results compatible with the declared contract?
```

Do not make type checking choose a different method unless Phalcom explicitly ratifies type-directed dispatch. The proposed typing architecture inspected in the repo explicitly recommends keeping types out of selector identity.

## 19. Bridge: effects -> refinement validity

A path fact depends on mutable locations:

```text
Fact: self._state is Ready
Deps: Field(self, _state)
```

A call summary with write effect on that location kills the fact. A call proven not to write it preserves it. This bridge is where effect precision directly increases flow/type/proof precision.

## 20. Bridge: escape -> concurrency interference

If object `o` is proven local to the current fiber and not reachable from global/native/shared state, a yield need not havoc facts about `o`. If it escapes across fibers, yield may invalidate mutable facts.

This bridge allows concurrency precision without baking scheduler concepts into every value fact.

## 21. Bridge: analysis fact -> optimizer

Require an explicit promotion function:

```text
try_promote_to_optimization_fact(fact, assumptions)
    -> ValidOptimizationFact | CannotPromote(reason)
```

Examples:

```text
Exact shape from immutable allocation + stable method table
    -> unique dispatch target

Heuristic shape from use-site constraint
    -> cannot promote

Declared type under unchecked dynamic boundary
    -> guard required
```

This makes unsafe promotions reviewable.

## 22. Knowledge-state taxonomy

Across domains, preserve reasons such as:

```text
Exact
FlowDerived
InterproceduralSound
DeclaredTrusted
Proven
Heuristic
Widened
DynamicByLanguageChoice
NotYetComputed
DependencyMissing
RecoveryBlocked
Ambiguous
Inconsistent
BudgetExceeded
SolverUnknown
Unreachable
```

Not every domain needs every variant. The doctrine is that uncertainty reason is semantically useful and should not be flattened prematurely.

## 23. Consumer acceptance matrix

A useful policy table:

| Fact class | LSP display | Checker | Prover | Optimizer | Lint |
|---|---:|---:|---:|---:|---:|
| exact syntax/semantic invariant | yes | yes | yes | usually | yes |
| sound flow/interprocedural fact | yes | yes | yes if modeled | yes if invariant sufficient | yes |
| declared trusted contract | yes | yes per policy | assumption/TBC | guard/trust policy | yes |
| runtime-guarded fact | maybe | runtime mode | conditional | yes with guard | maybe |
| heuristic inference | yes | no correctness acceptance | no | no | advisory only |
| widened top/unknown | explain | conservative policy | unknown | cannot optimize | depends |
| bottom/unreachable | hide/explain | vacuous path | valid unreachable assumption | remove if proven | yes |

This is conceptual, not a ratified API.

## 24. Incremental dependencies between domains

A future dependency chain might be:

```text
source revision
  -> parsed source/surface
  -> semantic identities/scopes
  -> type declarations/module interfaces
  -> callable/field/type dependencies
  -> shape/type/effect/path summaries
  -> proof/optimization products
  -> immutable published generation
```

If a declaration's type changes but body text does not, type-dependent consumers must invalidate even if the current body-only fingerprint would not. Extend declaration fingerprints as the language surface grows; do not hide new semantic inputs from invalidation.

## 25. One semantic source, multiple abstractions

“Share the semantic foundation” does **not** mean every consumer queries one gigantic `Fact` enum. It means:

```text
shared identities
shared source/module/dispatch semantics
shared dependency ownership
shared control-flow representation where justified
explicit bridges
consumer-specific abstract domains and trust thresholds
```

This preserves coherence without forcing an LSP latency domain to carry all prover machinery.

## 26. Review scenarios

### Scenario A — shape becomes type

Proposal:

```text
ValueShape::Union([Int, String]) -> Type::Union(Int, String)
ValueShape::Unknown -> Type::Dynamic
```

Reject the second mapping. `Unknown` may be a precision failure, not explicit language dynamicity. Require provenance-sensitive bridge rules.

### Scenario B — declared type picks method

Proposal: two methods with same selector but different annotations; checker picks target by argument types.

Reject unless type-directed dispatch is a separately ratified language feature. Current selector identity must remain the dispatch basis.

### Scenario C — exact type allows devirtualization

Not enough. Also require runtime/world stability of method lookup, receiver side, reflection assumptions, and effects/guards.

### Scenario D — prover uses LSP union cap

Reject. Widening to `Unknown` after eight shape alternatives is an advisory engineering policy. Proof/checker fallback must preserve their own contract.

## 27. Review questions

1. Which semantic domain owns this fact?
2. What does `⊤`/unknown mean in that domain?
3. Is this a runtime shape, formal type, path proposition, effect, alias fact, proof result, or optimizer assumption?
4. What bridge converts it to another domain, and under which assumptions?
5. What provenance/trust survives the conversion?
6. What invalidates the fact?
7. Which consumer is allowed to act on it?
8. Is source identity being confused with semantic identity or revision identity?
9. Does new syntax/type metadata require extending invalidation fingerprints?
10. Can this domain stay small and consumer-focused instead of absorbing neighboring theory?

If an implementation cannot answer those questions, it is likely collapsing semantic layers that Phalcom needs to keep distinct.
