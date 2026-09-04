# Phalcom Unified Typing Integration — Record Rows Amendment
## Repository-Grounded, Checkpoint-Driven, Patch-Grade Implementation Plan

**Repository:** `aureat/phalcom-lang`  
**Program:** extend the existing unified `typing_integration` conformance package with a dedicated classic row-polymorphism test suite and then make structural Record payloads participate in the existing Either, Monad/HKT, collection, and Expression/GADT flagship scenarios.  
**Plan type:** amendment to the existing Unified Typing Integration implementation program; this document does not replay or replace its completed migration work.  
**Prepared against remote branch:** `main`  
**Prepared against exact remote HEAD:** `e17f2733f98cb20e2a8ead5794d75ca647a950ce`  
**HEAD subject:** `feat: consolidate typing integration and runtime bootstrap`  
**Repository inspection mode:** remote GitHub repository inspection only. The local working tree was unavailable, so local uncommitted changes are unknown and must be checked before editing.  
**Plan execution state:** no implementation commands or tests were run while preparing this amendment.

Relevant recent commits:

```text
e17f2733f98cb20e2a8ead5794d75ca647a950ce
    feat: consolidate typing integration and runtime bootstrap

47abba0e5b44d091768748420fd21dd91ae43742
    feat(vm): add canonical universe bootstrap tiers

9f04681201e4e15388b4a32d09a2a502486e9367
    feat: extend semantic type-system closure

a37664e17e5e9f31378b7d497e51ad349d5ba905
    chore(semantic): record SC-3 Task 14 sign-off

8a7023fc432a74be94fbea1b9b1b7e572b44e81d
    feat(semantic): implement open record row typing
```

The original unified typing-integration plan established the package as a single cross-feature test ecosystem with one shared fixture authority and focused sub-suites. fileciteturn1file0 This amendment preserves that architecture.

The SC-3 implementation plan remains authoritative for the mechanism-level Record-row semantics that these tests consume: `RecordRow` is a distinct kind/domain, canonical Records are `TypeData::Record(RecordRowId)`, row solver variables are query-local, repeated stable row parameters correlate within an instantiation, Records use immutable width plus covariant-depth subtyping, and underconstrained row variables do not default to the empty row. fileciteturn0file0

---

# 1. Executive requirements analysis

## 1.1 What the repository already contains

The current `phalcom-core/tests/core/typing_integration/` package is already live and registered as the sole top-level typing integration module from `phalcom-core/tests/core/mod.rs`. Its root currently contains:

```text
typing_integration/
├── LAWS.md
├── README.md
├── either/
├── expression/
├── integration.rs
├── mod.rs
├── monads/
├── sources/
└── support.rs
```

The root module currently registers:

```rust
mod support;
mod either;
mod monads;
mod integration;
mod expression;
```

so this amendment should extend that established package instead of creating another Cargo target or sibling conformance universe.

The current package README already defines `typing_integration` as a user-level proving ground for generic ADTs, constructor-kinded generics, type lambdas, generic inheritance, GADT refinement, inference provenance, and VM execution. Rows fit that exact architectural purpose.

The package's current law catalog has stable `GEN-*`, `MON-*`, `GEX-*`, and `INT-*` families. The amendment adds a fifth major family, `ROW-*`, plus cross-feature `INT-ROW-*` laws. Existing law identifiers must not be renumbered.

## 1.2 What SC-3 already owns

SC-3 is not merely a plan on this HEAD. The production repository already contains:

```text
phalcom-semantic/src/checker/row_inference.rs
    GenericApplicationSession
    GenericInferenceBinding::{Type, RecordRow}
    InferenceRecord
    InferenceRecordTail
    CombinedInferenceFailure

phalcom-semantic/src/types/row.rs
    RecordRowField
    RecordRowTail
    RecordRowData
    RecordRowFormationError

phalcom-semantic/src/types/row_solver.rs
    RecordRowVarId
    RecordRowTerm
    RecordRowSolution
    RecordRowSolver
    RecordRowSolveResult

phalcom-semantic/src/checker/call.rs
    generic call integration with a separate row session
    canonical return materialization with row bindings
```

`GenericApplicationSession` allocates an ordinary `InferVarId` for type-domain binders and a distinct `RecordRowVarId` for `RecordRow` binders. It explicitly coordinates the two solvers without turning a row into a proper type.

The row solver itself operates over normalized query-local row terms and canonicalizes only successful solutions.

The ownership-level semantic suite already covers direct row algebra and source-level behavior. `phalcom-semantic/tests/semantic/advanced/record_rows.rs` contains canonicalization, subtraction, lacks, occurs-check, row-domain safety, immutable width subtyping, history independence, aliasing, and terminal-state tests.

`phalcom-semantic/tests/semantic/integration/record_row_polymorphism.rs` already covers source-reachable remainder preservation, combined type/row inference, repeated-row conflicts, contextual row inference, underconstraint, open-row patterns, row diagnostics, and solver-ID non-leakage.

Therefore this amendment must **not** duplicate those mechanisms in `phalcom-core`.

## 1.3 The correct new ownership boundary

The division is:

```text
phalcom-semantic
    owns correctness of the row machinery itself

typing_integration::rows
    owns sophisticated user-language programs whose correctness
    depends on that machinery

typing_integration cross-feature tests
    own composition of rows with independent typing mechanisms
```

Examples of mechanism tests that remain in `phalcom-semantic`:

```text
row canonicalization
duplicate formation
solver history independence
lacks propagation through aliases
direct/indirect occurs checks
budget and cancellation
query-local row variable isolation
metadata fingerprinting
incremental invalidation
scoped row lowering
```

Examples of new `typing_integration` responsibilities:

```text
classic extensible-record calculus
same-remainder correlation
higher-order row-preserving functions
multi-stage row decomposition
realistic Reader/environment capability typing
nominal-vs-structural boundaries
Map-vs-Record boundaries
open-row pattern use
Either<Record> transformations
Monad/HKT algorithms over Record payloads
List<Record> nested specialization
Record-valued GADT result indices
row-specialized GADT T carried through F<T>
full-stack Record payload runtime scenarios
```

The governing test doctrine is:

> Ownership tests prove each mechanism. `typing_integration` proves that independently implemented mechanisms compose.

## 1.4 One important observability constraint

The existing shared `support.rs` has excellent exact generic-solution support for ordinary type parameters. It inspects `ExplanationStep::GenericSolution` using stable `TypeParameterId` and `TypeId`.

The current generic-call path publishes those ordinary solutions after `InferenceSession::projected_solution`. Row solutions are solved by the separate `GenericApplicationSession`; they are used to construct a `GenericInstantiation` and materialize the canonical return Record, but the current explanation path does not publish a corresponding `RecordRowId` as an ordinary `GenericSolution`.

Therefore the amendment must **not** invent this invalid helper:

```rust
// FORBIDDEN
fn infer_expected_remainder(actual: TypeId, prefix: TypeId) -> RecordRowId
```

and must not reinterpret `RecordRowId` as `TypeId`.

For this amendment, exact row evidence is obtained through:

```text
stable row binder identity
    TypeParameterId whose kind == RecordRow

+

canonical specialized output
    TypeData::Record(RecordRowId)

+

canonical RecordRowData
    exact known fields
    exact closed/open tail

+

call success/conflict status and diagnostic
```

Tests that need to observe `R` must deliberately expose `R` in their specialized result type.

A future stable `GenericRowSolution` explanation product could be useful, but adding one is a separate observability feature and is not required for this testing amendment.

## 1.5 Existing Expression baseline requires reconciliation

The live repository contains:

```text
typing_integration/expression/
sources/expression.ph
sources/expression_semantic_probes.ph
sources/expression_runtime_probes.ph
```

and current Expression source already includes `Expression<F,T>`, constructor-local GADT variables, `ExpressionEvaluation`, and `ExpressionMonad<F>`.

However, the existing `typing-integration-implementation-state.md` still records an earlier C4 incident involving parser rejection of constructor-local variant generics and typed closure parameters.

That ledger is therefore older than the live source topology or otherwise unresolved relative to HEAD. This amendment must not guess whether Expression is currently green.

Checkpoint R0 explicitly reruns the live package and classifies the current Expression state before record-valued GADT work begins.

Isolated row work and the Either/Monad row composition checkpoints may proceed if Expression alone has a pre-existing baseline incident. The Expression-row checkpoint may not.

---

# 2. Target package architecture

The final intended topology is:

```text
phalcom-core/tests/core/typing_integration/
├── mod.rs
├── support.rs
├── README.md
├── LAWS.md
├── integration.rs
├── row_integration.rs
│
├── sources/
│   ├── either.ph
│   ├── monads.ph
│   ├── expression.ph
│   ├── ...
│   │
│   ├── rows/
│   │   ├── core.ph
│   │   ├── calculus.ph
│   │   ├── transformations.ph
│   │   ├── pipelines.ph
│   │   ├── structural_protocols.ph
│   │   ├── patterns.ph
│   │   ├── runtime.ph
│   │   └── invalid/
│   │       ├── repeated_remainder_conflict.ph
│   │       ├── duplicate_extension.ph
│   │       ├── nominal_is_not_record.ph
│   │       └── map_is_not_record.ph
│   │
│   ├── row_integration_probes.ph
│   ├── expression_row_probes.ph
│   └── expression_row_runtime_probes.ph
│
├── either/
│   └── ... unchanged existing suite
│
├── monads/
│   └── ... unchanged existing suite
│
├── rows/
│   ├── mod.rs
│   ├── calculus.rs
│   ├── correlation.rs
│   ├── transformations.rs
│   ├── pipelines.rs
│   ├── structural.rs
│   ├── patterns.rs
│   ├── rejection.rs
│   └── runtime.rs
│
└── expression/
    ├── ... existing files
    └── rows.rs
```

No `rows/support.rs` is permitted.

The one shared fixture authority remains:

```text
typing_integration/support.rs
```

The reusable source authority for row-specific user-level helpers becomes:

```text
typing_integration/sources/rows/core.ph
```

Cross-feature sources consume that file instead of copying its declarations.

---

# 3. Proposed row test language surface

The reusable source should be deliberately small. It is not a row standard library. It is a pressure vessel for the language.

Use a class such as:

```phalcom
class RowCalculus {
    @class
    preserve<R: RecordRow>(
        _ value: #{ name: String, | R }
    ) -> #{ name: String, | R } {
        value
    }

    @class
    preserveValue<T, R: RecordRow>(
        _ value: #{ value: T, | R }
    ) -> #{ value: T, | R } {
        value
    }

    @class
    annotate<A, B, R: RecordRow>(
        _ value: #{ value: A, | R },
        _ transform: (A) -> B
    ) -> #{ value: A, mapped: B, | R } {
        #{
            **value,
            mapped: transform.call(value.value)
        }
    }

    @class
    tagged<R: RecordRow>(
        _ value: #{ name: String, | R }
    ) -> #{ name: String, tag: String, | R } {
        #{
            **value,
            tag: "entity"
        }
    }

    @class
    sameRemainder<R: RecordRow>(
        _ left: #{ id: Int, | R },
        _ right: #{ id: Int, | R }
    ) -> #{ id: Int, | R } {
        left
    }

    @class
    consumeTagged<R: RecordRow>(
        _ value: #{ tag: String, | R }
    ) -> #{ tag: String, | R } {
        value
    }
}
```

This is **STRUCTURAL**, not guaranteed paste-ready. Reconcile formatting and Record literal expansion syntax with the landed parser at implementation time.

One previous candidate design is deliberately rejected:

```phalcom
transformValue(
    #{ value: A, | R }
) -> #{ value: B, | R }
```

implemented as:

```phalcom
#{
    **value,
    value: transformed
}
```

That would attempt to introduce a duplicate known `value` field through expansion. The SC-3 model treats known fields and row tails as disjoint and uses duplicate-safe checked row formation. The integration fixture should test legal extension, not rely on overwrite semantics that Records do not promise.

`annotate` therefore retains `value: A` and adds a new disjoint `mapped: B`.

---

# 4. Law catalog amendment

Add the following families to `typing_integration/LAWS.md`.

## `ROW-INFER-*` — row and ordinary generic inference

### ROW-INFER-01 — remainder inference preserves extra fields

Given:

```phalcom
RowCalculus.preserve(#{
    name: "Phalcom",
    version: 1,
    stable: true
})
```

the specialized return must be exactly:

```text
#{
    name: String,
    stable: Bool,
    version: Int
}
```

with a closed canonical row.

### ROW-INFER-02 — ordinary and row variables solve together

For:

```phalcom
RowCalculus.preserveValue(#{
    value: 42,
    label: "answer",
    cached: true
})
```

the result proves:

```text
T = Int

R contributes:
    cached: Bool
    label: String
```

through the exact specialized Record.

### ROW-INFER-03 — higher-order A/B inference composes with R

For `RowCalculus.annotate`:

```text
A = Int
B = Bool

R contributes:
    cached: Bool
    name: String
```

and the specialized result is:

```text
#{
    cached: Bool,
    mapped: Bool,
    name: String,
    value: Int
}
```

### ROW-INFER-04 — proven empty remainder is legal

Calling `preserve` with exactly the known prefix yields a closed one-field Record.

This is proof of an empty remainder, not a default.

### ROW-INFER-05 — result-only row inference remains underconstrained

A helper analogous to:

```phalcom
make<R: RecordRow>() -> #{ value: Int, | R }
```

without expected context must remain underconstrained.

### ROW-INFER-06 — expected result may select an otherwise underconstrained row

The same helper may specialize when an exact expected Record supplies the missing row information.

## `ROW-CORR-*` — stable remainder correlation

### ROW-CORR-01 — repeated stable `R` denotes one row

Two compatible actual Records passed to `sameRemainder<R>` must produce one consistent specialized remainder.

### ROW-CORR-02 — incompatible repeated remainders conflict

Different extra-field sets for the same `R` must reject rather than widen or become `Dynamic`.

## `ROW-XFORM-*` — immutable extension and higher-order transformation

### ROW-XFORM-01 — higher-order annotation preserves R

Adding a derived `mapped: B` field preserves every unrelated remainder field.

### ROW-XFORM-02 — extension preserves an inferred remainder

`tagged` adds `tag: String` while preserving the caller's extra fields.

### ROW-XFORM-03 — extension cannot collide with R

If the actual remainder already contains `tag`, materialization must fail closed.

## `ROW-PIPE-*` — repeated decomposition of canonicalized results

### ROW-PIPE-01 — each generic call decomposes the canonical value afresh

Example:

```text
input
    #{ name, age, enabled }

tagged input
    formal #{ name | R1 }
    R1 = #{ age, enabled }

result
    #{ name, tag, age, enabled }

consumeTagged(result)
    formal #{ tag | R2 }
    R2 = #{ name, age, enabled }
```

`R2` is not the old `R1`.

This protects against accidentally attaching solver-local row identity to values instead of publishing canonical Record types.

## `ROW-REL-*` — realistic immutable structural typing

### ROW-REL-01 — immutable width

A wider Record satisfies a narrower required prefix.

### ROW-REL-02 — covariant structural depth

A wider nested immutable Record may satisfy a narrower nested field requirement.

### ROW-REL-03 — width and depth compose

Use a realistic environment/capability object:

```text
#{
    config: #{
        port: Int,
        host: String,
        debug: Bool
    },
    cache: String,
    requestId: String
}
```

against:

```text
#{
    config: #{
        port: Int
    },
    | R
}
```

### ROW-REL-04 — nominal class shape is not Record evidence

A nominal class exposing equivalent member names is not structurally converted to a Record.

### ROW-REL-05 — Map key sets are not Record rows

`Map<K,V>` remains a dynamic-key mutable collection and does not satisfy a Record requirement based on possible runtime keys.

## `ROW-PATTERN-*` — open Record decomposition

### ROW-PATTERN-01 — known prefix fields decompose precisely

Fields in the open Record's statically known prefix receive their exact type.

### ROW-PATTERN-02 — a tail possibility is not a guarantee

A field absent from the known prefix but potentially present in `R` must not acquire fabricated precise evidence.

## `ROW-REJECT-*` — fail-closed language-level hostile cases

At minimum:

```text
ROW-REJECT-01 repeated remainder conflict
ROW-REJECT-02 extension collision/lacks failure
ROW-REJECT-03 nominal class is not Record
ROW-REJECT-04 Map is not Record
```

Every critical rejected expression must be `Invalid`/formally blocked as appropriate and must not become `Dynamic`.

## `ROW-RUNTIME-*` — static/runtime correspondence

### ROW-RUNTIME-01 — preserved/extended Record executes after semantic preflight

### ROW-RUNTIME-02 — multi-stage row pipeline executes with expected primitive observations

Runtime tests do not inspect row solver data. They prove that row-specialized source compiles and the ordinary Record runtime representation executes.

---

# 5. Cross-feature law amendment

Add:

## `INT-ROW-01` — direct Either transformation over a Record payload

Start from:

```text
Either<
    String,
    #{
        cached: Bool,
        name: String,
        value: Int
    }
>
```

Use direct `Either.map` and `RowCalculus.annotate`.

Expected:

```text
Either<
    String,
    #{
        cached: Bool,
        mapped: Bool,
        name: String,
        value: Int
    }
>
```

Assert:

```text
exact Either declaration identity
exact Either.map callable
exact nested Record type
exact RowCalculus.annotate callable
Ready analysis status
no Dynamic/Unknown escape
```

## `INT-ROW-02` — Monad/HKT inference over Record A/B

Use:

```text
F = <X> =>> Either<String, X>
```

and `MonadAlgorithms.bind`.

Require:

```text
A = input Record
B = output Record
F = canonical unary Either<String,_> lambda
```

while the continuation itself performs row-polymorphic inference.

## `INT-ROW-03` — nested `List<Record>` survives generic traversal

Use existing `MonadAlgorithms.traverse` with:

```text
List<InputRecord>
    ->
Either<String, List<OutputRecord>>
```

This checks Record specialization beneath both `List<_>` and `Either<_,_>` rather than only at the top level.

## `INT-ROW-04` — GADT result index may be a Record

Use:

```text
Expression<
    <X> =>> Either<String, X>,
    InputRecord
>
```

and map through a row-polymorphic transformation to:

```text
Expression<
    <X> =>> Either<String, X>,
    OutputRecord
>
```

## `INT-ROW-05` — row-specialized GADT `T` survives `F<T>`

Evaluation must yield:

```text
Either<String, OutputRecord>
```

with the exact same canonical output Record nested under `Either`.

## `INT-ROW-06` — full-stack Record flagship

The strongest scenario is:

```text
List<
    #{
        cached: Bool,
        name: String,
        value: Int
    }
>
        |
        | MonadAlgorithms.traverse
        | using StringEitherExpressionMonad
        | each element transformed via RowCalculus.annotate
        v

Expression<
    <X> =>> Either<String, X>,
    List<
        #{
            cached: Bool,
            mapped: Bool,
            name: String,
            value: Int
        }
    >
>
        |
        | ExpressionEvaluation.eval
        | using StringEitherMonad
        v

Either<
    String,
    List<
        #{
            cached: Bool,
            mapped: Bool,
            name: String,
            value: Int
        }
    >
>
```

The existing scalar flagship remains. This is a second, stronger level.

---

# 6. Source-of-truth matrix

| Concern | Source of truth | Consumers | Forbidden competing authority |
|---|---|---|---|
| Row binder domain | `TypeParameterId` + `KindId::RECORD_ROW` | call inference, test assertions | treating `R` as ordinary `TypeData::Parameter` |
| Canonical Record type | `TypeData::Record(RecordRowId)` | row tests, nested Either/List/Expression assertions | display strings |
| Canonical row structure | `TypeStore::record_row` → `RecordRowData` | shared test helpers | test-local record parser |
| Query-local row solving | `GenericApplicationSession` + `RecordRowSolver` | semantic call application | test-local solver |
| Structural relation | `phalcom-semantic` type relation | `ROW-REL-*` scenarios | Rust field-name comparison as a substitute for semantic checking |
| Row-call specialization | canonical call analysis/result type | `ROW-INFER-*`, `ROW-CORR-*` | reconstructed substitution map |
| Shared row helper source | `sources/rows/core.ph` | isolated and cross-feature fixtures | copied `RowCalculus` declarations |
| Generic ADT | existing `sources/either.ph` | `INT-ROW-*` | row-local Either copy |
| Monad/HKT | existing `sources/monads.ph` | `INT-ROW-*` | row-local Monad copy |
| GADT | existing `sources/expression.ph` | Expression row tests | row-local Expression copy |
| Runtime | `ProgramCompiler` + `VM` | row runtime tests | Rust emulation of Record/Either semantics |

---

# 7. Tempting wrong fixes

Do not take these shortcuts.

1. **Do not create `phalcom-core/tests/core/rows` as another top-level package.** Rows belong in the existing typing composition ecosystem.

2. **Do not create `rows/support.rs`.** `typing_integration/support.rs` remains the only fixture authority.

3. **Do not import or instantiate `RecordRowSolver` in `phalcom-core` tests.** Solver behavior belongs to `phalcom-semantic`.

4. **Do not calculate the expected remainder in Rust.** The compiler's canonical specialized Record is the evidence.

5. **Do not encode a row solution as `TypeId`.** `RecordRow` is a distinct domain.

6. **Do not weaken row conflicts to `Dynamic`.**

7. **Do not wrap Record payloads in nominal classes to make Either/Monad/Expression tests easier.** The whole purpose is to preserve structural Record typing through the stack.

8. **Do not replace Records with `Map<K,V>`.**

9. **Do not change basic Either/Monad/Expression builders to always load row fixtures.** Fixture isolation is an existing package invariant.

10. **Do not create a default `all_typing_source()` mega-fixture.**

11. **Do not modify SC-3 production inference because a new integration test fails until the Failure Protocol identifies a product defect.**

12. **Do not add row-valued nominal generic arguments or row-kinded type-lambda application.** Those remain outside SC-3's claimed scope.

13. **Do not replace the existing scalar Expression flagship with the Record version.** Keep both; their differential failure behavior is valuable.

14. **Do not add a test-only `GenericRowSolution` abstraction that merely reconstructs the solver result.**

---

# 8. Checkpoint map

| Checkpoint | Tasks | Semantic boundary | Required evidence | Deferred evidence |
|---|---:|---|---|---|
| R0 | 1–2 | Live typing-integration and SC-3 baseline is classified before amendment work | Git state; focused row semantic suites; existing typing integration baseline; Expression baseline | no new row tests |
| R1 | 3–5 | One isolated row source/test architecture and canonical Record inspection surface are established | first `ROW-INFER-*` laws; row source isolation search | transformation/correlation/structural/runtime |
| R2 | 6–8 | Complex row inference, correlation, extension, and re-decomposition work as one language-level row calculus | complete calculus/correlation/transformation/pipeline suites; hostile repeated-R/extension cases | structural/pattern/runtime; cross-feature |
| R3 | 9–11 | Structural Record boundaries, open-pattern behavior, and VM execution are covered without duplicating solver tests | structural hostile cases; pattern tests; row runtime semantic preflight + execution; full `rows::` suite | Either/Monad/GADT composition |
| R4 | 12–14 | Record payloads compose with direct Either, HKT Monad inference, and nested collections | `INT-ROW-01..03`; exact generic/callable assertions; full row cross-feature module | Expression/GADT |
| R5 | 15–17 | Record payloads survive GADT indexing, `F<T>`, ExpressionMonad traversal, and the full-stack flagship | existing Expression baseline; `INT-ROW-04..06`; hostile cross-domain case; runtime flagship | workspace delivery gates |
| R6 | 18–19 | Law/docs/state migration is complete and no parallel row-testing authority remains | full `typing_integration::`; negative gates; core target; final broad gates | none |

---

# 9. Checkpoint R0 — Baseline, Drift, and SC-3 Prerequisite Classification

Tasks:

- Task 1 — establish local execution state and reconcile plan drift
- Task 2 — certify focused SC-3 and current typing-integration prerequisites

Why this is a checkpoint:

The remote repository has moved substantially beyond the original plan baseline. In particular, the unified package is landed, SC-3 focused implementation is landed, and the current state file contains an Expression incident that may be stale relative to the live tree.

No row-test amendment should begin until the implementing checkout says what is actually green.

Entry conditions:

- local repository checkout exists;
- no amendment edit has started;
- remote-plan baseline is known as `e17f2733...`.

Working set:

Primary:

- `phalcom-core/tests/core/typing_integration/mod.rs`
- `phalcom-core/tests/core/typing_integration/support.rs`
- `phalcom-core/tests/core/typing_integration/README.md`
- `phalcom-core/tests/core/typing_integration/LAWS.md`
- `phalcom-core/tests/core/typing_integration/expression/`
- `phalcom-semantic/tests/semantic/advanced/record_rows.rs`
- `phalcom-semantic/tests/semantic/integration/record_row_polymorphism.rs`
- `docs/impl/semantic/typing-integration/typing-integration-implementation-state.md`
- `docs/impl/semantic/semantic-completeness/sc-3/SC-3-implementation-state.md`

Secondary — inspect only if evidence requires it:

- `phalcom-semantic/src/checker/row_inference.rs`
- `phalcom-semantic/src/checker/call.rs`
- parser/AST only if the current Expression baseline reproduces the historical parser incident.

Out of scope:

- new source fixtures;
- row helper implementation;
- production semantic fixes.

Semantic contract established:

- current local baseline is known;
- SC-3 focused row semantics needed by the amendment are green or explicitly classified;
- current Expression baseline is known;
- stale state-ledger claims are separated from current evidence.

Semantic risks:

- treating historical state as current truth;
- misclassifying an existing Expression failure as a row regression;
- building on local uncommitted semantic changes without recording them.

Hostile cases:

- SC-3 focused row suite fails before amendment edits;
- full typing integration is already red before rows;
- Expression alone is red while Either/Monad are green.

Required evidence:

```bash
git rev-parse --show-toplevel
git branch --show-current
git rev-parse HEAD
git status --short
git log -8 --oneline -- \
  phalcom-core/tests/core/typing_integration \
  phalcom-semantic/src/checker/row_inference.rs \
  phalcom-semantic/src/types/row_solver.rs
```

Proves local branch/HEAD/working-tree conditions.

Then:

```bash
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic \
  semantic::advanced::record_rows -- --nocapture
```

Proves the low-level row domain is green.

```bash
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic \
  semantic::integration::record_row_polymorphism -- --nocapture
```

Proves source-reachable row-polymorphic inference/pattern/diagnostic behavior is green.

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::either:: -- --nocapture

RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::monads:: -- --nocapture
```

Proves the non-GADT foundation remains stable.

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::expression:: -- --nocapture
```

Determines whether the state ledger's old C4 incident still applies.

Finally:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration:: -- --nocapture
```

Records the complete pre-amendment package baseline.

Do not run yet:

```bash
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

These add no new R0 evidence and the repository already has historical unrelated workspace blockers.

Escalate immediately if:

- focused SC-3 row semantic tests fail;
- row source syntax no longer matches the assumed `#{ ..., | R }` form;
- `RecordRow` generic call inference has been removed/reworked;
- a local change already introduces a separate row integration package.

Special baseline rule:

If:

```text
Either       green
Monad        green
SC-3 rows    green
Expression   baseline red
```

classify Expression as a `BASELINE` incident.

R1–R4 may proceed because they do not depend on Expression.

R5 is blocked until the Expression baseline is resolved.

Checkpoint completion:

- [ ] local HEAD/status recorded
- [ ] focused SC-3 row suites pass
- [ ] Either baseline recorded
- [ ] Monad baseline recorded
- [ ] Expression baseline classified
- [ ] complete package baseline recorded
- [ ] state file reconciled
- [ ] no row-specific active incident remains

Suggested commit grouping:

No implementation commit required.

---

### Task 1 — Establish Local Execution State

Purpose:

Anchor the amendment to the implementing checkout.

Risk:

- Semantic: LOW
- Implementation fanout: local

Owned files and symbols:

- no source edit

Inspect before editing:

- paths in R0 working set only.

Dependencies:

- none.

Source of truth:

- local Git checkout.

Implementation boundary:

Changes:

- record facts only.

Must not:

- reset unrelated local work;
- silently switch branch;
- assume remote HEAD equals local HEAD.

Current implementation:

Plan prepared remotely against `main@e17f2733...`.

Target implementation:

Recorded local state plus a drift classification.

Edit operations:

1. Run the Git commands above.
2. Compare local HEAD to `e17f2733...`.
3. If drift exists, inspect only relevant changed paths.
4. Record mechanical adaptations.
5. Do not alter semantic design to fit drift.

Testing classification:

No standalone test. Evidence belongs to R0.

Checkpoint state update:

Record branch, HEAD, dirty relevant paths, and drift.

---

### Task 2 — Reconcile Existing State and Prerequisite Evidence

Purpose:

Determine which existing semantic layers are safe foundations for the amendment.

Risk:

- Semantic: MEDIUM
- Implementation fanout: package-wide evidence

Owned files and symbols:

- `docs/impl/semantic/typing-integration/typing-integration-implementation-state.md`

Inspect before editing:

- existing `Active incident`
- current Expression test modules
- SC-3 state ledger.

Source of truth:

Actual focused test commands, not the older state prose.

Implementation boundary:

Changes:

- append a Record Rows Amendment baseline section;
- if the old Expression incident is demonstrably resolved, move it to historical/resolved status rather than deleting its evidence.

Must not:

- claim Expression is green without running it;
- mark SC-3 repository release-complete merely because focused row tests pass.

Testing classification:

R0 required evidence.

Checkpoint state update:

Record exact test counts and current Expression status.

---

# 10. Checkpoint R1 — Isolated Row Test Authority and Core Inference Calculus

Tasks:

- Task 3 — create row source/test topology
- Task 4 — extend shared fixture with canonical Record inspection/builders
- Task 5 — add core row calculus and first inference laws

Why this is a checkpoint:

Source ownership, fixture inspection, and the first row programs only become meaningful together. A standalone directory move or helper extraction proves nothing.

Entry conditions:

- R0 COMPLETE for SC-3/Either/Monad;
- source-level row-polymorphic call tests are green.

Working set:

Primary:

- `phalcom-core/tests/core/typing_integration/mod.rs`
- `phalcom-core/tests/core/typing_integration/support.rs`
- new `typing_integration/rows/`
- new `typing_integration/sources/rows/`

Secondary:

- `phalcom-semantic/src/types/row.rs`
- `phalcom-semantic/src/types/store.rs` for public inspection methods only.

Out of scope:

- Either/Monad source;
- Expression source;
- row solver internals;
- metadata/incrementality.

Semantic contract established:

- one shared user-level row calculus source exists;
- one shared fixture can assert canonical Record structure without solving rows itself;
- basic R and T+R inference is proven through real calls.

Semantic risks:

- accidentally turning test helper logic into a second type checker;
- comparing only field display strings;
- losing canonical tail checks;
- accidentally loading rows into every existing fixture.

Hostile cases:

- result has right field names but wrong `TypeId`;
- row result remains open instead of closed;
- underconstrained result silently closes.

Required evidence:

Focused first:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::rows::calculus:: -- --nocapture
```

Then:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::rows:: -- --nocapture
```

Negative source-authority search:

```bash
rg -n 'class RowCalculus' \
  phalcom-core/tests/core/typing_integration
```

Expected exactly one declaration:

```text
sources/rows/core.ph
```

Do not run yet:

- Either/Monad cross-feature tests;
- Expression;
- full core target.

Escalate immediately if:

- canonical result Record cannot be inspected through existing public semantic snapshot APIs;
- a helper seems to require `RecordRowVarId`;
- row tests require production code edits before the first semantic law can be expressed.

Checkpoint completion:

- [ ] Tasks 3–5 complete
- [ ] one row source authority
- [ ] canonical Record helper assertions exist
- [ ] ROW-INFER-01..06 covered as planned
- [ ] hostile underconstraint behavior passes
- [ ] no solver-local API enters core test support
- [ ] state updated
- [ ] no active incident

Suggested commit:

```text
test(core): add isolated record-row typing calculus
```

---

### Task 3 — Create the Row Source/Test Topology

Purpose:

Establish focused diagnostic ownership under the existing integration package.

Risk:

- Semantic: LOW
- Implementation fanout: multi-file

Owned files and symbols:

- `typing_integration/mod.rs`
- new `typing_integration/rows/mod.rs`
- new `typing_integration/sources/rows/`

Inspect before editing:

- current `typing_integration/mod.rs`;
- `either/mod.rs`, `monads/mod.rs`, `expression/mod.rs` for local style.

Dependencies:

- R0 baseline.

Source of truth:

Existing root package registration.

Implementation boundary:

Changes:

Add:

```rust
mod rows;
```

to the root package.

Create:

```rust
//! Structural Record and row-polymorphism integration suite.

mod calculus;
mod correlation;
mod transformations;
mod pipelines;
mod structural;
mod patterns;
mod rejection;
mod runtime;
```

Modules whose files are added in later checkpoints may be registered incrementally rather than as empty files.

Must not:

- create a Cargo test target;
- create `rows/support.rs`.

Current implementation:

No row sub-suite exists.

Target implementation:

Rows are a focused peer of Either/Monads/Expression.

Edit operations:

1. OPEN `typing_integration/mod.rs`.
2. ADD `mod rows;`.
3. CREATE `rows/mod.rs`.
4. CREATE `sources/rows/`.
5. Do not alter existing registrations.

Code instructions:

EXACT root addition:

```rust
mod rows;
```

STRUCTURAL child registry as above.

Testing classification:

No standalone behavioral test. Validated by R1.

---

### Task 4 — Extend Shared Support With Canonical Record Assertions and Row Source Builders

Purpose:

Give all later row tests a common, semantic-authority-preserving inspection surface.

Risk:

- Semantic: MEDIUM
- Implementation fanout: local shared helper

Owned files and symbols:

- `typing_integration/support.rs`
- `Fixture`
- current source constants/builders
- current `Ty` helper

Inspect before editing:

- `Fixture::assert_applied`
- `Fixture::assert_generic_solution_exact`
- `Fixture::callable_generic_parameter`
- `Fixture::ty`
- source-builder block
- imports from `phalcom_semantic::types`.

Dependencies:

- canonical `TypeData::Record`;
- public `TypeStore::record_row`.

Source of truth:

`TypeData::Record(RecordRowId)` and `TypeStore::record_row`.

Implementation boundary:

Changes:

Add row-specific inspection helpers shaped approximately as:

```rust
pub fn assert_closed_record(
    &self,
    actual: TypeId,
    expected_fields: &[(&str, TypeId)],
) -> RecordRowId;

pub fn assert_open_record(
    &self,
    actual: TypeId,
    expected_fields: &[(&str, TypeId)],
    expected_tail: TypeParameterId,
) -> RecordRowId;

pub fn assert_record_row_parameter(
    &self,
    parameter: TypeParameterId,
);
```

Expected behavior:

```text
assert actual is TypeData::Record
read canonical RecordRowData
assert exact field count
assert exact sorted names
assert each field TypeId exactly
assert expected tail
return RecordRowId for optional caller comparison
```

Add source constants/builders:

```text
ROWS_CORE_SOURCE
ROWS_CALCULUS_SOURCE

rows_core_source()
row_calculus_source()
with_rows(extra)
```

Do not add one builder containing every scenario.

Must not:

- use `RecordRowSolver`;
- use `RecordRowVarId`;
- calculate a remainder from an actual/prefix pair;
- add row fields to `Ty` unless later evidence shows it materially improves nested assertions.

Current implementation:

`Ty` supports Nominal/Applied/Tuple. The fixture already exposes canonical `TypeStore`, so a dedicated Record assertion is less invasive than making the shape DSL model row tails.

Target implementation:

Record structure can be inspected exactly while solver logic remains semantic-owned.

Edit operations:

1. ADD required imports:
  - `RecordRowId`
  - `RecordRowTail`
  - `TypeParameterId` already present
  - reuse current `TypeData`.
2. ADD helpers adjacent to existing type-shape assertions.
3. ADD row source constants next to existing source constants.
4. ADD only focused row builders.
5. SEARCH for `RecordRowSolver`/`RecordRowVarId` in `typing_integration`; expected zero.

Code instructions:

STRUCTURAL. Reconcile import module paths with current public re-exports.

Testing classification:

No separate helper test. R1 calculus tests exercise it against real compiler products.

Optional compile checkpoint:

```bash
cargo check -p phalcom-core --test core
```

Reason:

Useful after shared-helper API changes because it catches module/import/type fanout cheaply.

Does not prove row semantics.

---

### Task 5 — Add Core Row Calculus and Inference Tests

Purpose:

Create the isolated classic row-polymorphism pressure vessel.

Risk:

- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:

- `sources/rows/core.ph`
- `sources/rows/calculus.ph`
- `rows/calculus.rs`

Inspect before editing:

- current source syntax from semantic `record_row_polymorphism.rs`;
- current root fixture conventions;
- `ROW-INFER-*` law section in this plan.

Dependencies:

- Task 4 helpers/builders.

Source of truth:

Canonical call result and Record row structure.

Implementation boundary:

Changes:

Implement reusable `RowCalculus` functions and scenario probes for:

```text
ROW-INFER-01 remainder preservation
ROW-INFER-02 T + R
ROW-INFER-03 A + B + R higher-order annotation
ROW-INFER-04 proven empty remainder
ROW-INFER-05 result-only underconstraint
ROW-INFER-06 expected-result selection
```

Must not:

- infer row expectations in Rust;
- require a row explanation product that does not exist.

Current implementation:

Equivalent small ownership-level source tests exist in `phalcom-semantic`, but no reusable complex user-level row program exists in `typing_integration`.

Target implementation:

The suite proves exact source-language behavior using the shared core fixture.

Required Rust assertions:

For positive calls:

```text
exact selected RowCalculus callable
exact row generic parameter has kind RecordRow
ordinary generic solution asserted where applicable
exact canonical specialized Record fields
closed/open tail exact
Ready status
no diagnostics
```

For underconstraint:

```text
no published concrete type
Blocked status
RecordRowInferenceUnderconstrained
not Dynamic
```

Code instructions:

STRUCTURAL source shape from Section 3.

Testing classification:

Focused high-risk evidence at R1.

Checkpoint state update:

Record exact landed test names and source forms.

---

# 11. Checkpoint R2 — Correlation, Transformations, and Re-Decomposition

Tasks:

- Task 6 — add repeated-row correlation tests
- Task 7 — add immutable extension/higher-order transformation tests
- Task 8 — add canonical re-decomposition pipeline

Why this is a checkpoint:

These scenarios collectively test whether inferred remainders behave as semantic structure rather than incidental solver state.

Entry conditions:

- R1 COMPLETE.

Working set:

Primary:

- `sources/rows/core.ph`
- `sources/rows/transformations.ph`
- `sources/rows/pipelines.ph`
- `rows/correlation.rs`
- `rows/transformations.rs`
- `rows/pipelines.rs`

Secondary:

- current semantic row diagnostics only if a hostile assertion needs the exact code.

Out of scope:

- structural relation;
- patterns;
- VM;
- Either/Monad.

Semantic contract established:

- repeated R is genuinely correlated;
- legal immutable extension preserves R;
- collisions fail;
- published Record values can be decomposed under a different prefix in a later call.

Semantic risks:

- separate row variables accidentally allocated per parameter occurrence;
- return materialization silently loses extra fields;
- second call reuses a prior row substitution;
- duplicate extension degrades to Unknown rather than an explicit rejection.

Hostile cases:

- same function called with mismatched remainders;
- `tagged` input already contains `tag`;
- pipeline prefixes intentionally differ.

Required evidence:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::rows::correlation:: -- --nocapture

RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::rows::transformations:: -- --nocapture

RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::rows::pipelines:: -- --nocapture
```

Then:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::rows:: -- --nocapture
```

Proves isolated row calculus remains coherent.

Do not run yet:

- full typing package;
- runtime;
- workspace.

Escalate immediately if:

- a legal result loses canonical fields;
- the second pipeline stage appears to depend on solver IDs;
- extension collision silently succeeds.

Checkpoint completion:

- [ ] ROW-CORR-* pass
- [ ] ROW-XFORM-* pass
- [ ] ROW-PIPE-* pass
- [ ] hostile repeated/collision cases fail closed
- [ ] full isolated row semantic suite passes
- [ ] state updated
- [ ] no active incident

Suggested commit:

```text
test(core): stress row correlation and immutable transformations
```

---

### Task 6 — Add Stable Remainder Correlation

Purpose:

Prove repeated occurrences of one stable row binder use one inferred remainder.

Risk:

- Semantic: HIGH
- Implementation fanout: local

Owned files:

- `rows/correlation.rs`
- optional scenario additions to `sources/rows/calculus.ph`.

Source of truth:

One callable-owned `TypeParameterId` of kind `RecordRow`, plus call outcome/canonical return.

Implementation boundary:

Positive case uses two actual Records with equivalent extras.

Negative case uses incompatible extras.

For positive result, return a Record containing `R`, not merely `Bool`, so the canonical solved remainder is externally visible in the specialized type.

Must not:

- compare only call success;
- allocate two test-side expected rows.

Testing classification:

R2 high-risk evidence.

---

### Task 7 — Add Higher-Order Immutable Extension Tests

Purpose:

Exercise A/B ordinary inference and R inference in one function that extends a Record.

Risk:

- Semantic: HIGH
- Implementation fanout: multi-file

Owned files:

- `sources/rows/transformations.ph`
- `rows/transformations.rs`
- `sources/rows/invalid/duplicate_extension.ph`

Source of truth:

Specialized call result and semantic diagnostic.

Implementation boundary:

Positive:

```text
annotate
tagged
```

Negative:

```text
input remainder already contains extension label
```

For ordinary A/B parameters, reuse existing exact `GenericSolution` helpers.

For R, inspect the specialized result.

Must not:

- use field overwrite semantics;
- accept `Dynamic` on collision.

Testing classification:

R2 high-risk evidence.

---

### Task 8 — Add Re-Decomposition Pipeline

Purpose:

Prove canonical results are reinterpreted structurally at each call boundary.

Risk:

- Semantic: HIGH
- Implementation fanout: multi-file

Owned files:

- `sources/rows/pipelines.ph`
- `rows/pipelines.rs`

Target flow:

```text
Record
→ tagged<R1>
→ canonical Record
→ consumeTagged<R2>
→ canonical Record
```

Assertions:

```text
first result exact
second result exact
R2 contribution differs structurally from R1 contribution
both callables exact
both calls Ready
```

The test must not compare solver-local row variable identities.

Testing classification:

R2 high-risk evidence.

---

# 12. Checkpoint R3 — Structural Boundaries, Patterns, and Runtime

Tasks:

- Task 9 — add realistic structural protocol/capability tests
- Task 10 — add open-row pattern tests and hostile nominal/Map boundaries
- Task 11 — add focused row runtime scenarios

Why this is a checkpoint:

This checkpoint broadens from inference equations to the rest of the language surface while keeping rows isolated from Either/Monad/GADT.

Entry conditions:

- R2 COMPLETE.

Working set:

Primary:

- `sources/rows/structural_protocols.ph`
- `sources/rows/patterns.ph`
- `sources/rows/runtime.ph`
- `sources/rows/invalid/*`
- `rows/structural.rs`
- `rows/patterns.rs`
- `rows/rejection.rs`
- `rows/runtime.rs`
- root `support.rs` runtime builder block

Secondary:

- `phalcom-core/core/universe/src/collections/map.ph` only for current `Map<K,V>` surface;
- semantic pattern products only if assertions require them.

Out of scope:

- row relation production changes;
- compiler/VM Record representation changes.

Semantic contract established:

- immutable Record width/depth is useful in realistic APIs;
- nominal classes and Maps remain outside structural Record typing;
- open-row patterns preserve known proof boundaries;
- row-specialized source executes using ordinary runtime Records.

Semantic risks:

- structural typing leaks to nominal object layouts;
- Map gets treated as finite Record;
- open tail field is treated as guaranteed;
- runtime path requires a different Record representation from semantic typing.

Hostile cases:

- nominal class with same apparent members;
- `Map<String,Int>` with matching possible key names;
- pattern references field absent from known prefix;
- runtime source is executed without semantic preflight.

Required evidence:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::rows::structural:: -- --nocapture

RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::rows::patterns:: -- --nocapture

RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::rows::rejection:: -- --nocapture
```

Runtime:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::rows::runtime:: -- --nocapture
```

Then:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::rows:: -- --nocapture
```

Do not run yet:

- Expression;
- workspace.

Escalate immediately if:

- structural test requires inspecting nominal fields in Rust;
- runtime requires changing VM Record representation;
- Map boundary fails due a production semantic change.

Checkpoint completion:

- [ ] ROW-REL-* pass
- [ ] ROW-PATTERN-* pass
- [ ] ROW-REJECT-* pass
- [ ] runtime semantic preflight passes
- [ ] runtime observations pass
- [ ] complete isolated `rows::` suite passes
- [ ] state updated
- [ ] no active incident

Suggested commit:

```text
test(core): cover structural row protocols and runtime
```

---

### Task 9 — Add Reader/Environment Structural Protocol Tests

Purpose:

Exercise width and covariant depth in a realistic capability API.

Risk:

- Semantic: HIGH
- Implementation fanout: multi-file

Owned files:

- `sources/rows/structural_protocols.ph`
- `rows/structural.rs`

Target source shape:

```phalcom
class RowCapabilities {
    @class
    port<R: RecordRow>(
        _ environment: #{
            config: #{
                port: Int
            },
            | R
        }
    ) -> Int {
        environment.config.port
    }
}
```

Probe with a wider nested Record.

Assertions:

```text
exact call Ready
exact result Int
no structural mismatch
outer extra fields accepted
inner extra fields accepted
```

Add a depth failure where required nested field type is incompatible.

Must not:

- test width only with synthetic TypeStore construction; that is already semantic-owned.

Testing classification:

R3 high-risk evidence.

---

### Task 10 — Add Pattern and Scope-Boundary Hostility

Purpose:

Prove row typing stays structural only where the language says it is.

Risk:

- Semantic: HIGH
- Implementation fanout: multi-file

Owned files:

- `sources/rows/patterns.ph`
- `sources/rows/invalid/nominal_is_not_record.ph`
- `sources/rows/invalid/map_is_not_record.ph`
- `rows/patterns.rs`
- `rows/rejection.rs`

Inspect before editing:

- existing semantic open-row pattern test;
- current `Map<K,V>` Universe declaration.

Target hostile boundaries:

```text
class parameter with analogous fields
    != Record

Map<String,Int>
    != Record

unknown tail field
    != guaranteed field
```

For Map, use the current canonical generic `Map<K,V>` surface rather than a fabricated test-local Map.

Must not:

- make the fixture inspect object layout;
- accept generic `TypeMismatch` only if current semantic diagnostics expose a more specific stable code relevant to the case—inspect the first real result before freezing exact diagnostic assertions.

Testing classification:

R3 high-risk evidence.

---

### Task 11 — Add Row Runtime Scenarios

Purpose:

Verify semantic row specialization corresponds to executable ordinary Record behavior.

Risk:

- Semantic: MEDIUM
- Implementation fanout: multi-file / semantic-compiler-VM

Owned files:

- `sources/rows/runtime.ph`
- `rows/runtime.rs`
- root runtime builder.

Source of truth:

Same source string:

```text
analyze
→ compile
→ VM
```

Target runtime cases:

1. preserve a wider Record;
2. add a disjoint tag/mapped field;
3. pass transformed result through a second row-polymorphic function;
4. export primitive observations such as:
  - mapped Bool;
  - preserved Int;
  - preserved String size;
  - final tag size.

Do not require Rust to deserialize a Record if existing runtime support does not expose a convenient direct Record assertion. Observe primitive results computed in Phalcom.

Must not:

- skip semantic preflight;
- emulate Record operations in Rust.

Testing classification:

R3 cross-layer evidence.

---

# 13. Checkpoint R4 — Record Payloads Through Either, Monad/HKT, and Collections

Tasks:

- Task 12 — add direct Either<Record> composition
- Task 13 — add Monad/HKT Record A/B composition
- Task 14 — add nested `List<Record>` traversal

Why this is a checkpoint:

This is the first actual amendment to the package's cross-feature model. It proves rows are not an isolated side suite.

Entry conditions:

- R3 COMPLETE;
- existing Either and Monad suites green.

Working set:

Primary:

- new `sources/row_integration_probes.ph`
- new `row_integration.rs`
- root `support.rs`
- root `mod.rs`
- existing `sources/either.ph`
- existing `sources/monads.ph`

Secondary:

- existing `integration.rs` for assertion style;
- `monads/composition.rs` for exact F/A/B solution idioms.

Out of scope:

- Expression;
- modification of Either or Monad implementations.

Semantic contract established:

- Record types may be ordinary generic payloads;
- HKT application and constructor inference preserve exact structural Records;
- nested collections retain canonical Record element types.

Semantic risks:

- generic inference widens Record payload to `Dynamic`;
- same-shaped Record is reconstructed noncanonically;
- F is correct but A/B lose structural detail;
- row inference works only at top-level, not nested under applied types.

Hostile cases:

The strongest hostile evidence here is structural exactness itself:

```text
final A/B must contain all required Record fields
```

A weaker implementation that returns `Dynamic`, `{value:...}` only, or a nominal wrapper must fail the Rust assertions.

Required evidence:

Focused:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::row_integration::direct_either_record_payload_preserves_row_specialization \
  -- --nocapture
```

Then:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::row_integration:: -- --nocapture
```

Regression:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::either:: -- --nocapture

RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::monads:: -- --nocapture
```

Then package excluding Expression only if Expression has a baseline incident; otherwise full package may be run here.

Do not run yet:

- workspace;
- Expression Record flagship.

Escalate immediately if:

- direct Either requires copying `Either`;
- Monad test requires a row-specific Monad algorithm;
- HKT F inference loses exact Record A/B despite isolated row suite being green.

Checkpoint completion:

- [ ] INT-ROW-01 passes
- [ ] INT-ROW-02 passes
- [ ] INT-ROW-03 passes
- [ ] exact callable identities asserted
- [ ] exact F/A/B ordinary generic solutions asserted
- [ ] nested Record types asserted canonically
- [ ] existing Either/Monad regressions pass
- [ ] state updated
- [ ] no active R4 incident

Suggested commit:

```text
test(core): compose record rows with Either and Monad
```

---

### Task 12 — Direct Either With Record Payload

Purpose:

Combine the canonical Either ADT with a row-polymorphic higher-order transformation.

Risk:

- Semantic: HIGH
- Implementation fanout: multi-file

Owned files:

- `sources/row_integration_probes.ph`
- `row_integration.rs`
- root source builder.

Source of truth:

Existing canonical `Either` plus canonical specialized Record result.

Target source:

A probe takes:

```text
Either<String, InputRecord>
```

and performs direct:

```text
source.map(...)
```

where the closure calls `RowCalculus.annotate`.

Expected:

```text
Either<String, OutputRecord>
```

Required assertions:

```text
Either.map exact CallableId
RowCalculus.annotate exact CallableId
Either declaration exact identity
right-hand applied type is canonical OutputRecord
Ready
no diagnostics
```

Testing classification:

R4 high-risk evidence.

---

### Task 13 — Monad/HKT Bind Over Record Payloads

Purpose:

Make existing constructor-level inference solve `F`, Record-valued A/B, and an inner row-polymorphic call together.

Risk:

- Semantic: HIGH
- Implementation fanout: multi-file

Target:

```text
MonadAlgorithms.bind(
    StringEitherMonad,
    Either<String, InputRecord>,
    InputRecord -> Either<String, OutputRecord>
)
```

Required generic solutions:

```text
F = canonical <X> =>> Either<String,X>
A = canonical InputRecord TypeId
B = canonical OutputRecord TypeId
```

Inside continuation:

```text
RowCalculus.annotate
```

must produce exact OutputRecord.

Use current shared generic-provenance helpers.

Must not:

- add `recordBind`;
- specialize Monad source for Records.

Testing classification:

R4 high-risk evidence.

---

### Task 14 — Traverse a List of Records

Purpose:

Push structural types under two ordinary generic applications.

Risk:

- Semantic: HIGH
- Implementation fanout: multi-file

Target:

```text
List<InputRecord>
    ↓ MonadAlgorithms.traverse using StringEitherMonad
Either<String,List<OutputRecord>>
```

Assertions:

```text
traverse selected callable exact
F exact
A == InputRecord
B == OutputRecord
result == Either<String,List<OutputRecord>>
List element TypeId exactly OutputRecord
OutputRecord fields canonical
```

Testing classification:

R4 high-risk evidence.

---

# 14. Checkpoint R5 — Record-Valued GADT Indices and Full-Stack Flagship

Tasks:

- Task 15 — add Expression Record-index semantic composition
- Task 16 — add hostile GADT/row boundary and exact `F<T>` assertions
- Task 17 — add Record-payload full-stack runtime flagship

Why this is a checkpoint:

Rows, GADT result specialization, HKT application, Monad algorithms, and VM execution only become meaningful as one integration claim after simpler layers are independently green.

Entry conditions:

- R4 COMPLETE;
- **current existing `typing_integration::expression::` baseline green**.

If R0 classified Expression as a baseline incident, R5 remains blocked.

Working set:

Primary:

- existing `sources/expression.ph`
- new `sources/expression_row_probes.ph`
- new `sources/expression_row_runtime_probes.ph`
- existing `expression/integration.rs`
- new `expression/rows.rs`
- root support builders

Secondary:

- existing `expression/monad.rs`
- existing `expression/runtime.rs`
- existing `expression_semantic_probes.ph`.

Out of scope:

- redesigning Expression;
- parser/AST feature work;
- GADT proof-engine changes unless a separately diagnosed product defect is authorized.

Semantic contract established:

- a structural Record may be the exact GADT index T;
- row-polymorphic higher-order transformation changes T precisely;
- `ExpressionEvaluation.eval` carries the exact Record through `F<T>`;
- traverse can produce `Expression<F,List<Record>>`;
- final evaluation produces exact `Either<String,List<Record>>`.

Semantic risks:

- GADT refinement works for primitive T but not structural T;
- F<T> loses row-specialized T;
- type-lambda substitution loses nested Record identity;
- higher-order closure inference widens Record parameter;
- runtime source passes only because semantic checking is skipped.

Hostile case:

Use a Record-valued Expression where the transform's required field type disagrees with the actual index, e.g. an `Int`-valued field consumed as `String`.

It must reject without `Dynamic`.

Required evidence:

First re-prove prerequisite:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::expression:: -- --nocapture
```

Then semantic row/GADT:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::expression::rows:: -- --nocapture
```

Runtime flagship:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::expression::rows::record_payload_flagship_runtime \
  -- --nocapture
```

Then:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration:: -- --nocapture
```

Do not run yet:

- full workspace until R6.

Escalate immediately if:

- current scalar Expression suite regresses before row additions;
- new source requires parser syntax not already accepted by the current Expression fixtures;
- GADT Record test requires modifying `gadt_proof.rs`;
- VM needs a new runtime Record representation.

Checkpoint completion:

- [ ] Expression prerequisite green
- [ ] INT-ROW-04 passes
- [ ] INT-ROW-05 passes
- [ ] hostile Record-index mismatch rejects
- [ ] INT-ROW-06 semantic intermediate/final assertions pass
- [ ] Record runtime flagship passes semantic preflight
- [ ] Record runtime observations pass
- [ ] full typing integration package passes
- [ ] state updated
- [ ] no active incident

Suggested commits:

```text
test(core): compose Expression GADTs with structural record rows
test(core): execute record-payload typing flagship
```

---

### Task 15 — Add Record-Valued Expression Indices

Purpose:

Prove structural Records work as exact GADT result indices.

Risk:

- Semantic: HIGH
- Implementation fanout: multi-file

Owned files:

- `sources/expression_row_probes.ph`
- `expression/rows.rs`

Target source shape:

```text
Expression<
    <X> =>> Either<String,X>,
    InputRecord
>
```

mapped through a closure invoking `RowCalculus.annotate`.

Expected:

```text
Expression<
    <X> =>> Either<String,X>,
    OutputRecord
>
```

Then:

```text
ExpressionEvaluation.eval(...)
    ->
Either<String,OutputRecord>
```

Required assertions:

```text
Expression applied constructor exact
input index exact Record TypeId
output index exact Record TypeId
Expression::Map selection exact where exposed
RowCalculus.annotate selection exact
eval final result nests same output Record
```

Testing classification:

R5 high-risk evidence.

---

### Task 16 — Add Cross-Domain Hostility and `F<T>` Exactness

Purpose:

Defeat the easiest bad implementation: widen structural GADT T enough that F application succeeds anyway.

Risk:

- Semantic: HIGH
- Implementation fanout: local

Owned files:

- `expression/rows.rs`
- optional invalid row/Expression source.

Hostile source:

A Record-indexed Expression contains:

```text
value: Int
```

while a transform requires:

```text
value: String
```

Expected:

```text
Invalid
not Dynamic
no fake OutputRecord
```

Positive counterpart explicitly compares:

```text
Record TypeId nested in Expression index
==
Record TypeId nested in final Either argument
```

This is a cross-consumer consistency assertion for the same canonical type.

Testing classification:

R5 high-risk evidence.

---

### Task 17 — Add the Full-Stack Record Flagship

Purpose:

Create the package's strongest complex classic typing test.

Risk:

- Semantic: HIGH
- Implementation fanout: multi-file / cross semantic-compiler-VM

Owned files:

- `sources/expression_row_probes.ph`
- `sources/expression_row_runtime_probes.ph`
- `expression/rows.rs`
- shared runtime builder.

Target semantic program:

```text
List<InputRecord>
    ↓
MonadAlgorithms.traverse(
    StringEitherExpressionMonad,
    ...
)
    ↓
Expression<
    Either<String,_>,
    List<OutputRecord>
>
    ↓
ExpressionEvaluation.eval(
    StringEitherMonad,
    ...
)
    ↓
Either<String,List<OutputRecord>>
```

Required intermediate assertions:

```text
InputRecord exact
OutputRecord exact

traverse:
    F exact Expression constructor
    A exact InputRecord
    B exact OutputRecord

intermediate:
    Expression effect constructor exact
    index == List<OutputRecord>

eval:
    result == Either<String,List<OutputRecord>>
```

Runtime observations:

Prefer primitive projections such as:

```text
result is Right
list size
first mapped Bool
first preserved value Int
first preserved name size
```

rather than Rust-side Record decoding.

Testing classification:

R5 flagship cross-layer evidence.

---

# 15. Checkpoint R6 — Documentation, Migration Closure, and Delivery Gates

Tasks:

- Task 18 — update package contract/law catalog and state ledger
- Task 19 — run deletion, compatibility, and delivery gates

Why this is a checkpoint:

The amendment is not complete merely because new tests pass. The package contract must reflect rows, and the architecture must prove no second row fixture/solver authority was introduced.

Entry conditions:

- R5 COMPLETE, or R5 explicitly blocked by a separately recorded baseline incident if the user chooses to land isolated/Either/Monad row coverage independently. A full amendment release-complete claim still requires R5.

Working set:

Primary:

- `typing_integration/README.md`
- `typing_integration/LAWS.md`
- `typing_integration/**`
- existing implementation state file.

Secondary:

- workspace failures only for classification.

Out of scope:

- fixing unrelated repository-wide baseline failures unless separately authorized.

Semantic contract established:

- rows are a documented first-class axis of the classic typing integration package;
- focused row and cross-feature laws are discoverable;
- there is no parallel solver/fixture/source authority;
- broad compatibility status is known.

Semantic risks:

- duplicate `RowCalculus`;
- accidental row dependency in basic fixtures;
- stale README scope exclusion;
- workspace baseline failures misreported as amendment regressions.

Required evidence:

Focused package:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::rows:: -- --nocapture

RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::row_integration:: -- --nocapture

RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::expression::rows:: -- --nocapture

RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration:: -- --nocapture
```

Core integration:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core
```

Then final broad gates in Section 20.

Checkpoint completion:

- [ ] README amended
- [ ] `ROW-*` laws catalogued
- [ ] `INT-ROW-*` laws catalogued
- [ ] negative architecture searches pass
- [ ] complete typing package passes
- [ ] core target result recorded
- [ ] broad delivery gates recorded
- [ ] deferred-evidence ledger empty or release blocker explicitly recorded
- [ ] no unresolved amendment incident
- [ ] state finalized

Suggested commit:

```text
docs(test): finalize record-row typing integration amendment
```

---

### Task 18 — Update Package Contract and Law Catalog

Purpose:

Make rows an explicit permanent feature axis of the package.

Risk:

- Semantic: LOW
- Implementation fanout: local docs/state

Owned files:

- `typing_integration/README.md`
- `typing_integration/LAWS.md`
- `typing-integration-implementation-state.md`

Current implementation:

README lists focused Either, Monad, and Expression suites. LAWS title is currently `GEN, MON, GEX, and INT`.

Target implementation:

README says the package covers:

```text
generic ADTs
constructor-kinded generics
type lambdas
higher-order inference
generic inheritance
structural Record row polymorphism
GADT refinement
cross-feature composition
VM execution
```

Focused suites include `rows/`.

LAWS title becomes approximately:

```text
Unified Typing Integration Laws — GEN, MON, ROW, GEX, and INT
```

Add all `ROW-*` and `INT-ROW-*` definitions with exact landed test names.

Do not:

- duplicate SC-3 low-level acceptance laws;
- claim category-theoretic laws;
- renumber MON/GEX/INT existing entries.

Testing classification:

Documentation/deletion evidence at R6.

---

### Task 19 — Final Architecture and Delivery Verification

Purpose:

Prove the amendment is exclusive, compatible, and reviewable.

Risk:

- Semantic: LOW
- Implementation fanout: workspace-wide evidence

Owned files:

No code by default.

If a command fails, enter Failure Protocol before editing.

Required negative gates are listed next.

---

# 16. Final negative/deletion gates

## 16.1 One reusable row source authority

```bash
rg -n '^\s*class RowCalculus\b' \
  phalcom-core/tests/core/typing_integration
```

Expected exactly one source declaration:

```text
sources/rows/core.ph
```

## 16.2 No child row support authority

```bash
find phalcom-core/tests/core/typing_integration/rows \
  -name support.rs -print
```

Expected zero hits.

The only support authority remains:

```text
typing_integration/support.rs
```

## 16.3 No solver implementation leaks into core integration tests

```bash
rg -n 'RecordRowSolver|RecordRowVarId|GenericApplicationSession' \
  phalcom-core/tests/core/typing_integration
```

Expected zero hits.

`RecordRowId`, `RecordRowTail`, and `RecordRowData` inspection are legitimate.

## 16.4 No test-local remainder calculation

Search for temporary/helper names created during implementation, including:

```bash
rg -n 'infer.*remainder|solve.*row|subtract.*row|calculate.*row' \
  phalcom-core/tests/core/typing_integration
```

Every hit must be inspected.

Expected no helper that semantically derives R.

Names in comments/law descriptions may remain.

## 16.5 Basic fixtures remain row-independent

```bash
rg -n 'ROWS_.*SOURCE|with_rows|row_.*source' \
  phalcom-core/tests/core/typing_integration/support.rs
```

Inspect usages.

Required architecture:

```text
either_source
    does not include rows

monads_source
    does not include rows

expression_source
    does not include rows

row-specific builders
    explicitly compose rows
```

## 16.6 Existing canonical sources remain singular

```bash
rg -n '^\s*enum Either<L, R>' \
  phalcom-core/tests/core/typing_integration

rg -n '^\s*class Monad<' \
  phalcom-core/tests/core/typing_integration

rg -n '^\s*enum Expression<' \
  phalcom-core/tests/core/typing_integration
```

No row test may introduce copies.

## 16.7 No nominal/Map workaround in flagship source

Review:

```text
sources/row_integration_probes.ph
sources/expression_row_probes.ph
sources/expression_row_runtime_probes.ph
```

Required payload type is a Record.

A nominal wrapper used only for a hostile negative test is acceptable.

## 16.8 Current docs no longer exclude rows

```bash
rg -n 'record-row tests.*outside|structural record-row.*out of scope' \
  phalcom-core/tests/core/typing_integration \
  docs/impl/semantic/typing-integration
```

Any live-current exclusion from the older plan/state must be updated or explicitly marked historical.

Do not rewrite completed historical evidence solely to eliminate text matches.

---

# 17. Repository drift protocol

Before each checkpoint:

1. verify every primary file still exists;
2. verify `typing_integration/support.rs` remains shared fixture authority;
3. verify `GenericApplicationSession`/row semantics have not been materially replaced;
4. inspect changes made by previous amendment checkpoints;
5. search for newly introduced callers before changing shared source builders;
6. adapt only mechanics.

Allowed adaptation:

```text
TypeStore row accessor moved to a new public module.
Update imports/helper calls.
```

Not allowed adaptation:

```text
Row result is hard to inspect.
Compare pretty-printed strings instead.
```

Not allowed:

```text
Generic row inference conflicts.
Use Dynamic.
```

If the semantic design has changed, mark the current checkpoint `INCIDENT`.

---

# 18. Failure protocol

If required evidence fails:

```text
R<N> — INCIDENT
```

Do not continue dependent checkpoints.

Record:

## Exact reproduction

```text
command
failing test
important status/diagnostic/assertion
```

## Direct path

For an isolated row test:

```text
Phalcom row fixture
→ parse
→ semantic annotation lowering
→ canonical Record type
→ generic call application
→ GenericApplicationSession
→ RecordRowSolver
→ GenericInstantiation
→ canonical result TypeId
→ fixture assertion
```

For cross-feature tests:

```text
row fixture
→ RowCalculus
→ Either/Monad/Expression generic call
→ semantic result
→ optional compiler
→ VM
```

## Passing comparator

Useful comparisons include:

```text
ROW-INFER passes, INT-ROW fails
    likely cross-feature integration

Either<Record> passes, Monad<Record> fails
    likely HKT/generic composition

scalar Expression passes, Record Expression fails
    likely row/GADT composition

semantic flagship passes, runtime fails
    compiler/VM boundary
```

## Classification

Use:

```text
PRODUCT
FIXTURE
DEPENDENCY/PUBLICATION
BACKEND/HARNESS
BASELINE
PLAN DRIFT
```

## Narrow repair boundary

Default amendment write boundary is:

```text
phalcom-core/tests/core/typing_integration/**
docs/impl/semantic/typing-integration/**
```

Crossing into:

```text
phalcom-semantic
phalcom-ast
compiler
VM
```

requires explicit incident evidence first.

## Rejected broad fixes

Do not:

```text
restore duplicate source declarations
weaken exact type assertions
compare types only by display text
run a test-local row solver
encode row as TypeId
turn rejection into Dynamic
replace Record with class
replace Record with Map
skip semantic runtime preflight
change parser syntax opportunistically
special-case test class names in production
```

---

# 19. Working-state protocol

Reuse:

```text
docs/impl/semantic/typing-integration/typing-integration-implementation-state.md
```

Do not create a competing state authority.

Add a section:

```md
## Record Rows Amendment

### Baseline
- Plan baseline: `e17f2733...`
- Execution branch:
- Execution HEAD:
- Working-tree note:

### Established row integration invariants
- RI-01:
- ...

### Decisions
- RD-01:
- ...

### Evidence ledger

| Checkpoint | Command | Result | Proves |
|---|---|---|---|

### Negative/deletion evidence
- ...

### Deferred gates
- ...

### Active amendment incident
None.

### Next resume action
Begin R<N> Task <M>.
```

Historical C0–C4 evidence from the original program remains.

If the historical Expression C4 incident is no longer current, preserve it under historical/resolved findings instead of silently erasing it.

---

# 20. Final broad gates

Run only after focused R6 evidence.

## Format

```bash
cargo fmt --all -- --check
```

Proves:

- Rust amendment edits satisfy repository formatting.

Does not prove row semantics.

## Workspace compilation

```bash
RUSTFLAGS='' cargo check --workspace --all-targets
```

Proves:

- shared helper/module additions compile across workspace targets.

Does not prove row inference behavior.

## Workspace tests

```bash
RUSTFLAGS='' cargo test --workspace --all-targets
```

Proves:

- no broad regression after the amendment.

The SC-3 state ledger records historical unrelated workspace failures. If any recur, classify them against the R0 baseline before editing unrelated subsystems.

## Clippy

```bash
RUSTFLAGS='' cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Proves:

- changed Rust integration code introduces no lint errors under workspace policy.

Again, classify known pre-existing failures separately.

## Core target

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core
```

Proves:

- the `core` integration binary remains coherent after module/source additions.

It does not replace focused `ROW-*`/`INT-ROW-*` evidence.

---

# 21. Checkpoint evidence summary

Plan-time status is intentionally `PENDING`. No implementation tests were run in this planning session.

| Checkpoint | Semantic contract | Evidence | Status |
|---|---|---|---|
| R0 | current SC-3 and typing-integration baseline is known | Git + focused semantic rows + Either/Monad/Expression/package baselines | PENDING |
| R1 | one isolated row authority + canonical inspection + basic row inference | `rows::calculus` + authority search | PENDING |
| R2 | correlation, extension, higher-order row inference, re-decomposition | correlation/transformation/pipeline suites + hostile cases | PENDING |
| R3 | realistic structural relations, patterns, scope boundaries, runtime | structural/pattern/rejection/runtime + full `rows::` | PENDING |
| R4 | Records compose with Either, Monad/HKT, and nested List | `row_integration::` + Either/Monad regressions | PENDING |
| R5 | Records compose with GADT indices, F<T>, Expression traversal and VM | Expression prerequisite + `expression::rows` + flagship | PENDING |
| R6 | package contract and delivery are complete | full package + negative gates + core + broad gates | PENDING |

No row becomes COMPLETE without its actual command evidence in the shared state file.

---

# 22. Deferred-evidence audit

At R6 require:

```text
No deferred command remains unless:

1. it executed successfully;
2. it was explicitly removed from the amendment scope with a concrete reason;
3. it is recorded as a known release blocker / active INCIDENT.
```

Specifically account for:

```text
complete typing_integration package
full phalcom-core core test target
cargo fmt
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Do not inherit a historical PASS from the old program as evidence for the amended tree.

---

# 23. Suggested staged commit groups

Recommended:

```text
R1
test(core): add isolated record-row typing calculus

R2
test(core): stress row correlation and immutable transformations

R3
test(core): cover structural row protocols and runtime

R4
test(core): compose record rows with Either and Monad

R5.1
test(core): compose Expression GADTs with record rows

R5.2
test(core): execute record-payload typing flagship

R6
docs(test): finalize row typing integration laws
```

R1–R2 may be combined if review size remains reasonable, but do not combine the entire row suite, cross-feature work, and runtime flagship into one commit.

---

# 24. Known scope exclusions

This amendment deliberately does **not** include:

```text
new row solver algorithms
new row inference semantics
RecordRowVarId publication
GenericRowSolution provenance feature
metadata row serialization changes
metadata fingerprint changes
incremental row invalidation changes
cold/incremental SC-3 ownership tests
query budget/cancellation tests
solver history/alias/occurs-check tests
scoped open-row internal lowering tests
general row-valued nominal generic application
row-valued transparent-alias application
row-valued type-lambda application
generic getter support
effect rows
variant rows
mutable structural Records
Map key-set row typing
nominal structural object typing
new parser syntax
new GADT semantics
new Monad algorithms
replacement of existing scalar Expression tests
```

Those first several items already belong to SC-3 ownership tests. The unsupported type-application items remain outside SC-3's current scope. fileciteturn0file0

---

# 25. Release-complete criteria

The Record Rows Amendment is complete only when:

- [ ] R0 through R6 are COMPLETE;
- [ ] local baseline and relevant dirty files were recorded before edits;
- [ ] focused SC-3 row prerequisites pass;
- [ ] one `sources/rows/core.ph` is the only reusable RowCalculus authority;
- [ ] no `rows/support.rs` exists;
- [ ] no `RecordRowSolver`, `RecordRowVarId`, or `GenericApplicationSession` is used by `phalcom-core` row tests;
- [ ] ROW-INFER laws pass;
- [ ] repeated-row correlation succeeds for compatible rows;
- [ ] repeated-row conflict rejects without Dynamic;
- [ ] proven empty remainder works;
- [ ] result-only row remains underconstrained without context;
- [ ] expected-result row selection works;
- [ ] higher-order A/B/R inference works;
- [ ] immutable Record extension preserves the remainder;
- [ ] duplicate extension/lacks collision rejects;
- [ ] re-decomposition pipeline infers a fresh structural remainder at each call;
- [ ] realistic nested width + covariant-depth capability typing passes;
- [ ] nominal object does not satisfy Record structurally;
- [ ] `Map<K,V>` does not satisfy Record structurally;
- [ ] open-row known-prefix pattern typing is precise;
- [ ] possible tail fields are not fabricated as guaranteed;
- [ ] isolated row runtime scenarios pass after semantic preflight;
- [ ] direct `Either<Record>` composition passes;
- [ ] Monad/HKT Record A/B inference passes;
- [ ] `List<Record>` traversal preserves exact element structural type;
- [ ] Record-valued Expression/GADT indexing passes;
- [ ] row-specialized T survives exact `F<T>` application;
- [ ] Record/GADT hostile mismatch rejects without Dynamic;
- [ ] full-stack Record flagship has exact intermediate and final canonical types;
- [ ] full-stack runtime flagship passes semantic preflight and VM observations;
- [ ] existing Either laws remain green;
- [ ] existing Monad laws remain green;
- [ ] existing scalar Expression laws remain green;
- [ ] `ROW-*` and `INT-ROW-*` laws are documented with exact test names;
- [ ] package README presents rows as a first-class typing axis;
- [ ] all negative/deletion gates pass;
- [ ] no forgotten deferred gate remains;
- [ ] final format/check/test/clippy gates pass or an explicit repository release blocker is recorded;
- [ ] shared implementation-state file has no unresolved amendment INCIDENT.

---

# 26. Final architectural result

After this amendment, the package should read conceptually as:

```text
CLASSIC TYPE-SYSTEM PROGRAMS

    Either<L,R>
        generic nominal ADT

    RowCalculus / #{ ... | R }
        structural extensible records

    Functor<F>
    Applicative<F>
    Monad<F>
        higher-kinded abstraction

    Expression<F,T>
        indexed GADT

                 ↓

CROSS-FEATURE COMPOSITION

    Either<String, Record>

    Monad<
        <X> =>> Either<String,X>
    >
    operating on Record A/B

    List<Record>
        through generic algorithms

    Expression<
        <X> =>> Either<String,X>,
        Record
    >

                 ↓

FULL-STACK CLASSIC TYPING TORTURE TEST

    List<InputRecord>
        ↓
    row-polymorphic higher-order transformation
        ↓
    MonadAlgorithms.traverse
        ↓
    Expression<Either<String,_>,List<OutputRecord>>
        ↓
    GADT evaluation
        ↓
    Either<String,List<OutputRecord>>
        ↓
    ProgramCompiler
        ↓
    VM
```

That is the intended long-term model for `typing_integration`: each major typing mechanism receives a difficult representative program of its own, then it must survive composition with the other independent mechanisms. The package becomes a durable catalog of complex classic typing tests rather than a collection of feature-specific smoke tests.