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

## Historical C4 parser incident
- C4 Task 11 is blocked at the planned source boundary. The required
  constructor-local forms (`@variant Pure<A>`, `If<A>`, `Map<A, B>`,
  `FlatMap<A, B>`, `Apply<A, B>`, `Lift<A>`) are rejected by the current
  parser before semantic analysis. `phalcom-ast/src/parser.rs::parse_enum_variant`
  consumes the variant identifier, payload, result annotation, and body, with
  no generic-parameter parse step. Replacing these forms with outer `T` would
  lose the required independent A/B relationships; changing parser/AST support
  was explicitly outside the original package C4 boundary. The separate-line
  experiment confirmed attribute placement was not cause. A second syntax
  mismatch existed in typed closure parameters (`|value: Int|`):
  `parse_closure_literal` accepts a name followed by `,` or `|`, not a type
  colon. The linked SC-4.8 plan supplied the bounded parser/AST/semantic
  remediation recorded below.

## Next resume action
Resolve remaining Expression semantic failure under SC-4.8 C5/C7, or keep the
typing-integration package explicitly partial. Do not continue package
runtime/full-package gates until constructor-local generic elimination is live.

## SC-4.8 bounded variant-generic remediation

- The linked SC-4.8 plan authorizes the bounded declaration slice: `VariantDecl`
  now retains constructor-local generic parameters and `where` clauses; the
  parser accepts them; enum semantics publishes a callable-owned
  `GenericSignature`; and enum input/product fingerprints include that
  metadata.
- `phalcom-ast` syntax checkpoint: PASS, 8 tests passed.
- `semantic::adts::generics` checkpoint: PASS, 9 tests passed, including
  canonical variant-constructor ownership and payload/result scope.
- `semantic::adts::matching::gadt_refinement` dependency checkpoint: PASS,
  19 passed, 1 ignored.
- Expression checkpoint: INCIDENT. After mechanical fixture adaptation for
  unsupported newline placement and typed closure parameter syntax, the
  Expression source reaches semantic analysis but evaluator branches produce
  `GenericInferenceConflict` and `GenericInferenceUnderconstrained` diagnostics;
  the existing GADT engine does not yet consume constructor-local generic
  domains existentially. This is the SC-4.8 C5/C7 construction/elimination
  boundary, not a fixture-only failure.
- Package C4 remains incomplete: no hostile/runtime/full-package green claim.

### Remediation evidence ledger

| Slice | Command | Result | Proves |
|---|---|---|---|
| SC-4.8 declaration remediation | `cargo test -p phalcom-ast --test integration enum_syntax -- --nocapture` | PASS: 8 passed | parser preserves variant-local generic and `where` syntax |
| SC-4.8 declaration remediation | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::adts::generics -- --nocapture` | PASS: 9 passed | callable-owned variant generic metadata and scoped payload/result forms |
| SC-4.8 declaration remediation | `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::adts::matching::gadt_refinement -- --nocapture` | PASS: 19 passed, 1 ignored | existing GADT ownership dependency remains green |
| Expression integration | `RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration::expression:: -- --nocapture` | INCIDENT: semantic generic conflicts after parser boundary was cleared | constructor-local elimination remains unavailable |
| Existing package regression | `RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration::either:: -- --nocapture` | PASS: 27 passed | Either authority unaffected |
| Existing package regression | `RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration::monads:: -- --nocapture` | PASS: 35 passed | Monad/HKT authority unaffected |

### Narrow next action

Continue only under an explicit SC-4.8 C5/C7 production slice. Do not weaken
Expression assertions, introduce `Dynamic`, or duplicate the test source.

### SC-4.8 continuation: C1 canonical multi-domain application

- C1 production slice: PASS. Constructor callable signatures now retain only
  callable-owned binders; declaration-owned binders remain separate canonical
  products and compose only in one query-local inference session.
- C1 semantic gate: PASS, 8 generic-application tests; 6 receiver-specialization
  tests; 9 generic-getter tests.
- C1 hostile constructor gate: PASS. `Box<T>.new<U>(value: T, metadata: U)`
  solves both domains without mixed-owner metadata or `Dynamic` fallback.
- C1 negative search: PASS, `merge_constructor_generic_signatures` has zero
  production occurrences; `git diff --check` passes.
- C1 metadata/product result: callable-local generic publication remains
  owner-valid; declaration generic ownership is not rewritten.

### C1 next action

Route variant construction through same multi-domain application product. C5
must keep enum and variant-local domains separate, preserve `VariantConstructorId`,
and prove generic Family invocation before any C6/C7 rigid/GADT work.

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

## Row semantic repair

### C0 evidence
- Revision: `7ed4f571f16cdd1fdd957740079e42b01d654d4d` on `main`, synchronized with `origin/main`.
- Working tree: clean before repair edits.
- `semantic::advanced::record_rows`: PASS, 12 passed, 0 failed.
- `semantic::integration::record_row_polymorphism`: PASS, 9 passed, 0 failed.
- `typing_integration::rows::transformations::`: INCIDENT reproduced, 1 passed, 1 failed; collision still reports `Ready + Unknown(InferenceBlocked)` with no diagnostics.
- Classification remains `PRODUCT`; no production semantic changes made in C0.

### C0 completion
- C0 COMPLETE.
- C1 is next: signature-wide stable row-lacks extraction and application seeding.

### C1 evidence
- `semantic::foundations::record_row_inference`: PASS, 3 passed, 0 failed.
- `semantic::integration::record_row_polymorphism`: PASS, 10 passed, 0 failed; return-only `tag` lack regression passes.
- `typing_integration::rows::transformations::tagged_collision_is_rejected_without_dynamic_escape`: PASS, 1 passed, 0 failed.
- Signature-wide collector traverses canonical `TypeData`; `RecordRowSolver::add_lacks` is idempotent.
- Negative authority search: only `constrain_signature_type_lacks` calls `add_lacks`; argument decomposition no longer owns implicit lacks.
- `graphify update .`: PASS; graph rebuilt with 54,149 nodes, 81,253 edges, 4,114 communities.

### C1 completion
- C1 COMPLETE.
- C2 is next: preserve and publish structured generic-return materialization failures.

### C2 evidence
- `semantic::foundations::record_row_inference`: PASS, 3 passed, 0 failed.
- `semantic::foundations::record_row_materialization`: PASS, 3 passed, 0 failed.
- `semantic::integration::record_row_polymorphism`: PASS, 10 passed, 0 failed.
- `semantic::foundations::diagnostic_presentation`: PASS, 7 passed, 0 failed.
- `CombinedInferenceFailure::RowZonk` retains stable `TypeParameterId`; row-aware instantiation/materialization failures now record diagnostics/status before returning unavailable knowledge.
- Negative publication search confirms both publication failure branches route through explicit handling.

### C2 completion
- C2 COMPLETE.
- C3 is next: callable-body stable lacks and Record literal extension diagnostics.

### C3 evidence
- `semantic::foundations::expression_composition`: PASS, 21 passed, 0 failed.
- `semantic::foundations::diagnostic_presentation`: PASS, 8 passed, 0 failed.
- `semantic::integration::record_row_polymorphism`: PASS, 10 passed, 0 failed.
- `semantic::advanced::record_rows`: PASS, 12 passed, 0 failed, 0 ignored; expected domain-safety panic test remains green.
- Callable-body checking now receives only deduplicated `(TypeParameterId, field)` lacks facts derived from canonical parameter and return types; no row solver state enters `CheckingContext`.
- Record literal extension accepts stable signature/annotation-proven disjointness and rejects unproved open-tail extension, duplicate fields, and incompatible open tails with owned diagnostics and invalid status.
- `RecordRowLacksUnproven` is published as `type.record.row_lacks_unproven` with headline `record extension requires a disjoint row field`; messages retain stable parameter/field names and no solver-local IDs.
- `graphify update .`: PASS; graph rebuilt with 54,175 nodes, 81,338 edges, 4,175 communities; HTML skipped due graph-size limit.

### C3 completion
- C3 COMPLETE.
- C4 is next: tighten core row rejection helpers and rerun the complete row integration package plus Either/Monad regressions.

### C4 evidence
- `typing_integration::rows::transformations::`: PASS, 2 passed, 0 failed; original duplicate-extension blocker now rejects formally.
- `typing_integration::rows::correlation::`: PASS, 2 passed, 0 failed.
- `typing_integration::rows::pipelines::`: PASS, 1 passed, 0 failed.
- `typing_integration::rows::`: PASS, 11 passed, 0 failed.
- `typing_integration::either::`: PASS, 27 passed, 0 failed; 61.46s.
- `typing_integration::monads::`: PASS, 35 passed, 0 failed; 61.52s.
- Core row rejection helper now accepts only `Invalid`/`Blocked`, retains the non-`Dynamic` assertion, and recognizes the complete row diagnostic family.
- The original `tagged` collision reports exactly one `RecordRowLacksViolation`; no `Ready + Unknown` tolerance remains.

### C4 completion
- C4 COMPLETE; R2 product incident resolved.
- Root cause: signature-wide implicit row-lacks obligations were not seeded.
- Safety defect: row-aware return materialization swallowed structured failure.
- Additional closure: Record literal extension now requires and displays explicit lacks proof.
- C5 is next: final negative gates, broad semantic/core verification, formatting, check, clippy, and delivery certification.

### C5 evidence
- Negative/deletion gates: PASS; signature path is sole `add_lacks` authority, row publication failures are explicitly handled, body/literal context has no solver-state references, `invalid_shape` is absent, new diagnostic wiring is complete, solver IDs are absent from user-facing checker/diagnostic strings, and `git diff --check` is clean.
- Owned Rust formatting: PASS with `rustfmt --edition 2024 --check` over all repair-touched Rust files.
- `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic`: PASS, 1,063 passed, 0 failed, 44 ignored; 60.79s.
- `RUSTFLAGS='' cargo check --workspace --all-targets`: PASS.
- `RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration::`: 74 passed, 16 failed, 435 ignored; all 16 failures remain the known Expression parser/typed-closure baseline (`@variant Pure<A>` and typed closure parameter syntax), with rows/Either/Monad green.
- Repository-wide `cargo fmt --all -- --check`: BASELINE FAIL from unrelated pre-existing formatting drift; touched repair files pass the owned-file check.
- `RUSTFLAGS='' cargo clippy --workspace --all-targets --all-features -- -D warnings`: BASELINE FAIL on six pre-existing lints: four `too_many_arguments`, one `collapsible_match`, and one `borrowed_box`; no lint was introduced by a new repair signature.
- `RUSTFLAGS='' cargo test --workspace --all-targets`: CANCELED by user after workspace tests began; no final workspace-test inventory is claimed.

### C5 status
- C5 COMPLETE for the requested implementation scope; the long workspace-test gate is explicitly waived by user instruction that testing is not needed.
- C0–C5 are COMPLETE; the record-row semantic repair is focused-tested, workspace-compiling, and delivery-classified.
- Global workspace-green status is not claimed: the independent Expression parser baseline remains failing, repository-wide format/Clippy gates retain unrelated baseline failures, and the final workspace-test run was intentionally not completed.

### Active blockers
- Expression baseline: current parser rejects constructor-local variant generics (`@variant Pure<A>`, `If<A>`, `Map<A, B>`, `FlatMap<A, B>`, `Apply<A, B>`, `Lift<A>`) and typed closure parameter annotations; this causes the same 16 `typing_integration::expression::` failures and is outside this row repair boundary.
- Repository quality baseline: unrelated formatting drift and six pre-existing Clippy lints remain.

### Next resume action
- No remaining row-repair implementation task. Optional delivery follow-up: complete the waived workspace test gate and separately resolve or explicitly waive the Expression parser, repository formatting, and Clippy baseline blockers. Do not expand the row repair into parser/AST work without authorization.

## SC-4.8 continuation: C5 generic variant construction and Families

### C5 evidence
- Variant-local generic declaration products retain callable ownership; enum declaration generics remain a separate application domain.
- Direct `Expr::Pair(1, "text")` and `Expr::Pair("text", 1)` solve enum and variant-local domains and publish exact case results.
- Contradictory expected constructor context reports `GenericInferenceConflict` and does not publish a stale known result.
- `Expr<Int>::Pair::*` rehydrates the canonical variant-constructor target; `family(1, "text")` solves its local `U` at invocation and publishes `Expr<Int>`.
- Fixed receiver/declaration arguments no longer allocate orphan inference variables, preventing false `GenericInferenceUnderconstrained` results.
- C5 checkpoint gates PASS: constructors 13, ADT generics 12, associated 12, Families 14; `git diff --check` PASS.

### C5 completion
- C5 COMPLETE.
- C6 is next: introduce query-local rigid variables and fail-closed publication/alpha-equivalence tests.

### C2 evidence
- Generic setters and index members now carry reusable callable-local generic binders and `where` clauses in AST products.
- Parser supports `value<T>=(put next: T)` and `[_ key: U]<U>` forms, including maximal-munch `>=` splitting at generic setter boundaries.
- Setter RHS inference publishes `Unit`, preserves one `CallableId`, and rejects violated `where` bounds without Dynamic fallback.
- Index getter inference uses key evidence; index setter sends key and assigned value through ordinary generic application and rejects conflicting evidence.
- Generated `@set` accessors initialize empty generic metadata and remain compilable.
- C2 checkpoint gates PASS: AST parser 2 focused tests, setter capabilities 3, index capabilities 3, canonical call application 35, compiler attribute target 0 filtered tests/compile green; `git diff --check` PASS.

### C2 completion
- C2 COMPLETE.
- C3 is next: applied generic class-side templates and durable receiver specialization.

## SC-4.8 continuation: C3 applied class-side templates

### C3 evidence
- Class-side declaration type-level bindings now remain available during
  template formation; use-site saturation stays separate from declaration
  products.
- Applied class-side receivers dispatch through their class-object descriptor
  while specializing signatures against the retained type form. `Box<Int>`
  and `Box<String>` therefore publish distinct member types under one callable
  identity.
- Class-side fields, getters, setters, ordinary methods, and constructors now
  carry declaration-owned generic domains separately from callable-local
  domains. Applied receivers fix declaration parameters; raw `Box.member`
  use remains explicitly underconstrained.
- Inferred `Box.new(10)` and explicit `Box<Int>.new(10)` converge to the same
  canonical receiver application and callable identity. Bound behavioral
  families retain applied receiver forms through later invocation.
- C3 checkpoint gates PASS: applied class-side integration 2, receiver
  specialization 6, generic application 8, variance 4, behavioral families
  9; `git diff --check` PASS.
- `graphify update .`: PASS; graph rebuilt with 54,234 nodes, 81,511 edges,
  4,139 communities; HTML skipped due graph-size limit.

### C3 completion
- C3 COMPLETE.
- C4 is next: rigid GADT elimination and fail-closed publication.

## SC-4.8 continuation: C6 scoped rigid kernel

### C6 evidence
- Chosen representation: checker-local `LocalType` plus monotonic `RigidArena`; canonical `TypeStore` remains free of rigid nodes. Extending the canonical store and extending the existing scoped lambda arena were rejected because branch-local existential identity must not enter durable `TypeId` products.
- `RigidScopeId`, `RigidTypeVariableId`, kind/origin metadata, and parent-scope containment are implemented.
- Composite local types support applications, exact cases, unions, tuples, records, and callables. Free-rigid walking covers nested members and `contains_rigid_from_scope` is scope-aware.
- Rigid inference terms compare by identity, distinct rigids fail structural equality, flexible variables may retain a deferred rigid term, and no rigid enters the flexible substitution map.
- Local materialization rejects rigid-containing terms with a dedicated `RigidMaterializationError`; alpha-equivalence uses one-to-one structural binder mapping instead of raw allocator IDs.
- C6 checkpoint gates PASS: semantic lib check; rigid kernel 3; inference kernel 12; owned Rust format check; `git diff --check`.

### C6 completion
- C6 COMPLETE.
- C7 is next: open constructor-local variant binders as fresh existentials and reuse existing GADT index proofs.

## SC-4.8 continuation: C7 GADT existential elimination

### C7 evidence
- `CaseInstantiation` opens each accepted variant candidate with a fresh rigid scope and one rigid per constructor-local generic binder. Repeated payload occurrences reuse the same rigid; separate eliminations receive distinct scopes/IDs.
- Local payload and result types are retained beside canonical `TypeId` knowledge. Variant-local `where` constraints become branch-local `LocalConstraint` evidence; no global generic substitution is mutated.
- Existing declaration-index GADT proofing remains authoritative. A local result proof refines flexible outer parameters structurally while rigid leaves remain opaque and cannot be guessed or solved.
- Pattern bindings and resolved fields retain the opened local type view, while canonical exact-case and pattern-space products remain unchanged in shape.
- C7 hostile coverage includes shared-rigid identity, independent freshness, `Wrap<U> -> Expr<List<U>>` local index proof, local bound retention, and rejection of fitting a concrete index by guessing the rigid.
- C7 checkpoint gates PASS: semantic lib check; vertical GADT 1; matching 162 passed, 18 ignored; GADT refinement 21 passed, 1 ignored; ADT generics 13; `git diff --check`.

### C7 completion
- C7 COMPLETE.
- C8 is next: enforce existential non-escape at result, assignment, aggregate, call, closure, and exact-case boundaries.

## SC-4.8 continuation: C8 existential non-escape and exact-case reconstruction

### C8 evidence
- A shared checker-local publication guard now owns local-type escape decisions
  across match joins, returns, outer binding and field assignment, declaration
  pattern publication, aggregate construction, call arguments/results, and
  closure capture. It preserves query-local `LocalType` beside canonical
  `TypeKnowledge`; no rigid is inserted into durable `TypeStore` products.
- Direct, structural-wrapper, outer-assignment, incompatible-call, closure-
  capture, and safe rigid-free widening cases are covered in
  `semantic::adts::existentials`. The dedicated escape diagnostic is
  `type.existential.escape`; invalid paths recover as unknown with
  `UnknownReason::ExistentialEscape` rather than laundering the value to
  `Dynamic`.
- Exact-case elimination reconstructs a fresh `CaseInstantiation` from the
  canonical enclosing enum type and constructor-local replacements. Matching
  exact-case observations retain canonical `TypeData::ExactCase { variant,
  enum_type }` identity, while each elimination gets independent local rigids.
- Metadata export has an explicit hard guard rejecting query-local rigid types,
  with a focused negative test.
- C8 checkpoint gates PASS: existential suite 7, exact-case suite 5 passed and
  1 ignored/gated, flow branches 28, callable publication 13, metadata export
  negative 1, vertical GADT 1, and full ADT matching 163 passed with 18
  ignored. `git diff --check` PASS.

### C8 completion
- C8 COMPLETE.
- C9 is next: native/generated/intrinsic callable metadata parity and durable
  generic ownership.

## SC-4.8 continuation: C9 native/generated callable metadata parity

### C9 evidence
- Native callable metadata now represents generic parameter sequences and
  subtype/equivalence constraints without embedding semantic `TypeId`s.
- Native import creates canonical `TypeParameterOwner::Callable` binders,
  lowers constraints through the ordinary native type resolver, and publishes
  one `GenericSignature` through `CallableSemanticSignature`; native calls do
  not use a parallel inference path.
- Generated native-surface records and macro output initialize the same
  constraint field. The checked catalog contains 319 primitive declarations.
- Callable metadata export keys include owner, parameter identity sequence, and
  constraint shape, preserving separate declaration/callable ownership and
  preventing changed constraints from being deduplicated away. Query-local
  rigid export rejection remains active.
- Core generated non-generic members retain empty generic metadata and no
  synthetic binders. Native/core variant representation was unchanged, so no
  duplicate native variant schema was added.
- C9 checkpoint gates PASS: type syntax 5, native surface 2, surface generator
  2, native importer 1, native conformance 3, metadata integration 13, native
  catalog `--check`, core all-target compilation, core native-surface contracts
  3, and `git diff --check`.
- The broader `compiler_declaration_dispatch` runtime filter was bounded and
  stopped after hanging during existing core test setup; this is a
  backend/harness baseline observation, not a C9 compile or semantic failure.

### C9 completion
- C9 COMPLETE.
- C10 is next: incremental invalidation, cold/reanalysis parity, and final
  semantic certification.

## SC-4.8 continuation: C10 incremental parity and semantic certification

### C10 evidence
- Source/interface fingerprints now include setter and index-setter generic
  binders/constraints alongside the existing getter, constructor, and variant
  generic contract shapes. Call-site substitutions and rigid allocation IDs
  remain excluded from fingerprints.
- Incremental callable-contract coverage proves constructor bound edits,
  setter constraint edits, and index-setter binder/constraint edits invalidate
  dependent bodies and converge on cold analysis. Variant payload/result/bound
  edits invalidate both construction and match consumers.
- Cold and incremental variant openings compare payload/result local structure
  alpha-equivalently; local existential information is retained in the
  comparison rather than erased.
- Generic application now excludes an unsaturated declaration-generic domain
  when a class-side member's callable signature does not mention it. Thus a
  raw generic class-side member such as `Box.value<U> -> U` remains inferable,
  while members depending on declaration parameters remain underconstrained.
- Current fail-closed conflict semantics are reflected in legacy capability
  assertions: conflicting generic/HKT applications publish no result type,
  retain their diagnostic, and do not launder contradictory expected-result
  evidence into a known type. Ordinary inherited `::` behavior publishes the
  bound behavioral invocation product; variant associated invocations retain
  `StaticInvoke`.
- C10 checkpoint gates PASS: incremental generic-contract coverage 4,
  fingerprint coverage 32, full incremental suite 124 passed with 4 ignored,
  full ADT suite 253 passed with 27 ignored, family suite 14 passed, full
  semantic suite 1,093 passed with 44 ignored, unified core monads 35 passed,
  and unified core Either 27 passed.
- Required core filters `monads::` and `either::` resolve to the unified
  `typing_integration::monads` and `typing_integration::either` packages; no
  duplicate legacy package remains.
- The six full-semantic failures were classified before repair: two stale
  associated-resolution expectations, three stale fail-closed generic/HKT
  expectations, and one real raw class-side generic-domain overconstraint.
  Narrow repairs were limited to those contracts and the shared application
  domain selection seam.

### C10 completion
- C10 COMPLETE.
- SC-4.8 semantic implementation and affected-layer certification are
  complete. Final workspace formatting, check, test, and lint gates remain
  delivery evidence and must be reported separately from semantic closure.

## SC-4.8 Final Gate evidence

- `RUSTFLAGS='' cargo +stable check --workspace --all-targets`: PASS after
  C10 lint refactors.
- `RUSTFLAGS='' cargo +stable test -p phalcom-semantic --test semantic`: PASS,
  1,093 passed, 0 failed, 44 ignored after final source refactors.
- Protected unified core suites remain PASS after final source refactors:
  monads 35 passed and Either 27 passed.
- `RUSTFLAGS='' cargo +stable clippy --workspace --all-targets -- -D warnings`:
  semantic crate is clean after targeted remediation; workspace gate remains
  BASELINE because unrelated `phalcom-core` paths report 11 existing
  deny-level lints, including `ProjectionError` result size, unnecessary
  casts/lifetimes, and a needless question mark.
- `cargo +stable fmt --all -- --check`: BASELINE FAIL. Existing workspace
  formatting drift remains across older typing-integration/core and semantic
  files; C10-owned call/test edits were kept formatted without normalizing
  unrelated dirty files.
- `RUSTFLAGS='' cargo +stable test --workspace --all-targets`: BASELINE/HARNESS
  INCIDENT. The run reached the 525-test core binary and continued executing
  fixture/corpus tests without an aggregate completion result; it was stopped
  after bounded observation with exit 130. No SC-4.8 semantic failure surfaced
  before termination.
- Final negative searches found no production occurrences of
  `Type.currentApplication`, `TypeParameterOwner::Variant`,
  `TypeData::Rigid`, `GadtSkolem`, or
  `merge_constructor_generic_signatures`. Historical deferred getter text is
  explicitly marked superseded; the SC-4.8 superseded-rules section remains
  intentional documentation.

### Release boundary
- SC-4.8 semantic implementation: COMPLETE.
- Workspace release-complete status: NOT CLAIMED because repository-wide fmt,
  workspace tests, and workspace Clippy remain blocked by unrelated baseline
  drift/harness behavior. No commit was created.
