# Phalcom Advanced Kinds, Constraints, Effects, and Proofs

**Date:** 2026-08-22
**Status:** Ratified semantic architecture; public surfaces and backends remain behind named gates
**Authority:** advanced static semantic domains, solver/result boundaries, and proof-evidence architecture
**Depends on:** [01 — Implementation Architecture](01-implementation-architecture.md), [02 — Runtime Reification and Metadata](02-runtime-reification-and-metadata.md), [03 — Reflection API and Capabilities](03-reflection-api-and-capabilities.md), and [04 — User-Facing Type Syntax and Lowering](04-user-facing-type-syntax-and-lowering.md)
**Owners:** `phalcom-semantic`, native semantic surfaces, compiler contract bridge, metadata exporter, future proof component
**Scope:** kind schemes, type/row variables, variance, bounds, constraint solving, `Self`, effects, exits, totality, contracts, verification conditions, proof results, persistent artifacts, trust, incrementality, budgets, and diagnostics
**Non-goals:** dependent types, `Type :: Type`, universe polymorphism, proof terms, runtime dispatch changes, public backend selection, or implementation in this document

## 1. Purpose and semantic boundary

This document extends the implemented two-axis type/kind kernel without collapsing static knowledge into runtime objects. It defines separate domains for:

- proper types and type constructors;
- stable generic parameters and solver-local variables;
- record rows and future effect rows;
- subtype/assignability/equivalence/consistency constraints;
- effect capability and exit behavior;
- normal return type and termination knowledge;
- executable contracts and static proof obligations;
- proof results, trust, artifacts, cancellation, and budget outcomes.

It does not introduce `Type :: Type`, dependent types, a universe hierarchy, arbitrary kind-level evaluation, proof terms in ordinary APIs, or a new runtime dispatch mechanism.

Evidence labels use the [series contract](README.md#1-reading-contract-and-evidence-labels). In particular, **Observed current implementation** is not a promise that the target architecture exists, and **Proposed design needing ratification** is an implementation stop.

## 2. Current implementation inventory

### 2.1 Kinds, parameters, and application

**Observed current implementation.** [`KindData`](../../../../phalcom-semantic/src/types/kind.rs) contains atomic `Type` and arrow kinds. [`TypeParameterData`](../../../../phalcom-semantic/src/types/parameter.rs) records declaration/callable owner, index, name, and kind. [`GenericSignature`](../../../../phalcom-semantic/src/types/parameter.rs) and declaration tables currently derive generic signatures primarily from native universe specifications because source declarations have no generic binder syntax.

[`TypeStore`](../../../../phalcom-semantic/src/types/store.rs) contains `Never`, `Unit`, class-object, nominal, applied, union, tuple, closed record, callable, parameter, and inference forms. Kind-checked application supports residual arrow kinds for partial application.

Missing:

- stable kind parameters and solver-local kind variables;
- declaration-site variance and bounds;
- record-row kind/tail and row variables;
- a constraint IR with reasoned terminal outcomes;
- publishability checks preventing solver variables from escaping.

### 2.2 Relations and inference

**Observed current implementation.** Existing relation code in [`relation.rs`](../../../../phalcom-semantic/src/types/relation.rs) handles important nominal, callable, tuple, record, union, parameter, class-object, and dynamic cases, but several APIs still expose booleans or a coarse uncertain result. Inference variables are represented in `TypeData`, while no complete solver lifecycle proves they are absent from interfaces and metadata.

**Ratified/normative design.** Structural equality, semantic equivalence, subtyping, assignability, and consistency remain different operations. All recursive operations use query-local cycle state and budgets. A permissive compatibility predicate cannot implement multiple relations.

### 2.3 Native effects and returns

**Observed current implementation.** [`phalcom-native-meta::primitive`](../../../../phalcom-native-meta/src/primitive.rs) declares:

- `EffectSpec::{Unknown, Pure, Known}`;
- native effects for mutation, I/O, scheduling, reflection, nondeterminism, and blocking;
- `RaisesSpec::{Unknown, Known}`;
- `ReturnFlowSpec::{Value, Receiver, Argument, Never, Unknown}`.

These are useful native-surface facts. They are not yet one compiler-owned effect lattice, do not model solver variables or provenance, and do not prove totality.

### 2.4 Runtime contracts

**Observed current implementation.** [`phalcom-core/src/compiler/attributes.rs`](../../../../phalcom-core/src/compiler/attributes.rs) weaves executable `requires`, `ensures`, and invariant guards according to compile mode. Debug retains and executes all guards; release retains required preconditions while stripping selected postconditions/invariants; unchecked can strip guards and metadata. Purity checking is a conservative syntactic predicate, not an effect proof.

[`MethodObject.contracts`](../../../../phalcom-core/src/method/object.rs#L198) stores reflectable predicate closures when retained. [`build_contracts_metadata`](../../../../phalcom-core/src/compiler/lib/class_decl.rs#L1250) compiles standalone predicate closures. These closures are runtime checks/metadata, not verification certificates.

### 2.5 Prover status

**Observed current implementation.** No accepted static proof engine, verification-condition IR, kernel checker, trusted backend protocol, or persistent proof cache exists in the live implementation. Current contracts and runtime invariants provide executable enforcement and specification input only.

Any historical statement that current contracts are proofs is rejected.

## 3. Domain and identity model

### 3.1 Persistent identities

**Ratified/normative design.** Published semantic products may contain:

```rust
TypeId
KindId
TypeParameterId
KindParameterId
RecordRowId
EffectSetId
EffectParameterId
ConstraintId
ContractId
VerificationConditionId
ProofArtifactId
```

Each ID is scoped to an owning store or stable semantic owner as defined in [01](01-implementation-architecture.md). Serialized metadata uses indexed DAG node IDs from [02](02-runtime-reification-and-metadata.md), never raw store-local IDs.

### 3.2 Solver-local identities

Solver-local identities never escape a solver result:

```rust
InferVarId
KindVarId
KindSkolemId
RecordRowVarId
EffectVarId
ProofObligationVarId
```

A solution must be zonked/finalized before publication. Unsolved variables yield reasoned `Unknown`, invalid, or incomplete status according to the query contract. They cannot be replaced silently with `Dynamic`, `Any`, an empty effect set, a closed row, or a proven obligation.

### 3.3 Publishability invariant

Every interface, snapshot, metadata DAG, reflection descriptor, and proof artifact passes:

```rust
fn validate_publishable(product: &SemanticProduct) -> Result<(), PublishabilityError>;
```

At minimum it rejects:

- solver-local variables;
- store IDs owned by another generation;
- unresolved or invalid annotation nodes presented as types;
- noncanonical row field order;
- unnormalized constraint substitutions;
- proof results without trust/provenance;
- stale dependency or semantic-model fingerprints.

## 4. Kind system

### 4.1 Kinds

Target canonical kind data:

```rust
enum KindData {
    Type,
    RecordRow,
    Arrow { parameters: Box<[KindId]>, result: KindId },
    Parameter(KindParameterId),
}
```

`Type` is atomic. `TypeForm` is not a kind constructor or a superclass. `RecordRow` is distinct from `Type`; a row cannot annotate a value. Variant and effect row kinds are reserved for their own semantic decisions and do not reuse `RecordRow` merely because algorithms may look alike.

Source kind arrows associate right, then canonicalization flattens the constructor domain into `parameters`:

```text
Type -> Type -> Type
== Type -> (Type -> Type) in source grouping
== Arrow { parameters: [Type, Type], result: Type } canonically
```

No product kinds, dependent function kinds, kind-level lambdas, or runtime-computed kinds are accepted in this phase.

### 4.2 Prenex kind schemes

`DEC-KIND-POLY` is normative:

```rust
struct KindScheme {
    parameters: Box<[KindParameterId]>,
    body: KindId,
}
```

Generalization occurs only at a declaration/interface boundary, after solving its monomorphic kind constraints. Generalized parameters are ordered deterministically by first stable occurrence, then assigned `KindParameterId { owner, index }`. Instantiation creates fresh solver-local `KindVarId`s.

No higher-rank kind quantifier occurs inside a type argument or callable position. No unsolved kind variable escapes. Reflection may describe a closed kind scheme but cannot instantiate or solve it without an explicit capability and `TypingContext` from [03](03-reflection-api-and-capabilities.md).

Generalization eligibility:

- class, method, and alias generic surfaces may generalize at their completed declaration/interface boundary;
- a local value annotation cannot generalize a constructor merely because inference found an arrow kind;
- a value-typing position must finish at `Type` and cannot bind a kind scheme;
- native declarations may publish a kind scheme only through validated authoritative metadata;
- partial/failed modules publish no generalized interface for the failed declaration.

Checking a previously generalized scheme instantiates parameters as fresh rigid/skolem terms when testing subsumption, preventing an implementation from solving the declaration's universal parameter to fit one use. These rigid identities are solver-local and distinct from stable `KindParameterId` and flexible `KindVarId`.

### 4.3 Kind constraint solving

Solver terms:

```rust
enum KindTerm {
    Canonical(KindId),
    Var(KindVarId),
    Rigid(KindSkolemId),
    Arrow { parameters: Vec<KindTerm>, result: Box<KindTerm> },
}
```

Constraints are equality constraints over kind terms. Worklist unification performs:

1. representative lookup;
2. trivial equality elimination;
3. variable binding with occurs check;
4. arrow arity check and strict positional decomposition;
5. atomic mismatch diagnostic;
6. budget/cancellation check.

Result:

```rust
enum KindSolveResult {
    Solved(KindSubstitution),
    Invalid(KindMismatch),
    Unknown(KindUnknownReason),
    Cancelled,
    BudgetExceeded(KindBudgetReport),
    InternalError(IncidentId),
}
```

`Unknown` is reserved for incomplete external information or a cycle policy that lacks sufficient facts. Ordinary unification mismatch is `Invalid`, not unknown.

Parser recovery remains syntax-owned under [04 §8](04-user-facing-type-syntax-and-lowering.md#8-syntax-ast-and-recovery-contract). Kind solving accepts an explicit invalid term only to preserve diagnostic causality; it never binds another variable to that invalid term or publishes it.

### 4.4 Rejected kind designs

**Ratified/normative design.** Reject:

- `Type :: Type`;
- universes `Type0`, `Type1`, ...;
- kind inference that evaluates user code;
- dependent kinds indexed by runtime values;
- stable identities derived only from binder text;
- a single `TypeVarId` used for kinds, types, records, and effects;
- publishing a partial application where proper kind `Type` is required.

## 5. Type parameters, variance, and bounds

### 5.1 Parameter data

Target parameter record:

```rust
enum Variance {
    Covariant,
    Contravariant,
    Invariant,
}

struct TypeParameterData {
    owner: TypeParameterOwner,
    index: u32,
    name: Symbol,
    kind: KindId,
    variance: Variance,
    bounds: Box<[TypeBound]>,
    default: Option<TypeId>,
    source: Option<SourceOrigin>,
}
```

Defaults are represented now to avoid an incompatible metadata change but remain a **Proposed design needing ratification** for source syntax and inference behavior.

### 5.2 Variance algebra

Position composition:

| Outer | Inner | Result |
|---|---|---|
| positive | covariant | positive |
| positive | contravariant | negative |
| negative | covariant | negative |
| negative | contravariant | positive |
| any | invariant | invariant |
| invariant | any | invariant |

Callable parameters are negative; callable results positive. Mutable read/write fields are invariant unless the language surface proves read-only or write-only capability. A constructor argument's position follows the declared variance of that constructor parameter. Unknown variance metadata blocks validation and interface publication.

Declaration validation emits an occurrence path:

```text
declared +T
  -> field `consumer`
  -> callable parameter 0
  -> List element (+)
  -> T occurs negative
```

No bivariant escape hatch is accepted. Dynamic boundaries may defer an assignment check but do not retroactively validate an unsound generic declaration.

A higher-kinded parameter is invariant as a constructor unless its bound supplies a validated variance scheme for each constructor argument. Applying `F<T>` where `F :: Type -> Type` does not let the checker assume covariance from arrow kind alone.

Inherited member specialization substitutes the owning class arguments before override comparison. Instance-side and class-side surfaces use their distinct declaration owners and `Self` roles. Substitution preserves member identity/selector; it changes only the semantic signature being checked.

### 5.3 Bounds

Initial canonical bounds:

```rust
enum TypeBound {
    Upper(TypeId),
    FiniteSet(Box<[TypeId]>),
}
```

Upper bounds use semantic subtyping. Finite sets use semantic equivalence of normalized alternatives. Bounds are checked at explicit application and during inference finalization.

F-bounds are legal only when:

- referenced parameters belong to the same or an enclosing generic owner;
- kind checks succeed before relation checks;
- alias expansion and relation recursion are guarded;
- failure reports a finite evidence path;
- budget exhaustion is distinct from mismatch.

Lower bounds, equality bounds, protocol-only bounds, associated types, and negative bounds remain gated.

Constraint domains remain distinct even while gated:

| Constraint | Initial status | Relation/solver meaning |
|---|---|---|
| upper bound `T <: U` | Active target | subtype obligation |
| lower bound `L <: T` | Gated | directed lower obligation; never rewritten as upper |
| finite exact set `T in (...)` | Active target | equivalence to one listed member; not union subtyping |
| protocol constraint | Gated | conformance/coherence evidence, not nominal subtype by default |
| equality constraint | Gated | semantic equivalence/unification under binder policy |
| kind constraint | Active internal | kind unification before type relations |
| record lacks constraint | Active internal with rows | proves a label absent before row extension |
| effect constraint | Active internal with effects | effect subset/equality over effect domain |

An implementation cannot encode a gated constraint using a convenient active variant. It must stop for ratification.

### 5.4 Substitution

Substitution is simultaneous, owner-aware, capture-avoiding, and kind-checked. A substitution key is `TypeParameterId`, never parameter name. Substitution traverses canonical type forms, record tails, callable forms, `Self`, constraints, effects when effect parameters exist, and contract signatures. It does not rewrite runtime values or selector identities.

## 6. Record rows

### 6.1 Canonical representation

`DEC-RECORD-ROWS` is normative:

```rust
struct RecordType {
    fields: Box<[(FieldName, TypeId)]>,
    tail: RecordTail,
}

enum RecordTail {
    Closed,
    Parameter(TypeParameterId),
}

struct RecordRow {
    fields: Box<[(FieldName, TypeId)]>,
    tail: RecordRowTail,
}

enum RecordRowTail {
    Closed,
    Parameter(TypeParameterId),
}

enum RecordRowTermTail {
    Closed,
    Parameter(TypeParameterId),
    Infer(RecordRowVarId),
}
```

`RecordType` has kind `Type`; `RecordRow` has kind `RecordRow`. Known fields are sorted and unique. Tail is stored separately. Solver-local open structure uses `RecordRowVarId`, not a canonical fake parameter.

`RecordRowTermTail::Infer` exists only in solver terms. Finalization replaces it with a solved closed/parameter tail or returns an unsolved/unknown result. Canonical `RecordRow`, interfaces, metadata, and reflection never contain it.

### 6.2 Row relations

For read-only structural record conformance, a record with at least required fields may satisfy an open/width-compatible requirement when every required field type relates appropriately. Mutation capability can force invariance or an exact field set. The relation context must name the access capability; structural record subtyping cannot guess mutability from syntax.

Row solving handles equations such as:

```text
#{ name: String, age: Int } = #{ name: String, | R }
R = #{ age: Int }
```

It performs field subtraction and unification with occurs checks. Duplicate labels, impossible closed-row extras, kind mismatch, or conflicting shared-field types are invalid. Missing external facts, budget exhaustion, and cancellation use distinct outcomes.

Row extension `extend(R, label: T)` first proves `Lacks(R, label)`, then inserts the field into canonical sorted order. `Lacks` constraints propagate through known fields and row substitutions; an unresolved open tail leaves a pending solver constraint. Extending without a proved lack would permit duplicate fields after substitution and is rejected.

Normalization memoizes row-variable representatives, limits field/substitution depth, and rejects direct or indirect row occurs cycles. Width/depth subtyping and assignability call row solving under an explicit read/write capability. An open expected row does not authorize extra writable fields unless capability rules prove safety.

### 6.3 Separation from other rows

Record rows, variant rows, and effect rows may share:

- deterministic label maps;
- tail-variable union-find;
- occurs checks;
- worklist scheduling;
- budget accounting.

They do not share semantic ID types, label namespaces, relation rules, or reflection classes. A generic `RowSolver<L, V, Policy>` utility is permitted only after all domain wrappers prevent cross-domain mixing.

## 7. Constraint IR and solver

### 7.1 Constraint language

Target internal IR:

```rust
enum TypeConstraint {
    Equivalent(TypeTerm, TypeTerm),
    Subtype(TypeTerm, TypeTerm),
    Assignable { actual: TypeTerm, expected: TypeTerm },
    Consistent(TypeTerm, TypeTerm),
    HasKind(TypeTerm, KindTerm),
    MemberOfFiniteSet(TypeTerm, Box<[TypeId]>),
    RecordRowEqual(RecordRowTerm, RecordRowTerm),
    RecordRowLacks(RecordRowTerm, FieldName),
    EffectSubset(EffectTerm, EffectTerm),
    RequiresTotality(CallableId),
}
```

Each constraint carries origin, owner query, cause chain, and diagnostic policy. Relation direction is explicit. `Consistent` is used only for gradual/dynamic boundaries and cannot prove `Subtype`.

### 7.2 Worklist and SCCs

**Pyrefly architectural transfer.** Use explicit query keys, answer states, SCC-local placeholders, bounded fixed points, deterministic publication, and dependency fingerprints. Adapt semantics to Phalcom relations and open-world dispatch.

Algorithm:

```text
seed canonical constraints
    -> prioritize kind/equality constraints
    -> propagate substitutions
    -> decompose structured relations
    -> enqueue dependencies
    -> discover recursive query SCCs
    -> iterate monotone summaries under budget
    -> finalize/zonk
    -> validate publishability
    -> publish one immutable answer set
```

Local relation memoization includes relation kind, ordered operands, context policy, world/type-system revision, and budget class. Temporary coinductive assumptions never enter a global cache.

### 7.3 Outcomes

```rust
enum ConstraintSolveResult {
    Solved(Solution),
    Unsatisfied(UnsatisfiedConstraint),
    Unknown(ConstraintUnknownReason),
    Cancelled,
    BudgetExceeded(SolverBudgetReport),
    InternalError(IncidentId),
}
```

`Unsatisfied` means evidence establishes failure. `Unknown` means required knowledge is absent or an explicitly open-world boundary prevents a decision. Budget and cancellation remain terminal states outside unknown so callers can choose retry policy without confusing epistemology.

## 8. `Self` semantic model

`Self` lowering creates an owner-relative term before substitution:

```rust
struct SelfTypeTerm {
    owner: DeclarationId,
    side: DispatchSide,
    role: SelfRole,
}

enum SelfRole {
    ReceiverValue,
    ReturnValue,
    BoundReference,
}
```

An instance-side method resolves receiver/return `Self` against the selected nominal application. A class-side signature resolves through class-object typing, not by pretending that a class object and its denoted nominal type are identical. Overrides substitute `Self` at the use site and then check variance/relations.

`Self` does not mean “dynamic runtime class of this value” in static metadata. Explicit runtime description uses `value ⇝ T`/reflection APIs under [03](03-reflection-api-and-capabilities.md).

## 9. Effect and exit model

### 9.1 Effects are capabilities, exits are control outcomes

**Ratified/normative design.** Keep four axes separate:

```text
normal return type
effect capability set
exit summary
termination knowledge/requirement
```

`Never` is a normal-return type fact: no normal returned value. It does not distinguish divergence, unconditional raise, process exit, suspension forever, or unreachable code.

### 9.2 Effect atoms and sets

Initial effect atoms preserve native vocabulary:

```rust
enum EffectAtom {
    Mutation(MutationRegion),
    Io,
    Scheduling,
    Reflection(ReflectionCapability),
    Nondeterminism,
    Blocking,
    ForeignCall(ForeignBoundaryId),
}
```

`MutationRegion` begins as a bounded enum for unknown, receiver, global, or identified argument regions. `ForeignBoundaryId` names a stable native/FFI surface within the semantic generation. Neither is a runtime pointer.

`EffectSet` is deterministic, canonical, and supports subset/join. Mutation region begins with coarse `Unknown`/`Receiver`/`Global`/argument identities; field-sensitive regions are deferred until needed. Reflection capability aligns with [03](03-reflection-api-and-capabilities.md).

Knowledge is separate from the set:

```rust
enum EffectKnowledge {
    Known(EffectSetId),
    Unknown(EffectUnknownReason),
    Invalid(EffectDiagnosticSet),
}
```

An empty known set means pure under the defined effect model. Unknown does not mean pure. Budget/cancel/internal remain outer query statuses.

Declaration checking uses a separate requirement:

```rust
enum EffectRequirement {
    Inferred,
    Pure,
    AtMost(EffectSetId),
}
```

`Pure` is `AtMost(empty)` with a clearer source/diagnostic policy. A requirement is not inferred knowledge and never overwrites observed body/callee facts.

### 9.3 Effect variables and rows

Effect polymorphism is internally accommodated:

```rust
struct EffectRow {
    atoms: Box<[EffectAtom]>,
    tail: EffectTail,
}

enum EffectTail {
    Closed,
    Parameter(EffectParameterId),
}
```

Public effect-parameter syntax is **Proposed design needing ratification**. First implementation may use closed sets and solver-local `EffectVarId` for inference. No effect variable may escape a summary.

### 9.4 Exit summary

```rust
struct ExitSummary {
    may_return: bool,
    may_raise: RaiseKnowledge,
    may_diverge: DivergenceKnowledge,
    may_nonlocal_return: bool,
    may_exit_process: bool,
    may_suspend: bool,
}
```

Raise knowledge preserves known exception types versus unknown raise behavior. Return-flow facts such as “returns receiver” or “returns argument 0” remain value-flow summaries and can refine return knowledge; they do not belong in the effect set.

`Throws<T>`, non-local return, process exit, and divergence are control exits, not `EffectAtom`s. FFI is represented by `ForeignCall(boundary)` plus whatever effect/exit knowledge authoritative metadata provides. Surface rendering may combine effects and exits for readability but semantic APIs keep them separate.

### 9.5 Inference and checking

Effect inference is bottom-up over callable bodies and summaries:

1. local primitive operations contribute atoms;
2. calls join callee effects and exit facts;
3. dynamic/DNU/FFI/reflection boundaries contribute explicit unknown/open effects under policy;
4. recursive callable SCCs iterate effect/exit summaries monotonically;
5. an explicit declaration is checked by subset, not used to erase inferred effects;
6. published summary records knowledge, provenance, dependencies, and budget status.

Native `EffectSpec::Pure` is an authoritative declared surface only after native metadata validation. Runtime behavior contradicting it is a native-contract bug and may become a debug assertion/security incident; semantic analysis does not silently widen a declared pure native operation based on one observed run.

Containment law is set inclusion for known closed sets. Override/conformance requires implementation effects to be a subset of the declared/base allowance and exit behavior to satisfy its separate contract. Unknown/open effects cannot prove an override safe. Effect masking/handling may subtract an atom or exit only when a ratified handler operation proves it handles that domain; `try`/catch may refine raises but cannot erase mutation or I/O. Public handlers and effect-polymorphic syntax remain gated.

## 10. Totality

`DEC-TOTALITY` is normative:

```rust
enum TerminationRequirement {
    Partial,
    Total,
}

enum TerminationKnowledge {
    ProvenTerminates(TerminationEvidenceId),
    ProvenDiverges(DivergenceEvidenceId),
    Unknown(TerminationUnknownReason),
}
```

Ordinary declarations default to `Partial`. Partial correctness means: if execution returns normally, declared postconditions and return type hold. `Total` requires `ProvenTerminates`; unknown is a diagnostic at a total declaration, not an assumption.

Termination analysis ladder:

1. intraprocedural acyclic control flow;
2. structurally decreasing recursion over ratified measures;
3. SCC call-graph size-change analysis;
4. explicit trusted assumption under capability/policy;
5. unknown with reason.

Public totality syntax and accepted measure annotations remain **Proposed design needing ratification**. Architecture and result algebra are fixed so contracts/proofs do not misuse `Never` meanwhile.

## 11. Contract model

### 11.1 One source contract, two consumers

Source contracts lower once into canonical semantic contract IR:

```rust
struct CallableContract {
    callable: CallableId,
    requires: Box<[ContractPredicateId]>,
    ensures: Box<[ContractPredicateId]>,
    invariants: Box<[ContractPredicateId]>,
    termination: TerminationRequirement,
    modifies: EffectKnowledge,
}

struct ContractPredicate {
    expression: TypedExprId,
    phase: ContractPhase,
    free_bindings: Box<[BindingId]>,
    old_captures: Box<[OldCapture]>,
    effects: EffectKnowledge,
    source: SourceOrigin,
}
```

Runtime weaving and static verification consume this IR. Runtime compiler may still lower predicates to closures/guards. Prover lowers them to logic expressions under a separate admissibility check.

Source attributes map as follows:

- `@requires` creates entry assumptions and caller obligations;
- `@ensures` creates normal-return obligations with result and `old` substitution;
- `@invariant` creates entry/exit and mutation-boundary obligations under class invariant policy;
- future `@total` sets `TerminationRequirement::Total` and adds termination obligations.

`@total` spelling is still a public syntax gate; the semantic field and VC behavior are fixed.

### 11.2 Admissibility

An executable predicate is not automatically a logical predicate. Static proof accepts only a defined pure, deterministic, terminating subset with modeled operations. Unsupported reflection, FFI, scheduling, mutation, or dynamic dispatch yields `Unknown(UnsupportedOperation)` or a contract-admissibility diagnostic according to declaration policy.

`old(expr)` captures a pre-state value in runtime weaving and a pre-state symbolic term in proof IR. Both consumers share binding and source identity but need not share execution representation.

### 11.3 Compile modes and proof truth

Stripping a runtime guard does not manufacture proof. A release build may omit an `ensures` check only according to compilation policy; proof metadata independently records whether the obligation is proven, assumed, disproven, or unknown. An unchecked build cannot label stripped contracts `Proven`.

Required truth table:

| Evidence/event | Proof result |
|---|---|
| runtime contract exists; prover not run | `Unknown(NotAttempted)` |
| runtime checks passed in tests | `Unknown(NoStaticEvidence)` |
| prover timeout | `Unknown(Timeout)` at language boundary |
| validated counterexample | `Disproven(counterexample)` |
| trusted solver reports unsatisfiable without checked certificate | `Proven(TrustedBackend)` |
| certificate accepted by local kernel | `Proven(KernelChecked)` |
| stripped/disabled runtime guard | no change to proof result |

## 12. Proof IR and verification conditions

### 12.1 Typed proof IR

Proof lowering consumes a completed semantic snapshot and produces a side-effect-explicit IR:

```rust
struct ProofProcedure {
    callable: CallableId,
    blocks: Box<[ProofBlock]>,
    entry: ProofBlockId,
    parameters: Box<[ProofLocal]>,
    return_type: TypeId,
    effects: EffectKnowledge,
    contract: CallableContract,
    dependencies: DependencySet,
}

enum ProofTerminator {
    Goto(ProofBlockId),
    Branch { condition: LogicExprId, then_block: ProofBlockId, else_block: ProofBlockId },
    Return(Option<LogicExprId>),
    Raise(LogicExprId),
    Diverge,
    Unsupported(UnsupportedProofOperation),
}
```

Normal return, exceptional exit, and divergence are distinct terminators. SSA-like versions or explicit phi terms preserve flow facts. Heap/state operations use an explicit memory model; they cannot be silently treated as pure functions.

### 12.2 Verification-condition generation

Partial-correctness VC generation follows weakest-precondition rules over normalized proof IR:

```text
requires
  -> entry assumptions
  -> statement/block transformers
  -> call preconditions and summaries
  -> branch obligations
  -> return ensures with result substitution
  -> invariant preservation obligations
```

Every VC records source origin, assumption set, semantic dependencies, effect/termination policy, and canonical logic fingerprint. Callers use callee contracts/summaries, never arbitrary callee implementation facts unless an explicit inlining policy and dependency fingerprint allows it.

For total correctness, termination VCs are additional obligations. A partial-correctness proof never upgrades totality.

### 12.3 Logic boundary

The first proof logic supports:

- booleans and propositional connectives;
- equality over modeled values;
- integer arithmetic within backend/model policy;
- nominal type predicates and ratified type relations as uninterpreted or kernel-defined facts;
- tuple/record projections for immutable modeled values;
- quantifier-free contracts first.

Quantifiers, recursive predicates, floating-point theories, heap framing, concurrency, reflection, FFI, and dynamic dispatch remain backend/model gates. Unsupported theory returns unknown; it does not approximate a proof.

## 13. Proof results, trust, and artifacts

### 13.1 Result algebra

Proof queries have a complete semantic and operational key:

```rust
struct ProofQueryKey {
    vc: VerificationConditionId,
    vc_fingerprint: ContentHash,
    assumptions: ContentHash,
    interface_fingerprint: ContentHash,
    semantic_model_version: SemanticModelVersion,
    world_and_native_revision: ContentHash,
    backend: BackendIdentity,
    backend_options: ContentHash,
    kernel_version: Option<ProofKernelVersion>,
    policy_and_budget: ContentHash,
}
```

Counterexample models retain bounded symbolic assignments, branch/exit trace, violated obligation, source mapping, backend provenance, and validation status. They are evidence for `Disproven`, not certificates for `Proven`.

```rust
enum ProofResult {
    Proven(ProofEvidence),
    Disproven(Counterexample),
    Unknown(ProofUnknownReason),
    Cancelled,
    BudgetExceeded(ProofBudgetReport),
    InternalError(IncidentId),
}

enum ProofEvidence {
    Certificate(CertificateArtifactRef),
    TrustedBackendAttestation(BackendAttestation),
    ConditionalOnAssumptions(AssumptionSetId),
}
```

This is the internal query result. User-facing/reflected proof results are only `Proven`, `Disproven`, and reasoned `Unknown`. Cancellation or budget exhaustion maps to a reasoned unknown when policy permits a semantic answer; infrastructure/internal failure remains a tool failure. None becomes proof evidence.

`Disproven` requires a validated counterexample trace or model that maps back to the VC and source assumptions. Backend `sat`/`unsat` text alone is not user evidence until parsed and validated under backend protocol.

### 13.2 Trust tiers

`DEC-PROOF-ARTIFACTS` is normative:

```rust
enum ProofTrust {
    KernelChecked,
    TrustedBackend { backend: BackendIdentity, version: Version },
    Assumed { authority: AssumptionAuthority, reason: AssumptionReason },
}
```

Only a certificate successfully checked by the in-process proof kernel is `KernelChecked`. A trusted solver without a checked certificate is `TrustedBackend`. User/native/core axioms are explicit assumptions. UI, diagnostics, metadata, and reflection preserve this distinction.

### 13.3 Persistent artifact

```rust
struct ProofArtifact {
    schema_version: ProofSchemaVersion,
    artifact_id: ProofArtifactId,
    vc_fingerprint: ContentHash,
    assumption_fingerprint: ContentHash,
    interface_fingerprints: Box<[(SemanticOwnerId, ContentHash)]>,
    semantic_model_version: SemanticModelVersion,
    backend: BackendIdentity,
    backend_version: Version,
    kernel_version: Option<ProofKernelVersion>,
    result: PersistentProofResult,
    trust: ProofTrust,
    dependencies: PersistentDependencySet,
    provenance: ProofProvenance,
}
```

Artifact identity is content/fingerprint based, not a path, process pointer, or store ID. Counterexamples and certificates are depth/size bounded. Hostile artifacts are schema-validated before allocation-heavy decoding. Metadata profiles in [02](02-runtime-reification-and-metadata.md) decide whether full artifacts, summaries, or no proof data ship.

### 13.4 Cache validity

A cached artifact is reusable only when all match:

- canonical VC fingerprint;
- assumptions and trust policy;
- referenced semantic interface fingerprints;
- semantic-model/type-system version;
- backend name/version/options;
- kernel version when certificate checked;
- proof policy and budget class;
- schema compatibility.

A source body edit that produces the same canonical VC may reuse the artifact. A contract, effect model, alias, relation, native summary, or imported interface change invalidates when named by the dependency set.

## 14. Query architecture, incrementality, and concurrency

### 14.1 Queries

Compiler-owned semantic DB owns:

```rust
KindOf(TypeFormKey)
ValidateVariance(DeclarationId)
SolveSignatureConstraints(DeclarationId)
InferEffects(CallableId)
AnalyzeTermination(CallableId)
LowerContract(CallableId)
BuildProofProcedure(CallableId)
GenerateVerificationConditions(CallableId)
Prove(VerificationConditionId, ProofPolicy)
CheckCertificate(ProofArtifactId)
```

Each answer carries status, generation stamp, dependencies, provenance, metrics, and terminal reason. LSP and CLI consume published answers; neither owns a second solver.

### 14.2 SCC publication

**Pyrefly architectural transfer.** Recursive kinds/aliases/callables/effects/termination summaries use SCC-local computation and batch publication. Safe worker-owned or mutex-backed cells come first. Duplicate cross-thread computation or atomic first-writer-wins publication requires profiling plus a separate memory-ordering proof/review.

Diagnostics and proof traces publish only with the canonical winning answer. Stale/cancelled candidates publish nothing.

### 14.3 Invalidation

Dependencies distinguish:

- declaration existence and kind scheme;
- normalized type surface and variance/bounds;
- alias body;
- callable contract;
- effect/exit summary;
- totality result;
- native metadata/catalog revision;
- proof backend/model/kernel policy.

Changing a body without changing a summary may preserve downstream proofs whose fingerprints remain equal. Clean full rebuild and incremental recomputation must produce structurally equal semantic/proof results and diagnostics.

## 15. Budgets, cancellation, and robustness

Required budgets:

```rust
KindBudget { unifications, depth }
RelationBudget { pairs, depth, union_width }
RowBudget { equations, fields, occurs_depth }
EffectBudget { call_edges, scc_rounds, set_width }
TerminationBudget { call_edges, scc_rounds, measure_depth }
VcBudget { blocks, terms, path_splits, formula_nodes }
ProofBudget { wall_time, solver_steps, memory_bytes, model_bytes }
```

Budgets are policy inputs and cache-key components. Exceeding one yields a named terminal result and metrics. It does not widen to `Dynamic`, assume purity, close an open row, accept a relation, or mark a VC proven.

Cancellation is checked at worklist batches, SCC rounds, CFG blocks, VC expansion, backend I/O, artifact decoding, and before publication. Backend processes receive bounded shutdown/kill handling. Cancelled work produces no reusable artifact unless a previously complete matching artifact already existed.

All recursive syntax/metadata/artifact decoders impose depth and allocation limits. Diagnostic cause chains and counterexample traces are cycle checked and bounded.

## 16. Diagnostics and observability

Required diagnostic categories:

| Domain | Examples |
|---|---|
| Kinds | occurs check, atomic mismatch, unsaturated proper position, escaping kind variable |
| Variance | invalid occurrence with composed path, unknown referenced variance |
| Bounds | unsatisfied upper/set bound, recursive budget, invalid F-bound owner |
| Rows | duplicate field, closed-row extra/missing, tail kind mismatch, row occurs check |
| Effects | declared-pure violation, effect subset failure, unknown dynamic/FFI effect |
| Totality | total required but unknown/diverging, unsupported measure |
| Contracts | impure/nonterminating predicate, invalid `old`, unsupported proof operation |
| Proofs | disproven with source-mapped model, unknown reason, trust policy rejection, stale artifact |

Metrics per generation/query:

- constraint count by kind;
- substitutions and unresolved variables;
- relation cache hits and pair/depth maxima;
- row equations/fields/occurs checks;
- effect/termination SCC sizes and rounds;
- proof IR blocks/terms and VC formula size;
- cache hits by artifact trust tier;
- backend duration/memory/cancellation;
- diagnostic and trace counts;
- stale-result rejections.

Tracing is separate from diagnostics. Turning traces off cannot change semantic/proof results.

## 17. Implementation units and gates

### Unit A — Kind variables and schemes

**Files/symbols:** `phalcom-semantic/src/types/kind.rs`, `parameter.rs`, `application.rs`, `store.rs`, new `kind_solver.rs`, export/metadata conversions, focused kind/application tests.

**Test first:** occurs checks, arrow unification, prenex generalization/instantiation, stable binder IDs, unsolved-variable rejection, deterministic scheme rendering.

**Command:** `cargo test -p phalcom-semantic kind`

**Gate:** no public kind-parameter syntax; no `Type :: Type`.

### Unit B — Variance and bounds

**Files/symbols:** `parameter.rs`, declarations/interface builders, new `variance.rs` and `constraint.rs`, substitution/relation modules, metadata schema, focused semantic/workspace tests.

**Test first:** callable position composition, nested constructor variance, mutable invariance, F-bound cycles, finite sets, publishability.

**Gate:** source binder syntax from [04](04-user-facing-type-syntax-and-lowering.md) lands only after semantic validators exist.

### Unit C — Record rows

**Files/symbols:** `kind.rs`, `store.rs`, `relation.rs`, new `row.rs`/`row_solver.rs`, annotation lowering, metadata DAG, reflection adapters.

**Test first:** canonical sorting, tail substitution, row equations, closed/open relations under capabilities, occurs checks, metadata round-trip, deterministic fingerprints.

**Gate:** no variant/effect row semantic reuse through untyped IDs.

### Unit D — Result-rich constraint solver

**Files/symbols:** relation/equality/application code, new constraint worklist/query modules, `SemanticDb` query/state modules from [01](01-implementation-architecture.md), diagnostics and metrics.

**Test first:** relation distinctions, recursive SCC convergence, open-world unknown, cancellation, every budget, stale publication, clean/incremental equivalence.

**Deletion criterion:** no formal relation path returns an unqualified boolean where recursion, dynamic knowledge, or budget can affect answer.

### Unit E — Effects and exits

**Files/symbols:** `phalcom-native-meta/src/primitive.rs`, semantic callable summary/flow/checker modules, metadata DAG, native adapters, compiler/LSP consumers.

**Test first:** pure/known/unknown, calls and SCC joins, raise/return/diverge separation, dynamic/DNU/FFI fallback, declaration subset, native metadata validation.

**Gate:** no public effect-polymorphism syntax; no purity proof from current syntactic `is_pure_expr`.

### Unit F — Totality

**Files/symbols:** semantic CFG/call graph, new `termination.rs`, contract summary, metadata/proof bridge.

**Test first:** acyclic termination, simple structural recursion, mutual SCC unknown, explicit total requirement failure, `Never` separation.

**Gate:** public syntax and trusted assumptions need separate ratification.

### Unit G — Canonical contract IR

**Files/symbols:** AST attributes, semantic typed-expression/contract modules, `phalcom-core/src/compiler/attributes.rs`, compile metadata adapters, runtime method metadata.

**Test first:** one contract identity feeds runtime/proof lowerings; `old` capture parity; compile-mode stripping never changes proof status; unsupported predicate remains executable but unproved when allowed.

**Migration:** preserve existing woven behavior while adding semantic IR; remove duplicate predicate interpretation only after differential tests.

### Unit H — Proof IR and VCs

**Files/symbols:** new `phalcom-prover` crate only after workspace/ownership ADR, semantic snapshot APIs, proof IR/lowering/WP modules, CLI integration behind opt-in flag.

**Test first:** straight-line/branch/loop obligations, normal/raise/diverge separation, calls, counterexample source mapping, deterministic VC fingerprints, unsupported operations.

**Gate:** crate/API ADR, proof logic subset, heap model, and CLI policy must be ratified first.

### Unit I — Backend, kernel, and artifacts

**Files/symbols:** prover backend protocol, certificate checker, artifact schema, metadata profile integration, cache store, reflection result adapters.

**Test first:** trust-tier preservation, certificate corruption, backend mismatch, cache invalidation matrix, hostile artifacts, budget/cancel, no false `Proven`.

**Gate:** first backend and certificate format require threat-model review. No backend result is `KernelChecked` without local checking.

## 18. Verification matrix

| Invariant | Required evidence |
|---|---|
| Stable binders differ from solver variables | type-level API separation plus escape tests |
| Kind generalization is prenex | nested/higher-rank rejection and deterministic schemes |
| Variance is sound | occurrence-path property tests and declaration fixtures |
| Record rows remain record-specific | compile-time wrappers and cross-domain rejection tests |
| Relations stay distinct | counterexamples where consistency differs from subtype/assignability |
| Effects do not encode exits | raise/diverge/return examples with identical effect sets |
| `Never` does not prove divergence | unconditional raise and infinite-loop comparison |
| Total means proven termination | total-declaration unknown rejection |
| Runtime contracts are not proofs | compile-mode and artifact/trust assertions |
| Proof cache is sound | every key component mutation invalidates as specified |
| Incremental equals clean | differential corpus across imports/contracts/effects/aliases |
| Budgets never become success | terminal-state assertions for every solver/prover layer |

## 19. Intentional gates

Stop before implementation or public exposure of:

- kind-parameter syntax;
- higher-rank kinds, `Type :: Type`, universes, or dependent kinds;
- lower/equality/negative/associated-type constraints;
- generic parameter defaults;
- variant rows or shared untyped row IDs;
- public effect rows/effect-polymorphism syntax;
- totality/measure/assumption syntax;
- quantified logic, heap framing, concurrency proofs, or floating-point proof semantics;
- a trusted solver/backend choice;
- a proof kernel/certificate format;
- runtime proof authority or proof-term programming.

Each gate requires a decision entry in [07](07-consolidated-implementation-plan-and-decision-register.md), focused tests, migration plan, and reviewer ownership.

## 20. What this must not preclude

Architecture must preserve room for:

- explicitly ratified higher-rank or richer kind features without changing current scheme identity;
- protocol coherence and associated abstractions through new constraint variants;
- distinct variant/effect rows sharing typed solver utilities;
- effect handlers, regions, and resource/ownership analyses;
- richer termination measures and total-correctness methods;
- multiple prover backends and an offline certificate kernel;
- quantifiers, heap framing, concurrency, and floating-point theories behind versioned models;
- package-carried proof artifacts and multiple retention profiles;
- safe parallel query publication after ownership and performance evidence.

It need not preserve compatibility with one untyped variable domain, universal rows, `Type :: Type`, unknown-as-success, assumed purity/totality, or backend verdicts without trust.

## 21. Take directly / Adapt / Reject

### Take directly

- existing atomic/arrow kind kernel and owner/index type parameter identity;
- kind-checked partial type application and canonical TypeStore construction;
- current callable variance and closed-record relation seeds;
- native effect, raises, and return-flow vocabulary as declared input;
- current executable contract attributes as runtime behavior to preserve;
- Pyrefly query identity, SCC, invalidation, budget, observability, and publication disciplines.

### Adapt

- type/kind store APIs into persistent-versus-solver identity boundaries;
- boolean/coarse relations into explicit result-rich constraints;
- closed records into record-specific rows with separate tails;
- native effect specs into compiler-owned effect/exit summaries;
- executable contracts into one semantic IR with runtime and proof lowerings;
- solver results into persistent fingerprinted artifacts with named trust.

### Reject

- `Type :: Type`, dependent kinds/types, universes, and arbitrary kind evaluation;
- parameter identity by text or solver-variable escape;
- one universal row ID or semantic domain;
- `Never` as divergence/termination evidence;
- missing/unknown effect as purity;
- totality by default;
- runtime guards, stripped checks, solver text, or runtime object identity as proof authority;
- unsafe/atomic cache publication before tests, profiling, and memory-model review.

## 22. Final normative summary

1. `Type` and `RecordRow` are distinct atomic kinds; arrow builds constructors.
2. Kind polymorphism is prenex with stable `KindParameterId`; `KindVarId` is solver-local.
3. Variance is declaration-site and validated compositionally.
4. Bounds and finite sets constrain types without inventing coercions or subtyping.
5. Record rows store sorted fields separately from an explicit tail.
6. Structural equality, equivalence, subtyping, assignability, and consistency have distinct result-rich APIs.
7. Effects, exits, return types, and termination are separate semantic products.
8. Ordinary correctness is partial; total declarations require termination evidence.
9. Runtime contract guards and static proofs share contract identity but not authority.
10. Proof results preserve unknown/cancel/budget/internal states and explicit trust tiers.
11. Proof artifacts are persistent, fingerprinted, validated evidence.
12. No solver variable, stale answer, dynamic fallback, or stripped guard may masquerade as proof.
