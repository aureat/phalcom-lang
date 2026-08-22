# Specification 03: Bidirectional Flow Checking, Semantic Dispatch, and Proof-Ready Facts

**Status:** Draft implementation specification<br>
**Depends on:** Specification 01’s sound type kernel/relations and Specification 02’s typed module interfaces, query database, and immutable snapshots<br>
**Enables:** full expression checking, inherited/class-side/generic dispatch, protocols, definite assignment, strict-mode obligations, and honest future static proving<br>
**Primary owners:** `phalcom-semantic` checker/dispatch/CFG layers; module header surfaces; native descriptors; compiler diagnostics<br>
**Non-goal:** claiming that `@requires`/`@ensures` are statically proven today, or implementing a complete theorem prover in the type checker

## 1. Problem statement

The current expression engine demonstrates the right instinct—synthesize a local fact, preserve `Known`/`Unknown`/`Dynamic`, and reject only authoritative contradictions—but it is not yet a general typed expression system. `TypedExpression` carries constraints/provenance, yet regular synthesis does not collect and solve a unified constraint graph ([`expression.rs`](../../../../phalcom-semantic/src/checker/expression.rs)). `if let` and `while let` are checked as limited structured cases rather than CFG fixed points; assignments check compatibility without updating a flow environment; lists/maps/sets infer through special cases, and `List.add` directly binds an inference variable. Control flow does not establish definite assignment, reachability, recursive callable summaries, or a complete `Never` analysis.

Dispatch is exact registered-surface lookup. `CheckerContext` creates a local resolver and hardcoded native surface; applied types fall back to their origin, while inheritance, protocol, class-side, substituted generic members, reflective sends, and dynamic behavior are absent ([`context.rs`](../../../../phalcom-semantic/src/checker/context.rs)). The expression checker synthesizes `super` using `Object` as a static fallback rather than the actual lexical superclass lookup start ([`expression.rs`](../../../../phalcom-semantic/src/checker/expression.rs)). Missing dispatch often becomes `Unknown`, which conservatively avoids a false error but also makes a confirmed missing member indistinguishable from incomplete type evidence.

The runtime contract decorators prove a related architectural point. `@requires` and `@ensures` are runtime-woven checks, with explicitly documented current limitations; no static verification engine consumes them. A later prover cannot safely be bolted onto local expression types. It needs the same typed CFG, dispatch/effect results, source/module identity, native/dynamic boundary classification, and result-state honesty required by a production type checker.

This specification turns current local checks into a bidirectional, flow-sensitive semantic layer while defining the proof-ready facts it must publish.

## 2. Design invariants

1. **Type checking follows Phalcom evaluation and message sends.** Static checking must model receiver evaluation, argument evaluation/order, selector labels, class/instance side, and `super` lookup start from the language semantics. It may not invent a Python-style attribute model.
2. **Synthesis and checking are different judgments.** Expressions synthesize a type/evidence or are checked against an expected type. Expected types are propagated where they reduce ambiguity without discarding information.
3. **Facts are flow-node and snapshot scoped.** A name string is not a binding identity. A flow fact identifies a binding, a CFG point, and the snapshot/query in which it was proven.
4. **All bounded analysis has a terminal status.** Loop, recursion, constraint, dispatch, and proof queries end in solved/proved, contradiction, blocked/unknown reason, cancelled, or budget-exhausted state. No loop silently runs until it happens to converge.
5. **Dynamic boundaries are explicit.** `Dynamic`, opaque native metadata, reflection, `perform`, `doesNotUnderstand`, FFI, callbacks, and unmodelled effects degrade particular facts; they do not make all nearby expressions untyped or all claims true.
6. **Dispatch and conformance are separate.** A member lookup can find an inherited member without proving a protocol conformance. A protocol conformance result cannot be faked by incidental structural lookup.
7. **Proof facts are not type facts.** Type checking establishes well-formedness/relations; proving establishes a proposition subject to contracts, effects, aliases, and trusted boundaries. Each has its own result domain and cache key.

## 3. Semantic data model

### 3.1 Bindings, CFG, and expression facts

Introduce an HIR/semantic layer if the current AST cannot assign stable identities after parser recovery. The minimal fact model is:

```rust
struct BindingId {
    owner: CallableOrModuleId,
    declaration_site: SyntaxNodeId,
    ordinal: u32,
}

struct FlowPointId {
    cfg: ControlFlowGraphId,
    block: BasicBlockId,
    statement_index: u16,
}

struct ExpressionFact {
    expression: SyntaxNodeId,
    flow_point: FlowPointId,
    knowledge: TypeKnowledge,
    obligations: Arc<[TypeObligationId]>,
    effects: EffectSummary,
    reachability: Reachability,
    provenance: Arc<[FactOrigin]>,
}

struct FlowEnvironment {
    bindings: PersistentMap<BindingId, BindingFact>,
    path_conditions: PathConditionSet,
    reachability: Reachability,
}
```

`BindingFact` separates declared type, current flow type, initialization state, mutability, ownership/capture facts once specified, and evidence. Shadowed variables obtain distinct `BindingId`s even if they render identically. Each expression fact records range/origin and query revision. Do not use `HashMap<String, TypeKnowledge>` as the production semantic environment; it remains suitable only for isolated bootstrap tests.

`Reachability` is at least `Reachable`, `UnreachableByNever`, `UnreachableByControl`, and `Unknown`. It cannot be inferred solely from an expression returning `Never` because exceptions, dynamic calls, finally/defer semantics, and malformed source may affect control-flow once language semantics define them.

### 3.2 Constraint and solver facts

Replace direct local bindings with attributed constraints:

```rust
enum TypeObligation {
    Equal { left: InferenceTerm, right: InferenceTerm, origin: Origin },
    Subtype { source: InferenceTerm, target: InferenceTerm, mode: AcceptanceMode, origin: Origin },
    HasMember { receiver: InferenceTerm, selector: Selector, side: DispatchSide, expected: MemberExpectation, origin: Origin },
    Callable { callee: InferenceTerm, arguments: Arc<[ArgumentFact]>, expected_return: Option<InferenceTerm>, origin: Origin },
    WellKinded { term: InferenceTerm, expected: KindId, origin: Origin },
}

enum SolveStatus {
    Solved,
    Underconstrained,
    Ambiguous,
    Inconsistent { failure: RelationFailure },
    BlockedByDynamicBoundary,
    RecursiveFixpoint,
    BudgetExceeded,
    Cancelled,
}
```

An inference session owns fresh variables, bounds, substitutions, dependency reads, active constraints, and cancellation/budget counters. It implements occurs check before binding variable `α` to a term containing `α`; it normalizes substitutions lazily with path compression only inside the session; it reports an origin path for conflicts. Constraints are solved by worklist and dependency/SCC structure, not one source-order pass. A `HasMember` constraint delegates to the unified dispatch query. Constraint solving does not directly mutate global `TypeStore` facts or publish an unsolved variable.

### 3.3 Dispatch, callable, protocol, and proof facts

```rust
struct DispatchRequest {
    receiver: CanonicalTypeId,
    selector: Selector,
    side: DispatchSide,
    type_arguments: Arc<[CanonicalTypeId]>,
    call_shape: CallShape,
    lookup_start: LookupStart,
    mode: DispatchMode,
}

enum DispatchResult {
    Found(InstantiatedMemberSurface),
    Missing(DispatchFailure),
    Ambiguous(Arc<[CandidateSurface]>),
    DynamicBoundary(DynamicDispatchReason),
    Blocked(DispatchBlocker),
}

struct CallableSummary {
    callable: CallableId,
    interface_revision: InterfaceRevision,
    parameters: Arc<[BindingFact]>,
    result: TypeKnowledge,
    effects: EffectSummary,
    throws: ThrowSummary,
    dependencies: Arc<[DependencyKey]>,
    completion: SummaryCompletion,
}

enum ProofResult {
    Proven { evidence: ProofEvidence },
    Disproven { witness: CounterexampleOrPath },
    Unknown { reason: ProofUnknownReason },
    Cancelled,
    BudgetExceeded,
}
```

`LookupStart` represents normal receiver lookup, lexical superclass lookup for `super`, an explicit class-object/metaclass start, or an explicitly dynamic/reflection start. It avoids overloading `Object` as a superclass guess. `InstantiatedMemberSurface` carries the declaration chosen, declaration-side identity, inherited path, parameter substitution, receiver type application, effective callable signature, and provenance. It must be immutable/cacheable by actual request input and interface revisions.

Protocol conformance is a separate query keyed by `(candidate type/application, protocol, interface revision, mode)`. It may be nominal first, structural only after the protocol design ratifies exact member/variance/associated-type rules, and must return blocked/ambiguous/inconsistent outcomes rather than “find a similarly named method.”

`ProofResult` is intentionally distinct from `RelationOutcome` and `TypeKnowledge`. It makes future clients report the difference between a counterexample and an unavailable proof because a reflective/native/effectful assumption was not modeled.

## 4. Bidirectional expression checking

### 4.1 Judgments

The checker implements the mutually recursive judgments:

```text
Σ; Δ; Γ; Φ ⊢ e ⇒ F             synthesize expression fact F
Σ; Δ; Γ; Φ ⊢ e ⇐ τ ⇒ F         check expression e against expected type τ
Σ; Δ; Γ; Φ ⊢ stmt ⇒ (Γ', flow) check statement and outgoing environment
```

`Σ` is the immutable semantic snapshot/header environment; `Δ` is generic binder/substitution context; `Γ` is flow environment; `Φ` contains accepted path conditions/effects/reachability. Synthesis returns an evidence envelope plus obligations. Checking first pushes an expected type into literals, collection elements, closures/callables, return expressions, assignments, arguments, and branch joins; it falls back to synthesis then applies the named acceptance relation. Neither judgment treats `Unknown` as a positive subtype proof. A known mismatch gets a diagnostic only where authoritative evidence makes rejection sound; a known missing member is diagnosed even if the surrounding expression result is then unknown for recovery.

### 4.2 Core expression rules

- **Bindings and assignment.** Annotated bindings lower their declared type. Initializer checks against it. Unannotated bindings synthesize/infer according to mode and retain `Unknown` if evidence is insufficient. Assignment checks declared mutability and acceptance, then updates the flow type only where assignment semantics allow narrowing/widening; it never changes the declared contract silently.
- **Literals and collections.** Expected `List[T]`/`Map[K,V]` pushes `T`/`K,V` into elements. Without expectation, collection literals create inference variables and constraints for every ordinary element and spread. Empty collection literals remain underconstrained unless an annotation/context resolves them; they do not publish an arbitrary concrete element type. Tuple/record fields preserve labels/order/mutability; unknown elements do not become `Unit` merely to manufacture a complete tuple.
- **Calls and sends.** Synthesize receiver/callee then resolve dispatch/call surface; instantiate generic parameters; check positional/keyword labels, arity/rest rules, every argument against effective parameter type, and expected return type. An unqualified callable reference is typed through its own callable surface, not by ignoring arguments after finding its return type. Binary/unary forms are ordinary message sends with the same argument checking path.
- **Fields/getters/setters/indexing.** They all lower to dispatch requests whose side and selector form match runtime semantics. Indexing is not hardcoded to `List`/`Map`; builtins may have optimized registered surfaces, but the semantic operation remains member/call resolution.
- **Blocks/closures.** Build an inner binding/flow context and check an expected callable signature bidirectionally. Without it, allocate an inference callable with parameter/result/effect variables; generalization policy is explicit and bounded. Capture/effect semantics are recorded for future proving.
- **Returns/throws.** Check a return against callable expected result and mark successor flow unreachable. A tail expression does not replace a stated return contract. `throw`/terminal control expressions synthesize `Never` only after their semantics establish no normal continuation.

### 4.3 Branching and loops

For `if`, `if let`, type tests, match/pattern forms, and boolean predicates with specified semantic refinements:

1. Synthesize the condition/pattern source once.
2. Derive branch path conditions from a registered refinement rule; bind the pattern only to the matching refined component, not raw source value.
3. Analyze each reachable branch from a persistent base environment.
4. Join outgoing facts at the post-dominator: type joins form unions only where that is the specified least upper bound; initialization is definitely assigned only if every reachable predecessor initializes it; effect/throw summaries join conservatively.
5. Diagnose an impossible branch if the contradiction is established by authoritative static facts, otherwise retain a blocked/unknown branch status.

`while`, `while let`, loops with back edges, and recursive callable groups need a fixed point. The initial loop head is the incoming environment; each iteration transfers through the body and joins with previous head; stop on semantic equality or a documented iteration/node budget. On budget exhaustion, widen selected facts (for example, to a declared supertype or known union cap) and record `BudgetExceeded`/widening provenance. Do not run a single iteration then claim loop refinement, and do not run without a bound. The reusable LSP worklist mechanism is useful inspiration, but formal type joins/relations must come from Specification 01.

### 4.4 Callable summaries and recursion

Build a call dependency graph from resolved callable surfaces. Summary computation operates per SCC. A callable with an explicit result annotation can seed recursive checking from that contract. A recursive group with no sufficient annotations returns `Underconstrained`/`RecursiveFixpoint` according to the language policy; it must not accidentally generalize a speculative body type as an exported signature. Summary equality ignores volatile revision/timing fields and compares semantic facts/dependencies only. Completion status is published alongside every summary.

## 5. Semantic dispatch and object model

### 5.1 Ordered lookup algorithm

For an authoritative known receiver type, dispatch proceeds in this order:

1. Validate receiver kind/type application and call shape.
2. Choose lookup start: receiver’s nominal/application surface for normal sends; lexical direct superclass of the enclosing class for `super`; class-object/metaclass surface for class-side sends; explicit boundary for reflective/dynamic sends.
3. Search the declared surface, then instantiated inherited superclass surfaces using a substitution environment at each edge. Detect inheritance cycles using typed header graph state.
4. Search protocol extension/default surfaces only when protocol design specifies their precedence and coherence rules. Do not add ad hoc “has same selector” structural fallback.
5. Resolve candidates by Phalcom selector/call-shape rules. If overloads are introduced later, collect candidates and solve arguments/expected result with explicit ambiguity diagnostics; do not select first registration order.
6. Return one `DispatchResult` with origin path, substitution, and diagnostics/evidence.

For `Dynamic`, opaque native, `perform`, reflective selector construction, and declared `doesNotUnderstand` paths, return `DynamicBoundary` with reason/capabilities. Strict mode can require an annotation, runtime contract, explicit cast/check, or trusted native declaration at that boundary. A normal known receiver and missing exact/inherited selector returns `Missing`, not generic `Unknown`; recovery may subsequently use unknown knowledge for continuation.

### 5.2 Generic instantiation and `Self`

An inherited member surface is viewed through a receiver-specific substitution. If `Child[X] <: Parent[F[X]]`, looking up `Parent` member `m: T -> U` on `Child[A]` requires a composed substitution that maps parent binders through `F[A]`, respects owner-qualified parameter IDs, validates kind/bounds, and preserves variance declared at the relation level. Caching must include the receiver application and interface revisions, not merely `CallableId`.

`Self` is not a synonym for lexical class nominal. Its exact meaning needs a ratified design: likely an anchored self type whose substitution differs in instance/class/constructor contexts. Constructor typing must decide whether `new` returns `Self`, whether factory class-side methods have an anchored receiver, and how subclasses inherit constructors. Until ratified, report an unsupported annotation/dispatch form instead of approximating `Self` with `Object` or current class nominal.

### 5.3 Protocols, aliases, ADTs, and exhaustiveness

Protocol conformance, structural rules, inheritance/composition, associated types, intersection types, ADTs, and exhaustiveness depend on later typing documents. This spec sets their integration boundary:

- Headers expose protocol/ADT/type-alias declarations and references even if their bodies/checkers are feature-gated.
- Conformance has its own cached result and diagnostics, not an implicit branch of dispatch.
- Pattern matching consumes explicit ADT/tag/constructor facts. Exhaustiveness is a coverage query over finite known variants; it reports `Unknown` for dynamic/open/opaque domains, never “exhaustive” by absence of current cases.
- Type aliases have a specified transparency/equivalence policy before they affect relation canonicalization.

## 6. Native, reflection, effects, and dynamic boundaries

Native types and operations enter through versioned typed descriptors from Specification 01. Each callable/member has an effect summary contract, initially conservative:

```text
Pure | Reads(receiver/field) | Writes(receiver/field) | Allocates |
Throws(type) | DynamicSend | Reflects | NativeOpaque | UnknownEffect
```

Effects compose in evaluation order and flow into callable summaries. This is not an attempt to type every runtime detail immediately; it is the minimum information that prevents a future prover from treating a `perform` call or opaque native operation as a pure total function. Trusted native metadata may give strong evidence only under version/ABI identity. An unsupported native annotation emits an opaque boundary with provenance rather than hardcoding a misleading `Dynamic` special case.

Reflection distinguishes an explicit type/kind inspection bridge from runtime class inspection. A value from `x.class` can participate in runtime code; it does not establish `x : T` statically unless a specified reflective assertion/check returns a proof-producing refinement. `perform` accepts a runtime selector and is dynamically dispatched unless selector/value knowledge plus a specified safe reflective API proves a finite target set.

## 7. Proof-ready foundation

### 7.1 Scope

This phase does **not** implement a general SMT backend, symbolic executor, separation logic, or proof of current contract decorators. It creates facts/interfaces so a future prover can be soundly integrated rather than reverse-engineering semantics from diagnostics.

The current `@requires`/`@ensures` implementation remains runtime weaving. Preserve its present behavior and document any result-binding/metadata divergence separately. A runtime guard failing is a runtime observation, not a proof counterexample for all calls; a guard passing once is not a proof.

### 7.2 Contract and proof IR requirements

Lower a future-proof subset of contracts into an explicit, versioned semantic product:

```rust
struct ContractFact {
    owner: CallableId,
    kind: ContractKind,                 // requires, ensures, invariant later
    expression: ContractExprId,
    bindings: ContractBindingMap,
    source_range: TextRange,
    snapshot: SemanticRevision,
}

struct VerificationCondition {
    callable: CallableId,
    path: ControlFlowPathId,
    assumptions: Arc<[LogicalFactId]>,
    goal: LogicalFactId,
    heap_frame: HeapFrameSummary,
    effects: EffectSummary,
    trusted_boundaries: Arc<[TrustBoundary]>,
}
```

Only contract expressions with defined typed semantics lower into logical facts. A contract that calls dynamic/reflection/opaque native code, reads mutable aliasable state without frame knowledge, or exceeds resource limits returns `ProofResult::Unknown` with precise reason. Proof caching keys include callable body revision, typed headers, contract revision, effect/native model revision, solver configuration, and trust assumptions. Type snapshot revision alone is insufficient.

The smallest initial proof service may prove simple local arithmetic/nullability/type-refinement obligations over pure expressions and return `Unknown` for every unsupported feature. It must report a model/witness only when the backend’s model maps to Phalcom semantics; otherwise use `Unknown`, not fabricated `Disproven`.

### 7.3 Relationship to gradual/dynamic code

Static type acceptance and proof success differ. Code behind `Dynamic` may typecheck in `check` mode but creates a proof boundary. Strict mode may require explicit runtime validation/cast/contract at the boundary. Trusted native facts and runtime checks become assumptions marked in evidence, never invisible facts. This preserves gradual adoption without invalidating claims made by a future verifier.

## 8. Implementation sequence

### Phase 1 — introduce stable semantic IR and facts

- Assign binding and flow point identities after parser recovery; construct CFGs for current structured control flow.
- Replace string-only local environment paths with persistent binding maps while keeping test adapters for one-program fixtures.
- Introduce `ExpressionFact`, `FlowEnvironment`, reachability, and effect shells. Publish them only through Specification 02 snapshots.

**Exit criterion:** a nested shadowing/assignment fixture has distinct binding identities and a diagnostic can name the correct declaration/range after branches merge.

### Phase 2 — constraints and bidirectional core

- Upgrade `TypedExpression` to emit attributed obligations; move inference variables into session-owned solver state.
- Implement expected-type propagation for annotation initializers, returns, arguments, literals, collections, and blocks.
- Add occurs checks, worklist/SCC solve, failure paths, cancellation, and explicit underconstrained/ambiguous outcomes.
- Remove or isolate hardcoded `List.add`/indexing inference shortcuts behind descriptor-driven dispatch tests.

**Exit criterion:** typed and inferred collection/call fixtures use one obligation engine; empty literals and recursive inference report a documented non-success state instead of arbitrary type selection.

### Phase 3 — CFG fixed point and summaries

- Implement branch refinement registry and path condition facts for ratified predicates/patterns.
- Implement loop and recursive-callable SCC convergence with deterministic join/widen/budget policy.
- Add definite assignment and `Never` reachability diagnostics only where control semantics prove them.

**Exit criterion:** a loop/recursive fixture reaches the same stable diagnostics across repeated runs; nonconvergence/cancellation is deterministic and does not publish a stale partial summary.

### Phase 4 — authoritative dispatch

- Replace local exact surface resolver with snapshot-backed `DispatchRequest`/`DispatchResult` query.
- Implement inherited lookup/substitution, real `super` lookup start, class-side versus instance-side surfaces, and precise known-missing diagnostics.
- Add dynamic/reflection/native boundary reasons; add generic callable instantiation after Specification 01 type applications are live.

**Exit criterion:** inherited member and `super` fixtures resolve the same runtime selector surface with correct type substitution; missing known members diagnose while dynamic sends remain explicit boundaries.

### Phase 5 — protocols and proof readiness

- Gate protocol conformance, `Self`, ADT/exhaustiveness, and bounds on ratified design documents. Implement one feature at a time with independent relation/query tests.
- Emit typed contract/effect/proof-IR facts and `ProofResult` infrastructure. Begin with `Unknown` for unsupported boundaries and no public claim of whole-program proof.

**Exit criterion:** a contract fixture can show the exact reason a proof is unknown (dynamic call, opaque native, alias/effect gap, budget) without changing runtime decorator behavior.

## 9. Test matrix and acceptance evidence

| Area | Required tests |
|---|---|
| Bindings/flow | Shadowing, mutability, assignment after branch, definite assignment, unreachable code, `Never`, malformed recovered syntax. |
| Bidirectional checking | Expected literal/collection types, empty collections, callable return/arguments, labels/rest, type mismatch evidence ranges. |
| Solver | Occurs check, cyclic constraints, bounds, underconstrained/ambiguous status, deterministic worklist order, cancellation/budget. |
| CFG/SCC | Branch join, pattern refinement, loop fixed point/widening, recursion annotation seed, mutually recursive callables, summary invalidation. |
| Dispatch | Instance/class side, inheritance, `super`, generic inheritance substitution, setter/getter/indexing, missing/ambiguous/dynamic/opaque-native results. |
| Protocol/ADT | Feature-gated nominal conformance, later structural counterexamples, protocol cycle, open/dynamic exhaustiveness unknown. |
| Effects/proofs | Pure local obligation, thrown/dynamic/native/reflective unknown, contract source bindings, result-name regression, cache revision change. |
| Regression | Existing Phase-2 expression tests retained and recast as formal snapshot/diagnostic tests; no false-error behavior explicitly tested by evidence state. |

Use small executable semantic examples alongside property/fuzz tests. Fuzzing must constrain generated programs to parser/runtime-valid constructs and assert termination/no panic/deterministic terminal status; it cannot claim soundness by comparing arbitrary generated text to itself. Add differential tests only where a tiny reference evaluator or manually derived typing judgment exists.

## 10. Pyrefly transfer: direct, adapted, rejected

**Take directly:** explicit query states; type-query demand/cycle handling; SCC worklists; stable summary comparison excluding volatile revision fields; cancellation propagation; bounded convergence; provenance-rich diagnostics; dependency-driven summary invalidation. Existing LSP analysis already demonstrates some of these operational ideas in `solve_workspace_callables`; formal typing should reuse the discipline, not the `ValueShape` domain.

**Adapt:** use Phalcom CFG semantics, message sends, selector labels, class/metaclass side, inheritance, native surfaces, and explicit dynamic/reflection behavior. Let formal relation and dispatch queries feed worklists. Use an owned Rust worker and immutable snapshots, then measure SCC-local parallelism as specified by the transfer dossiers.

**Reject:** Python narrowing heuristics as language law, descriptor/attribute lookup as dispatch model, treating a missing member as generic unknown, acceptance of all dynamic code as proof-safe, unbounded recursive inference, and a single “type result” enum that erases unknown/dynamic/blocked/proof states. Reject a copied Python contract/prover expectation: Phalcom proofs must encode Phalcom effects and runtime semantics.

## 11. What this must not preclude

- Higher-kinded/generic constraints whose kind/solver interfaces were established in Specification 01.
- `Self`, class-side constructors, protocol composition, F-bounds, associated types, overloads, ADTs, intersections, and refined effects as independent later work units.
- IDE runtime-shape assistance remaining useful even when formal type facts are incomplete.
- A future static prover with an external SMT/SAT/abstract-interpretation backend, proof certificates/models where sound, and strict resource/cancellation controls.
- Runtime compatibility for dynamic dispatch and reflective code, with an explicit path to safer typed APIs rather than a forbidden-language subset.

## 12. Risks and decisions required before implementation

The largest technical risk is attempting inherited generic dispatch before substitutions, variance, type-parameter identity, and typed interface revisions are complete. That produces cache poisoning and erroneous member signatures. The largest semantic risk is assigning `Self`/constructor/class-side meaning by analogy rather than Phalcom’s class-object semantics. The largest user-experience risk is strict mode turning every current unknown into an error and thereby collapsing gradual adoption; strictness must operate on named obligations/evidence, not one catch-all diagnostic.

Before implementation, ratify: evaluation order/effect semantics relevant to sends and blocks; pattern and type-test refinement rules; `super` lookup semantics; `Self` and constructor return policy; protocol nominal/structural/coherence rules; mutation/alias capability model; first proof scope and trusted native/dynamic assumptions. If one is not decided, preserve a feature gate and `Unknown`/`Blocked` status with reason rather than smuggling a rule into checker control flow.
