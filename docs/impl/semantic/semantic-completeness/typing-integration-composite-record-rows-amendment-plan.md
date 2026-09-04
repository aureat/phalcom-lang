# Phalcom Unified Typing Integration — Composite Record/Row Amendment
## Repository-Grounded, Checkpoint-Driven, Patch-Grade Implementation Plan

> **For agentic workers:** implement checkpoint-by-checkpoint. Use the existing `typing_integration` package as the only integration-test authority; do not create a parallel test universe.

**Program goal:** extend the existing `phalcom-core/tests/core/typing_integration` system so structural Records and row polymorphism participate in the same complex typing programs as `Either`, ordinary generics, higher-order functions, type lambdas/HKTs, `Monad<F>`, `MonadAlgorithms`, nested `List<T>` applications, and—once the existing Expression/GADT elimination prerequisite is green—`Expression<F,T>` GADTs.

This amendment does **not** replace the isolated `typing_integration::rows` suite. The isolated row suite remains the diagnostic ownership layer for user-level row calculus. This amendment adds a second layer: composite programs that prove row typing composes with independent typing mechanisms.

---

# 1. Repository Baseline

**Repository:** `aureat/phalcom-lang`  
**Prepared against remote branch:** `main`  
**Exact remote HEAD:** `e932aac4e21a5b346e719ede5a24f94e7b924ab3`  
**HEAD subject:** `feat(semantic): complete SC-4.8 typing integration`  
**Repository inspection mode:** remote GitHub inspection. The implementing agent must record local branch/HEAD/working-tree state before editing.

Relevant repository facts at this revision:

- `phalcom-core/tests/core/typing_integration/mod.rs` already owns:
  - `support`
  - `either`
  - `monads`
  - `integration`
  - `expression`
  - `rows`
- `typing_integration/support.rs` is the one shared fixture/source-composition authority.
- `support.rs` currently composes:
  - `sources/either.ph`
  - `sources/monads.ph`
  - `sources/integration_probes.ph`
  - `sources/expression.ph`
  - Expression semantic/runtime probes
  - `sources/rows/core.ph`
  - `sources/rows/calculus.ph`
- `support.rs::Ty` currently models only:
  - `Nominal`
  - `Applied`
  - `Tuple`
- Record assertions currently use:
  - `Fixture::assert_closed_record`
  - `Fixture::assert_open_record`
  - `Fixture::assert_record_row_parameter`
- `sources/either.ph` is the single canonical user-level `Either<L,R>` definition.
- `sources/monads.ph` is the single canonical source for:
  - `Functor<F>`
  - `Applicative<F>`
  - `Monad<F>`
  - `EitherMonad<E>`
  - `StringEitherMonad`
  - `MonadAlgorithms`
- `sources/rows/core.ph` is the single reusable row-calculus source and currently exposes:
  - `preserve`
  - `preserveValue`
  - `annotate`
  - `tagged`
  - `sameRemainder`
  - `consumeTagged`
  - `make`
- `integration.rs` currently proves direct `Either` and `MonadAlgorithms.bind` share the same canonical `Either` source, but its payloads are scalar `Int`/`Bool`.
- `expression/integration.rs` currently intends to prove the flagship:
  - `List<Int>`
  - `MonadAlgorithms.traverse`
  - `Expression<...,List<Int>>`
  - `ExpressionEvaluation.eval`
  - `Either<String,List<Int>>`
- The row semantic repair is recorded through its C5 checkpoint as complete on the current repository lineage:
  - signature-wide row `lacks` obligations are implemented;
  - row-aware generic-return publication no longer silently drops structured failure;
  - open Record literal extension requires proven disjointness;
  - the original `ROW-XFORM-03` blocker is resolved;
  - `typing_integration::rows`, `either`, and `monads` were recorded green after the repair.
- The former Expression parser blocker has changed. Constructor-local variant generic syntax is now represented, but the implementation-state ledger still records an Expression semantic incident at constructor-local generic/GADT elimination. Therefore this amendment must not assume the full Expression suite is green merely because parsing is now available.

The repository is authoritative during implementation. Re-run the C0 baseline; do not rely on this document as proof that the local checkout has not drifted.

---

# 2. Program Purpose

The package-level testing model becomes:

```text
isolated classic typing programs
    |
    +-- Either<L,R>
    +-- Functor / Applicative / Monad
    +-- Expression<F,T>
    +-- RowCalculus / #{ ... | R }
    |
    v
pairwise composition
    |
    +-- Either<Record>
    +-- HKT<Record>
    +-- Monad<Record>
    +-- List<Record>
    +-- expected-result inference over ADT<Record>
    |
    v
multi-feature composition
    |
    +-- type lambda F + MonadAlgorithms + Record A/B
    +-- nested ADT + structural Record
    +-- row failure through higher-order generic APIs
    |
    v
flagship GADT composition
    |
    +-- Expression<Either<String,_>, Record>
    +-- traverse List<Record> through ExpressionMonad
    +-- eval to Either<String,List<Record>>
```

The core doctrine is:

> `typing_integration::rows` proves row semantics in sophisticated isolation. Composite integration tests prove that rows survive interaction with independent type-system mechanisms.

This package should test one coherent type system, not a collection of individually working feature islands.

---

# 3. Requirements Analysis

## 3.1 Required observable behaviors

The amendment must prove all of the following.

1. A closed structural Record can be an exact generic ADT argument:
   ```text
   Either<String, InputRecord>
   ```

2. A row-polymorphic function may be called from inside an `Either.map` closure and preserve the inferred remainder in the outer `Either` result.

3. Ordinary generic variables and row variables may be solved in the same nested higher-order program:
   ```text
   Either.map<R2>
       +
   RowCalculus.annotate<A,B,R>
   ```

4. A higher-kinded constructor variable may be inferred while its `A` and `B` arguments are exact structural Record types:
   ```text
   F = <X> =>> Either<String,X>
   A = InputRecord
   B = OutputRecord
   ```

5. `MonadAlgorithms.bind` must accept Record-valued `A` and `B` without:
   - widening either to `Dynamic`;
   - erasing fields;
   - converting the Record to a nominal wrapper;
   - constructing a noncanonical duplicate Record type.

6. `MonadAlgorithms.traverse` must preserve exact Record element types beneath:
   ```text
   List<...>
   ```
   and:
   ```text
   Either<String,...>
   ```

7. Type-lambda beta application must preserve the exact Record type:
   ```text
   (<X> =>> Either<String,X>)<OutputRecord>
       ==
   Either<String,OutputRecord>
   ```

8. Expected-result inference must be able to solve ordinary and row generic dimensions through an outer applied ADT when the call itself is otherwise underconstrained.

9. A generic ordinary type parameter may itself be an applied ADT while another independent row variable preserves unrelated Record fields:
   ```text
   T = Either<String,Int>
   R = #{ label: String, cached: Bool }
   ```

10. Composite hostile cases must fail in the semantic domain that actually owns the contradiction. A row `lacks` collision occurring inside `Either.map` or `MonadAlgorithms.bind` must not degrade to:
    - `Dynamic`;
    - unexplained `Unknown`;
    - successful outer ADT materialization;
    - a generic conflict if a precise row diagnostic already exists.

11. No query-local `RecordRowVarId` may survive into the outer ADT/HKT/GADT canonical result.

12. Once Expression/GADT elimination is green, structural Records must be valid exact `Expression<F,T>` result indices and survive branch-local GADT refinement and `F<T>` application.

13. The strongest flagship test must use Record payloads, not nominal wrappers.

---

# 4. Ownership Boundaries

## Source of truth: canonical Record structure

```text
TypeData::Record(RecordRowId)
    ↓
RecordRowData
```

Consumers:

- `Fixture::assert_closed_record`
- nested applied-type assertions
- composite integration tests

Forbidden competing authority:

- parsing `TypeStore::format_type` output;
- reconstructing Record field sets from source text;
- test-local row subtraction.

## Source of truth: generic ADT

```text
sources/either.ph
```

Consumers:

- direct Either tests
- Monad tests
- row composite tests
- Expression tests

Forbidden competing authority:

- a `RowEither` enum duplicated under `sources/rows`;
- copied `Either` declarations in composite probe files.

## Source of truth: HKT/Monad hierarchy

```text
sources/monads.ph
```

Consumers:

- monad sub-suite
- scalar package integration
- Record composite integration
- Expression integration

Forbidden competing authority:

- `RecordMonad`
- row-specific `bind`
- row-specific `traverse`

The tests must prove that the **existing** generic algorithms accept structural Record payloads.

## Source of truth: row-polymorphic transformation

```text
sources/rows/core.ph
```

Consumers:

- isolated row suite
- direct Either<Record> integration
- Monad/HKT Record integration
- later GADT integration

Forbidden competing authority:

- a copied `annotate`/`tagged` implementation in `row_integration_probes.ph`.

## Source of truth: GADT model

```text
sources/expression.ph
```

Consumers:

- existing Expression tests
- later Record/GADT tests

Forbidden competing authority:

- a simplified row-specific GADT.

## Source of truth: test fixture

```text
typing_integration/support.rs
```

Forbidden competing authority:

```text
rows/support.rs
row_integration/support.rs
expression/row_support.rs
```

---

# 5. Tempting Wrong Fixes

Do not take these shortcuts.

## Do not replace structural payloads with nominal classes

Bad:

```phalcom
class InputRow {
    const value: Int
    const name: String
}
```

The test exists specifically to prove `TypeData::Record` composes with ADTs/HKTs/GADTs.

## Do not duplicate `Either`

A composite test must use `sources/either.ph`.

## Do not add Record-specific Monad algorithms

Bad:

```phalcom
RowMonadAlgorithms.bindRecord(...)
```

Use `MonadAlgorithms.bind`, `traverse`, `sameConstructor`, or `constructorIdentity`.

## Do not infer expected Record shape in Rust

The semantic analyzer must produce the canonical type. Rust asserts it.

## Do not use display-string equality as semantic evidence

`TypeStore::format_type` may appear in assertion failure messages only.

## Do not make all fixtures load rows

The existing isolated Either/Monad/Expression builders remain row-independent.

## Do not add GADT/Record tests while the scalar Expression suite is still an active semantic incident

The correct response is a gated checkpoint, not weaker assertions.

## Do not convert a composite row contradiction to generic `Dynamic`

`Dynamic` is not an error-recovery mechanism.

## Do not add new production semantics merely because a composite test fails

A composite test failure is an incident. Trace whether it exposes:
- a product defect;
- a fixture defect;
- a still-unimplemented semantic prerequisite.

Crossing from `phalcom-core/tests/...` into production code requires a separately diagnosed product incident.

---

# 6. Target Repository Topology

Target additions:

```text
phalcom-core/tests/core/typing_integration/
├── mod.rs
├── support.rs
├── integration.rs
├── row_integration.rs                  # NEW: non-GADT composite row tests
│
├── sources/
│   ├── either.ph
│   ├── monads.ph
│   ├── integration_probes.ph
│   ├── expression.ph
│   ├── ...
│   ├── rows/
│   │   └── core.ph                     # existing sole row helper source
│   │
│   ├── row_integration_probes.ph       # NEW
│   ├── row_integration_invalid.ph      # NEW, or split if diagnostics demand
│   ├── expression_row_probes.ph        # NEW, gated
│   └── expression_row_runtime_probes.ph# OPTIONAL/GATED, see C5
│
└── expression/
    ├── mod.rs
    ├── ...
    └── rows.rs                         # NEW, gated
```

Do **not** move existing isolated row files.

---

# 7. Proposed Composite Law Families

Extend `LAWS.md` with a distinct family rather than overloading isolated `ROW-*` laws.

Recommended namespace:

```text
INT-ROW-*
```

These laws describe cross-feature composition. The existing isolated `ROW-*` laws continue to describe row-specific behavior.

## INT-ROW-01 — direct Either preserves row-specialized output

Input:

```text
Either<String, InputRecord>
```

where:

```text
InputRecord =
#{
    cached: Bool,
    name: String,
    value: Int
}
```

Inside `Either.map`, invoke:

```phalcom
RowCalculus.annotate(record, |value| { value > 0 })
```

Expected:

```text
Either<String, OutputRecord>
```

where:

```text
OutputRecord =
#{
    cached: Bool,
    mapped: Bool,
    name: String,
    value: Int
}
```

## INT-ROW-02 — nested ADT can be an ordinary Record field generic

Call:

```phalcom
RowCalculus.preserveValue(
    #{
        value: someEither,
        cached: true,
        label: "payload"
    }
)
```

with:

```text
someEither : Either<String,Int>
```

Prove:

```text
T = Either<String,Int>
R contributes cached + label
```

and the returned Record contains exactly that nested canonical `Either`.

## INT-ROW-03 — Monad bind solves HKT F and Record A/B simultaneously

For:

```text
F = <X> =>> Either<String,X>
A = InputRecord
B = OutputRecord
```

`MonadAlgorithms.bind` must produce:

```text
Either<String,OutputRecord>
```

while the continuation calls `RowCalculus.annotate`.

## INT-ROW-04 — type-lambda application preserves canonical Record identity

The exact inferred `F` from `MonadAlgorithms.bind` must be a canonical unary `TypeData::Lambda`.

Applying that canonical constructor to `OutputRecord` must yield the same canonical family/type used by the final result:

```text
apply_type_form(F, [OutputRecord])
    ==
Either<String,OutputRecord>
```

This is a test-side use of the **existing canonical TypeStore application**, not a new semantic solver.

## INT-ROW-05 — traverse preserves Record beneath List and Either

Given:

```text
List<InputRecord>
```

run existing:

```text
MonadAlgorithms.traverse
```

with `StringEitherMonad`.

Expected:

```text
Either<String,List<OutputRecord>>
```

The test must assert the nested `List` element is the exact `OutputRecord` TypeId.

## INT-ROW-06 — expected outer ADT result can solve ordinary + row generic dimensions

Add a test helper source such as:

```phalcom
class CompositeRowInference {
    @class
    make<E, T, R: RecordRow>() -> Either<E, #{ value: T, | R }> {
        throw Error.new("type-level inference probe")
    }
}
```

Then:

```phalcom
let result:
    Either<
        String,
        #{
            source: String,
            value: Int
        }
    >
    =
    CompositeRowInference.make()
```

Prove:

```text
E = String
T = Int
R = #{ source: String }
```

`E` and `T` use existing exact generic-solution proof inspection. `R` is verified through the exact canonical specialized result Record; do not fabricate a row-solution explanation if the semantic product does not publish one.

## INT-ROW-07 — row contradiction inside Either map fails closed

Use:

```text
Either<String, #{ name: String, tag: String }>
```

and map with:

```phalcom
|record| { RowCalculus.tagged(record) }
```

The inner call must emit:

```text
RecordRowLacksViolation
```

The outer result must not become a known `Either<...>` through error recovery.

## INT-ROW-08 — row contradiction inside Monad bind remains a row contradiction

Put the same collision inside a `MonadAlgorithms.bind` continuation.

Require:

- row-specific root diagnostic remains present;
- no `Dynamic`;
- no successful `F<B>`;
- outer call is invalid/suppressed according to existing causal propagation.

## INT-ROW-09 — same canonical Record survives ADT/HKT consumer boundaries

Compare the `TypeId` for `OutputRecord` observed in:

1. `RowCalculus.annotate` result;
2. `Either<String,OutputRecord>` argument;
3. `MonadAlgorithms.bind` solved `B`;
4. nested `List<OutputRecord>` element.

They must be the same canonical semantic Record type.

## INT-ROW-10 — existing scalar integration remains unchanged

`integration::direct_either_and_monad_paths_share_one_canonical_source` remains green.

This ensures the Record amendment adds composition without replacing the existing scalar baseline.

---

# 8. GADT/Expression Composite Laws — Gated

These laws become mandatory only after the scalar Expression suite is green.

## INT-ROW-GADT-01 — Record may be exact Expression result index

Construct:

```text
Expression<
    <X> =>> Either<String,X>,
    InputRecord
>
```

using existing `Expression::Pure` or another appropriate constructor.

## INT-ROW-GADT-02 — Expression::Map changes a Record index through a row-polymorphic transform

Transform:

```text
InputRecord
    ->
OutputRecord
```

inside `Expression::Map`.

Expected:

```text
Expression<
    <X> =>> Either<String,X>,
    OutputRecord
>
```

## INT-ROW-GADT-03 — eval carries exact row-specialized T through F<T>

Evaluate the prior expression with `StringEitherMonad`.

Expected:

```text
Either<String,OutputRecord>
```

The exact `OutputRecord` nested under `Expression` must be the same canonical `TypeId` nested under `Either`.

## INT-ROW-GADT-04 — traverse flagship with Record payload

The strongest semantic flagship becomes:

```text
List<InputRecord>
        |
        | MonadAlgorithms.traverse
        | using StringEitherExpressionMonad
        v
Expression<
    <X> =>> Either<String,X>,
    List<OutputRecord>
>
        |
        | ExpressionEvaluation.eval
        | using StringEitherMonad
        v
Either<
    String,
    List<OutputRecord>
>
```

The transformation used by `traverse` must exercise row polymorphism rather than constructing `OutputRecord` manually.

## INT-ROW-GADT-05 — row failure survives GADT/higher-order propagation

A `RowCalculus.tagged` collision inside an `Expression::Map` transform must remain an explicit row failure rather than:
- a successful GADT value;
- `Dynamic`;
- an unexplained unknown GADT index.

---

# 9. Checkpoint Map

| Checkpoint | Tasks | Semantic boundary | Required evidence | Deferred evidence |
|---|---:|---|---|---|
| C0 | 1–2 | Current row, Either, Monad, and Expression prerequisites are classified against the live checkout | Git state; focused package baselines | no new composite code |
| C1 | 3–4 | Shared fixture/source architecture can express nested Record types without a second authority | compile + focused harness test | composite semantics |
| C2 | 5–7 | Direct ADT + row + higher-order generic composition works and fails closed | `INT-ROW-01`, `02`, `07` | Monad/HKT, GADT |
| C3 | 8–10 | HKT/type-lambda/Monad/collection composition preserves exact Record A/B | `INT-ROW-03..05`, `08..10` | GADT |
| C4 | 11–12 | Expected-result inference can solve outer ADT ordinary generics together with a row-specialized result | `INT-ROW-06` + hostile underconstraint comparator | GADT/runtime |
| C5 | 13–15 | Record payloads participate in Expression/GADT indexing and the flagship traverse/eval pipeline | existing Expression suite green + `INT-ROW-GADT-*` | runtime if unsupported |
| C6 | 16–17 | Law/docs/state/deletion gates and broad package compatibility are complete | full non-GADT composite suite; GADT suite when prerequisite green; negative searches | unrelated workspace baseline |

---

# 10. Checkpoint C0 — Baseline and Prerequisite Classification

Tasks:
- Task 1 — establish local Git state and repository drift
- Task 2 — re-run the exact feature foundations this amendment composes

## Why this is a checkpoint

The current remote repository has advanced from the original row-integration commit. The row semantic repair is now recorded complete, while Expression has a different semantic blocker than before. The implementing checkout must establish what is actually green before adding composition tests.

## Entry conditions

- no composite-row amendment edits have started.

## Working set

Primary:
- `phalcom-core/tests/core/typing_integration/mod.rs`
- `support.rs`
- `integration.rs`
- `rows/`
- `expression/`
- `sources/either.ph`
- `sources/monads.ph`
- `sources/rows/core.ph`
- `sources/expression.ph`
- `docs/impl/semantic/typing-integration/typing-integration-implementation-state.md`

Secondary:
- production semantic files only if baseline evidence contradicts the recorded state.

Out of scope:
- changing semantic implementation;
- fixing Expression during C0.

## Semantic contract established

C0 establishes evidence, not new semantics.

## Required evidence

```bash
git rev-parse --show-toplevel
git branch --show-current
git rev-parse HEAD
git status --short
git log -8 --oneline -- \
  phalcom-core/tests/core/typing_integration \
  phalcom-semantic/src/checker/row_inference.rs \
  phalcom-semantic/src/checker/call.rs
```

Record whether local HEAD equals:

```text
e932aac4e21a5b346e719ede5a24f94e7b924ab3
```

Then:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::rows:: -- --nocapture
```

Proves isolated row integration remains green.

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::either:: -- --nocapture
```

Proves canonical Either foundation.

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::monads:: -- --nocapture
```

Proves HKT/Monad foundation.

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::integration:: -- --nocapture
```

Proves existing scalar cross-feature integration.

Finally:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::expression:: -- --nocapture
```

Classify:

- `PASS` → C5 may execute after C4;
- `BASELINE/PRODUCT INCIDENT` matching the current GADT elimination state → C1–C4 proceed; C5 remains gated.

## Do not run yet

```bash
cargo test --workspace --all-targets
```

## Escalate immediately if

- isolated rows regress;
- Either or Monad baseline is red;
- `support.rs` is no longer the fixture authority;
- current source topology already contains composite row tests not covered here.

## Checkpoint completion

- [ ] local revision recorded
- [ ] dirty relevant files recorded
- [ ] rows green
- [ ] Either green
- [ ] Monad green
- [ ] scalar integration green
- [ ] Expression state classified
- [ ] implementation-state amendment section opened
- [ ] no unexplained baseline incident

---

## Task 1 — Record Local Execution State

Purpose:
Anchor the amendment to the actual checkout.

Risk:
- Semantic: LOW
- Implementation fanout: local

Owned files:
- no production/test code.

Edit operations:
1. Run C0 Git commands.
2. Record branch, HEAD, status.
3. If local HEAD differs from plan baseline, inspect only changed typing-integration paths first.
4. Adapt mechanics, not semantic law design.

Testing classification:
No standalone test.

---

## Task 2 — Re-Certify Composition Foundations

Purpose:
Ensure a future composite failure can be attributed to composition rather than a pre-existing feature regression.

Risk:
- Semantic: MEDIUM
- Implementation fanout: evidence only

Source of truth:
Existing focused suites.

Must not:
- fix Expression opportunistically;
- weaken a baseline assertion.

Testing classification:
C0 evidence only.

---

# 11. Checkpoint C1 — Shared Composite Source and Nested Record Assertion Surface

Tasks:
- Task 3 — add composite source builders without contaminating focused fixtures
- Task 4 — extend the shared type-shape assertion DSL for nested Records

## Why this is a checkpoint

Composite tests become cumbersome and error-prone if every test manually unwraps `Either` → `List` → `Record`. This checkpoint adds a **test-only inspection convenience**, not a semantic implementation.

## Entry conditions

- C0 COMPLETE.

## Working set

Primary:
- `typing_integration/support.rs`
- `typing_integration/mod.rs`

Out of scope:
- source probes;
- semantic production code.

## Semantic contract established

- composite source assembly is explicit;
- existing focused builders remain unchanged;
- nested structural types can be asserted canonically.

## Semantic risks

- accidentally loading rows into `either_source()`/`monads_source()`;
- introducing display-string comparison;
- making the shape DSL calculate row semantics.

## Required evidence

```bash
cargo check -p phalcom-core --test core
```

Proves test helper/module API compiles.

Then one small harness-only test may be added if necessary to prove nested Record matching; otherwise C2's first real composite test is sufficient.

## Negative fixture gate

Inspect:

```bash
rg -n 'ROWS_CORE_SOURCE|ROW_INTEGRATION' \
  phalcom-core/tests/core/typing_integration/support.rs
```

Required:
- `either_*` builders remain row-free;
- `monad_*` builders remain row-free;
- `expression_*` builders remain row-free;
- only new composite builders add `ROWS_CORE_SOURCE`.

## Checkpoint completion

- [ ] source builders added
- [ ] existing builders unchanged semantically
- [ ] nested Record assertion surface added
- [ ] compile green
- [ ] no second fixture authority
- [ ] state updated

Suggested commit:

```text
test(core): prepare composite record typing fixtures
```

---

## Task 3 — Add Explicit Composite Source Builders

Purpose:
Create named source compositions matching the new semantic boundaries.

Risk:
- Semantic: LOW
- Implementation fanout: local

Owned file:
`phalcom-core/tests/core/typing_integration/support.rs`

Add constants:

```rust
const ROW_INTEGRATION_PROBES: &str =
    include_str!("sources/row_integration_probes.ph");

const ROW_INTEGRATION_INVALID: &str =
    include_str!("sources/row_integration_invalid.ph");

const EXPRESSION_ROW_PROBES: &str =
    include_str!("sources/expression_row_probes.ph");
```

Add builders:

```rust
pub fn row_integration_source() -> String {
    format!(
        "{EITHER_SOURCE}\n{MONADS_SOURCE}\n{ROWS_CORE_SOURCE}\n{ROW_INTEGRATION_PROBES}"
    )
}

pub fn row_integration_invalid_source() -> String {
    format!(
        "{EITHER_SOURCE}\n{MONADS_SOURCE}\n{ROWS_CORE_SOURCE}\n{ROW_INTEGRATION_INVALID}"
    )
}

pub fn expression_row_source() -> String {
    format!(
        "{EITHER_SOURCE}\n{MONADS_SOURCE}\n{ROWS_CORE_SOURCE}\n{EXPRESSION_SOURCE}\n{EXPRESSION_ROW_PROBES}"
    )
}
```

Code confidence:
**EXACT architecture; STRUCTURAL formatting.**

Must not change:

```rust
either_source()
monads_source()
expression_source()
rows_core_source()
with_rows(...)
```

unless only a mechanical helper extraction is needed.

Update root `mod.rs`:

```rust
mod row_integration;
```

Do not register `expression::rows` until C5 is actually executable.

Testing classification:
No standalone behavior; validated by C2.

---

## Task 4 — Extend `Ty` With Closed Record Shape Support

Purpose:
Allow exact nested assertions such as:

```text
Either<String,List<OutputRecord>>
```

without open-coding `TypeData` traversal in every test.

Risk:
- Semantic: LOW
- Implementation fanout: local test helper

Owned symbols:
- `support.rs::Ty`
- `Fixture::assert_type`
- helper constructors near `nominal`, `either`, `tuple`

Current:

```rust
pub enum Ty<'a> {
    Nominal(&'a str),
    Applied(&'a str, Vec<Ty<'a>>),
    Tuple(Vec<Ty<'a>>),
}
```

Target:

```rust
pub enum Ty<'a> {
    Nominal(&'a str),
    Applied(&'a str, Vec<Ty<'a>>),
    Tuple(Vec<Ty<'a>>),
    ClosedRecord(Vec<(&'a str, Ty<'a>)>),
}
```

Add:

```rust
pub fn record<'a>(
    fields: impl IntoIterator<Item = (&'a str, Ty<'a>)>
) -> Ty<'a> {
    Ty::ClosedRecord(fields.into_iter().collect())
}
```

`assert_type` branch:

1. require `TypeData::Record(row_id)`;
2. require `RecordRowTail::Closed`;
3. require exact field count;
4. compare canonical stored field order to expected field order;
5. recursively call `assert_type` for each field type.

Callers should provide canonical sorted field order, matching existing `assert_closed_record` discipline.

Do **not** add row inference or open-row solving to `Ty`.

Open Record assertions remain in:

```rust
assert_open_record(...)
```

because open tails carry exact stable `TypeParameterId`, which does not fit the small source-shape DSL as cleanly.

Testing classification:
Mechanical helper; C2 tests prove it.

---

# 12. Checkpoint C2 — Direct ADT + Record + Higher-Order Row Composition

Tasks:
- Task 5 — add direct Either<Record> map scenario
- Task 6 — add nested ADT-inside-Record scenario
- Task 7 — add direct Either hostile row collision

## Why this is a checkpoint

This is the smallest cross-feature semantic boundary: generic ADT + higher-order closure + row-polymorphic function.

It deliberately does not involve Monad/HKT yet, making failures easy to attribute.

## Entry conditions

- C1 COMPLETE.
- `typing_integration::rows` green.
- `typing_integration::either` green.

## Working set

Primary:
- new `sources/row_integration_probes.ph`
- new `sources/row_integration_invalid.ph`
- new `row_integration.rs`
- existing `sources/either.ph`
- existing `sources/rows/core.ph`

Secondary:
- `support.rs` only if assertions genuinely need a missing generic helper.

Out of scope:
- production semantic changes;
- monadic algorithms;
- Expression.

## Semantic contract established

After C2:

> A row-specialized structural Record can move through canonical `Either` higher-order APIs without losing fields, type identity, or explicit row failure semantics.

## Semantic risks

- `Either.map` widens output to `Dynamic`;
- inner row transform succeeds but outer result loses extra fields;
- outer ADT uses a distinct/noncanonical Record type;
- row collision is hidden by outer generic inference.

## Hostile cases

Mandatory:
- `tagged` collision inside `Either.map`.

## Required evidence

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::row_integration::direct_either_ \
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
```

## Do not run yet

- Monad package;
- Expression;
- workspace.

## Escalate immediately if

- direct `Either.map` with scalar payload still works but Record payload returns `Unknown`;
- a row error becomes only `GenericInferenceConflict`;
- test requires changes outside `phalcom-core/tests`.

## Checkpoint completion

- [ ] INT-ROW-01 passes
- [ ] INT-ROW-02 passes
- [ ] INT-ROW-07 passes
- [ ] exact `Either.map` callable asserted
- [ ] exact `RowCalculus` callable asserted
- [ ] no Dynamic/Unknown escape on positive path
- [ ] row hostile diagnostic remains precise
- [ ] Either regression green
- [ ] state updated
- [ ] no incident

Suggested commit:

```text
test(core): compose Either with structural record rows
```

---

## Task 5 — Direct `Either<Record>` Map

Purpose:
Prove outer ADT generic substitution preserves exact row-specialized result.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file test

Owned files:
- `sources/row_integration_probes.ph`
- `row_integration.rs`

Add source class:

```phalcom
class RowEitherIntegrationProbe {
    @class
    mapRecord(
        _ source: Either<
            String,
            #{
                cached: Bool,
                name: String,
                value: Int
            }
        >
    ) {
        let mapped = source.map(|record| {
            RowCalculus.annotate(
                record,
                |value| { value > 0 }
            )
        })
    }
}
```

Expected `mapped`:

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

Rust assertions:

1. `Fixture::assert_no_errors`.
2. locate `RowEitherIntegrationProbe.mapRecord`.
3. assert binding `mapped` using nested `Ty`.
4. locate `source.map(` and assert exact:
   ```text
   Either.map
   ```
5. assert `Either.map` result generic `R2` equals exact `OutputRecord`.
6. locate inner `RowCalculus.annotate(` and assert exact callable.
7. assert ordinary solutions for `annotate`:
   ```text
   A = Int
   B = Bool
   ```
8. assert row parameter is `RecordRow` kind.
9. assert inner result Record exact.
10. assert outer Either right argument TypeId == inner result Record TypeId.

This last identity comparison is mandatory.

---

## Task 6 — ADT as an Ordinary Generic Field Inside a Row

Purpose:
Prove ordinary generic substitution and row substitution remain orthogonal when the ordinary generic type is itself an applied ADT.

Risk:
- Semantic: HIGH
- Implementation fanout: local test/source

Source shape:

```phalcom
class RowNestedAdtProbe {
    @class
    preserveNested(
        _ payload: Either<String, Int>
    ) {
        let result = RowCalculus.preserveValue(
            #{
                value: payload,
                cached: true,
                label: "nested"
            }
        )
    }
}
```

Expected:

```text
T = Either<String,Int>
R contributes:
    cached: Bool
    label: String
```

Result:

```text
#{
    cached: Bool,
    label: String,
    value: Either<String,Int>
}
```

Required assertions:

- `preserveValue` selected callable exact;
- `T` exact generic solution is canonical `Either<String,Int>`;
- `R` parameter is RecordRow kind;
- returned Record exact;
- nested `Either` origin is the one canonical declaration.

Do not add a new `Either` fixture.

---

## Task 7 — Row Collision Inside `Either.map`

Purpose:
Defeat an implementation where the inner row call rejects but outer generic `Either.map` still publishes a plausible result.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file test

Source:

```phalcom
class RowEitherInvalidProbe {
    @class
    collision(
        _ source: Either<
            String,
            #{
                name: String,
                tag: String
            }
        >
    ) {
        let result = source.map(|record| {
            RowCalculus.tagged(record)
        })
    }
}
```

Required outcome:

- exactly one root `RecordRowLacksViolation` for the inner `tagged` call, unless current causal suppression policy deliberately retains one additional downstream diagnostic;
- inner row call:
  - no known result;
  - `Invalid`/formal rejection;
  - not `Dynamic`;
- outer map/binding:
  - must not publish a successful known `Either<String,...>` through error recovery;
  - may be `Invalid` or `Suppressed` according to existing causal propagation;
- no internal incident.

Do not accept a bare `GenericInferenceConflict` as the only error if the row diagnostic is available.

---

# 13. Checkpoint C3 — HKT, Type Lambda, Monad, and Collection Composition

Tasks:
- Task 8 — add `MonadAlgorithms.bind` over Record A/B
- Task 9 — prove canonical type-lambda application to Record
- Task 10 — add `traverse` over `List<Record>` and hostile bind collision

## Why this is a checkpoint

This is the main non-GADT flagship layer. It adds the constructor kind/type-lambda domain to the already proven ADT+row composition.

## Entry conditions

- C2 COMPLETE.
- Monad suite green.

## Working set

Primary:
- `row_integration_probes.ph`
- `row_integration_invalid.ph`
- `row_integration.rs`
- existing `sources/monads.ph`
- `support.rs`

Out of scope:
- modifying `MonadAlgorithms`;
- Expression.

## Semantic contract established

After C3:

> Existing generic HKT/Monad algorithms can infer exact structural Record `A`/`B` values while independently inferring the higher-kinded constructor `F`.

## Semantic risks

- F is correct but A/B widen;
- Record fields survive direct Either but disappear under `F<A>`;
- beta-reduction produces an equivalent-looking but noncanonical Record application;
- `List<Record>` traversal loses element precision.

## Required evidence

Focused tests by exact names, then:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::row_integration:: -- --nocapture
```

Regress:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::monads:: -- --nocapture
```

and:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::integration:: -- --nocapture
```

## Checkpoint completion

- [ ] INT-ROW-03 passes
- [ ] INT-ROW-04 passes
- [ ] INT-ROW-05 passes
- [ ] INT-ROW-08 passes
- [ ] INT-ROW-09 passes
- [ ] INT-ROW-10 passes
- [ ] Monad regression green
- [ ] scalar package integration green
- [ ] state updated
- [ ] no incident

Suggested commit:

```text
test(core): compose record rows with HKT Monad algorithms
```

---

## Task 8 — `MonadAlgorithms.bind` With Record A/B

Purpose:
Exercise the three independent inference dimensions together.

Risk:
- Semantic: HIGH
- Implementation fanout: local test/source

Source:

```phalcom
class RowMonadIntegrationProbe {
    @class
    bindRecord(
        _ monad: StringEitherMonad,
        _ source: Either<
            String,
            #{
                cached: Bool,
                name: String,
                value: Int
            }
        >
    ) {
        let result = MonadAlgorithms.bind(
            monad,
            source,
            |record| {
                let transformed = RowCalculus.annotate(
                    record,
                    |value| { value > 0 }
                )

                let next:
                    Either<
                        String,
                        #{
                            cached: Bool,
                            mapped: Bool,
                            name: String,
                            value: Int
                        }
                    >
                    =
                    Either::Right(transformed)

                next
            }
        )
    }
}
```

Assert:

```text
F = canonical <X> =>> Either<String,X>
A = InputRecord
B = OutputRecord
result = Either<String,OutputRecord>
```

Use exact callable generic parameters:

```rust
callable_generic_parameter(
    "MonadAlgorithms",
    "bind",
    DispatchSide::Class,
    index
)
```

and `assert_generic_solution_exact`.

Expected evidence status should follow current scalar `integration.rs` behavior:
- `F`: assumed from contextual/receiver evidence;
- `A`: inspect actual current result before freezing if repository drift changed provenance;
- `B`: established from continuation result when current semantics support it.

Do not cargo-cult evidence statuses from scalar tests if the Record closure establishes them differently. Inspect trace once, then lock the intended provenance.

---

## Task 9 — Exact Type-Lambda Application With Record

Purpose:
Prove the inferred HKT constructor can consume an exact structural Record without losing canonical identity.

Risk:
- Semantic: HIGH
- Implementation fanout: local Rust assertion

Use the `F` solution from Task 8:

```rust
let constructor =
    fixture.generic_solution_type_for(...);
```

Assert:

```rust
TypeData::Lambda(_)
```

and unary constructor kind.

Clone the snapshot store only as existing test helpers already do for canonical type-form application, then:

```rust
let applied =
    store.apply_type_form(constructor, &[output_record]).unwrap();
```

Assert:

```text
applied
==
family_type(result)
```

or exact `Either<String,OutputRecord>` depending the resulting case/family representation.

This test does **not** solve inference in Rust. It verifies the semantic solution's denotation under the canonical TypeStore operation.

Also inspect lambda free types as existing `integration.rs` does:

```text
String is captured
bound index 0 remains
```

This protects against a constructor solution accidentally specialized directly to a Record-specific nominal type.

---

## Task 10 — Traverse `List<Record>` and Add Monad Hostility

Purpose:
Create the strongest currently executable non-GADT composite program.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file test/source

Positive source:

```phalcom
@classmethod
traverseRecords(
    _ monad: StringEitherMonad,
    _ values: List<InputRecord>
) {
    let result = MonadAlgorithms.traverse(
        monad,
        values,
        |record| {
            let transformed =
                RowCalculus.annotate(
                    record,
                    |value| { value > 0 }
                )

            let next: Either<String, OutputRecord> =
                Either::Right(transformed)

            next
        }
    )
}
```

Use literal Record annotations rather than unsupported aliases unless a current source-level alias is already proven to preserve the exact structure.

Expected:

```text
Either<
    String,
    List<OutputRecord>
>
```

Assert:

```text
F = <X> =>> Either<String,X>
A = InputRecord
B = OutputRecord
List element == same canonical OutputRecord
```

Hostile companion:

```text
MonadAlgorithms.bind
    continuation
        RowCalculus.tagged(record)
```

where the source Record already includes `tag`.

Require:
- `RecordRowLacksViolation`;
- no successful `F<B>`;
- no `Dynamic`;
- outer invalid/suppressed state is causally linked to inner failure.

---

# 14. Checkpoint C4 — Expected-Result Inference Across ADT + Row Domains

Tasks:
- Task 11 — add a deliberately underconstrained composite factory
- Task 12 — add contextual and no-context paired tests

## Why this is a checkpoint

Expected-result inference is a separate semantic risk from argument-driven inference. It deserves a paired positive/negative test so the package proves context solves the intended dimensions without defaulting unsolved rows.

## Entry conditions

- C3 COMPLETE.

## Working set

Primary:
- `row_integration_probes.ph`
- `row_integration.rs`

Secondary:
- existing isolated row expected-result tests for assertion style only.

Out of scope:
- production inference changes.

## Semantic contract established

The outer expected ADT can simultaneously constrain:
- ordinary `E`;
- ordinary `T`;
- row `R`;

without fabricating a solution when expected context is absent.

## Hostile comparator

Same call without expected type must remain underconstrained.

## Required evidence

Run the two expected-result tests directly, then full `row_integration::`.

## Checkpoint completion

- [ ] contextual call solves E/T/R
- [ ] no-context call remains underconstrained
- [ ] row does not default to empty
- [ ] no Dynamic
- [ ] exact specialized outer Either asserted
- [ ] state updated

Suggested commit:

```text
test(core): combine expected ADT and row inference
```

---

## Task 11 — Add Composite Underconstrained Factory

Purpose:
Create one reusable source probe where expected-result typing is the only complete source of generic evidence.

Risk:
- Semantic: HIGH
- Implementation fanout: local source

Add:

```phalcom
class CompositeRowInference {
    @class
    make<E, T, R: RecordRow>()
        -> Either<E, #{ value: T, | R }>
    {
        throw Error.new("typing probe")
    }
}
```

Using `throw` matches the existing test-library pattern for contract-only generic methods and avoids inventing a runtime value for unconstrained `E/T/R`.

Do not add arguments that would accidentally solve the same variables.

---

## Task 12 — Contextual vs Uncontextualized Pair

Positive:

```phalcom
let contextual:
    Either<
        String,
        #{
            source: String,
            value: Int
        }
    >
    =
    CompositeRowInference.make()
```

Assert:

```text
E = String
T = Int
result exact
inner Record exact
R contribution == source:String
Ready
```

Ordinary generic solutions use `GenericSolution`.

The row solution is inferred from the canonical materialized Record.

Negative:

```phalcom
let unresolved = CompositeRowInference.make()
```

Require:
- no concrete result type;
- `Blocked`/underconstrained status;
- generic and/or row underconstraint diagnostics according to current multi-domain policy;
- no empty-row default;
- no Dynamic.

If the analyzer deliberately publishes both ordinary and row underconstraint diagnostics, assert the exact intentional set rather than requiring one arbitrary code.

---

# 15. Checkpoint C5 — GADT/Expression + Record Composite Flagship

Tasks:
- Task 13 — enforce the scalar Expression prerequisite
- Task 14 — add Record-valued Expression index/map/eval tests
- Task 15 — add Record traverse-to-eval flagship and hostile row propagation

## Why this is a checkpoint

This is the highest-risk composition boundary. It combines:
- generic ADT;
- HKT/type lambdas;
- Monad algorithms;
- row polymorphism;
- structural Record canonicalization;
- constructor-local GADT generic variables;
- GADT result-index refinement.

It must not be used to diagnose an already failing scalar Expression implementation.

## Entry conditions

Mandatory:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::expression:: -- --nocapture
```

must be green.

If it is not green:

```text
C5 — GATED
```

Record the exact existing Expression incident and stop this checkpoint.

C0–C4 may still be complete.

## Working set

Primary:
- `expression/mod.rs`
- new `expression/rows.rs`
- `sources/expression.ph`
- new `sources/expression_row_probes.ph`
- `support.rs`

Secondary:
- current Expression semantic probe/test files for style.

Out of scope:
- repairing GADT elimination;
- changing variant generics;
- changing Expression source architecture.

## Semantic contract established

Once complete:

> Structural Records may be exact GADT result indices and remain exact through constructor-local generic refinement, HKT application, Monad traversal, and evaluation.

## Required evidence

Scalar prerequisite first.

Then:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::expression::rows:: -- --nocapture
```

Then full Expression suite.

## Checkpoint completion

- [ ] scalar Expression prerequisite green
- [ ] INT-ROW-GADT-01 passes
- [ ] INT-ROW-GADT-02 passes
- [ ] INT-ROW-GADT-03 passes
- [ ] INT-ROW-GADT-04 passes
- [ ] INT-ROW-GADT-05 passes
- [ ] no Dynamic escape
- [ ] exact Record TypeId preserved across Expression and Either
- [ ] state updated
- [ ] no incident

Suggested commit:

```text
test(core): compose Expression GADTs with structural records
```

---

## Task 13 — Gate on Scalar Expression Semantics

Purpose:
Prevent row tests from becoming a disguised GADT implementation project.

Risk:
- Semantic: LOW
- Implementation fanout: evidence only

If scalar suite fails:
- record exact failures;
- compare with current SC-4.8 implementation state;
- do not create `expression/rows.rs` yet.

If green:
- proceed.

---

## Task 14 — Record-Valued Expression Index and Map/Eval

Purpose:
Prove GADT `T` can be a structural Record.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file test/source

Add `expression_row_probes.ph` with a probe shaped approximately as:

```phalcom
class ExpressionRowProbe {
    @class
    mapAndEvaluate(
        _ monad: StringEitherMonad,
        _ source:
            Expression<
                <X> =>> Either<String, X>,
                InputRecord
            >
    ) {
        let mapped:
            Expression<
                <X> =>> Either<String, X>,
                OutputRecord
            >
            =
            Expression::Map(
                source,
                |record| {
                    RowCalculus.annotate(
                        record,
                        |value| { value > 0 }
                    )
                }
            )

        let result =
            ExpressionEvaluation.eval(
                monad,
                mapped
            )
    }
}
```

Do not introduce aliases `InputRecord`/`OutputRecord` if alias syntax would reduce repository certainty. Inline structural Record types are acceptable in this test source.

Assertions:

```text
source index == InputRecord
mapped index == OutputRecord
eval result == Either<String,OutputRecord]
```

and:

```text
mapped index TypeId
==
Either result right-argument TypeId
```

Also assert:
- exact `Expression::Map` constructor target/provenance where the current fixture exposes it;
- exact `RowCalculus.annotate` callable;
- exact inner ordinary A/B solutions.

---

## Task 15 — Full `List<Record>` Traverse → Expression → Either Flagship

Purpose:
Make Records part of the package's strongest classic typing scenario.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Target:

```text
List<InputRecord>
        |
        | MonadAlgorithms.traverse
        | using StringEitherExpressionMonad
        | transform each element through RowCalculus.annotate
        v
Expression<
    <X> =>> Either<String,X>,
    List<OutputRecord>
>
        |
        | ExpressionEvaluation.eval
        | using StringEitherMonad
        v
Either<
    String,
    List<OutputRecord>
>
```

Assertions must cover both intermediate and final type.

For `traverse`:

```text
F = canonical <X> =>> Expression<<Y> =>> Either<String,Y>,X>
A = InputRecord
B = OutputRecord
```

For intermediate:

```text
Expression effect = canonical Either<String,_> lambda
index = List<OutputRecord>
```

For eval:

```text
result = Either<String,List<OutputRecord>>
```

Canonical consistency:

```text
OutputRecord from row transform
==
traverse B
==
List element in Expression index
==
List element in final Either result
```

Hostile companion:
- input Record already contains `tag`;
- transform calls `RowCalculus.tagged`;
- require `RecordRowLacksViolation`;
- no successful GADT index publication;
- no Dynamic.

This is the flagship composite row test.

---

# 16. Runtime Policy

This amendment is primarily a **typing** integration amendment.

Do not automatically expand it into new Record runtime implementation.

Before adding a Record composite VM probe, investigate:

1. whether current compiler `RecordLiteral` lowering executes the exact required source;
2. whether Record expansion used by `RowCalculus.annotate` is executable;
3. whether `Either`/Monad runtime path accepts Record payload objects without new production work.

If all are already implemented, a runtime composite probe may be added after C5 as an optional C5-R subtask:

```text
semantic preflight
    ↓
compile same source
    ↓
VM
    ↓
primitive observations
```

Preferred observations:
- final Either is Right;
- list size;
- mapped Bool;
- preserved Int/String fields.

Do not require Rust-side deep Record decoding if no existing helper supports it.

If runtime support is missing, classify:

```text
GATED BY RUNTIME FEATURE
```

and do **not** broaden this typing amendment into compiler/VM feature implementation.

Semantic completion of this amendment does not depend on adding new runtime Record machinery.

---

# 17. Checkpoint C6 — Documentation, Migration Closure, and Delivery

Tasks:
- Task 16 — update `LAWS.md`, `README.md`, and implementation state
- Task 17 — execute negative/deletion and broad compatibility gates

## Entry conditions

- C4 COMPLETE.
- C5 either:
  - COMPLETE, or
  - explicitly GATED by a pre-existing Expression semantic prerequisite.

A fully complete **GADT composite** claim requires C5 COMPLETE.

## Documentation changes

### `LAWS.md`

Update title:

```text
Unified Typing Integration Laws — GEN, MON, ROW, GEX, and INT
```

Add:
- `INT-ROW-01..10`
- `INT-ROW-GADT-01..05`

Map each law to exact Rust test names after implementation.

Do not renumber existing `INT-00..03`.

### `README.md`

Add `rows/` to focused sub-suites.

Explain two row layers:

```text
rows/
    isolated complex row semantics

row_integration.rs
    composite ADT/HKT/Monad/Record tests

expression/rows.rs
    GADT + Record composition, when prerequisite is green
```

State that `sources/rows/core.ph` is the single reusable row helper source.

### implementation state

Continue using:

```text
docs/impl/semantic/typing-integration/
typing-integration-implementation-state.md
```

Add a section:

```markdown
## Composite Record/Row Typing Amendment
```

Do not create another state file.

---

# 18. Final Negative/Deletion Gates

## One `Either`

```bash
rg -n '^\s*enum Either<L, R>' \
  phalcom-core/tests/core/typing_integration
```

Expected:
one canonical source declaration in `sources/either.ph`.

## One reusable RowCalculus

```bash
rg -n '^\s*class RowCalculus\b' \
  phalcom-core/tests/core/typing_integration
```

Expected:
one source declaration in `sources/rows/core.ph`.

## No row-specific Monad hierarchy

```bash
rg -n 'class .*Row.*Monad|class RowMonad|bindRecord<|traverseRecord<' \
  phalcom-core/tests/core/typing_integration/sources
```

Inspect every hit.

Expected:
no competing abstraction.
Probe method names such as `bindRecord` are allowed; new algorithm definitions are not.

## No second support file

```bash
find phalcom-core/tests/core/typing_integration \
  -path '*row*' -name support.rs -print
```

Expected zero hits.

## No solver leakage

```bash
rg -n 'RecordRowSolver|RecordRowVarId|GenericApplicationSession' \
  phalcom-core/tests/core/typing_integration
```

Expected zero production-test references.

## Existing focused builders remain isolated

Inspect:

```bash
rg -n 'fn (either|monad|expression).*source|ROWS_CORE_SOURCE' \
  phalcom-core/tests/core/typing_integration/support.rs
```

Required:
basic Either/Monad/Expression builders do not load row sources.

## No nominal Record workaround

Inspect new `.ph` files for payload declarations.

The positive composite payloads must use:

```text
#{ ... }
```

not a nominal wrapper.

---

# 19. Verification Schedule

Use smallest-first evidence.

## C2 direct ADT/row

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::row_integration::direct_either_ \
  -- --nocapture
```

## C3 HKT/Monad/Record

Run exact bind/traverse tests, then:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::row_integration:: -- --nocapture
```

## Focused foundations

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::rows:: -- --nocapture

RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::either:: -- --nocapture

RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::monads:: -- --nocapture

RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::integration:: -- --nocapture
```

## GADT checkpoint

Only once scalar Expression is green:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::expression::rows:: -- --nocapture

RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::expression:: -- --nocapture
```

## Full package

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration:: -- --nocapture
```

Interpretation:
- if C5 is gated by a recorded Expression baseline, do not claim full-package green;
- all newly added non-GADT Record composite tests must still be green.

## Core target

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core
```

## Format

```bash
cargo fmt --all -- --check
```

Classify repository baseline formatting separately if the current branch retains unrelated drift.

## Workspace check

```bash
RUSTFLAGS='' cargo check --workspace --all-targets
```

## Clippy

```bash
RUSTFLAGS='' cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Classify known pre-existing lints separately.

## Workspace tests

```bash
RUSTFLAGS='' cargo test --workspace --all-targets
```

Run only at final delivery if required by current project policy; do not repeatedly run it per checkpoint.

---

# 20. Failure Protocol

If a new composite test fails, stop before modifying production semantics.

## Exact reproduction

Record:
- exact command;
- exact test;
- expression status;
- type knowledge;
- diagnostics;
- selected callable if any.

## Direct path examples

### Either<Record>

```text
InputRecord
→ Either<String,InputRecord>
→ Either.map
→ closure
→ RowCalculus.annotate
→ OutputRecord
→ Either<String,OutputRecord>
```

### Monad<Record>

```text
StringEitherMonad
→ MonadAlgorithms.bind
→ solve F
→ solve A = InputRecord
→ continuation
→ RowCalculus.annotate
→ solve B = OutputRecord
→ beta-apply F<B>
→ Either<String,OutputRecord>
```

### GADT<Record>

```text
Expression<F,InputRecord>
→ Expression::Map
→ constructor-local A/B
→ RowCalculus.annotate
→ Expression<F,OutputRecord>
→ ExpressionEvaluation.eval
→ F<OutputRecord>
→ Either<String,OutputRecord>
```

## Passing comparator

Use the nearest lower layer.

Examples:

```text
RowCalculus.annotate passes
Either<Record> fails
    → ADT composition

Either<Record> passes
Monad bind<Record> fails
    → HKT/Monad composition

Monad<Record> passes
Expression<Record> fails
    → GADT composition

scalar Expression fails too
    → baseline GADT prerequisite, not row composition
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

## Narrow write boundary

Default:

```text
phalcom-core/tests/core/typing_integration/**
docs/impl/semantic/typing-integration/**
```

Crossing into:

```text
phalcom-semantic/src/**
phalcom-ast/**
compiler
VM
```

requires an explicit product incident with evidence.

---

# 21. State-File Protocol

After every checkpoint record:

```markdown
## Composite Record/Row Typing Amendment

### Established invariants
- CRI-01: ...
- CRI-02: ...

### Decisions
- CRD-01: `sources/rows/core.ph` remains the one row helper authority.
- CRD-02: composite builders load rows explicitly.
- CRD-03: GADT composite work is gated on scalar Expression green.

### Evidence

| Checkpoint | Command | Result | Proves |
|---|---|---|---|

### Hostile cases
- ...

### Negative gates
- ...

### Deferred
- Expression/GADT composite → C5 if baseline remains incident.
- runtime composite → existing runtime support gate.

### Active incident
None.

### Next resume action
Begin C<N>.
```

Do not delete historical row semantic repair evidence.

---

# 22. Checkpoint Completion Report

Example:

```text
Checkpoint C3 COMPLETE

Established:
    Existing Monad/HKT algorithms preserve exact structural
    Record A/B while independently solving constructor F.

Changed:
    sources/row_integration_probes.ph
    row_integration.rs

Evidence:
    bind Record composite — PASS
    type-lambda Record application — PASS
    traverse List<Record> — PASS
    full row_integration module — PASS
    monads regression — PASS

Hostile:
    row collision inside bind — PASS
    no Dynamic escape — PASS

Negative:
    no RowMonad algorithm introduced
    no solver-local row type used by tests

Deferred:
    Expression/GADT Records → C5

Next:
    C4 expected-result ADT + row inference.
```

---

# 23. Suggested Commit Groups

```text
C1
test(core): prepare composite record typing fixtures

C2
test(core): compose Either with structural record rows

C3
test(core): compose record rows with HKT Monad algorithms

C4
test(core): combine expected ADT and row inference

C5
test(core): compose Expression GADTs with structural records

C6
docs(test): catalog composite record typing laws
```

Do not force one commit per test.

---

# 24. Known Scope Exclusions

This amendment does not implement:

- new row solver behavior;
- new `lacks` semantics;
- new generic-return publication rules;
- `RecordRowVarId` reflection/publication;
- row-kinded nominal type arguments;
- row-kinded type-lambda parameters;
- explicit `where R lacks field` syntax;
- generic getters/setters;
- a new `Either`;
- a new Monad hierarchy;
- new `MonadAlgorithms`;
- new GADT semantics;
- constructor-local GADT elimination fixes;
- new Record runtime representation;
- Map-as-Record semantics;
- nominal structural typing;
- runtime Record feature work if current compiler/VM support is insufficient.

If a composite test exposes one of these missing production capabilities, stop and create a separately authorized product-repair plan.

---

# 25. Checkpoint Evidence Summary

| Checkpoint | Semantic contract | Status at plan time |
|---|---|---|
| C0 | current foundations/prerequisites classified | PENDING |
| C1 | explicit composite source + nested Record assertion authority | PENDING |
| C2 | direct Either/ADT + row composition | PENDING |
| C3 | HKT/type-lambda/Monad/List + row composition | PENDING |
| C4 | expected-result ADT + row inference composition | PENDING |
| C5 | GADT/Expression + row flagship | GATED until scalar Expression green |
| C6 | docs/deletion/broad delivery | PENDING |

No status may be changed to COMPLETE without recorded command evidence.

---

# 26. Release-Complete Criteria

The **non-GADT composite row amendment** is complete only when:

- [ ] C0–C4 are COMPLETE;
- [ ] new composite builders exist without contaminating focused fixtures;
- [ ] `Ty` can assert nested closed Records canonically;
- [ ] direct `Either<Record>` mapping preserves exact Record fields;
- [ ] nested `Either` can be an ordinary generic Record field;
- [ ] `MonadAlgorithms.bind` solves exact `F`, Record `A`, Record `B`;
- [ ] canonical type-lambda application to `OutputRecord` yields the final exact Either type;
- [ ] `MonadAlgorithms.traverse` preserves `OutputRecord` under `List` and `Either`;
- [ ] expected-result inference solves ordinary `E/T` and row `R` together;
- [ ] same call without expected context remains underconstrained;
- [ ] row collision inside `Either.map` fails with precise row diagnostics;
- [ ] row collision inside Monad composition fails without Dynamic;
- [ ] canonical Record TypeId is identical across inner row result and outer generic containers;
- [ ] existing rows suite remains green;
- [ ] existing Either suite remains green;
- [ ] existing Monad suite remains green;
- [ ] existing scalar integration remains green;
- [ ] no duplicate Either/Monad/RowCalculus source exists;
- [ ] no row solver API leaks into `phalcom-core` tests;
- [ ] `LAWS.md`, README, and state file are updated.

The **full composite row amendment including GADT** is complete only when all of the above are true **and**:

- [ ] scalar `typing_integration::expression::` is green;
- [ ] C5 is COMPLETE;
- [ ] structural Record is accepted as exact `Expression<F,T>` index;
- [ ] `Expression::Map` transforms `InputRecord → OutputRecord` through `RowCalculus`;
- [ ] `ExpressionEvaluation.eval` preserves exact `OutputRecord` through `F<T>`;
- [ ] flagship `List<InputRecord> → Expression<...,List<OutputRecord>> → Either<String,List<OutputRecord>>` passes;
- [ ] hostile row failure inside GADT composition remains explicit and non-Dynamic.

Runtime composite coverage is required only if the existing runtime Record surface supports the program without new feature implementation.

---

# 27. Final Intended Package Model

After this amendment, `typing_integration` should express the following architecture:

```text
FEATURE PRESSURE VESSELS

Either<L,R>
    generic ADT

RowCalculus / #{ ... | R }
    structural row polymorphism

Functor / Applicative / Monad
    HKT + type lambda + generic inheritance

Expression<F,T>
    GADT / indexed result typing

                ↓

PAIRWISE COMPOSITION

Either<String,Record>

Record<
    value: Either<String,Int>,
    ...R
>

MonadAlgorithms.bind<
    F = Either<String,_>,
    A = InputRecord,
    B = OutputRecord
>

                ↓

NESTED GENERIC COMPOSITION

List<InputRecord>
    ↓ traverse
Either<String,List<OutputRecord>>

                ↓

CONTEXTUAL MULTI-DOMAIN INFERENCE

expected
Either<String,#{value:Int,source:String}>>

solves
E + T + R

                ↓

GADT FLAGSHIP

List<InputRecord>
    ↓
MonadAlgorithms.traverse
    ↓
Expression<
    Either<String,_>,
    List<OutputRecord>
>
    ↓
ExpressionEvaluation.eval
    ↓
Either<
    String,
    List<OutputRecord>
>
```

The package therefore becomes a genuine classic type-system integration corpus: each major mechanism is first tested in isolation, then required to coexist with the others under exact canonical semantic assertions.
