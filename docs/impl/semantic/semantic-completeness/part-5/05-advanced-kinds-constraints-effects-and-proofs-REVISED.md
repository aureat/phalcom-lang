# 05 — Advanced Type-Level Semantics, Effects, Totality, Contracts, and Proofs

**Date:** 2026-08-22
**Status:** Ratified advanced semantic architecture; implementation is staged behind explicit gates; prover/backend choices remain unratified
**Authority:** advanced semantic domains above the canonical generic calculus, including record-row solving, effect and exit summaries, termination and `@total`, canonical contract semantics, verification conditions, proof evidence/trust, and advanced kind-polymorphism extension points
**Primary owners:** `phalcom-semantic`, `phalcom-native-meta`, compiler contract bridge in `phalcom-core`, metadata extensions defined through Spec 02, reflection projections defined through Spec 03, CLI/LSP/REPL consumers
**Hard semantic dependencies:** [01 — Compiler-Owned Typing Implementation Architecture](01-implementation-architecture.md), [01.5 — Canonical Generic Type Semantics and Declaration Model](01.5-canonical-generic-type-semantics-and-declaration-model.md)
**Source-syntax dependency:** [04 — User-Facing Type Syntax and Lowering](04-user-facing-type-syntax-and-lowering.md)
**Runtime projection dependencies:** [02 — Runtime Reification, Semantic Metadata, and Artifact Contract](02-runtime-reification-and-metadata.md), [03 — User-Facing Typing Reflection API and Capabilities](03-reflection-api-and-capabilities.md)
**Non-goals:** redefining ordinary generic semantics; redefining type-lambda semantics; changing selector identity or runtime dispatch; dependent types; `Type :: Type`; universe polymorphism; proof terms as ordinary values; effect handlers; a mandatory SMT/prover vendor; trusting runtime guards as proofs

---

## 0. Revision contract

This document supersedes the earlier `05-advanced-kinds-constraints-effects-and-proofs.md` wherever that document overlaps with Spec 01.5 or the revised Specs 02–04.

The previous document mixed two different layers:

1. the **base generic type calculus** needed by every typed Phalcom program; and
2. genuinely **advanced semantic analyses** such as effects, termination, rows, contracts, and proofs.

That ownership is now corrected.

Spec 01.5 is authoritative for:

- generic declaration scopes;
- class and method generic parameters;
- parameter identity by owner and index;
- explicit kind annotations such as `F: Type -> Type`;
- `Type` and ordinary arrow-kind application;
- partial type application;
- type lambdas such as `<T> =>> Result<T, Error>`;
- alpha equivalence and beta normalization of type lambdas;
- declaration-site `+T` / `-T` variance;
- `where` constraints;
- subtype and semantic-equivalence generic constraints;
- owner-relative `Self`;
- generic inheritance templates;
- lazy specialization environments/views;
- canonical callable and field semantic signatures;
- ordinary method-generic local inference;
- generic publishability rules.

This specification **must not create parallel representations for any of those concepts**.

In particular, the following shapes from the older Spec 05 are deleted:

```rust
// superseded — constraints do not live here
struct TypeParameterData {
    ...
    bounds: Box<[TypeBound]>,
    default: Option<TypeId>,
}

// superseded — finite sets are not a core generic constraint
TypeBound::FiniteSet(...)

// superseded — ordinary generic relations are not owned by Spec 05
TypeConstraint::Subtype(...)
TypeConstraint::Equivalent(...)

// superseded — Self is already defined in 01.5
struct SelfTypeTerm { ... }
```

The initial generic constraint language remains the 01.5 model:

```text
where T <: U
where T == U
where L <: T
```

Finite exact-set constraints, generic defaults, associated types, negative constraints, and protocol-specific constraint syntax remain separate future decisions.

The revised document therefore owns only semantic domains that **extend or analyze programs built on top of 01.5**.

### 0.1 Normative ownership table

| Concern | Authority |
|---|---|
| Type/kind/class stratification | 01 + completed two-axis tower |
| Canonical generic/type-lambda calculus | 01.5 |
| Source spelling and parser recovery | 04 |
| Durable metadata carriage | 02 |
| Runtime reflection selectors/capabilities | 03 |
| Open record-row solving | **05** |
| Effects and exits | **05** |
| Termination and `@total` | **05** |
| Canonical contracts as semantic inputs | **05** |
| VC/proof semantics, evidence, trust | **05** |
| Optional future kind polymorphism | **05, gated** |
| Runtime contract weaving behavior | existing language/runtime contract specifications + compiler implementation; 05 must preserve it |

### 0.2 Central design law

The advanced checker publishes six independent semantic products for a callable:

```text
1. normal return type
2. effect summary
3. exit summary
4. termination knowledge
5. contract set
6. proof evidence/status
```

No product may silently stand in for another.

The following inferences are invalid:

```text
return type == Never      => diverges                  // false
return type == Never      => terminates                // false
known empty effects       => terminates                // false
known empty effects       => cannot raise              // false
@total                    => pure                      // false
@total                    => cannot raise              // false
runtime contract passed   => statically proved         // false
backend said SAT/UNSAT    => trusted proof             // false without policy/evidence
missing effect metadata   => pure                      // false
no detected loop          => total                     // false
```

This separation is the single most important correctness invariant in this specification.

---

# Part I — Repository state and semantic boundaries

## 1. Purpose

Phalcom already has a useful type/kind kernel, native primitive metadata, executable contracts, immutable semantic snapshots, and runtime reflection infrastructure. The next semantic layer must use those systems without turning static reasoning into a second runtime or changing the object model.

This specification defines how Phalcom can answer questions such as:

- Does this callable have externally visible effects?
- Which exceptional/control exits are possible?
- Can termination be proven?
- Does `@total` hold?
- Can an open structural record be specialized safely?
- What do `@requires`, `@ensures`, and `@invariant` mean to a static verifier?
- Which verification conditions are generated?
- What exactly does `Proven` mean?
- Which assumptions and semantic versions make a proof reusable?
- How do cancellation, budgets, opacity, dynamic dispatch, reflection, and FFI block reasoning without being mistaken for counterexamples?
- How are these answers shared by compiler, CLI, LSP, REPL, metadata, and runtime reflection without forcing ordinary execution to allocate proof/type objects?

The guiding philosophy is the same as the rest of the typing architecture:

> rich semantics when information is requested; minimal runtime cost when it is not.

## 2. Repository-grounded current state

This revision was checked against `aureat/phalcom-lang` `main` at commit `a43f26e0ddd6b1d6e37ddf7a0b9588769bb41f3e` (2026-08-22).

### 2.1 Native effects, raises, and return flow already exist as declarations

`phalcom-native-meta/src/primitive.rs` already defines:

```rust
pub enum RaisesSpec {
    Unknown,
    Known(&'static [TypeExprSpec]),
}

pub enum NativeEffect {
    Mutation,
    Io,
    Scheduling,
    Reflection,
    Nondeterminism,
    Blocking,
}

pub enum EffectSpec {
    Unknown,
    Pure,
    Known(&'static [NativeEffect]),
}

pub enum ReturnFlowSpec {
    Value,
    Receiver,
    Argument(usize),
    Never,
    Unknown,
}
```

`PrimitiveSurfaceSpec` stores `raises`, `effects`, and `flow` beside its semantic callable type.

These are valuable authoritative declarations for native code. They are **not yet** a compiler-owned effect/exit/termination analysis:

- `EffectSpec::Unknown` does not carry a reason;
- there is no source-callable effect summary table;
- there is no fixed-point inference across source call graphs;
- raises are metadata, not a complete control-exit summary;
- `ReturnFlowSpec::Never` says there is no normal returned value, not why;
- there is no termination metadata field;
- there is no proof provenance attached to any of these declarations.

The revised architecture adapts these existing facts rather than replacing them.

### 2.2 Runtime contracts are executable guards today

`phalcom-core/src/compiler/attributes.rs` already has an explicit `CompileMode` contract-retention matrix:

- `Debug`: requires/ensures/invariant guards woven, metadata retained;
- `Release`: requires woven, ensures/invariants stripped, metadata normally retained;
- `Unchecked`: all guards stripped, metadata stripped by default.

The same file has `is_pure_expr`, a conservative syntax-tree predicate used by the existing contract implementation. It recognizes assignments, mutating sends, collection expansion, blocks, calls, and other AST shapes.

That predicate is useful as a **current runtime-contract eligibility heuristic**. It is not the semantic effect analysis defined by this document. A method call that happens not to have a known mutating selector name cannot become statically pure merely because `is_pure_expr` returned true.

The revision therefore preserves the existing runtime behavior while removing any temptation to treat the heuristic as proof evidence.

### 2.3 Method objects carry runtime contracts, not static proof state

`phalcom-core/src/method/object.rs` stores the runtime method implementation, calling signature, holder/access metadata, contract predicate closures, and attributes.

That is the right boundary.

This specification does **not** add effect sets, termination proofs, VCs, proof certificates, generic static signatures, or prover state to every `MethodObject`.

Those products remain external semantic metadata indexed by stable semantic IDs, with the VM side-table bridge defined by Specs 02–03 when runtime reflection explicitly requests them.

### 2.4 The current type relation kernel is not the proof engine

`phalcom-semantic/src/types/relation.rs` currently contains nominal/union/tuple/record/callable subtype logic and a coarse assignability result. Spec 01 and Spec 01.5 are already responsible for replacing/hardening those base relations.

Spec 05 consumes those relation APIs as trusted semantic queries. It must not create a second subtype checker inside the prover.

A proof engine may ask:

```text
is A a subtype of B?
is this type-form semantically equivalent to that one?
what specialized callable signature is selected?
```

but the answers come from the canonical semantic database and its bounded relation machinery.

### 2.5 No static prover exists yet

There is currently no accepted implementation of:

- canonical contract IR;
- VC generation;
- a proof logic model;
- a heap/frame model for object fields;
- a solver-backend protocol;
- a certificate kernel;
- a proof artifact cache;
- static counterexample replay;
- proof trust policy.

Existing executable contracts are therefore specification inputs and runtime enforcement only.

This document must not describe them as an already-existing proof system.

### 2.6 Record-row syntax is ratified but semantically gated

Revised Spec 04 reserves and specifies:

```phalcom
#{ name: String, age: Int }
#{ name: String, | R }
#{ | R }

type Named<R: RecordRow> = #{ name: String, | R }
```

The comma before a tail after known fields is mandatory. The tail must resolve to a binder of kind `RecordRow`.

Spec 01.5 deliberately leaves the full row solver here. Therefore this document is the semantic enablement gate for that syntax.

### 2.7 Ordinary kind polymorphism is intentionally absent

Spec 01.5 gives Phalcom explicit constructor kinds such as:

```text
Type
Type -> Type
Type -> Type -> Type
```

and explicit higher-kinded binders such as:

```phalcom
class Functor<F: Type -> Type>
```

This already supports higher-kinded programming without quantified kind variables.

No source syntax for a kind variable, no `forall K`, and no generalized kind scheme is currently ratified. The revised Spec 05 preserves an architecture for future prenex kind polymorphism, but does not make it a prerequisite for effects, rows, totality, or proofs.

---

## 3. Non-negotiable semantic laws

### LAW-ADV-1 — The runtime object model does not change

Effects, proofs, row solving, and totality never enter:

- selector identity;
- method lookup keys;
- ordinary runtime dispatch;
- class identity;
- metaclass identity;
- object layout;
- allocation layout;
- per-instance generic state.

### LAW-ADV-2 — `Never` is only a normal-return type fact

A callable with normal return type `Never` has no normal value return.

It may:

- always raise;
- diverge;
- terminate by process exit;
- transfer control through another terminal operation;
- reach an unreachable state;
- combine several of those behaviors.

Therefore:

```text
Never != divergence
Never != termination proof
Never != exception freedom
```

### LAW-ADV-3 — `@total` means termination proven only

The ratified meaning is:

> every admitted execution of the annotated callable terminates according to the language's termination model.

It does **not** imply:

- purity;
- zero effects;
- no exceptions;
- no allocation;
- no mutation;
- no I/O;
- no scheduling;
- no reflection;
- a normal returned value.

A total callable may terminate by raising if the language's chosen termination model counts exceptional completion as termination. This document does; `@total` is a liveness/termination claim, not a normal-return claim.

### LAW-ADV-4 — Unknown is never success

Unknown effect information is not purity.

Unknown termination is not totality.

Unknown contract proof is not proven.

Opaque native code is not trusted because it compiled.

A dynamic boundary is not a proof hole that defaults to true.

### LAW-ADV-5 — Cancellation and budget exhaustion are terminal states

They never become:

- `Unknown` merely to simplify callers;
- `Blocked` if retry policy matters;
- `Proven` because a partial fixed point looked stable;
- `Disproven` because the backend timed out.

### LAW-ADV-6 — Runtime guards are not proof evidence

A runtime `@requires`/`@ensures`/`@invariant` check may enforce a contract on one execution.

That observation does not prove the contract for all executions.

Stripping a guard does not invalidate a separately established static proof. Retaining a guard does not create one.

### LAW-ADV-7 — Backend output is not automatically trusted proof

A solver process returning `unsat`, `sat`, `proved`, or equivalent text is an input to the proof pipeline.

`ProofResult::Proven` requires evidence accepted by the active trust policy.

### LAW-ADV-8 — Every published advanced product is snapshot-scoped and publishable

No solver-local row/effect/proof variable may escape into:

- declaration interfaces;
- immutable semantic snapshots;
- durable metadata;
- runtime reflection descriptors;
- proof artifacts.

### LAW-ADV-9 — Static semantic authority remains compiler-owned

The LSP, CLI, REPL, metadata loader, runtime reflection, and proof backend do not implement independent versions of the formal semantics.

### LAW-ADV-10 — Advanced reasoning is demand-driven

Normal compilation and execution do not require:

- building proof IR;
- invoking a prover;
- materializing reflection descriptors;
- allocating proof objects;
- deep-validating runtime generic structures.

Declared requirements such as `@total`, explicit verification modes, or requested tooling queries may trigger the relevant work.

---

# Part II — Advanced semantic product model

## 4. Callable semantic product bundle

A canonical callable signature from Spec 01.5 is the identity anchor. Advanced analyses publish additional products keyed by `CallableId` rather than embedding them into runtime methods.

Conceptually:

```rust
struct CallableAdvancedFacts {
    callable: CallableId,
    effects: EffectKnowledge,
    exits: ExitKnowledge,
    termination: TerminationKnowledge,
    contracts: ContractSetId,
    proof_summary: ProofSummary,
}
```

This is a conceptual aggregation, not a requirement to allocate one monolithic struct. The `SemanticDb` should expose independent queries so a hover request for a return type does not trigger termination or proof generation.

Recommended query decomposition:

```text
callable_signature(CallableId)
callable_effects(CallableId)
callable_exits(CallableId)
callable_termination(CallableId)
callable_contracts(CallableId)
callable_proof_summary(CallableId, ProofPolicy)
```

Dependencies are explicit:

```text
signature
   │
   ├──> effects
   ├──> exits
   ├──> termination
   └──> contracts
            │
            └──> verification conditions
                       │
                       └──> proof result
```

Termination may depend on exit/call-graph information, and proof VCs may depend on effects/termination/contracts, but the products remain separately queryable and separately cached.

### 4.1 Why not one `CallableType` containing everything?

Because return typing, effect analysis, termination, and proving have different:

- invalidation dependencies;
- budgets;
- cancellation points;
- retention profiles;
- runtime relevance;
- trust requirements;
- computational costs.

Putting them into one canonical type would make a type lookup depend on proof state and would contaminate type equivalence with analysis configuration.

Phalcom explicitly rejects that architecture.

### 4.2 Knowledge wrappers are separate from semantic values

A known empty effect set is a semantic fact.

An unavailable/unknown effect set is epistemic state.

Likewise, a known termination proof and a missing termination answer must not share a sentinel semantic value.

Every advanced domain follows this separation.


# Part III — Record rows

## 5. Record-row semantic domain

Record rows are the first advanced kind/domain that this document actively defines.

The surface syntax was already reserved by Spec 04; this section makes its semantics precise enough to implement.

### 5.1 Kind

Add one atomic kind:

```rust
RecordRow
```

The complete kind grammar becomes conceptually:

```text
Kind ::= Type
       | RecordRow
       | Kind -> Kind
```

`RecordRow` is not a proper type. A value cannot be annotated merely as `RecordRow`.

These are valid kind declarations:

```phalcom
<R: RecordRow>
<F: Type -> Type>
```

These roles remain distinct:

```text
#{ name: String }      :: Type
R                      :: RecordRow
#{ name: String, | R } :: Type
```

Arrow kinds may mention `RecordRow` if a future constructor genuinely consumes or produces rows, but no such public abstraction is assumed by this specification.

### 5.2 Binder identity

A row parameter may use the same stable generic binder identity mechanism from 01.5:

```rust
TypeParameterId { owner, index }
```

The binder's declared kind determines its semantic domain.

A parameter with kind `RecordRow` must **not** be lowered to ordinary `TypeData::Parameter`, because that would falsely make it a type form.

Instead, the row domain references the binder identity directly.

This preserves the useful 01.5 law that generic binders are owner/index identities while preventing cross-domain term confusion.

### 5.3 Canonical row representation

Recommended canonical representation:

```rust
pub struct RecordRowData {
    pub fields: Box<[RecordRowField]>,
    pub tail: RecordRowTail,
}

pub struct RecordRowField {
    pub name: FieldName,
    pub ty: TypeId,
}

pub enum RecordRowTail {
    Closed,
    Parameter(TypeParameterId),
}

pub struct RecordTypeData {
    pub row: RecordRowId,
}
```

`RecordRowId` belongs to a row store/segment associated with the same semantic store epoch as the referenced `TypeId`s.

The implementation may choose a compact embedded closed-row representation for very small records if benchmarks justify it, but the semantic API must behave as though every record type refers to one canonical row.

Canonicalization rules:

1. field names are sorted by stable field-name ordering;
2. duplicate fields are invalid, never last-write-wins;
3. each field type must be a proper type;
4. a row-tail parameter must have kind `RecordRow`;
5. a closed row has no hidden fresh variable;
6. semantically equivalent closed rows canonicalize equivalently regardless of source field order;
7. source field order may be retained separately for diagnostics/formatting but never controls semantic equality.

### 5.4 Solver-local row terms

Inference needs variables that never enter canonical metadata:

```rust
enum RecordRowTerm {
    Canonical(RecordRowId),
    Var(RecordRowVarId),
    Extend {
        fields: SmallVec<[RecordRowFieldTerm; 4]>,
        tail: Box<RecordRowTerm>,
    },
}
```

`RecordRowVarId` is query-local.

It is not:

- `TypeParameterId`;
- `InferVarId`;
- `TypeId`;
- an ObjRef;
- a durable metadata ID.

A solved result is zonked to canonical row data before publication.

### 5.5 Row equality

Row equality is semantic field-set equality plus tail equality after substitution.

For closed rows:

```text
#{ a: A, b: B }
==
#{ b: B, a: A }
```

provided `A` and `B` are semantically equivalent in their matching fields.

Open rows unify by subtracting common fields and solving tails.

Example:

```text
#{ name: String, age: Int }
=
#{ name: String, | R }
```

solves:

```text
R = #{ age: Int }
```

### 5.6 Lacks constraints

Row extension must prove that a newly introduced label is absent from the unresolved tail.

Internal constraint:

```rust
RecordRowLacks {
    row: RecordRowTerm,
    field: FieldName,
}
```

Without this constraint, substitution could create duplicates:

```text
R = #{ x: Int }
extend(R, x: String)
```

which would be semantically ambiguous and unsound.

Lacks constraints propagate through known fields and row substitutions. If the tail remains open, the constraint remains pending rather than being guessed true.

### 5.7 Occurs check

Reject direct and indirect infinite rows:

```text
R = #{ next: Int, | R }
```

unless Phalcom later adopts an explicit recursive-row binder. This specification does not.

The row solver therefore performs an occurs check before binding a row variable to a term containing that same representative.

### 5.8 Record relations and access capability

Structural record compatibility depends on how the record is used.

For **read-only** access, width/depth subtyping is useful:

```text
#{ name: String, age: Int }
<:
#{ name: String }
```

if every required field type relates covariantly.

For a writable structural view, blindly applying width/depth covariance is unsound. Mutation may require invariant field types and potentially exact field sets depending on the capability.

Therefore relation queries receive an explicit capability/policy:

```rust
enum RecordAccess {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}
```

The exact mutation-capability framework may later grow, but relation code must never infer mutability from whether a record happened to appear in one syntax position.

Initial rules:

- `ReadOnly`: required-field width + covariant field relation;
- `ReadWrite`: matching writable fields invariant; extra fields do not disappear from the actual value, but whether width is admitted depends on the enclosing operation's aliasing model;
- `WriteOnly`: contravariant requirements may be modeled later; initial implementation may conservatively require exact/equivalent field types.

If the relevant capability is unknown, the relation returns a blocked/unknown outcome rather than choosing the most permissive rule.

### 5.9 Row solver result

```rust
enum RecordRowSolveResult {
    Solved(RecordRowSolution),
    Rejected(RecordRowFailure),
    Blocked(RecordRowBlockedReason),
    Cancelled,
    BudgetExceeded(RowBudgetReport),
    InternalFailure(IncidentId),
}
```

`Rejected` means the row equations are contradictory.

`Blocked` means required semantic knowledge is absent/open.

Neither cancellation nor budget exhaustion is folded into `Blocked`.

### 5.10 Separate row domains

Record rows are not a universal “row” type shared with effects or variants.

Future effect rows or variant rows may share implementation utilities such as:

- sorted label/set storage;
- union-find;
- tail substitution;
- occurs checks;
- worklists;
- budget accounting.

But they must retain distinct:

- ID types;
- term types;
- relation rules;
- label domains;
- reflection classes;
- metadata tags.

A generic Rust helper is acceptable only behind typed wrappers that make cross-domain mixing impossible.

---

# Part IV — Optional future kind polymorphism

## 6. Kind polymorphism is a gated extension, not base semantics

Higher-kinded Phalcom already works with explicit kinds:

```phalcom
class Functor<F: Type -> Type>
```

and type lambdas:

```phalcom
<T> =>> Result<T, Error>
```

That does **not** require kind polymorphism.

Kind polymorphism would mean quantifying over kinds themselves, conceptually enabling a declaration to abstract over whether an argument has kind `Type`, `Type -> Type`, and so on.

This feature is deliberately deferred.

### 6.1 Requirements if introduced

If kind polymorphism is later ratified, use a **prenex scheme** architecture:

```rust
struct KindScheme {
    parameters: Box<[KindParameterId]>,
    body: KindId,
}
```

with three identity classes:

```rust
KindParameterId  // stable published binder
KindVarId        // flexible solver-local variable
KindSkolemId     // rigid solver-local instantiation
```

They are never aliases for `TypeParameterId` or `InferVarId`.

### 6.2 Generalization boundary

Generalization may occur only at an explicit declaration/interface publication boundary.

No local expression acquires an implicitly generalized kind scheme merely because inference discovered an unconstrained kind variable.

No solver variable escapes.

### 6.3 Rigid checking

Checking whether one generalized scheme subsumes another instantiates universally quantified binders as rigid skolems. An implementation must not solve a declaration's universal kind variable to fit one use site.

### 6.4 Kind unification

A future solver may use:

```rust
enum KindTerm {
    Canonical(KindId),
    Var(KindVarId),
    Rigid(KindSkolemId),
    Arrow {
        parameters: SmallVec<[KindTerm; 2]>,
        result: Box<KindTerm>,
    },
}
```

with occurs checks, arity checks, deterministic representatives, cancellation, and budgets.

### 6.5 Explicit rejections

Even if prenex kind polymorphism lands, reject:

- `Type :: Type`;
- `Type0`, `Type1`, ... universes as an implicit consequence;
- dependent kinds indexed by runtime values;
- arbitrary kind-level execution;
- higher-rank kind quantification unless separately ratified;
- kind lambdas as an automatic consequence of type lambdas;
- one variable namespace shared by kinds and types.

### 6.6 Implementation gate

Do not add `KindParameterId`, `KindVarId`, public syntax, metadata nodes, or reflection selectors merely to “future-proof” the current implementation.

The existing `Type`/arrow kind kernel plus `RecordRow` is sufficient for the features actively specified here.

The architecture must leave room for kind schemes, but unused complexity should not enter hot paths before there is a concrete use case.

---

# Part V — Effects and exits

## 7. Effect semantics

### 7.1 Definition

An effect summary describes externally relevant capabilities exercised by evaluating a callable, directly or transitively, under the current semantic model.

Effects are **not** exceptions and are **not** termination facts.

The initial canonical atoms intentionally align with the native metadata already present in the repository:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
enum EffectAtom {
    Mutation,
    Io,
    Scheduling,
    Reflection,
    Nondeterminism,
    Blocking,
}
```

This first version is deliberately coarse.

Do not prematurely add field regions, ownership regions, capabilities parameterized by arbitrary values, or a region calculus merely because a future effect system might benefit from them.

A later extension may refine `Mutation` into receiver/global/argument/region-specific atoms behind a semantic-model version gate.

### 7.2 Canonical set

```rust
struct EffectSetData {
    atoms: Box<[EffectAtom]>,
}
```

Atoms are sorted and unique.

Required algebra:

```text
join(A, B) == join(B, A)
join(A, A) == A
join(A, empty) == A
subset(A, B) is reflexive and transitive
```

Interned canonical sets may use `EffectSetId` for compact snapshot-local storage.

### 7.3 Knowledge is not the set

```rust
enum EffectKnowledge {
    Known(EffectSetId),
    Opaque(EffectOpaqueReason),
    Invalid(EffectDiagnosticSet),
}
```

Outer query execution adds:

```text
Cancelled
BudgetExceeded
InternalFailure
```

A known empty set means “no effects in the current effect model.”

`Opaque(...)` never means empty.

Useful opacity reasons include:

```rust
enum EffectOpaqueReason {
    MissingNativeMetadata,
    DynamicDispatch,
    ReflectivePerform,
    DoesNotUnderstandBoundary,
    ForeignBoundary,
    UnknownDependency,
    UnsupportedConstruct,
}
```

### 7.4 Native adaptation

Current native metadata maps as follows:

```text
EffectSpec::Pure       -> Known(empty)
EffectSpec::Known(xs)  -> Known(canonicalize(xs))
EffectSpec::Unknown    -> Opaque(MissingNativeMetadata)
```

The current `NativeEffect` variants map one-to-one to the initial `EffectAtom`s.

No native metadata string or macro spelling becomes a separate semantic lattice.

### 7.5 Source inference

Effects are inferred bottom-up from expression semantics and callee summaries.

Conceptually:

```text
literal / local read                  -> {}
field read                            -> {}
local assignment                      -> {} or local-only internal fact
receiver/global/object mutation       -> {Mutation}
I/O primitive                         -> {Io}
scheduler/fiber scheduling            -> {Scheduling}
reflection/type-runtime inspection    -> {Reflection}
nondeterministic primitive            -> {Nondeterminism}
blocking primitive                    -> {Blocking}
call f(args)                           -> effects(f) joined with argument evaluation
```

Whether local-variable assignment counts as semantic `Mutation` should follow the observable-effect definition: mutating a compiler-local slot that cannot escape is not an externally visible mutation effect. Mutating object state, captured mutable state, globals, or externally visible storage is.

The first implementation may conservatively classify uncertain writes as `Mutation`.

### 7.6 Dynamic/open-world calls

A dynamic call with no sound target summary does **not** become the top set of all effects and does not become empty.

It becomes opaque:

```text
Opaque(DynamicDispatch)
```

Why not “all effects”? Because that loses the epistemic distinction between “known to perform every capability” and “we do not know which capabilities it performs.”

Policies that only care whether purity is proven can treat either state as “not proven pure,” while diagnostics and future retry strategies retain the reason.

### 7.7 Recursive call graphs

Effect inference over recursive callables is a monotone fixed-point problem over a finite lattice of known effect atoms plus opacity.

Use the Spec 01 SCC/query infrastructure:

```text
build callable dependency SCC
seed direct local effects
iterate callee joins
stop when stable
publish atomically
```

Rules:

- iteration order is deterministic;
- opacity propagates unless a more specific sound summary is available;
- budgets are checked per SCC round and edge expansion;
- cancellation prevents publication;
- a previous complete cached result may remain available but is not replaced by partial work;
- incremental invalidation follows callable/body/dependency fingerprints.

### 7.8 “Pure” semantics

For the initial system:

```text
pure(callable) iff callable_effects(callable) == Known(empty)
```

This is an effect-model notion of purity, not an assertion of mathematical referential transparency.

Fresh internal allocation is not an effect atom in version 1. If future optimization or proof work needs allocation/identity effects, add them explicitly rather than silently changing what empty means.

### 7.9 Effects do not imply exits

A callable can be effect-free and raise.

A callable can mutate and always return.

A callable can block and still terminate.

A callable can have no modeled effects and diverge.

No effect relation answers any of those questions.

---

## 8. Exit semantics

### 8.1 Purpose

The exit summary describes how control may leave or fail to leave a callable activation.

Recommended model:

```rust
struct ExitSummary {
    may_return_normally: bool,
    raises: RaiseKnowledge,
    divergence: DivergenceKnowledge,
    may_exit_process: bool,
    may_suspend: bool,
}
```

This model is independent from the callable's normal return type.

### 8.2 Raises

```rust
enum RaiseKnowledge {
    None,
    Known(TypeSetId),
    Opaque(RaiseOpaqueReason),
}
```

A known set contains exception types that may escape the callable under the current static model.

The set uses canonical semantic types, not runtime class handles.

`RaisesSpec::Known([])` maps to `None`.

`RaisesSpec::Unknown` maps to an opaque reason.

### 8.3 Divergence

```rust
enum DivergenceKnowledge {
    ProvenAbsent,
    Possible,
    Opaque(DivergenceOpaqueReason),
}
```

`ProvenAbsent` is stronger than “no loop detected.” It may only be emitted when the termination analysis establishes that every admitted path completes.

`Possible` means analysis has found a path/cycle that may not terminate or cannot satisfy a ranking proof under current facts.

`Opaque` means the analyzer lacks enough semantic information to decide.

A more refined future model may distinguish `ProvenDiverges` from `MayDiverge`; version 1 does not need that distinction to enforce `@total`.

### 8.4 Normal return

`may_return_normally` is a control-flow fact.

It does not carry the normal return type; that type already belongs to the canonical callable signature.

This keeps:

```text
normal-return shape
```

separate from:

```text
whether a normal-return path exists.
```

### 8.5 Suspension and blocking

`Blocking` is an effect capability.

`may_suspend` is a control property.

A scheduler operation may be nonblocking but suspend the current fiber; a blocking primitive may return without semantic suspension. Do not collapse the two.

### 8.6 Return-flow refinement

Current native `ReturnFlowSpec::{Receiver, Argument(i), Value, Never, Unknown}` remains useful as a **value-flow summary**.

It can improve type/effect/proof reasoning:

```text
Receiver     -> normal result aliases receiver
Argument(i)  -> normal result aliases argument i
Never        -> no normal return
Value        -> normal value, no alias refinement
Unknown      -> no refinement
```

It is not itself the exit summary.

---

# Part VI — Termination and `@total`

## 9. Termination knowledge

### 9.1 Semantic result

Termination is a separate query:

```rust
enum TerminationKnowledge {
    Proven(TerminationEvidence),
    Refuted(TerminationCounterevidence),
    Blocked(TerminationBlockedReason),
}
```

Execution status remains outside the semantic value:

```text
Cancelled
BudgetExceeded
InternalFailure
```

`Refuted` means the analyzer has sound evidence that the callable is not total under the active model, for example a reachable recurrence with no decreasing measure where the recurrence itself establishes a nonterminating cycle in the supported logic.

Most difficult cases should be `Blocked`, not `Refuted`.

The implementation must prefer incompleteness to false totality.

### 9.2 Definition of termination

For this specification, an activation **terminates** when it reaches a finite terminal completion:

- normal return;
- exception escape/raise;
- explicit process-termination operation if modeled as a terminal completion.

An activation does not terminate if it:

- diverges forever;
- remains permanently suspended;
- enters an infinite blocking state under the semantic model.

This definition means a method that always raises may still be `@total`.

That is deliberate and follows the ratified meaning “termination proven only.”

### 9.3 `@total`

`@total` is a declaration requirement:

```phalcom
@total
method(...) {
  ...
}
```

The declaration is accepted only if:

```text
callable_termination(id) == Proven(...)
```

`Blocked` is a diagnostic failure for an explicitly `@total` declaration, even though it is not proof that the function diverges.

Suggested diagnostic distinction:

```text
totality.refuted
  analyzer established a nonterminating behavior

totality.unproven
  analyzer could not prove termination
```

This distinction matters for developer action.

### 9.4 `@total` has no effect on the type

Adding or removing `@total` does not change:

- selector identity;
- callable parameter types;
- return type;
- generic signature;
- overload/dispatch selection;
- runtime class;
- object layout.

It attaches a semantic obligation/product to the `CallableId`.

### 9.5 First termination analyzer

The first implementation should be intentionally conservative.

It should prove at least:

1. straight-line acyclic code whose callees are proven terminating;
2. finite branch composition where every reachable branch terminates;
3. calls to trusted native/source callables declared/proven terminating;
4. bounded loops whose bound is statically established by a supported rule;
5. simple structural recursion where a recognized measure strictly decreases;
6. mutual recursion only when the SCC has a common accepted well-founded measure.

Everything else may remain `Blocked(UnsupportedTerminationPattern)` initially.

### 9.6 CFG-based analysis

Termination analysis consumes compiler-owned control flow, not source-text heuristics.

Conceptually:

```text
CallableId
   -> CFG / control summary
   -> call edges
   -> loop/recursion SCCs
   -> ranking obligations
   -> termination result
```

Acyclic control flow is straightforward.

Cycles require evidence.

A cycle is never accepted merely because:

- it contains a `return` somewhere;
- no obvious infinite loop was found;
- tests happened to finish;
- a recursion depth limit exists at runtime.

A runtime recursion limit converts some divergence into runtime failure; it does not make the mathematical program total.

### 9.7 Ranking evidence

A ranking argument proves that a well-founded measure decreases on every back edge/recursive call.

The first recognized measures may include:

- integer value proven nonnegative and strictly decreasing;
- finite collection length strictly decreasing;
- structural subterm recursion where the recursive argument is a proven strict child/substructure.

Do not introduce public measure syntax in the first implementation unless separately ratified.

Compiler-synthesized/recognized measures are enough to establish the architecture.

Future explicit measure syntax can attach to the same obligation model.

### 9.8 Native termination metadata

Current native primitive metadata has no termination field.

Add an explicit native declaration only when the semantic consumer exists. Recommended shape:

```rust
enum TerminationSpec {
    Unknown,
    Terminates,
    MayDiverge,
}
```

`Terminates` is trusted-native input, analogous to authoritative native type/effect metadata. It is not a solver-generated certificate.

A primitive with `TerminationSpec::Unknown` blocks a caller's totality proof unless the call is unreachable or otherwise eliminated by sound analysis.

`ReturnFlowSpec::Never` must not be reused for this purpose.

### 9.9 Termination and effects

These combinations are all valid:

```text
@total + Mutation
@total + Io
@total + Reflection
@total + raises Error
@total + return type Never   // e.g. always raises in finite time
pure + non-total
pure + total
```

No diagnostic should imply otherwise.

### 9.10 Termination and contracts

A proof of a postcondition on normal return is a partial-correctness fact unless totality is separately established.

If callers need total correctness, the proof bundle combines:

```text
partial correctness proof
+
termination proof
```

Do not make every contract proof implicitly require termination.

---

# Part VII — Canonical contracts

## 10. Contract semantics

### 10.1 Preserve existing runtime meaning

Phalcom already has executable:

```text
@requires
@ensures
@invariant
```

with established compile-mode behavior and existing runtime weaving.

This specification does not redefine when those runtime guards execute.

Instead, it creates a canonical semantic representation that both:

- static verification; and
- existing runtime guard generation

can reference by stable identity.

### 10.2 One source contract, two projections

Architecture:

```text
source attribute
      │
      ▼
Canonical ContractDecl / ContractId
      │
      ├──────────────► runtime guard lowering
      │
      └──────────────► proof/VC lowering
```

This prevents runtime contracts and proof contracts from drifting into similar-looking but semantically different systems.

### 10.3 Contract identity

Recommended identity:

```rust
struct ContractId {
    owner: ContractOwner,
    kind: ContractKind,
    index: u16,
}

enum ContractOwner {
    Callable(CallableId),
    Declaration(DeclarationId),
}

enum ContractKind {
    Requires,
    Ensures,
    Invariant,
}
```

`index` is declaration order within the owner/kind.

Source ranges, normalized expression fingerprints, and documentation are metadata associated with the identity, not the identity itself.

### 10.4 Canonical contract record

Conceptual shape:

```rust
struct ContractDecl {
    id: ContractId,
    expression: ContractExprId,
    source: SourceOrigin,
    runtime_policy: RuntimeContractPolicy,
}
```

`ContractExprId` refers to a compiler-owned normalized semantic expression/IR, not an executable closure object.

The runtime compiler may separately compile the same source predicate into a closure for enforcement.

### 10.5 Preconditions

A precondition expresses the admitted input state for the callee.

For static proof:

- proving the callee body may assume its own preconditions;
- proving a call site must establish the callee's preconditions, unless an explicit unchecked/dynamic boundary policy says otherwise;
- assuming a precondition is not equivalent to proving it globally true.

A runtime `@requires` guard remains useful even when static proof is unavailable or when dynamic callers can reach the method.

### 10.6 Postconditions

A postcondition is proven for every **normal return** satisfying the callable's preconditions unless the contract syntax explicitly specifies exceptional outcomes in a future extension.

This matters for a method that may raise:

```text
@ensures P
```

means:

```text
if the call returns normally, P holds
```

not:

```text
the call cannot raise.
```

Exception contracts, if introduced later, are a separate feature.

### 10.7 Invariants

Class invariants are declaration-owned semantic predicates.

The static verifier should use the same boundary policy as the runtime invariant semantics. It must not invent stronger checking points simply because proving them is convenient.

Typical obligations include invariant preservation across operations that are specified by the existing runtime contract model, but the exact entry/exit weave remains owned by the accepted invariant semantics and compiler implementation.

### 10.8 `old(...)`

The existing contract implementation recognizes `old(...)` as a special pre-state capture in postconditions.

Canonical proof lowering represents it explicitly:

```rust
ContractExpr::Old(expr)
```

rather than treating `old` as a normal callable.

Semantics:

- evaluate/reference `expr` in the method-entry state;
- use that pre-state value when checking/proving the postcondition;
- prohibit `old` where existing contract syntax does not permit it;
- preserve runtime and proof meaning exactly.

### 10.9 Result reference

If the existing contract language exposes a result binding/special form, the semantic IR must model it as a normal-return result symbol rather than a source variable guessed by text.

If no such form is currently ratified, this specification does not invent one.

### 10.10 Predicate effect requirements

The formal verifier needs contract predicates to be semantically stable enough to reason about.

Long-term eligibility should be expressed in terms of canonical effect/termination facts, not `is_pure_expr` alone.

Recommended requirement for statically proved contracts:

```text
predicate effects == Known(empty)
predicate termination == Proven
```

plus any additional restrictions imposed by the proof logic.

This does **not** require immediately deleting the current syntactic predicate from runtime compilation. During migration:

- existing runtime contract legality continues to use its current conservative rule;
- formal proof eligibility uses the new semantic facts when available;
- discrepancies are diagnostics/unsupported-proof states, not silent semantic changes.

### 10.11 Reading state is not mutation

A contract may read receiver fields or immutable/global facts if the proof model supports them.

Purity means no modeled externally visible effects, not “the expression contains no field access.”

### 10.12 Contract failures versus proof failures

Keep distinct:

```text
runtime contract violation
```

and:

```text
static proof disproved / unproven
```

The former is an execution-time error for one execution.

The latter is an analysis result about all executions in the modeled domain.

---

# Part VIII — Verification conditions and proof architecture

## 11. Proof goal model

### 11.1 Proof obligations

A `ProofObligation` is a normalized, snapshot-scoped logical claim generated from canonical semantics.

Initial obligation kinds:

```rust
enum ProofObligationKind {
    CallPrecondition,
    CallablePostcondition,
    InvariantPreservation,
    Assertion,
    TerminationMeasure,
}
```

Termination may use a specialized checker rather than the general proof backend in version 1; the obligation identity still allows future convergence.

### 11.2 Verification-condition identity

```rust
struct VerificationConditionId {
    owner: ProofOwner,
    ordinal: u32,
}
```

The stable artifact identity additionally uses a structural fingerprint of the normalized VC and all semantic assumptions.

Numeric IDs are not cross-run proof identity.

### 11.3 Proof subject

```rust
enum ProofOwner {
    Callable(CallableId),
    Contract(ContractId),
    Declaration(DeclarationId),
}
```

Each obligation records source provenance for diagnostics, but source ranges do not affect logical equivalence unless the contract expression itself changes.

## 12. Verification-condition generation

### 12.1 Compiler-owned IR

VC generation consumes a stable semantic control-flow representation produced by the compiler/semantic layer.

It does not parse source again.

Conceptually:

```text
AST
 -> semantic binding/type resolution
 -> canonical callable signature
 -> CFG / semantic operation IR
 -> contract lowering
 -> VC IR
```

The exact CFG/SSA data structure may evolve. The invariant is that proof generation uses resolved identities and canonical operations, not textual variable names.

### 12.2 Initial proof logic subset

The first proof implementation should support a deliberately small, testable logic before object heaps and concurrency.

Good initial operations:

- booleans;
- integers with explicitly modeled arithmetic semantics;
- equality and ordered comparisons;
- conjunction/disjunction/negation;
- local immutable bindings;
- branches;
- normal returns;
- simple pre/postconditions;
- calls whose summaries/contracts are already available;
- finite tuples/records only where the proof model has explicit immutable semantics.

Unsupported operations produce:

```text
ProofResult::Unknown(UnsupportedOperation(...))
```

not a weakened formula.

### 12.3 Numeric semantics must match Phalcom

Proof arithmetic must model Phalcom's actual numeric behavior, including distinctions among integer and floating-point operations.

Do not silently model arbitrary-precision integers as machine integers or floats as mathematical reals.

If a backend theory cannot soundly model an operation, block the proof.

Floating-point proof semantics remain gated until an explicit IEEE/Phalcom model is ratified.

### 12.4 Heap/object state

General field mutation requires a heap model and frame conditions.

Do not approximate:

```text
obj._x = value
```

as assignment to a local logical variable if aliases can observe the same object.

A future heap model should distinguish:

- object identity;
- field selection;
- field update;
- pre-state (`old`);
- aliasing;
- write sets/frame conditions.

Until that model is available, obligations requiring unsupported alias-sensitive heap reasoning are `Unknown`.

### 12.5 Dynamic dispatch

VC generation may inline/use a callee contract only when the semantic dispatch target set is soundly known under the pinned snapshot/model.

Dynamic/open reflection boundaries such as:

- `perform`;
- DNU-dependent calls;
- runtime-added/replaced methods;
- opaque FFI;
- unknown native behavior

block proof unless a separately trusted contract/model exists.

### 12.6 Sealed/closed knowledge

Closed or sealed semantic facts may improve proof precision, but proof generation must distinguish source/declaration closure from runtime mutation authority.

A proof about a compiled snapshot remains valid as a static artifact under its recorded assumptions. Applying it as a claim about a live VM after reflective method mutation requires a compatible world/version assumption.

### 12.7 Exceptional control flow

Postcondition VCs are generated on normal exits.

Raise paths are modeled separately and need not establish ordinary postconditions unless a future exceptional-postcondition syntax says so.

Preconditions still hold as assumptions on all admitted entries.

### 12.8 Partial correctness by default

A postcondition VC proves:

```text
if execution reaches a normal return, the postcondition holds
```

unless paired with termination evidence.

This prevents an infinite loop from vacuously becoming a total-correctness proof.

### 12.9 Total correctness composition

Total correctness is established only by combining:

```text
partial correctness proof
+
TerminationKnowledge::Proven
```

No backend shortcut may infer totality from a postcondition proof alone.

---

## 13. Proof result algebra

### 13.1 Runtime/public variant family

Use native sealed variants, consistent with the result philosophy of Specs 01.5 and 03:

```text
ProofResult
├ Proven(evidence)
├ Disproven(counterexample)
├ Unknown(reason)
├ Cancelled
├ BudgetExceeded(report)
└ InternalFailure(id)
```

This is the public semantic outcome model.

The internal Rust implementation should use compact enums and allocate runtime variant objects only when reflection explicitly requests them.

### 13.2 `Proven`

`Proven` means the active trust policy has accepted evidence that establishes the exact normalized obligation under the recorded assumptions and semantic model.

It does not mean:

- backend process exited successfully;
- no counterexample was found before timeout;
- tests passed;
- runtime guard happened to pass;
- proof metadata existed;
- a cache key approximately matched.

### 13.3 `Disproven`

`Disproven` requires a model/counterexample that falsifies the obligation under the modeled semantics.

Where practical, validate/replay the counterexample through a small deterministic VC evaluator before publishing `Disproven`.

If a backend returns a candidate model that cannot be validated because the operation/model is unsupported, return `Unknown(UnvalidatedCounterexample)` rather than a false refutation.

### 13.4 `Unknown`

Representative reasons:

```rust
enum ProofUnknownReason {
    UnsupportedOperation,
    OpaqueNative,
    DynamicDispatch,
    ReflectiveMutationBoundary,
    MissingEffectSummary,
    MissingTerminationFact,
    UnsupportedHeapReasoning,
    UnsupportedFloatingPointTheory,
    UntrustedBackendVerdict,
    UnvalidatedCounterexample,
    MissingDependencyArtifact,
    SemanticModelMismatch,
}
```

Do not use `Unknown` for cancellation or budget exhaustion.

### 13.5 Cancellation and budget

These are observable terminal states because callers may retry with:

- more time;
- a different backend;
- a larger budget;
- a different interactive priority.

Collapsing them into `Unknown` destroys useful scheduling semantics.

### 13.6 Internal failure

Internal failures carry an incident/error identity suitable for logs without exposing arbitrary backend process output as a stable language API.

---

## 14. Proof trust model

### 14.1 Trust is explicit

A proof artifact records how its evidence became trusted.

Recommended initial tiers:

```rust
enum ProofTrust {
    KernelChecked,
    TrustedBackend,
}
```

Additional imported/signed tiers may be added later with explicit semantics.

### 14.2 Kernel-checked evidence

`KernelChecked` means a local, versioned checker validated a proof certificate against the exact normalized obligation and assumptions.

The kernel should be materially smaller and easier to audit than the producing prover.

No certificate format is ratified by this document; this is an architectural slot.

### 14.3 Trusted backend

`TrustedBackend` means policy explicitly places the backend implementation/version in the trusted computing base for that proof.

This can enable useful early prover integration before a certificate kernel exists, but the trust tier must remain visible in artifacts/reflection/tooling.

### 14.4 Default policy is a product decision

This document does not mandate whether ordinary `phalcom verify` initially accepts `TrustedBackend` proofs by default.

It mandates only:

- the policy is explicit;
- the result records trust;
- a backend cannot silently promote itself to `KernelChecked`;
- changing policy does not mutate cached evidence identity.

### 14.5 Assumptions are first-class

Proof evidence records the assumptions used, including as applicable:

- callable preconditions;
- class invariants;
- callee contracts;
- native semantic declarations;
- sealed/closed-world assumptions;
- effect summaries;
- termination facts;
- semantic model version;
- arithmetic model;
- runtime-world assumptions when relevant.

A proof with stronger/different assumptions is not interchangeable with another merely because both display `Proven`.

---

# Part IX — Proof artifacts, metadata, and runtime projection

## 15. Persistent proof artifacts

### 15.1 Artifact identity

A proof artifact is reusable only when every semantically relevant input matches.

Recommended fingerprint inputs:

```text
proof semantic model version
normalized VC fingerprint
owner declaration/callable fingerprint
canonical generic signature fingerprint
contract fingerprints
callee contract/signature fingerprints
relevant effect summaries
relevant termination summaries
native semantic metadata fingerprints
arithmetic/heap theory version
proof backend identity/version
certificate/kernel version when present
assumption set fingerprint
build/package dependency fingerprints
```

Source whitespace, raw `TypeId` numeric values, heap `ObjRef`s, and nondeterministic hash-map iteration order must not affect the fingerprint.

### 15.2 Artifact structure

Conceptual payload:

```rust
struct ProofArtifact {
    id: ProofArtifactId,
    owner: ProofOwnerRef,
    vc_fingerprint: SemanticFingerprint,
    semantic_model: ProofSemanticModelVersion,
    assumptions: Box<[ProofAssumptionRef]>,
    result: PersistedProofResult,
    evidence: Option<ProofEvidenceBlob>,
    trust: Option<ProofTrust>,
    backend: BackendIdentity,
    dependency_fingerprint: SemanticFingerprint,
}
```

The durable schema is implemented as a Spec 02 advanced extension section. This Rust-like structure is normative in information content, not binary layout.

### 15.3 Cache key exactness

A cached `Proven` result is reusable only on an exact compatible key.

Changes that must invalidate as applicable:

- body semantics;
- generic signature;
- constraint semantics;
- contract expression;
- callee contract/signature;
- native effect/raises/termination metadata;
- relevant class invariant;
- proof model version;
- solver/backend trust policy if the policy changes whether evidence is acceptable;
- dependency package version/fingerprint;
- mutable-world assumption used by the proof.

Changing documentation alone should not invalidate logical evidence unless documentation participates in generated contracts.

### 15.4 Stale artifacts

A stale proof artifact remains historical data but is not a current proof.

The loader/query layer returns an explicit stale/mismatch reason rather than presenting it as `Unknown` semantic logic or silently discarding provenance.

### 15.5 Hostile artifact handling

Proof extension decoding is untrusted input.

Enforce:

- byte limits;
- node-count limits;
- nesting/depth limits;
- certificate-size limits;
- string limits;
- dependency count limits;
- deterministic duplicate rejection;
- unknown-version handling;
- no arbitrary code execution during decode.

Parsing a proof artifact never establishes trust by itself.

---

## 16. Spec 02 metadata integration

Spec 02 deliberately left advanced effect/proof payloads as versioned extension carriage. This document now defines their semantic content.

Recommended extension sections:

```text
advanced.effects.v1
advanced.exits.v1
advanced.termination.v1
advanced.contracts.v1
advanced.proofs.v1
advanced.record_rows.v1     // if rows are exported outside core type graph
```

The physical names/version encoding are implementation details, but the sections must remain independently versionable.

### 16.1 Effect metadata

Per exported callable where retained:

```text
Known(set-of-atoms)
Opaque(reason)
```

Do not serialize solver-local effect variables.

### 16.2 Exit metadata

Store enough to distinguish:

- may return normally;
- known/opaque raise summary;
- divergence knowledge;
- suspension/process-exit flags if modeled.

### 16.3 Termination metadata

Store:

```text
requirement: none | total
result: proven | refuted | blocked
provenance/evidence reference when retained
```

Do not infer `result` from return type `Never` on load.

### 16.4 Contract metadata

Store canonical contract identity and normalized semantic expression references as allowed by the active profile.

Runtime executable predicate closures are runtime compiler artifacts, not the durable proof representation.

### 16.5 Proof profile

Spec 02's `Proof` metadata profile may retain:

- normalized obligation IDs/fingerprints;
- proof results;
- assumptions;
- trust tier;
- backend identity;
- certificate/evidence payload or external reference according to artifact policy;
- source maps needed for diagnostics/counterexamples.

`RuntimeMinimal` must not carry proof payloads unless execution explicitly requires some proof-derived runtime feature in a future specification.

### 16.6 Unknown extension handling

A runtime/tool that does not understand an optional proof extension may ignore it for execution if the artifact does not require it.

It must not translate “unknown proof extension” into `Proven` or `pure` or `total`.

---

## 17. Spec 03 reflection integration

Spec 03 intentionally reserved advanced reflection selectors until this document fixed the semantics.

The following projection is now permitted once implementation exists.

### 17.1 Callable effects

Conceptual selectors:

```phalcom
Typing.current.effectsOf(method)
Typing.current.exitsOf(method)
Typing.current.terminationOf(method)
Typing.current.contractsOf(method)
Typing.current.proofsOf(method)
```

Exact method names should remain aligned with the finalized Spec 03 naming conventions. This document defines the semantic payload, not a parallel top-level reflection API.

### 17.2 Result honesty

Reflection exposes the same terminal distinctions:

```text
Known / Proven / Disproven / Blocked / Unknown
Cancelled
BudgetExceeded
Unavailable
InternalFailure
```

as applicable to the domain.

A stripped metadata profile returns an availability result, not empty effects/contracts/proofs.

### 17.3 No eager runtime objects

Effect sets, contracts, and proof artifacts remain compact metadata until explicitly reflected.

Ordinary method lookup does not allocate:

- `EffectDescriptor`;
- `ContractDescriptor`;
- `ProofResult` objects;
- tuples containing those objects.

Indexed/lazy access remains preferred for large collections.

### 17.4 Proof identity is not object identity

A runtime proof descriptor may be garbage-collected and recreated.

Semantic proof identity comes from artifact/obligation fingerprints and trust metadata, not `===` across contexts.

---

# Part X — Incremental analysis and solver execution

## 18. Query ownership

All advanced analyses are compiler-owned `SemanticDb` queries.

Suggested key families:

```rust
EffectQueryKey(CallableId)
ExitQueryKey(CallableId)
TerminationQueryKey(CallableId)
ContractQueryKey(ContractOwner)
RecordRowQueryKey(...)
VerificationQueryKey(ProofOwner, ProofPolicyId)
```

The exact key structs follow Spec 01's identity/stamp architecture.

The LSP and CLI do not own private caches that can disagree with compiler facts.

## 19. Dependency tracking

### 19.1 Effects

Effect query dependencies include:

- callable body semantic fingerprint;
- selected callee IDs/summaries;
- native effect metadata;
- dynamic/open-world boundary facts.

### 19.2 Exits

Exit query dependencies include:

- CFG/control facts;
- callee exit summaries;
- native raises/flow metadata;
- exception handling edges.

### 19.3 Termination

Termination dependencies include:

- CFG cycles;
- recursive call SCC;
- callee termination facts;
- recognized ranking-measure facts;
- native termination metadata;
- relevant preconditions if they constrain the termination domain.

### 19.4 Proofs

Proof dependencies include everything in the artifact key, but invalidation should still operate through semantic dependency edges rather than globally flushing all proofs.

## 20. Cancellation

Cancellation checks must occur at least at:

- SCC round boundaries;
- row-solver worklist batches;
- CFG block traversal;
- call-edge expansion;
- termination ranking search;
- VC block/term generation;
- backend request/response boundaries;
- certificate checking batches;
- artifact decoding batches;
- immediately before publication.

Cancelled work publishes no partial semantic answer.

## 21. Budgets

Separate budget classes are appropriate because workloads differ.

Examples:

```rust
struct EffectBudget {
    max_call_edges: u32,
    max_scc_rounds: u16,
}

struct RowBudget {
    max_constraints: u32,
    max_fields: u32,
    max_substitutions: u32,
}

struct TerminationBudget {
    max_cfg_nodes: u32,
    max_call_edges: u32,
    max_ranking_candidates: u32,
}

struct ProofBudget {
    max_ir_nodes: u32,
    max_vc_terms: u32,
    max_backend_time_ms: u64,
    max_backend_memory_bytes: u64,
}
```

The concrete defaults are performance-policy decisions and must be benchmarked.

A larger budget may enable a retry; therefore budget exhaustion must stay observable.

## 22. Determinism

For identical semantic inputs and policy:

- effect atom order is stable;
- row field order is stable;
- SCC publication order is stable;
- VC numbering is stable;
- normalized VC fingerprints are stable;
- diagnostic ordering is stable;
- artifact fingerprints are stable;
- proof backend nondeterminism does not leak into semantic identity.

Backend-generated auxiliary text may be nondeterministic, but it cannot participate in canonical fingerprints unless normalized by a ratified scheme.

---

# Part XI — Diagnostics and developer experience

## 23. Diagnostic taxonomy

Advanced diagnostics must explain *why* reasoning failed rather than collapsing everything to “type error.”

### 23.1 Rows

Suggested codes:

```text
row.duplicate_field
row.tail_kind_mismatch
row.closed_missing_field
row.closed_extra_field
row.occurs_check
row.constraint_rejected
row.blocked
row.budget_exceeded
```

### 23.2 Effects

```text
effect.expected_pure
effect.unknown_native
effect.dynamic_boundary
effect.reflective_boundary
effect.inference_blocked
effect.budget_exceeded
```

Where a declaration explicitly requires purity in a future syntax, diagnostics show the effect path:

```text
method `parseConfig` is not proven pure
  -> calls `readFile(_)`
  -> native effect: Io
```

### 23.3 Totality

```text
totality.refuted
totality.unproven
totality.opaque_native
totality.dynamic_call
totality.recursive_cycle
totality.measure_not_proven
totality.budget_exceeded
```

For `@total`, the diagnostic should distinguish a discovered nonterminating cycle from inability to prove a complex loop.

### 23.4 Contracts

```text
contract.invalid_context
contract.effectful_predicate
contract.nonterminating_predicate
contract.invalid_old
contract.unsupported_proof_operation
contract.precondition_unproven
contract.postcondition_disproved
contract.invariant_disproved
```

Existing runtime contract syntax/legality diagnostics remain intact unless a dedicated migration updates them.

### 23.5 Proofs

```text
proof.disproved
proof.unknown
proof.untrusted_backend
proof.semantic_model_mismatch
proof.stale_artifact
proof.counterexample_unvalidated
proof.cancelled
proof.budget_exceeded
proof.internal_failure
```

### 23.6 Causal evidence paths

Diagnostics should preserve bounded cause paths such as:

```text
@total requirement on `f`
  -> recursive call `g(n - 1)`
  -> `g` calls `f(n)`
  -> no recognized decreasing SCC measure
```

or:

```text
postcondition could not be proved
  -> call to `plugin.perform(selector)`
  -> runtime selector target is open
  -> no trusted contract for reflective boundary
```

Paths must be cycle-checked and bounded.

---

## 24. CLI behavior

The formal semantic engine is shared with compiler/LSP/REPL.

### 24.1 Ordinary checking

Ordinary `phalcom check` should always enforce semantic requirements that are part of the source program, including eventually:

- `@total` declarations;
- statically mandatory contract well-formedness;
- row/type correctness.

It should not automatically invoke an expensive general theorem prover for every method merely because contracts exist unless product policy explicitly enables that mode.

### 24.2 Verification mode

A dedicated verification mode/subcommand/flag may request static proof of contracts and emit proof artifacts.

Exact CLI spelling is not fixed here. The semantic contract is:

- same `SemanticDb` snapshot;
- same diagnostic codes;
- explicit trust policy;
- explicit budgets;
- machine-readable result option;
- no result downgrade/upgrade based on presentation mode.

### 24.3 Exit codes

CLI policy may distinguish:

- semantic invalidity;
- disproved required proof;
- unproven required proof;
- infrastructure/backend failure;
- cancellation/budget.

Do not map all five to a single internal “type failure” state before the presentation layer has a chance to explain them.

---

## 25. LSP behavior

The LSP consumes published formal facts.

Potential features after implementation:

- hover: effect summary;
- hover: known raises/exits;
- hover: `@total` status and evidence reason;
- hover: contracts;
- hover: proof status/trust tier;
- diagnostics with causal effect/termination/proof paths;
- code lenses such as “verified”, “unproven”, “disproved” when useful;
- go-to source for contract/counterexample origin;
- proof counterexample value display;
- inferred row-tail information.

Interactive policy should use smaller budgets and cancellation than batch verification but preserve result semantics.

A cancelled LSP proof request must not leave a false “verified” cache entry.

---

## 26. REPL behavior

The persistent REPL architecture from Spec 01/01.5 should treat each cell's advanced facts as snapshot/generation products.

Rules:

- a new cell invalidates only dependent effect/termination/proof queries;
- earlier pinned `TypingContext`s continue to refer to their older metadata generation;
- proof results may be reused only if fingerprints match;
- runtime method mutation after evaluation does not mutate historical proof artifacts;
- a live reflective query that wants applicability to the current VM world must include the relevant world version/policy.

The REPL should not run expensive proof search on every entered expression by default.

---

# Part XII — Runtime and object-model boundaries

## 27. Runtime contracts versus static proof

The three valid configurations are independent:

```text
runtime guards ON  + static proof absent
runtime guards OFF + static proof present
runtime guards ON  + static proof present
```

and, in an unchecked build:

```text
runtime guards OFF + static proof absent
```

is also technically possible according to existing compile policy, though product policy may warn.

Static proof must not depend on whether guard closures were retained in the VM artifact.

## 28. Runtime mutation and proof applicability

Phalcom permits reflective method operations sufficiently powerful that live runtime behavior can diverge from a compiled semantic snapshot.

Therefore distinguish:

```text
proof validity for the pinned compiled semantic artifact
```

from:

```text
proof applicability to the current mutated VM world
```

A proof artifact remains logically about its fingerprinted program.

A runtime API claiming that proof applies to a live method after method replacement must verify the relevant world/semantic binding assumptions.

This uses the world-version/runtime binding seams from Specs 02–03; it does not invalidate structural type equivalence itself.

## 29. No proof-guided dispatch

Proof state never decides which method is called.

No selector key contains:

- effect set;
- proof status;
- `@total`;
- contract identity;
- generic type argument.

Optimizers may use proven facts internally if they preserve observable semantics and deopt/invalidate correctly, but that is an optimization specification, not dispatch semantics.

## 30. No proof tokens on ordinary values

Do not attach:

- proof IDs;
- generic validation tokens;
- invariant-certification bits;
- effect capabilities

to every ordinary object merely to accelerate reflection/proving.

Explicit validated wrappers or boundary-specific witnesses may be added by future designs if they solve a concrete problem.

---

# Part XIII — Exact implementation map

## 31. `phalcom-semantic`

The exact file layout may adapt to Spec 01's in-flight SemanticDb implementation. The ownership boundaries below are normative even if filenames change during that work.

### 31.1 Existing files to extend

#### `phalcom-semantic/src/types/kind.rs`

After row enablement:

- add canonical `RecordRow` atomic kind;
- preserve `Type` and arrow-kind semantics from 01.5;
- do **not** add kind variables/schemes until the kind-polymorphism gate is separately approved.

#### `phalcom-semantic/src/types/store.rs`

- evolve closed structural record storage to refer to canonical row data or an equivalent typed row representation;
- preserve proper-type validation;
- do not place solver-local row variables in canonical `TypeData`;
- keep row-related storage compatible with store epochs/snapshot ownership from Spec 01/01.5.

#### `phalcom-semantic/src/types/annotation.rs`

Once Spec 04 S7 is enabled:

- lower `RecordRow` kind references;
- lower open record tails;
- reject non-row binders in tail position with dedicated diagnostics;
- preserve invalid children for parser/diagnostic recovery without publishing invalid semantic forms.

Do not implement effect/termination/proof syntax here.

#### `phalcom-semantic/src/export.rs`

- export canonical record-row structures when publishable;
- export advanced summaries only through Spec 02 extension adapters, not by growing `CompiledTypeRef` into a universal semantic dump;
- reject solver-local row variables.

#### `phalcom-semantic/src/snapshot.rs`

Spec 01 may substantially change this file. After the SemanticDb substrate stabilizes, published snapshots need query access/indices for advanced facts without forcing eager computation of every fact.

Do not simply add giant eager maps for all proofs/effects if query cells can remain lazily published and snapshot-addressable.

### 31.2 New row modules

Recommended:

```text
phalcom-semantic/src/types/row.rs
phalcom-semantic/src/types/row_solver.rs
```

Responsibilities:

- `RecordRowId`/data;
- canonical sorting/interner;
- row-term query-local representation;
- row substitution;
- row equality;
- lacks constraints;
- occurs checks;
- relation adapters under explicit `RecordAccess`;
- row budgets and diagnostics.

### 31.3 New effect modules

Recommended:

```text
phalcom-semantic/src/effects/mod.rs
phalcom-semantic/src/effects/atom.rs
phalcom-semantic/src/effects/summary.rs
phalcom-semantic/src/effects/infer.rs
```

or a flatter equivalent if the implementation remains small.

Responsibilities:

- canonical `EffectAtom`/`EffectSetId`;
- known/opaque state;
- native adaptation;
- body/direct effect extraction;
- SCC inference;
- provenance paths;
- budgets/cancellation;
- query integration.

### 31.4 New exit/control module

Recommended:

```text
phalcom-semantic/src/control_summary.rs
```

Responsibilities:

- `ExitSummary`;
- raise knowledge;
- divergence knowledge;
- native return-flow adaptation;
- CFG composition rules;
- call summary propagation.

Keep this separate from effect atoms.

### 31.5 New termination modules

Recommended:

```text
phalcom-semantic/src/termination/mod.rs
phalcom-semantic/src/termination/cfg.rs
phalcom-semantic/src/termination/ranking.rs
```

Responsibilities:

- termination result algebra;
- CFG cycle analysis;
- call-SCC analysis;
- recognized ranking measures;
- `@total` requirement checking;
- provenance/counterevidence;
- budgets/cancellation.

### 31.6 New contract semantic module

Recommended:

```text
phalcom-semantic/src/contracts/mod.rs
phalcom-semantic/src/contracts/ir.rs
phalcom-semantic/src/contracts/lower.rs
```

Responsibilities:

- stable `ContractId`;
- contract owner/kind;
- normalized semantic predicate IR;
- `old` representation;
- source provenance;
- proof eligibility facts;
- bridge records for runtime lowering.

This module must not allocate runtime closures.

### 31.7 New proof modules/crate boundary

Start inside `phalcom-semantic` only if dependencies remain small:

```text
phalcom-semantic/src/proof/mod.rs
phalcom-semantic/src/proof/ir.rs
phalcom-semantic/src/proof/vc.rs
phalcom-semantic/src/proof/result.rs
phalcom-semantic/src/proof/fingerprint.rs
```

Before integrating an external backend/process/certificate system, strongly consider a separate crate with an ADR, e.g.:

```text
phalcom-proof
```

Dependency rule:

```text
phalcom-semantic
   -> backend-neutral obligation structures/interfaces only

phalcom-proof
   -> backend protocol, process management, certificate checking, persistent evidence
```

Do not make the core type checker depend on a heavy solver SDK.

---

## 32. `phalcom-native-meta`

### `phalcom-native-meta/src/primitive.rs`

Preserve the current:

```text
RaisesSpec
NativeEffect
EffectSpec
ReturnFlowSpec
```

and adapt them into canonical semantic facts.

Add termination metadata only when the termination consumer exists:

```rust
enum TerminationSpec {
    Unknown,
    Terminates,
    MayDiverge,
}
```

Then add a corresponding `termination` field to `PrimitiveSurfaceSpec` with an explicit migration of every native primitive.

Do not default missing legacy entries to `Terminates`.

The migration must force each primitive to make an explicit decision or remain `Unknown`.

### Native effect evolution

If effect granularity grows later, version/extend the native vocabulary explicitly. Do not reinterpret an old `Mutation` record as a new receiver-only region silently.

---

## 33. `phalcom-native-macros`

`phalcom-native-macros/src/lib.rs` already generates the primitive surface metadata consumed above.

When termination metadata lands:

- parse explicit native termination declaration syntax;
- emit `TerminationSpec`;
- produce compile errors for contradictory/unknown spellings;
- preserve deterministic generated metadata;
- do not infer termination from `flow = never`;
- do not infer purity from the absence of an `effects` argument.

Tests must include deliberately contradictory examples.

---

## 34. `phalcom-core` compiler contract bridge

### 34.1 `phalcom-core/src/compiler/attributes.rs`

Preserve the accepted runtime guard weave and compile-mode matrix.

Migration goal:

1. assign stable contract identities before or during attribute expansion;
2. retain enough source/canonical predicate information for semantic lowering;
3. keep runtime closure generation as a separate projection;
4. stop treating `is_pure_expr` as formal proof of purity once semantic effect facts are available.

Do not delete `is_pure_expr` prematurely if current runtime contract legality depends on it.

Instead, give it an explicit transitional role and a deletion/replacement criterion.

### 34.2 `phalcom-core/src/method/object.rs`

Do **not** add:

- static `EffectSetId`;
- `TerminationEvidence`;
- `ProofArtifactId`;
- full canonical contract IR.

Continue storing only runtime-relevant contract closures/attributes as required by current semantics.

Runtime static metadata access goes through the VM semantic side tables from Specs 02–03.

### 34.3 Compiler analyzed program/artifacts

Artifact compilation exports advanced metadata from the analyzed semantic snapshot, not from re-parsing attributes after type checking.

The ordinary code generator should not invoke a prover unless the selected build/verification policy requests proof products.

---

## 35. Metadata crate / Spec 02 implementation

Once Spec 02's schema crate exists, define separate advanced extension structs there.

Requirements:

- no raw `TypeId`, `EffectSetId`, `ContractId`, etc. cross artifact boundaries;
- use indexed schema-local IDs/fingerprints;
- independent extension versions;
- hostile-input validation;
- deterministic ordering;
- proof trust/version fields mandatory where proof results are present;
- unknown sections cannot imply success.

The core metadata schema should not be rewritten every time proof logic grows.

---

## 36. `phalcom-core` runtime reflection

Runtime classes/descriptors for advanced facts land only as demanded by Spec 03.

Likely implementation areas:

```text
phalcom-core/src/heap/reflection/...
phalcom-core/src/primitive/typing/...
phalcom-core/core/universe/src/reflection/typing/...
```

Exact layout follows Spec 03 C9 and the repository's universe package conventions.

Rules:

- lazy materialization;
- weak cache semantics from Spec 02;
- no new `Value` tag;
- no static facts copied into every `MethodObject`;
- capability/profile checks before private/proof data exposure.

---

## 37. LSP and CLI

### LSP

Likely consumers:

```text
phalcom-lsp/src/semantic/...
hover/signature/diagnostic presentation modules
```

The LSP asks formal semantic queries; it does not re-infer effects or proof validity from AST text.

### CLI

`phalcom-core/bin/phalcom/cli.rs` or its post-Spec-01 successor should:

- present shared formal diagnostics;
- configure verification/trust/budget policy;
- never reinterpret a backend timeout as a type failure or proof success;
- offer machine-readable proof summaries when the CLI contract later defines them.

---

# Part XIV — Implementation workstreams

## 38. Workstream A — Ownership cleanup and readiness gate

Before implementing advanced features:

1. Spec 01 semantic result/cancellation/budget infrastructure is stable enough to consume.
2. Spec 01.5 canonical callable IDs/signatures and generic environments are stable.
3. revised Spec 04 record-row AST/lowering shapes are available or implementation remains behind parser gate.
4. revised Spec 02 extension carriage API is either implemented or advanced products remain snapshot-only.

Tasks:

- remove old Spec 05 assumptions from implementation plans;
- ensure no parameter-owned bounds/default fields are added because the old document mentioned them;
- ensure no finite-set generic constraint is implemented;
- add domain-specific advanced query result enums using the common outcome philosophy.

**Gate A:** no advanced code chooses semantics already owned by 01.5.

---

## 39. Workstream B — Record rows

Implement semantic rows before enabling public row syntax.

Order:

1. `RecordRow` kind;
2. canonical row store/data;
3. open-tail binder lowering;
4. row solver terms;
5. equality/substitution;
6. lacks constraints;
7. occurs checks;
8. read-only relation integration;
9. writable-policy conservative integration;
10. metadata round trip;
11. reflection projection if requested;
12. enable Spec 04 row parser/public feature.

Tests first:

- source field permutation canonicalization;
- duplicate rejection;
- tail-kind mismatch;
- row subtraction/unification;
- indirect occurs cycle;
- lacks propagation;
- clean/incremental equivalence;
- cancellation/budget;
- no solver row var in exported metadata.

**Gate B:** only after all tests may `#{ ..., | R }` become an enabled public semantic feature.

---

## 40. Workstream C — Effects

Order:

1. canonical effect atoms/sets;
2. `EffectKnowledge` and explicit opacity reasons;
3. native metadata adapter;
4. direct source-expression effect extraction;
5. callable call-graph dependencies;
6. SCC fixed-point inference;
7. diagnostics/provenance;
8. snapshot/query publication;
9. metadata extension;
10. LSP/CLI presentation.

Tests first:

- pure literal function;
- object mutation;
- I/O native call;
- call-chain propagation;
- recursive SCC stabilization;
- opaque native call;
- reflective/dynamic call;
- cancellation;
- budget;
- incremental body edit invalidation;
- `Unknown` never equal to empty set.

**Deletion criterion:** no formal purity decision depends solely on `compiler::attributes::is_pure_expr`.

---

## 41. Workstream D — Exit summaries

Order:

1. canonical raise-set/opacity representation;
2. `ExitSummary`;
3. native `RaisesSpec`/`ReturnFlowSpec` adapter;
4. source CFG return/raise composition;
5. exception-handler edges;
6. call propagation;
7. suspension/process-exit hooks as semantics require;
8. diagnostics/metadata.

Tests first:

- always normal return;
- unconditional raise with `Never` normal type;
- mixed return/raise branches;
- native `flow = receiver` alias refinement;
- native `flow = never` does not imply divergence;
- opaque raise metadata;
- handled exception does not escape;
- recursive propagation determinism.

---

## 42. Workstream E — Termination and `@total`

Order:

1. termination result/evidence taxonomy;
2. source `@total` semantic requirement attachment;
3. acyclic CFG proving;
4. callee termination dependencies;
5. loop SCC discovery;
6. first ranking measures;
7. recursive-call SCC reasoning;
8. native `TerminationSpec` migration;
9. diagnostics;
10. metadata/reflection projection.

Tests first:

- straight-line total method;
- total method that raises;
- total method with mutation/I/O to prove independence from effects;
- obvious infinite loop;
- simple decreasing integer recursion;
- unrecognized complex recursion => unproven, not refuted/proven;
- native `Unknown` blocks totality;
- `ReturnFlowSpec::Never` alone never proves totality;
- budget/cancel never produce `Proven`.

**Gate E:** no public `@total` acceptance until false-positive-focused termination tests and code review pass.

---

## 43. Workstream F — Canonical contracts

Order:

1. stable `ContractId`;
2. source contract extraction before runtime-only erasure;
3. normalized contract semantic IR;
4. explicit `old` node;
5. semantic effect/termination eligibility checks;
6. bridge existing runtime weave to same contract identity;
7. metadata extension;
8. tooling/reflection projections.

Tests first:

- declaration order identity;
- requires/ensures/invariant distinction;
- `old` pre-state mapping;
- runtime guard behavior unchanged in Debug/Release/Unchecked;
- static contract identity unchanged by runtime metadata stripping;
- effectful predicate blocks proof;
- nonterminating predicate blocks proof;
- no runtime closure ObjRef enters semantic contract IR.

**Deletion criterion:** runtime and proof paths cannot assign unrelated identities to the same source contract.

---

## 44. Workstream G — VC IR and deterministic generator

Start with a backend-free implementation.

Order:

1. proof owner/obligation IDs;
2. proof IR types;
3. deterministic CFG-to-VC lowering;
4. scalar boolean/integer logic;
5. precondition assumptions;
6. call-site precondition obligations;
7. postcondition normal-exit obligations;
8. invariant hooks;
9. source mapping;
10. structural fingerprints;
11. unsupported-operation reasons.

Tests first:

- straight-line postcondition;
- branch merge;
- raise path excluded from normal postcondition;
- precondition use;
- callee precondition obligation;
- deterministic VC across hash-map/order noise;
- unsupported heap op => `Unknown` path, not weakened formula;
- fingerprint changes exactly when logical input changes.

**Gate G:** no external prover before the VC generator has deterministic golden/property tests.

---

## 45. Workstream H — Backend, trust, and artifacts

Requires a dedicated threat-model/design review.

Order:

1. backend-neutral request/response protocol;
2. process/resource limits;
3. backend identity/version;
4. raw verdict taxonomy distinct from public proof result;
5. counterexample validation/replay where supported;
6. trust-policy adapter;
7. `ProofResult` sealed variants;
8. artifact fingerprint/cache;
9. Spec 02 proof extension;
10. optional certificate checker/kernel.

Tests first:

- backend crash;
- timeout;
- malformed response;
- contradictory backend result;
- untrusted proof verdict cannot become `Proven` under strict policy;
- corrupted certificate rejected;
- stale artifact rejected;
- dependency change invalidates artifact;
- cancellation kills/abandons backend safely;
- counterexample replay failure becomes `Unknown`;
- deterministic artifact keys.

**Gate H:** no backend result is `KernelChecked` without local certificate checking.

---

## 46. Workstream I — Ecosystem integration

### Compiler

- demand advanced queries only for explicit requirements/build mode;
- export retained summaries;
- preserve runtime semantics.

### CLI

- present formal statuses and trust;
- configure budgets/policy;
- support batch verification workflow.

### LSP

- cancellable low-budget interactive queries;
- rich status hovers/diagnostics;
- never own semantic truth.

### REPL

- generation-aware invalidation;
- proof reuse only on exact fingerprints;
- no eager proving on every cell.

### Runtime reflection

- explicit capability/profile access;
- lazy descriptor allocation;
- static/live-world applicability distinction.

---

## 47. Workstream J — Optional prenex kind polymorphism

This is intentionally **not** on the critical path.

Only start after a concrete Phalcom library/use case demonstrates that explicit constructor kinds and type lambdas are insufficient.

If started:

1. write language-design decision with motivating examples;
2. ratify source syntax separately;
3. add stable/solver/rigid kind IDs;
4. implement unifier + occurs check;
5. define declaration generalization/subsumption;
6. metadata/reflection extensions;
7. property tests;
8. benchmark generic workloads.

**Gate J:** no `Type :: Type`, no dependent escalation, no higher-rank quantification by accident.

---

# Part XV — Mathematical laws and verification matrix

## 48. Effect laws

For canonical known sets:

```text
E ∪ ∅ = E
E ∪ E = E
E1 ∪ E2 = E2 ∪ E1
(E1 ∪ E2) ∪ E3 = E1 ∪ (E2 ∪ E3)
E ⊆ E
E1 ⊆ E2 and E2 ⊆ E3 => E1 ⊆ E3
```

Epistemic state law:

```text
Opaque(reason) != Known(∅)
```

A join involving an opaque callee must preserve opacity unless a sound analysis policy explicitly provides a closed summary.

## 49. Exit laws

- raise-set union is commutative/idempotent for known sets;
- handled exceptions are removed only when the handler is soundly known to catch them;
- `ReturnFlowSpec::Never` implies `may_return_normally == false` for that primitive, not `divergence == Possible`;
- normal return type `Never` does not determine termination.

## 50. Termination laws

```text
TerminationKnowledge::Proven
```

is the only state that satisfies an explicit `@total` obligation.

Neither:

```text
Blocked
Refuted
Cancelled
BudgetExceeded
InternalFailure
```

satisfies it.

Composition:

- a finite acyclic sequence of proven-terminating operations terminates;
- a branch terminates only if every reachable branch terminates;
- a call blocks a proof when callee termination is not established, unless the call is unreachable by sound reasoning;
- a cycle requires well-founded evidence.

## 51. Contract/proof laws

- preconditions are assumptions inside the callee proof, obligations at proved call sites;
- ordinary postconditions constrain normal returns only;
- runtime guard success is not universal proof;
- partial correctness does not imply termination;
- total correctness requires partial correctness + termination;
- source contract stripping at runtime does not rewrite semantic contract identity.

## 52. Proof-result laws

Only evidence accepted by trust policy yields `Proven`.

A timeout yields `BudgetExceeded`/backend timeout state, never `Unknown(no counterexample)` and never `Proven`.

A backend crash yields `InternalFailure`/backend failure, never `Disproven`.

A model that cannot be validated where validation is required yields `Unknown(UnvalidatedCounterexample)`.

A stale artifact never yields current `Proven`.

## 53. Record-row laws

- canonical field order is permutation-invariant;
- duplicates are invalid;
- row substitution is capture/owner aware;
- a row variable cannot contain itself after solving;
- `extend` requires `Lacks`;
- a row binder of kind `RecordRow` cannot appear as a proper type annotation;
- solver row vars never publish.

## 54. Runtime invariance laws

Adding effects/contracts/proofs to a method changes none of:

```text
selector
runtime class
metaclass
instance layout
method lookup key
ordinary allocation layout
generic instance token state
```

## 55. Incremental laws

For identical source/project inputs and policy:

```text
clean analysis == incremental analysis
```

for:

- effect summaries;
- exit summaries;
- termination results;
- row solutions;
- contract identities;
- normalized VCs;
- proof cache keys;
- diagnostics.

Cancellation of one generation must not publish partial results into a later generation.

---

## 56. Verification matrix

| Invariant | Required evidence |
|---|---|
| 01.5 owns ordinary generics | no duplicate variance/bounds/Self/type-lambda representation added by 05 |
| Row domain is typed | compile-time distinct row IDs/vars plus cross-domain negative tests |
| Row syntax safe | tail-kind, duplicate, lacks, occurs-check tests |
| Effects are not exits | examples with same effects/different raise/divergence outcomes |
| Unknown is not pure | explicit opacity tests |
| `Never` is not termination | always-raise and infinite-loop comparison |
| `@total` means termination only | total effectful/raising examples |
| Runtime contracts are not proof | compile-mode matrix + proof-state independence tests |
| Proof trust explicit | untrusted/corrupt backend evidence cannot become accepted `Proven` |
| Budgets never succeed | terminal-state assertions in row/effect/termination/proof layers |
| Counterexamples honest | replay/validation failure does not become `Disproven` |
| Artifacts exact | every key-component mutation invalidates as specified |
| Incremental equals clean | differential corpus across rows/effects/contracts/proofs |
| Ordinary runtime stays cheap | no descriptor/proof allocation in nonreflective execution benchmarks |
| MethodObject stays lean | no advanced static payload copied into every runtime method |

---

# Part XVI — Performance and security contract

## 57. Performance requirements

### 57.1 Normal runtime

When no typing reflection/runtime validator is invoked, ordinary execution must pay **zero per-call proof/effect lookup cost**.

Specifically, dispatch must not query:

- effect summaries;
- termination status;
- proof status;
- contract proof artifacts.

Existing runtime contract guards remain the only runtime contract cost selected by compile mode.

### 57.2 Semantic queries

Warm common queries should normally be allocation-light:

```text
callable_effects(id)
callable_exits(id)
callable_termination(id)
```

Use compact IDs, immutable arrays/bitsets, SCC caches, and snapshot-owned data.

For six initial effects, a bitset representation is likely superior to heap sets:

```rust
struct EffectBits(u16);
```

provided the semantic API does not expose the physical encoding and version growth is handled explicitly.

### 57.3 Rows

Small record rows are common enough that canonicalization should avoid pathological map allocation.

Benchmark:

- 0–4 fields;
- 5–16 fields;
- open vs closed;
- repeated canonicalization;
- row subtraction;
- relation queries.

Small-vector/sorted-slice representations are preferred until evidence favors hashing.

### 57.4 Proof generation

Proof work is explicitly demand-driven and separately budgeted.

Do not block ordinary LSP hover/type completion on a long backend proof.

Publish partial presentation such as “verification pending/cancelled” only as UI state; never as semantic proof state.

### 57.5 Cache sizing

Proof artifacts may be large.

Use bounded disk/cache policies outside the canonical semantic store. The `TypeStore`/SemanticDb must not become a certificate blob warehouse.

## 58. Security requirements

External prover backends are process/tool boundaries.

Requirements:

- explicit executable selection/trust configuration;
- bounded time and memory where platform permits;
- sanitized temp/artifact paths;
- no shell interpolation of source text;
- bounded stdout/stderr capture;
- protocol parser limits;
- cancellation/kill policy;
- version capture;
- artifact integrity validation;
- no automatic execution of proof payload code.

A proof certificate is data, never an executable plugin.

---

# Part XVII — Migration and deletion ledger

## 59. Delete/supersede these old Spec 05 concepts

| Old concept | Replacement |
|---|---|
| `TypeParameterData.bounds` in Spec 05 | signature-owned 01.5 generic constraints |
| `TypeParameterData.default` reserved eagerly | no defaults until separately designed |
| `TypeBound::FiniteSet` | deferred; not core generic semantics |
| variance owned by 05 | 01.5 |
| ordinary subtype/equivalence constraint IR owned by 05 | 01/01.5 relation + generic-constraint infrastructure |
| `SelfTypeTerm` owned by 05 | 01.5 owner-relative `Self` |
| ordinary generic substitution owned by 05 | 01.5 environments/materialization |
| kind polymorphism required for HKT | explicit 01.5 arrow kinds/type lambdas; kind polymorphism optional |
| one monolithic solver for types/rows/effects/proofs | typed domain solvers sharing infrastructure only |
| proof/effect core metadata frozen in Spec 02 | versioned advanced extensions from revised Spec 02 |
| symbol/status bags for proof results | sealed result variants |

## 60. Transitional implementation that may remain temporarily

- current `EffectSpec`/`RaisesSpec`/`ReturnFlowSpec` native metadata;
- current runtime contract weaving;
- current `is_pure_expr` for runtime contract eligibility;
- current `MethodObject.contracts` closure storage;
- recursive `CompiledTypeRef` bridge where Spec 02 still uses it internally;
- existing relation code until Spec 01/01.5 migration replaces it.

Each is acceptable only in its current role. None may silently become the new formal proof/effect authority.

---

# Part XVIII — Intentional gates

## 61. Do not implement without a separate decision

The following remain gated:

- public kind-variable syntax;
- prenex kind-polymorphism syntax/inference;
- higher-rank kinds;
- `Type :: Type`;
- universe levels;
- dependent types/kinds;
- public effect-row/effect-variable syntax;
- effect handlers;
- region/ownership effect system;
- allocation effect atom;
- exceptional postcondition syntax;
- user-authored termination-measure syntax;
- proof-term programming;
- quantified logic beyond the selected initial verifier subset;
- full alias-sensitive heap separation logic;
- concurrency proof semantics;
- floating-point proof theory;
- a specific SMT/backend as permanent ABI;
- a proof-certificate format;
- package signature/distributed proof trust;
- runtime proof-guided dispatch;
- per-instance proof/type tokens.

Each gate needs:

1. motivating use cases;
2. semantic design;
3. syntax if applicable;
4. performance/trust analysis;
5. metadata/reflection impact;
6. migration plan;
7. focused tests;
8. decision register entry in revised Spec 07.

---

# Part XIX — Cross-spec amendment ledger

## 62. Amend Spec 01.5

No semantic rewrite required.

Clarify only that:

- `RecordRow` becomes an active distinct kind/domain when this Spec 05 row gate lands;
- advanced callable facts attach by `CallableId` and do not enter callable type identity;
- `@total` meaning remains termination-only;
- proof/effect analysis consumes generic specialization views rather than eagerly materializing every generic signature.

## 63. Amend revised Spec 02

Its extension envelope should now consume the payloads defined here:

- effects;
- exits;
- termination;
- contracts;
- proofs;
- trust/artifacts.

Core type metadata remains independent of proof backend schema versions.

## 64. Amend revised Spec 03

C9 may now expose advanced reflection using these exact semantic result families.

Do not restore the older proof status bag.

Reflection must preserve:

- proof trust;
- unknown reasons;
- cancel/budget distinction;
- metadata availability;
- static snapshot versus live-world applicability.

## 65. Amend revised Spec 04

Row syntax can be enabled only after Workstream B.

No effect-row or kind-variable public syntax is implied by this document.

`@total` remains an attribute-level semantic declaration; its exact parser treatment follows the ordinary attribute machinery and accepted attribute syntax rather than a new type grammar production.

## 66. Amend Spec 06

The rationale document must be updated to reflect:

- ordinary HKT/type lambdas are in 01.5, not advanced kind polymorphism;
- `where` constraints supersede parameter-owned bounds;
- finite-set constraints are deferred;
- proof correctness uses explicit trust;
- effects/exits/termination are orthogonal;
- runtime guards are not static proof.

## 67. Amend Spec 07

The consolidated plan must be rebuilt around:

```text
01 infrastructure
  -> 01.5 generic semantic calculus
       -> 04 source syntax/lowering
       -> 02 metadata
       -> 03 reflection
       -> 05 advanced analyses
            -> rows
            -> effects/exits
            -> termination/@total
            -> canonical contracts
            -> VC/proof platform
```

Rows may overlap effect work after the required 01.5/01 infrastructure is stable because their solver domains are separate.

General theorem-prover backend work should be one of the final semantic phases, not a prerequisite for implementing ordinary typing.

---

# Part XX — Acceptance gates

## 68. Row acceptance

All must hold:

- canonical permutation-insensitive record identity;
- tail kind enforcement;
- duplicate rejection;
- lacks constraints;
- occurs checks;
- explicit relation capability;
- bounded/cancellable solver;
- metadata round trip;
- no solver vars publish;
- clean/incremental equivalence.

## 69. Effect acceptance

All must hold:

- current native metadata maps losslessly into canonical atoms/opacity;
- source direct effects inferred;
- calls/SCCs propagate deterministically;
- dynamic/reflection boundaries remain explicit opacity;
- known empty distinct from unknown;
- no formal use of `is_pure_expr` as sufficient proof;
- cancellation/budgets terminal;
- benchmarks show no ordinary runtime cost.

## 70. `@total` acceptance

All must hold:

- exact termination-only semantics documented and tested;
- always-raise finite callable can be total;
- effectful total callable can be total;
- infinite loop not total;
- unknown complex loop not total merely because not refuted;
- native termination metadata explicit;
- `Never` never used as proof;
- no timeout/budget/cancel can satisfy the attribute.

## 71. Contract semantic acceptance

All must hold:

- stable `ContractId`;
- runtime semantics unchanged;
- canonical predicate representation;
- `old` pre-state semantics identical between runtime and proof projections;
- effect/termination eligibility explicit;
- runtime metadata stripping independent from semantic contract identity;
- no runtime closure identity used as proof identity.

## 72. Proof-platform acceptance

Before claiming a public prover exists:

- deterministic VC generation;
- explicit supported logic subset;
- unsupported operations yield honest unknown;
- backend protocol resource limits;
- trust policy;
- counterexample validation policy;
- exact artifact fingerprints;
- stale artifact rejection;
- cancellation/budget states;
- no false `Proven` corpus;
- fuzz/hostile artifact tests;
- clean/incremental proof-key equivalence.

---

# 73. Final normative summary

1. **Spec 01.5 owns ordinary generic semantics.** Spec 05 does not redefine generic parameters, variance, `where` constraints, type lambdas, substitution, inheritance, `Self`, or generic-method inference.
2. **Record rows are a distinct semantic domain and kind.** `RecordRow` is not `Type`; open rows use typed row terms/variables, lacks constraints, occurs checks, and bounded solving.
3. **Kind polymorphism is optional and gated.** Explicit `Type -> Type` kinds and type lambdas already support higher-kinded programming. If kind polymorphism later lands, it is prenex with distinct stable/flexible/rigid IDs.
4. **Effects describe capabilities, not control exits.** The initial effect atoms align with existing native metadata: mutation, I/O, scheduling, reflection, nondeterminism, and blocking.
5. **Known empty effects mean pure under the active effect model. Unknown/opaque never means pure.**
6. **Exit summaries are separate.** Normal return possibility, raises, divergence, process exit, and suspension do not live in the effect set.
7. **`Never` is only a normal-return type fact.** It proves neither divergence nor termination.
8. **`@total` means termination proven only.** It does not imply purity, no effects, no exceptions, no allocation, or a normal result. A finite always-raising callable may be total.
9. **Runtime contracts and static proof share canonical contract identity but not authority.** Runtime guard success is not universal proof; stripped guards do not erase proof evidence.
10. **Postcondition verification is partial correctness by default.** Total correctness additionally requires termination evidence.
11. **Proof outcomes are sealed honest variants:** `Proven`, `Disproven`, `Unknown`, `Cancelled`, `BudgetExceeded`, `InternalFailure`.
12. **`Proven` requires evidence accepted by explicit trust policy.** Backend text alone is not proof; `KernelChecked` and `TrustedBackend` remain distinguishable.
13. **Proof artifacts are exact, versioned, fingerprinted semantic evidence.** Stale/mismatched artifacts are never current proofs.
14. **Advanced metadata uses Spec 02 versioned extensions.** The core type schema does not freeze a prover implementation.
15. **Advanced reflection uses Spec 03 lazy explicit APIs.** Ordinary runtime execution allocates no proof/effect descriptor objects and performs no effect/proof lookup.
16. **All advanced analysis is compiler-owned and incremental.** CLI, LSP, REPL, metadata, and runtime reflection consume the same published semantic facts.
17. **Cancellation and budgets never become semantic success.**
18. **Dynamic dispatch, reflection, DNU, opaque native code, unsupported heap reasoning, and unsupported theories block proof explicitly rather than being guessed safe.**
19. **No advanced fact changes selector identity, dispatch, class/metaclass identity, object layout, or per-instance representation.**
20. **The proof backend is last-mile infrastructure, not the foundation of Phalcom typing.** The language can implement rows, effects, exits, totality, and canonical contracts before choosing a permanent prover or certificate format.

