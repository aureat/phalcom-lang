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
