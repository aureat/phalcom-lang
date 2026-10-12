---
id: LANG007.C4.P7
category: LANG
program: LANG007
checkpoint: LANG007.C4
kind: implementation-plan
title: "Recursive Generic Static Computation, Normalized Residualization, Specialization, and Memoization"
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
branch: main
prepared: 2026-10-12
prepared_against_revision: 3f8cd71358b5c5155ac0a92c7ab232e5a86c4fb6
requires:
  - LANG007.C4.P2
  - LANG007.C4.P3
  - LANG007.C4.P5
  - LANG007.C4.P6
---

# LANG007.C4.P7 — Luna Patch-Grade Implementation Plan

## 1. Purpose

Implement recursive generic static computation on top of the completed C4 static evaluator, replacing the current coarse recursion/symbolic seams with exact application-key cycle detection, evaluator-local memoization, canonical normalized residuals, deterministic specialization, bounded growth, and diagnostic-quality cycle/resource reporting.

P7 must make parameterized `type` definitions practical reusable compile-time programs rather than only closed or non-recursive evaluator entries.

The implementation must preserve the architecture already established by P2–P6:

```text
source syntax
    ↓
ordinary semantic checker
    ↓
CheckedStaticBody
    ├─ ExpressionAnalysisIndex
    ├─ BindingAnalysisIndex
    ├─ CheckedStaticApplication
    ├─ MatchResolutionIndex
    ├─ PatternSiteResolutionIndex
    ├─ StaticIterationLaneResolutionIndex
    ├─ ProofPredicateResolutionIndex
    └─ semantic dependencies
    ↓
StaticEvaluator
    ├─ explicit Work stack
    ├─ SavedFrame
    ├─ StaticValueStore
    ├─ QueryBudget
    └─ CancellationToken
```

P7 extends that pipeline with:

```text
StaticEvaluator
    ├─ exact StaticApplicationKey
    ├─ active-application cycle index
    ├─ evaluator-local application memo
    ├─ StaticResidualStore
    ├─ semantic residual normalization
    └─ residual specialization
```

P7 must **not** introduce a second checker, a second pattern resolver, a second conformance engine, a VM-side static evaluator, or serialized/durable residual metadata. Durable cross-module residual/value representation remains P8.

---

# 2. Audited Current Baseline

## 2.1 Repository state

Prepared against:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   3f8cd71358b5c5155ac0a92c7ab232e5a86c4fb6
subject:    feat(LANG007.C4.P6): implement static control flow, structural iteration, and proof predicates
```

The C4 checkpoint currently records:

```text
active_plan:            LANG007.C4.P7
latest_completed_plan:  LANG007.C4.P6
next_plan:              LANG007.C4.P7
```

P6 is recorded as:

```text
COMPLETE / IMPLEMENTED / FOCUSED_TESTED
```

The P6 handoff establishes:

- canonical match/pattern products on checked static artifacts;
- static structural iteration products;
- static pattern evaluation over canonical `PatternResolution`;
- branch proof transport in `StaticEvaluator`;
- explicit-frame `match` / `for` / `while` / return/break/continue execution;
- checked proof predicate sites;
- canonical generic conformance evaluation for `Conforms`;
- current `Stop::Symbolic` propagation as the handoff point to P7.

## 2.2 Current evaluator recursion seam

`StaticEvaluator` currently carries:

```rust
active_definitions: Vec<DeclarationId>,
saved_frames: Vec<SavedFrame<'a>>,
pending_application_arguments: BTreeMap<StaticApplicationId, Vec<Option<StaticArgument>>>,
```

and nested source static applications run through explicit work items:

```rust
Work::StartStaticApplication(...)
Work::EnterStaticApplication(...)
Work::FinishStaticApplication(...)
Work::AbortStaticApplication(...)
```

The current recursion check is:

```rust
if self.active_definitions.contains(&application.target) {
    return Err(Stop::Recursive);
}
```

This intentionally rejects all same-definition re-entry. P7 must replace this rule because legal recursive static computation requires changing-argument recursion.

## 2.3 Current evaluator terminal seam

Current public outcome:

```rust
pub enum StaticEvalOutcome {
    Reduced(StaticValueRef),
    ReducedKinded(TypeId),
    Symbolic,
    Invalid(StaticEvalError),
    Recursive,
    BudgetExceeded(BudgetReport),
    Cancelled,
}
```

This is intentionally too coarse for P7.

P7 requires at least:

```rust
Symbolic(StaticResidualRef)
Recursive(StaticCycle)
```

with canonical residual and cycle payloads.

## 2.4 Current static application identity

`CheckedStaticApplication` already has:

```rust
pub struct CheckedStaticApplication {
    pub id: StaticApplicationId,
    pub expression: ExpressionId,
    pub source_range: SourceRange,
    pub target: DeclarationId,
    pub signature: StaticDefinitionSignature,
    pub arguments: Box<[CheckedStaticApplicationArgument]>,
}
```

`StaticApplicationId` is a **checked source-site identity**:

```rust
pub struct StaticApplicationId {
    pub body: StaticBodyId,
    pub ordinal: u32,
}
```

It is suitable for retrieving checked application facts.

It is **not** a P7 memo/cycle key.

Two source sites may invoke the same semantic application and should memo-hit. One source site may recursively invoke the same definition with different arguments and must produce different application keys.

## 2.5 Current static argument carrier

The evaluator currently receives:

```rust
pub enum StaticArgument {
    Kinded(TypeId),
    Value(StaticValueRef),
}
```

This is already sufficient for a session-local exact application identity, subject to explicit documentation that raw `TypeId` / `StaticValueRef` are **session-local canonical identities**, not P8 durable metadata identities.

## 2.6 Current proof environment

P6 transports canonical branch proofs through:

```rust
active_branch_proofs: Vec<&BranchProofEnvironment>
```

with:

```rust
pub struct BranchProofEnvironment {
    pub bindings: BTreeMap<TypeParameterId, TypeId>,
    pub equalities: Box<[TypeConstraint]>,
    pub local_bindings: BTreeMap<TypeParameterId, LocalType>,
    pub local_equalities: Box<[LocalConstraint]>,
}
```

The P7 application key must include the **relevant semantic specialization/proof environment** because an application result can differ under different established generic/GADT facts.

Do not use the entire checker context or blindly hash query-local rigid identities.

## 2.7 Current static value substrate

`StaticValueStore` already provides:

- store-qualified canonical `StaticValueRef`;
- interned immutable primitive/tuple/sequence/record/data/variant/Pack nodes;
- O(1) Pack slicing via `intern_pack_slice`;
- semantic equality/hash;
- content fingerprinting;
- canonical Type/StaticTerm values.

This is sufficient for P7 session-local memoization and reduced-term sharing.

P7 must not treat `StaticValueStore::content_fingerprint` as a durable P8 identifier. Type-bearing nodes currently fingerprint raw `TypeId` identity.

---

# 3. Mandatory P7 Takeover Hardening

P6 is lifecycle-complete, but current `main` retains several semantic seams that would become dangerous if encoded into residual/application identity. P7 therefore begins with a bounded takeover-hardening task.

These changes are part of P7 execution only because P6 is already closed; they are not license to redesign P6 generally.

## 3.1 Proof predicate recognition must become canonical

Current checker code still recognizes:

```rust
match reference.root.as_str() {
    "Equivalent" => ...
    "Subtype" => ...
    "Conforms" => ...
}
```

This cannot be the authority feeding P7 residuals.

Required law:

```text
source syntax
    ↓
ordinary name/declaration resolution
    ↓
canonical compiler-owned proof predicate identity
    ↓
ProofPredicateResolution
```

not:

```text
raw source spelling
    ↓
ProofPredicateKind
```

### Required patch

Primary files:

```text
phalcom-semantic/src/checker/proof_predicate.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/core_surface/**
```

Use canonical declaration identities returned by the normal resolver. Keep `proof_predicate_declaration(kind)` only if those declarations are actual canonical surface identities and resolution proves the expression denotes exactly one of them.

Shadowing or a user declaration named `Equivalent`/`Subtype`/`Conforms` must not be silently reclassified as the compiler predicate.

## 3.2 `Conforms` must retain a canonical trait proposition

Current product is:

```rust
pub struct ProofPredicateResolution {
    pub kind: ProofPredicateKind,
    pub left: TypeId,
    pub right: TypeId,
    ...
}
```

For `Conforms`, `right` is reconstructed into a `TraitRef` from a type representation during evaluation.

That is too indirect for P7 residual identity.

Prefer one of:

```rust
pub enum ProofPredicateGoal {
    Equivalent {
        left: TypeId,
        right: TypeId,
    },
    Subtype {
        lower: TypeId,
        upper: TypeId,
    },
    Conforms {
        key: GenericConformanceKey,
    },
}
```

or a faithful reuse of `GenericConstraint` if it already exactly matches the public proof-predicate proposition.

Then:

```rust
pub struct ProofPredicateResolution {
    pub expression: ExpressionId,
    pub range: SourceRange,
    pub goal: ProofPredicateGoal,
    pub result_type: TypeId,
}
```

P7 residual proof nodes must retain canonical semantic goals, not reconstruct them from source syntax.

## 3.3 `Equivalent` must use proof/equivalence authority, not raw identity fallback

Current evaluator behavior approximately performs:

```text
resolved left == resolved right
    -> true

unresolved parameter
    -> Symbolic

otherwise
    -> false
```

This is not sufficient for canonical equivalence established through proof constraints, normalization, associated projections, or other accepted equality evidence.

P7.T0 must route proof predicate evaluation through the repository's canonical proof/relation authority.

Preferred direction:

```text
Equivalent
    -> GenericConstraint::Equivalent / canonical equivalence proof session

Subtype
    -> existing bounded subtype relation

Conforms
    -> GenericConformanceKey + prove_conformance_with_context
```

Do not convert "not syntactically/canonically identical at this moment" into proof refutation.

## 3.4 Heterogeneous static iteration must preserve exact-item checker correlation

`StaticIterationItemDomain::FiniteExact` is currently collapsed by:

```rust
consolidated_item_knowledge(...)
```

to one representative/union `TypeKnowledge`, after which the body is checked once.

That loses the P6 invariant that a heterogeneous finite structural loop is valid under each exact reachable iteration refinement.

P7 depends on exact iteration semantics for source-defined structural algorithms and residual specialization.

Required correction in:

```text
phalcom-semantic/src/checker/static_iteration.rs
phalcom-semantic/src/checker/statement.rs
```

For `FiniteExact(items)`:

1. preserve the exact item sequence;
2. evaluate loop-body checker transfer under each exact item knowledge;
3. reuse ordinary pattern resolution for each item;
4. join the resulting flow states at the ordinary loop boundary;
5. publish a canonical pattern-site product that remains executable by the evaluator;
6. do not introduce a static-specific type checker.

Avoid cloning the entire `TypeStore` once per item. Reuse current flow/probe mechanisms and only clone/query-local state already required by the checker architecture.

## 3.5 Static `for` execution must consume its checked mode

Current evaluator builds a `Vec<StaticValueRef>` by inspecting the reduced `StaticValueData` variant and structural type data.

P7 must first make:

```rust
StaticIterationLaneResolution.mode
```

the semantic execution authority.

The reduced value/type representation may be validated against that mode, but must not independently decide what static iteration means.

Required invariant:

```text
checker publishes StaticIterationMode
    ↓
evaluator validates representation
    ↓
mode-specific cursor
    ↓
iteration
```

## 3.6 Replace eager `Vec<StaticValueRef>` loop state with cursor state

Current work variants retain the complete materialized item vector:

```rust
ForNext(..., Vec<StaticValueRef>, usize)
AfterForBody(..., Vec<StaticValueRef>, usize, ...)
```

This is avoidable O(n) auxiliary storage and is a poor substrate for recursive/residual static algorithms.

Introduce an internal cursor, e.g.:

```rust
enum StaticIterationCursor {
    Pack {
        pack: StaticValueRef,
        positional_index: u32,
        labeled_index: u32,
    },
    Tuple {
        tuple: StaticTupleId,
        index: u32,
    },
    Sequence {
        sequence: StaticSequenceId,
        index: u32,
    },
    Record {
        record: StaticRecordId,
        index: u32,
    },
    Data {
        data: StaticDataValueId,
        index: u32,
    },
    TypeOrdered {
        subject: TypeId,
        index: u32,
    },
    TypeUnordered {
        subject: TypeId,
        index: u32,
    },
}
```

Exact private representation may differ.

Requirements:

- O(1) cursor state;
- O(n) total traversal;
- Pack rest/subrange operations reuse `StaticPackData` slices;
- cancellation/budget safe point per iteration;
- no repeated structural reclassification per iteration.

## 3.7 Open unordered tails: enforce the canonical fixed-field rule

The C4 requirements state:

```text
anonymous open tails are not iterable finite collections;
fixed fields remain iterable even when a tail is open.
```

Current checker/evaluator rejects non-closed unordered structures wholesale.

P7 takeover hardening must converge to the normative law before residual structural algorithms rely on the behavior.

Required semantics:

```text
fixed fields:
    finite, deterministic, iterable

anonymous open tail:
    not enumerated

captured concrete tail:
    iterable only when the captured tail is itself exact finite structure
```

Do not invent entries for an open tail.

## 3.8 Remove empty-semantic-authority fallbacks where they alter meaning

`resolve_static_iteration_lane` currently substitutes empty Data/Enum tables when those semantic authorities are absent.

Absence of semantic authority is not equivalent to an empty program.

When classification requires unavailable semantic data, preserve:

```text
Unknown / Blocked / unavailable semantic dependency
```

rather than definitively classifying the structure as unsupported.

## 3.9 Record pattern `FieldKey` and Pack structural pattern gates

Current ordinary record semantic product still stores:

```rust
ResolvedRecordFieldPattern {
    label: Box<str>,
    ...
}
```

while C4 generalized unordered keys use canonical `FieldKey`.

P7 must not invent a residual-only correction.

For any P7 acceptance involving generalized Record structural patterns, require the existing P4 migration to:

```rust
ResolvedRecordFieldPattern {
    key: FieldKey,
    ...
}
```

Similarly, current ordinary pattern language does not expose a canonical Pack structural pattern matching the historical recursive `Count<*P>` example.

P7 must not add a private residual/static-only Pack pattern.

For P7 acceptance:

- if ordinary Pack structural patterns are available before implementation reaches recursive structural probes, use them;
- otherwise certify recursion first with existing ordinary static syntax and the iterative Pack path;
- leave the exact `Pack(_, *rest)` source acceptance gated rather than fabricating P7-only grammar.

---

# 4. P7 Architectural Invariants

The implementation must preserve these invariants.

## P7-I1 — One evaluator

P7 extends `StaticEvaluator`. It does not create a second residual evaluator that reinterprets source AST.

## P7-I2 — One pattern language

Residual `match` consumes the canonical P6 match/pattern products. No `StaticPattern` or `ResidualPattern` resolver may be introduced.

## P7-I3 — One proof system

Residual proof evaluation invokes existing equivalence/subtype/conformance authorities. No P7 proof mini-engine.

## P7-I4 — Exact cycle identity

Only exact repetition of:

```text
definition
+
canonical arguments
+
relevant specialization/proof environment
```

is a recursion cycle.

Definition-name repetition alone is not.

## P7-I5 — Host stack safety

Source recursion uses explicit evaluator work/frame state. Guest recursion must not grow the Rust call stack.

## P7-I6 — Residuals are semantic IR

Residuals contain no:

```text
raw AST pointers
Work values
SavedFrame
source parser nodes as semantic identity
local VM values
runtime ObjRef
cancellation token
budget state
memo table
```

## P7-I7 — Residual IR is smaller than source AST

Only forms genuinely necessary to preserve unresolved static semantics are represented.

## P7-I8 — Execution-context terminals are not residuals

`Cancelled` and `BudgetExceeded` never become valid residual terms.

Transient `Blocked` is not silently converted into a durable semantic residual unless the blocked reason is itself a semantically valid unresolved generic dependency.

## P7-I9 — P7 identity is session-local; P8 owns durability

P7 may use canonical `TypeId` and `StaticValueRef` inside one semantic store lineage.

P7 must not serialize them or claim cross-session identity.

## P7-I10 — Cold exact equals symbolic specialization

For supported cases:

```text
evaluate exact application from checked definition
==
specialize canonical residual with exact arguments
```

in semantic result and diagnostics.

## P7-I11 — Memoization is evaluator-local by default

Do not add `QueryKey::StaticApplication` globally unless profiling proves a need and the lifecycle plan is amended.

## P7-I12 — Relevant proof environment participates in application identity

Applications under semantically different branch/generic proof facts cannot collide.

## P7-I13 — No runtime/VM residual semantics

P7 finishes or rejects static computation before runtime compiler lowering. The VM does not execute residuals.

## P7-I14 — Determinism

Equivalent semantic inputs produce equal residual structure independent of:

```text
source formatting
incremental history
hash-map iteration order
allocation order inside transient work structures
```

## P7-I15 — Bounded growth

Steps alone are not sufficient when a program can create exponentially growing residual/value graphs. Residual node growth must be explicitly bounded.

---

# P7 Level 2 amendment — composite kinded TypeForm specialization (2026-10-12)

This amendment follows the architecture consultation recorded in
[`LANG007.C4.P7-INC-001-composite-static-arguments.md`](LANG007.C4.P7-INC-001-composite-static-arguments.md).
It supersedes the P6/T7.10 immutable-evaluator restriction only for P7 static
evaluation.

### Canonicalization owner

- The active `SemanticWorkspaceSession` `TypeStore` remains the sole canonical
  interning authority for P7 evaluation.
- The session already owns a baseline fork. Evaluation must receive an
  exclusive mutable borrow of that active store (`Arc::make_mut(&mut
  self.store)` at the workspace-update boundary); do not make another
  per-evaluation `fork_from_baseline` or maintain a parallel type
  canonicalizer.
- Copy-on-write preserves every previously published snapshot. Newly
  specialized forms are interned in the current session lineage and may be
  used by its evaluator, static values, and products under the existing
  snapshot/store-local `TypeId` contract.
- Do not allow generated IDs to cross into another session or snapshot without
  that store context. P8 still owns durable/cross-module identity.

### Composite specialization

- Before an application key is formed, recursively substitute the active
  definition's kinded arguments through checked composite TypeForms using the
  existing `TypeSubstitution` and canonical `TypeStore` formation APIs.
- Preserve kind validation and fail closed on malformed checked products or
  unsupported forms. Do not infer, resolve, or synthesize source types in the
  evaluator.
- Precharge the existing query step budget in proportion to the checked
  TypeForm/application source span before recursive substitution and canonical
  interning. Check cancellation before and after the bounded substitution call.
  Budget/cancellation terminals must not become memoized results or published
  static products.
- T3 exact-cycle identity is computed from the specialized canonical
  arguments. `Grow<T> -> Grow<Boxed<T>>` must progress until a base case or the
  query budget, while exact direct and mutual cycles remain `Recursive`.

### Scope of the P6/T7.10 amendment

P6's historical fail-closed behavior remains the P6 completion boundary. P7
may now specialize composite parameter-dependent TypeForms through the active
session-owned copy-on-write store. The prohibition on mutating or publishing a
parallel clone-qualified TypeStore remains in force. The existing raw
`TypeId` result and type-bearing static-value interfaces remain valid because
the evaluator and its products share the current session store lineage.

This is a prerequisite to accepting P7.T3/G3 and T5/G5. It does not waive the
focused snapshot-immutability, budget, cancellation, exact-cycle, and cold
versus-residual-specialization checks named by those gates.

# 5. Target P7 Data Model

The exact Rust spelling may vary, but implementation should converge on the following concepts.

## 5.1 Residual store identity

Create:

```text
phalcom-semantic/src/static_eval/residual.rs
```

with store-qualified identity patterned after `StaticValueStore`:

```rust
pub struct StaticResidualStoreId(u64);

pub struct StaticResidualId(u32);

pub struct StaticResidualRef {
    store: StaticResidualStoreId,
    residual: StaticResidualId,
}
```

Requirements:

- store-qualified refs reject cross-store mixing;
- append-only/interned nodes;
- cloning a semantic snapshot/store lineage preserves valid existing refs if the surrounding ownership model requires snapshot cloning;
- node equality/hash is structural semantic identity;
- no source spans in node identity.

If snapshot ownership makes a separate global store ID awkward, an alternative session-owned arena is acceptable, but cross-store misuse must remain detectable.

## 5.2 Static term envelope

Introduce a domain-preserving term envelope:

```rust
pub enum StaticEvalTerm {
    Value(StaticValueRef),
    Kinded(TypeId),
    Residual(StaticResidualRef),
}
```

If a residual can represent both value-domain and kinded results, retain result domain explicitly either:

```rust
StaticResidualData::<variant> { result_domain: StaticValueDomain, ... }
```

or through a store-side node header.

Do not infer a residual's result domain by re-running the checker.

## 5.3 Residual parameter identity

Do not reuse a source parameter name as semantic identity.

Conceptually:

```rust
pub struct StaticResidualParameter {
    pub owner: DeclarationId,
    pub index: u32,
    pub domain: StaticValueDomain,
}
```

For kinded parameters, retain the existing `TypeParameterId` bridge only as needed; the residual parameter identity must remain stable within the checked static definition.

## 5.4 Minimal initial residual enum

Start minimal:

```rust
pub enum StaticResidualData {
    Parameter {
        parameter: StaticResidualParameter,
    },

    Apply {
        key: StaticResidualApplication,
        result_domain: StaticValueDomain,
    },

    Proof {
        goal: ProofPredicateGoal,
    },

    MemberProjection {
        projection: MemberProjection,
        result_domain: StaticValueDomain,
    },
}
```

Only after these work, extend with:

```rust
If {
    condition: StaticResidualTerm,
    then_term: StaticResidualTerm,
    else_term: StaticResidualTerm,
    result_domain: StaticValueDomain,
}

Match {
    scrutinee: StaticResidualTerm,
    semantics: StaticResidualMatch,
    result_domain: StaticValueDomain,
}

Intrinsic {
    operation: StaticIntrinsicOperation,
    arguments: Box<[StaticResidualTerm]>,
    result_domain: StaticValueDomain,
}

Pack { ... }
Tuple { ... }
Record { ... }
Data { ... }
Variant { ... }
Sequence { ... } // only if real P7 residual cases require it
```

Do not implement every illustrative enum case up front.

---

# 6. Static Application Identity

## 6.1 New semantic key

Create:

```text
phalcom-semantic/src/static_eval/application.rs
```

or keep the types in `residual.rs` if the module split would be artificial.

Conceptually:

```rust
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StaticApplicationKey {
    pub definition: DeclarationId,
    pub arguments: Box<[StaticArgumentIdentity]>,
    pub environment: StaticSpecializationEnvironmentKey,
}
```

## 6.2 Argument identity

Use a distinct type to avoid confusing evaluator arguments with type-system static arguments:

```rust
pub enum StaticArgumentIdentity {
    Kinded(TypeId),
    Value(StaticValueRef),
}
```

Within one `TypeStore` / `StaticValueStore` lineage, these are exact canonical identities.

Document explicitly:

```text
StaticArgumentIdentity = evaluator-local/session-local semantic identity
StaticArgumentRef      = metadata/durable identity (P8 concern)
```

## 6.3 Environment identity

Add:

```rust
pub struct StaticSpecializationEnvironmentKey {
    pub argument_bindings: Box<[(StaticResidualParameter, StaticSpecializationArgument)]>,
    pub type_bindings: Box<[(TypeParameterId, TypeId)]>,
    pub constraints: Box<[CanonicalStaticProofFact]>,
}

pub enum StaticSpecializationArgument {
    Kinded(TypeId),
    Value(StaticValueRef),
}
```

The exact carrier should be based on current canonical proof machinery, not
invented duplication. `argument_bindings` identifies exact kinded/value
arguments by their stable formal static parameter (`owner`, `index`, and
`domain`). It excludes residual arguments: those remain represented in the
residual term itself and must not be mislabeled as exact substitutions.

Projection rules:

1. collect only facts capable of changing this application's result;
2. materialize/canonicalize branch proof equalities where possible;
3. normalize ordering deterministically;
4. exclude source ranges;
5. exclude local flow metadata;
6. exclude explanation IDs;
7. exclude cancellation/budget state;
8. exclude query-local rigid identities unless converted to canonical semantic facts.

If `local_bindings` / `local_equalities` cannot be canonicalized safely, do not hash raw `LocalType`/rigid identities. Instead:
- materialize the subset expressible in canonical `TypeId`/`TypeConstraint`, or
- classify the application as not memo-shareable across that proof environment and use an explicit conservative environment identity local to the active branch.

STOP if correctness appears to require hashing the entire `CheckingContext`.

---

# 7. Exact Cycle Detection

## 7.1 Replace definition-only activity state

Remove:

```rust
active_definitions: Vec<DeclarationId>
```

as the recursion authority.

Replace with:

```rust
active_applications: Vec<StaticApplicationKey>,
active_application_index: HashMap<StaticApplicationKey, usize>,
```

A `BTreeMap` is acceptable if deterministic ordering/debuggability is preferred over constant-factor speed; the lookup itself is not serialized.

## 7.2 Root application

The root `StaticEvaluator::new(body, arguments)` must synthesize its root `StaticApplicationKey` from:

```text
body.owner
arguments
initial specialization/proof environment
```

so direct self-recursion can point back to the root exact key.

Constants and independently checked static expressions are not definition applications and should not be inserted as fake `StaticApplicationKey`s.

## 7.3 Enter algorithm

Before `load_frame`:

```text
construct exact key
charge application work
check cancellation

if key already active:
    construct StaticCycle
    return Recursive(cycle)

if key has deterministic memo result:
    consume memo result without entering frame

otherwise:
    mark key active
    push saved frame
    load target frame
```

## 7.4 Exit algorithm

On every normal/error/control path:

```text
unmark exact active key
restore caller frame
optionally memoize deterministic completed result
```

Cleanup must occur for:

```text
ordinary return
implicit expression completion
propagated return
Invalid
Recursive
Symbolic
```

Cancellation/Budget may abort the entire root evaluation, but active/memo state must still remain internally consistent if reused by diagnostics before drop.

## 7.5 Static cycle payload

Add:

```rust
pub struct StaticCycle {
    pub applications: Box<[StaticCycleFrame]>,
    pub cycle_start: u32,
}

pub struct StaticCycleFrame {
    pub definition: DeclarationId,
    pub arguments: Box<[StaticArgumentSummary]>,
    pub source_range: Option<SourceRange>,
}
```

Do not retain giant full pretty-printed argument strings.

The cycle object may retain exact keys internally and format summaries lazily.

Required tests:

```text
F<A> -> F<A>                 => Recursive(cycle)
F<A> -> G<A> -> F<A>         => Recursive(cycle)
F<A> -> F<B> -> base         => succeeds
F<0> -> F<1> -> F<2> -> ...  => BudgetExceeded, not Recursive
```

---

# 8. Evaluator-Local Memoization

## 8.1 Memo ownership

Add to `StaticEvaluator`:

```rust
application_memo: HashMap<StaticApplicationKey, StaticMemoResult>,
```

Do not make each recursive application a semantic DB query in P7.

## 8.2 Memo result

Use a result that preserves deterministic reduced/symbolic outcomes:

```rust
pub enum StaticMemoResult {
    Reduced(StaticEvalTerm),
    Symbolic(StaticResidualRef),
}
```

Initially keep deterministic `Invalid` / `Recursive` uncached unless implementation demonstrates stable diagnostic/dependency semantics.

Never memoize as semantic success:

```text
Cancelled
BudgetExceeded
transient Blocked
```

## 8.3 Memo lookup point

Perform lookup only after:

```text
all application arguments evaluated/normalized
application key built
environment key built
argument domains validated
```

Do not memoize by `StaticApplicationId`.

## 8.4 Memo insertion point

Memoize only after:

```text
target result-domain validation
caller frame restoration
normalization of any residual result
```

If an application produces a residual that embeds the identical application as an unresolved self-reference, run exact-cycle logic before interning to avoid silently creating a cyclic residual graph.

Residual store itself should remain an acyclic DAG.

## 8.5 Tests

Add an internal evaluator counter/test hook or deterministic instrumentation sufficient to establish:

```text
same exact subproblem evaluated logically once
```

Do not expose production public counters merely for tests.

---

# 9. Outcome Model Cutover

## 9.1 Public outcome

Update:

```rust
pub enum StaticEvalOutcome {
    Reduced(StaticValueRef),
    ReducedKinded(TypeId),
    Symbolic(StaticResidualRef),
    Invalid(StaticEvalError),
    Recursive(StaticCycle),
    BudgetExceeded(BudgetReport),
    Cancelled,
}
```

If `StaticEvalTerm` is made public enough, a single:

```rust
Reduced(StaticEvalTerm)
```

could reduce enum duplication, but do not broaden the API solely for elegance. Minimize churn across existing P2–P6 consumers.

## 9.2 Internal stop

Current internal:

```text
Stop::Symbolic
Stop::Recursive
```

must also gain payloads or be replaced with a structured internal result path.

Preferred:

```rust
enum Stop {
    Symbolic(StaticResidualRef),
    Recursive(StaticCycle),
    ...
}
```

## 9.3 Mandatory consumers to update

Audit and patch all exhaustive matches, at minimum:

```text
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/static_eval/error.rs
phalcom-semantic/src/static_key.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/checker/static_body.rs
phalcom-semantic/src/static_constant.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/tests/**
```

`StaticKeyProjection::DeferredToP7` should be retired or renamed once a real residual exists.

For a computed Record key:

- a fully reduced residual specialization can become a concrete `FieldKey`;
- an unresolved generic key remains symbolic;
- a closed constant requiring a concrete key must diagnose rather than publish a fake key.

---

# 10. Residual Normalization

All residual construction must go through normalization/interning helpers. Do not directly allocate arbitrary nodes from many evaluator sites.

Suggested API:

```rust
impl StaticResidualStore {
    fn parameter(...)

    fn apply(...)

    fn proof(...)

    fn member_projection(...)

    fn if_then_else(...)

    fn match_term(...)

    fn intrinsic(...)

    fn pack(...)

    ...
}
```

Each helper:

```text
1. validates store/domain ownership
2. substitutes immediately-known terms
3. folds reducible forms
4. canonicalizes child order where semantics is unordered
5. reuses interned equivalent nodes
6. charges residual growth budget
7. checks cancellation at bounded intervals
```

## 10.1 Normalization laws

### Apply

```text
all arguments exact and target can reduce
    -> evaluate through ordinary evaluator/memo

equivalent normalized application
    -> same residual ref
```

### Proof

```text
Proven    -> Bool(true)
Refuted   -> Bool(false)
Unknown generic proposition -> Proof residual
Blocked erroneous/incomplete dependency -> blocked/invalid terminal
Dynamic boundary -> symbolic only if static semantics explicitly permits;
                    otherwise closed-context invalid
```

### If

```text
true  -> then
false -> else

if C then X else X
    -> X
```

only when X is semantically identical and branch proof environments do not make the two occurrences observably different.

### Match

When scrutinee becomes reduced:

```text
run P6 match_pattern_against_value
select first matching canonical arm
apply arm proof environment
normalize selected branch
```

Do not evaluate untaken arms.

### Products/Packs

When all children are reduced:

```text
residual product
    -> canonical StaticValueStore node
```

Pack identity/rest views must reuse existing Pack backing/slicing APIs.

---

# 11. Minimal Residual Vertical Slice

Do not begin with general residual control flow.

Implement a narrow vertical slice first.

## 11.1 Parameter residual

When a static definition parameter is valid but unavailable during generic/open evaluation:

```text
parameter read
    -> Residual::Parameter
```

rather than bare `Stop::Symbolic`.

## 11.2 Application residual

For a generic static application whose arguments contain residual terms:

```text
Apply(F, residual_args)
```

is valid even when the target cannot yet execute.

This single form can defer substantial imperative/static-body computation until arguments specialize, avoiding premature residual CFG design.

## 11.3 First acceptance example

Use a source definition whose body is already supported and whose open result can be represented with only `Parameter`/`Apply`.

The first P7 acceptance gate should prove:

```text
open application
    -> canonical Apply residual

same open application twice
    -> same normalized residual identity

specialize with exact args
    -> same result as cold exact evaluation
```

Do not make `match`/`for` residual support a prerequisite to proving the core application-key architecture.

---

# 12. Proof Residuals

## 12.1 Canonical goal

After P7.T0, residual proof nodes carry:

```rust
ProofPredicateGoal
```

or canonical `GenericConstraint`.

## 12.2 Evaluation

Use the existing proof engines and active specialization environment.

Mapping:

```text
Proven                  -> true
Refuted / Missing       -> false
Unknown due to open generic facts
                        -> normalized Proof residual
Blocked dependency      -> blocked/invalid according to semantic cause
Dynamic                 -> residual only where language semantics permits it
Cancelled               -> Cancelled
BudgetExceeded          -> BudgetExceeded
InternalFailure         -> Invalid/InternalFailure path
```

Do not map all `Unknown`/`Blocked`/`Dynamic` to one indistinguishable residual.

## 12.3 Structural conformance

`Conforms<T, Product>` and ordinary user-trait `Conforms<T, SomeTrait<A>>` must use the same `GenericConformanceKey`/proof entry point.

Compiler-derived structural evidence from P5 must continue to work without synthetic impl fabrication.

---

# 13. Member Projection Residuals

P5 already exposes:

```rust
StaticMemberProjectionOutcome::Stuck(MemberProjection)
```

This is a canonical semantic unresolved operation.

P7 must map it directly to:

```rust
StaticResidualData::MemberProjection {
    projection,
    ...
}
```

Do not collapse to bare symbolic and reconstruct later from source.

Specialization:

```text
substitute exact subject/arguments
    ↓
StructuralReflectionProjector / canonical member projector
    ↓
ReducedValue / ReducedKinded / still Stuck / terminal
```

This is a priority residual form because it allows generic reflection utilities to remain semantic and compiler-assisted.

---

# 14. Conditional Residuals

Add residual `If` only after Parameter/Apply/Proof/MemberProjection are stable.

Required evaluator behavior:

```text
condition reduced Bool
    -> current P6 branch execution

condition residual Bool
    -> residualize semantically
```

Do not execute both branches merely to build a residual if doing so would violate selected-branch semantics.

Preferred approach:

1. residualize branch bodies under isolated evaluator frames;
2. preserve their result domains;
3. do not commit side effects/local mutations from speculative branch evaluation to caller state;
4. reject/rescope if branch-local imperative state cannot be represented safely.

A simpler first implementation may leave the enclosing application as `Apply` rather than creating `Residual::If` for imperative block bodies.

This is acceptable and preferable to a suspended evaluator continuation.

---

# 15. Match Residuals

## 15.1 Canonical match authority

Use:

```text
CheckedStaticBody.match_resolutions
MatchResolution
MatchArmResolution
PatternResolution
BranchProofEnvironment
```

No source re-resolution.

## 15.2 Reduced scrutinee

Continue current P6 execution.

## 15.3 Residual scrutinee

Only introduce a `Match` residual if the body can be represented as normalized semantic branch terms.

Possible internal projection:

```rust
pub struct StaticResidualMatch {
    pub resolution_owner: StaticBodyId,
    pub expression: ExpressionId,
    pub scrutinee: StaticResidualTerm,
    pub arms: Box<[StaticResidualMatchArm]>,
}
```

The residual must not rely on a raw AST pointer for semantic identity.

If current session-local residuals need to retain a compact `MatchResolution` clone/reference, that is acceptable for P7 as long as:

- equality/hash does not depend on pointer address;
- P8 is explicitly responsible for durable projection;
- no raw parser node becomes semantic identity.

## 15.4 Branch proofs

When specialization makes a scrutinee concrete and selects an arm:

```text
push arm BranchProofEnvironment
evaluate/normalize arm
pop proof environment
```

Application keys created inside the arm must include relevant proof facts.

---

# 16. `for` / `while` Residual Policy

Do not create a residual imperative continuation unless forced by a ratified semantic requirement.

## 16.1 Concrete finite `for`

Execute through the P6 cursor path.

## 16.2 Unresolved `for` source

Preferred P7 behavior:

```text
residualize the enclosing static application
```

rather than:

```text
ResidualFor {
    saved_locals,
    statement_index,
    remaining_items,
    pending_control,
    ...
}
```

A suspended evaluator frame is not normalized semantic IR.

Only add `Residual::For` if an actual source-defined generic static algorithm cannot be represented through `Apply`/other semantic residual forms and an explicit architecture amendment approves a canonical loop residual.

## 16.3 `while`

Same rule.

Concrete static while continues to execute under QueryBudget.

Unknown condition should generally preserve the enclosing application instead of serializing loop machine state.

---

# 17. Structural Construction Residuals

Add product/Pack residual constructors only when required by actual generic static algorithms.

Priority order:

```text
Pack
Tuple
Record
Data
Variant
Sequence (if required)
```

## 17.1 Pack

Preserve:

```text
positional/labeled lane distinction
canonical labels
exact visible slice identity
```

If all children are reduced, intern directly into `StaticValueStore`.

Do not construct a linked-list/cons representation solely for recursion.

## 17.2 Record

Require canonical `FieldKey` for any generalized key acceptance.

Canonicalize unordered fields exactly as the semantic Record model requires.

## 17.3 Data/Variant

Retain canonical:

```text
DataConstructorId
DataComponentId
VariantId
VariantFieldId
```

Do not residualize selector spellings.

---

# 18. Residual Specialization

Create a focused specialization module if it improves separation:

```text
phalcom-semantic/src/static_eval/specialize.rs
```

or keep it in `residual.rs` until complexity justifies a split.

Conceptual API:

```rust
pub fn specialize_residual(
    residual: StaticResidualRef,
    substitutions: &StaticSpecializationEnvironment,
    context: &mut StaticEvalContext<'_>,
) -> StaticEvalOutcome
```

## 18.1 Specialization environment

Contains canonical, source-syntax-free:

```text
formal static parameter -> exact kinded/value argument bindings
canonical proof facts
```

Proof and MemberProjection residuals capture the effective environment at the
point where they are created. This makes either child residual self-sufficient
if it escapes directly from a static application without an enclosing Apply
node. Their environment participates in residual identity and is re-interned
after normalization.

When specialization supplies another environment, compose substitutions
through captured bindings before reevaluating the residual. Normalize by
formal-parameter identity and canonical proof fact; duplicate identical
bindings/facts coalesce, while conflicting concrete bindings fail through the
normal fail-closed semantic outcome. Captured and supplied proof facts are
canonically unioned and checked by the existing proof authority. Never use
last-writer-wins or store source ranges, source syntax, query-local rigid IDs,
or raw checker-local proof state.

## 18.2 Reuse ordinary semantic authorities

Specialization calls:

```text
StaticValueStore
StaticDefinitionTable
existing static application evaluation
generic proof engine
member projection engine
P6 static matcher
static intrinsic evaluator
```

It does not re-run parser/checker resolution.

## 18.3 Idempotence

Required:

```text
normalize(normalize(R)) == normalize(R)
```

and:

```text
specialize(R, no_new_facts)
    == normalized R
```

## 18.4 Cold exact equivalence

For each supported residual family:

```text
cold exact application
==
open evaluation -> residual -> specialize exact
```

Test semantic equality rather than incidental arena IDs when cold paths use distinct store lineages.

---

# 19. Static Evaluation Context Consolidation

`StaticEvaluator` currently accumulates optional semantic authorities:

```text
definitions
constants
canonical_types
semantic_tables
hierarchy
conformance
```

P7 adds further residual/specialization dependencies.

Before the constructor surface becomes unmanageable, consolidate them into a context object if doing so reduces partial-configuration errors.

Conceptual:

```rust
pub struct StaticEvalSemanticContext<'a> {
    pub types: &'a TypeStore,
    pub definitions: &'a StaticDefinitionTable,
    pub constants: &'a StaticConstantTable,
    pub data: &'a DataSemanticTable,
    pub enums: &'a EnumSemanticTable,
    pub hierarchy: &'a dyn TypeHierarchy,
    pub conformance_index: Option<&'a ConformanceIndex>,
    pub witness_plans: Option<&'a ...>,
    pub trait_surfaces: Option<&'a TraitSurfaceTable>,
    pub declarations: Option<&'a DeclarationTypeTable>,
}
```

Do not force contexts that genuinely lack a proof authority to fabricate empty tables.

The context should make missing authority explicit and map it to the correct terminal category.

---

# 20. Budget and Growth Accounting

Current `QueryBudget` has:

```text
Steps
RelationPairs
SccIterations
TypeDepth
DiagnosticNotes
```

P7 requires explicit residual growth protection.

## 20.1 Add residual-node budget

Extend:

```rust
BudgetKind::ResidualNodes
```

and `QueryBudget` with:

```rust
pub max_residual_nodes: u32,
pub residual_nodes_created: u32,
```

plus:

```rust
pub fn charge_residual_node(&mut self) -> Result<(), BudgetReport>
```

Use repository-consistent types/limits.

Do not overload `RelationPairs` or `SccIterations`.

## 20.2 Optional static-value growth budget

Measure first.

If recursive static programs can construct exponentially growing reduced `StaticValue` graphs while staying under the step limit, add:

```rust
BudgetKind::StaticValueNodes
```

with explicit accounting.

Do not add speculative dimensions without a reproducing benchmark/test.

## 20.3 Required charge points

At minimum:

```text
definition application entry
recursive frame push
memo insertion
residual node creation
residual specialization node visit
P6 loop iteration
pattern candidate application where not already charged
large product/Pack construction proportional to size
```

Avoid double-charging the exact same semantic unit through nested helpers.

## 20.4 Cancellation

Check cancellation:

```text
before application entry
before/inside residual normalization walks
once per loop iteration
once per large child sequence chunk
during specialization
during cycle diagnostic summarization if argument graphs are large
```

P7 must remain editor-safe.

---

# 21. Dependency and Incremental Semantics

## 21.1 Reuse checked-body dependencies

`CheckedStaticBody.dependencies` already records semantic dependencies including:

```text
StaticDefinition
StaticConstant
declaration/type/trait/conformance products
```

P7 evaluator-local memo entries do not become independent DB nodes.

A root evaluation depends on the already-published checked-body/static-definition products.

## 21.2 Do not add global application DB keys in P7

Rejected by default:

```rust
QueryKey::StaticApplication(...)
```

Reason:

- recursive algorithms can create large key cardinalities;
- evaluator-local memo solves repeated subproblems inside one root;
- P8 owns durable cross-query/cross-module optimization decisions.

Add a global application query only through explicit plan amendment backed by profiling.

## 21.3 Residual fingerprints

P7 still needs deterministic fingerprints for any root product that can publish `Symbolic(residual)`.

Add semantic residual hashing that:

- traverses normalized residual structure;
- hashes declaration/variant/member/proof identity semantically;
- is independent of `StaticResidualId` allocation order;
- does not treat raw source range as identity.

Within P7, raw `TypeId` may participate in session-local equality, but any root product fingerprint already expected to survive incremental reconstruction must use the same stable semantic hashing discipline used elsewhere in the DB.

If a correct stable fingerprint requires metadata-grade type identity not yet available, STOP and preserve the residual as session-local/non-exported rather than inventing unstable durability. P8 owns final serialization.

For the current P7 residual-store API, `content_fingerprint` is only a
store-lineage-local fingerprint: it may be independent of residual-node
allocation order within one TypeStore/StaticValueStore lineage, but it is not a
cross-lineage or durable semantic fingerprint while `TypeId` and
`StaticValueRef` participate directly. Do not publish or reuse it as an
incremental root-product fingerprint. Any stronger cross-lineage contract
requires a separate stable-identity design and plan amendment.

## 21.4 Invalidation tests

Add tests:

```text
edit referenced static definition
    -> root recomputes

edit referenced constant
    -> root recomputes

edit conformance fact used by residual/proof
    -> root recomputes

edit unrelated declaration
    -> deterministic checked root remains reusable

cold rebuild
    == incremental rebuild
```

---

# 22. Diagnostics

## 22.1 Cycle diagnostic

Use a structured `StaticCycle`.

Required message shape:

```text
static evaluation entered an exact recursive cycle

  Count<Pack<Int>>
  → Count<Pack<Int>>

cycle begins at the first application above
```

For mutual recursion:

```text
F<Int>
→ G<Int>
→ F<Int>
```

For changing arguments that do not repeat exactly, do not emit `static.cycle`.

## 22.2 Budget diagnostic

Include:

```text
budget kind
limit
used
root definition
application chain summary
```

Examples:

```text
static evaluation exceeded residual_nodes budget
limit: 10000
used: 10001
root: Expand<T>
```

## 22.3 Residual-too-large

Prefer `BudgetExceeded(BudgetKind::ResidualNodes)` as the semantic terminal and map it to diagnostic code:

```text
static.residual_too_large
```

if the existing diagnostic architecture distinguishes the user-facing code.

Do not invent an unbudgeted arbitrary residual-size error.

## 22.4 Closed-context symbolic result

A top-level `const` or another context requiring a concrete value must diagnose:

```text
static result remains symbolic after required specialization
```

with a summarized residual cause:

```text
unresolved parameter
proof goal
member projection
application
```

Do not dump full residual trees.

## 22.5 Dynamic/blocked distinction

Retain distinct user-facing notes for:

```text
Unknown generic input
Blocked semantic dependency
Dynamic runtime boundary
Cancelled
BudgetExceeded
InternalFailure
```

Do not flatten them all into "could not evaluate".

---

# 23. Detailed Task Plan

## P7.T0 — Takeover hardening and lifecycle rebaseline

### Files

```text
docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P7/**
phalcom-semantic/src/checker/proof_predicate.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/static_iteration.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/match_semantics.rs          # only if P4 FieldKey convergence lands here
phalcom-semantic/tests/semantic/integration/**
```

### Work

1. Correct checkpoint metadata:
   - `active_plan_record` must point to the P7 plan, not the completed P6 plan.
   - set P7 lifecycle state to `IN_PROGRESS / NOT_STARTED-or-PARTIAL / UNVERIFIED` as repository convention dictates when implementation begins.

2. Replace raw proof predicate spelling dispatch with canonical declaration resolution.

3. Replace `Conforms` right-hand `TypeId` proxy with canonical `TraitRef` / `GenericConformanceKey` goal.

4. Route `Equivalent` through canonical equivalence/proof authority.

5. Implement exact-item checker refinement for `FiniteExact`.

6. Make `StaticIterationMode` the evaluator execution authority.

7. Replace eager static-for item vectors with O(1) cursor state.

8. Converge open-tail fixed-field iteration with the normative C4 law.

9. Remove empty Data/Enum semantic-authority fallbacks when absence should be `Blocked`/`Unknown`.

10. Record Pack-pattern and Record-`FieldKey` acceptance gates; do not add private P7 pattern semantics.

### Gate P7.G0

Pass only when:

```text
proof predicate identity canonical
proof goals canonical
heterogeneous loop checking exact
for execution resolution-driven
for traversal O(1) cursor
open fixed fields behave normatively
no semantic authority is fabricated as empty
```

---

## P7.T1 — Residual store and structured terminal model

### Files

```text
phalcom-semantic/src/static_eval/residual.rs          # new
phalcom-semantic/src/static_eval/error.rs
phalcom-semantic/src/static_eval/mod.rs
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/static_key.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/tests/semantic/foundations/**        # residual-store unit tests
```

### Work

1. Add `StaticResidualStore` / IDs / refs.
2. Add minimum `Parameter` and placeholder application/proof/member forms.
3. Add structural interning.
4. Change outcome:
   - `Symbolic(StaticResidualRef)`
   - `Recursive(StaticCycle)`
5. Change internal `Stop`.
6. Update exhaustive matches.
7. Add deterministic semantic residual hash.
8. Add unit tests:
   - same node interns once;
   - different parameter owner/index does not collide;
   - cross-store refs rejected;
   - hash independent of allocation order for equivalent child structure.

### Gate P7.G1

No bare `StaticEvalOutcome::Symbolic` or payload-less `Recursive` remains in production code.

---

## P7.T2 — Exact application key and environment projection

### Files

```text
phalcom-semantic/src/static_eval/application.rs       # new
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/match_semantics.rs               # read/reuse only unless helper projection needed
phalcom-semantic/src/generic_evidence.rs              # reuse APIs; avoid semantic duplication
```

### Work

1. Define `StaticApplicationKey`.
2. Define `StaticArgumentIdentity`.
3. Define `StaticSpecializationEnvironmentKey`.
4. Implement canonical environment projection from:
   - root generic/kinded arguments;
   - active branch proof environment;
   - any static definition generic proof facts that change results.
5. Normalize key ordering.
6. Add tests proving:
   - two source sites with same target/args/env have equal key;
   - one site with different args has unequal key;
   - different proof environment can produce unequal key;
   - source range/formatting does not affect key.

### Gate P7.G2

No recursion/memo decision uses `StaticApplicationId` or declaration identity alone.

---

## P7.T3 — Exact cycle detection

### Files

```text
phalcom-semantic/src/static_eval/application.rs
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/static_eval/error.rs
phalcom-semantic/tests/semantic/integration/static_recursive_evaluation.rs  # new preferred
phalcom-semantic/tests/semantic/integration/mod.rs
```

### Work

1. Replace `active_definitions`.
2. Add active key vector/index.
3. Insert root definition key.
4. Detect exact repeat before frame entry.
5. Construct `StaticCycle`.
6. Preserve cleanup on all exits.
7. Keep explicit machine frames; no host recursion.

### Tests

```text
direct exact cycle
mutual exact cycle
changed argument recursion
changing nonterminating recursion -> budget
cycle inside selected match arm
cycle inside untaken arm is never observed
```

### Gate P7.G3

`Count`-style changing arguments are no longer rejected simply because the definition reappears.

---

## P7.T4 — Evaluator-local memoization

### Files

```text
phalcom-semantic/src/static_eval/application.rs
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/tests/semantic/integration/static_recursive_evaluation.rs
```

### Work

1. Add memo table.
2. Memo lookup after exact key formation.
3. Memo reduced deterministic result.
4. Memo canonical symbolic residual result.
5. Do not memo context terminals.
6. Add bounded memo growth charging via step budget initially; residual-specific budget arrives T10.
7. Add test instrumentation under `cfg(test)` or internal harness.

### Tests

```text
same exact subproblem reused
same definition / different argument not reused
same args / different proof environment not reused
Cancelled result not reused
BudgetExceeded result not reused
```

### Gate P7.G4

A repeated exact subproblem within one root evaluation is semantically evaluated once.

---

## P7.T5 — Parameter and Apply residualization

### Files

```text
phalcom-semantic/src/static_eval/residual.rs
phalcom-semantic/src/static_eval/application.rs
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/static_definition.rs             # only if stable parameter identity helper needed
phalcom-semantic/tests/semantic/integration/static_recursive_evaluation.rs
```

### Work

1. Residualize open parameter reads.
2. Permit `StaticEvalTerm::Residual`.
3. Collect residual arguments for checked applications.
4. Create normalized `Apply` residual.
5. Collapse fully exact Apply through normal evaluator.
6. Integrate memoization.
7. Validate result domain.

### Tests

```text
open generic application -> Apply residual
same open application -> same normalized residual
partially known arguments preserved
all-known arguments reduce
incorrect argument domain remains Invalid
```

### Gate P7.G5

P7 has a usable symbolic application model without general residual control flow.

---

## P7.T6 — Proof and member-projection residuals

### Files

```text
phalcom-semantic/src/static_eval/residual.rs
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/checker/proof_predicate.rs
phalcom-semantic/src/member_projection.rs
phalcom-semantic/src/structural_reflection.rs
phalcom-semantic/tests/semantic/integration/static_recursive_evaluation.rs
```

### Work

1. Convert valid unresolved proof goal to `Residual::Proof`.
2. Convert `Stuck(MemberProjection)` to `Residual::MemberProjection`.
3. Preserve Unknown/Blocked/Dynamic distinctions.
4. Normalize proof residual when specialization supplies enough facts.
5. Reinvoke canonical structural reflection/member projection after specialization.

#### Adopted Level 2 amendment — checked projection-site bridge (2026-10-12)

The consultation for [INC-002](LANG007.C4.P7-INC-002-member-projection-bridge.md)
identified that `CheckedStaticBody` has no canonical projection-site product and
that `Product::Shape` is a static-value projection (`ValueType(ProductShape)`),
not a kinded TypeForm. Apply this amendment to T6:

- Publish a compact canonical member-projection product/index on
  `CheckedStaticBody`, keyed by the checked expression/site. Derive its member
  identity and result domain from ordinary associated/type-member resolution
  and the canonical signature. Carry the canonical `MemberProjection`
  identity, arguments, evidence, and `StaticValueDomain`; do not resolve names
  or infer a domain in the evaluator.
- Route that checked product through
  `StructuralReflectionProjector::project_member` after specializing its
  subject and arguments. Preserve the projector's typed outcome and validate
  every reduced result against the checked site's result domain.
- Complete `ReducedValue` as an ordinary static value and `ReducedKinded` as a
  kinded result. Convert `Stuck` to a typed residual using the request's
  canonical identity and domain. Keep `Unknown`, `Blocked`, and `Dynamic` as
  distinct terminal evaluator outcomes; `Cancelled` and `BudgetExceeded` keep
  their existing terminal paths.
- Keep TypeData-to-residual conversion as a separate kinded case for canonical
  `AssociatedProjection` and `TypeMemberProjection` forms. It does not replace
  checked static-value projection sites such as `Product::Shape`.

Extend G6 coverage to prove that an open `Product::Shape<T>` keeps its canonical
member identity and `ValueType(ProductShape)` domain in the residual, an exact
specialization completes to an ordinary reflected `StaticValue`, and
Unknown/Blocked/Dynamic remain distinguishable. Existing kinded projection and
proof residual behavior remains covered.

### Tests

```text
Subtype<T, Number> open -> residual
specialize T -> true/false
Conforms<T, Product> open -> residual
specialize product/non-product
user generic trait conformance residual
member projection open -> residual
member projection exact -> ordinary reflected StaticValue
```

### Gate P7.G6

No valid generic proof/member projection is represented only by an anonymous `Symbolic` state.
Checked static-value projection sites retain their checker-owned canonical
identity/domain across reflection, residualization, and exact completion.

---

## P7.T7 — Residual specialization engine

#### Adopted Level 2 amendment — self-sufficient proof/projection residuals (2026-10-12)

The consultation for [INC-003](LANG007.C4.P7-INC-003-residual-specialization-environment.md)
found that `FinishStaticApplication` may return a Proof or MemberProjection
child residual directly. An enclosing Apply therefore cannot be relied on to
retain the exact arguments or branch facts that affected that child. The
existing specialization key also contains type bindings and constraints but
does not identify exact ordinary value arguments.

Apply this amendment to T7: extend the normalized environment with exact
formal-static-parameter bindings to kinded/value arguments, and capture that
environment in both Proof and MemberProjection payloads. Include it in
residual identity. During specialization, compose supplied substitutions
through captured bindings, normalize equivalent facts, fail closed on
conflicting concrete bindings, union canonical proof facts, check them through
the existing proof authority, and re-intern the normalized residual. Do not
depend on an enclosing Apply wrapper. Residual arguments and query-local proof
objects are not exact environment bindings.

Focused acceptance must cover direct child-residual escape and later
specialization for both families; environment-sensitive identity; reordered
and duplicate equivalent inputs; cold exact versus open-then-specialize;
idempotence; no-new-facts identity; and fail-closed conflicting bindings.
The [INC-003 packet](LANG007.C4.P7-INC-003-residual-specialization-environment.md)
records the evidence and advisory decision. T6's checked projection bridge is
unchanged.

#### Follow-up review amendment — activation shadowing and identity limits (2026-10-12)

The read-only review of the partial T7 implementation found that flattening
every active application frame with strict `merge` rejects ordinary recursive
calls such as `Grow<T> -> Grow<Next<T>>`: the same formal parameter is expected
to hold a different exact argument in the inner activation. Capture active
application bindings by frame, with the nearest active frame shadowing older
bindings for the same formal. Retain non-shadowed outer bindings and union
canonical branch facts. The specialization operation itself remains strict:
after composing supplied substitutions through captured bindings, conflicting
concrete values for one formal fail closed. Do not apply last-writer-wins to
two values supplied at the same specialization layer.

The current evaluator's relevant consumers read only
`BranchProofEnvironment.bindings` and canonical `equalities`/`Subtype` facts:
`active_generic_evidence` and `resolve_type_term_bounded` do not read
`local_bindings`, `local_equalities`, or `HasMember` facts. This is the bounded
reason those checker-local structures are not projected into residual
identity. If a T7 path begins consuming them, stop and amend the canonical
environment contract before hashing them; raw local/rigid identity remains
forbidden.

All environment construction must go through normalization; the current public
`new` constructor must not provide a bypass. Composition must substitute
through captured kinded types before conflict checking and re-evaluation.
Residual fingerprints remain store-lineage-local as clarified in §21.3; direct
`TypeId`/`StaticValueRef` hashing is not a cross-lineage identity guarantee.

Before T7 source work resumes, record this amendment in INC-003, remove the
flat-frame conflict, and add focused coverage for recursive same-formal
shadowing, strict supplied conflicts, canonical normalization, and the T7.G7
parity cases. Until then T7 remains paused and `UNVERIFIED`.

### Files

```text
phalcom-semantic/src/static_eval/specialize.rs         # new if justified
phalcom-semantic/src/static_eval/residual.rs
phalcom-semantic/src/static_eval/application.rs
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/tests/semantic/integration/static_recursive_evaluation.rs
```

### Work

1. Define specialization environment.
2. Substitute Parameter nodes.
3. Re-evaluate Apply nodes through memo/evaluator.
4. Capture self-sufficient normalized environments in Proof and MemberProjection
   residuals at creation; re-evaluate each through its existing semantic
   authority after composing supplied substitutions with captured bindings.
5. Re-intern the normalized result with its effective environment in identity.
6. Detect exact cycles during specialization using same key machinery.
7. Preserve budget/cancellation.

### Tests

For each supported residual family:

```text
cold exact == open + specialize
specialize twice is idempotent
no-new-facts specialization preserves residual identity
cancellation during deep specialization
```

For Proof and MemberProjection specifically, also verify direct child-residual
escape, identity differences for distinct effective environments, normalized
identity for reordered/duplicate equivalent bindings and proof facts, and
fail-closed behavior for conflicting concrete bindings.

### Gate P7.G7

Cold exact and specialized symbolic results are semantically equivalent.

---

## P7.T8 — Conditional residualization

### Files

```text
phalcom-semantic/src/static_eval/residual.rs
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/static_eval/specialize.rs
```

### Work

1. Add `Residual::If` only for semantically representable branch terms.
2. Preserve selected-branch-only semantics for reduced condition.
3. Isolate branch-local state while residualizing unknown condition.
4. Normalize constant condition.
5. Collapse equal branches where sound.
6. For imperative branch bodies that cannot be represented safely, residualize enclosing Apply instead of serializing evaluator frames.

### Tests

```text
known true/false unchanged from P6
unknown condition -> canonical residual
untaken invalid branch remains immune after specialization
branch-local mutation does not leak
```

### Gate P7.G8

No residual stores evaluator continuation state.

---

## P7.T9 — Match residualization and branch-proof specialization

### Files

```text
phalcom-semantic/src/static_eval/residual.rs
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/static_eval/pattern.rs
phalcom-semantic/src/static_eval/specialize.rs
phalcom-semantic/src/match_semantics.rs                # consume canonical products
```

### Work

1. Add semantic residual match carrier.
2. Reference/project canonical `MatchResolution`.
3. Preserve canonical pattern/variant identity.
4. When scrutinee specializes:
   - run P6 matcher;
   - select first matching arm;
   - push exact arm proof environment;
   - normalize selected branch.
5. Do not evaluate untaken arms.
6. Include relevant branch proof facts in nested application keys.

### Tests

```text
generic enum/sum residual match
specialize exact variant
GADT branch proof changes nested proof/application result
repeated variant names use VariantId identity
untaken arm immunity
exhaustiveness remains checker-owned
```

### Gate P7.G9

Residual match does not contain/re-resolve raw pattern syntax.

---

## P7.T10 — Structural construction residuals and recursive algorithms

### Files

```text
phalcom-semantic/src/static_eval/residual.rs
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/static_eval/specialize.rs
phalcom-semantic/src/static_value/store.rs
phalcom-semantic/tests/semantic/integration/static_recursive_evaluation.rs
```

### Work

Implement only residual constructors required by certification algorithms.

Priority:

1. Pack
2. Tuple
3. Record
4. Data
5. Variant
6. Sequence only if needed

Reuse existing static-value interning whenever all components reduce.

### Recursive acceptance algorithms

Certify available forms of:

```text
decreasing recursion
ContainsLabel
AllSubtype
Pack map/filter/fold
recursive UnorderedShape view walk
VariantsNamed
nested ProductShape transform
```

If ordinary Pack pattern syntax is available, add canonical:

```telos
type Count<*P>: Int =
    match P {
        Pack() => 0
        Pack(_, *rest) => 1 + Count<*rest>
    }
```

If it is not yet available, do not invent P7-only syntax. Certify recursive application with another ordinary semantic form and keep the exact Pack-pattern source test gated.

Always certify iterative equivalent:

```telos
type Count<*P>: Int {
    let mut count = 0
    for _ in P { count += 1 }
    count
}
```

### Gate P7.G10

Common structural recursive algorithms do not require runtime Iterator dispatch, host recursion, or quadratic Pack-tail copying.

---

## P7.T11 — Explicit residual growth budget

### Files

```text
phalcom-semantic/src/types/outcome.rs
phalcom-semantic/src/static_eval/residual.rs
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/tests/semantic/integration/static_recursive_evaluation.rs
```

### Work

1. Add `BudgetKind::ResidualNodes`.
2. Extend `QueryBudget`.
3. Charge one unit per newly interned residual node, not per memo hit.
4. Charge specialization traversal work.
5. Add deterministic `BudgetReport`.
6. Measure reduced static-value expansion; add `StaticValueNodes` only if required by a concrete failure/benchmark.

### Tests

```text
exponential symbolic expansion -> ResidualNodes budget
same interned residual reuse does not repeatedly consume node budget
changing nonterminating recursion -> Steps budget
cancellation still wins promptly when requested
```

### Gate P7.G11

There is no unbounded residual-growth path hidden behind a generous step limit.

---

## P7.T12 — Incremental dependency/fingerprint integration

### Files

```text
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/checker/static_body.rs
phalcom-semantic/src/static_constant.rs
phalcom-semantic/src/static_key.rs
phalcom-semantic/tests/semantic/integration/**
```

### Work

1. Hash structured symbolic outcomes by semantic residual content.
2. Hash cycle results deterministically where root products retain them.
3. Preserve existing checked-body dependencies.
4. Do not publish evaluator memo internals as DB products.
5. Add cold/incremental equivalence tests.
6. Ensure cancellation/budget results are not retained as stable semantic successes.
7. Ensure proof/conformance edits invalidate residual roots that depend on them.

### Gate P7.G12

Incremental history does not alter normalized residual/result identity.

---

## P7.T13 — Diagnostics, performance certification, and lifecycle closure

### Files

```text
phalcom-semantic/src/diagnostic.rs
phalcom-semantic/src/static_eval/error.rs
phalcom-semantic/src/static_eval/**
phalcom-semantic/tests/**
docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P7/LANG007.C4.P7-walkthrough.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P7/LANG007.C4.P7-handoff.md
```

### Work

1. Add/confirm diagnostics:
   - `static.recursive`
   - `static.cycle`
   - `static.budget_exceeded`
   - `static.residual_too_large`
   - `static.dynamic_dependency`
   - `static.blocked_dependency`
   - `static.specialization_failed`
   - `static.application_domain_mismatch`

2. Bound argument/residual formatting.

3. Benchmark required scenarios.

4. Run focused and broad repository gates according to workflow/build-mode doctrine.

5. Update checkpoint:
   - P7 `COMPLETE / IMPLEMENTED / FOCUSED_TESTED` only when all required gates pass.
   - advance to P8.
   - record residual limitations/gated syntax explicitly.

6. Produce P7 walkthrough and P8 handoff.

---

# 24. Testing Matrix

## 24.1 Application/cycle identity

| Test | Expected |
|---|---|
| direct `F<A> -> F<A>` | `Recursive(StaticCycle)` |
| mutual `F<A> -> G<A> -> F<A>` | exact cycle |
| `F<A> -> F<B>` | allowed |
| endlessly changing args | `BudgetExceeded(Steps)` |
| same call from two source sites | same application key |
| same args, different relevant proof env | different key |
| formatting/source span change | no key change |

## 24.2 Memoization

| Test | Expected |
|---|---|
| repeated exact subproblem | one logical evaluation |
| changed argument | miss |
| changed proof env | miss |
| same residual application | residual interning + memo reuse |
| Cancelled | not memoized |
| BudgetExceeded | not memoized |

## 24.3 Residual store

```text
parameter identity
apply identity
proof identity
member-projection identity
cross-store rejection
interning stability
allocation-order-independent semantic hash
```

## 24.4 Proofs

```text
Equivalent proven by exact identity
Equivalent proven by branch/generic equality fact
Equivalent refuted
Equivalent open -> residual

Subtype proven
Subtype refuted
Subtype open -> residual
Subtype blocked != false

Conforms exact source impl
Conforms compiler-derived structural proof
Conforms missing -> false
Conforms open -> residual
Conforms blocked/dynamic != false
```

## 24.5 Member projection

```text
concrete Product/Sum reflection -> reduced StaticValue
generic projection -> MemberProjection residual
specialize -> reduced
still-generic -> normalized residual
```

## 24.6 Match

```text
concrete selected branch
symbolic scrutinee residual
specialization selects arm
GADT branch proof visible
untaken arm not evaluated
repeated selector names preserve VariantId
```

## 24.7 Static `for`

```text
Pack empty/one/many
heterogeneous exact-item body checking
Tuple
Sequence
Record fixed fields
open unordered fixed fields
captured exact tail
break
continue
return
nested loops
index binding
O(1) cursor auxiliary state
```

## 24.8 Recursive structural algorithms

```text
decreasing recursion
iterative equivalent
nested recursive calls
memoized repeated subproblem
Pack rest view no O(n²) copying
recursive reflection/member projection
```

## 24.9 Resource safety

```text
cancellation mid-recursion
cancellation during specialization
Steps exhaustion
ResidualNodes exhaustion
large cycle trace formatting remains bounded
large Pack/shape traversal remains linear
```

## 24.10 Incremental

```text
relevant definition edit invalidates
relevant constant edit invalidates
relevant conformance edit invalidates
irrelevant edit reuses deterministic root
cold result == incremental result
cold residual hash == incremental residual hash
```

---

# 25. Performance Gates

Required benchmarks/probes:

## 25.1 Recursive Count

Run at:

```text
10
100
1000
```

entries.

Expected:

```text
O(n) applications
O(n) Pack-view steps
no O(n²) tail copying
host stack depth independent of guest recursion depth
```

## 25.2 Repeated-subproblem algorithm

Measure with memoization and with a test-only/no-memo baseline where feasible.

Expected:

```text
memoized work proportional to unique application keys
```

not total recursion-tree nodes.

## 25.3 Deep symbolic residual

Build deep generic residual structures.

Expected:

```text
linear normalization
bounded memory
ResidualNodes budget terminates adversarial growth
```

## 25.4 Specialization vs cold evaluation

Measure:

```text
specialize normalized residual
vs
cold exact evaluation
```

Specialization should not asymptotically regress compared with cold evaluation.

## 25.5 Cancellation latency

Long recursive/static structural computation must observe editor cancellation at cooperative safe points.

## 25.6 Memory

Track:

```text
active application stack
memo entries
residual nodes
StaticValue nodes
Pack backing/slices
```

No accidental retention of complete old evaluator frames after application completion.

---

# 26. P7 / P8 Boundary

## P7 owns

```text
session-local normalized residual IR
exact static application key
active exact cycle detection
evaluator-local memoization
recursive source static computation
residual specialization
symbolic proof/member/match support
bounded residual growth
deterministic session semantic fingerprints
```

## P8 owns

```text
durable residual encoding
cross-module residual import/export
metadata schema revision
stable serialized StaticValue/Type references
global application-result caches if justified
compiler materialization of exported static results
artifact compatibility
```

## Explicit P7 non-goals

Do not implement:

```text
VM residual object
VM static evaluator
new residual bytecode
runtime recursive static definition execution
metadata serialization of StaticResidualRef
cross-session raw TypeId/StaticValueRef persistence
global QueryKey::StaticApplication by default
new structural conformance solver
new match/pattern resolver
```

---

# 27. STOP / CONSULT Gates

Stop implementation and request architecture ruling if any occur.

1. Correct residualization appears to require storing `Work`, `SavedFrame`, or local evaluator program counters.
2. Residual semantics require raw AST nodes as semantic identity.
3. A second pattern resolver seems necessary.
4. A second subtype/equivalence/conformance engine seems necessary.
5. Guest recursion requires Rust recursion.
6. Exact cycle detection cannot distinguish same definition with different arguments.
7. Application correctness seems to require hashing the whole `CheckingContext`.
8. Environment identity requires raw query-local rigid IDs that cannot be canonicalized.
9. A durable fingerprint requires serializing raw `TypeId` or `StaticValueRef`.
10. P7 needs a new VM opcode/runtime residual object.
11. Compiler lowering needs to invoke `StaticEvaluator`.
12. `Cancelled` or `BudgetExceeded` is proposed as a residual node.
13. `Unknown`, `Blocked`, or `Dynamic` would be converted to false.
14. P7 needs a private Pack pattern because ordinary pattern semantics are missing.
15. P7 needs a private generalized Record-key pattern representation.
16. Residual specialization requires reparsing/rechecking source.
17. Memoization would use source call-site ID as semantic key.
18. Pack recursion copies each rest suffix.
19. Open unordered tails would be enumerated as invented fields.
20. Global DB application caching is proposed without profiling and lifecycle amendment.
21. Residual normalization can create cyclic residual graphs.
22. P7 semantic correctness depends on process-local allocation order.
23. Fixing P7 requires changing runtime object layout unrelated to static materialization.

---

# 28. File-Level Patch Map

## New files

Preferred:

```text
phalcom-semantic/src/static_eval/application.rs
phalcom-semantic/src/static_eval/residual.rs
phalcom-semantic/src/static_eval/specialize.rs
phalcom-semantic/tests/semantic/integration/static_recursive_evaluation.rs

docs/implementation/LANG007/LANG007.C4/LANG007.C4.P7/LANG007.C4.P7-walkthrough.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P7/LANG007.C4.P7-handoff.md
```

`specialize.rs` may remain folded into `residual.rs` initially if the module would otherwise contain only forwarding helpers.

## Primary modified files

```text
phalcom-semantic/src/static_eval/mod.rs
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/static_eval/error.rs

phalcom-semantic/src/checker/proof_predicate.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/static_iteration.rs
phalcom-semantic/src/checker/statement.rs

phalcom-semantic/src/types/outcome.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/static_key.rs

phalcom-semantic/tests/semantic/integration/mod.rs
docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md
```

## Conditional modified files

Only if the owning prerequisite work is landed as part of the takeover gate:

```text
phalcom-semantic/src/match_semantics.rs
phalcom-semantic/src/checker/pattern.rs
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
```

Do not broaden P7 into an unbounded pattern-language migration.

---

# 29. Verification Commands

Use repository workflow/BUILD MODE conventions. At minimum, focused implementation gates should include:

```bash
RUSTFLAGS='' cargo check -p phalcom-semantic

RUSTFLAGS='' cargo test -p phalcom-semantic \
  --test semantic integration::static_recursive_evaluation

RUSTFLAGS='' cargo test -p phalcom-semantic \
  --test semantic integration::static_definition_bodies

RUSTFLAGS='' cargo test -p phalcom-semantic \
  --test semantic adts::matching
```

Also run focused suites for:

```text
generic evidence/conformance
structural reflection
static values / Pack slicing
incremental static definitions/constants
```

Before closure, run repository-prescribed format/diff checks and the broader semantic/core tests permitted by current build-mode doctrine.

At every phase:

```bash
git diff --check
git status --short
```

must remain clean/understood.

---

# 30. Acceptance Scenarios

## A. Exact changing-argument recursion

A recursive static definition must be able to call itself with a smaller/different argument until base case.

Expected:

```text
no definition-only recursion failure
no Rust stack growth
exact cycle only on repeated application key
```

## B. Exact cycle

```text
F<A> -> F<A>
```

Expected:

```text
StaticEvalOutcome::Recursive(StaticCycle)
```

with deterministic compact trace.

## C. Symbolic generic proof

```telos
type IsNumeric<T: Type>: Bool =
    Subtype<T, Number>
```

Open `T`:

```text
Symbolic(Proof(...))
```

Exact `T`:

```text
Reduced(true/false)
```

## D. Residual specialization equivalence

```text
evaluate open
    -> R

specialize R with A
    -> X

cold evaluate exact A
    -> Y

X semantically equals Y
```

## E. Recursive Pack/structural computation

When ordinary source syntax supports the necessary structural pattern:

```text
Count<Pack<A,B,C>> -> 3
```

and the iterative `for` equivalent also produces `3`.

No O(n²) Pack copying.

## F. Memoized repeated subproblem

A source algorithm with repeated identical subcalls must demonstrate one logical evaluation per exact application key.

## G. Resource termination

Changing non-repeating recursion:

```text
F<0> -> F<1> -> F<2> -> ...
```

Expected:

```text
BudgetExceeded(Steps)
```

not `Recursive`.

Exponential residual growth:

```text
BudgetExceeded(ResidualNodes)
```

## H. Cancellation

Editor/query cancellation interrupts recursion/specialization promptly without publishing a valid residual/result for the cancelled request.

---

# 31. Exit Criteria

P7 is complete only when all of the following are true.

1. Same-definition changing-argument recursion is supported through the explicit evaluator machine.
2. Exact application-key repetition, not declaration repetition, defines a static cycle.
3. `StaticEvalOutcome::Recursive` carries structured cycle evidence.
4. `StaticEvalOutcome::Symbolic` carries a canonical residual reference.
5. Static application keys contain canonical target, arguments, and relevant specialization/proof environment.
6. Repeated exact subproblems memoize evaluator-locally.
7. Cancelled/BudgetExceeded states are never memoized as valid semantic results.
8. Residuals are normalized semantic IR, not AST or suspended evaluator state.
9. Open proof predicates residualize canonically and specialize through the existing proof engines.
10. `Stuck(MemberProjection)` becomes a first-class residual operation.
11. Supported residuals specialize without reparsing/rechecking source.
12. Cold exact evaluation and symbolic specialization are semantically equivalent for certified cases.
13. Match residual specialization reuses canonical P6 match/pattern products and branch proof environments.
14. Static structural recursion does not use runtime Iterator dispatch.
15. Pack/rest processing remains linear with O(1) view/cursor behavior.
16. Residual growth has an explicit named budget.
17. Cancellation remains cooperative throughout recursive evaluation and specialization.
18. Incremental and cold semantic results/fingerprints converge.
19. No P7 feature introduces VM/runtime residual execution.
20. P7 lifecycle records include focused verification and explicit deferred/gated syntax.
21. The checkpoint advances cleanly to P8 with a precise handoff describing session-local residual/value identities that P8 must make durable.

---

# 32. Implementation Order Summary

Execute in this order:

```text
T0   takeover hardening
 ↓
T1   residual store + structured outcomes
 ↓
T2   exact application key + proof environment identity
 ↓
T3   exact cycle detection
 ↓
T4   evaluator-local memoization
 ↓
T5   Parameter + Apply residual vertical slice
 ↓
T6   Proof + MemberProjection residuals
 ↓
T7   specialization engine
 ↓
T8   conditional residuals where semantically safe
 ↓
T9   match residuals + GADT proof specialization
 ↓
T10  structural/product residuals + recursive algorithms
 ↓
T11  residual growth budget
 ↓
T12  incremental/fingerprint integration
 ↓
T13  diagnostics + performance + lifecycle closure
```

This ordering deliberately proves the application identity/cycle/memo architecture **before** broadening the residual language.

---

# 33. Final Architecture Target

```text
                     CheckedStaticBody
                           │
       ┌───────────────────┼────────────────────┐
       │                   │                    │
 MatchResolution     StaticIteration      ProofPredicate
 PatternResolution      Resolution           Goal
       │                   │                    │
       └───────────────────┼────────────────────┘
                           ▼
                    StaticEvaluator
                           │
                 exact application key
                           │
              ┌────────────┴────────────┐
              │                         │
        active-key index           local memo
              │                         │
        exact cycles               reuse
              └────────────┬────────────┘
                           ▼
              ┌────────────────────────┐
              │                        │
       StaticValueStore        StaticResidualStore
              │                        │
          reduced                 normalized
          values                  residuals
              │                        │
              └────────────┬───────────┘
                           ▼
                    specialization
                           │
               same semantic authorities
                           │
          ┌────────────────┼────────────────┐
          │                │                │
       Reduced          Symbolic        terminal
                                       Invalid /
                                       Recursive /
                                       Budget /
                                       Cancelled
```

P7 should end with a **session-local, deterministic, bounded compile-time programming engine**. P8 then makes its public/exported products durable across module and artifact boundaries.
