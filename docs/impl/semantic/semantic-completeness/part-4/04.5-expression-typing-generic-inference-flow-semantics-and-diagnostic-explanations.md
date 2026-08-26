# 04.5 — Expression Typing, Generic Inference, Flow Semantics, Type Derivations, and Diagnostic Explanations

**Date:** 2026-08-23
**Status:** Repository-grounded implementation specification; intended to begin after the Spec 04 integration gate and to land before general Spec 05 verification
**Authority:** executable-expression typing, bidirectional checking, semantic call resolution, ordinary method-generic inference, flow-sensitive type refinement, compiler-owned control-flow facts, type-derivation evidence, diagnostic causality, and semantic-diagnostic presentation contracts
**Primary owner:** `phalcom-semantic`
**Supporting owners:** `phalcom-lsp` for consumer migration and tooling presentation, `phalcom-core` for CLI/compiler diagnostic adaptation, a proposed VM-free `phalcom-diagnostics` crate for shared terminal presentation substrate
**Hard dependencies:** 01 — compiler-owned typing architecture; 01.5 — canonical generic type semantics; 04 — user-facing type syntax and lowering
**Coordinates with:** 03.5 — canonical core/native semantic surface; 05 — effects, exits, contracts, termination, verification conditions, and proof
**Repository snapshot inspected:** `aureat/phalcom-lang` `main` at `edbced8914b924f95f4dbc83508c93ca77cd36ce` (`extend runtime typing metadata and reflection`, 2026-08-22)
**Source Spec 04 basis:** `04-user-facing-type-syntax-and-lowering-REVISED.md`, revision dated 2026-08-22
**Non-goals:** redefining type syntax; changing selector identity or runtime dispatch; redefining canonical generic/type-lambda semantics; introducing overload-by-type dispatch; dependent types; general arithmetic theorem proving; effect inference; contract verification; termination proving; proof artifacts; a solver-backend commitment; changing ordinary runtime object layout; making LSP advisory shape inference compiler-authoritative

---

## 0. Executive implementation contract

Spec 04 establishes how source text becomes canonical type forms, generic signatures, `where` constraints, type lambdas, generic superclass templates, and explicit lowering outcomes. Spec 04.5 starts **after that publication boundary** and answers the next question:

> Given canonical declarations and an executable source body, what can the compiler soundly know about every expression and reachable program point, how does it infer omitted local type information, and how can it explain those conclusions to a human?

The target pipeline is:

```text
Spec 04
source syntax
    ↓
recoverable AST / TypeSyntax
    ↓
canonical declaration + callable signatures
    ↓
────────────────────────────────────────────────────────
Spec 04.5
callable body
    ↓
compiler-owned flow graph
    ↓
bidirectional expression analysis
    ↓
receiver specialization + call resolution
    ↓
session-local generic inference
    ↓
flow refinement + joins + bounded fixed points
    ↓
type derivations / explanation graph
    ↓
structured semantic diagnostics
    ↓
CLI / LSP / REPL presentation
    ↓
────────────────────────────────────────────────────────
Spec 05
effects / exits / contracts / termination / VCs / proofs
```

The central implementation law is:

> **Every compiler-authoritative type conclusion that may matter to a user must be derivable from structured semantic evidence. Diagnostics are projections of that evidence; they are not independently reconstructed guesses.**

The second central law is:

> **Inference variables are solver-local reasoning entities, not canonical types.**

Normatively:

```text
InferVarId ≠ TypeId
```

The current repository still interns `TypeData::Infer(InferVarId)` into `TypeStore`. This specification defines the staged migration that removes that representation from publishable canonical type state.

The third central law is:

> **Compiler-owned flow semantics live in `phalcom-semantic`.**

The current LSP already contains a sophisticated advisory flow engine (`phalcom-lsp/src/semantic/flow.rs`, `infer.rs`, `facts.rs`). Its algorithms are valuable evidence and migration material, but its `ValueShape` model explicitly describes advisory runtime shapes rather than the language's canonical static types. 04.5 therefore adapts its proven flow algorithms while preventing a permanent second formal type checker.

---

# Part I — Authority and semantic boundaries

## 1. What 04.5 owns

04.5 is authoritative for the following ordinary static semantics:

1. expression type synthesis;
2. contextual/bidirectional type checking;
3. lexical binding state during body analysis;
4. receiver type interpretation for sends;
5. compiler-semantic member/selector resolution;
6. specialization of class-owned generic parameters for an applied receiver;
7. owner-relative `Self` substitution during member viewing;
8. instantiation and local inference of method-owned generic parameters;
9. expected-result participation in local inference;
10. callable/block contextual typing;
11. argument/label/rest compatibility;
12. `where`-constraint validation after local generic solution;
13. union-receiver call validity and result joining;
14. `Unknown`, `Dynamic`, blocked, cancelled, budget-exceeded, and invalid propagation policy;
15. flow-sensitive narrowing;
16. branch reachability and merge rules;
17. stable declared type versus current flow type;
18. conservative fact invalidation on mutation/opaque calls;
19. bounded loop fixed points and widening;
20. type-derivation/explanation provenance;
21. diagnostic causality and cascade suppression;
22. structured diagnostic notes/help/fixes;
23. compiler-owned semantic facts consumed by CLI/LSP/REPL;
24. incremental publication of callable-body semantic products.

## 1.1 What 04.5 does not own

| Concern | Authority |
|---|---|
| Source type grammar and parser recovery | Spec 04 |
| Type-form lowering and generic binder publication | Spec 04 |
| Canonical generic semantics, kinding, substitution, type lambdas | Spec 01.5 |
| Runtime metadata/reification | Specs 02 / 02.5 |
| Runtime reflection API | Spec 03 |
| Native/core implementation provenance and canonical native surface | Spec 03.5 |
| Open-record row solving | Spec 05 |
| Effect summaries | Spec 05 |
| Exceptional/control exit summaries | Spec 05 |
| Contracts as verification inputs | Spec 05 |
| Verification-condition generation | Spec 05 |
| Termination and `@total` proof | Spec 05 |
| General proof artifacts/backends/trust | Spec 05 |
| Runtime selector identity and dispatch | Existing runtime semantics |

04.5 may consume later Spec-05 facts as optional precision improvements, but **04.5 cannot require Spec 05 to produce a sound baseline result**. In particular, unknown effect information means flow analysis conservatively invalidates mutable projection facts; it does not block ordinary type checking.

## 1.2 No type-directed runtime dispatch

Static call analysis may select the semantic declaration that ordinary runtime dispatch would reach for a known receiver type. It must never change runtime lookup keys.

Forbidden consequences include:

```text
selector + inferred type arguments
selector + expected return type
selector + generic specialization
```

as runtime selector identities.

The runtime remains selector/object-model driven. Static semantics merely prove or fail to prove that the ordinary send is valid for all statically reachable receiver alternatives.

---

# Part II — Repository archaeology and current gaps

## 2. Current checker architecture

At the inspected snapshot, `phalcom-semantic/src/checker/` contains:

```text
call.rs
context.rs
declaration.rs
expression.rs
mod.rs
result.rs
statement.rs
typed_expr.rs
```

`checker/mod.rs` exposes the Phase-2-style API:

```rust
check_program(...)
synthesize_expr(...)
synthesize_typed_expr(...)
check_statement(...)
match_callable_arguments(...)
```

and constructs one mutable `CheckingContext` for a parsed program.

This implementation is useful groundwork, but its ownership boundaries are too coarse for the post-Spec-04 semantic engine.

### 2.1 `CheckingContext` mixes lexical state, inference, dispatch, and diagnostics

Current `phalcom-semantic/src/checker/context.rs` conceptually contains:

```rust
pub struct CheckingContext<'a> {
    pub store: &'a mut TypeStore,
    pub hierarchy: &'a dyn TypeHierarchy,
    pub resolver: &'a dyn TypeResolver,
    pub declarations: &'a DeclarationTypeTable,

    pub current_module: ModuleId,
    pub current_class: Option<DeclarationId>,
    pub current_side: DispatchSide,
    pub expected_return: Option<TypeKnowledge>,

    pub local_envs: Vec<LocalEnv>,
    pub dispatch: SurfaceDispatchResolver,
    pub solver: LocalConstraintSolver,
    pub diagnostics: Vec<SemanticDiagnostic>,
}
```

`LocalEnv` is keyed by `String`, even though `phalcom-semantic/src/identity.rs` already defines snapshot-local `BindingId`.

Target change:

- lexical identity becomes `BindingId`-based;
- static declaration environment and changing flow state become separate concepts;
- local inference becomes a bounded session rather than a context-global solver;
- diagnostics are emitted from structured failures and explanation IDs;
- dispatch resolution returns stable callable identity plus a semantic view rather than only a copied signature.

### 2.2 `TypedExpression` is already the correct seam, but underpowered

Current `checker/typed_expr.rs` contains:

```rust
pub struct TypedExpression {
    pub knowledge: TypeKnowledge,
    pub denotation: Option<SemanticDenotation>,
    pub dispatch_lookup: DispatchLookup,
    pub constraints: Vec<TypeConstraint>,
    pub provenance: EvidenceSet,
}
```

This is the predecessor of the target `ExpressionAnalysis`.

The current shape already proves that the checker needs more than a `TypeId`. 04.5 extends this concept with:

- stable expression identity;
- explicit analysis status;
- structured explanation ID;
- optional call-resolution ID;
- flow effects/refinements needed by enclosing control flow.

The implementation may preserve `TypedExpression` as a compatibility alias during migration, but the final API must no longer imply that “typed expression” means only a synthesized type.

### 2.3 Empty-collection inference currently pollutes `TypeStore`

Current `expression.rs` allocates an inference variable for an empty list by calling:

```rust
let (_var, infer_ty) = ctx.solver.fresh_var(ctx.store);
```

`LocalConstraintSolver::fresh_var` then calls:

```rust
store.infer(var)
```

and `TypeStore::infer` interns:

```rust
TypeData::Infer(InferVarId)
```

into the canonical store.

This violates the target lifetime boundary. Solver-local variables:

- are not publishable semantic types;
- must not participate in stable metadata fingerprints;
- must not leak into immutable snapshots;
- must not survive cancellation or an abandoned inference branch;
- must not become observable through reflection.

04.5 therefore stages out `TypeData::Infer`.

### 2.4 Calls currently validate an already-concrete signature

Current `checker/call.rs`:

1. synthesizes every argument independently;
2. checks assignability against the parameter's already-resolved `TypeKnowledge`;
3. emits a flat `ArgumentMismatch` string;
4. returns the signature's preexisting return type.

It does not yet own:

- method generic instantiation;
- expected-result constraints;
- contextual closure typing;
- generic `where` solving at call sites;
- inference ambiguity;
- underconstrained generic results;
- structured reasoning explaining why an argument type was expected.

This file becomes the primary call-resolution/inference integration seam.

### 2.5 Receiver specialization exists, but callable inference is not separated from it

`CheckingContext::resolve_dispatch` already recognizes:

```text
ClassObject
Nominal
Applied
```

receiver types, resolves the declaration/side, and applies `substitution_for_applied` to parameter/return types.

This is valuable and must be retained conceptually.

However, receiver specialization and method-local inference are distinct semantic environments.

For:

```phalcom
class Box<T> {
  map<U>(f: (T) -> U) -> Box<U> { ... }
}
```

on `Box<User>`, the checker must maintain:

```text
receiver environment:
    T := User

method inference environment:
    U := ?u0
```

They compose; they are never the same map.

### 2.6 Statement analysis overwrites declaration information

Current assignment handling:

1. checks an RHS against the currently stored binding fact;
2. emits a mismatch only on `Assignability::Refuted`;
3. overwrites the binding fact with the RHS fact.

This cannot correctly represent:

```phalcom
let x: Number = 1
x = 2.0
```

where the stable admissibility requirement is `Number`, but the current flow type may be `Int` and later `Float`.

04.5 introduces:

```text
binding declared/base knowledge
≠
binding current flow knowledge
```

### 2.7 Branching is result-type aware, not path-state aware

Current `IfLet` analysis checks the two scopes and joins their **expression result types** with `TypeStore::union`, but the compiler checker has no general persistent `FlowState` containing branch facts.

Comparisons and membership operations similarly analyze operands and return `Bool`; the truth of a condition does not become a compiler-owned path fact.

This is the primary flow-analysis gap.

### 2.8 `For` bodies are currently seeded with `Dynamic`

Current `statement.rs` synthesizes each iterable but binds loop pattern lanes using:

```rust
TypeKnowledge::Dynamic(DynamicReason::ExplicitEscape)
```

This loses known collection element types. 04.5 replaces that fallback with formal element inference where semantics are known, and preserves `Unknown`/`Dynamic` distinctions where they are not.

### 2.9 Relation outcomes are richer than checker consumers

`types/relation.rs` already distinguishes:

```text
Assignable
Refuted
DynamicBoundary
Blocked
Cancelled
BudgetExceeded
InternalFailure
Uncertain
```

and the lower-level `RelationOutcome` distinguishes:

```text
Proven
Refuted
DynamicBoundary
Blocked
Cancelled
BudgetExceeded
InternalFailure
```

Yet current body/call checking commonly branches only on `Refuted`.

04.5 must consume the entire result algebra. In particular:

- `Cancelled` never becomes a mismatch;
- budget exhaustion never becomes a mismatch;
- `DynamicBoundary` never becomes a false compile-time proof;
- `Blocked` carries dependency/cycle information;
- `Refuted` is the only ordinary type contradiction.

### 2.10 `SemanticDb` is ready for callable-scoped products

`phalcom-semantic/src/db/key.rs` already defines:

```rust
QueryKey::CallableBody(CallableId)
```

and `SemanticDb` already supports:

- revisions;
- fingerprints;
- dependency recording;
- reverse invalidation;
- blocked/cancelled states;
- query metrics.

04.5 must use this database rather than adding a second body-analysis cache.

### 2.11 Workspace Phase H is still module-recursive

`phalcom-semantic/src/workspace.rs` Phase H currently creates a `CheckingContext` per module and recursively walks all bodies.

The target is:

```text
module signatures published
        ↓
CallableBody(CallableId) query
        ↓
CallableAnalysis
        ↓
diagnostic aggregation + snapshot publication
```

This makes incremental rechecking naturally callable-scoped.

### 2.12 LSP already has a separate advisory flow engine

This is the largest convergence issue.

`phalcom-lsp/src/semantic/flow.rs` already has:

```rust
FlowState
StatementFlow
ReturnEvidence
ResolvedCall
AnalysisEvent
BlockEffects
SurfaceFlowAnalysis
```

and implements:

- branch joins;
- `is` / `is!` type-test narrowing;
- reachability;
- returns/breaks/continues/throws;
- bounded loop fixed points;
- widening;
- assignment propagation;
- call-event collection.

`phalcom-lsp/src/semantic/infer.rs` adds:

- callable worklists;
- parameter contribution propagation;
- bounded interprocedural rounds;
- cooperative cancellation;
- dirty-callable solving.

`phalcom-lsp/src/semantic/facts.rs` explicitly states that `ValueShape` is an **advisory runtime shape**, not a language type.

Therefore 04.5 adopts this rule:

> **Take the flow algorithms; do not promote `ValueShape` into the formal type system.**

Formal compiler facts move into `phalcom-semantic`. LSP `ValueShape` may remain only for genuinely tooling-specific runtime-shape intelligence that has no formal static-type counterpart.

---

# Part III — Semantic laws

## 3. Non-negotiable laws

### LAW-EXPR-1 — Static analysis never changes runtime semantics

Expression typing, flow facts, generic inference, and explanations do not enter:

- selectors;
- runtime dispatch keys;
- inline-cache keys;
- class/metaclass identity;
- object layout;
- allocation layout.

### LAW-EXPR-2 — Inference variables are session-local

No `InferVarId` may appear in a published semantic snapshot, metadata record, runtime descriptor, reflection result, or stable fingerprint.

### LAW-EXPR-3 — Unknown is not Dynamic

```text
Unknown
```

means current evidence cannot establish a static fact.

```text
Dynamic
```

means the program intentionally crossed a dynamic/open-world boundary.

The checker must not convert one into the other merely to continue.

### LAW-EXPR-4 — Failure classes remain distinct

```text
Refuted
Blocked
Cancelled
BudgetExceeded
InternalFailure
DynamicBoundary
Underconstrained
Ambiguous
```

are not interchangeable.

Only sound contradictions may reject code as type mismatches.

### LAW-EXPR-5 — Expected types constrain inference, not dispatch identity

Expected context may solve method/local generic variables. It cannot select a different runtime selector.

### LAW-EXPR-6 — Receiver specialization precedes method inference

Class-owned generic bindings come from the receiver. Method-owned generic bindings are solved locally for the call.

### LAW-EXPR-7 — No generic defaulting to `Object` or `Dynamic`

When local inference cannot determine a parameter, the result is explicit underconstraint/ambiguity unless a separately ratified language default applies.

### LAW-EXPR-8 — Flow facts are path-scoped

A fact established in one branch is valid only on program points dominated by that branch unless it survives a defined join.

### LAW-EXPR-9 — Joins retain only sound information

At a control-flow merge:

- base declarations remain stable;
- current types join conservatively;
- logical facts survive only if valid on every reachable incoming path.

### LAW-EXPR-10 — Mutation invalidates dependent facts

A flow fact may not survive a write or opaque operation that could make its referenced value stale.

### LAW-EXPR-11 — 04.5 records direct predicates; 05 proves general implications

04.5 may record:

```text
amount > 10
```

because a branch establishes it.

04.5 does not need to prove:

```text
amount > 10 ⇒ amount > 0
```

That implication belongs to Spec 05 verification.

### LAW-EXPR-12 — Type derivations are proof-irrelevant to canonical identity

Two expressions with the same canonical `TypeId` do not become different types because they have different explanation graphs.

### LAW-EXPR-13 — Diagnostics come from semantic causes

A human diagnostic is generated from a structured contradiction/status and its explanation graph. The checker must not maintain a second, contradictory prose-only interpretation of the analysis.

### LAW-EXPR-14 — Root-cause suppression is semantic

Dependent failures caused by an already-invalid expression do not become independent primary errors unless they add genuinely new information.

### LAW-EXPR-15 — LSP cannot overrule formal semantics

LSP advisory shape inference may supplement presentation. It cannot claim a formal type, narrowing, call target, or generic solution that contradicts `phalcom-semantic`.

---

# Part IV — Target semantic products

## 4. Snapshot-local identities

Add or finalize snapshot-local identities in `phalcom-semantic/src/identity.rs`.

Representative shape:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExpressionId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FlowNodeId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FlowEdgeId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PredicateId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExplanationId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DiagnosticCauseId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CallResolutionId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AnalysisIncidentId(pub u32);
```

If Spec 04 introduces stable AST node IDs before 04.5 starts, reuse/adapt them rather than introducing duplicate expression identity.

If no AST node ID exists, assign `ExpressionId` deterministically per callable in source/preorder traversal. Never use pointer addresses as semantic identity.

Every ID is:

- snapshot-local;
- deterministic for one semantic generation where source shape is unchanged;
- cheap to store;
- not serialized as durable identity unless a later metadata spec defines a stable mapping.

## 4.1 `ExpressionAnalysis`

Target conceptual form:

```rust
pub struct ExpressionAnalysis {
    pub id: ExpressionId,
    pub range: SourceRange,

    pub knowledge: TypeKnowledge,
    pub denotation: Option<SemanticDenotation>,

    pub status: AnalysisStatus,
    pub explanation: Option<ExplanationId>,
    pub call: Option<CallResolutionId>,
}
```

`AnalysisStatus`:

```rust
pub enum AnalysisStatus {
    Ready,
    Invalid(DiagnosticCauseId),
    Blocked(BlockReason),
    DynamicBoundary(DynamicReason),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(AnalysisIncidentId),
}
```

A `Dynamic` `TypeKnowledge` and `DynamicBoundary` analysis status may coexist only where semantically meaningful; avoid duplicating state mechanically.

## 4.2 `CallableAnalysis`

The primary published body product is callable-scoped:

```rust
pub struct CallableAnalysis {
    pub callable: CallableId,
    pub body_range: SourceRange,

    pub graph: Arc<CallableFlowGraph>,
    pub expressions: ExpressionAnalysisIndex,
    pub bindings: BindingAnalysisIndex,
    pub calls: Arc<[CallResolution]>,

    pub explanations: Arc<ExplanationArena>,
    pub diagnostics: Arc<[SemanticDiagnostic]>,

    pub dependencies: Arc<[CallableId]>,
    pub status: CallableAnalysisStatus,
}
```

The precise ownership of `ExplanationArena` may be snapshot-global rather than per-callable if interning across callables proves useful. Start callable-local to simplify cancellation and publication; a later snapshot can concatenate/freeze arenas.

## 4.3 Do not make every expression a database query

`ExpressionAnalysis` is indexed inside the callable product.

Do **not** begin with one `SemanticDb` query key per AST expression. That would create large query-cardinality and dependency bookkeeping overhead.

The default granularity is:

```text
QueryKey::CallableBody(CallableId)
    -> CallableAnalysis
```

Fine-grained expression retrieval is a read-side index within that ready product.

A future profiler may justify finer query granularity for giant methods, but it is not the initial architecture.

---

# Part V — Bidirectional expression typing

## 5. Judgments

04.5 standardizes two ordinary checking modes.

Synthesis:

```text
Γ ; Φ ⊢ e ⇒ T
```

means the analyzer derives a type for `e`.

Checking:

```text
Γ ; Φ ⊢ e ⇐ E
```

means the analyzer checks `e` under an expected type/form `E`, using that expectation as a constraint where sound.

Operationally, expose one internal result-rich API:

```rust
fn analyze_expression(
    ctx: &mut BodyAnalysisContext<'_>,
    expression: ExpressionId,
    expected: ExpectedType,
    flow: &FlowState,
) -> ExpressionAnalysis;
```

Representative expectation:

```rust
pub enum ExpectedType {
    None,
    Proper(TypeId),
    Inference(InferenceTerm),
}
```

This `ExpectedType` is not Spec-04's kind/category `TypeExpectation`; use distinct naming if both appear in the same module.

## 5.1 Context sources

Expected types come from:

- explicit binding annotations;
- explicit field annotations;
- method parameter signatures;
- expected return types;
- tail-expression return checking;
- call parameters;
- block/callable parameter expectations;
- generic expected return constraints;
- tuple/record structural contexts;
- collection element contexts;
- assignment into an annotated binding.

## 5.2 Contextual empty collection literals

Example:

```phalcom
let users: List<User> = []
```

Target:

```text
expected List<User>
        ↓
[] : List<User>
```

No `TypeData::Infer` is needed.

Unconstrained:

```phalcom
let users = []
```

produces a solver-local element variable during analysis. If no later local evidence constrains it, the final body fact is explicitly underconstrained/unknown according to the accepted local-inference policy; it is never canonicalized as `List<?0>` into the snapshot.

The existing Phase-2 behavior:

```phalcom
let xs = []
xs.add(42)
```

must remain supported, but the evidence flows through a local inference session or binding inference slot rather than through a canonical infer type.

## 5.3 Contextual blocks

For:

```phalcom
users.map({ user => user.name })
```

after receiver specialization and method instantiation establish:

```text
expected block type:
    (User) -> ?U
```

the block is checked under:

```text
user : User
```

and its tail/result constrains `?U`.

The block must not first be independently typed with all parameters `Unknown` and later guessed from usage.

## 5.4 Return checking

`check_callable_body` currently checks a tail expression after the statement walker has already analyzed it. Remove this double analysis.

Target:

- the flow/body analyzer owns statement sequencing once;
- every reachable `return` is checked when encountered;
- a reachable tail expression is checked once under `expected_return`;
- all return sites contribute to inferred return knowledge where no explicit return annotation exists and inference policy permits it.

---

# Part VI — Local generic inference

## 6. Session-local inference representation

Create a dedicated module, recommended:

```text
phalcom-semantic/src/checker/inference.rs
```

with:

```rust
pub struct InferenceSession { ... }

pub struct InferenceVariable {
    pub id: InferVarId,
    pub kind: KindId,
    pub state: InferVarState,
}

pub enum InferenceTerm {
    Canonical(TypeId),
    Variable(InferVarId),

    Applied {
        origin: Box<InferenceTerm>,
        arguments: Box<[InferenceTerm]>,
    },

    Union(Box<[InferenceTerm]>),
    Tuple(Box<[InferenceTupleElement]>),
    Record(Box<[InferenceRecordField]>),
    Callable(InferenceCallable),
}
```

The exact local term algebra may be smaller if substitution can delay materialization against canonical signature templates. It must nevertheless support unresolved variables inside composite expected types without interning them as `TypeData`.

## 6.1 Reuse canonical generic signature truth

Do not create a second generic declaration model.

Consume:

```rust
TypeParameterData
TypeParameterOwner
GenericSignature
GenericConstraint
TypeTerm
TypeEnvironment
TypeView
```

from the implemented 01.5 model.

Method inference maps method-owned `TypeParameterId`s to local inference variables.

## 6.2 Constraint origin is mandatory

Every inference constraint records why it exists.

Representative form:

```rust
pub struct InferenceConstraint {
    pub relation: InferenceRelation,
    pub origin: ConstraintOrigin,
    pub explanation: ExplanationId,
}

pub enum InferenceRelation {
    Equivalent(InferenceTerm, InferenceTerm),
    Subtype(InferenceTerm, InferenceTerm),
}

pub enum ConstraintOrigin {
    Argument {
        call: ExpressionId,
        argument: ExpressionId,
        parameter_index: u16,
    },
    ExpectedResult {
        expression: ExpressionId,
    },
    BlockParameter {
        block: ExpressionId,
        parameter_index: u16,
    },
    BlockResult {
        block: ExpressionId,
    },
    CollectionElement {
        literal: ExpressionId,
    },
    GenericWhere {
        callable: CallableId,
        constraint_index: u16,
    },
}
```

This origin data is what later produces explanations such as:

```text
T is User here because the receiver has type List<User>
```

or:

```text
U is String because the block returns user.name : String
```

## 6.3 Inference outcomes

Use an explicit result:

```rust
pub enum InferenceOutcome {
    Solved(InferenceSolution),
    Underconstrained(UnderconstrainedInference),
    Ambiguous(AmbiguousInference),
    Conflicting(InferenceConflict),
    DynamicBoundary(DynamicBoundaryObligation),
    Blocked(BlockReason),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(AnalysisIncidentId),
}
```

Do not return `bool`.

## 6.4 Determinism

Given the same semantic snapshot and body source, local inference must:

- allocate variables in deterministic declaration/source order;
- process constraints in deterministic order;
- choose no arbitrary hash-map iteration winner;
- produce the same canonical solution/failure classification in cold and incremental analysis.

## 6.5 Occurs checking

Retain the current occurs-check principle, but run it against `InferenceTerm`, not canonical `TypeData::Infer`.

A constraint such as:

```text
?T == List<?T>
```

must fail as recursive local inference unless a future recursive-type feature explicitly changes it.

## 6.6 Kind-aware inference variables

Every method generic parameter has a canonical `KindId`. The local variable representing it inherits that kind.

Therefore higher-kinded local method inference can reject:

```text
?F :: Type -> Type
```

being solved with:

```text
Int :: Type
```

without first constructing an invalid canonical application.

## 6.7 No initial explicit method type-argument call syntax

Spec 04 ratifies method generic declarations, but does not ratify a separate explicit call-site syntax such as:

```phalcom
object.method<Int>(...)
```

04.5 must not invent one.

The initial tranche supports ordinary local inference from:

- receiver specialization;
- arguments;
- contextual block checking;
- expected result;
- `where` constraints.

Explicit method type arguments require a separate source-syntax decision if desired later.

---

# Part VII — Canonical call-resolution engine

## 7. One call-resolution operation

Replace the current split of copied-signature lookup plus flat argument checking with one reusable semantic operation.

Conceptually:

```rust
pub fn resolve_call(
    ctx: &mut BodyAnalysisContext<'_>,
    receiver: &ExpressionAnalysis,
    selector: &Selector,
    arguments: &[CallArgumentRef],
    expected_result: ExpectedType,
    flow: &FlowState,
) -> CallResolutionOutcome;
```

`CallResolution` should retain:

```rust
pub struct CallResolution {
    pub call: ExpressionId,
    pub target: CallableId,
    pub receiver_type: TypeId,

    pub receiver_environment: TypeEnvironment,
    pub method_solution: MethodInferenceSolution,

    pub parameter_types: Box<[TypeId]>,
    pub result: TypeKnowledge,

    pub explanation: ExplanationId,
}
```

Do not store a runtime method pointer.

## 7.1 Resolution algorithm

For a normal send:

```text
1. Analyze receiver.
2. Determine whether receiver knowledge is Known / Dynamic / Unknown / invalid.
3. For Known receiver, enumerate statically reachable receiver arms.
4. Resolve selector against each arm using canonical declaration surfaces.
5. Obtain stable CallableId and canonical semantic signature.
6. Build receiver TypeEnvironment from applied receiver and owner-relative Self.
7. Create method-local inference variables for method-owned generic parameters.
8. View callable parameters/result under receiver environment + method inference terms.
9. Analyze arguments contextually.
10. Accumulate equality/subtype constraints.
11. Apply expected-result constraints.
12. Solve method inference.
13. Materialize solved parameter/result forms only after solution is publishable.
14. Validate canonical GenericSignature `where` constraints.
15. Join arm results for union receivers.
16. Publish CallResolution + explanation.
```

## 7.2 Receiver specialization

Use 01.5's `TypeEnvironment` / `TypeView`.

For:

```text
receiver : Box<User>
```

and:

```phalcom
class Box<T> {
  get -> T
}
```

the selected callable is still the callable declared on `Box`; the view supplies:

```text
T := User
```

without creating a specialized runtime method or class.

## 7.3 `Self`

Bind `Self` in the receiver environment according to owner/side rules already defined by 01.5/04.

No special string substitution is allowed.

## 7.4 Method inference example

```phalcom
class List<T> {
  map<U>(transform: (T) -> U) -> List<U> { ... }
}
```

Call:

```phalcom
users.map({ user => user.name })
```

Derivation:

```text
receiver:
    users : List<User>

receiver environment:
    T := User

method variable:
    U := ?u0

specialized expected argument:
    transform : (User) -> ?u0

block:
    user : User
    user.name : String

constraint:
    String <: ?u0
    / equivalence according to chosen callable-inference rule

solution:
    ?u0 := String

result:
    List<String>
```

The explanation graph records each edge.

## 7.5 Expected result participation

For:

```phalcom
empty<T>() -> List<T>
```

and:

```phalcom
let users: List<User> = empty()
```

the expected result contributes:

```text
List<?T> <: List<User>
```

under the applicable variance/equality inference rule, yielding `T := User`.

Without an expected type:

```phalcom
let x = empty()
```

the call is `Underconstrained`, not silently `List<Dynamic>`.

## 7.6 Argument labels and rest parameters

Retain exact selector/parameter label semantics.

The resolver must distinguish:

- missing positional argument;
- extra positional argument;
- missing required label;
- unexpected label;
- duplicate label if parser permits recovery;
- rest argument acceptance;
- dynamic pack expansion.

Each failure gets a structured call-shape cause, not a generic type mismatch.

## 7.7 Dynamic argument packs

A dynamic/rest expansion may create a runtime-check boundary.

Do not claim that arguments are proven compatible when their positions/labels are unknown.

Return a call result with a `DynamicBoundary` explanation if runtime semantics permit the send.

---

# Part VIII — Union, Dynamic, Unknown, and invalid receivers

## 8. Union receiver semantics

For:

```text
receiver : A | B | C
```

a statically valid send requires every reachable arm to support the selector and argument shape.

Algorithm:

```text
for arm in receiver union:
    resolve call on arm

if every arm is valid:
    join result types
else:
    emit one compressed diagnostic describing failing arms
```

Example:

```text
Cat.name -> String
Dog.name -> String

Cat | Dog
    .name
=> String
```

If return types differ:

```text
Cat.name -> String
Dog.name -> Symbol
=> String | Symbol
```

If only one arm fails:

```text
error[type.member.unavailable]:
    `name` is unavailable when receiver is Dog
```

Do not emit one error for every successful arm.

## 8.1 Dynamic receiver

A `Dynamic` receiver preserves open-world execution.

The analyzer:

- does not emit a false missing-member error;
- records a dynamic call boundary;
- returns `Dynamic`/appropriate open-world result knowledge;
- records why formal checking stopped.

## 8.2 Unknown receiver

`Unknown` means insufficient evidence, not an intentional runtime escape.

Policy must remain non-rejecting unless a language mode explicitly requires complete typing.

The explanation distinguishes:

```text
unknown because unannotated
unknown because unresolved dependency
unknown because recursive analysis
```

where available.

## 8.3 Invalid receiver

If the receiver already has a root semantic error, the send is causally blocked.

Do not additionally report:

```text
missing member
cannot infer method generic
argument mismatch
```

unless one of those failures is independent of the invalid receiver.

---

# Part IX — Compiler-owned flow graph

## 9. Why a reusable flow graph is required

Flow-sensitive typing, later effect analysis, exit analysis, termination, contract VCs, and LSP reachability all need the same executable control structure.

04.5 therefore introduces a compiler-owned callable flow graph.

Recommended target module:

```text
phalcom-semantic/src/checker/flow/
    mod.rs
    graph.rs
    state.rs
    predicate.rs
    transfer.rs
```

Do not reuse VM bytecode CFGs. This graph is source/semantic and exists before code generation.

## 9.1 Graph shape

Representative:

```rust
pub struct CallableFlowGraph {
    pub callable: CallableId,
    pub entry: FlowNodeId,
    pub nodes: Box<[FlowNode]>,
    pub edges: Box<[FlowEdge]>,
}

pub struct FlowNode {
    pub id: FlowNodeId,
    pub kind: FlowNodeKind,
    pub range: SourceRange,
}

pub enum FlowNodeKind {
    Entry,
    Statement(StatementRef),
    Expression(ExpressionId),
    Branch(ExpressionId),
    Join,
    LoopHeader,
    Return,
    Throw,
    Break,
    Continue,
    Exit,
}
```

The exact AST reference representation depends on Spec-04 post-implementation AST identities.

## 9.2 Preserve Phalcom's actual control semantics

Phalcom control flow is message-oriented. The LSP flow engine already recognizes selectors such as:

```text
ifTrue(_)
ifFalse(_)
ifTrue(_,ifFalse)
whileTrue(_)
```

and short-circuit Boolean sends.

The compiler flow builder must model the actual language semantics rather than assuming a C-like `if` AST that does not exist.

Prefer deriving recognized control behavior from canonical core/native semantic annotations from 03.5 where available, instead of permanently hardcoding strings in multiple analyzers.

A staged bootstrap may use the existing selector vocabulary while 03.5 semantic control annotations are landing, but the final ownership should be one canonical control-semantic marker.

## 9.3 Non-local block returns

The existing LSP flow engine already tracks block effects/non-local returns. The formal compiler graph must preserve the language's real non-local return semantics.

Do not simplify blocks into ordinary nested functions if Phalcom's runtime semantics differ.

---

# Part X — Flow state and binding model

## 10. Binding state

Move from string-keyed mutable facts to `BindingId`.

Representative:

```rust
pub struct BindingState {
    pub binding: BindingId,
    pub declared: Option<TypeKnowledge>,
    pub current: TypeKnowledge,
    pub mutable: bool,
    pub version: u32,
    pub explanation: Option<ExplanationId>,
}
```

`declared` is the persistent admissibility envelope when a source annotation or authoritative parameter signature exists.

`current` is the type known on this particular flow path.

## 10.1 Annotated mutable binding

```phalcom
let x: Number = 1
```

at initialization:

```text
declared = Number
current  = Int
```

after:

```phalcom
x = 2.0
```

if `Float <: Number`:

```text
declared = Number
current  = Float
```

A later narrowing may produce an even more specific `current` type without changing `declared`.

## 10.2 Unannotated mutable binding

```phalcom
let x = 1
x = "hello"
```

does not become invalid merely because the initializer was `Int`, unless another language rule says local `let` inference freezes a declared type.

The 04.5 decision is:

> **For an unannotated mutable binding, the first inferred value is flow knowledge, not an implicit permanent annotation.**

Thus:

```text
after initializer: current = Int
after assignment:  current = String
```

At a branch merge, if one path has `Int` and another `String`:

```text
current = Int | String
```

## 10.3 Immutable/constant binding

For immutable bindings, initializer inference may be treated as stable because no later local assignment can invalidate it.

This distinction improves both precision and diagnostics.

---

# Part XI — Proof-neutral flow predicates

## 11. Purpose

04.5 needs a representation for facts such as:

```text
x is String
x is not None
amount > 0
index < size
```

without becoming a theorem prover.

Recommended:

```rust
pub enum FlowPredicate {
    IsType(FlowPlace, TypeId),
    NotType(FlowPlace, TypeId),

    Equal(FlowValue, FlowValue),
    NotEqual(FlowValue, FlowValue),

    Less(FlowValue, FlowValue),
    LessEqual(FlowValue, FlowValue),
    Greater(FlowValue, FlowValue),
    GreaterEqual(FlowValue, FlowValue),

    PredicateTrue(PredicateKey),
    PredicateFalse(PredicateKey),
}
```

## 11.1 Flow values

Do not embed arbitrary effectful AST expressions into facts.

Use a normalized bounded domain:

```rust
pub enum FlowValue {
    Place(FlowPlace),
    IntLiteral(Box<str>),
    BoolLiteral(bool),
    NoneValue,
    SymbolLiteral(Box<str>),
}

// Keep the source/canonical integer spelling here initially rather than
// introducing a second numeric runtime. Arithmetic reasoning belongs to 05.
```

The initial exact literal set may be smaller.

## 11.2 Places

Start with soundly trackable places:

```rust
pub enum FlowPlace {
    Binding(BindingId),
    Field {
        base: PlaceRoot,
        field: FieldId,
    },
}

pub enum PlaceRoot {
    Binding(BindingId),
    SelfValue,
}
```

Local bindings are strong places.

Mutable field/projection facts are weak/volatile unless mutation/effect knowledge proves stability.

Do not implement full alias/heap reasoning in 04.5.

## 11.3 Fact set

```rust
pub struct FactSet {
    facts: BTreeMap<FlowPredicate, PredicateEvidence>,
}

pub struct PredicateEvidence {
    pub predicate: PredicateId,
    pub explanation: ExplanationId,
}
```

Use deterministic ordering and canonical predicate normalization where straightforward.

04.5 may:

- add direct branch facts;
- add direct negations;
- recognize contradictions in trivial structural cases;
- intersect facts at joins;
- kill invalidated facts.

04.5 does not need a general arithmetic closure.

---

# Part XII — Branch refinement

## 12. Type tests

The LSP currently refines `is` / `is!` tests. Formalize that behavior in `phalcom-semantic`.

For:

```phalcom
value.is(String).ifTrue { ... }
```

or the canonical source spelling after parser/runtime semantics are accounted for:

true branch:

```text
value current type := narrowed type compatible with String
fact: IsType(value, String)
```

false branch:

```text
fact: NotType(value, String)
```

For a union:

```text
value : String | Int
```

true:

```text
String
```

false:

```text
Int
```

No general intersection type syntax is required; narrowing may compute the surviving union members.

## 12.1 `None`/Option-like facts

Do not conflate nominal `Option<T>` with a union unless the canonical type system defines that equivalence.

However, direct identity/type tests against the singleton `None` may refine reachable alternatives when canonical semantics can establish them.

## 12.2 Pattern binding

`if let` / pattern forms:

- analyze scrutinee once;
- bind pattern variables with component knowledge;
- scope bindings to the successful branch;
- record pattern-success facts;
- do not leak branch-only bindings after merge.

## 12.3 Direct comparison facts

A branch condition such as:

```text
amount > 0
```

may record the predicate on the true path and its direct logical complement on the false path when the relation has a defined complement:

```text
true:  amount > 0
false: amount <= 0
```

This is normalization, not theorem proving.

Similarly:

```text
x == y
```

true/false may yield equality/inequality facts if the operator semantics are compiler-trusted for that comparison form. Do not infer facts from arbitrary user-overridable `==(_)` unless the language gives that syntax a static semantic guarantee.

## 12.4 Short-circuit Boolean control

Phalcom's lazy Boolean primitives/intrinsics matter.

For `and`/`or` block-style control:

- right-hand block is checked only on the path where runtime would execute it;
- right-hand flow facts inherit the appropriate left-side condition;
- join follows actual lazy control flow.

03.5 intrinsic/control annotations should eventually identify these core semantics.

---

# Part XIII — Joins and reachability

## 13. Join rules

Given reachable incoming states `S1 ... Sn`:

### Binding current types

```text
join(T1, T2, ...)
```

uses canonical type union where all known alternatives are proper publishable types.

If one branch is `Dynamic`, the result observes dynamic-boundary policy rather than inventing a closed union.

If one branch is invalid/blocked, causal/reachability rules determine whether it contributes.

### Declared binding type

A declaration-level annotation does not change at a join.

### Facts

Keep only facts valid on every reachable incoming state.

Conceptually:

```text
facts(join) = intersection(facts(S1), facts(S2), ...)
```

with evidence merged as a `Join` explanation.

### Reachability

Unreachable paths do not weaken reachable facts.

## 13.1 Early exits

Example:

```phalcom
amount.<= (0).ifTrue {
  return
}

use(amount)
```

After the branch, only the false edge reaches `use(amount)`.

If the branch predicate is represented canonically as `amount <= 0`, continuation may record:

```text
amount > 0
```

This enables the diagnostic/explanation statement:

```text
amount is known to be positive here
```

without invoking Spec-05 theorem proving.

## 13.2 Unreachable code

04.5 may mark nodes unreachable from formal control flow.

Whether unreachable code is an error, warning, hint, or only an LSP visualization is a diagnostic-policy choice. The semantic fact itself is compiler-owned.

---

# Part XIV — Loops and bounded fixed points

## 14. Reuse the LSP algorithmic precedent, not its authority

The existing LSP flow analyzer already:

- computes loop headers;
- joins back edges;
- detects stable fixed points;
- uses a maximum iteration budget;
- widens when needed.

This is the correct algorithmic family.

04.5 reimplements/adapts it in `phalcom-semantic` against canonical types and `QueryBudget`.

## 14.1 No debug panic / release semantic divergence

The formal compiler analyzer must not have behavior like:

```text
debug build: panic when solver does not converge
release build: silently widen
```

for ordinary user source.

Instead:

```text
fixed point reached
    -> publish precise result

widening policy reached
    -> publish sound widened result + optional analysis note

semantic budget exhausted before sound publishable state
    -> BudgetExceeded / Blocked
```

The chosen widening must be sound and deterministic.

## 14.2 Budget reuse

Reuse existing `QueryBudget` dimensions:

```text
Steps
SccIterations
TypeDepth
DiagnosticNotes
```

Add a budget kind only if measurement shows an independent dimension is necessary.

## 14.3 Loop state

At a loop header:

```text
header_{n+1} = join(entry, back_edges(header_n))
```

until stable or widened.

Fact sets generally shrink under joins; current types may widen to unions.

---

# Part XV — Mutation and fact invalidation

## 15. Local assignments

A write to binding `x`:

- increments its flow version;
- replaces current type knowledge;
- kills predicates depending on the previous version of `x`;
- checks the RHS against `declared` knowledge if one exists.

This avoids stale facts:

```phalcom
if x > 0 {
    x = -1
    // x > 0 no longer valid
}
```

## 15.1 Fields and projections

For:

```text
account.balance
```

a fact is valid only while the analyzer can justify that the referenced field value has not changed.

Initial conservative rule:

- direct write to that field kills the fact;
- call on potentially aliased receiver kills mutable field facts;
- unknown/dynamic/reflection call kills affected projection facts conservatively;
- unrelated stable local facts survive.

## 15.2 Spec-05 effect feedback is optional precision

Later, if Spec 05 proves a call pure/non-mutating for relevant state, 04.5 may preserve more facts.

The dependency direction is optional:

```text
04.5 baseline flow
    does not require effects

05 effect knowledge
    may improve 04.5 invalidation precision
```

Avoid a mandatory circular query dependency.

---

# Part XVI — Type derivations and explanation graph

## 16. Terminology

04.5 standardizes:

### Evidence

Raw support for a semantic claim.

Examples:

- explicit annotation;
- literal syntax;
- trusted native signature;
- declaration surface;
- branch condition.

### Derivation

Applications of Phalcom's formal semantic/type rules establishing a judgment.

Examples:

```text
e : T
A <: B
F :: Type -> Type
T := User
```

### Proof

Evidence/derivation sufficient, under the relevant trust policy, to establish a proposition.

04.5 produces ordinary **type derivations**. It does not define the general Spec-05 proof artifact/trust system.

## 16.1 Type-proof domains in 04.5

04.5 may retain derivations for:

```text
Typing:
    expression : Type

Kinding:
    type-form :: Kind

Subtyping:
    A <: B

Equivalence:
    A ≡ B

Generic solution:
    parameter := type

Generic constraint satisfaction:
    substituted where relation holds

Flow refinement:
    under path predicate P, binding has narrowed type T
```

These are ordinary type-system proofs/derivations.

## 16.2 Arena-based representation

Recommended target:

```text
phalcom-semantic/src/explain/
    mod.rs
    arena.rs
    node.rs
    slice.rs
```

Representative node:

```rust
pub struct ExplanationNode {
    pub kind: ExplanationKind,
    pub parents: Box<[ExplanationId]>,
}

pub enum ExplanationKind {
    ExactSyntax {
        span: SemanticSourceSpan,
        ty: TypeId,
    },

    DeclaredType {
        span: SemanticSourceSpan,
        ty: TypeId,
    },

    TrustedNative {
        callable: CallableId,
    },

    MemberSelection {
        callable: CallableId,
        receiver: TypeId,
    },

    ReceiverSpecialization {
        parameter: TypeParameterId,
        argument: TypeId,
    },

    GenericBinding {
        parameter: TypeParameterId,
        solution: TypeId,
    },

    ExpectedType {
        expected: TypeId,
        source: SemanticSourceSpan,
    },

    Relation {
        actual: TypeId,
        expected: TypeId,
        relation: ExplainedRelationKind,
        outcome: ExplainedRelationOutcome,
    },

    GenericConstraint {
        owner: TypeParameterOwner,
        index: u16,
    },

    FlowRefinement {
        binding: BindingId,
        before: TypeKnowledge,
        after: TypeKnowledge,
        predicate: PredicateId,
    },

    Join {
        node: FlowNodeId,
    },

    DynamicBoundary {
        reason: DynamicReason,
    },
}
```

The initial helper enums are intentionally small:

```rust
pub enum ExplainedRelationKind {
    Assignable,
    Subtype,
    Equivalent,
}

pub enum ExplainedRelationOutcome {
    Proven,
    Refuted,
    DynamicBoundary,
    Blocked,
}
```

If cloning `TypeKnowledge` into refinement nodes measures poorly, replace those fields with arena-local compact knowledge IDs; do not introduce a second semantic type representation.

## 16.3 Explanation is not semantic identity

Explanation IDs never participate in:

- type interning;
- subtype keys;
- selector identity;
- metadata canonical fingerprints unless a future debug metadata extension explicitly records them.

## 16.4 Lazy human slicing

Do not eagerly format prose for every derivation.

The analyzer retains structured nodes.

A diagnostic explanation builder computes a bounded human slice:

```text
failure
    ↑
expected type source
    ↑
receiver specialization
    ↑
generic binding
```

and converts only the relevant path into notes.

This is both faster and more readable than dumping a solver trace.

## 16.5 Relation evidence migration

`RelationEvidence` currently stores `Vec<String>` notes.

Target:

```rust
pub struct RelationEvidence {
    pub explanation: Option<ExplanationId>,
}
```

or an equivalent structured bridge.

During migration, retain old notes only for compatibility tests; do not keep both as independent formal truth.

---

# Part XVII — Diagnostic causality

## 17. Causal status

Introduce a structured cause arena or compact cause IDs.

Representative:

```rust
pub enum DiagnosticCause {
    TypeRelationRefuted {
        actual: TypeId,
        expected: TypeId,
        relation: RelationFailure,
    },

    CallShape(CallShapeFailure),
    MissingMember(MissingMemberFailure),
    GenericInference(InferenceFailure),
    Constraint(GenericConstraintFailure),
    Flow(FlowFailure),
    UpstreamInvalid {
        source: DiagnosticCauseId,
    },
}
```

## 17.1 Suppression

If:

```text
UnresolvedName(foo)
```

already invalidates a receiver expression, downstream member lookup becomes:

```text
BlockedBy(foo-cause)
```

and does not create a new primary diagnostic.

This rule applies recursively to:

- missing member;
- generic underconstraint caused solely by invalid argument;
- argument mismatch caused solely by invalid actual;
- return mismatch caused solely by invalid body expression.

## 17.2 Do not suppress independent contradictions

If two arguments independently violate two distinct parameter requirements, both may deserve diagnostics.

Suppression is causal, not merely “one error per line”.

---

# Part XVIII — Structured diagnostics

## 18. Extend `SemanticDiagnostic`

Current semantic diagnostics carry:

```text
code
severity
message
primary span
labels
```

Target conceptual model:

```rust
pub struct SemanticDiagnostic {
    pub code: DiagnosticCode,
    pub severity: DiagnosticSeverity,

    pub title: String,
    pub primary: DiagnosticLabel,
    pub secondary: Vec<DiagnosticLabel>,

    pub notes: Vec<DiagnosticNote>,
    pub helps: Vec<DiagnosticHelp>,
    pub fixes: Vec<DiagnosticFix>,

    pub cause: Option<DiagnosticCauseId>,
    pub explanation: Option<ExplanationId>,
}
```

A staged implementation may preserve current `message`, `primary_range`, and `labels` accessors to avoid breaking LSP/compiler consumers in one commit.

## 18.1 Primary versus secondary labels

The existing renderer already distinguishes primary and secondary visual spans. Carry that distinction semantically.

Do not encode “secondary” only by list position.

## 18.2 Fixes

```rust
pub struct DiagnosticFix {
    pub title: String,
    pub applicability: FixApplicability,
    pub edits: Vec<SourceEdit>,
}

pub enum FixApplicability {
    Always,
    Maybe,
    Manual,
}

pub struct SourceEdit {
    pub span: SemanticSourceSpan,
    pub replacement: String,
}
```

Spec-04 syntax migration fixes remain owned by 04, but 04.5 supplies the shared structured mechanism consumed by LSP CodeActions.

## 18.3 Diagnostic examples

### Generic argument mismatch

```text
error[type.call.argument_mismatch]: incompatible argument

   ╭─[users.ph:18:11]
18 │   users.add("Hasan")
   ·             ───┬───
   ·                ╰── found String
   ╰────
  note: add(_) expects User
  note: users has type List<User>, so T is User here
```

Internal evidence:

```text
users : List<User>
receiver specialization T := User
List<T>.add(value: T)
"Hasan" : String
String <: User refuted
```

### Underconstrained generic

```text
error[type.inference.underconstrained]: cannot infer T

   ╭─[example.ph:7:14]
 7 │   let result = empty()
   ·                ───┬──
   ·                   ╰── T has no constraining use
   ╰────
  note: empty<T>() returns List<T>
  help: provide an expected type
```

### Union member failure

```text
error[type.member.unavailable]: `length` is not available on every possible receiver

   ╭─[example.ph:18:7]
18 │   value.length
   ·         ──┬───
   ·           ╰── unavailable when value is Int
   ╰────
  note: value has type String | Int here
```

### Generic invariance explanation

```text
error[type.assignment.mismatch]: incompatible assignment

   ╭─[animals.ph:21:29]
21 │   let animals: Cell<Animal> = dogs
   ·                ─────┬────    ──┬─
   ·                     │          ╰── Cell<Dog>
   ·                     ╰── expected Cell<Animal>
   ╰────
  note: Cell<T> is invariant in T
  note: Dog is a subtype of Animal, but Cell<Dog> is not a subtype of Cell<Animal>
```

## 18.4 Expanded explanations

The default diagnostic is concise.

A future/initial CLI option:

```text
phalcom check --explain
```

may render an explanation slice:

```text
why User?

  users : List<User>
       ↓ receiver specialization

  List<T>.add(value: T)
       ↓ T = User

  expected argument : User
```

LSP may expose the same graph through hover/detail/code action without re-running inference.

---

# Part XIX — Diagnostic presentation convergence

## 19. Preserve PDR-0014

Phalcom has already ratified an in-house diagnostic renderer.

Do not add `miette` or another competing report framework.

The existing semantic visual language includes:

```text
╭─ │ · ╰──
```

plus roles for:

- error/warning/help severity;
- locations;
- identifiers;
- rails;
- line numbers;
- source;
- primary span;
- secondary span;
- elision;
- causal/fiber chains.

04.5 extends this style to static semantic diagnostics.

## 19.1 Proposed `phalcom-diagnostics` crate

The reusable presentation substrate currently lives under `phalcom-core/src/diagnostics`, forcing VM-independent compiler/tooling code to avoid it.

Introduce:

```text
phalcom-diagnostics/
    Cargo.toml
    src/
        lib.rs
        style.rs
        snippet.rs
        report.rs
```

Move/adapt:

```text
Role
AnsiColor
Weight
ColorMode
GlyphSet
Glyphs
RenderConfig
Styler
Snippet
Label
LabelKind
```

from core.

Keep in `phalcom-core`:

```text
VM traceback walking
RuntimeError adaptation
native-frame traceback construction
runtime-only suggestions
```

`phalcom-diagnostics` must not depend on `phalcom-core`.

Preferred initial dependencies:

```text
phalcom-common
unicode-width
```

Only add module/source identity dependencies if required by the report adapter.

## 19.2 Semantic report renderer

Render `SemanticDiagnostic` through an adapter that resolves:

```text
ModuleId
    -> source text
    -> display path
```

A diagnostic with cross-file supporting labels renders multiple source blocks rather than pretending all labels belong to one source string.

## 19.3 Structural tests, not byte-format prison

Preserve the traceback philosophy:

- unit-test column arithmetic exactly;
- test role/glyph behavior;
- test semantic report structure;
- avoid making every whitespace byte of the human format a permanent compatibility contract.

Machine-readable diagnostics should have their own stable structure.

---

# Part XX — LSP convergence

## 20. Formal versus advisory layers

After 04.5:

```text
phalcom-semantic
    formal compiler-authoritative types
    formal narrowing
    formal resolved call targets
    formal generic substitutions
    formal diagnostics
    formal explanation graph

phalcom-lsp
    consumes formal facts
    +
    may retain advisory runtime ValueShape intelligence
```

The LSP must not maintain another formal answer for concepts 04.5 publishes.

## 20.1 Migration from `semantic/flow.rs`

Do not delete the existing LSP flow engine immediately.

Migration stages:

1. add compiler-owned flow products;
2. expose them through `SemanticSnapshot`;
3. switch hover/diagnostic/type consumers to compiler facts;
4. run parity tests against existing LSP inference;
5. identify remaining tooling-only `ValueShape` capabilities;
6. retain only those advisory capabilities;
7. delete duplicated formal narrowing/call-resolution logic.

## 20.2 What can remain advisory

Examples likely appropriate for LSP-only `ValueShape`:

- captured method-family snapshots;
- bound-method runtime shape;
- module value shape;
- exact literal list positional shape when no formal type distinction exists;
- heuristic interprocedural parameter shape used only for completion ranking.

These facts may supplement, never override, formal typing.

## 20.3 LSP diagnostics

Current `phalcom-lsp/src/diagnostics.rs` maps every semantic label using the caller's single URI.

That is insufficient once cross-module labels are first-class.

Change the adapter to accept:

```rust
trait ModuleUriResolver {
    fn uri_for(&self, module: &ModuleId) -> Option<Url>;
    fn line_index_for(&self, module: &ModuleId) -> Option<&LineIndex>;
}
```

or an equivalent snapshot-backed mapping.

Each `DiagnosticRelatedInformation` uses the label's own `SemanticSourceSpan.module`.

## 20.4 Code actions

Translate `SemanticDiagnostic.fixes` into LSP `CodeAction` / `WorkspaceEdit`.

Do not parse human `help:` strings to reconstruct edits.

## 20.5 “Why this type?”

A future LSP request can render `ExplanationId`:

```text
expression → type
receiver → selected callable
generic parameter → inferred argument
flow predicate → narrowed type
```

No new semantic analysis is performed in the request handler.

---

# Part XXI — `SemanticDb`, workspace, and snapshot integration

## 21. Query ownership

Use existing:

```rust
QueryKey::CallableBody(CallableId)
```

as the main analysis key.

The value conceptually represents a fingerprint/serialized handle to `CallableAnalysis`.

If query values remain byte-oriented in the current staged DB, add a typed product layer adjacent to `QueryValue` rather than serializing rich analysis to JSON internally.

Recommended future direction:

```rust
pub enum SemanticProduct {
    CallableAnalysis(Arc<CallableAnalysis>),
    ...
}
```

or a typed arena keyed by `QueryKey`, depending on Spec-01 database architecture.

Do not duplicate cache ownership in the checker.

## 21.1 Dependencies

A callable-body analysis records dependencies on:

- its source body fingerprint;
- owning declaration surface;
- its canonical callable signature;
- generic signature;
- referenced declarations/types;
- resolved callable targets;
- relevant field signatures;
- supertype templates;
- native/core surfaces;
- optional effect facts if used for precision.

A body-only source edit should invalidate:

```text
that CallableBody
+ reverse semantic dependents that consume its published facts
```

not every module surface.

## 21.2 Workspace migration

Current Phase H:

```text
for module:
    create CheckingContext
    recursively check every statement/body
```

Target:

```text
Phase H1:
    enumerate source CallableIds / top-level body units

Phase H2:
    evaluate CallableBody queries

Phase H3:
    aggregate diagnostics

Phase H4:
    freeze CallableAnalysis products into snapshot
```

Top-level executable module statements need a stable synthetic callable/body identity or a separate `TopLevelBody(ModuleId)` key. Prefer a semantic identity model already used by the module/compiler rather than fabricating selector text if possible.

## 21.3 Snapshot additions

Representative:

```rust
pub struct SemanticSnapshot {
    // existing fields...

    pub callable_analyses:
        Arc<BTreeMap<CallableId, Arc<CallableAnalysis>>>,

    pub top_level_analyses:
        Arc<BTreeMap<ModuleId, Arc<BodyAnalysis>>>,
}
```

If explanations are per-callable, no giant global arena is required.

Expose read-side helpers:

```rust
analysis_for_callable(...)
expression_analysis(...)
call_resolution(...)
flow_state_at(...)
explain(...)
```

## 21.4 Partial snapshots

`SnapshotStatus::Partial` must reflect blocked/cancelled/budget-limited formal products according to Spec-01 publication policy.

A partial snapshot must never masquerade as a complete proof of absence of diagnostics.

---

# Part XXII — Spec-05 handoff

## 22. Published inputs to advanced analysis

04.5 publishes enough semantic structure that Spec 05 does not need to re-parse/re-type method bodies.

Required handoff:

```text
CallableFlowGraph
flow state / fact set at relevant nodes
resolved call targets
receiver + method generic substitutions
expression canonical types
binding identities
reachability
type derivations / explanations
mutation invalidation boundaries
```

## 22.1 Example: contract call

Source:

```phalcom
if amount > 0 {
    account.withdraw(amount)
}
```

04.5 may publish at the call node:

```text
amount : Int
account : Account
fact: amount > 0
target: Account.withdraw(_)
```

If `withdraw` has:

```phalcom
@requires(amount <= balance)
```

Spec 05 constructs the proof obligation:

```text
{ amount > 0 } ⊨ amount <= account.balance ?
```

04.5 does not answer that implication.

## 22.2 Type derivations versus program proofs

04.5 can already establish:

```text
amount : Int
List<User> <: Sequence<User>
T := User
```

through type derivations.

Spec 05 adds proofs of arbitrary program propositions:

```text
amount > 10 ⇒ amount > 0
precondition holds
postcondition holds
method terminates
invariant preserved
```

This boundary prevents ordinary type checking from accidentally becoming dependent on a heavyweight prover.

---

# Part XXIII — Target file layout

## 23. Current-to-target touch map

### Existing files to modify

| File | Current role | 04.5 change |
|---|---|---|
| `phalcom-semantic/src/checker/mod.rs` | Phase-2 checker exports | expose new body-analysis/bidirectional APIs; retain compatibility wrappers temporarily |
| `phalcom-semantic/src/checker/context.rs` | mixed lexical/solver/dispatch state | split immutable body context from changing `FlowState`; remove global `LocalConstraintSolver` |
| `phalcom-semantic/src/checker/typed_expr.rs` | type/constraint/provenance expression result | evolve into `ExpressionAnalysis` or compatibility wrapper |
| `phalcom-semantic/src/checker/expression.rs` | 41KB synthesis engine | route through bidirectional analyzer; move control/flow-specific logic to flow modules |
| `phalcom-semantic/src/checker/call.rs` | flat argument checking | canonical call resolution + receiver specialization + method inference |
| `phalcom-semantic/src/checker/statement.rs` | lexical statement walk | flow-transfer integration; declared/current binding distinction |
| `phalcom-semantic/src/checker/declaration.rs` | re-resolves signatures and checks bodies | consume Spec-04 published callable signatures; analyze by `CallableId`; remove tail double-synthesis |
| `phalcom-semantic/src/checker/result.rs` | diagnostics-only report | compatibility adapter over richer body/callable products |
| `phalcom-semantic/src/types/constraint.rs` | store-backed local inference solver | replace with session-local solver; later delete old solver |
| `phalcom-semantic/src/types/store.rs` | canonical type store including `Infer` | remove `TypeData::Infer` and `TypeStore::infer` after migration |
| `phalcom-semantic/src/types/environment.rs` | lazy specialization | reuse for receiver specialization; remove infer pass-through after store cleanup |
| `phalcom-semantic/src/types/evidence.rs` | knowledge + string provenance | attach structured explanation refs; retain authority semantics |
| `phalcom-semantic/src/types/relation.rs` | bounded subtype/assignability | preserve full outcomes/evidence; integrate explanation IDs; avoid `DUMMY` loss in diagnostics |
| `phalcom-semantic/src/types/outcome.rs` | budgets/outcomes | reuse; structured relation evidence |
| `phalcom-semantic/src/dispatch.rs` | selector → copied signature | return stable callable identity + semantic signature/view |
| `phalcom-semantic/src/surface.rs` | declaration/member lookup maps | converge on canonical callable semantic signatures after Spec 04 |
| `phalcom-semantic/src/identity.rs` | semantic IDs | add expression/flow/explanation IDs if not supplied by Spec 04 |
| `phalcom-semantic/src/scope.rs` | minimal binding table | strengthen compiler-owned per-callable binding identity/scope mapping |
| `phalcom-semantic/src/db/key.rs` | semantic query keys | keep `CallableBody`; add only justified top-level/body keys |
| `phalcom-semantic/src/db/state.rs` | query outcomes | typed product integration if needed |
| `phalcom-semantic/src/workspace.rs` | phases A–J, recursive Phase H | query-driven callable body analysis/publication |
| `phalcom-semantic/src/snapshot.rs` | immutable type/surface/diagnostic snapshot | add formal callable/body analysis indexes |
| `phalcom-semantic/src/diagnostic.rs` | flat structured semantic diagnostic | causes, note/help/fix/explanation support |
| `phalcom-lsp/src/semantic/flow.rs` | advisory formal-like flow duplicate | migrate formal responsibilities to semantic; retain only tooling-only shape logic |
| `phalcom-lsp/src/semantic/infer.rs` | advisory callable solver | reduce/remove formal duplicated solver after parity |
| `phalcom-lsp/src/semantic/facts.rs` | `ValueShape` advisory facts | keep explicitly advisory; map formal facts from semantic |
| `phalcom-lsp/src/diagnostics.rs` | semantic diagnostic → LSP | cross-module labels, fixes, explanation hooks |
| `phalcom-core/src/modules/compile.rs` | semantic error wrapper | preserve source-backed structured diagnostics for pretty CLI rendering |
| `phalcom-core/src/diagnostics/*` | shared style currently in VM crate | extract generic substrate to VM-free crate |
| root `Cargo.toml` | workspace members | add `phalcom-diagnostics` |
| `phalcom-core/Cargo.toml` | core deps | depend on `phalcom-diagnostics` |
| `phalcom-lsp/Cargo.toml` | LSP deps | depend only if direct shared formatting helpers are genuinely needed; LSP normally consumes structured diagnostics |

### New files recommended

```text
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/expected.rs

phalcom-semantic/src/checker/flow/mod.rs
phalcom-semantic/src/checker/flow/graph.rs
phalcom-semantic/src/checker/flow/state.rs
phalcom-semantic/src/checker/flow/predicate.rs
phalcom-semantic/src/checker/flow/transfer.rs

phalcom-semantic/src/explain/mod.rs
phalcom-semantic/src/explain/arena.rs
phalcom-semantic/src/explain/node.rs
phalcom-semantic/src/explain/slice.rs

phalcom-diagnostics/Cargo.toml
phalcom-diagnostics/src/lib.rs
phalcom-diagnostics/src/style.rs
phalcom-diagnostics/src/snippet.rs
phalcom-diagnostics/src/report.rs
```

Exact splitting may be adjusted after Spec-04 refactors land. The semantic ownership and dependency directions are normative.

---

# Part XXIV — Detailed implementation workstreams

## 24. Workstream E0 — Post-Spec-04 archaeology and conflict freeze

**Purpose:** 04.5 must not implement against stale AST/signature shapes while Spec 04 is landing.

### Inspect

- latest `main` commit;
- `phalcom-ast/src/ast.rs`;
- `phalcom-semantic/src/types/annotation.rs`;
- `phalcom-semantic/src/declarations.rs`;
- `phalcom-semantic/src/signature.rs`;
- `phalcom-semantic/src/surface.rs`;
- `phalcom-semantic/src/checker/declaration.rs`;
- `phalcom-semantic/src/checker/call.rs`;
- Spec-04 tests.

### Verify before changing code

1. class/method generic binders exist in AST;
2. `where` constraints lower into canonical `GenericSignature`;
3. callable generic signature is addressable by `CallableId`;
4. generic superclass templates are published;
5. source callable signatures are published before body checking;
6. type-lambda forms are canonical;
7. 03.5 changes to native surface, if already landed, are incorporated.

### Conflict rule with 03.5

04.5 must not reintroduce:

- handwritten native method signature tables;
- `register_standard_surfaces` replacements competing with 03.5;
- fake LSP native AST identities.

03.5 owns native/core surface convergence. 04.5 consumes whichever canonical semantic surface is current after rebase.

### Baseline commands

```bash
cargo test -p phalcom-semantic
cargo test -p phalcom-lsp
cargo test -p phalcom-core --lib
```

Run repository-registered integration targets required by touched code.

**Exit criterion:** write a short implementation note recording the actual post-04 symbols used by the workstreams below.

---

## 25. Workstream E1 — Expression-analysis and causal-status model

### Tests first

Create:

```text
phalcom-semantic/tests/spec04_5_expression_analysis.rs
```

Test:

- literal has `Ready` + exact type + explanation;
- unresolved name has root invalid/unknown status;
- dependent member send is causally blocked;
- no duplicate dependent diagnostic;
- expression IDs deterministic within unchanged source.

### Implementation

Add:

```text
checker/analysis.rs
```

Introduce:

```rust
ExpressionAnalysis
AnalysisStatus
ExpressionAnalysisIndex
DiagnosticCauseId
```

Update `TypedExpression` compatibility.

### Migration

Existing callers of:

```rust
synthesize_expr
synthesize_typed_expr
```

may remain as wrappers returning `.knowledge` during migration.

**Deletion criterion:** formal consumers no longer rely on a `TypeKnowledge`-only expression API when they need causality/explanation.

---

## 26. Workstream E2 — Binding identity and stable declared/current type split

### Tests first

Create cases:

```phalcom
let x: Number = 1
x = 2.0
```

Expected:

```text
declared Number
current Int → Float
no error
```

Test:

```phalcom
let x: Number = 1
x = "text"
```

Expected one assignment error; current invalid state must not erase the declaration requirement.

Test unannotated mutable branching:

```phalcom
let x = 1
condition.ifTrue { x = "text" }
```

Expected merge is bounded `Int | String` when both paths reachable.

### Implementation

- use `BindingId`;
- expand `scope.rs` or add callable binding table;
- define `BindingState`;
- stop using `HashMap<String, ValueSemanticFact>` as the sole changing body state;
- update assignments.

### Deletion criterion

`CheckingContext::assign_existing(name, fact)` is removed or becomes a compatibility shim over binding IDs and flow state.

---

## 27. Workstream E3 — Session-local inference and canonical-store cleanup

### Tests first

Create:

```text
phalcom-semantic/tests/spec04_5_inference_session.rs
```

Test:

- fresh inference variable does not increase canonical `TypeStore` type count;
- occurs check rejects recursive local solution;
- kind mismatch is explicit;
- underconstraint does not materialize `Dynamic`/`Object`;
- cancellation leaves no partial canonical type;
- deterministic solution independent of hash iteration.

### Implementation phase A

Add `checker/inference.rs`.

Keep old `TypeData::Infer` temporarily for untouched callers, but new 04.5 inference must not use it.

### Implementation phase B

Migrate:

- empty list/set/map inference;
- call generic inference;
- block result inference;
- any old local constraint test.

### Implementation phase C

Search entire workspace for:

```text
TypeData::Infer
TypeStore::infer
LocalConstraintSolver
fresh_var
```

Migrate every remaining semantic use.

### Delete

From `types/store.rs`:

```rust
TypeData::Infer(...)
TypeStore::infer(...)
```

Remove infer branches from:

- environment materialization;
- substitution;
- relation code;
- printers/fingerprints;
- metadata guards.

`TypeTerm::Infer` may remain only if it is explicitly a non-canonical local term and never published; otherwise replace it with the new inference representation.

### Acceptance invariant

```text
publish(snapshot)
⇒ no canonical TypeId contains an inference variable
```

---

## 28. Workstream E4 — Bidirectional checker

### Tests first

Create:

```text
phalcom-semantic/tests/spec04_5_bidirectional.rs
```

Cases:

```phalcom
let xs: List<Int> = []
```

```phalcom
let pair: (Int, String) = (1, "x")
```

contextual block parameters and return:

```phalcom
users.map({ user => user.name })
```

explicit return context:

```phalcom
method -> Number { 1 }
```

### Implementation

Add `checker/expected.rs`.

Introduce:

```rust
ExpectedType
analyze_expression(...)
check_expression(...)
synthesize_expression(...)
```

Preserve compatibility wrappers.

### Rule

Prefer checking under context where available; synthesis remains necessary for independent expressions.

### Exit criterion

No empty collection or block parameter requires canonical infer types merely because context is available.

---

## 29. Workstream E5 — Canonical call resolution and generic method inference

### Tests first

Create:

```text
phalcom-semantic/tests/spec04_5_generic_calls.rs
```

Required cases:

```phalcom
identity<T>(x: T) -> T
identity(42)                  // Int
```

```phalcom
empty<T>() -> List<T>
let users: List<User> = empty() // T := User
```

```phalcom
let x = empty()                // Underconstrained
```

```phalcom
class Box<T> {
  map<U>(f: (T) -> U) -> Box<U>
}
Box<User>.map(...)             // receiver T and method U distinct
```

method `where` success/failure.

HKT kind mismatch for a method generic if source semantics permit one.

### Implementation

Refactor `checker/call.rs`.

Introduce:

```rust
CallResolutionOutcome
CallResolution
ReceiverSpecialization
MethodInferenceSolution
CallShapeFailure
```

Change dispatch resolution to retain `CallableId`.

### Important rule

Argument analysis may require an expected callable type produced by the same inference session. Do not solve all generics before checking blocks.

### `where`

After tentative solution, substitute canonical generic constraints and use bounded canonical relation/equivalence APIs.

### Exit criterion

Adding a generic source/native method with a canonical `GenericSignature` requires no method-specific inference code.

---

## 30. Workstream E6 — Full relation-outcome consumption

### Tests first

Inject/test:

- refuted relation;
- dynamic boundary;
- blocked;
- cancelled;
- budget exceeded;
- internal failure.

Ensure only `Refuted` becomes ordinary mismatch.

### Implementation

Replace patterns like:

```rust
if let Assignability::Refuted { .. } = assignability
```

with exhaustive policy handling.

Create one adapter:

```rust
fn apply_relation_outcome(...)
```

or equivalent to avoid inconsistent policies across assignments/returns/calls.

### Repair evidence loss

Avoid using `TypeId::DUMMY` for human diagnostic actual/expected types when direct input types are known. Prefer retaining original query operands beside `RelationFailure`.

---

## 31. Workstream F1 — Compiler flow graph

### Tests first

Create:

```text
phalcom-semantic/tests/spec04_5_flow_graph.rs
```

Verify:

- deterministic node order;
- branch edges;
- return ends normal path;
- throw ends normal path;
- break/continue connect correctly;
- lazy Boolean block is conditional;
- non-local block return follows language semantics.

### Implementation

Add `checker/flow/graph.rs`.

Adapt actual AST control shapes.

Use source ranges and stable expression IDs.

### LSP reference

Use `phalcom-lsp/src/semantic/flow.rs` as algorithmic reference, not a dependency. `phalcom-semantic` must not depend on `phalcom-lsp`.

---

## 32. Workstream F2 — Flow state, predicates, and narrowing

### Tests first

Create:

```text
phalcom-semantic/tests/spec04_5_flow_refinement.rs
```

Cases:

- `is` / `is!` type narrowing;
- union removal on false branch;
- direct comparison fact;
- early return continuation fact;
- `if let` pattern binding;
- join intersects facts;
- unreachable path does not weaken state;
- assignment kills stale predicate.

### Implementation

Add:

```text
flow/state.rs
flow/predicate.rs
flow/transfer.rs
```

### Initial predicate domain

Keep intentionally bounded.

Do not add SMT, arithmetic closure, alias logic, or heap theorem proving.

---

## 33. Workstream F3 — Loop fixed point and widening

### Tests first

Cases:

- stable loop converges;
- union grows then stabilizes;
- loop assignment invalidates predicate;
- continue contributes back edge;
- break contributes exit;
- budget exhaustion produces explicit status;
- release/debug semantics identical.

### Implementation

Use `QueryBudget.max_scc_iterations`.

Define deterministic widening for current-type unions/fact sets.

### Exit criterion

No compiler flow loop can spin indefinitely or panic merely due to adversarial user code.

---

## 34. Workstream F4 — Iteration element typing

### Tests first

At minimum:

```phalcom
for x in List<Int> { ... }  // x : Int
```

and tuple/list known cases supported by canonical semantics.

### Implementation

Derive element type from canonical collection/iteration semantics.

Do not permanently bind loop lanes to `Dynamic`.

For collection kinds whose iteration semantics are not yet published, produce `Unknown`/dynamic boundary according to actual language behavior without false certainty.

---

## 35. Workstream X1 — Explanation/type-derivation arena

### Tests first

Create:

```text
phalcom-semantic/tests/spec04_5_explanations.rs
```

Verify derivation edges for:

- literal typing;
- declared binding;
- subtype success/failure;
- receiver specialization;
- method generic solution;
- expected-result inference;
- flow narrowing;
- branch join.

Test bounded slicing.

### Implementation

Add `src/explain/*`.

Replace string-only provenance paths incrementally.

### Important performance rule

Do not eagerly render strings during checking.

---

## 36. Workstream D1 — Structured diagnostics and suppression

### Tests first

Create:

```text
phalcom-semantic/tests/spec04_5_diagnostics.rs
```

Test structure, not terminal bytes:

- code;
- severity;
- primary label;
- secondary labels;
- notes;
- help;
- fix edits;
- cause ID;
- explanation ID;
- suppression.

### Implement

Extend `diagnostic.rs`.

Add dedicated diagnostic builders per semantic failure family, e.g.:

```text
diagnostics/call
diagnostics/inference
diagnostics/flow
```

if `diagnostic.rs` becomes too large.

### Rule

Builders consume structured causes; they do not recompute type semantics.

---

## 37. Workstream D2 — VM-free diagnostic renderer extraction

### Tests first

Move/copy tests for:

- Unicode widths;
- tabs;
- CJK/emoji;
- primary/secondary labels;
- ASCII fallback;
- color/no-color;
- width windowing.

### Create

```text
phalcom-diagnostics/
```

Add to workspace.

### Move

Generic style/snippet code from core while preserving API or temporary re-exports.

### Implement report renderer

Support:

- one primary source snippet;
- same-file secondary spans;
- additional cross-file supporting snippets;
- notes;
- helps;
- diagnostic code;
- optional expanded explanation.

### Core adaptation

`traceback.rs` imports shared style/snippet types.

Runtime traceback output behavior remains unchanged except necessary import paths.

---

## 38. Workstream D3 — CLI compiler diagnostic path

### Problem

`ProgramCompileError::Semantic` currently formats essentially as a debug representation.

### Goal

The CLI has access to:

- structured semantic diagnostics;
- source text;
- module/display-path mapping;
- shared render configuration.

### Implement

Introduce a lightweight source-backed diagnostic bundle or semantic-failure adapter in `phalcom-core/src/modules/compile.rs`.

Do not make terminal ANSI rendering part of `phalcom-semantic`.

### Test

Run a semantic type mismatch through:

```text
phalcom check
phalcom run/compile path where semantic errors reject compilation
```

and assert both use the same static diagnostic register.

---

## 39. Workstream L1 — LSP formal semantic migration

### Tests first

Create parity tests:

```text
same source snapshot
compiler formal type == LSP formal type
compiler call target == LSP formal call target
compiler narrowing == LSP formal narrowing
compiler diagnostic code/spans == LSP published diagnostic
```

### Implement

- query `SemanticSnapshot` for formal expression/call/flow facts;
- use formal type results in hover/completion/signature help;
- map cross-file diagnostic labels correctly;
- map structured fixes to code actions;
- expose explanation rendering hook.

### Advisory retention rule

Keep `ValueShape` only where it adds non-formal tooling value.

### Deletion criterion

No LSP feature may use advisory `ValueShape` as the authoritative answer for a formal type/call/narrowing fact already published by 04.5.

---

## 40. Workstream Q1 — Incremental callable publication

### Tests first

Create:

```text
phalcom-semantic/tests/spec04_5_incremental.rs
```

Test:

1. cold and incremental outputs equal;
2. body-only edit invalidates changed callable;
3. unrelated callable remains ready/hit;
4. direct dependent callables invalidate only when they consume changed published facts;
5. cancellation publishes no partial body product;
6. stale revision cannot publish;
7. diagnostic/explanation IDs never cross snapshots incorrectly.

### Implement

Use:

```text
QueryKey::CallableBody
DependencyRecorder
SemanticDb::publish_ready
SemanticDb::invalidate
QueryMetrics
```

### Performance assertion

A local edit must not force unconditional whole-workspace body analysis.

---

## 41. Workstream V1 — Full conformance and performance verification

Required command family at completion:

```bash
cargo test -p phalcom-semantic
cargo test -p phalcom-lsp
cargo test -p phalcom-core --lib

cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

If repository CI uses a narrower/registered matrix, run that authoritative matrix as well.

No “complete” claim without actual command evidence.

---

# Part XXV — Migration and deletion register

## 42. Delete only after parity

### `TypeData::Infer`

Delete when:

- no 04.5 inference uses it;
- no tests depend on canonical infer types;
- no metadata/snapshot contains it;
- store-wide search shows no legitimate use.

### `TypeStore::infer`

Delete with `TypeData::Infer`.

### `LocalConstraintSolver`

Delete after all call/collection inference uses `InferenceSession`.

### `TypedExpression` old shape

Either rename/evolve or retain as compatibility wrapper. Delete old constraints/string-provenance fields once `ExpressionAnalysis`/explanations own them.

### String-keyed `LocalEnv`

Delete once `BindingId` flow state handles all body locals.

### `match_callable_arguments` old flat checker

Delete once `resolve_call` owns argument matching/inference.

### Repeated annotation lowering in `checker/declaration.rs`

Delete after Spec-04 published signatures are consumed by body analysis.

### Tail-expression double checking

Delete immediately when body transfer owns tail checking.

### LSP formal duplicate flow

Delete formal narrowing/call/type responsibility after parity. Retain only explicitly advisory features.

### Core-local copies of `style.rs` / `caret.rs`

Delete after `phalcom-diagnostics` extraction and runtime traceback parity.

---

# Part XXVI — Performance requirements

## 43. Runtime cost

04.5 is static semantic infrastructure.

Ordinary program execution must not gain:

- per-object type fields;
- changed instance layout;
- changed dispatch keys;
- inference allocations;
- flow facts;
- explanation arenas.

## 43.1 TypeStore growth

For the same canonical source semantics, repeated local inference sessions must not continually intern new temporary infer types.

After `TypeData::Infer` deletion:

```text
temporary inference count
    has no direct effect on canonical TypeStore size
```

Only solved canonical forms that are actually needed may be interned.

## 43.2 Explanation cost

Store compact nodes/edges, not formatted prose.

Bound:

- parent edge count where necessary;
- human diagnostic note slicing by `QueryBudget.max_diagnostic_notes`;
- duplicate equivalent local explanation nodes if profiling shows pathological growth.

Expanded explanations are computed on demand from retained structure.

## 43.3 Flow-state sharing

Use copy-cheap/persistent or carefully cloned small maps.

Do not clone every workspace declaration surface at every flow node.

## 43.4 Call lookup

No per-expression full registry scan.

Resolved declaration surfaces and `CallableId` lookup should remain indexed by owner/side/selector.

## 43.5 Incremental boundaries

A body edit should scale with:

```text
changed callable
+ affected reverse dependency closure
```

not total workspace size.

## 43.6 LSP duplication reduction

Once formal flow is compiler-owned, remove repeated whole-body analysis in the LSP for facts available from the semantic snapshot.

This is both a correctness and performance win.

---

# Part XXVII — Verification matrix

## 44. Expression semantics

| Case | Expected |
|---|---|
| integer literal | `Int`, exact-syntax derivation |
| explicit binding annotation | initializer checked contextually |
| unannotated immutable | stable inferred current fact |
| annotated mutable assignment | declared envelope preserved |
| unannotated mutable assignment | current flow type changes |
| invalid RHS | one root cause, dependents suppressed |
| tail expression | checked once |
| `Never` expression | normal-return behavior respected without implying termination |

## 44.1 Generic inference

| Case | Expected |
|---|---|
| `identity(42)` | `T := Int` |
| expected-result-only inference | solution from expected result |
| receiver + method generics | separate environments |
| underconstrained | explicit underconstraint |
| conflicting constraints | explicit conflict |
| HKT kind mismatch | explicit kind failure |
| method `where` success | accepted |
| method `where` refuted | structured constraint diagnostic |
| cancellation | no published solution |
| budget exhaustion | no fabricated mismatch |

## 44.2 Flow

| Case | Expected |
|---|---|
| type test true | narrowed current type |
| type test false | excluded compatible arm |
| early return | continuation receives false-edge facts |
| branch join | union current types, intersect facts |
| unreachable branch | does not weaken join |
| local assignment | kills dependent facts |
| mutable field + unknown call | projection facts killed conservatively |
| stable loop | fixed point |
| expanding loop | bounded widening |
| break/continue | correct exit/back edges |

## 44.3 Diagnostics

| Case | Expected |
|---|---|
| generic mismatch | actual + expected + substitution explanation |
| union missing member | failing arms compressed |
| underconstrained inference | cause + actionable help |
| cross-file constraint source | supporting module span preserved |
| dynamic boundary | explanation, not false error |
| blocked/cancelled | distinct status |
| root invalid receiver | no cascade |
| LSP | same code/severity/spans as compiler |
| CLI | traceback-family visual register |

## 44.4 Incremental

| Case | Expected |
|---|---|
| cold vs incremental | equivalent formal result |
| body-only edit | callable-scoped invalidation |
| unrelated callable | cache hit/unchanged |
| signature edit | dependents invalidate |
| stale publication | rejected |
| cancelled generation | no partial product leaks |

---

# Part XXVIII — Acceptance criteria

## 45. Required end state

04.5 is complete only when all of the following are true:

1. `phalcom-semantic` is authoritative for ordinary formal expression typing.
2. Bidirectional checking is used wherever expected context exists.
3. Method-generic inference is one canonical reusable operation.
4. Receiver specialization and method inference are distinct environments.
5. Expected result types may constrain method inference.
6. Underconstrained inference never silently becomes `Dynamic` or `Object`.
7. Solver-local inference variables are not canonical `TypeId`s.
8. `TypeData::Infer` and `TypeStore::infer` are deleted after migration.
9. Formal flow uses `BindingId`.
10. Declared/base binding type is separate from current flow type.
11. Compiler-owned branch narrowing exists.
12. Direct flow predicates are retained as proof-neutral facts.
13. Branch joins are reachability-aware and sound.
14. Loops have bounded deterministic fixed-point/widening behavior.
15. Mutation invalidation prevents stale projection facts.
16. Union-receiver calls validate every reachable arm.
17. `Dynamic`, `Unknown`, invalid, blocked, cancelled, and budget states remain distinct.
18. Type derivations are retained as structured explanation nodes.
19. Explanation structure is not part of canonical type identity.
20. Diagnostic builders consume semantic causes rather than recomputing semantics.
21. Causal cascade suppression is implemented.
22. Diagnostics support primary/secondary labels, notes, help, and structured fixes.
23. Static CLI diagnostics use the same visual language as runtime tracebacks.
24. The reusable style/snippet substrate is VM-independent or equivalently consumable without linking VM semantics.
25. LSP publishes compiler-owned formal diagnostics/types/narrowing/call targets.
26. LSP cross-file related diagnostic locations use the correct URI.
27. LSP advisory `ValueShape` does not override formal semantics.
28. `SemanticDb` owns callable-body caching/invalidation.
29. `SemanticSnapshot` exposes immutable callable analyses.
30. A body-only edit does not unconditionally reanalyze the whole workspace.
31. Spec 05 can consume flow graphs/facts/call resolutions without reimplementing ordinary typing.
32. Runtime selector/class/layout/allocation invariants remain unchanged.
33. Existing Spec-01/01.5/02/03/03.5/04 regression suites remain green.
34. Full workspace verification has been run before claiming completion.

---

# Part XXIX — Decision register

| Decision ID | Decision | Status |
|---|---|---|
| `DEC-EXPR-01` | 04.5 owns executable-expression static semantics | Ratified |
| `DEC-EXPR-02` | Checker is bidirectional, not synthesis-only | Ratified |
| `DEC-EXPR-03` | Expected result may constrain local generic inference | Ratified |
| `DEC-EXPR-04` | Expected type never enters runtime selector identity | Ratified |
| `DEC-EXPR-05` | Receiver specialization precedes method inference | Ratified |
| `DEC-EXPR-06` | Receiver and method generic environments are separate | Ratified |
| `DEC-EXPR-07` | Inference variables are session-local, not canonical types | Ratified |
| `DEC-EXPR-08` | No fallback of unsolved generics to `Dynamic`/`Object` | Ratified |
| `DEC-EXPR-09` | Initial explicit method type-argument call syntax is not invented here | Ratified deferral |
| `DEC-EXPR-10` | Union send must be valid for every reachable arm | Ratified |
| `DEC-EXPR-11` | Union call results join canonically | Ratified |
| `DEC-FLOW-01` | Compiler-owned formal flow lives in `phalcom-semantic` | Ratified |
| `DEC-FLOW-02` | LSP `ValueShape` remains advisory only | Ratified |
| `DEC-FLOW-03` | Stable declared binding type differs from current flow type | Ratified |
| `DEC-FLOW-04` | Unannotated mutable bindings are not frozen by first assignment | Ratified |
| `DEC-FLOW-05` | Flow facts are proof-neutral direct predicates | Ratified |
| `DEC-FLOW-06` | Facts survive joins only if true on all reachable paths | Ratified |
| `DEC-FLOW-07` | Mutable projection facts are conservatively invalidated | Ratified |
| `DEC-FLOW-08` | Loops use bounded deterministic fixed points/widening | Ratified |
| `DEC-PROV-01` | Type derivations are structured DAG/arena data | Ratified |
| `DEC-PROV-02` | Derivations are proof-irrelevant to canonical type identity | Ratified |
| `DEC-PROV-03` | General program proofs remain Spec 05 | Ratified |
| `DEC-DIAG-01` | Analyzer owns why; renderer owns how | Ratified |
| `DEC-DIAG-02` | Root-cause suppression is semantic | Ratified |
| `DEC-DIAG-03` | Static diagnostics reuse Phalcom traceback visual language | Ratified |
| `DEC-DIAG-04` | Diagnostic renderer remains in-house; no miette | Existing PDR preserved |
| `DEC-DB-01` | Callable body is initial incremental query granularity | Ratified |
| `DEC-DB-02` | No per-expression global query explosion initially | Ratified |
| `DEC-05-HANDOFF-01` | 04.5 publishes facts; 05 proves general implications | Ratified |

---

# Part XXX — Recommended commit sequence

## 46. Small reviewable milestones

A practical implementation sequence:

### Commit group A — foundational identities/results

```text
A1 expression/cause IDs + analysis status
A2 BindingId flow-state skeleton
A3 compatibility wrappers for existing checker APIs
```

### Commit group B — inference lifetime repair

```text
B1 InferenceSession + unit tests
B2 migrate empty collections
B3 migrate call inference
B4 remove TypeData::Infer / old LocalConstraintSolver
```

### Commit group C — bidirectional calls

```text
C1 ExpectedType/check API
C2 receiver specialization refactor
C3 generic method inference
C4 expected-result inference
C5 where-constraint validation
C6 union receiver calls
```

### Commit group D — formal flow

```text
D1 CallableFlowGraph
D2 binding transfer + branches
D3 type-test/pattern narrowing
D4 direct predicates
D5 reachability/joins
D6 loops/widening
D7 mutation invalidation
```

### Commit group E — explanations/diagnostics

```text
E1 ExplanationArena
E2 relation/generic/flow derivations
E3 diagnostic cause/suppression
E4 notes/help/fixes
```

### Commit group F — presentation

```text
F1 phalcom-diagnostics extraction
F2 semantic report renderer
F3 core CLI semantic error rendering
```

### Commit group G — LSP convergence

```text
G1 formal snapshot reads
G2 formal hover/call/narrowing
G3 structured fixes/related info
G4 advisory-flow reduction/deletion
```

### Commit group H — incremental publication

```text
H1 CallableBody query integration
H2 dependency/fingerprint plumbing
H3 clean-vs-incremental parity
H4 performance counters/regressions
```

Every group should remain bisectable and keep the repository compiling.

---

# Part XXXI — Final architecture

## 47. End-state data flow

```text
                    ┌─────────────────────┐
                    │       Spec 04       │
                    │ syntax + signatures │
                    └──────────┬──────────┘
                               │
                               ▼
                    canonical callable surface
                               │
               ┌───────────────┴────────────────┐
               │                                │
               ▼                                ▼
     receiver specialization           callable body source
      TypeEnvironment                           │
               │                                ▼
               └──────────────┐       CallableFlowGraph
                              │                │
                              ▼                ▼
                         call resolution ← FlowState
                              │                │
                              ▼                ▼
                       InferenceSession    predicates
                              │                │
                              └───────┬────────┘
                                      ▼
                              ExpressionAnalysis
                                      │
                         ┌────────────┴────────────┐
                         ▼                         ▼
                  ExplanationArena          SemanticDiagnostic
                         │                         │
                         │                 ┌───────┴────────┐
                         │                 ▼                ▼
                         │                CLI              LSP
                         │
                         ▼
                       Spec 05
            effects / contracts / VCs / proof
```

The end state is intentionally unified:

- one parser/lowering authority from 04;
- one canonical type/generic calculus from 01.5;
- one formal body analyzer in `phalcom-semantic`;
- one semantic explanation graph;
- one structured diagnostic truth;
- multiple presentation consumers;
- one future handoff to advanced proof analysis.

This is the point where Phalcom's typing architecture becomes not merely a checker that returns types, but a compiler-owned semantic intelligence layer capable of answering both:

```text
What is this?
```

and:

```text
Why is it this?
```

without changing the runtime semantics that make Phalcom Phalcom.

---

# Appendix A — Spec 04 dependency gates

The source specification deliberately stages parser/lowering work. 04.5 must not wait for unrelated syntax work, but it also must not bypass the semantic publication gates that its algorithms consume.

| 04.5 work | Minimum Spec-04 prerequisite | Reason |
|---|---|---|
| expression-analysis result model | S2 explicit lowering outcomes | causal statuses must agree with missing/unresolved/invalid/dynamic semantics |
| binding/flow graph infrastructure | S1/S2 stable executable AST + outcomes | does not require method generics yet |
| bidirectional literals/collections | S1/S2 | expected proper types must be canonical |
| class receiver specialization | S3 plus implemented 01.5 generic declarations | class binders need owner/index identity |
| method generic inference | S3 | method-owned parameters/signatures must exist |
| `where` call validation | S4 | constraints must be signature-owned canonical relations |
| type-lambda participation in inference | S5 | never fake lambda semantics in the checker |
| inherited generic member specialization | S6 | superclass templates and `Self` must be published |
| alias-sensitive body typing | S7 alias tranche | consume transparent canonical alias result, do not reimplement expansion |
| record-row inference | **not 04.5 initial gate**; waits on Spec 05 row solver | row solving is not ordinary 04.5 local inference |
| `Expr::TypeForm` value typing | S8 | type-form value island must have canonical AST/lowering |
| source/native call parity | S9 + 03.5 convergence | native/source signatures must be one semantic target |

The implementation may therefore begin E1/E2/F1 before all of Spec 04 is complete, but E5 generic-call acceptance cannot be declared complete before S3/S4 are landed and re-inspected.

## A.1 Rebase rule

Any workstream that starts before Spec 04 lands must be rebased against the actual post-04 types. Do not preserve an interim 04.5 compatibility structure merely because it was convenient before the parser/lowering branch merged.

Examples of structures that must not become permanent duplicates:

```text
04.5-owned shadow GenericSignature
04.5-owned shadow GenericConstraint
04.5-owned shadow TypeSyntax
04.5-owned shadow type-lambda binder identity
```

The canonical 01.5/04 representation wins.

---

# Appendix B — Expression coverage matrix

The current checker handles a broad AST surface, but several cases are coarse. 04.5 is not complete if only generic calls and flow tests work while existing expression forms regress.

| Expression family | Current repository behavior | 04.5 target |
|---|---|---|
| integer / float / string / bool literals | nominal literal class with `ExactSyntax` | retain exact nominal type evidence; no new literal-singleton type feature |
| symbol literal | resolves `Symbol`, with fallback behavior in current code | canonical `Symbol` type; no String fallback once core surface is authoritative |
| local variable | string-keyed lexical lookup | `BindingId` lookup; current flow type + declared envelope |
| `self` | current class nominal/class object | owner/side-relative canonical `Self` view, explanation edge |
| `super` | dispatch lookup marker + current class type | preserve explicit super lookup semantics with stable target identity |
| field | declaration surface field knowledge | field identity + flow/current projection fact where sound |
| assignment | checks current fact then overwrites | check declared envelope; update current flow version; kill dependent facts |
| list literal | join element types; empty uses store infer var | contextual element checking; local-only inference for empty/unconstrained |
| set literal | analogous collection synthesis | contextual/local-only inference |
| map literal | joined key/value types | contextual key/value checking; local-only empty inference |
| tuple literal | canonical positional/labeled tuple | contextual per-element checking; exact canonical tuple |
| record literal | canonical record fields | contextual field checking; preserve duplicate/recovery diagnostics from 04 |
| block | lexical scope + tail synthesis | contextual callable/block analysis + actual non-local-return semantics |
| `if let` | branch result union only | branch flow state, pattern facts, reachability-aware result join |
| `while let` | scoped body check | loop header/back-edge fixed point + pattern facts |
| method call | dispatch + already-concrete signature | stable target + receiver specialization + generic inference |
| unqualified call | current callable resolution path | same canonical call engine, with implicit receiver semantics preserved |
| binary operator | lowered/analyzed as message send | canonical message semantics; control/intrinsic annotations for lazy operators |
| unary operator | message send | same canonical call engine |
| property getter | member lookup | canonical call/member resolution, union-arm validation |
| property setter | type/member check | call engine + assignment/mutation invalidation |
| index get | subscript selector | canonical call engine |
| index set | subscript setter selector | canonical call engine + mutation invalidation |
| comparison chain | operands analyzed, result Bool | result Bool plus direct proof-neutral predicates where syntax semantics justify them |
| membership | operands analyzed, result Bool | Bool; flow fact only if a trusted semantic rule exists |
| `is` membership/type test | current Bool analysis; richer LSP advisory narrowing | formal type-test predicate and narrowing |
| range | currently coarse in checker | use canonical core/native range surface and type if available; never silently keep `Object` as permanent fallback |
| ellipsis/recovery expression | current checker has a coarse fallback | preserve actual language/recovery semantics; invalid recovery nodes must not fabricate useful type evidence |
| Spec-04 type-form value | lands in S8 | `SemanticDenotation::TypeForm` + static value-side reflection type; no user-overridable semantic application |

## B.1 No hidden expression fallthrough

Current `synthesize_typed_expr` has a final unchecked-expression fallback. The target analyzer may still return an explicit `Unknown(UncheckedExpression)` for deliberately gated syntax, but acceptance testing must enumerate every public executable AST variant.

Add a test/tooling guard that fails when a new public `Expr` variant is added without an explicit 04.5 analysis policy. The implementation technique can be an exhaustive match without `_` once Spec-04 recovery variants stabilize.

---

# Appendix C — Concrete repository evidence at the inspected snapshot

This appendix records the implementation seams used to derive this plan. It is not a substitute for E0 re-inspection after Spec 04 lands.

## C.1 Formal checker

`phalcom-semantic/src/checker/mod.rs`

- exposes `check_program`;
- pre-registers class surfaces;
- creates one `CheckingContext`;
- aggregates only `ctx.diagnostics` into `TypeCheckReport`.

`phalcom-semantic/src/checker/context.rs`

- `LocalEnv` is `HashMap<String, ValueSemanticFact>`;
- `CheckingContext` owns `LocalConstraintSolver`;
- `resolve_dispatch` already specializes applied receiver parameter/return types.

`phalcom-semantic/src/checker/typed_expr.rs`

- `TypedExpression` already combines type knowledge, denotation, dispatch lookup, constraints, and provenance.

`phalcom-semantic/src/checker/call.rs`

- `match_callable_arguments` synthesizes arguments independently;
- checks only concrete parameter knowledge;
- returns `signature.return_type` directly.

`phalcom-semantic/src/checker/statement.rs`

- annotation/RHS assignability is checked on let declarations;
- `For` currently binds lane variables as explicit dynamic escape;
- many statement forms are not yet handled by the formal checker.

`phalcom-semantic/src/checker/declaration.rs`

- surface registration re-resolves annotations;
- callable body checking re-binds parameters from syntax annotations;
- last expression may be synthesized once as a statement and again for return checking.

## C.2 Canonical generic/type substrate

`phalcom-semantic/src/types/parameter.rs`

already contains:

```text
TypeParameterOwner
TypeParameterData
GenericSignature
GenericConstraint
SelfTypeTerm
TypeTerm
```

including callable-owned parameters, variance, constraints, `Self`, and an inference term hook.

`phalcom-semantic/src/types/environment.rs`

already contains `TypeEnvironment`, `TypeView`, and specialized callable viewing machinery. This is the receiver-specialization substrate; 04.5 should extend it rather than copy it.

`phalcom-semantic/src/types/store.rs`

already contains canonical:

```text
Applied
Union
Tuple
Record
Callable
Parameter
Lambda
SelfType
```

and currently also `Infer`, which 04.5 removes from canonical publication.

## C.3 Relation/outcome substrate

`phalcom-semantic/src/types/outcome.rs` already defines bounded outcomes, budgets, block reasons, and relation evidence.

`phalcom-semantic/src/types/relation.rs` already handles nominal/applied variance, unions, generic superclass templates, tuples, records, callables, cycles, cancellation, and budgets.

04.5 must not create a second subtype relation inside `InferenceSession`; the local solver generates/solves variable constraints and delegates fully concrete canonical relations to these APIs.

## C.4 Incremental database

`phalcom-semantic/src/db/key.rs` already includes `CallableBody(CallableId)`.

`phalcom-semantic/src/db/mod.rs` already enforces revision-safe publication and reverse dependency invalidation.

`phalcom-semantic/src/db/state.rs` already has non-success query states suitable for blocked/cancelled/budget-aware body analysis.

## C.5 Snapshot/compiler boundary

`phalcom-semantic/src/snapshot.rs` currently publishes:

```text
store
sources
surfaces
dispatch
declarations
hierarchy
diagnostics
semantic_graph
status
```

04.5 adds body/call/explanation products here rather than creating an LSP-only semantic snapshot.

`phalcom-core/src/modules/compile.rs` already builds an `AnalyzedProgram` with an `Arc<SemanticSnapshot>`. Semantic errors are grouped as `ProgramSemanticDiagnostics`, but their `thiserror` presentation is not the rich source renderer. This is the CLI integration seam.

## C.6 LSP duplicate/advisory analysis

`phalcom-lsp/src/semantic/flow.rs` is already a large structured flow implementation.

It has formal-looking concepts such as `FlowState` and `StatementFlow`, but its values come from `InferredValue`/`ValueShape`.

`phalcom-lsp/src/semantic/facts.rs` explicitly describes `ValueShape` as advisory and not a language type. It also has bounded unions and provenance/confidence.

`phalcom-lsp/src/semantic/infer.rs` already demonstrates a bounded, incremental, cancellation-aware callable worklist. Its scheduling ideas should be adapted into compiler-owned queries where useful.

The architectural conclusion is not “throw it away”; it is:

```text
promote formal semantics to phalcom-semantic
retain useful advisory tooling semantics in phalcom-lsp
remove overlapping formal claims after parity
```

## C.7 Diagnostics renderer

`phalcom-core/src/diagnostics/style.rs` is the single ANSI styling substrate and uses semantic roles.

`phalcom-core/src/diagnostics/caret.rs` implements display-column-correct source snippets with primary/secondary labels, Unicode/ASCII glyphs, tab expansion, and width windowing.

`phalcom-core/src/diagnostics/traceback.rs` uses that substrate for the runtime experience whose visual language 04.5 adopts.

The accepted PDR-0014 requires the renderer to remain in-house and explicitly rejects reintroducing `miette` as a second presentation system.

---

# Appendix D — Review checklist before implementation starts

The implementer should answer every item with repository evidence after Spec 04 lands:

- [ ] What exact AST identity will back `ExpressionId`?
- [ ] Where is a method's canonical `GenericSignature` stored/read by `CallableId`?
- [ ] Does `CallableSignature` still duplicate canonical signature data after Spec 04?
- [ ] Has 03.5 removed `register_standard_surfaces`, or must 04.5 temporarily coexist with it?
- [ ] Which control-flow selectors have canonical intrinsic/control annotations after 03.5?
- [ ] What exact source forms compile to non-local block returns?
- [ ] What is the authoritative iteration element semantic API for `for`?
- [ ] Is `None` represented as nominal singleton, special type form, or another canonical semantic fact relevant to narrowing?
- [ ] Which expression/recovery variants were added by Spec 04?
- [ ] Does the type-form value island introduce a new `Expr::TypeForm` exactly as proposed, or a different wrapper?
- [ ] Can `SemanticDb` store typed `Arc` products directly yet, or does 04.5 need a typed side arena owned by the DB revision?
- [ ] Which source resolver gives CLI diagnostics display paths for all linked modules?
- [ ] Which LSP handlers still read `ValueShape` for formal type claims?
- [ ] Which tests assert the current human semantic-error `Debug` representation and need migration?
- [ ] Has any newer work added a diagnostics crate already?

No unresolved answer should be papered over by inventing a second representation. If a dependency is not yet implemented, gate the relevant workstream and land an earlier independent workstream instead.
