# Typing Integration Implementation State

## Baseline
- Plan baseline: `9f04681201e4e15388b4a32d09a2a502486e9367`.
- Execution branch: `main`.
- Execution HEAD at start: `47abba0e5b44d091768748420fd21dd91ae43742`.
- Working-tree note: pre-existing modifications and untracked files are preserved. Relevant target trees were clean at C0. HEAD drift adds only `phalcom-core/tests/core/mod.rs` `vm_support` registration relative to plan baseline.

## Established invariants
- I-01: C0 confirms existing Either and Monad conformance packages are still separate authorities before migration.
- I-02: `phalcom-semantic` remains static semantic authority; `ProgramCompiler`/`VM` remain runtime authority.
- I-03: C0 baseline suites are green: Either 27 passed; Monad 35 passed.
- I-04: C1 nests both focused suites under one `typing_integration` package without source/helper/test-content changes.
- I-05: C2 establishes one root `support.rs`; all source composition routes through explicit root builders.
- I-06: C2 establishes `sources/either.ph` as the only live `Either<L, R>` declaration, and Monads consume that full source.
- I-07: Runtime probe builders remain isolated by feature; Either and Monad runtime bindings are not concatenated.
- I-08: C3 adds one package-level semantic scenario using direct `Either.map` and `MonadAlgorithms.bind` against the same canonical source universe.

## Decisions
- D-01: Adapt only mechanics for local HEAD drift; preserve planned semantic and ownership boundaries.
- D-02: Implement inline in current checkout; no delegation.
- D-03: Preserve unrelated dirty and untracked work.
- D-04: Keep `either/` and `monads/` as focused nested diagnostic sub-suites; defer shared source and support changes to C2.
- D-05: Retain exact Monads `TypeId`/identity generic-solution API; migrate Either callers to canonical `f.ty(...)` values.
- D-06: `monad-testing.md` remains a focused walkthrough, but canonical Either ownership now points to root `sources/either.ph`.
- D-07: Preserve existing MON law IDs/text; root `LAWS.md` references GEN authority and reserves GEX/INT namespaces.

## Evidence ledger

| Checkpoint | Command | Result | Proves |
|---|---|---|---|
| C0 | `git rev-parse --show-toplevel`, branch/HEAD/status/log, scoped drift diff | PASS; `main` at `47abba0`; target Either/Monad trees unchanged; only unrelated `vm_support` registration drift | local execution state and migration contract are known |
| C0 | `RUSTFLAGS='' cargo test -p phalcom-core --test core either:: -- --nocapture` | PASS: 27 passed, 0 failed, 0 ignored; 59.34s | pre-migration Either baseline |
| C0 | `RUSTFLAGS='' cargo test -p phalcom-core --test core monads:: -- --nocapture` | PASS: 35 passed, 0 failed, 0 ignored; 60.64s | pre-migration Monad baseline |
| C1 | `RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration::either:: -- --nocapture` | PASS: 27 passed, 0 failed, 0 ignored; 57.43s | moved Either suite preserves C0 count and behavior |
| C1 | `RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration::monads:: -- --nocapture` | PASS: 35 passed, 0 failed, 0 ignored; 56.27s | moved Monad suite preserves C0 count and behavior |
| C2 | `RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration::either::rejection::conflicting_constructor_context_is_rejected_without_dynamic_escape -- --nocapture` | PASS: 1 passed | hostile Either rejection retains fail-closed semantics |
| C2 | `RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration::monads::rejection::monad_constructor_conflicts_with_unrelated_value_constructor -- --nocapture` | PASS: 1 passed | hostile Monad/HKT constructor conflict retains fail-closed semantics |
| C2 | `RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration::either:: -- --nocapture` | PASS: 27 passed, 0 failed, 0 ignored; 57.25s | full Either subtree survives shared source/harness migration |
| C2 | `RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration::monads:: -- --nocapture` | PASS: 35 passed, 0 failed, 0 ignored; 62.31s | full Monad subtree consumes canonical Either without behavior loss |
| C3 | `RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration::integration::direct_either_and_monad_paths_share_one_canonical_source -- --nocapture` | PASS: 1 passed | direct and Monad paths resolve exact canonical callables, constructor lambda, callable-owned A/B, and Ready results |
| C3 | `RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration:: -- --nocapture` | PASS: 63 passed, 0 failed, 0 ignored; 61.99s | complete unified package remains green |
| C4 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::adts::matching::gadt_refinement -- --nocapture` | PASS: 19 passed, 0 failed, 1 ignored; 1.04s | existing GADT ownership layer is green before Expression integration |
| C4 | `RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration::expression::refinement -- --nocapture` | INCIDENT: fixture parsing fails at `@variant Pure<A>` and subsequent constructor-local generic forms before semantic analysis | current parser accepts `@variant` followed by an identifier, but not the constructor-local generic syntax required by the plan |
| C4 | same focused refinement command after moving every `@variant` marker to its own line | INCIDENT reproduced: first failure remains `A` in `Pure<A>`; separate-line attribute placement does not change parser behavior; independent typed-closure `:` failure remains | newline placement is supported; constructor-local variant generics and typed closure parameter annotations are not |

## Negative/deletion evidence
- C0 scoped drift diff shows no target-package content changes.
- C1 root registration search → zero old top-level `either`/`monads` path registrations.
- C1 moved inventory → all C0 Either/Monad files present under `typing_integration/`.
- C2 `rg -n '^\s*enum Either<L, R>' phalcom-core/tests/core/typing_integration` → one hit: `sources/either.ph`.
- C2 `find phalcom-core/tests/core/typing_integration -name support.rs -print` → one hit: root `support.rs`.
- C2 child include search → zero old local fixture includes.
- C2 `git diff --check` scoped to changed package/state → PASS.
- C3 current-doc old-path search → zero live `phalcom-core/tests/core/either|monads` ownership references.
- C3 MON heading count → 53 preserved headings including all existing MON catalog entries plus INT-00.
- C3 `graphify update .` → PASS; graph rebuilt with 53,636 nodes, 80,674 edges, 4,088 communities; HTML skipped due graph-size limit.

## Deferred gates
- C5: full `phalcom-core` core target, format, workspace check/tests, and clippy.

## Active incident
- C4 Task 11 is blocked at the planned source boundary. The required
  constructor-local forms (`@variant Pure<A>`, `If<A>`, `Map<A, B>`,
  `FlatMap<A, B>`, `Apply<A, B>`, `Lift<A>`) are rejected by the current
  parser before semantic analysis. `phalcom-ast/src/parser.rs::parse_enum_variant`
  consumes the variant identifier, payload, result annotation, and body, with
  no generic-parameter parse step. Replacing these forms with outer `T` would
  lose the required independent A/B relationships; changing parser/AST support
  is explicitly outside C4. The separate-line experiment confirms attribute
  placement is not cause. A second syntax mismatch exists in typed closure
  parameters (`|value: Int|`): `parse_closure_literal` accepts a name followed
  by `,` or `|`, not a type colon. No production files were changed.

## Next resume action
Resolve C4 incident boundary: authorize parser/AST support for constructor-local
variant generics, or revise the Expression source contract/implementation plan.
Do not continue C4 semantic/runtime gates until that boundary is decided.

## Record Rows Amendment

### Baseline
- Plan baseline: `e17f2733f98cb20e2a8ead5794d75ca647a950ce`.
- Execution branch: `main`.
- Execution HEAD: `e17f2733f98cb20e2a8ead5794d75ca647a950ce`.
- Working-tree note: pre-existing plan-document deletion, modification, and untracked companion plan preserved; typing-integration and semantic row source trees were clean before amendment edits.

### Established row integration invariants
- RI-01: `phalcom-semantic` owns row formation, solving, canonicalization, and structural relation semantics; core integration tests consume published semantic products only.
- RI-02: `RecordRow` remains distinct from ordinary `Type`; tests inspect `TypeData::Record(RecordRowId)` and `RecordRowData` without reconstructing remainders.
- RI-03: `typing_integration::support.rs` is the only shared fixture authority; no row-local support or solver is permitted.
- RI-04: row-specific sources explicitly compose `sources/rows/core.ph`; Either, Monad, and Expression base builders remain isolated.

### Decisions
- RD-01: execute inline on current checkout; no delegation.
- RD-02: preserve unrelated dirty and untracked work.
- RD-03: proceed R1–R4 independently of the pre-existing Expression baseline incident; R5 remains blocked until the existing Expression source parses and its scalar suite is green.

### Evidence ledger

| Checkpoint | Command | Result | Proves |
|---|---|---|---|
| R0 | `git rev-parse --show-toplevel`, branch/HEAD/status/log | PASS; `main` at `e17f2733`; only pre-existing plan-document changes are dirty | local execution state and amendment scope are known |
| R0 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::advanced::record_rows -- --nocapture` | PASS: 12 passed, 0 failed, 0 ignored | low-level row domain is green |
| R0 | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::integration::record_row_polymorphism -- --nocapture` | PASS: 9 passed, 0 failed, 0 ignored | source-reachable row-polymorphic behavior is green |
| R0 | `RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration::either:: -- --nocapture` | PASS: 27 passed, 0 failed, 0 ignored | Either foundation is green |
| R0 | `RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration::monads:: -- --nocapture` | PASS: 35 passed, 0 failed, 0 ignored; bounded serial capture exited 0 after 110.15s | Monad/HKT foundation is green |
| R0 | `RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration::expression:: -- --nocapture` | BASELINE: 0 passed, 16 failed; existing shared Expression source parse errors at `@variant Pure<A>` and typed closure forms | Expression baseline incident predates row amendment |
| R0 | `RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration:: -- --nocapture` | BASELINE: 63 passed, 16 failed; same Expression parse incident; exit 101 | complete pre-amendment package baseline |

### Negative/deletion evidence
- R0 target-tree status showed no pre-existing row integration package, source, or helper edits.
- Existing package has no `RowCalculus` declaration and no `rows/` sub-suite.
- R1 `rg -n '^\\s*class RowCalculus\\b' phalcom-core/tests/core/typing_integration` → one hit: `sources/rows/core.ph`.
- R1 search of row tests/support contains no `RecordRowSolver`, `RecordRowVarId`, or `GenericApplicationSession`.
- R1 `graphify update .` → PASS; graph rebuilt with 54,128 nodes, 81,206 edges, 4,177 communities; HTML skipped due graph-size limit.
- R2 `graphify update .` → PASS; graph rebuilt with 54,139 nodes, 81,222 edges, 4,145 communities; HTML skipped due graph-size limit.

### R1 evidence

| Checkpoint | Command | Result | Proves |
|---|---|---|---|
| R1 | `RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration::rows::calculus:: -- --nocapture` | PASS: 6 passed, 0 failed, 0 ignored | ROW-INFER-01..06 pass through source-level generic calls |
| R1 | `RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration::rows:: -- --nocapture` | PASS: 6 passed, 0 failed, 0 ignored | isolated row sub-suite remains coherent |
| R2 | `RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration::rows::correlation:: -- --nocapture` | PASS: 2 passed, 0 failed, 0 ignored | compatible repeated rows correlate and incompatible rows reject |
| R2 | `RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration::rows::transformations:: -- --nocapture` | INCIDENT: 1 passed, 1 failed; collision is `Ready` + `Unknown(InferenceBlocked)` with no diagnostic | disjoint extension works; duplicate extension exposes product publication gap |
| R2 | `RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration::rows::pipelines:: -- --nocapture` | PASS: 1 passed, 0 failed, 0 ignored | canonical Record can be decomposed again under a new prefix |

### Deferred gates
- R2 Task 7 completion and R3–R4: blocked by the duplicate-extension product incident; Expression suite and full package remain deferred.
- R5: blocked by the R0 Expression baseline incident.
- R6: core target, format, workspace check/tests, and Clippy remain deferred until focused amendment evidence exists.

### Active amendment incident
- `BASELINE`: existing `typing_integration::expression::` source does not parse at `support.rs:112` (`@variant Pure<A>` and subsequent constructor-local generic/typed-closure forms). This is outside the amendment write boundary. Do not treat it as a row regression or fix it opportunistically.
- `PRODUCT / R2`: `RowCalculus.tagged(#{ name: "Phalcom", tag: "existing" })` reaches `publish_generic_return_with_rows` after row solving, but materialization failure at `phalcom-semantic/src/checker/call.rs:1500` returns `Unknown(InferenceBlocked)` without recording the row diagnostic; outer expression status remains `Ready`. Required `ROW-XFORM-03` evidence cannot pass. No production semantic edits made.

### Next resume action
Resolve R2 Task 7 product incident: authorize a narrow semantic diagnostic/status repair or revise amendment evidence boundary; then rerun R2 transformation and full-row gates.
