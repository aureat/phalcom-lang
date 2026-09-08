# Phalcom Recursive Pattern Coverage Remediation
## Repository-Grounded, Checkpoint-Driven, Patch-Grade Implementation Plan

**Program:** Recursive pattern matching, usefulness, exhaustiveness, GADT proof integration, witness generation, and coverage-performance architecture  
**Prepared against repository:** `aureat/phalcom-lang`  
**Remote branch:** `main`  
**Remote HEAD:** `e932aac4e21a5b346e719ede5a24f94e7b924ab3`  
**HEAD subject:** `feat(semantic): complete SC-4.8 typing integration`  
**Prepared:** 2026-09-04  
**Scope:** Planning only. This document does not claim that any implementation task or verification command below has been executed.

> Repository-state limitation: this plan was prepared from the GitHub repository view. The remote `main` revision above is confirmed, but the local checkout's active branch, uncommitted changes, untracked files, and local-only commits are not observable from the planning environment. The implementing agent MUST perform the C0 local drift check before editing and preserve unrelated local work.

---

# 1. Executive Summary

Phalcom's current exhaustiveness checker eagerly constructs a recursively expanded `PatternSpace` for the scrutinee type before it evaluates source patterns. That architecture is not valid for recursive ADTs/GADTs.

At current HEAD, `phalcom-semantic/src/checker/exhaustiveness.rs::build_initial_pattern_space` recursively traverses closed enum payloads, tuples, union members, exact cases, and recursively nested enum fields. Its sole recursion barrier is an exact `BTreeSet<TypeId>` visitation guard. This happens to terminate for simple nominal recursion when the same canonical `TypeId` recurs, but it has no termination argument for indexed recursive types whose recursive payload changes index at every level.

The SC-4.8 constructor-local generic work made that architectural defect observable. The `Expression<F, T>` typing-integration fixture now correctly exposes recursive fields such as:

```phalcom
Apply<A, B>(
    _ function: Expression<F, (A) -> B>,
    _ argument: Expression<F, A>
) -> Expression<F, B>
```

For a scrutinee `Expression<F, T>`, the `Apply` result establishes `B ~ T`, so eager payload expansion can generate:

```text
Expression<F, T>
→ Expression<F, A1 -> T>
→ Expression<F, A2 -> (A1 -> T)>
→ Expression<F, A3 -> (A2 -> (A1 -> T))>
→ ...
```

These are genuinely different type structures. Correct `TypeStore` interning therefore gives them different canonical `TypeId`s. A more efficient interner or a larger exact-ID visited set cannot solve the problem.

The eager universe is then fed to `PatternSpace::normalize`, `intersect`, and `subtract`. Current `PatternSpace` operations clone recursively owned trees, flatten unions, perform structural `Vec::contains` deduplication, distribute intersections through unions, and materialize Cartesian product differences. Eight independently analyzed fixtures can multiply the same runaway CPU/allocation behavior.

The root correction is:

> **Pattern matching must be a finite symbolic elimination procedure driven by source-pattern structure, not recursive enumeration of all values admitted by a recursive type.**

The implementation program therefore replaces the current residual-space proof authority with a **GADT-aware pattern-matrix/usefulness engine**. A closed subject is decomposed one constructor layer only when a source pattern requires that constructor split. Constructor payloads remain typed universal subjects until a nested pattern explicitly inspects them. GADT constructor-local generics are freshly opened with the existing `CaseInstantiation`/`LocalType` machinery. One shared constructor-decomposition service becomes the semantic authority used by both source-pattern resolution and coverage analysis.

The program also:

- preserves exact GADT branch-local proofs and fresh rigid semantics;
- distinguishes a closed-but-unopened subject from a truly open/unknown domain;
- moves or-pattern redundancy onto the same usefulness engine;
- generates missing witnesses from usefulness search instead of residual-tree traversal;
- keeps public `PatternSpaceSummary` products bounded and deterministic without requiring an internal residual tree;
- removes eager recursive universe construction;
- removes Cartesian `PatternSpace` subtraction from the match-analysis hot path;
- removes repeated normalize/clone cycles;
- reuses the existing shared `CheckerControl` budget/cancellation authority;
- adds bounded witness and matrix-search work;
- adds productive/recursive inhabitation analysis as a separate fixed-point concern;
- adds safe, query-local caching only where keys are semantically stable;
- adds focused hostile tests for recursive indexed GADTs, nested patterns, local rigids, open domains, and performance regressions;
- closes the real `phalcom-core` `typing_integration::expression` incident as the final cross-crate acceptance gate.

---

# 2. Confirmed Repository Architecture

## 2.1 Relevant ownership map

| Concern | Current authoritative location | Important symbols/products |
|---|---|---|
| Enum/GADT declaration semantics | `phalcom-semantic/src/checker/enum_declaration.rs` | `build_enum_semantics` |
| Published enum/variant metadata | `phalcom-semantic/src/enum_semantics.rs` | `EnumSemanticTable`, `EnumInfo`, `VariantInfo`, `VariantConstructorSignature` |
| Constructor-local existential opening | `phalcom-semantic/src/types/case_instantiation.rs` | `CaseInstantiation::open` |
| Query-local rigid type language | `phalcom-semantic/src/types/rigid.rs` | `RigidArena`, `LocalType`, `LocalType::alpha_equivalent` |
| Canonical + local GADT proof solving | `phalcom-semantic/src/checker/gadt_proof.rs` | `solve_gadt_branch_proof`, `solve_local_case_proof`, `merge_branch_proofs`, `apply_branch_proof` |
| Source-pattern semantic resolution | `phalcom-semantic/src/checker/pattern.rs` | `resolve_pattern`, `resolve_variant_pattern` |
| Current coverage value-space algebra | `phalcom-semantic/src/checker/pattern_space.rs` | `PatternSpace`, `VariantSpace`, `normalize`, `intersect`, `subtract` |
| Current usefulness/exhaustiveness | `phalcom-semantic/src/checker/exhaustiveness.rs` | `build_initial_pattern_space`, `evaluate_match_arm_usefulness`, `finalize_match_exhaustiveness` |
| Match expression orchestration | `phalcom-semantic/src/checker/expression.rs` | current `Expr::Match` analysis path |
| Shared semantic budget/cancellation | `phalcom-semantic/src/checker/context.rs` | `CheckerControl`, `charge_step`, `charge_scc_iteration`, `relation` |
| Public match semantic products | `phalcom-semantic/src/match_semantics.rs` | `MatchResolution`, `MatchArmResolution`, `PatternResolution`, `ExhaustivenessResult`, `CoverageWitness`, `PatternSpaceSummary` |
| Match-product fingerprinting | `phalcom-semantic/src/db/fingerprint.rs` | `hash_match_resolution`, `hash_pattern_space_summary` |
| ADT matching ownership tests | `phalcom-semantic/tests/semantic/adts/matching/` | `exhaustiveness.rs`, `gadt_refinement.rs`, `patterns.rs`, `pattern_space.rs`, `flow.rs`, `resolution.rs` |
| Incremental ADT/match tests | `phalcom-semantic/tests/semantic/incremental/adts.rs` | cold/incremental semantic-product scenarios |
| Real reproducer | `phalcom-core/tests/core/typing_integration/sources/expression.ph` | `Expression<F,T>`, `ExpressionEvaluation.eval` |
| Real cross-crate tests | `phalcom-core/tests/core/typing_integration/expression/` | refinement, higher-order, monad, rejection, integration, runtime |
| Execution state | `docs/impl/semantic/typing-integration/typing-integration-implementation-state.md` | checkpoint/evidence ledger |
| Existing Part 05.1 spec | `docs/impl/adt-gadt-associated-lookup/part-5/05.1-match-surface-pattern-semantics-exhaustiveness-gadt-proofs-technical-spec.md` | current residual-space model |
| Forward pattern spec | `docs/spec/next/phalcom-pattern-matching-spec.md` | already states symbolic reasoning should replace eager product enumeration |

## 2.2 Identities that must remain distinct

The implementation must not collapse these concepts:

```text
DeclarationId
    canonical enum declaration identity

VariantId
    canonical exact variant declaration identity

TypeId
    canonical/global interned semantic type identity

CaseInstantiation
    one query-local opening of one constructor-local generic binder set

RigidTypeVariableId
    query-local rigid identity; never durable TypeStore metadata

LocalType
    query-local type term that can contain rigids

CoveragePatternId / CoverageSubjectId
    proposed match-query-local analysis identities only; not semantic declaration identities
```

A fresh `CaseInstantiation` is not another declaration. A `RigidTypeVariableId` is not a `TypeId`. A coverage arena ID must never escape into stable semantic identity or persistence.

---

# 3. Root-Cause Evidence Model

| Claim | Repository evidence |
|---|---|
| Exhaustiveness eagerly expands recursive payload types | `checker/exhaustiveness.rs` — `build_initial_pattern_space_inner`, `build_enum_or_opaque_space` |
| Exact `TypeId` identity is the only recursion barrier | `checker/exhaustiveness.rs` — `visiting: BTreeSet<TypeId>` |
| Enum payload fields are recursively expanded after GADT specialization | `checker/exhaustiveness.rs` — field loop calling `build_initial_pattern_space_inner` |
| `PatternSpace` copies/rebuilds recursively owned trees | `checker/pattern_space.rs` — `Clone` representation, `normalize`, `union`, `intersect`, `subtract` |
| Union normalization uses structural linear duplicate search | `checker/pattern_space.rs` — `if !flat.contains(&other)` |
| Product subtraction creates Cartesian branches and clones accumulated fields | `checker/pattern_space.rs` — “Multi-field Cartesian difference” branch |
| Actual source-pattern resolution already leaves uninspected payloads opaque | `checker/pattern.rs` — `PatternSpace::Opaque(field_expected_ty)` and callable-pattern field initialization |
| Actual source-pattern GADT elimination opens constructor-local rigids | `checker/pattern.rs` — `CaseInstantiation::open` + `solve_local_case_proof` |
| Constructor-local generics are callable-owned in enum declaration semantics | `checker/enum_declaration.rs` — `TypeParameterOwner::Callable(CallableId::variant_constructor(...))` |
| Fresh local rigids are explicitly forbidden from canonical metadata | `types/case_instantiation.rs` module/type documentation |
| `LocalType` already supports rigid-containing applied/callable structures | `types/rigid.rs` — `LocalType` |
| `LocalType` already supports alpha-equivalence | `types/rigid.rs` — `LocalType::alpha_equivalent` |
| `CheckerControl` already owns query budget/cancellation | `checker/context.rs` |
| `ExhaustivenessResult` already supports fail-closed `Blocked(BlockReason)` | `match_semantics.rs` |
| `PatternSpaceSummary` is a public/fingerprinted semantic product | `match_semantics.rs`, `db/fingerprint.rs` |
| The real recursive indexed reproducer is in the repository | `phalcom-core/tests/core/typing_integration/sources/expression.ph` |
| Existing tests require nested closed-domain reasoning | `matching/exhaustiveness.rs::match_exh_11_nested_totality_preserves_child_coverage` |
| Existing tests require fresh local-rigid GADT semantics | `matching/gadt_refinement.rs::match_gadt_12_*`, `match_gadt_13_*` |
| Existing hostile tests require open record/map conservatism | `matching/patterns.rs` entries recorded in `COVERAGE_LEDGER.md` |

---

# 4. Normative Semantic Requirements

The implementation program must establish the following invariants.

## PAT-COV-01 — Demand-driven decomposition

A constructor payload MUST NOT be recursively decomposed because its static type is itself a closed ADT/GADT.

A payload is decomposed only if:

1. a source pattern explicitly inspects that payload;
2. witness search needs one finite constructor choice at that position; or
3. a separate bounded inhabitation proof requests constructor productivity.

## PAT-COV-02 — Finite source syntax is the recursion bound

Recursive matching depth is determined by the finite source pattern matrix, not by recursive datatype depth.

For:

```phalcom
Expression::Apply(function, argument)
```

the coverage engine may open `Apply` once, observe two wildcard/binding children, and MUST stop.

For:

```phalcom
Expression::Add(Expression::IntLiteral(x), _)
```

the engine may open the first `Add` payload one additional level because the source pattern asks for that structure.

## PAT-COV-03 — Closed-but-unopened is not open/unknown

The coverage engine MUST distinguish:

```text
Closed subject, not decomposed yet
```

from:

```text
Open/unbounded domain whose constructors cannot be enumerated
```

`Expression<F, Int>` inside an uninspected `Add` payload is closed-but-unopened. `Object` is open.

## PAT-COV-04 — One constructor-decomposition authority

Pattern resolution and coverage MUST share one semantic operation for opening a variant case against an expected subject.

That operation owns:

- enum owner/variant metadata lookup;
- declaration-level generic specialization;
- canonical GADT result-index proof;
- fresh `CaseInstantiation`;
- local GADT proof;
- canonical + local payload subject formation;
- exact-case identity;
- `BranchProofEnvironment`.

No second coverage-only GADT solver is permitted.

## PAT-GADT-01 — Equality-producing constructor selection

GADT elimination remains equality-producing, not subtype filtering.

A generic `Expr<T>` can observe an `Expr<Int>` constructor and establish `T = Int`. A concrete incompatible index must be refuted.

## PAT-GADT-02 — Fresh local constructor generics

Each observation of a constructor-local binder domain gets a fresh `CaseInstantiation`.

Do not cache/reuse opened rigid identities across independent constructor observations.

## PAT-GADT-03 — No local-rigid materialization

Coverage analysis MUST retain `LocalType` when a term contains constructor-local rigids.

It MUST NOT call `LocalType::materialize` merely to force a coverage key into `TypeStore`.

## PAT-GADT-04 — Parent/local proof preservation

Nested patterns may observe a constructor whose expected subject already contains parent rigids. The local proof layer must preserve those parent rigids and may bind flexible declaration parameters to local terms, but must not guess a rigid to fit a concrete type.

## PAT-USE-01 — One usefulness authority

The same usefulness engine determines:

- impossible match arms;
- redundant match arms;
- redundant or-pattern alternatives;
- final exhaustiveness;
- witness existence.

Do not maintain independent set-subtraction logic for one of these cases.

## PAT-USE-02 — Impossible vs redundant remains distinct

`Impossible` means the pattern has no values in the original scrutinee domain.

`Redundant` means the pattern has values in the original domain, but all such values are already covered by preceding applicable rows.

## PAT-EXH-01 — No false proof

`Open`, `Blocked`, cancellation, budget exhaustion, or semantically unresolved coverage MUST never produce `ExhaustivenessResult::Proven`.

## PAT-EXH-02 — Exhaustiveness is wildcard uselessness

For the rows contributing unconditional structural coverage, the match is exhaustive when a wildcard candidate is not useful.

## PAT-WIT-01 — Witnesses use the same domain semantics

Witness generation must use the same constructor feasibility, local-rigid, and inhabitation rules as usefulness.

No independently reconstructed variant universe is permitted.

## PAT-PERF-01 — No eager recursive closure

The outer-only `ExpressionEvaluation.eval` match must not recursively decompose any `Expression` payload merely to prove root-constructor coverage.

## PAT-PERF-02 — No Cartesian residual hot path

Ordinary match usefulness/exhaustiveness MUST NOT build Cartesian residual `PatternSpace` trees.

## PAT-PERF-03 — Query budget is shared

Coverage charges the existing `CheckerControl`. It must not create a private `QueryBudget` that can silently reset the callable/query limit.

## PAT-PERF-04 — Stable bounded semantic products

Published match summaries/witnesses must be bounded and deterministic; they must not force the checker to retain an exponentially large residual proof structure.

---

# 5. Sources of Truth and Forbidden Competing Authorities

## 5.1 Variant declaration metadata

**Source of truth:** `EnumSemanticTable` / `VariantInfo`

**Consumers:**
- source-pattern resolution;
- coverage domain decomposition;
- GADT proof solving;
- witness construction;
- lowering through already published `PatternResolution`.

**Forbidden competing authority:**
- matching variants by short string name inside coverage;
- a second coverage enum registry;
- re-parsing AST enum declarations during body coverage.

## 5.2 GADT constructor observation

**Source of truth:** shared constructor-opening operation built from:

```text
solve_gadt_branch_proof
→ CaseInstantiation::open
→ solve_local_case_proof / local-subject equivalent
→ canonical/local payload specialization
```

**Forbidden competing authority:**
- coverage-only ad hoc `TypeId` substitution;
- treating constructor-local A/B as flexible inference variables;
- nominal subtype filtering instead of GADT equality solving.

## 5.3 Query-local generic terms

**Source of truth:** `LocalType` + `RigidArena`

**Forbidden competing authority:**
- synthetic canonical `TypeId`s containing query-local rigids;
- name-based local generic keys;
- a second coverage-specific rigid AST.

## 5.4 Coverage work control

**Source of truth:** `CheckingContext::control` / `CheckerControl`

**Forbidden competing authority:** new standalone coverage budget/cancellation state.

## 5.5 Public match semantic identity

**Source of truth:** `MatchResolution` / `PatternResolution` using canonical `VariantId`, `VariantFieldId`, `TypeKnowledge`, branch proofs.

**Forbidden competing authority:** LSP/compiler recomputation of coverage or variant identity.

---

# 6. Tempting Wrong Fixes — Explicitly Forbidden

1. **Do not just increase recursion depth.**  
   A depth cap turns semantic output into a compiler-constant-dependent approximation.

2. **Do not replace `BTreeSet<TypeId>` with “seen nominal enum declaration.”**  
   That stops legitimate nested source patterns such as `Node(Node(_, _), _)`.

3. **Do not merely improve type interning.**  
   `Expression<F,T>` and `Expression<F,A -> T>` are legitimately distinct canonical types.

4. **Do not permanently map recursive payloads to current `PatternSpace::Opaque`.**  
   Current opaque subtraction is intentionally conservative and cannot prove nested totality such as `Some(Left)`, `Some(Right)`, `None`.

5. **Do not only optimize `PatternSpace::normalize`.**  
   That would make the checker construct the wrong unbounded object more efficiently.

6. **Do not disable test parallelism as a semantic fix.**  
   It reduces simultaneous blow-ups, not the individual blow-up.

7. **Do not introduce `Dynamic`/`Unknown` to make `Expression` pass.**  
   SC-4.8 exists specifically to preserve constructor-local generic semantics.

8. **Do not reuse one opened `CaseInstantiation` globally.**  
   Independent constructor observations require independent rigid scopes.

9. **Do not hash/memoize proof states by debug string.**  
   Cache only keys with explicit semantic stability.

10. **Do not make LSP/runtime/compiler independent coverage authorities.**  
    Published semantic products remain the shared source.

11. **Do not keep exact residual-tree construction merely because `MatchArmResolution` exposes `residual_after`.**  
    Public summaries must adapt to the proof architecture, not dictate it.

---

# 7. Target Architecture

## 7.1 New checker ownership

Target module layout:

```text
phalcom-semantic/src/checker/
├── coverage/
│   ├── mod.rs
│   ├── subject.rs
│   ├── domain.rs
│   ├── pattern.rs
│   ├── usefulness.rs
│   ├── witness.rs
│   └── inhabitation.rs
├── pattern.rs
├── gadt_proof.rs
├── exhaustiveness.rs        # thin orchestration/compatibility during migration
├── expression.rs
└── ...
```

`coverage` is an internal checker subsystem. It does not create new stable declaration identities.

## 7.2 Core internal types — STRUCTURAL

The exact Rust field layout may be reconciled during implementation, but the responsibilities are fixed.

```rust
// STRUCTURAL — non-paste-ready.
pub(crate) struct CoverageSubject {
    /// Canonical projection used for declaration ownership, diagnostics,
    /// and existing canonical proof operations.
    pub canonical: TypeId,

    /// Exact query-local term. For ordinary subjects this can simply be
    /// LocalType::Canonical(canonical). For constructor-local GADT payloads
    /// it preserves rigid-containing structure.
    pub local: LocalType,
}

pub(crate) enum DomainDecomposition {
    Empty,
    Closed(Box<[ConstructorCase]>),
    Open,
    Blocked(BlockReason),
}

pub(crate) enum ConstructorHead {
    Variant(VariantId),
    Tuple { arity: usize },
    // List should use finite head constructors rather than recursive type expansion.
    ListNil,
    ListCons,
    // Literal-domain heads can be added as supported by current pattern surface.
}

pub(crate) struct ConstructorCase {
    pub head: ConstructorHead,
    pub fields: Box<[CoverageSubject]>,
    pub proof: BranchProofEnvironment,
    pub exact_case: Option<TypeId>,
    pub case_instantiation: Option<CaseInstantiation>,
}
```

### Critical freshness rule

`ConstructorCase` values containing `CaseInstantiation` are **opened observations**.

They MUST NOT be stored in a global or cross-observation cache.

A cache may retain immutable canonical enum metadata or other proof-free templates, but opening a constructor-local binder must remain fresh.

## 7.3 Coverage-pattern arena — STRUCTURAL

Coverage patterns should represent source constraints, not sets of all values.

```rust
// STRUCTURAL — non-paste-ready.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct CoveragePatternId(u32);

pub(crate) struct CoveragePatternArena {
    nodes: Vec<CoveragePattern>,
    wildcard: CoveragePatternId,
}

pub(crate) enum CoveragePattern {
    Wildcard,

    Variant {
        candidates: Box<[VariantId]>,
        fields: Box<[CoveragePatternId]>,
    },

    Or(Box<[CoveragePatternId]>),

    Tuple(Box<[CoveragePatternId]>),

    List {
        prefix: Box<[CoveragePatternId]>,
        rest: Option<CoveragePatternId>,
    },

    RecordPredicate(/* existing resolved field information */),
    MapPredicate(/* existing resolved entry information */),
}
```

A binding becomes `Wildcard` for coverage while remaining a full `PatternResolution::Binding` for semantic products and branch bindings.

Arena IDs avoid recursively cloning pattern trees into every matrix state.

## 7.4 Usefulness engine — STRUCTURAL

```rust
// STRUCTURAL — non-paste-ready.
pub(crate) struct CoverageEngine<'ctx, 'a> {
    ctx: &'ctx mut CheckingContext<'a>,
    root: CoverageSubject,
    patterns: CoveragePatternArena,
    prior_rows: Vec<CoveragePatternId>,
    // Safe query-local caches only; see C5.
}

pub(crate) struct ArmCoverage {
    pub usefulness: PatternUsefulness,
    pub proof: BranchProofEnvironment,
    pub reachable_summary: PatternSpaceSummary,
    pub residual_summary: PatternSpaceSummary,
}

pub(crate) enum UsefulnessSearch {
    Useful(CoverageWitnessSeed),
    NotUseful,
    Blocked(BlockReason),
}
```

The concrete API may differ, but `expression.rs` must hold one match-local coverage engine and add rows sequentially.

---

# 8. Checkpoint Map

| Checkpoint | Tasks | Semantic boundary | Required evidence | Deferred evidence |
|---|---:|---|---|---|
| C0 | 1–3 | Incident is reproducible, locally bounded, and protected by minimal regressions without triggering the known runaway test | local drift record; existing matching/GADT baselines; minimal ordinary-recursion regression; ignored indexed-recursion reproducer present | full Expression suite; crate/workspace gates |
| C1 | 4–8 | One shared constructor/GADT decomposition authority serves both pattern resolution and future coverage, including canonical + local payload subjects | GADT refinement suite; new constructor-domain unit tests; local-rigid hostile cases | matrix usefulness; full exhaustiveness |
| C2 | 9–16 | Match usefulness/exhaustiveness and or-alternative redundancy are demand-driven and pattern-matrix based; eager recursive universe is no longer needed for correctness | recursive coverage suite including `Apply`; exhaustiveness/pattern/GADT suites; focused `Expression::refinement` | witness/public-summary migration; full core package |
| C3 | 17–20 | Witnesses and published summaries are derived from the new coverage authority without requiring a residual tree; blocked coverage fails closed | witness tests; summary determinism; incremental ADT/match product evidence | PatternSpace deletion; performance hardening |
| C4 | 21–24 | Legacy `PatternSpace` residual algebra is removed from the production hot path and allocation-heavy clone/normalize/Cartesian mechanisms cannot silently re-enter | negative searches; matching crate suite; compile fanout; summary/fingerprint tests | inhabitation; full Expression suite |
| C5 | 25–29 | Coverage is resource-bounded, cancellation-aware, productive-recursion aware, and query-locally optimized without violating rigid freshness | budget/cancellation tests; inhabitation tests; operation-count/perf regressions; hostile recursive families | broad core/workspace |
| C6 | 30–34 | Real Expression integration is green; specs/state are authoritative; all migrations/deletions and broad delivery gates are complete | full Expression package; full typing integration; incremental; semantic crate; format/check/test/clippy; negative gates | none |

---

# 9. Checkpoint C0 — Lock the Incident and Establish a Safe Baseline

Tasks:
- Task 1 — Record local repository state and scoped baseline
- Task 2 — Add minimal recursive-coverage regression module
- Task 3 — Record the remediation program in implementation state

## Why this is a checkpoint

The known indexed `Expression` reproducer can consume extreme CPU/memory under the current algorithm. The first checkpoint must establish local repository state and create a safe, minimal RED regression without repeatedly running the known explosive cross-crate suite.

C0 is evidence preparation only. No semantic production fix belongs here.

## Entry conditions

- Remote planning baseline is `main` at `e932aac4e21a5b346e719ede5a24f94e7b924ab3`.
- Current `phalcom-semantic` GADT/matching tests exist.
- `typing_integration/sources/expression.ph` contains the indexed recursive GADT.

## Working set

### Primary

- `phalcom-semantic/tests/semantic/adts/matching/mod.rs`
- new `phalcom-semantic/tests/semantic/adts/matching/recursive_coverage.rs`
- `docs/impl/semantic/typing-integration/typing-integration-implementation-state.md`

### Secondary — inspect only if evidence requires it

- `phalcom-semantic/tests/semantic/adts/support.rs`
- `phalcom-core/tests/core/typing_integration/expression/refinement.rs`
- `phalcom-core/tests/core/typing_integration/support.rs`

### Out of scope

- production coverage code;
- parser;
- VM/compiler lowering;
- LSP;
- runtime ADT representation.

## Semantic contract established by this checkpoint

- The ordinary recursive-ADT baseline is known.
- The indexed-recursion defect has a small repository-local reproducer that can be enabled immediately after the root fix.
- No implementation agent needs to use the full `Expression` package as its inner development loop.

## Semantic risks

- Accidentally running a known explosive test in parallel during baseline.
- Misclassifying a pre-existing local failure as a C0 product failure.
- Modifying unrelated dirty files.

## Hostile cases

- Recursive binary tree payload with wildcard children must be analyzable.
- Indexed recursive constructor whose recursive payload grows the result index must remain present as the RED case; do not simplify it into ordinary `Tree<T>` recursion.

## Required evidence

1. Local repository-state commands:
   ```bash
   git rev-parse --show-toplevel
   git branch --show-current
   git rev-parse HEAD
   git status --short
   git log -5 --oneline
   ```
   **Proves:** actual execution baseline and drift.

2. Existing ownership-layer baselines:
   ```bash
   RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic \
     semantic::adts::matching::gadt_refinement -- --nocapture

   RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic \
     semantic::adts::matching::exhaustiveness -- --nocapture
   ```
   **Proves:** current GADT and coverage baseline before refactor.

3. New ordinary recursive focused test:
   ```bash
   RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic \
     semantic::adts::matching::recursive_coverage::recursive_binary_adt_outer_patterns_terminate_and_are_exhaustive \
     -- --nocapture
   ```
   **Proves:** baseline handling of exact-ID ordinary recursion.

4. Verify the indexed-recursion regression exists but is not yet enabled:
   ```bash
   rg -n 'indexed_recursive_apply_outer_match' \
     phalcom-semantic/tests/semantic/adts/matching/recursive_coverage.rs
   ```
   **Proves:** root defect is locked into the suite without intentionally triggering runaway analysis before C2.

## Do not run yet

```bash
cargo test -p phalcom-core --test core typing_integration::expression::
cargo test --workspace --all-targets
```

Deferred to C2/C6 because current coverage may blow up and these commands add no useful C0 evidence.

## Escalate immediately if

- local HEAD differs materially in the target semantic files;
- `gadt_refinement` is already failing independently;
- the ordinary recursive tree test itself runs away;
- local dirty changes already modify `exhaustiveness.rs`, `pattern.rs`, `gadt_proof.rs`, or `pattern_space.rs`.

## Task 1 — Record local repository state and scoped baseline

### Purpose

Establish the exact execution revision and protect unrelated work.

### Risk

- Semantic: LOW
- Implementation fanout: local

### Owned files and symbols

- No production edits.
- `docs/impl/semantic/typing-integration/typing-integration-implementation-state.md` — append baseline facts only after commands complete.

### Inspect before editing

- `git status --short`
- `git diff -- phalcom-semantic/src/checker phalcom-semantic/tests/semantic/adts/matching`
- current state-file tail

### Do not inspect unless evidence forces expansion

- parser;
- runtime;
- LSP;
- module resolver.

### Source of truth

Local Git checkout state.

### Edit operations

1. Run the exact local-state commands above.
2. Compare local HEAD to the plan baseline.
3. If HEAD drift is mechanical/non-conflicting, record it.
4. If target semantic files contain unrelated local edits, classify as PLAN DRIFT and stop.
5. Append a new state-file section titled:
   ```md
   ## Recursive Match Coverage Remediation
   ```
6. Record baseline SHA, branch, local status note, and planned C0 status.

### Testing classification

No standalone behavioral test; state establishment only.

---

## Task 2 — Add minimal recursive-coverage regression module

### Purpose

Create a narrow ownership-layer fixture for recursive coverage architecture.

### Risk

- Semantic: MEDIUM
- Implementation fanout: local

### Owned files and symbols

- `phalcom-semantic/tests/semantic/adts/matching/mod.rs`
- new `phalcom-semantic/tests/semantic/adts/matching/recursive_coverage.rs`

### Inspect before editing

- `matching/exhaustiveness.rs`
- `matching/gadt_refinement.rs`
- `adts/support.rs::analyze_adt`

### Source of truth

Semantic match products returned by the existing ADT test harness.

### Implementation boundary

Add tests only. Do not patch production behavior in C0.

### Changes

Add at least:

1. `recursive_binary_adt_outer_patterns_terminate_and_are_exhaustive`
   - recursive `Tree<T>`;
   - `Leaf(_)` and `Node(_, _)`;
   - expected exhaustive.

2. `indexed_recursive_apply_outer_match`
   - minimal GADT reproducer containing a constructor equivalent to:
     ```phalcom
     @variant
     Apply<A, B>(
         _ function: Expr<(A) -> B>,
         _ argument: Expr<A>
     ) -> Expr<B>
     ```
     adapted to the smallest legal Phalcom source;
   - root-only patterns;
   - expected exhaustive;
   - mark `#[ignore = "RED: eager recursive coverage universe expands indexed recursion"]` until C2.

3. `explicit_nested_recursive_pattern_requires_only_source_depth`
   - can also start ignored until C2 if current engine becomes expensive;
   - proves nested pattern support is not removed by the fix.

### Must not

- use a timeout as the semantic assertion;
- change test thread count globally;
- duplicate the full Monad/Expression integration source.

### Testing classification

Focused regression required now because it establishes the incident independently from the integration package.

---

## Task 3 — Initialize checkpoint state/evidence ledger

### Purpose

Make the remediation resumable and supervisor-friendly.

### Risk

- Semantic: LOW
- Implementation fanout: local

### Owned files and symbols

- `docs/impl/semantic/typing-integration/typing-integration-implementation-state.md`

### Edit operations

Append:

```md
### Recursive coverage established invariants

- RC-I01: ...
- RC-I02: ...

### Recursive coverage decisions

- RC-D01: ...

### Recursive coverage evidence ledger

| Checkpoint | Command | Result | Proves |
|---|---|---|---|

### Recursive coverage deferred gates

...

### Recursive coverage active incident

...

### Recursive coverage next resume action

...
```

Do not rewrite historical state.

### Testing classification

No standalone test.

## Checkpoint completion

- [ ] local branch/HEAD/status recorded
- [ ] target-file drift classified
- [ ] existing GADT/exhaustiveness baselines recorded
- [ ] recursive coverage module added
- [ ] ordinary recursion focused test passes or baseline incident documented
- [ ] indexed recursion RED test exists and remains intentionally ignored
- [ ] state file updated
- [ ] C0 marked `COMPLETE` or `INCIDENT`

### Suggested commit grouping

```text
test(semantic): lock recursive match coverage regressions
docs(semantic): record recursive coverage remediation baseline
```

---

# 10. Checkpoint C1 — Establish One GADT Constructor-Decomposition Authority

Tasks:
- Task 4 — Introduce coverage subject/domain module skeleton
- Task 5 — Refactor local GADT proof solving to accept local expected subjects
- Task 6 — Implement shared variant-case opening
- Task 7 — Route source-pattern candidate opening through shared authority
- Task 8 — Prove canonical/local payload specialization and rigid freshness

## Why this is a checkpoint

The current repository has two semantic paths:

```text
pattern.rs
    solve_gadt_branch_proof
    → CaseInstantiation::open
    → solve_local_case_proof

exhaustiveness.rs
    solve_gadt_branch_proof
    → canonical substitution only
```

The future matrix engine must not become a third path. C1 first establishes one authoritative constructor observation operation, independently of the coverage algorithm.

## Entry conditions

- C0 COMPLETE.
- Existing `VariantInfo`, `CaseInstantiation`, `LocalType`, and GADT proof APIs remain present.
- Local drift check confirms C1 primary symbols still own the same responsibilities.

## Working set

### Primary

- new `phalcom-semantic/src/checker/coverage/mod.rs`
- new `phalcom-semantic/src/checker/coverage/subject.rs`
- new `phalcom-semantic/src/checker/coverage/domain.rs`
- `phalcom-semantic/src/checker/gadt_proof.rs`
- `phalcom-semantic/src/checker/pattern.rs`
- `phalcom-semantic/src/checker/mod.rs`
- `phalcom-semantic/tests/semantic/adts/matching/gadt_refinement.rs`
- `phalcom-semantic/tests/semantic/adts/matching/recursive_coverage.rs`

### Secondary

- `types/case_instantiation.rs`
- `types/rigid.rs`
- `enum_semantics.rs`
- `checker/enum_declaration.rs`

### Out of scope

- usefulness algorithm;
- witness generation;
- `PatternSpace` deletion;
- compiler/runtime lowering;
- public `MatchResolution` shape.

## Semantic contract established by this checkpoint

- Pattern resolution and coverage call the same variant-case opening implementation.
- A constructor-local generic observation always gets a fresh rigid scope.
- Payload subjects preserve both the canonical projection and exact `LocalType`.
- Nested local expected subjects can be checked without materializing rigids into `TypeStore`.

## Semantic risks

- Reusing one `CaseInstantiation` across observations.
- Applying enum-level substitution but failing to localize constructor-local parameters.
- Keeping current raw `case_instantiation.payload_type` without specializing enum-level generic arguments.
- Binding a rigid to concrete merely to make a candidate fit.
- Leaking query-local local types into canonical match fingerprint identities.

## Hostile cases

1. `Wrap<U>(_ value: U) -> Expr<List<U>>` over generic `Expr<T>` retains one fresh rigid and local equality.
2. `Wrap<U> ... -> Expr<List<U>>` over concrete incompatible `Expr<Int>` remains impossible.
3. Two independent observations of `Wrap<U>` get different rigid IDs/scopes while being alpha-equivalent in shape.
4. A nested constructor expected under a parent local rigid can establish declaration-parameter-to-local-term equalities without rewriting the parent rigid.

## Required evidence

```bash
cargo check -p phalcom-semantic
```

**Proves:** module/API migration compiles.

```bash
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic \
  semantic::adts::matching::gadt_refinement -- --nocapture
```

**Proves:** canonical and constructor-local GADT behavior survives centralization.

New focused domain tests should prove:
- fresh opening;
- full enum-level + constructor-level payload specialization;
- local-subject nesting.

## Do not run yet

- full `typing_integration::expression`;
- workspace tests.

## Escalate immediately if

- correct nested local-subject solving requires canonicalizing a rigid;
- `CaseInstantiation` would need to become persistent/shared across queries;
- constructor opening appears to require parser/compiler/runtime changes;
- `PatternResolution` cannot preserve existing local type products.

---

## Task 4 — Introduce `coverage::subject` and `coverage::domain`

### Purpose

Create the internal ownership seam before migrating behavior.

### Risk

- Semantic: HIGH
- Implementation fanout: multi-file

### Owned files and symbols

- `checker/coverage/mod.rs`
- `checker/coverage/subject.rs`
- `checker/coverage/domain.rs`
- `checker/mod.rs`

### Source of truth

- `LocalType` for query-local exact type term;
- `TypeId` for canonical projection;
- `EnumSemanticTable` / `VariantInfo` for constructor declaration metadata.

### Implementation boundary

#### STRUCTURAL

Introduce:

```rust
pub(crate) struct CoverageSubject {
    canonical: TypeId,
    local: LocalType,
}
```

Required constructors/helpers:

```text
CoverageSubject::canonical(TypeId)
CoverageSubject::from_parts(TypeId, LocalType)
canonical()
local()
contains_local_rigids()       # optional convenience
```

Do not duplicate `LocalType::alpha_equivalent`.

Introduce domain result types separating:

```text
Empty
Closed(...)
Open
Blocked(BlockReason)
```

### Edit operations

1. CREATE `checker/coverage/mod.rs`.
2. CREATE `subject.rs`.
3. CREATE `domain.rs`.
4. ADD `pub(crate) mod coverage;` in `checker/mod.rs`.
5. Do not publicly re-export new internal analysis identities.
6. Keep old coverage path compiling until C2.

### Must not

- add stable serialization for coverage IDs;
- add a second enum metadata table;
- add a new rigid arena.

### Optional compile checkpoint

```bash
cargo check -p phalcom-semantic
```

Reason: catches module visibility/import/lifetime mistakes before semantic edits.

---

## Task 5 — Generalize local GADT proof solving to a local expected subject

### Purpose

Allow nested coverage/pattern decomposition when the expected subject already contains parent rigids.

### Risk

- Semantic: HIGH
- Implementation fanout: local shared semantic

### Owned files and symbols

- `checker/gadt_proof.rs`
  - `solve_local_case_proof`
  - private `unify_local_types`
  - new local-proof application helper if needed

### Inspect before editing

- `solve_local_case_proof`
- `unify_local_types`
- `merge_branch_proofs`
- all production callers of `solve_local_case_proof`

### Source of truth

`BranchProofEnvironment` + `LocalType`.

### Target implementation

Refactor, do not replace, the current canonical entry point.

#### STRUCTURAL

Prefer:

```rust
pub(crate) fn solve_local_case_proof(
    store: &mut TypeStore,
    proof: &BranchProofEnvironment,
    expected_ty: TypeId,
    case: &CaseInstantiation,
) -> LocalCaseProof {
    let expected = ...existing canonical-to-local conversion...;
    solve_local_case_proof_against_local(store, proof, &expected, case)
}

pub(crate) fn solve_local_case_proof_against_local(
    store: &mut TypeStore,
    proof: &BranchProofEnvironment,
    expected: &LocalType,
    case: &CaseInstantiation,
) -> LocalCaseProof
```

If the existing proof's `local_bindings` can affect the expected term, add one well-named helper that applies canonical and local parameter bindings to `LocalType`. It must:

- replace only flexible canonical parameter forms;
- preserve `LocalType::Rigid` leaves;
- recursively preserve applied/callable/tuple/record shape;
- terminate without materializing rigids.

### Must not

- make `unify_local_types` bind a rigid variable as a flexible inference variable;
- change the successful semantics of current `match_gadt_12`/`13`.

### Testing classification

Focused unit tests in `gadt_proof.rs` or existing GADT integration tests are required because this is a shared high-risk equality change.

---

## Task 6 — Implement shared `open_variant_case`

### Purpose

Create one semantic authority for observing a variant against a coverage/pattern subject.

### Risk

- Semantic: HIGH
- Implementation fanout: multi-file

### Owned files and symbols

- `checker/coverage/domain.rs`
- consumes:
  - `VariantInfo`
  - `solve_gadt_branch_proof`
  - `CaseInstantiation::open`
  - local proof solver
  - `substitution_for_applied`
  - `apply_branch_proof`

### Source of truth

`VariantInfo` + subject + existing GADT proof engine.

### Target data flow

```text
CoverageSubject
→ identify canonical enum owner
→ solve canonical GADT branch proof
→ refuted? return None
→ fresh CaseInstantiation::open
→ solve local case proof against CoverageSubject.local
→ contradictory? return None
→ specialize each payload canonically:
     declaration substitution
     → apply canonical branch proof
→ localize each specialized payload using case.replacements()
→ return OpenedVariantCase
```

### Critical payload requirement

Do not use raw `CaseInstantiation::payload_type(index)` as the final coverage payload subject if it leaves enclosing enum declaration parameters unspecialized.

The payload's local term should correspond to the *already declaration-specialized canonical field* with constructor-local replacements applied.

#### STRUCTURAL target

```rust
pub(crate) struct OpenedVariantCase {
    pub variant: VariantId,
    pub exact_case: TypeId,
    pub proof: BranchProofEnvironment,
    pub case_instantiation: CaseInstantiation,
    pub fields: Box<[CoverageSubject]>,
}
```

### Must not

- cache `OpenedVariantCase` across independent observations;
- use `TypeId` equality as a local-rigid compatibility rule;
- mutate `VariantInfo`.

---

## Task 7 — Route `pattern.rs` through the shared opening authority

### Purpose

Delete duplicated constructor/GADT opening semantics from source-pattern resolution.

### Risk

- Semantic: HIGH
- Implementation fanout: local

### Owned files and symbols

- `checker/pattern.rs`
  - `resolve_variant_pattern`
  - contextual singleton/family candidate path using the same old sequence

### Current implementation

`pattern.rs` currently directly calls:

```text
solve_gadt_branch_proof
CaseInstantiation::open
solve_local_case_proof
```

and separately specializes payload fields.

### Target implementation

For every selected `VariantInfo` candidate:

```text
selector/family/name filtering remains in pattern.rs
→ open selected case via coverage::domain shared authority
→ use returned exact_case/proof/CaseInstantiation/field subjects
→ resolve nested source child patterns
→ publish existing ResolvedVariantCandidate/ResolvedFieldPattern
```

The source resolver remains responsible for:

- source spelling;
- explicit/contextual owner selection;
- selector pattern matching;
- labels/prefix/suffix source mapping;
- binding identities;
- diagnostics specific to source syntax.

The shared domain service remains responsible for semantic constructor observation.

### Edit operations

1. FIND every direct `CaseInstantiation::open` in `checker/pattern.rs`.
2. REPLACE the GADT/opening sequence with the shared service.
3. UPDATE field-specialization paths to consume returned `CoverageSubject`s.
4. Preserve `ResolvedFieldPattern.field_type` as canonical `TypeKnowledge`.
5. Preserve `ResolvedFieldPattern.local_type` from `CoverageSubject.local` when local information exists.
6. Preserve existing `ResolvedVariantCandidate.case_instantiation`.
7. SEARCH for another direct constructor-opening sequence in checker production code.
8. Justify any remaining caller.

### Negative search

After C1:

```bash
rg -n 'CaseInstantiation::open' phalcom-semantic/src/checker
```

Expected:
- shared domain authority;
- any low-level proof tests;
- no duplicate source-pattern-specific opening sequence.

---

## Task 8 — Add constructor-domain hostile tests

### Purpose

Prove the new authority, not merely compilation.

### Risk

- Semantic: HIGH
- Implementation fanout: local tests

### Tests

Extend `matching/gadt_refinement.rs` or add domain-specific unit tests.

Required hostile assertions:

1. Existing `match_gadt_12_variant_local_generic_opens_shared_rigid_and_keeps_index_proof` remains green.
2. Existing `match_gadt_13_variant_local_rigid_is_not_guessed_to_fit_concrete_index` remains green.
3. Two independent candidates/openings:
   - have different `RigidScopeId`/`RigidTypeVariableId`;
   - have alpha-equivalent payload/result `LocalType` shape.
4. Enclosing enum generic specialization appears in local payload:
   - e.g. `Expression<F,A>` under `F = SomeConcreteTypeConstructor` retains that concrete outer argument while `A` becomes local rigid.
5. Nested local expected subject:
   - parent rigid survives;
   - nested constructor can bind declaration parameter to parent-local term;
   - no rigid materialization.

## Checkpoint completion

- [ ] coverage module skeleton compiles
- [ ] local-subject GADT proof API established
- [ ] shared variant opening established
- [ ] pattern resolver uses shared authority
- [ ] existing GADT suite passes
- [ ] local-rigid hostile tests pass
- [ ] direct duplicate constructor-opening paths removed
- [ ] state file updated
- [ ] no active INCIDENT

### Suggested commits

```text
refactor(semantic): centralize GADT constructor decomposition
test(semantic): enforce local-rigid constructor opening laws
```

---

# 11. Checkpoint C2 — Make Coverage Demand-Driven and Pattern-Matrix Based

Tasks:
- Task 9 — Introduce coverage-pattern arena
- Task 10 — Translate source patterns to coverage patterns
- Task 11 — Implement finite domain decomposition
- Task 12 — Implement pattern-matrix usefulness
- Task 13 — Integrate GADT proof branches during specialization
- Task 14 — Migrate or-pattern redundancy
- Task 15 — Switch match expression analysis to the new engine
- Task 16 — Enable recursive indexed regressions and prove root fix

## Why this is a checkpoint

These tasks only prove the root semantic correction when integrated. A coverage-pattern AST without usefulness does not solve the bug; a usefulness algorithm without source-pattern integration cannot classify real arms.

C2 establishes the dominant claim:

> **After C2, exhaustiveness no longer depends on recursively materializing the scrutinee's value universe.**

## Entry conditions

- C1 COMPLETE.
- Source-pattern resolution and coverage can invoke one shared constructor opening.
- Current match semantic products remain unchanged externally.

## Working set

### Primary

- `checker/coverage/pattern.rs`
- `checker/coverage/domain.rs`
- `checker/coverage/usefulness.rs`
- `checker/coverage/mod.rs`
- `checker/pattern.rs`
- `checker/expression.rs`
- `checker/exhaustiveness.rs` — transitional wrapper only
- `checker/mod.rs`
- `matching/recursive_coverage.rs`
- `matching/exhaustiveness.rs`
- `matching/gadt_refinement.rs`
- `matching/patterns.rs`

### Secondary

- `match_semantics.rs`
- `types/rigid.rs`
- `enum_semantics.rs`

### Out of scope

- witness overhaul;
- full `PatternSpace` deletion;
- inhabitation SCC;
- runtime/compiler.

## Semantic contract established by this checkpoint

- Wildcards/bindings stop decomposition immediately.
- Variant payloads are opened only when child source patterns require them.
- Root enum constructor enumeration is finite even when payload recursion is infinite/index-growing.
- GADT constructor branches carry proof state.
- Or-pattern redundancy uses the same usefulness authority.
- Final exhaustiveness is wildcard uselessness.

## Semantic risks

- treating an open domain as closed;
- confusing wildcard with an empty pattern;
- losing order-sensitive redundancy;
- leaking a GADT proof to sibling matrix branches;
- incorrectly memoizing opened local rigids;
- regressing tuple/list/open-record semantics;
- using source pattern candidate resolution by name in the coverage engine.

## Hostile cases

- indexed recursive `Apply`;
- nested recursive pattern;
- GADT concrete impossible case;
- generic GADT keeps all satisfiable cases;
- open Object still requires wildcard;
- record/map patterns remain refutable;
- duplicate or-pattern alternative remains redundant;
- tuple product exact coverage remains exhaustive;
- list `[]` + `[head,*tail]` remains exhaustive.

## Required evidence

```bash
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic \
  semantic::adts::matching::recursive_coverage -- --nocapture
```

**Proves:** root recursion architecture.

```bash
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic \
  semantic::adts::matching::exhaustiveness -- --nocapture
```

**Proves:** ordered usefulness/nested/tuple/list semantics.

```bash
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic \
  semantic::adts::matching::gadt_refinement -- --nocapture
```

**Proves:** GADT proof locality and impossible cases.

```bash
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic \
  semantic::adts::matching::patterns -- --nocapture
```

**Proves:** or-pattern and open record/map hostile behavior.

Then the first real cross-crate gate:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::expression::refinement -- --nocapture
```

**Proves:** the real `ExpressionEvaluation.eval` semantic match no longer blows up at the root coverage path.

## Do not run yet

- full workspace;
- full typing-integration package;
- clippy.

## Escalate immediately if

- usefulness needs recursive universe construction to handle nested patterns;
- open-domain exhaustiveness becomes permissive;
- nested local rigids require materialization;
- `expression.rs` still keeps `remaining_space` as correctness state after migration;
- the real `Expression` test is still CPU/allocator-bound inside legacy `PatternSpace`.

---

## Task 9 — Implement `CoveragePatternArena`

### Purpose

Represent finite source constraints without copying recursive value-space trees.

### Risk

- Semantic: MEDIUM
- Implementation fanout: multi-file

### Owned files and symbols

- new `checker/coverage/pattern.rs`
- `checker/coverage/mod.rs`

### Source of truth

Resolved source pattern structure.

### Target implementation

Use query-local arena IDs.

Required behavior:

- one canonical wildcard node;
- each source structural pattern allocates at most O(source-pattern nodes);
- binding nodes map to wildcard for coverage;
- variant candidate identity is canonical `VariantId`, not spelling;
- or-pattern stores alternative IDs;
- tuple/list preserve finite source structure;
- record/map retain refutable predicate shape.

### Allocation requirements

- `Vec::with_capacity` from known child counts;
- no deep clone of child patterns when rows are copied;
- matrix rows carry IDs, not recursively owned `CoveragePattern`s.

---

## Task 10 — Make `resolve_pattern` produce coverage patterns rather than `PatternSpace`

### Purpose

Decouple source pattern semantics from residual value-space algebra.

### Risk

- Semantic: HIGH
- Implementation fanout: multi-file

### Owned files and symbols

- `checker/pattern.rs::resolve_pattern`
- `resolve_pattern_with_mode`
- `resolve_variant_pattern`
- or-pattern branch
- tuple/list/record/map resolution
- `checker/expression.rs` caller

### Current signature

```rust
pub fn resolve_pattern(
    ctx: &mut CheckingContext<'_>,
    pattern: &Pattern,
    expected_ty: TypeId,
    expected_space: &PatternSpace,
    bindings: &mut Vec<PatternBindingResolution>,
) -> (PatternResolution, PatternSpace)
```

### Target signature — STRUCTURAL

Prefer passing a `CoverageSubject` plus arena:

```rust
pub(crate) fn resolve_pattern(
    ctx: &mut CheckingContext<'_>,
    pattern: &Pattern,
    expected: &CoverageSubject,
    arena: &mut CoveragePatternArena,
    bindings: &mut Vec<PatternBindingResolution>,
) -> (PatternResolution, CoveragePatternId)
```

Reconcile lifetime/borrowing mechanics with the repository.

### Changes by pattern kind

#### Wildcard

Return shared wildcard ID.

#### Name binding

If not a contextual singleton, return wildcard coverage ID while publishing binding resolution.

#### Variant

Resolve owner/selector/candidate semantics as today; return a variant coverage node with exact candidate `VariantId`s and finite child pattern IDs.

#### Or

Resolve alternatives independently for semantic binding checks; create `CoveragePattern::Or`.

Do not run local `PatternSpace::intersect/subtract` here after Task 14.

#### Tuple

Build child coverage patterns only from tuple child source syntax.

#### List

Represent the finite source list constraint; do not recursively expand `List<T>`.

#### Record/Map

Keep as refutable predicates. They cannot by themselves claim an open domain is exhausted.

### Must not

- store `PatternSpace` inside `CoveragePattern`;
- reintroduce a value-space universe through another name.

---

## Task 11 — Implement finite domain decomposition

### Purpose

Answer “what can appear at this scrutiny position?” one layer at a time.

### Risk

- Semantic: HIGH
- Implementation fanout: local

### Owned files and symbols

- `coverage/domain.rs`

### Behavior

#### `Never`

Return `Empty`.

#### Canonical/local union

Return finite member decomposition or branch each union member symbolically. Do not recursively close member payloads.

#### Closed enum/GADT

Enumerate root `VariantInfo`s from `EnumSemanticTable`; open each feasible case through the C1 authority.

#### Exact case

Return only its exact variant if semantically compatible.

#### Tuple

Return one tuple constructor with element subjects; do not decompose elements.

#### List

Use finite head semantics (`Nil`/`Cons` or equivalent internal list-shape decomposition) rather than recursive list expansion.

#### Open nominal/object/dynamic domain

Return `Open`.

#### Record/map predicate domains

Keep domain openness separate from pattern predicate shape.

### Must not

Call `build_initial_pattern_space_inner`.

---

## Task 12 — Implement pattern-matrix usefulness core

### Purpose

Make usefulness the correctness authority.

### Risk

- Semantic: HIGH
- Implementation fanout: multi-file

### Owned files and symbols

- `coverage/usefulness.rs`

### Algorithm contract

Use a Maranget-style specialization model:

```text
useful(matrix, candidate_row, subjects, proof)
```

At each step:

1. If there are no pattern columns:
   - candidate is useful iff no prior row covers the empty row under current proof.

2. If candidate head is wildcard:
   - if subject domain is a finite closed constructor set, specialize default/constructor branches as required;
   - if open, prior irrefutable wildcard determines coverage; otherwise candidate remains potentially useful.

3. If candidate head is constructor:
   - ask domain decomposition for that subject;
   - retain only compatible matching constructor cases;
   - replace head column with constructor fields and child pattern columns;
   - merge constructor proof into branch proof;
   - recurse.

4. Or-pattern:
   - evaluate alternatives as alternative rows without materializing a union space.

5. Contradictory proof merge:
   - branch is empty/unreachable.

### Termination property

Every recursive structural specialization consumes one constructor node from a finite source pattern row or reduces matrix shape. Type recursion alone does not call usefulness recursively.

### Budget

Task 26 adds full budgeting, but wire the recursion so a `CheckerControl` charge can be inserted without redesign.

---

## Task 13 — Integrate GADT proof branches into specialization

### Purpose

Preserve indexed constructor feasibility while avoiding type-driven recursion.

### Risk

- Semantic: HIGH
- Implementation fanout: local

### Source of truth

C1 `open_variant_case`.

### Required behavior

For:

```text
subject: Expression<F,T>
candidate head: Apply
```

open `Apply<A,B>` and obtain approximately:

```text
proof: B ~ T
fields:
    Expression<F, α -> T>
    Expression<F, α>
```

If source children are bindings/wildcards:

```text
[_, _]
```

the branch terminates immediately.

The engine MUST NOT decompose `Expression<F, α -> T>` unless a nested child pattern is present.

### Proof locality

Constructor proof applies only inside that specialized matrix branch.

Sibling constructors receive independent proofs/openings.

---

## Task 14 — Move or-pattern redundancy onto usefulness

### Purpose

Delete the second residual-space authority inside `pattern.rs`.

### Risk

- Semantic: HIGH
- Implementation fanout: local

### Current implementation

`resolve_pattern_with_mode` currently uses:

```text
expected_space.intersect
local_remaining.intersect
local_remaining.subtract
```

for or-pattern redundancy.

### Target

Resolve all alternatives to coverage-pattern IDs first, then ask the same usefulness engine sequentially for alternatives within the arm.

The semantic binding-join logic remains in `pattern.rs`.

### Hostile evidence

Existing duplicate/subsumed or-pattern tests must remain green.

### Negative gate

After this task, `pattern.rs` should no longer call `PatternSpace::intersect` or `PatternSpace::subtract`.

---

## Task 15 — Switch `expression.rs` match analysis to `CoverageEngine`

### Purpose

Remove `remaining_space` as the match correctness state.

### Risk

- Semantic: HIGH
- Implementation fanout: multi-file

### Current path

```text
build_initial_pattern_space
remaining_space = initial_space.clone().normalize()

for arm:
    resolve_pattern → arm_space
    evaluate_match_arm_usefulness
    remaining_space = residual_after
```

### Target path — STRUCTURAL

```text
scrutinee TypeKnowledge
→ CoverageSubject
→ CoverageEngine::new

for arm:
    resolve_pattern → PatternResolution + CoveragePatternId
    coverage.classify_arm(pattern_id)
    if useful:
        install pattern-derived branch proof
        analyze branch
    coverage.commit_unconditional_row(pattern_id)
    publish bounded summaries

coverage.finish()
→ Proven / Missing / Blocked
```

If guards are not yet in the current implemented surface, keep the API ready for a future “does this row contribute total structural coverage?” flag; do not fabricate guard behavior now.

### Branch body proof

Preserve the existing source-pattern `BranchProofEnvironment` used for branch typing. Coverage classification does not replace the branch-body proof product.

### Must not

- publish coverage arena IDs;
- keep a hidden `remaining_space` fallback that can silently take over.

---

## Task 16 — Enable indexed-recursion regressions and close the root incident

### Purpose

Prove the architecture using the defect that defeated exact `TypeId` recursion.

### Risk

- Semantic: HIGH
- Implementation fanout: tests

### Edit operations

1. REMOVE `#[ignore]` from `indexed_recursive_apply_outer_match`.
2. Add/assert a nested source-depth test.
3. Run only the exact new tests first.
4. Run the full `recursive_coverage` module.
5. Run existing matching suites.
6. Run `typing_integration::expression::refinement`.

### Performance evidence

Do not use wall-clock as the sole proof.

At minimum, instrument/query the new engine in unit tests to assert:

```text
outer-only indexed Expression:
    root enum decomposed
    recursive payload constructor decompositions == 0
```

The exact metrics exposure is completed in C5; C2 may use a crate-private test hook.

## Checkpoint completion

- [ ] no correctness path needs eager recursive initial universe
- [ ] indexed Apply regression passes
- [ ] nested source-depth regression passes
- [ ] exhaustiveness suite passes
- [ ] GADT refinement suite passes
- [ ] patterns/or/open-domain suite passes
- [ ] focused real Expression refinement test completes without runaway allocation
- [ ] state file updated
- [ ] no active INCIDENT

### Suggested commits

```text
feat(semantic): introduce demand-driven coverage patterns
fix(semantic): prove match usefulness with pattern matrix
test(semantic): cover recursive indexed exhaustiveness
```

---

# 12. Checkpoint C3 — Move Witnesses and Public Summaries Off Residual Trees

Tasks:
- Task 17 — Generate witnesses from usefulness search
- Task 18 — Define bounded deterministic summary projection
- Task 19 — Migrate `MatchResolution` construction/fingerprinting
- Task 20 — Formalize blocked coverage behavior

## Why this is a checkpoint

After C2, correctness no longer needs a residual `PatternSpace`, but public semantic products still expose `initial_space`, `reachable_space`, and `residual_after`; fingerprinting depends on them. C3 prevents those products from forcing the old internal architecture to remain alive.

## Entry conditions

- C2 COMPLETE.
- Matrix usefulness is authoritative.
- Existing public match product consumers are known.

## Working set

### Primary

- `coverage/witness.rs`
- `coverage/usefulness.rs`
- `match_semantics.rs`
- `checker/expression.rs`
- `db/fingerprint.rs`
- matching witness/flow/resolution tests
- `semantic/incremental/adts.rs`

### Secondary

- docs/tooling consumers discovered by `rg 'PatternSpaceSummary|residual_after|initial_space'`.

### Out of scope

- LSP-specific independent proof logic;
- compiler/runtime lowering changes unless a concrete consumer unexpectedly depends on residual summaries.

## Semantic contract

- Missing witnesses come from the same symbolic search that proves wildcard usefulness.
- Public summaries are bounded diagnostic/tooling projections.
- Semantic fingerprints remain deterministic.
- Budget/unknown blockage cannot become `Proven`.

## Hostile cases

- nested missing witness preserves `Some(Right)`-like shape;
- GADT impossible constructors never appear as witnesses;
- open-domain witness remains opaque/wildcard, not fabricated closed constructor;
- two identical cold analyses produce equal products/fingerprints;
- incremental recomputation produces the same semantic result as cold analysis.

## Required evidence

Existing:
- `match_exh_12_nested_missing_witness_preserves_shape`
- `review_m5_05_witness_generation_is_deterministic`
- GADT impossible coverage tests.

Incremental:
```bash
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic \
  semantic::incremental::adts -- --nocapture
```

## Task 17 — Generate bounded witnesses from usefulness

### Purpose

Remove dependency on `push_coverage_witnesses(PatternSpace)`.

### Risk

- Semantic: HIGH
- Implementation fanout: local

### Target

When:

```text
useful(previous_rows, wildcard)
```

returns useful, retain the constructor choices made along one search path.

Convert that search path to existing `CoverageWitness`.

Repeat/search alternatives until:

```text
MAX_COVERAGE_WITNESSES == 8
```

or no additional representative witness is required.

### Allocation rules

- witness search is bounded;
- do not clone full matrices for every witness if row/slice views or arena IDs suffice;
- never enumerate infinite recursive payload structure;
- use `Wildcard`/`Opaque` for uninspected/open payloads.

---

## Task 18 — Define bounded `PatternSpaceSummary` projection

### Purpose

Preserve published semantic observability without preserving residual algebra.

### Risk

- Semantic: MEDIUM
- Implementation fanout: multi-file

### Source of truth

Coverage engine state and source patterns, not legacy `PatternSpace`.

### Required projection semantics

#### `initial_space`

For a closed root enum, a root-level union/variant summary is acceptable, but payload fields MUST remain bounded/unopened (`Opaque`/equivalent summary) unless summary generation is explicitly depth-bounded.

For open roots, use `Opaque(root_ty)`.

#### `reachable_space`

Summarize the source pattern's reachable structural head and child pattern shape.

#### `residual_after`

Do not rebuild the exact full residual domain.

Required minimum guarantees:
- `Empty` iff coverage is proven empty after that row;
- non-empty residual remains non-empty;
- when a bounded witness gives a precise cheap shape, it may be reflected;
- never fabricate exactness beyond the proof.

### Compatibility review

Before changing semantics, inspect every production and test consumer of:
- `initial_space`
- `reachable_space`
- `residual_after`

Update tests that were asserting implementation-specific residual representation while preserving their actual semantic intent.

---

## Task 19 — Update deterministic fingerprinting and incremental evidence

### Purpose

Keep match semantic products stable under the new bounded summaries.

### Risk

- Semantic: HIGH
- Implementation fanout: multi-file / query boundary

### Owned files/symbols

- `match_semantics.rs`
- `db/fingerprint.rs::hash_match_resolution`
- `hash_pattern_space_summary`
- `tests/semantic/incremental/adts.rs`

### Must not

- hash arena IDs;
- hash `RigidTypeVariableId` from query-local openings as durable semantic identity;
- introduce nondeterministic `HashMap` iteration into published ordering.

### Determinism requirements

- variant ordering follows canonical `EnumInfo.variants` or explicit deterministic sort;
- witness ordering deterministic;
- summary union ordering deterministic;
- no pointer/address-based ordering.

### Cold-vs-incremental evidence

Compare actual:
- `ExhaustivenessResult`;
- arm `PatternUsefulness`;
- canonical `VariantId` candidate identities;
- deterministic summary/witness products as appropriate.

---

## Task 20 — Formalize `Blocked` coverage behavior

### Purpose

Make fail-closed behavior explicit before resource budgeting.

### Risk

- Semantic: HIGH
- Implementation fanout: local

### Target

Any domain decomposition/search result that cannot establish a safe answer produces:

```rust
ExhaustivenessResult::Blocked(reason)
```

or an internal blocked result propagated to that public product.

A blocked arm classification must not be silently downgraded to “impossible” if the distinction affects branch analysis.

If current `PatternUsefulness` has no blocked variant, keep usefulness conservative and record match-level blocked state; do not widen the public enum casually without inspecting all consumers.

### Required hostile test

Force a known `BlockReason` through a controlled test fixture and prove the final match is not `Proven`.

## Checkpoint completion

- [ ] residual-tree witness traversal removed from authority
- [ ] bounded witness search passes nested/GADT tests
- [ ] public summaries no longer require full residual tree
- [ ] fingerprints deterministic
- [ ] incremental ADT/match product evidence passes
- [ ] blocked coverage cannot yield Proven
- [ ] state file updated

### Suggested commits

```text
refactor(semantic): derive match witnesses from usefulness
refactor(semantic): bound published match coverage summaries
test(semantic): enforce match product incremental determinism
```

---

# 13. Checkpoint C4 — Retire Legacy `PatternSpace` Hot-Path Algebra and Eliminate Avoidable Allocation

Tasks:
- Task 21 — Remove eager universe builders and residual APIs
- Task 22 — Migrate/delete legacy `PatternSpace` tests
- Task 23 — Remove clone/normalize/Cartesian allocation paths
- Task 24 — Tighten query-local memory layout and TypeStore growth

## Why this is a checkpoint

C2/C3 establish replacement correctness. C4 proves migration completeness:

> **The old authority cannot silently run anymore.**

This is where positive evidence is paired with deletion evidence.

## Entry conditions

- C3 COMPLETE.
- No public consumer requires internal `PatternSpace` values.
- Bounded `PatternSpaceSummary` remains independently constructible.

## Working set

### Primary

- `checker/exhaustiveness.rs`
- `checker/pattern_space.rs`
- `checker/pattern.rs`
- `checker/mod.rs`
- `checker/coverage/*`
- `tests/semantic/adts/matching/pattern_space.rs`
- `tests/semantic/adts/matching/mod.rs`

### Secondary

- `db/fingerprint.rs`
- docs mentioning internal `PatternSpace`.

### Out of scope

- changing runtime representation;
- reworking generic inference;
- parser.

## Semantic contract

- No production match correctness path uses residual set subtraction.
- No production match correctness path recursively builds a `PatternSpace` universe.
- `PatternSpace` either disappears internally or survives only as clearly non-authoritative bounded diagnostic compatibility code.
- Deep clone/normalize/Cartesian operations are absent from the hot path.

## Required evidence

### Negative searches

```bash
rg -n 'build_initial_pattern_space_inner|build_enum_or_opaque_space' \
  phalcom-semantic/src
```

Expected: zero production hits.

```bash
rg -n 'remaining_space|evaluate_match_arm_usefulness|evaluate_match_exhaustiveness' \
  phalcom-semantic/src/checker
```

Expected: zero legacy residual-authority hits, unless a named compatibility wrapper is intentionally retained and documented.

```bash
rg -n 'Multi-field Cartesian difference|flat\.contains\(&other\)' \
  phalcom-semantic/src/checker
```

Expected: zero production hot-path hits if `pattern_space.rs` is deleted.

```bash
rg -n 'PatternSpace' phalcom-semantic/src
```

Every remaining occurrence must be justified. `PatternSpaceSummary` is not the same thing and may remain.

### Behavioral

```bash
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic \
  semantic::adts::matching:: -- --nocapture
```

## Task 21 — Remove eager universe and residual APIs

### Purpose

Delete the root defect rather than leave a dormant fallback.

### Risk

- Semantic: MEDIUM after C3
- Implementation fanout: multi-file

### EXACT deletion anchors

Remove/de-authoritize:

- `exhaustiveness.rs::build_initial_pattern_space`
- `build_initial_pattern_space_inner`
- `build_enum_or_opaque_space`
- `evaluate_match_arm_usefulness`
- `evaluate_match_exhaustiveness`
- `finalize_match_exhaustiveness` in its residual-space form
- old exports from `checker/mod.rs`

If `exhaustiveness.rs` remains, make it a thin façade over `coverage` only.

### Must not

Leave a “fallback to old PatternSpace if new engine blocks.” Blocked must remain blocked.

---

## Task 22 — Migrate/delete `matching/pattern_space.rs`

### Purpose

Stop testing an obsolete internal algebra as if it were the language's semantic contract.

### Risk

- Semantic: MEDIUM
- Implementation fanout: tests

### Current tests to preserve semantically, not mechanically

Examples:

- nested subtraction exact residual
  → migrate to nested usefulness + missing witness test;

- disjoint variants intersection
  → migrate to constructor usefulness/impossibility;

- tuple Cartesian subtraction
  → migrate to tuple matrix coverage;

- opaque subtraction conservative
  → migrate to open-domain usefulness.

### Must not

Preserve `PatternSpace` merely because old tests instantiate it directly.

If a bounded summary helper needs unit tests, create summary-specific tests under the new subsystem.

---

## Task 23 — Remove allocation-heavy normalization/subtraction mechanisms

### Purpose

Eliminate known allocation amplifiers.

### Risk

- Semantic: LOW after authority migration
- Implementation fanout: local

### Required removals

If `PatternSpace` is deleted:

- recursive owned `Union(Box<[PatternSpace]>)`;
- repeated `.clone().normalize()` match state;
- `flat.contains(&other)` structural union dedupe;
- Cartesian branch construction;
- `accumulated_inter.clone()`;
- repeated deep proof/tree cloning.

If a compatibility summary builder remains:

- it must operate over bounded source/witness shapes;
- dedupe via stable small keys/IDs, not deep structural tree comparison.

---

## Task 24 — Tighten arena/scratch allocation and protect `TypeStore`

### Purpose

Make the new engine allocation-conscious without compromising semantics.

### Risk

- Semantic: MEDIUM
- Implementation fanout: local

### Required optimizations

1. `CoveragePatternArena`
   - `Vec` storage;
   - integer IDs;
   - one wildcard node;
   - reserve using estimated source pattern count when cheaply available.

2. Matrix rows
   - store pattern IDs;
   - use `Vec::with_capacity`;
   - pass slices/views where possible instead of cloning complete rows.

3. Constructor specialization
   - reserve exactly variant field arity;
   - avoid converting back/forth between boxed trees.

4. Proof values
   - clone only when a branch actually forks;
   - do not clone proof environments for wildcard fields that terminate immediately.

5. Local types
   - never materialize a rigid-containing `LocalType`;
   - use canonical projection only where existing APIs require declaration lookup.

6. TypeStore growth
   - add a focused test around the indexed `Apply` match showing analysis does not intern an unbounded callable-index chain.

### Testing classification

Focused perf/operation assertions; no benchmark suite required yet.

## Checkpoint completion

- [ ] old eager builder removed
- [ ] old residual hot-path APIs removed
- [ ] legacy direct PatternSpace tests migrated/deleted
- [ ] negative searches match expected state
- [ ] matching suite passes
- [ ] TypeStore growth is bounded for indexed recursion
- [ ] state file updated

### Suggested commits

```text
refactor(semantic): retire residual PatternSpace coverage authority
perf(semantic): remove recursive coverage clone-normalize hot path
```

---

# 14. Checkpoint C5 — Resource Bounds, Inhabitation, Safe Memoization, and Performance Hardening

Tasks:
- Task 25 — Add productive inhabitation fixed-point analysis
- Task 26 — Integrate shared budget/cancellation
- Task 27 — Add only semantically safe query-local caches
- Task 28 — Add operation-count/performance regression evidence
- Task 29 — Add hostile recursive-family stress cases

## Why this is a checkpoint

Demand-driven matrix specialization fixes the root infinite universe. C5 makes the engine robust against difficult but finite source matrices and recursive inhabitation questions.

## Entry conditions

- C4 COMPLETE.
- Legacy residual hot path removed.
- `CheckerControl` remains the shared query policy.

## Working set

### Primary

- `coverage/inhabitation.rs`
- `coverage/usefulness.rs`
- `coverage/domain.rs`
- `coverage/witness.rs`
- `checker/context.rs` only if an adapter is genuinely necessary
- recursive coverage tests

### Secondary

- `db/budget.rs`
- existing fixed-point/loop analysis for style/policy only.

### Out of scope

- global persistent coverage cache;
- cross-query rigid reuse;
- generic inference redesign.

## Semantic contract

- Recursive uninhabited closed families are recognized without eager unfolding.
- Coverage consumes the same query budget/cancellation authority as the rest of semantic analysis.
- Budget exhaustion returns blocked, never exhaustive.
- Caches cannot merge distinct existential observations.
- Performance regressions are measured by semantic work counts, not only wall time.

## Hostile cases

- `enum Loop { @variant Next(_ next: Loop) }` — no finite inhabitant.
- recursive family with at least one base constructor — inhabited.
- mutually recursive productive/unproductive groups if source surface permits the fixture.
- wide enum + nested patterns.
- many or-pattern alternatives.
- recursive indexed GADT where index grows but source depth is one.
- cancellation/budget zero/low cases.

---

## Task 25 — Implement tri-state inhabitation

### Purpose

Separate “can this type have a finite value?” from ordinary pattern decomposition.

### Risk

- Semantic: HIGH
- Implementation fanout: local

### Source of truth

`coverage::domain` constructor decomposition.

### Target — STRUCTURAL

```rust
pub(crate) enum Inhabitation {
    Inhabited,
    Uninhabited,
    Unknown,
    Blocked(BlockReason),
}
```

Use memoized fixed-point/SCC-style productivity reasoning.

For a constructor to be productively inhabited:

- constructor itself is feasible;
- every required payload that must contain a value is inhabited;
- function/native/open payload domains may conservatively be `Unknown`/inhabited according to current type semantics, not enumerated.

A recursive type with no finite base path reaches `Uninhabited`.

### Budget

Charge `CheckerControl::charge_scc_iteration()` for fixed-point rounds.

### Must not

Use recursive `TypeId` descent without a fixed-point state table.

---

## Task 26 — Wire coverage to `CheckerControl`

### Purpose

Prevent any finite but hostile matrix from monopolizing the process.

### Risk

- Semantic: HIGH
- Implementation fanout: local

### Existing authority

`checker/context.rs::CheckerControl` already has:

```text
charge_step
charge_scc_iteration
is_cancelled
relation
```

### Required charging points

At minimum:

- matrix usefulness state expansion;
- constructor-domain specialization;
- witness search branch;
- inhabitation fixed-point iteration.

Do not charge every trivial field access; use semantically meaningful work units.

### Propagation

```text
BudgetExceeded(report)
→ BlockReason::BudgetExceeded(report)
→ coverage blocked
→ ExhaustivenessResult::Blocked(...)
```

Cancellation must follow current checker cancellation conventions; inspect existing relation/inference call sites rather than inventing a new public reason.

---

## Task 27 — Add safe query-local caches only

### Purpose

Avoid repeated constructor metadata work without violating local-generic freshness.

### Risk

- Semantic: HIGH
- Implementation fanout: local

### Allowed cache targets

Good candidates:

- canonical enum owner → immutable variant metadata list;
- proof-free closed-domain head metadata;
- inhabitation state table;
- exact source `CoveragePatternId` specialization subresults where branch proof/key semantics are explicit.

### Forbidden cached values

- `OpenedVariantCase` containing a `CaseInstantiation`;
- `RigidTypeVariableId` across independent observations;
- a result keyed only by enum nominal head when index/proof changes feasibility;
- a proof-bearing state with an unstable/debug-string key.

### `LocalType::alpha_equivalent`

May be used for:

- defensive structural comparison;
- test assertions that fresh openings have the same shape;
- a carefully designed normalized memo key **only if** scope freshness remains semantically separate.

It MUST NOT cause two independent existential openings to share the same rigid identities.

### INVESTIGATE-BEFORE-EDIT

Before adding a proof-bearing usefulness memo table, inspect whether a stable hashable proof key already exists. `BranchProofEnvironment` currently derives equality but not a durable hash contract. If no safe key exists, skip this cache. Correct demand-driven analysis is preferable to unsafe memoization.

---

## Task 28 — Add crate-private coverage metrics and allocation regressions

### Purpose

Test the architecture rather than merely hoping it stays fast.

### Risk

- Semantic: LOW
- Implementation fanout: local

### STRUCTURAL metrics

```rust
pub(crate) struct CoverageMetrics {
    constructor_decompositions: usize,
    matrix_specializations: usize,
    proof_merges: usize,
    witness_states: usize,
    inhabitation_iterations: usize,
    // optional cache hit/miss counters
}
```

Do not add metrics to stable public `MatchResolution` unless a real external consumer requires them.

Expose them through crate-private engine test APIs/unit tests.

### Required architecture assertions

For the minimal indexed `Apply` outer match:

```text
recursive payload decomposition count == 0
```

after root constructor decomposition.

For a nested source pattern of depth two:

```text
decomposition occurs only along the explicitly inspected path
```

For repeated wide root constructor use:

```text
root metadata enumeration remains bounded
```

### TypeStore regression

Record `store.len()` before/after focused coverage in a unit-level or fixture-level test if the harness can expose it cleanly.

Assert a bounded delta rather than a brittle exact number.

---

## Task 29 — Add hostile performance/termination cases

### Purpose

Ensure the architecture survives more than the original fixture.

### Risk

- Semantic: MEDIUM
- Implementation fanout: tests

### Required cases

1. binary recursive tree;
2. indexed `Apply` recursion;
3. recursive type with three recursive children;
4. recursive type under function return payload;
5. deep but finite nested source pattern;
6. wide enum;
7. large or-pattern;
8. uninhabited recursive family;
9. open domain + wildcard;
10. GADT generic root + local constructor generics.

Avoid arbitrary enormous fixtures. Tests should isolate one complexity axis each.

## Checkpoint completion

- [ ] inhabitation fixed-point passes productive/unproductive tests
- [ ] shared budget/cancellation wired
- [ ] budget exhaustion is blocked
- [ ] no private reset budget exists
- [ ] no opened rigid case is cached across observations
- [ ] operation metrics prove source-driven recursion
- [ ] TypeStore growth bounded
- [ ] hostile recursive tests pass
- [ ] state file updated

### Suggested commits

```text
feat(semantic): add productive coverage inhabitation analysis
perf(semantic): budget and harden match usefulness search
test(semantic): enforce recursive coverage work bounds
```

---

# 15. Checkpoint C6 — Integration Closure, Specification Migration, and Delivery Gates

Tasks:
- Task 30 — Run and repair the complete real Expression typing integration
- Task 31 — Prove semantic/incremental cross-consumer consistency
- Task 32 — Amend authoritative pattern/exhaustiveness specifications
- Task 33 — Complete deletion/state/documentation gates
- Task 34 — Run final broad delivery gates

## Why this is a checkpoint

C6 proves the remediation in the real feature stack and prevents architecture drift back toward eager value-space construction.

## Entry conditions

- C5 COMPLETE.
- No unresolved coverage INCIDENT.
- Legacy hot path deletion gates passed.

## Working set

### Primary

- `phalcom-core/tests/core/typing_integration/expression/`
- `phalcom-core/tests/core/typing_integration/sources/expression.ph`
- `phalcom-semantic/tests/semantic/incremental/adts.rs`
- Part 05.1 technical spec
- `docs/spec/next/phalcom-pattern-matching-spec.md`
- implementation state file

### Secondary

- fingerprint code;
- testing scenario catalog if law IDs need amendment.

### Out of scope

- new pattern syntax;
- guards if not already implemented;
- runtime representation redesign;
- LSP feature additions;
- reflection changes.

## Semantic contract

- `ExpressionEvaluation.eval` completes and is semantically correct.
- All existing GADT/local-rigid laws remain valid.
- Match semantic products are stable cold vs incremental.
- Authoritative specs describe demand-driven symbolic coverage.
- No old coverage authority remains silently executable.

## Required evidence — smallest first

### Expression focused

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::expression::refinement -- --nocapture
```

### Full Expression subtree

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::expression:: -- --nocapture
```

### Unified typing integration

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration:: -- --nocapture
```

### Full semantic matching

```bash
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic \
  semantic::adts::matching:: -- --nocapture
```

### Incremental

```bash
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic \
  semantic::incremental::adts -- --nocapture
```

## Task 30 — Close the complete `Expression` integration

### Purpose

Use the real recursive higher-kinded GADT as cross-crate proof.

### Risk

- Semantic: HIGH
- Implementation fanout: cross-crate tests; production fixes should remain semantic-only unless evidence proves otherwise

### Required behavior

The existing source:

```text
phalcom-core/tests/core/typing_integration/sources/expression.ph
```

must remain semantically rich:

- constructor-local generics;
- higher-kinded `F`;
- recursive `If`, `Map`, `FlatMap`, `Apply`;
- function-valued recursive payload;
- nested type lambdas;
- evaluator root match.

Do not simplify the fixture to make coverage pass.

### Failure protocol

If the full Expression subtree fails after coverage terminates:

1. classify the failure separately;
2. prove whether it is coverage, generic inference, branch typing, runtime, or harness;
3. do not reopen the old coverage mechanism.

---

## Task 31 — Prove cold/incremental semantic product consistency

### Purpose

Protect `MatchResolution` query/fingerprint semantics.

### Risk

- Semantic: HIGH
- Implementation fanout: query/incremental

### Required invariant

For the same source state, cold and incremental analysis agree on:

- `ExhaustivenessResult`;
- arm `PatternUsefulness`;
- canonical `VariantId` candidates;
- branch proof semantics;
- stable bounded summaries/witnesses.

Add or extend the ownership-layer incremental ADT test instead of an LSP duplicate.

---

## Task 32 — Amend authoritative specifications

### Purpose

Make the root architecture explicit so a future refactor does not reintroduce eager recursion.

### Risk

- Semantic: MEDIUM
- Implementation fanout: docs

### Primary docs

1. `docs/impl/adt-gadt-associated-lookup/part-5/05.1-match-surface-pattern-semantics-exhaustiveness-gadt-proofs-technical-spec.md`
2. `docs/spec/next/phalcom-pattern-matching-spec.md`

### Required normative sections

Add rules equivalent to:

```text
PAT-COV-01 Demand-driven decomposition
PAT-COV-02 Closed-but-unopened vs open
PAT-COV-03 Finite source-pattern termination
PAT-COV-04 One constructor authority
PAT-GADT-01 Equality-producing decomposition
PAT-GADT-02 Fresh constructor locals
PAT-GADT-03 No canonical rigid leakage
PAT-GADT-04 Branch proof locality
PAT-EXH-01 No false proof on blocked/open state
PAT-EXH-02 Wildcard usefulness defines totality
PAT-WIT-01 Witnesses use the same domain/proof authority
```

Document that residual `PatternSpace` subtraction is no longer the correctness model.

The forward spec already states that the compiler should reason symbolically rather than eagerly enumerate large products; align the implementation spec with that rule.

---

## Task 33 — Complete migration/deletion/state gates

### Purpose

Prove old and new authorities cannot coexist silently.

### Risk

- Semantic: MEDIUM
- Implementation fanout: repository-wide search/docs

### Required negative searches

```bash
rg -n 'build_initial_pattern_space_inner|build_enum_or_opaque_space' \
  phalcom-semantic/src

rg -n 'remaining_space|evaluate_match_arm_usefulness|evaluate_match_exhaustiveness' \
  phalcom-semantic/src/checker

rg -n 'Multi-field Cartesian difference|flat\.contains\(&other\)' \
  phalcom-semantic/src/checker

rg -n 'CaseInstantiation::open' phalcom-semantic/src/checker

rg -n 'PatternSpace' phalcom-semantic/src
```

Expected:
- eager/residual authorities: zero production hits;
- `CaseInstantiation::open`: centralized shared authority plus justified low-level tests only;
- `PatternSpace`: zero internal proof type if deleted; `PatternSpaceSummary` may remain;
- any intentionally retained compatibility symbol is listed with exact justification.

### State file

Record all checkpoints, evidence, negative gates, deferred gates, decisions, rejected approaches, and final resume action.

If the repository still uses `graphify update .` for documentation graph maintenance, run it after docs changes and record the result. If unavailable or no longer policy, record that fact rather than inventing success.

---

## Task 34 — Run final broad delivery gates

### Purpose

Prove workspace compatibility after focused semantic evidence is already green.

### Risk

- Semantic: LOW as a gate
- Implementation fanout: workspace

### Commands

```bash
cargo +stable fmt --all -- --check
```

**Proves:** formatting consistency.

```bash
RUSTFLAGS='' cargo +stable check --workspace --all-targets
```

**Proves:** Rust API/caller compatibility across all workspace targets.

```bash
RUSTFLAGS='' cargo +stable test --workspace --all-targets
```

**Proves:** broad workspace regression compatibility.

```bash
RUSTFLAGS='' cargo +stable clippy --workspace --all-targets -- -D warnings
```

**Proves:** lint cleanliness under project-wide compilation.

These commands do **not** replace the focused semantic evidence from earlier checkpoints.

## Checkpoint completion

- [ ] focused Expression refinement passes
- [ ] full Expression subtree passes
- [ ] unified typing-integration package passes
- [ ] full semantic matching suite passes
- [ ] incremental ADT/match suite passes
- [ ] specs updated
- [ ] state file complete
- [ ] all negative gates pass
- [ ] format passes
- [ ] workspace check passes
- [ ] workspace tests pass
- [ ] clippy passes
- [ ] no active INCIDENT
- [ ] no deferred gate forgotten

### Suggested commits

```text
test(core): close recursive Expression typing integration
docs(semantic): specify demand-driven recursive match coverage
chore(semantic): complete coverage migration gates
```

---

# 16. Detailed Algorithm Notes for the Implementing Agent

## 16.1 Why the matrix must be driven by patterns

Consider the current outer-only arm:

```phalcom
Expression::Apply(function, argument)
```

The shared domain service may open `Apply<A,B>` against `Expression<F,T>` and derive a branch subject approximately equivalent to:

```text
function : Expression<F, α -> T>
argument : Expression<F, α>
```

The coverage pattern's two child nodes are both wildcard because source names bind arbitrary values.

The usefulness specialization therefore becomes:

```text
Apply(_, _)
→ [_, _]
→ covered
```

There is no semantic reason to ask:

```text
what constructors inhabit Expression<F, α -> T>?
```

and the new engine MUST NOT ask.

The index-growing chain is eliminated by architecture, not by a recursion guard.

## 16.2 Nested pattern example

For:

```phalcom
Expression::Add(
    Expression::IntLiteral(x),
    _
)
```

specialization does:

```text
Expression<F,T>
→ Add branch
→ columns:
     left  : Expression<F,Int>  pattern IntLiteral(_)
     right : Expression<F,Int>  pattern _
```

Only the left column requires another constructor split.

The right column stays universal/unopened.

## 16.3 GADT branch proof example

For:

```text
subject = Expr<T>
constructor = Int(_) -> Expr<Int>
```

constructor opening establishes `T = Int`.

That equality scopes to the constructor branch.

For concrete:

```text
subject = Expr<Bool>
```

the same constructor is refuted.

The usefulness engine must invoke the same constructor opening semantics as pattern resolution, not duplicate this equality logic.

## 16.4 Local constructor generics

For:

```phalcom
@variant
Wrap<U>(_ value: U) -> Expr<List<U>>
```

opening `Wrap` creates fresh rigid `ρ`.

The local result is:

```text
Expr<List<ρ>>
```

If the expected subject is generic `Expr<T>`, branch proof can relate:

```text
T ~ List<ρ>
```

The payload is locally:

```text
ρ
```

A second `Wrap` observation gets a fresh `ρ₂`.

Their shapes can be alpha-equivalent while identities remain distinct.

## 16.5 Closed-unopened vs open

The engine should conceptually treat:

```text
Closed(Expression<F,Int>)
```

as a subject that *can be decomposed later*.

It should treat:

```text
Open(Object)
```

as a domain whose complete constructors are not enumerable.

Both may render compactly as an opaque-looking public summary, but they are different internal semantic states.

---

# 17. Allocation and Performance Design Rules

These rules are mandatory implementation constraints, not optional cleanup.

## 17.1 Eliminate recursive universe allocation

No per-match object equivalent to:

```text
Union(
    Pure(...),
    Add(Expression(...), Expression(...)),
    If(Expression(...), ...)
)
```

may recursively contain the complete payload domains.

## 17.2 Avoid deep `Clone`

Coverage patterns are arena nodes referenced by IDs.

Matrix state copies IDs and small subject/proof handles, not recursive trees.

## 17.3 Avoid normalize-after-every-operation

The new algorithm should not have the old pattern:

```rust
result = result.union(&next).normalize();
```

inside recursive loops.

Matrix row specialization maintains normalized structural form by construction.

## 17.4 Avoid Cartesian residual products

Do not materialize:

```text
(A \ C) × B
∪
(A ∩ C) × (B \ D)
```

as a residual tree.

Usefulness recursively specializes columns and returns only the existence/witness needed by the semantic question.

## 17.5 Bounded witnesses

Retain `MAX_COVERAGE_WITNESSES = 8` or an equivalent established bound.

Do not make witness completeness require enumerating every residual.

## 17.6 Avoid structural O(n²) union dedupe

The old `flat.contains(&other)` disappears with residual union normalization.

Any remaining bounded summary dedupe should use small stable keys or arena IDs.

## 17.7 Preserve shared enum metadata

`EnumSemanticTable` already stores `Arc<EnumInfo>` / `Arc<VariantInfo>`.

Reuse those immutable products; do not clone/rebuild semantic enum declaration trees.

## 17.8 Reuse scratch buffers where clear

In hot specialization code:

- preallocate child vectors from constructor arity;
- preallocate specialized rows from known row count;
- where borrow rules remain clear, reuse scratch vectors between branches;
- do not introduce complex unsafe pooling.

## 17.9 Do not optimize by weakening proofs

Performance is not a reason to:
- erase local rigids;
- turn blocked into Dynamic;
- omit impossible-case checking;
- coarsen exact variant identity.

## 17.10 Cache conservatively

The root fix should be fast even with minimal caching.

Add caches only after:
- the cache key is semantically stable;
- freshness boundaries are explicit;
- tests defeat accidental existential sharing.

---

# 18. Migration of Existing Tests

## Keep and reuse

### `matching/exhaustiveness.rs`

Retain semantic tests for:
- full root coverage;
- missing witness;
- redundant arm;
- GADT impossible omission;
- exact cases;
- nested totality;
- nested missing witness;
- tuple product;
- list partition;
- open domain wildcard.

These prove language behavior, not the old algorithm.

### `matching/gadt_refinement.rs`

Retain all branch-local proof and local-rigid tests.

Especially:
- `match_gadt_12_variant_local_generic_opens_shared_rigid_and_keeps_index_proof`
- `match_gadt_13_variant_local_rigid_is_not_guessed_to_fit_concrete_index`

### `matching/patterns.rs`

Retain hostile record/map/open-domain tests and or-pattern behavior.

## Migrate/delete

### `matching/pattern_space.rs`

Direct tests of:
- normalize algebra;
- subtract identities;
- Cartesian residual construction;
- deep `PatternSpace` exact equality

are implementation tests for the obsolete authority.

Migrate their semantic intent to usefulness/witness tests, then delete direct algebra tests when `PatternSpace` leaves production.

Do not keep dead internal machinery solely to satisfy these tests.

---

# 19. Failure Protocol for This Program

If a checkpoint test fails, do not expand scope immediately.

Record:

## Exact reproduction

```text
command:
test:
important output:
```

## Direct path

Example:

```text
recursive_coverage fixture
→ expression match analysis
→ resolve_pattern
→ coverage classify
→ domain open variant
→ GADT proof
→ failed assertion
```

## Passing comparator

Find a nearby case that still works, e.g.:

```text
Tree recursion works; indexed Apply fails
```

or:

```text
canonical GADT works; constructor-local nested local subject fails
```

## Classification

Choose exactly one:

```text
PRODUCT
FIXTURE
DEPENDENCY/PUBLICATION
BACKEND/HARNESS
BASELINE
PLAN DRIFT
```

## Narrow repair boundary

State which module/symbol may change.

## Rejected broad repair

Record at least the tempting incorrect fix that must not be used.

A required checkpoint with failing evidence is:

```text
C<N> — INCIDENT
```

not “mostly complete.”

Later dependent checkpoints do not proceed.

---

# 20. Repository Drift Protocol

Before every checkpoint:

1. `git status --short`
2. verify primary files exist;
3. verify primary symbols still own expected responsibilities;
4. inspect diffs from completed checkpoints;
5. search new callers if a signature is about to change.

Adapt mechanics if needed.

Do not silently alter semantic design.

Examples:

Allowed:
- helper moved to a nearby file;
- signature needs an extra lifetime;
- enum uses a smallvec already available.

Escalate:
- someone reintroduced recursive universe construction;
- `CaseInstantiation` ownership changed;
- `MatchResolution` became a runtime lowering authority;
- coverage now crosses into parser/runtime unexpectedly.

---

# 21. State-File Protocol

After every checkpoint append/update the remediation section in:

`docs/impl/semantic/typing-integration/typing-integration-implementation-state.md`

Required structure:

```md
## Recursive Match Coverage Remediation

### Established invariants

- RC-I01: ...
- RC-I02: ...

### Decisions

- RC-D01: ...

### Evidence ledger

| Checkpoint | Command | Result | Proves |
|---|---|---|---|

### Negative/deletion evidence

- ...

### Deferred gates

- command → destination checkpoint

### Active incident

None.
```

or:

```md
### Active incident

C2 — INCIDENT

Reproduction:
...

Classification:
PRODUCT

Allowed repair boundary:
...

Rejected broad fixes:
...
```

Then:

```md
### Next resume action

Begin C3 Task 17.
```

Do not request or store chain-of-thought. Store only reviewable facts, evidence, and decisions.

---

# 22. Checkpoint Evidence Summary Template

The implementing agent should maintain this table during execution.

| Checkpoint | Semantic contract | Evidence | Status |
|---|---|---|---|
| C0 | incident/baseline safely locked | local state + semantic baselines + regression presence | NOT RUN |
| C1 | one GADT constructor opening authority | GADT/local-rigid tests | NOT RUN |
| C2 | demand-driven matrix usefulness is authoritative | recursive coverage + matching + focused Expression | NOT RUN |
| C3 | witnesses/summaries independent from residual tree | witness + fingerprint + incremental tests | NOT RUN |
| C4 | old PatternSpace authority deleted | negative searches + full matching | NOT RUN |
| C5 | resource bounded and performance hardened | budget + inhabitation + metrics | NOT RUN |
| C6 | real integration + docs + broad gates | Expression/full package/workspace gates | NOT RUN |

No status becomes COMPLETE without its checkpoint evidence.

---

# 23. Final Negative/Deletion Gates

Before declaring release complete, run all:

```bash
rg -n 'build_initial_pattern_space_inner|build_enum_or_opaque_space' \
  phalcom-semantic/src
```

Expected: zero.

```bash
rg -n 'remaining_space|evaluate_match_arm_usefulness|evaluate_match_exhaustiveness' \
  phalcom-semantic/src/checker
```

Expected: zero legacy residual authority.

```bash
rg -n 'Multi-field Cartesian difference|flat\.contains\(&other\)' \
  phalcom-semantic/src/checker
```

Expected: zero if `PatternSpace` implementation is removed.

```bash
rg -n 'CaseInstantiation::open' phalcom-semantic/src/checker
```

Expected: one shared semantic constructor-opening authority plus specifically justified tests/helpers; no duplicate pattern/exhaustiveness opening implementations.

```bash
rg -n 'PatternSpace' phalcom-semantic/src
```

Expected:
- no old internal proof algebra;
- `PatternSpaceSummary` remains if public compatibility requires it.

```bash
rg -n 'QueryBudget' phalcom-semantic/src/checker/coverage
```

Expected:
- no independently constructed/reset budget;
- coverage receives/uses `CheckerControl`.

Search ignored regression:

```bash
rg -n 'RED: eager recursive coverage universe expands indexed recursion' \
  phalcom-semantic/tests
```

Expected: zero by C6.

Any nonzero intentional occurrence must be documented individually.

---

# 24. Deferred-Evidence Audit

Before final completion:

```text
No deferred test/check remains unless it is:
1. executed successfully;
2. explicitly removed from scope with written justification; or
3. recorded as a release blocker.
```

Specifically audit:

- focused Expression refinement;
- full Expression subtree;
- unified typing integration;
- full semantic matching;
- incremental ADTs;
- semantic crate if separately scheduled;
- format;
- workspace check;
- workspace tests;
- clippy;
- project-specific graph/doc index update if required.

---

# 25. Recommended Staged Commit Groups

These are coherent suggestions, not mandatory one-task-per-commit rules.

## C0

```text
test(semantic): lock recursive match coverage regressions
docs(semantic): record recursive coverage remediation baseline
```

## C1

```text
refactor(semantic): centralize GADT constructor decomposition
test(semantic): enforce local-rigid constructor observation
```

## C2

```text
feat(semantic): add finite coverage pattern representation
fix(semantic): use matrix usefulness for match coverage
test(semantic): cover recursive indexed match termination
```

## C3

```text
refactor(semantic): generate coverage witnesses from usefulness
refactor(semantic): bound match coverage semantic summaries
test(semantic): preserve incremental match products
```

## C4

```text
refactor(semantic): remove residual PatternSpace proof authority
perf(semantic): eliminate recursive coverage allocation hot path
```

## C5

```text
feat(semantic): add productive recursive inhabitation
perf(semantic): budget and memoize safe coverage work
test(semantic): assert recursive coverage work bounds
```

## C6

```text
test(core): close recursive Expression typing integration
docs(semantic): specify demand-driven pattern coverage
chore(semantic): complete recursive coverage migration gates
```

---

# 26. Known Scope Exclusions

This implementation program does **not** include:

- new pattern syntax;
- guard syntax design (`if` vs `when`);
- runtime pattern execution redesign unless a concrete regression proves a dependency;
- bytecode format changes;
- enum runtime representation changes;
- reflection surface changes;
- LSP-specific coverage recomputation;
- exhaustiveness completion UI;
- rank polymorphism;
- changes to `Expression` language design;
- generic getter/setter work;
- parser changes unrelated to existing legal recursive pattern fixtures;
- general TypeStore interning redesign;
- global persistent coverage caches.

If implementation appears to require one of these, stop and classify as PLAN DRIFT or a newly discovered dependency before editing.

---

# 27. Release-Complete Criteria

The remediation is complete only when all of the following are true:

- [ ] C0–C6 are all `COMPLETE`;
- [ ] no checkpoint is `INCIDENT`;
- [ ] the indexed `Apply` recursive coverage regression passes without an ignore;
- [ ] `ExpressionEvaluation.eval` no longer triggers recursive payload universe expansion;
- [ ] source-pattern nesting still drives correct finite nested decomposition;
- [ ] existing GADT impossible-case tests pass;
- [ ] existing constructor-local rigid tests pass;
- [ ] independent constructor observations retain fresh rigids;
- [ ] open record/map/object hostile cases remain conservative;
- [ ] or-pattern redundancy uses the same usefulness authority;
- [ ] missing witnesses come from the new symbolic engine;
- [ ] `ExhaustivenessResult::Blocked` cannot be transformed into `Proven`;
- [ ] old eager universe builders are deleted;
- [ ] old Cartesian residual hot path is deleted;
- [ ] no independent coverage budget exists;
- [ ] TypeStore growth for outer-only indexed recursion is bounded;
- [ ] operation metrics show recursive payload decomposition is source-driven;
- [ ] incremental match products agree with cold products;
- [ ] full `typing_integration::expression` passes;
- [ ] full `typing_integration::` passes;
- [ ] full semantic matching suite passes;
- [ ] all negative/deletion gates pass;
- [ ] authoritative specs are updated;
- [ ] implementation state contains final evidence;
- [ ] format/check/test/clippy final gates pass;
- [ ] no deferred evidence remains forgotten.

---

# 28. Final Implementation Principle

The final architecture should make this statement true by construction:

> **Phalcom does not prove match exhaustiveness by recursively enumerating the inhabitants of a type. It proves usefulness and totality by finitely specializing source patterns against one-layer constructor domains. Recursive payloads remain typed, closed subjects until the source pattern explicitly inspects them. GADT constructor observation is equality-producing, branch-local, and freshly opens constructor-local generic binders.**

For the real `Expression<F,T>` evaluator, `Apply` therefore behaves as:

```text
Expression<F,T>
→ open Apply once
→ fields:
     Expression<F, α -> T>
     Expression<F, α>
→ source children are bindings
→ stop
```

There is no:

```text
Expression<F, α -> T>
→ Expression<F, β -> (α -> T)>
→ ...
```

because type recursion no longer drives coverage recursion.

That is the root fix, the termination argument, and the architecture this plan requires the implementation to preserve.
