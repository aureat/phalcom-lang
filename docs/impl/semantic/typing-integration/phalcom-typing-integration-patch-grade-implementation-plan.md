# Phalcom Unified Typing Integration — Repository-Grounded Patch-Grade Implementation Plan

**Repository:** `aureat/phalcom-lang`  
**Program:** unify the existing `Either` and `monads` conformance packages into one cross-feature typing integration package, then land the first effectful typed-Expression GADT vertical test on top of that shared foundation.  
**Plan preparation mode:** remote GitHub repository inspection only. The local working tree was not available to this planning session, so local uncommitted changes are **unknown** and must be checked by the implementing agent before any edit.  
**Prepared against remote branch:** `main`  
**Prepared against exact remote HEAD:** `9f04681201e4e15388b4a32d09a2a502486e9367`  
**HEAD subject:** `feat: extend semantic type-system closure`  
**HEAD timestamp:** 2026-09-03T09:30:10Z  
**Relevant recent test-package commits:**
- `309d9e5d8cae5336d9010b89597c30ca4466db44` — `test: add monads HKT and generic-inheritance conformance package (#16)`
- `3a7300082368214c04552f425edf4649ba1597b5` — `test: move Either conformance into core`
- `483a43cd81c9b8a8123c126ad5d886577889b6e7` — `test: add Either generic conformance target`

> Repository authority rule: if the implementing checkout has drifted from the revision above, adapt mechanics only after the checkpoint drift procedure in this plan. Do not silently change the semantic design because current file layout is more convenient.

---

# 1. Implementation Program

The current repository already has two strong but separate conformance packages under the single `phalcom-core` integration test target:

```text
phalcom-core/tests/core/either/
phalcom-core/tests/core/monads/
```

They overlap substantially:

- both define an `Either<L, R>` source-level ADT;
- both own near-duplicate Rust fixture infrastructure;
- both analyze ordinary Phalcom source through `phalcom-semantic`;
- both compile source through `ProgramCompiler` and execute it in the VM;
- the monads suite already consumes `Either` as its concrete HKT/type-lambda specialization;
- the newly proposed typed Expression GADT is intended to consume the same Monad contracts and the same `Either<String, _>` effect.

The program therefore creates one reusable integration ecosystem rather than three independent fixture universes.

The target package is:

```text
phalcom-core/tests/core/typing_integration/
```

Its purpose is:

> Prove that independently implemented Phalcom typing features compose correctly through realistic source programs, while preserving focused sub-suites that make failures attributable.

It is **not** intended to become a second implementation of the type system. Production semantic authority remains in `phalcom-semantic`.

---

# 2. Checkpoint Map

| Checkpoint | Tasks | Semantic boundary | Required evidence | Deferred evidence |
|---|---:|---|---|---|
| C0 | 1–2 | Baseline and migration contract are certified against the implementing checkout | exact Git state; existing `either::` suite; existing `monads::` suite; inventory/state ledger | no workspace-wide tests yet |
| C1 | 3–4 | `Either` and `monads` are one Rust integration-package namespace with behavior unchanged | both suites pass under `typing_integration::...`; old top-level registrations absent | shared fixture/source deduplication deferred to C2 |
| C2 | 5–8 | One canonical `Either` source and one shared semantic/VM fixture authority serve both sub-suites | hostile rejection probes; complete Either subtree; complete Monad subtree; duplicate-source/support negative searches | unified cross-feature scenario and docs deferred to C3 |
| C3 | 9–10 | The unified package has one explicit law/documentation contract and one cross-feature semantic scenario using the same canonical `Either` through both direct and Monad paths | new integration semantic test; full `typing_integration::` suite; current-doc path audit | Expression GADT work deferred to C4 |
| C4 | 11–15 | Typed `Expression<F,T>` composes GADT branch refinement, HKTs, higher-order callables, Monad specialization, type lambdas, and `Either<String,_>` execution | existing ownership-layer GADT proof suite; Expression semantic positives; hostile negatives; Expression VM tests; flagship cross-feature scenario | repository-wide delivery gates deferred to C5 |
| C5 | 16–18 | Migration is complete, obsolete authorities are absent, all deferred delivery evidence is discharged | deletion searches; `phalcom-core` core target; format/check/workspace test/clippy gates | none |

No checkpoint is complete merely because its edits exist. A checkpoint becomes `COMPLETE` only after the required evidence listed in its section passes.

---

# 3. Repository Architecture Relevant to This Program

## 3.1 Test-target ownership

`phalcom-core/Cargo.toml` disables Cargo autotests and explicitly defines:

```toml
[[test]]
name = "core"
path = "tests/core/mod.rs"
```

Therefore `phalcom-core/tests/core/mod.rs` is the registration root for this program. This work does **not** create a new Cargo test target.

Current registration at the inspected revision is:

```rust
#[path = "either/mod.rs"]
mod either;

#[path = "monads/mod.rs"]
mod monads;
```

The target state is one registration:

```rust
#[path = "typing_integration/mod.rs"]
mod typing_integration;
```

## 3.2 Static semantic authority

The fixture in both packages calls:

```rust
phalcom_semantic::analyze_single_module(...)
```

and retains the resulting `SemanticAnalysis` / snapshot. The snapshot and its canonical products are the authority for static assertions:

```text
DeclarationId
CallableId
TypeParameterId
TypeId / TypeStore
CallableAnalysis
ExpressionAnalysis
explanation/provenance graph
match products
```

The unified helper layer may make those products easier to inspect, but helper strings, names, and test-local type-shape enums are never a competing semantic authority.

## 3.3 Runtime authority

Both packages compile ordinary Phalcom source through:

```rust
ProgramCompiler::compile_entry_selection(EntrySelection::Inline(...))
```

and execute it with:

```rust
VM::run_compiled(...)
```

That path remains the runtime integration authority. Do not replace it with mocked values or test-only execution semantics.

## 3.4 GADT authority

GADT result specialization and branch proof behavior already belong to `phalcom-semantic`:

```text
phalcom-semantic/src/checker/gadt_proof.rs
    GadtProofResult
    solve_gadt_branch_proof

phalcom-semantic/src/checker/pattern.rs
    consumes GADT proof results during pattern resolution

phalcom-semantic/src/checker/exhaustiveness.rs
    consumes the same proof authority for reachable pattern space
```

The ownership-layer regression suite already exists at:

```text
phalcom-semantic/tests/semantic/adts/matching/gadt_refinement.rs
```

It covers, among other things:

- generic evaluator branch proofs;
- incompatible concrete-case exclusion;
- multiple GADT parameters;
- nested GADT proof locality;
- proof non-leakage to sibling arms.

The new `typing_integration` Expression tests therefore **must not duplicate the proof engine**. They test what happens when existing GADT evidence is consumed by recursive generic calls, Monad methods, HKTs, type lambdas, and runtime execution.

## 3.5 Toolchain and delivery environment

The inspected repository pins:

```text
rust-toolchain.toml
    nightly-2026-07-10
```

and `.cargo/config.toml` configures workspace feature unification, `sccache`, build flags, and `RUST_MIN_STACK=33554432`.

The plan uses repository-native Cargo commands and intentionally defers workspace-wide gates until C5.

---

# 4. Current Source/Test Topology

## 4.1 Existing `Either` package

`phalcom-core/tests/core/either/mod.rs` owns:

```text
higher_order.rs
inference.rs
isolation.rs
nested.rs
rejection.rs
runtime.rs
substitution.rs
support.rs
```

and source fixtures including:

```text
either.ph
semantic_probes.ph
runtime_probes.ph
invalid/*.ph
```

The current `either.ph` is the richer of the two `Either` definitions. It contains:

```text
Left
Right
isLeft
isRight
fold
map
mapLeft
bimap
flatMap
orElse
recover
getOrElse
swap
zip
```

This file is the correct basis for the single canonical shared `Either` source.

The Either tests exercise existing `GEN-*` laws documented in `docs/spec/typing-generics.md` and include hostile no-escape cases such as:

- conflicting constructor context;
- wrong `map` result;
- wrong `mapLeft` result;
- repeated generic-variable conflict;
- `flatMap` attempting to change the preserved left parameter;
- nested repeated-variable conflict;
- an underconstrained `Left` remaining `Unknown(UnderconstrainedTypeVariable)` rather than becoming `Dynamic`.

## 4.2 Existing `monads` package

`phalcom-core/tests/core/monads/mod.rs` owns:

```text
bodies.rs
composition.rs
constructor_agreement.rs
inference.rs
inheritance.rs
inherited_methods.rs
kinds.rs
overrides.rs
rejection.rs
runtime.rs
support.rs
type_lambdas.rs
```

and:

```text
monads.ph
semantic_probes.ph
runtime_probes.ph
README.md
LAWS.md
monad-testing.md
```

The current `monads.ph` begins by declaring a second, reduced `Either<L,R>` with only `fold`, `map`, and `flatMap`, then defines:

```text
Box<T>
Functor<F: Type -> Type>
Applicative<F: Type -> Type> is Functor<F>
Monad<F: Type -> Type> is Applicative<F>
BoxMonad
ContractEitherMonad<E>
StringContractEitherMonad
EitherMonad<E>
StringEitherMonad
MonadAlgorithms
```

`MonadAlgorithms` currently contains:

```text
bind
sameConstructor
sequenceSeed
sequence
constructorIdentity
kleisli
traverse
```

The duplicate `Either` declaration is the main source-authority defect this migration should eliminate.

The distinction between these two classes is semantically intentional and must be preserved:

```text
ContractEitherMonad<E>
    pure inherited-contract specialization probe; deliberately no overrides

EitherMonad<E>
    executable specialization; supplies concrete overrides for VM tests
```

Do not collapse them during cleanup.

## 4.3 Existing fixture duplication

Both `either/support.rs` and `monads/support.rs` define essentially the same fixture lifecycle:

```text
parse source
→ analyze one module
→ assert no internal incidents
→ inspect declarations/callables/bindings/expressions
→ compile inline source
→ execute VM
→ read module slots
```

`monads/support.rs` is the semantic-inspection superset. In addition to the common fixture behavior, it already owns helpers for:

```text
DeclarationTypeInfo
canonical type forms
KindId / KindData
TypeParameterOwner / exact TypeParameterId lookup
callable generic parameters
receiver specialization
specialization path
GenericConstraintOrigin
GenericConstraintRelation
exact generic solution evidence
callable selection identity
```

`either/support.rs` contributes a useful lightweight type-shape DSL and helpers around exact-case family normalization:

```rust
Ty::{Nominal, Applied, Tuple}
nominal(...)
either(...)
tuple(...)
Fixture::family_type(...)
Fixture::assert_type(...)
Fixture::assert_known_generic_binding(...)
Fixture::assert_receiver_selection(...)
```

The target shared harness should evolve from the Monads harness and selectively absorb these Either conveniences.

Important collision: both helpers currently define methods named `assert_generic_solution` with different expected-value types. Rust has no method overloading. The shared harness must retain the Monads exact `TypeId`/identity-based API as authoritative and migrate Either callers rather than attempting to keep both overloads.

---

# 5. Target Package Topology

The intended final package is:

```text
phalcom-core/tests/core/typing_integration/
├── mod.rs
├── support.rs
├── README.md
├── LAWS.md
├── integration.rs
├── sources/
│   ├── either.ph
│   ├── monads.ph
│   ├── either_semantic_probes.ph
│   ├── either_runtime_probes.ph
│   ├── monad_semantic_probes.ph
│   ├── monad_runtime_probes.ph
│   ├── integration_probes.ph
│   ├── expression.ph
│   ├── expression_semantic_probes.ph
│   └── expression_runtime_probes.ph
├── either/
│   ├── mod.rs
│   ├── higher_order.rs
│   ├── inference.rs
│   ├── isolation.rs
│   ├── nested.rs
│   ├── rejection.rs
│   ├── runtime.rs
│   ├── substitution.rs
│   └── invalid/
├── monads/
│   ├── mod.rs
│   ├── bodies.rs
│   ├── composition.rs
│   ├── constructor_agreement.rs
│   ├── inference.rs
│   ├── inheritance.rs
│   ├── inherited_methods.rs
│   ├── kinds.rs
│   ├── overrides.rs
│   ├── rejection.rs
│   ├── runtime.rs
│   ├── type_lambdas.rs
│   └── monad-testing.md
└── expression/
    ├── mod.rs
    ├── refinement.rs
    ├── higher_order.rs
    ├── monad.rs
    ├── rejection.rs
    ├── runtime.rs
    └── integration.rs
```

This topology intentionally preserves `either/` and `monads/` as focused diagnostic sub-suites while moving their fixture/source authorities to the package root.

Do not flatten every Rust test into one directory: `inference.rs`, `rejection.rs`, and `runtime.rs` already collide by name, and retaining the conceptual sub-suites makes failures easier to attribute.

---

# 6. Semantic Invariants

The implementation program must preserve or establish all of the following.

### I-01 — One canonical user-level `Either` definition

Within the live `typing_integration` source universe, exactly one source declaration of `enum Either<L, R>` exists: `sources/either.ph`.

### I-02 — Existing Either semantics are preserved

The full current rich `Either` surface and all existing `GEN-*` tests remain intact. The migration must not reduce `Either` to the smaller monads copy.

### I-03 — Existing Monad/HKT semantics are preserved

All current `MON-*` laws retain their identifiers, exact proof-path expectations, and runtime coverage.

### I-04 — One shared fixture authority

There is one `typing_integration/support.rs`. It exposes semantic products from `phalcom-semantic` and VM execution through `ProgramCompiler`/`VM`. Child-local `support.rs` files are removed after C2.

### I-05 — Identity/provenance checks must not regress to strings

Where existing Monads tests assert `DeclarationId`, `CallableId`, `TypeParameterId`, exact receiver paths, generic relation origin, or evidence status, those checks remain canonical. A helper merge is not permission to replace them with name-only assertions.

### I-06 — Feature fixture isolation is retained

There is no single always-loaded mega-source. Basic `Either` tests can analyze `Either` without requiring Monad or Expression declarations; Monad tests load `Either + monads`; Expression tests load only the layers they need.

### I-07 — Runtime fixture isolation is retained

The existing Either and Monad runtime probe files both currently use names such as `runtimeLeft`, `runtimeRight`, and `runtimeAll`. They are not blindly concatenated. Each runtime test uses a purpose-built source assembler.

### I-08 — Negative typing failures remain fail-closed

Conflicting or underconstrained generic/HKT/GADT cases must remain `Invalid`, `Blocked`, or formally `Unknown` according to the existing contract. They must not become `Dynamic` to make integration pass.

### I-09 — GADT proof ownership stays in `phalcom-semantic`

The Expression suite consumes real branch proof/refinement products; it does not invent test-local equality substitution or special-case expression branches in the fixture.

### I-10 — Expression uses the existing abstractions

`ExpressionMonad<F>` specializes the existing `Monad` contract, and concrete evaluation uses existing `EitherMonad<String>` / `StringEitherMonad`. No duplicate `TestMonad`, `TestEither`, or GADT-specific Monad abstraction is added.

### I-11 — Static/runtime correspondence is explicit

For runtime scenarios, the exact source executed by the VM is semantically analyzed first. Runtime success does not substitute for static proof assertions.

### I-12 — Historical documents remain historical

Old completed plans/logs may legitimately mention `phalcom-core/tests/core/either` or `.../monads`. Migration deletion gates are scoped to live test registration/current package documentation. Do not rewrite historical evidence solely to make repository-wide text searches empty.

---

# 7. Source of Truth Matrix

| Concern | Source of truth | Derived consumers | Forbidden competing authority |
|---|---|---|---|
| Canonical test `Either` declaration | `typing_integration/sources/either.ph` | Either tests, Monad source, Expression effect scenarios, VM fixtures | reduced copy inside `monads.ph` |
| Monad/HKT contracts | `typing_integration/sources/monads.ph` | MON tests, ExpressionMonad, generic algorithms | Expression-local duplicate contracts |
| Static type identity | `phalcom-semantic` snapshot / `TypeStore` / canonical IDs | shared test helpers | string name equality as semantic authority |
| Generic solution/provenance | callable explanation graph and exact `TypeParameterId` | MON and integration assertions | test-local reconstructed substitution map |
| GADT branch proof | `GadtProofResult` / match branch proof product | Expression evaluator checking | manually substituting `T` in test harness |
| Runtime behavior | compiled Phalcom source executed by `VM` | runtime test observations | direct Rust emulation of Either/Monad/Expression semantics |
| GEN law definitions | `docs/spec/typing-generics.md` | Either test comments / unified law index | duplicate divergent law definitions |
| MON law definitions | migrated unified `LAWS.md`, preserving existing `MON-*` text/IDs | Monads tests | renumbered/reinvented MON catalog |
| GEX/INT laws | unified `LAWS.md` after C4 | Expression/integration tests | production semantic code comments as test spec |

---

# 8. Evidence Model From Repository Inspection

| Architectural conclusion | Repository evidence inspected |
|---|---|
| `core` is one explicit integration test target | `phalcom-core/Cargo.toml` — `[[test]] name = "core" path = "tests/core/mod.rs"` |
| Either and Monads are sibling modules today | `phalcom-core/tests/core/mod.rs` |
| Either package is a user-defined generic ADT integration suite | `phalcom-core/tests/core/either/mod.rs` |
| Monads package is HKT/generic-inheritance integration | `phalcom-core/tests/core/monads/mod.rs`, `README.md`, `LAWS.md` |
| Monads source duplicates a reduced Either | `phalcom-core/tests/core/monads/monads.ph` |
| Either source is richer and suitable as canonical shared definition | `phalcom-core/tests/core/either/either.ph` |
| Monads support is the richer fixture implementation | `phalcom-core/tests/core/monads/support.rs` |
| Either support adds useful shape helpers but has overlapping method names | `phalcom-core/tests/core/either/support.rs` |
| Existing MON tests require exact semantic identities/provenance | `monads/README.md`, `monads/LAWS.md`, `monads/inference.rs`, `monads/inherited_methods.rs` |
| Existing negative suites explicitly forbid Dynamic escape | `either/rejection.rs`, `monads/rejection.rs` |
| GADT syntax/result specialization is ratified | `docs/spec/adts.md` |
| Constructor-local generic GADT examples already exist | `examples/ide-golden/phalcom-enum-highlighting-showcase.ph` |
| GADT proof engine exists as semantic authority | `phalcom-semantic/src/checker/gadt_proof.rs` |
| Ownership-layer GADT refinement tests already exist | `phalcom-semantic/tests/semantic/adts/matching/gadt_refinement.rs` |
| Historical baseline recorded 27 Either / 35 Monad tests green | current SC-4 implementation-state ledger at inspected revision; must still be rerun by implementer |

---

# 9. Tempting Wrong Fixes

Do **not** take any of these shortcuts.

1. **Do not copy the rich `Either` into `monads.ph`.** The point of the migration is one source authority, not a larger duplicate.
2. **Do not retain both support modules behind re-export shims indefinitely.** C2 requires one real fixture authority and deletion evidence.
3. **Do not flatten all probe sources into one source string.** Existing runtime names collide, and focused fixture isolation is valuable diagnostic evidence.
4. **Do not replace exact IDs/provenance assertions with type display strings.** That would make the unified suite weaker than the existing Monads suite.
5. **Do not keep two `assert_generic_solution` methods by inventing a test-helper trait trick.** Retain the exact `TypeId`/`TypeParameterId` API and migrate the simpler Either call sites.
6. **Do not delete `ContractEitherMonad<E>`.** It proves inherited contract specialization without executable overrides contaminating callable-selection evidence.
7. **Do not make Expression carry its own `Monad` definitions.** Its value is precisely that it composes with the existing contracts.
8. **Do not repair an Expression test by widening an impossible/underconstrained path to `Dynamic`.** Mark the checkpoint `INCIDENT` and diagnose the production semantic boundary.
9. **Do not patch `phalcom-semantic`, parser, compiler, or VM opportunistically from this test-migration plan.** A newly exposed production defect requires incident classification and a bounded remediation decision before scope expands.
10. **Do not rewrite completed historical plans/logs simply because they contain old paths.** Only current/live documentation is migration-owned.

---

# 10. Checkpoint C0 — Baseline and Migration Contract

Tasks:
- Task 1 — certify local repository state and drift
- Task 2 — establish baseline evidence and implementation state ledger

Why this is a checkpoint:
The migration is supposed to preserve two already-valuable conformance suites. Their exact local baseline must be known before any namespace/source/harness changes, otherwise a pre-existing failure can be misclassified as migration damage.

Entry conditions:
- a local checkout of `aureat/phalcom-lang` is available;
- the implementer can run the pinned Rust toolchain;
- no edit for this program has started.

Working set:

Primary:
- `phalcom-core/tests/core/mod.rs` — test registration root
- `phalcom-core/tests/core/either/` — existing Either package
- `phalcom-core/tests/core/monads/` — existing HKT package
- `phalcom-core/Cargo.toml` — test target definition

Secondary — inspect only if evidence requires it:
- recent Git history for the two test directories
- `docs/impl/semantic/semantic-completeness/sc-4/...implementation-state.md` — historical baseline comparator only

Out of scope for this checkpoint:
- production semantic code;
- parser/compiler/VM edits;
- Expression source design;
- package movement.

Semantic contract established by this checkpoint:
- the exact implementing revision and working-tree state are known;
- the pre-migration Either and Monad suites have a locally observed pass/fail baseline;
- pre-existing incidents are separated from migration incidents.

Semantic risks:
- building on uncommitted semantic changes without recording them;
- assuming the remote 27/35-test historical result is still the local result;
- treating a pre-existing failure as a migration regression.

Hostile cases:
- local HEAD differs from the plan baseline and contains changes under either test package;
- one baseline suite fails before any program edit.

Required evidence:
1. Git-state commands below — establish branch/HEAD/status and recent area history.
2. `RUSTFLAGS='' cargo test -p phalcom-core --test core either:: -- --nocapture` — establishes current local Either baseline.
3. `RUSTFLAGS='' cargo test -p phalcom-core --test core monads:: -- --nocapture` — establishes current local Monad baseline.

Do not run yet:
- `cargo test --workspace --all-targets` — no cross-workspace invariant has changed.
- clippy/workspace check — deferred to C5.

Escalate immediately if:
- either baseline suite is red before edits;
- HEAD drift changes source ownership materially (for example, package already moved/merged);
- a new third package already owns shared typing fixtures.

Checkpoint completion:
- [ ] Tasks 1–2 complete
- [ ] local revision/status recorded
- [ ] Either baseline recorded
- [ ] Monad baseline recorded
- [ ] any baseline incident classified
- [ ] implementation state updated
- [ ] no active incident remains

Suggested commit grouping:
No code commit is required for C0. The state ledger may be committed with C1 if repository convention prefers not to commit a state-only file separately.

## Task 1 — Certify Local Repository State and Drift

Purpose:
Anchor execution to an exact local repository state and determine whether the remote-grounded plan needs mechanical adaptation.

Risk:
- Semantic: LOW
- Implementation fanout: local

Owned files and symbols:
- no production/test edit yet

Inspect before editing:
- `phalcom-core/tests/core/mod.rs`
- `phalcom-core/tests/core/either/mod.rs`
- `phalcom-core/tests/core/monads/mod.rs`

Do not inspect unless evidence forces expansion:
- parser
- semantic checker internals
- compiler
- VM
- LSP

Dependencies:
- none

Source of truth:
- local Git checkout

Implementation boundary:

Changes:
- none

Must not:
- clean or reset unrelated user changes;
- silently switch branches;
- assume remote GitHub state equals the local checkout.

Current implementation:
The plan was prepared against remote `main` at `9f04681201e4e15388b4a32d09a2a502486e9367`; local working-tree state was unavailable.

Target implementation:
A recorded local baseline identifying any drift relevant to this program.

Edit operations:

1. Run:

```bash
git rev-parse --show-toplevel
git branch --show-current
git rev-parse HEAD
git status --short
git log -8 --oneline -- phalcom-core/tests/core/either phalcom-core/tests/core/monads
```

2. Compare local HEAD with `9f04681201e4e15388b4a32d09a2a502486e9367`.
3. If different, inspect only diffs affecting the primary working set:

```bash
git diff 9f04681201e4e15388b4a32d09a2a502486e9367...HEAD -- \
  phalcom-core/tests/core/mod.rs \
  phalcom-core/tests/core/either \
  phalcom-core/tests/core/monads \
  phalcom-core/Cargo.toml
```

4. Adapt path/signature mechanics where needed; do not change semantic goals without an escalation note.

Code instructions:

STRUCTURAL:
No code should be changed in this task.

Testing classification:
- No standalone behavioral test; baseline tests belong to Task 2/C0.

Checkpoint state update:
Record:
- branch;
- HEAD;
- dirty files relevant to this program;
- whether plan mechanics need adaptation;
- any new consumers discovered.

## Task 2 — Establish Baseline Evidence and State Ledger

Purpose:
Capture the pre-migration evidence boundary and create the concise state file used by later checkpoints.

Risk:
- Semantic: LOW
- Implementation fanout: local

Owned files and symbols:
- `docs/impl/semantic/typing-integration/typing-integration-implementation-state.md` — new concise execution ledger

Inspect before editing:
- existing semantic implementation-state files under `docs/impl/semantic/semantic-completeness/` for repository-native style

Do not inspect unless evidence forces expansion:
- unrelated implementation plans

Dependencies:
- Task 1 Git state

Source of truth:
- actual command output from the local checkout

Implementation boundary:

Changes:
- create/update the execution ledger;
- do not mark historical remote evidence as local PASS evidence.

Must not:
- claim 27/35 tests passed locally unless the commands actually do so.

Current implementation:
The repository has historical evidence that these suites were green at this lineage, but no state file exists for this unification program.

Target implementation:
A new ledger using the protocol in Section 21 of this plan.

Edit operations:

1. Create directory/file if absent:

```text
docs/impl/semantic/typing-integration/typing-integration-implementation-state.md
```

2. Record baseline revision/status from Task 1.
3. Run:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core either:: -- --nocapture
RUSTFLAGS='' cargo test -p phalcom-core --test core monads:: -- --nocapture
```

4. Record exact result counts and any ignored tests.
5. If a suite fails, stop C0 and follow the Failure Protocol before any migration edit.

Code instructions:

STRUCTURAL:
Use the state-file schema in Section 21; keep it factual and concise.

Testing classification:
- Required C0 baseline evidence.

Checkpoint state update:
Record:
- both command lines;
- exact result counts;
- baseline incidents if any;
- next action: C1 Task 3 only if C0 is COMPLETE.

---

# 11. Checkpoint C1 — Unified Rust Package Namespace, Semantics Unchanged

Tasks:
- Task 3 — move the existing package directories under `typing_integration`
- Task 4 — replace the two top-level core registrations with one package registration

Why this is a checkpoint:
C1 deliberately changes only Rust module/file ownership. It creates the package boundary before source/harness deduplication. If tests fail here, the cause is path/module migration rather than semantic fixture changes.

Entry conditions:
- C0 COMPLETE;
- both baseline suites have recorded local results;
- primary files still match their responsibilities.

Working set:

Primary:
- `phalcom-core/tests/core/mod.rs`
- `phalcom-core/tests/core/either/`
- `phalcom-core/tests/core/monads/`
- new `phalcom-core/tests/core/typing_integration/mod.rs`

Secondary — inspect only if evidence requires it:
- Rust imports referring to absolute test-module paths

Out of scope for this checkpoint:
- changes inside `either.ph` / `monads.ph`;
- support harness merge;
- law renumbering;
- Expression source.

Semantic contract established by this checkpoint:
- `Either` and `monads` are sub-suites of one integration package;
- every existing source fixture and helper remains unchanged;
- pre/post namespace results are equivalent.

Semantic risks:
- accidentally losing a test module during `git mv`;
- path attributes registering old and new copies simultaneously;
- relative `include_str!` paths being disturbed by partial movement.

Hostile cases:
- one module is omitted from a moved `mod.rs`;
- both old top-level modules and new nested module remain registered, causing duplicate execution.

Required evidence:
1. `RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration::either:: -- --nocapture` — same test count/results as C0 Either baseline.
2. `RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration::monads:: -- --nocapture` — same test count/results as C0 Monad baseline.
3. scoped registration search — proves only the new top-level registration remains.

Do not run yet:
- workspace tests — no production code changed;
- full `core` target — focused subtrees prove the moved boundary more directly.

Escalate immediately if:
- a moved test requires production changes to compile;
- source relative paths unexpectedly depend on their old parent directory outside the moved subtree;
- test counts differ from C0 without an intentional deletion/addition (none is allowed in C1).

Checkpoint completion:
- [ ] Tasks 3–4 implemented
- [ ] Either moved-suite parity passes
- [ ] Monad moved-suite parity passes
- [ ] old top-level registrations absent
- [ ] test counts match C0
- [ ] state updated
- [ ] no active incident remains

Suggested commit grouping:

```text
test(core): nest Either and monads under typing integration
```

## Task 3 — Move Existing Package Directories Under `typing_integration`

Purpose:
Create the physical package boundary with zero source/helper/test-content change.

Risk:
- Semantic: LOW
- Implementation fanout: multi-file

Owned files and symbols:
- `phalcom-core/tests/core/either/**` — move only
- `phalcom-core/tests/core/monads/**` — move only
- new `phalcom-core/tests/core/typing_integration/`

Inspect before editing:
- both current `mod.rs` files;
- any `include_str!` paths in each local `support.rs`.

Do not inspect unless evidence forces expansion:
- production crates;
- docs outside current package docs.

Dependencies:
- C0 baseline

Source of truth:
- current directories and their complete Git-tracked contents

Implementation boundary:

Changes:
- file ownership/location only.

Must not:
- merge helpers;
- edit Phalcom fixture definitions;
- delete any Rust test;
- rename law IDs.

Current implementation:
Two sibling directories are registered independently.

Target implementation:

```text
phalcom-core/tests/core/typing_integration/either/
phalcom-core/tests/core/typing_integration/monads/
```

with their contents unchanged.

Edit operations:

1. Create the parent directory.
2. Use Git-aware moves:

```bash
mkdir -p phalcom-core/tests/core/typing_integration
git mv phalcom-core/tests/core/either phalcom-core/tests/core/typing_integration/either
git mv phalcom-core/tests/core/monads phalcom-core/tests/core/typing_integration/monads
```

3. Do not edit child files yet.
4. Verify the moved subtrees still contain every C0 file.

Code instructions:

EXACT:
The `git mv` operations above are the intended migration when the local tree still matches the inspected revision.

Testing classification:
- No standalone test before Task 4; the new parent is not registered yet.

Optional compile checkpoint:
None. Registration is established in Task 4.

Checkpoint state update:
Record:
- moved directory roots;
- any local-only files preserved during the move.

## Task 4 — Register One `typing_integration` Module

Purpose:
Make the new directory the single `phalcom-core` test ownership boundary.

Risk:
- Semantic: LOW
- Implementation fanout: local

Owned files and symbols:
- `phalcom-core/tests/core/mod.rs` — top-level integration registration
- `phalcom-core/tests/core/typing_integration/mod.rs` — new sub-suite registry

Inspect before editing:
- current `core/mod.rs` registration block;
- moved `either/mod.rs` and `monads/mod.rs`.

Do not inspect unless evidence forces expansion:
- unrelated `core` modules.

Dependencies:
- Task 3 moves complete

Source of truth:
- `phalcom-core/tests/core/mod.rs` for integration-module registration

Implementation boundary:

Changes:
- replace exactly two top-level module registrations with one;
- create parent module registering the two moved children.

Must not:
- alter unrelated `core/mod.rs` entries;
- register old paths through compatibility aliases.

Current implementation:

```rust
#[path = "either/mod.rs"]
mod either;

#[path = "monads/mod.rs"]
mod monads;
```

Target implementation:

```rust
#[path = "typing_integration/mod.rs"]
mod typing_integration;
```

with parent registry:

```rust
//! Cross-feature static/runtime typing integration conformance package.
//!
//! Focused sub-suites remain separate for diagnosis, while shared source and
//! fixture ownership is introduced by the following checkpoint.

mod either;
mod monads;
```

Edit operations:

1. OPEN `phalcom-core/tests/core/mod.rs`.
2. FIND the current `either` and `monads` path registrations.
3. REPLACE only that block with the single `typing_integration` registration above.
4. CREATE `phalcom-core/tests/core/typing_integration/mod.rs` with the parent registry above.
5. SEARCH `core/mod.rs` for remaining top-level old path registrations.
6. Run C1 evidence commands.

Code instructions:

EXACT:
The replacement snippets above are supported by the inspected test-root structure.

Testing classification:
- No separate unit test; the two moved suites are the checkpoint evidence.

Checkpoint state update:
Record:
- new module path;
- old test counts vs new test counts;
- exact test commands/results.

---

# 12. Checkpoint C2 — Canonical Shared `Either` and Shared Fixture Authority

Tasks:
- Task 5 — create one shared root support harness
- Task 6 — move/rename source fixtures into a shared `sources/` layer
- Task 7 — remove the duplicate `Either` from Monads and compose source builders explicitly
- Task 8 — migrate child imports/callers and delete obsolete support/source authorities

Why this is a checkpoint:
The helper merge and source deduplication are tightly coupled. A shared harness is not meaningful while each child still owns independent fixture constants, and Monads cannot consume the canonical full `Either` until source assembly is changed. Evidence is therefore scheduled after the entire authority migration.

Entry conditions:
- C1 COMPLETE;
- all old tests pass under the nested namespace;
- no content changes have yet been made to source fixtures.

Working set:

Primary:
- `typing_integration/either/support.rs`
- `typing_integration/monads/support.rs`
- `typing_integration/either/either.ph`
- `typing_integration/monads/monads.ph`
- both semantic/runtime probe files
- `typing_integration/either/*.rs`
- `typing_integration/monads/*.rs`
- new `typing_integration/support.rs`
- new `typing_integration/sources/*`

Secondary — inspect only if evidence requires it:
- individual test modules that use support functions whose signatures change
- `phalcom-semantic` public helper types imported by the existing Monads harness

Out of scope for this checkpoint:
- new Expression declarations/tests;
- production semantic code;
- changing any GEN/MON law meaning;
- rewriting old historical docs.

Semantic contract established by this checkpoint:
- the full existing `Either` source is the single shared declaration;
- Monads execute/typecheck against that same `Either`;
- one shared fixture exposes canonical semantic evidence and VM execution;
- existing Either and MON behavior remains unchanged;
- hostile no-Dynamic semantics remain intact.

Semantic risks:
- helper merge weakens identity assertions;
- richer `Either` introduces unexpected resolution changes in Monads;
- source concatenation order leaves `Either` unavailable to Monad declarations;
- generic-solution helper name collision causes a weaker compatibility API to replace the precise one;
- one runtime fixture accidentally concatenates both current runtime probe files and creates duplicate bindings.

Hostile cases:
- `Either<String,_>` vs `Either<Bool,_>` constructor conflict must still fail;
- unrelated `Box<_>` must not satisfy a Monad fixed to `Either<String,_>`;
- underconstrained Either/constructor variables must not become `Dynamic`;
- `ContractEitherMonad` inherited method tests must still select the declaring Functor/Applicative/Monad callables, not executable overrides.

Required evidence:
1. exact hostile Either rejection test — verifies anti-Dynamic behavior after support/source migration.
2. exact hostile Monad constructor-conflict test — verifies shared source did not weaken HKT conflict evidence.
3. full `typing_integration::either::` subtree — preserves all GEN/Either behavior.
4. full `typing_integration::monads::` subtree — preserves all MON behavior.
5. negative searches — exactly one live `Either` declaration and one live support file.

Recommended smallest-first commands:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::either::rejection::conflicting_constructor_context_is_rejected_without_dynamic_escape -- --nocapture

RUSTFLAGS='' cargo test -p phalcom-core --test core \
  typing_integration::monads::rejection::monad_constructor_conflicts_with_unrelated_value_constructor -- --nocapture

RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration::either:: -- --nocapture
RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration::monads:: -- --nocapture
```

Do not run yet:
- full workspace tests — shared production semantics have not changed;
- full `core` target — C2 concerns only the migrated package.

Escalate immediately if:
- Monads only pass when the reduced `Either` copy is restored;
- a test relies on accidental declaration duplication;
- merging helpers appears to require changing `phalcom-semantic` APIs;
- any existing negative test changes from `Invalid`/`Blocked`/formal `Unknown` to `Dynamic`.

Checkpoint completion:
- [ ] Tasks 5–8 implemented
- [ ] shared support is sole support authority
- [ ] shared full Either is sole live Either declaration
- [ ] hostile cases pass
- [ ] full Either subtree passes
- [ ] full Monad subtree passes
- [ ] negative/deletion searches pass
- [ ] state updated
- [ ] no active incident remains

Suggested commit grouping:

```text
test(core): share typing integration fixtures and Either source
```

## Task 5 — Create the Shared Root Support Harness

Purpose:
Replace two near-duplicate fixture implementations with one helper layer that preserves the strongest existing semantic evidence surface.

Risk:
- Semantic: MEDIUM
- Implementation fanout: multi-file

Owned files and symbols:
- new `phalcom-core/tests/core/typing_integration/support.rs`
- current `typing_integration/monads/support.rs` — primary implementation donor
- current `typing_integration/either/support.rs` — selective helper donor

Inspect before editing:
- every public function/method in both support files;
- `rg 'super::support|support::'` in both child test trees;
- all uses of `assert_generic_solution` in the Either tree.

Do not inspect unless evidence forces expansion:
- `phalcom-semantic` internals behind already-public APIs;
- compiler internals.

Dependencies:
- C1 namespace established

Source of truth:
- `phalcom-semantic` canonical IDs and analysis snapshot;
- Monads support API where duplicate helpers disagree.

Implementation boundary:

Changes:
- copy/evolve `monads/support.rs` into the parent;
- add non-conflicting Either shape conveniences;
- migrate conflicting Either helper call sites to exact TypeId APIs.

Must not:
- add string-only substitutes for existing exact identity helpers;
- add two overloaded `assert_generic_solution` APIs;
- remove specialization/kind/provenance helpers to simplify the merge.

Current implementation:
Two child support modules independently parse/analyze/compile/run sources. Monads support has the richer canonical semantic inspection surface. Either support has a convenient `Ty` shape helper but a less strict nominal comparison path and a conflicting `assert_generic_solution` method name.

Target implementation:
One root `support.rs` with:

```text
Fixture / parse + analyze lifecycle
canonical declaration/type lookup
kind helpers
exact TypeParameterId lookup
receiver specialization helpers
exact callable/generic/provenance helpers
Ty shape DSL for ergonomic composite binding assertions
family_type exact-case normalization
run_inline / slot VM helpers
explicit source assembler functions from Task 7
```

Edit operations:

1. COPY the current Monads support implementation to `typing_integration/support.rs` as the base.
2. ADD `TypeKnowledge` / `UnknownReason` imports if required by retained Either helpers.
3. PORT from Either support:
   - `Ty::{Nominal, Applied, Tuple}`;
   - `nominal`, `either`, `tuple` constructors;
   - `family_type`;
   - `assert_type` implemented on top of canonical `assert_nominal`/`TypeStore` identities rather than declaration leaf-name equality;
   - `assert_known_generic_binding`;
   - `assert_binding_applied` / `assert_binding_nominal` if still used;
   - `assert_ready` if still used;
   - `assert_receiver_selection` if still used and non-conflicting;
   - `assert_unknown_underconstrained` if referenced.
4. KEEP the Monads exact `assert_generic_solution(..., expected: TypeId)` and exact-parameter variants.
5. SEARCH Either callers:

```bash
rg 'assert_generic_solution\(' phalcom-core/tests/core/typing_integration/either
```

6. UPDATE those callers to use the canonical root helper, usually with `f.ty("...")`, or exact `TypeParameterId` where names could be ambiguous.
7. DO NOT introduce an overload/shadow API.
8. Leave source constants/builders for Tasks 6–7; compile errors from missing source builder symbols may exist until the checkpoint tasks integrate.

Code instructions:

STRUCTURAL:
The Monads helper is the implementation baseline. Preserve its exact-identity semantics. Port only non-conflicting convenience behavior from Either. Reconcile method bodies with current local API if repository drift changed signatures.

Testing classification:
- No standalone behavioral test; helper correctness is evidenced by both full migrated suites at C2.

Optional compile checkpoint:

```bash
cargo check -p phalcom-core --test core
```

Use only if the helper merge causes broad Rust API fanout and compiler feedback is faster than semantic test execution. It proves Rust caller migration, not semantic equivalence.

Checkpoint state update:
Record:
- final shared helper path;
- any renamed helper and migrated callers;
- whether any old helper behavior was deliberately dropped and why.

## Task 6 — Move and Rename Fixture Sources Into `sources/`

Purpose:
Make fixture ownership explicit and prevent ambiguous `semantic_probes.ph` / `runtime_probes.ph` names from competing across sub-suites.

Risk:
- Semantic: LOW
- Implementation fanout: multi-file

Owned files and symbols:
- current child `.ph` files
- new `typing_integration/sources/`

Inspect before editing:
- all `include_str!` calls in both support modules;
- runtime probe top-level binding names.

Do not inspect unless evidence forces expansion:
- unrelated Phalcom examples/specs.

Dependencies:
- Task 5 parent support location established

Source of truth:
- existing fixture file content; this task moves/renames but does not semantically rewrite it.

Implementation boundary:

Changes:
- move source files to explicit shared names.

Must not:
- concatenate runtime probe files;
- alter probe logic during movement.

Current implementation:
Both child directories contain generically named `semantic_probes.ph` / `runtime_probes.ph`; each support module includes files relative to itself.

Target implementation:

```text
sources/either.ph
sources/either_semantic_probes.ph
sources/either_runtime_probes.ph
sources/monads.ph
sources/monad_semantic_probes.ph
sources/monad_runtime_probes.ph
```

Edit operations:

```bash
mkdir -p phalcom-core/tests/core/typing_integration/sources

git mv phalcom-core/tests/core/typing_integration/either/either.ph \
       phalcom-core/tests/core/typing_integration/sources/either.ph

git mv phalcom-core/tests/core/typing_integration/either/semantic_probes.ph \
       phalcom-core/tests/core/typing_integration/sources/either_semantic_probes.ph

git mv phalcom-core/tests/core/typing_integration/either/runtime_probes.ph \
       phalcom-core/tests/core/typing_integration/sources/either_runtime_probes.ph

git mv phalcom-core/tests/core/typing_integration/monads/monads.ph \
       phalcom-core/tests/core/typing_integration/sources/monads.ph

git mv phalcom-core/tests/core/typing_integration/monads/semantic_probes.ph \
       phalcom-core/tests/core/typing_integration/sources/monad_semantic_probes.ph

git mv phalcom-core/tests/core/typing_integration/monads/runtime_probes.ph \
       phalcom-core/tests/core/typing_integration/sources/monad_runtime_probes.ph
```

Keep `either/invalid/` under the Either sub-suite; its `include_str!("invalid/...")` paths remain local to `either/rejection.rs` and do not represent shared source authority.

Code instructions:

EXACT:
The move mapping above follows inspected source ownership.

Testing classification:
- No standalone behavioral test; Task 7 reconnects source builders.

Checkpoint state update:
Record moved file mapping.

## Task 7 — Deduplicate `Either` and Define Explicit Source Assemblers

Purpose:
Make the full shared `Either` the only declaration consumed by both suites and give every test layer an explicit source composition API.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- `typing_integration/sources/either.ph`
- `typing_integration/sources/monads.ph`
- `typing_integration/support.rs` source constants/builders

Inspect before editing:
- beginning of moved `sources/monads.ph` through the end of its duplicate `Either` declaration;
- every current source-builder call in both child trees.

Do not inspect unless evidence forces expansion:
- production generic inference;
- core Universe `Either`/Result implementation (this is a user-defined test type).

Dependencies:
- Tasks 5–6

Source of truth:
- `sources/either.ph` for the test ADT;
- `sources/monads.ph` for Box/Functor/Applicative/Monad and algorithms.

Implementation boundary:

Changes:
- delete the duplicate `Either` block from `sources/monads.ph`;
- ensure Monad fixtures assemble `Either` before Monad declarations;
- keep semantic/runtime probe builders feature-specific.

Must not:
- remove methods from the canonical rich Either;
- make `monads.ph` independently redeclare a partial Either;
- concatenate Both runtime probe files into one fixture.

Current implementation:
`monads.ph` begins with a reduced duplicate `Either`, allowing the Monads package to be self-contained but creating two authorities across the broader core suite.

Target implementation:
`monads.ph` begins with:

```phalcom
class Box<T> {}
```

and contains only the Monad/HKT ecosystem. The shared harness composes sources.

Edit operations:

1. OPEN `sources/monads.ph`.
2. REMOVE the entire initial `enum Either<L, R> { ... }` block; retain `class Box<T> {}` onward unchanged unless formatting requires otherwise.
3. In `support.rs`, define constants:

```rust
const EITHER_SOURCE: &str = include_str!("sources/either.ph");
const EITHER_SEMANTIC_PROBES: &str = include_str!("sources/either_semantic_probes.ph");
const EITHER_RUNTIME_PROBES: &str = include_str!("sources/either_runtime_probes.ph");
const MONADS_SOURCE: &str = include_str!("sources/monads.ph");
const MONAD_SEMANTIC_PROBES: &str = include_str!("sources/monad_semantic_probes.ph");
const MONAD_RUNTIME_PROBES: &str = include_str!("sources/monad_runtime_probes.ph");
```

4. Provide explicit builders with these semantics:

```text
either_source
    Either only

either_semantic_source
    Either + Either semantic probes

either_runtime_source
    Either + Either runtime probes

with_either(extra)
    Either + extra

monads_source
    Either + Monad/HKT core

monad_semantic_source
    Either + Monad/HKT core + Monad semantic probes

monad_runtime_source
    Either + Monad/HKT core + Monad runtime probes

with_monads(extra)
    Either + Monad/HKT core + extra
```

5. The exact return type (`String` vs borrowed `&str`) should follow composition needs. Because the combined Monad source is assembled from two files, returning `String` is the straightforward target; update callers accordingly rather than inventing a second concatenated source authority.
6. Search and update every source-builder caller.
7. Preserve separate runtime builders so `runtimeAll` and other top-level probe names never collide.

Code instructions:

STRUCTURAL:
The constant names and composition semantics above are required. Reconcile exact function signatures with local caller ergonomics. Do not use unsafe lifetime tricks or duplicate static blobs merely to retain an old `&'static str` signature.

Testing classification:
- High-risk shared source change; validated at C2 after Task 8 caller migration.

Checkpoint state update:
Record:
- canonical source order for each builder;
- changed builder signatures and callers;
- confirmation that `sources/monads.ph` no longer defines `Either`.

## Task 8 — Migrate Child Imports and Remove Obsolete Authorities

Purpose:
Finish the helper/source migration so old child-local authorities cannot silently continue to run.

Risk:
- Semantic: MEDIUM
- Implementation fanout: multi-file

Owned files and symbols:
- `typing_integration/either/mod.rs`
- `typing_integration/monads/mod.rs`
- all child Rust test modules importing `super::support`
- old child `support.rs` files

Inspect before editing:
- `rg 'super::support' typing_integration/either typing_integration/monads`
- `rg 'mod support;' ...`
- source builder usages.

Do not inspect unless evidence forces expansion:
- unrelated core integration modules.

Dependencies:
- shared root support/builders from Tasks 5–7

Source of truth:
- parent `typing_integration::support`

Implementation boundary:

Changes:
- register `mod support;` once in parent `typing_integration/mod.rs`;
- remove child `mod support;` declarations;
- update child imports to parent support;
- delete old support files.

Must not:
- leave re-export compatibility files after C2;
- hide remaining child-local source constants behind aliases.

Current implementation:
Each child owns `mod support;` and its tests import `super::support`.

Target implementation:
Parent:

```rust
mod support;
mod either;
mod monads;
```

Child test modules resolve shared support via the parent, typically:

```rust
use super::super::support::{...};
```

or another equally explicit local-module path consistent with Rust visibility. Do not add a second support module solely to preserve `super::support` syntax.

Edit operations:

1. ADD `mod support;` to `typing_integration/mod.rs`.
2. REMOVE `mod support;` from both child `mod.rs` files.
3. UPDATE every child test import from child-local support to parent support.
4. UPDATE Either generic-solution call sites per Task 5's canonical API decision.
5. DELETE:

```text
typing_integration/either/support.rs
typing_integration/monads/support.rs
```

6. SEARCH:

```bash
rg 'mod support;' phalcom-core/tests/core/typing_integration
rg 'super::support' phalcom-core/tests/core/typing_integration/either \
                   phalcom-core/tests/core/typing_integration/monads
```

Expected:
- one `mod support;` in parent;
- no imports targeting a nonexistent child support.

7. Run C2 required evidence.
8. Run deletion gates:

```bash
rg -n '^\s*enum Either<L, R>' phalcom-core/tests/core/typing_integration
find phalcom-core/tests/core/typing_integration -name support.rs -print
```

Expected:
- exactly one `enum Either<L, R>` hit: `sources/either.ph`;
- exactly one support file: `typing_integration/support.rs`.

Code instructions:

STRUCTURAL:
Mechanical caller updates should not change test assertions other than adapting to the canonical shared helper API.

Testing classification:
- Required C2 evidence as listed above.

Checkpoint state update:
Record:
- deletion-search outputs;
- exact test counts vs C0/C1;
- any caller whose assertion semantics had to be strengthened during helper collision resolution.

---

# 13. Checkpoint C3 — Unified Package Contract and Explicit Cross-Feature Scenario

Tasks:
- Task 9 — unify current documentation/law catalog without renumbering existing laws
- Task 10 — add one explicit direct-Either + Monad cross-feature semantic probe

Why this is a checkpoint:
After C2 the package is structurally unified, but it could still be merely two unrelated sub-suites sharing files. C3 establishes the package-level semantic contract and proves that one analysis can traverse both direct `Either` operations and higher-kinded Monad algorithms over the same canonical declaration.

Entry conditions:
- C2 COMPLETE;
- one canonical Either and one shared support module verified;
- both legacy sub-suites green.

Working set:

Primary:
- `typing_integration/README.md`
- `typing_integration/LAWS.md`
- existing `typing_integration/monads/README.md`
- existing `typing_integration/monads/LAWS.md`
- `typing_integration/monads/monad-testing.md`
- new `typing_integration/integration.rs`
- new `typing_integration/sources/integration_probes.ph`
- `typing_integration/support.rs`
- `typing_integration/mod.rs`

Secondary — inspect only if evidence requires it:
- `docs/spec/typing-generics.md` — GEN authority

Out of scope for this checkpoint:
- adding new Monad laws unrelated to integration;
- Expression GADT implementation;
- production semantic changes.

Semantic contract established by this checkpoint:
- current package docs explain the unified ownership model;
- all existing `MON-*` law IDs remain stable;
- GEN laws remain sourced from `docs/spec/typing-generics.md` rather than duplicated;
- a single `SemanticAnalysis` proves direct `Either.map` and `MonadAlgorithms.bind` on the same `Either` declaration/specialization.

Semantic risks:
- law catalog accidentally renumbered during relocation;
- integration smoke test proves only final pretty-printed types, not exact call/generic evidence;
- current docs continue claiming `monads/` is the entire package authority.

Hostile cases:
- integration probe would still pass if a second hidden Either declaration existed; defeat this with the C2 single-declaration gate and exact `DeclarationId` assertions;
- final type is correct through `Dynamic`; defeat this by asserting Ready/canonical type/generic solution provenance.

Required evidence:
1. new focused `typing_integration::integration::...` test — proves same canonical Either participates in both paths.
2. `RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration:: -- --nocapture` — proves unified package coherence.
3. current-doc path/law searches — prove migrated live docs no longer claim old package ownership.

Do not run yet:
- full workspace tests — C5.

Escalate immediately if:
- the cross-feature probe only passes with a duplicate source declaration;
- exact `F` inference cannot be asserted using existing explanation products;
- documentation migration reveals conflicting law definitions rather than path-only drift.

Checkpoint completion:
- [ ] Tasks 9–10 implemented
- [ ] MON IDs preserved
- [ ] GEN authority referenced, not duplicated
- [ ] cross-feature semantic probe passes
- [ ] entire unified package passes
- [ ] current documentation references are correct
- [ ] state updated
- [ ] no active incident remains

Suggested commit grouping:

```text
test(core): establish unified typing integration contract
```

## Task 9 — Unify Current Documentation and Law Catalog

Purpose:
Make the package's test contract discoverable and stable before adding GADT laws.

Risk:
- Semantic: LOW
- Implementation fanout: multi-file

Owned files and symbols:
- existing `typing_integration/monads/README.md`
- existing `typing_integration/monads/LAWS.md`
- existing `typing_integration/monads/monad-testing.md`
- new root README/LAWS

Inspect before editing:
- full existing Monads README/LAWS;
- `docs/spec/typing-generics.md` GEN headings referenced by Either tests.

Do not inspect unless evidence forces expansion:
- historical completed plans/logs.

Dependencies:
- C2 unified ownership

Source of truth:
- `docs/spec/typing-generics.md` for GEN semantics;
- existing `MON-*` law text/IDs for Monad conformance semantics.

Implementation boundary:

Changes:
- move or adapt current Monads README/LAWS into root package documentation;
- preserve MON IDs exactly;
- describe Either as foundation and Monads as abstraction layer;
- reserve GEX/INT sections for C4 additions.

Must not:
- renumber existing MON laws;
- duplicate the entire GEN spec into test docs;
- claim category-theoretic Functor/Applicative/Monad law proof (the current README explicitly disclaims that).

Current implementation:
Monads has a mature README/LAWS contract; Either relies on `GEN-*` spec references and has no equivalent root package doc.

Target implementation:

```text
typing_integration/README.md
    purpose, architecture, fixture/source layers, focused sub-suites

typing_integration/LAWS.md
    GEN mapping/reference
    preserved MON-* catalog
    GEX-* / INT-* sections added by C4

monads/monad-testing.md
    retained as a focused walkthrough, with paths updated to the unified package
```

Edit operations:

1. Use the existing Monads README as the factual base for the unified README.
2. Move/adapt the Monads LAWS catalog to `typing_integration/LAWS.md`.
3. Update its statement of package path/authority.
4. Preserve all existing `MON-*` headings and mapped Rust test function names.
5. Add a concise GEN section pointing to `docs/spec/typing-generics.md` and the `either/` sub-suite.
6. Update `monad-testing.md` paths/tree to the new package.
7. Remove obsolete current-package README/LAWS copies if they would create two authorities. A focused walkthrough may remain under `monads/` but must not call itself the authoritative law catalog.
8. Search current live package docs for old ownership claims.

Code instructions:

STRUCTURAL:
This is a documentation migration, not a semantic rewrite. Preserve existing law wording unless a path/ownership statement is objectively stale.

Testing classification:
- No standalone behavioral test; doc/deletion checks and C3 package test provide evidence.

Checkpoint state update:
Record:
- authoritative root documentation paths;
- preserved law-ID count if easily obtainable;
- intentionally retained historical references outside the live package.

## Task 10 — Add an Explicit Unified Direct-Either + Monad Semantic Scenario

Purpose:
Prove the new package is semantically integrated rather than merely co-located.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- new `typing_integration/sources/integration_probes.ph`
- new `typing_integration/integration.rs`
- `typing_integration/support.rs` — add only the necessary source assembler
- `typing_integration/mod.rs` — register `integration`

Inspect before editing:
- Either semantic probes (`EitherInferenceProbe` patterns);
- Monad `algorithmBind` / inference tests;
- shared support exact callable/generic helpers.

Do not inspect unless evidence forces expansion:
- production inference implementation.

Dependencies:
- canonical shared source/harness from C2

Source of truth:
- one shared semantic analysis using the same `DeclarationId` for `Either`.

Implementation boundary:

Changes:
- add a small cross-feature Phalcom probe;
- assert direct `Either.map` and generic `MonadAlgorithms.bind` exact types/callables/constructor inference in one fixture.

Must not:
- add another Either/Monad definition;
- assert only display strings;
- use runtime as the only evidence.

Current implementation:
The existing suites independently prove direct Either behavior and Monad behavior, but no current test intentionally asserts both through one canonical shared source universe because the packages previously owned separate Either declarations.

Target implementation:
The probe should have the shape:

```phalcom
class UnifiedTypingProbe {
    @class
    run(
        _ monad: StringEitherMonad,
        _ source: Either<String, Int>
    ) {
        let mapped = source.map(|value| {
            value > 0
        })

        let bound = MonadAlgorithms.bind(
            monad,
            source,
            |value| {
                let next: Either<String, Bool> = Either::Right(value > 0)
                next
            }
        )
    }
}
```

Required Rust assertions:

```text
mapped : Either<String,Bool>
bound  : Either<String,Bool>

direct call target == canonical Either.map callable
bind call target   == canonical MonadAlgorithms.bind callable

bind F solution == unary constructor equivalent to <X> =>> Either<String,X>
A/B solutions are exact and callable-owned as expected
both expressions are AnalysisStatus::Ready
no semantic errors/internal incidents
```

Edit operations:

1. CREATE `sources/integration_probes.ph` with the probe above, reconciled to current syntax if needed.
2. ADD a builder such as `integration_semantic_source()` composing:

```text
Either
+ monads core
+ integration probes
```

3. CREATE `integration.rs` using exact shared support assertions.
4. REGISTER `mod integration;` in parent `mod.rs`.
5. Assert the exact `Either` declaration identity used by the direct call and by the reduced/application result; do not accept same-name-only equivalence.
6. Run the focused test then the full C3 package gate.

Code instructions:

STRUCTURAL:
The Phalcom probe shape above is expected to work with the inspected syntax; exact assertion calls should reuse the existing Monads support APIs rather than introducing new generic-proof machinery.

Testing classification:
- Focused regression required because this is the first independently meaningful package-level invariant.

Checkpoint state update:
Record:
- test function name;
- exact F/A/B solutions observed;
- callable identities asserted;
- complete package command/result.

---

# 14. Checkpoint C4 — Typed Effectful Expression GADT Vertical Integration

Tasks:
- Task 11 — add canonical `Expression<F,T>` source, evaluator, and `ExpressionMonad<F>`
- Task 12 — add semantic GADT/HKT/higher-order positive tests
- Task 13 — add Monad/type-lambda/Either specialization tests
- Task 14 — add hostile Expression rejection tests
- Task 15 — add VM execution and one flagship full-stack scenario

Why this is a checkpoint:
The Expression test is valuable only when the whole chain composes:

```text
GADT result index
→ branch-local equality proof
→ recursive generic eval specialization
→ Monad<F> generic method specialization
→ higher-order closure inference
→ type-lambda constructor capture
→ concrete Either<String,_> effect
→ compiler/VM execution
```

Testing isolated constructor syntax after each source edit would produce little semantic evidence. The checkpoint instead builds the source and focused Rust assertions, then proves the integrated boundary.

Entry conditions:
- C3 COMPLETE;
- existing shared Monad/Either ecosystem is stable;
- the ownership-layer GADT refinement suite is present.

Working set:

Primary:
- `typing_integration/sources/expression.ph`
- `typing_integration/sources/expression_semantic_probes.ph`
- `typing_integration/sources/expression_runtime_probes.ph`
- `typing_integration/expression/*.rs`
- `typing_integration/support.rs`
- `typing_integration/LAWS.md`
- `phalcom-semantic/tests/semantic/adts/matching/gadt_refinement.rs` — evidence dependency, not normally edited

Secondary — inspect only if evidence requires it:
- `docs/spec/adts.md` — ratified GADT source syntax
- `phalcom-semantic/src/checker/gadt_proof.rs` — ownership boundary during diagnosis only
- `phalcom-semantic/src/checker/pattern.rs`
- `phalcom-semantic/src/checker/exhaustiveness.rs`

Out of scope for this checkpoint:
- changing GADT semantics;
- new parser syntax;
- rank-2 polymorphism;
- Free monads, transformers, recursion schemes, lenses;
- rewriting the existing semantic GADT ownership suite;
- production fixes discovered incidentally unless separately authorized after incident diagnosis.

Semantic contract established by this checkpoint:
- `Expression<F,T>` is a real higher-kinded GADT whose cases refine `T`;
- `ExpressionEvaluation.eval<F,T>(Monad<F>, Expression<F,T>) -> F<T>` typechecks using branch-local GADT evidence;
- higher-order `Map`, `FlatMap`, and `Apply` relationships preserve callable inputs/results;
- `<X> =>> Expression<F,X>` is a valid captured unary constructor for `Monad` specialization;
- concrete `F = <X> =>> Either<String,X>` evaluates to exact `Either<String,T>`;
- invalid GADT/HKT compositions fail rather than widening to `Dynamic`;
- VM behavior agrees with semantic preflight for success and short-circuit failure.

Semantic risks:
- GADT proof exists but is not consumed when checking `F<T>` branch results;
- constructor-local generic variables are accidentally shared across variants/calls;
- captured outer `F` is lost in `<X> =>> Expression<F,X>`;
- recursive `eval` calls prematurely specialize the outer generic `T` globally;
- higher-order function type parameters inside `Apply` are inverted or widened;
- failure `Lift(Either::Left(...))` becomes underconstrained unless expected type evidence is provided;
- VM lowering supports basic GADTs but not the more complex nested higher-order variant shapes;
- an integration test exposes a real production defect and the implementer expands scope without diagnosis.

Hostile cases:
- `Add(Expression<F,Bool>, Expression<F,Int>)` is rejected;
- `If` with an `Int` condition is rejected;
- `If` branches with `Int` vs `String` result indexes are rejected;
- `Apply` of `(String)->Int` to `Expression<F,Int>` is rejected;
- `FlatMap` source `Expression<F,Int>` with continuation `(String)->Expression<F,_>` is rejected;
- an `Expression<Either<String,_>,T>` cannot be evaluated with `BoxMonad` as though constructors were interchangeable;
- none of those failures is repaired as `Dynamic`.

Required evidence:
1. existing semantic GADT ownership suite — verifies dependency before blaming integration:

```bash
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic \
  semantic::adts::matching::gadt_refinement -- --nocapture
```

If local module filtering differs due repository drift, adapt only the Cargo filter; run the same file/module ownership suite.

2. focused Expression semantic positive subtree.
3. focused Expression hostile negative subtree.
4. focused Expression runtime subtree.
5. full `typing_integration::expression::` subtree.
6. full `typing_integration::` package gate after Expression passes.

Do not run yet:
- workspace delivery gates — C5.

Escalate immediately if:
- ownership-layer GADT tests are red before Expression work;
- Expression requires a parser/AST syntax change despite using already-ratified forms;
- the only way to typecheck evaluator branches is a production change outside planned semantics;
- runtime requires modifying ADT lowering or VM representation;
- any hostile test becomes `Dynamic`.

Checkpoint completion:
- [ ] Tasks 11–15 implemented
- [ ] existing ownership-layer GADT suite passes
- [ ] Expression positive semantics pass
- [ ] hostile negatives pass
- [ ] Expression runtime passes
- [ ] flagship full-stack scenario passes
- [ ] entire typing integration package passes
- [ ] GEX/INT law entries added
- [ ] state updated
- [ ] no active incident remains

Suggested commit grouping:

```text
test(core): add effectful GADT expression typing integration
```

Optionally split source/semantic vs runtime evidence if review size benefits:

```text
test(core): add typed Expression GADT semantic integration
test(core): execute Expression over Either Monad
```

## Task 11 — Add `Expression<F,T>`, Evaluator, and `ExpressionMonad<F>`

Purpose:
Create the reusable Phalcom source program that forces GADT refinement to compose with the existing HKT/Monad ecosystem.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- new `typing_integration/sources/expression.ph`
- `Expression<F,T>`
- `ExpressionEvaluation.eval<F,T>`
- `ExpressionMonad<F>`
- concrete expression-monad specialization used by probes/runtime
- `typing_integration/support.rs` expression source builders

Inspect before editing:
- `docs/spec/adts.md` GADT syntax;
- existing GADT example in `examples/ide-golden/phalcom-enum-highlighting-showcase.ph`;
- `sources/monads.ph` contract method signatures;
- existing semantic GADT evaluator test for branch-result behavior.

Do not inspect unless evidence forces expansion:
- parser implementation;
- `gadt_proof.rs` internals.

Dependencies:
- C3 existing Monad/Either ecosystem

Source of truth:
- ratified GADT source syntax;
- existing `Monad<F>` API;
- semantic branch proof engine.

Implementation boundary:

Changes:
- add user-level source only;
- no production compiler/type-system code.

Must not:
- add a duplicate Monad hierarchy;
- add test-only GADT proof annotations;
- invent new language syntax.

Current implementation:
No effectful Expression fixture exists in the unified package. `phalcom-semantic` already proves a simple `Expr<T>` generic evaluator.

Target implementation:
The initial vertical test should use this conceptual source shape:

```phalcom
enum Expression<F: Type -> Type, T> {
    @variant Pure<A>(
        _ value: A
    ) -> Expression<F, A>

    @variant IntLiteral(
        _ value: Int
    ) -> Expression<F, Int>

    @variant BoolLiteral(
        _ value: Bool
    ) -> Expression<F, Bool>

    @variant Add(
        _ left: Expression<F, Int>,
        _ right: Expression<F, Int>
    ) -> Expression<F, Int>

    @variant If<A>(
        _ condition: Expression<F, Bool>,
        _ yes: Expression<F, A>,
        _ no: Expression<F, A>
    ) -> Expression<F, A>

    @variant Map<A, B>(
        _ source: Expression<F, A>,
        _ transform: (A) -> B
    ) -> Expression<F, B>

    @variant FlatMap<A, B>(
        _ source: Expression<F, A>,
        _ next: (A) -> Expression<F, B>
    ) -> Expression<F, B>

    @variant Apply<A, B>(
        _ function: Expression<F, (A) -> B>,
        _ argument: Expression<F, A>
    ) -> Expression<F, B>

    @variant Lift<A>(
        _ effect: F<A>
    ) -> Expression<F, A>
}
```

The exact field-label style may be reconciled to the local parser conventions, but the type relationships above are required.

The evaluator contract is:

```phalcom
class ExpressionEvaluation {
    @class
    eval<F: Type -> Type, T>(
        _ monad: Monad<F>,
        _ expression: Expression<F, T>
    ) -> F<T> {
        match expression {
            // every branch must use existing Monad operations and real recursive eval
        }
    }
}
```

Required branch semantics:

```text
Pure<A>
    monad.pure(value)

IntLiteral
    monad.pure(value)

BoolLiteral
    monad.pure(value)

Add
    eval left : F<Int>
    eval right: F<Int>
    combine with flatMap/map → F<Int>

If<A>
    eval condition : F<Bool>
    flatMap; selected branch recursively evals to F<A>

Map<A,B>
    monad.map(eval(source), transform) → F<B>

FlatMap<A,B>
    monad.flatMap(eval(source), |value| eval(next.call(value))) → F<B>

Apply<A,B>
    eval function : F<(A)->B>
    eval argument : F<A>
    flatMap/map callable application → F<B>

Lift<A>
    effect : F<A>
```

Add:

```phalcom
class ExpressionMonad<F: Type -> Type>
    is Monad<<X> =>> Expression<F, X>> {
    // executable map/pure/map2/flatMap implemented by constructing
    // Expression::Map / Pure / FlatMap nodes
}
```

and a concrete specialization suitable for tests, for example:

```phalcom
class StringEitherExpressionMonad
    is ExpressionMonad<<X> =>> Either<String, X>> {}
```

Use the existing `StringEitherMonad` as the evaluator's outer effect Monad. Do not conflate it with the expression-construction Monad; they serve different constructors:

```text
StringEitherMonad
    Monad<<X> =>> Either<String,X>>

StringEitherExpressionMonad
    Monad<<X> =>> Expression<<Y> =>> Either<String,Y>, X>>
```

Source builders to add in `support.rs`:

```text
expression_source
    Either + monads + Expression core

expression_semantic_source
    above + Expression semantic probes

expression_runtime_source
    above + Expression runtime probes

with_expression(extra)
    above + extra
```

Code instructions:

STRUCTURAL:
The type signatures and relationships above are mandatory. This task has not been compiled during planning, so do not treat the bodies as paste-ready if current Phalcom syntax requires small mechanical adjustment. Do not alter the semantic shape to avoid a compiler/type-checker failure; classify such a failure first.

Testing classification:
- No standalone evidence until semantic probes in Tasks 12–13; parse/analyze smoke may be used during development, but checkpoint evidence is scheduled after integrated tests exist.

Checkpoint state update:
Record:
- exact landed variant set/signatures;
- any mechanical syntax adaptation;
- no production files changed.

## Task 12 — Add GADT/HKT/Higher-Order Positive Semantic Tests

Purpose:
Prove branch-local GADT evidence is usable inside higher-kinded and higher-order evaluator code.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- new `sources/expression_semantic_probes.ph`
- new `expression/refinement.rs`
- new `expression/higher_order.rs`
- `expression/mod.rs`

Inspect before editing:
- ownership-layer `gadt_refinement.rs` test style;
- shared `Fixture` callable/expression helpers;
- existing Either/Monad higher-order tests.

Do not inspect unless evidence forces expansion:
- GADT implementation internals.

Dependencies:
- Task 11 source declarations

Source of truth:
- `SemanticAnalysis` expressions/callable identities plus existing match/GADT products;
- ownership-layer GADT suite for low-level proof correctness.

Implementation boundary:

Changes:
- add positive integration tests only.

Must not:
- reassert every existing low-level GADT proof-engine law;
- accept “no diagnostic” as the only proof.

Current implementation:
Simple GADT evaluator proofs exist in `phalcom-semantic`; no test combines those with `Monad<F>` and function-valued GADT fields.

Target implementation:
At minimum, establish these initial laws in `LAWS.md` and corresponding tests:

```text
GEX-01
Result-specialized GADT variants allow eval<F,T> branches to produce F<T>
without widening T.

GEX-02
Constructor-local A/B variables in Map/FlatMap/Apply are fresh and branch-local.

GEX-03
A GADT equality such as T = Int propagates through HKT application F<T>.

GEX-04
Apply<A,B> preserves the relationship:
Expression<F,(A)->B> × Expression<F,A> -> Expression<F,B>.

GEX-05
FlatMap<A,B> composes recursive eval and Monad<F>.flatMap to produce exact F<B>.
```

Recommended semantic probes:

```text
evaluate Int expression under Either effect -> Either<String,Int>
evaluate Bool expression under Either effect -> Either<String,Bool>
construct Apply with Int -> Bool callable -> Expression<F,Bool>
construct Map Int -> String/Bool -> exact indexed result
construct FlatMap Int -> Bool -> exact indexed result
```

Required assertion style:
- binding type exact and canonical;
- call expression `Ready`;
- exact selected callable where applicable;
- exact generic solution identities for `Monad.map`/`flatMap` calls where exposed;
- no semantic errors/internal incidents;
- no `Dynamic`/unexplained `Unknown` on the critical path.

Edit operations:

1. CREATE the semantic probe source with small named classes/methods rather than one giant `run` body.
2. CREATE `expression/refinement.rs` for evaluator/index preservation.
3. CREATE `expression/higher_order.rs` for `Map`/`FlatMap`/`Apply` callable relationships.
4. REGISTER both modules in `expression/mod.rs`; register `mod expression;` in parent.
5. Add `GEX-01` through initial GEX entries to root `LAWS.md` with exact test names.
6. Run focused tests as they become meaningful, then defer package-level gate to end of C4.

Code instructions:

STRUCTURAL:
Reuse existing exact-ID/provenance helpers. If a needed branch-proof product is not exposed through the core fixture, prefer relying on the ownership-layer GADT test plus observable exact evaluator call types rather than adding a parallel proof parser to the test helper.

Testing classification:
- Focused high-risk semantic regressions required.

Checkpoint state update:
Record test names and exact canonical types/solutions asserted.

## Task 13 — Prove `ExpressionMonad<F>` and Concrete Either Type-Lambda Specialization

Purpose:
Force captured constructor variables and the existing generic Monad algorithms to operate over the GADT constructor itself.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- new `expression/monad.rs`
- `sources/expression_semantic_probes.ph`
- root `LAWS.md`

Inspect before editing:
- existing `monads/type_lambdas.rs`;
- existing `monads/inheritance.rs`;
- existing `monads/composition.rs`;
- `ExpressionMonad<F>` from Task 11.

Do not inspect unless evidence forces expansion:
- specialization implementation internals.

Dependencies:
- Task 11 source
- Task 12 core Expression inference stable

Source of truth:
- canonical `TypeData::Lambda`, type-parameter identities, receiver specialization paths, generic explanation graph.

Implementation boundary:

Changes:
- add tests reusing existing Monad contract/algorithms with the Expression unary constructor.

Must not:
- create Expression-specific copies of `bind`, `kleisli`, or `traverse` merely because inference is difficult.

Current implementation:
Monads proves captured type lambda `<X> =>> Either<E,X>`. Expression introduces a nested captured constructor:

```text
<X> =>> Expression<F,X>
```

and, concretely:

```text
<X> =>> Expression<<Y> =>> Either<String,Y>, X>
```

Target implementation:
Add laws/test cases such as:

```text
GEX-06
<X> =>> Expression<F,X> has kind Type -> Type and captures F without capturing X.

GEX-07
ExpressionMonad<F> projects through Monad/Applicative/Functor with the exact
Expression constructor.

INT-01
For F = <X> =>> Either<String,X>, evaluating Expression<F,T> yields exact
Either<String,T>.

INT-02
Existing MonadAlgorithms.bind/kleisli/traverse can infer the Expression
constructor without a dedicated algorithm copy.
```

At minimum, test one existing generic algorithm over `StringEitherExpressionMonad`. Prefer a manageable first composition such as `bind` or `kleisli`; the flagship `traverse → Expression<F,List<B>> → eval → F<List<B>>` belongs to Task 15 after base semantics are proven.

Required evidence details:
- exact constructor kind;
- exact lambda/capture structure or beta-reduced applications, using existing type-lambda helper style;
- inherited callable selection path when relevant;
- exact result type after algorithm application.

Code instructions:

STRUCTURAL:
Mirror existing Monads test idioms. The point is constructor substitution composition, not inventing new helper abstractions.

Testing classification:
- Focused high-risk semantic regressions required.

Checkpoint state update:
Record the exact nested constructor forms and the existing algorithms successfully reused.

## Task 14 — Add Hostile Expression Rejection Tests

Purpose:
Ensure the integrated GADT/HKT program remains sound and does not use gradual-typing escape hatches to paper over contradictions.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file

Owned files and symbols:
- new `expression/rejection.rs`
- optional `expression/invalid/*.ph` fixtures
- root `LAWS.md`

Inspect before editing:
- `either/rejection.rs` anti-Dynamic patterns;
- `monads/rejection.rs` HKT conflict/proof patterns;
- current diagnostic codes produced by the smallest representative invalid source.

Do not inspect unless evidence forces expansion:
- production diagnostics implementation.

Dependencies:
- positive Expression source/inference established enough to make invalid deltas meaningful

Source of truth:
- analyzer status/knowledge/diagnostic products

Implementation boundary:

Changes:
- add invalid source probes and exact fail-closed assertions.

Must not:
- overfit to one diagnostic code if repository semantics legitimately classify the same contradiction at an adjacent binding/generic layer; follow the established Either approach when multiple accepted conflict codes are semantically equivalent;
- assert merely “has any error.”

Current implementation:
Existing Either/Monad suites already defend no-Dynamic behavior, but Expression adds GADT-index and effect-constructor contradictions.

Target hostile cases:

```phalcom
// wrong Add operand
Expression::Add(
    Expression::BoolLiteral(true),
    Expression::IntLiteral(1)
)

// wrong If condition
Expression::If(
    Expression::IntLiteral(1),
    Expression::IntLiteral(10),
    Expression::IntLiteral(20)
)

// incompatible branch index
Expression::If(
    Expression::BoolLiteral(true),
    Expression::IntLiteral(10),
    Expression::Pure("twenty")
)

// Apply mismatch
Expression::Apply(
    Expression::Pure(|value: String| { value.size }),
    Expression::IntLiteral(42)
)

// FlatMap continuation input mismatch
Expression::FlatMap(
    Expression::IntLiteral(42),
    |value: String| { Expression::Pure(value) }
)
```

Also add an effect-constructor mismatch at evaluator call level: an expression fixed to the Either effect must not be accepted with a `BoxMonad` merely because both constructors have kind `Type -> Type`.

For every case assert:

```text
critical call/expression is Invalid or otherwise formally rejected;
critical knowledge is not Dynamic;
expected conflict/mismatch diagnostic belongs to the probe range;
no internal semantic incident.
```

Add `GEX-REJECT-*` or `INT-REJECT-*` entries to `LAWS.md` using a stable numbering convention chosen once in this checkpoint.

Code instructions:

STRUCTURAL:
Use the current analyzer's canonical diagnostic classifications after running the smallest invalid probe. Do not weaken the semantic assertion to accommodate an unexpected result; classify unexpected behavior first.

Testing classification:
- Mandatory hostile-case evidence for high-risk semantics.

Checkpoint state update:
Record:
- invalid fixtures/test names;
- exact statuses/diagnostics;
- confirmation no critical path became Dynamic.

## Task 15 — Add Expression VM Execution and Flagship Full-Stack Scenario

Purpose:
Prove the static constructor/index/effect relationships correspond to executable behavior and exercise an existing generic Monad algorithm over the Expression constructor.

Risk:
- Semantic: HIGH
- Implementation fanout: multi-file / cross semantic-compiler-VM integration

Owned files and symbols:
- new `sources/expression_runtime_probes.ph`
- new `expression/runtime.rs`
- new `expression/integration.rs`
- shared `expression_runtime_source()` builder
- root `LAWS.md`

Inspect before editing:
- existing Either runtime test pattern;
- existing Monads runtime test pattern, especially semantic preflight and short-circuit assertions;
- `MonadAlgorithms.traverse` semantics;
- VM-visible primitive slot extraction helper.

Do not inspect unless evidence forces expansion:
- compiler/VM internals.

Dependencies:
- Tasks 11–14 semantic layer green

Source of truth:
- exact source semantically analyzed then executed by ProgramCompiler/VM

Implementation boundary:

Changes:
- add runtime probes only;
- add one deliberately complex cross-feature program.

Must not:
- skip semantic preflight because VM output looks correct;
- modify compiler/VM in this checkpoint without entering INCIDENT diagnosis.

Current implementation:
Either and Monads independently execute user-level source. Expression has no VM integration fixture.

Target runtime coverage:

1. **Pure/add success**

```text
Expression<Either<String,_>,Int>
→ eval with StringEitherMonad
→ Right(42)
```

2. **Higher-order map/apply**

An expression containing a function value and/or `Map` evaluates to the exact expected result.

3. **Lift success**

`Lift(Right(...))` flows through unchanged effect semantics.

4. **Lift failure / short-circuit**

Use an explicitly typed failed effect to avoid making runtime success depend on an unrelated underconstrained-Left test:

```phalcom
let failed: Either<String, Int> = Either::Left("boom")
let expression: Expression<<X> =>> Either<String, X>, Int> = Expression::Lift(failed)
```

Evaluation returns `Left("boom")`, and a continuation-side counter proves the continuation is not executed where applicable.

5. **Flagship integration**

Prefer the existing `MonadAlgorithms.traverse` over the `ExpressionMonad` constructor:

```text
List<Int>
    |
    | MonadAlgorithms.traverse(
    |   StringEitherExpressionMonad,
    |   values,
    |   Int -> Expression<Either<String,_>, B>
    | )
    v
Expression<
    <X> =>> Either<String,X>,
    List<B>
>
    |
    | ExpressionEvaluation.eval(StringEitherMonad, ...)
    v
Either<String,List<B>>
```

This scenario should have both:
- a semantic Rust test asserting the complete intermediate and final canonical types / constructor solutions;
- a VM observation asserting the final successful list contents (and, if practical, a failure variant that preserves short-circuit semantics).

Do not start with the flagship scenario before the simpler Expression tests pass; its purpose is final composition evidence, not debugging primitive syntax.

Code instructions:

STRUCTURAL:
Reuse the existing semantic-preflight/runtime harness discipline. Runtime probe export names must be Expression-specific (`expressionRuntimeAll`, etc.) to avoid ambiguity even though source assemblers are isolated.

Testing classification:
- Mandatory cross-layer integration evidence.

Checkpoint state update:
Record:
- semantic preflight command/result;
- runtime test command/result;
- exact intermediate `Expression<...,List<B>>` type;
- exact final `Either<String,List<B>>` type;
- short-circuit observations.

---

# 15. Checkpoint C5 — Migration Closure and Delivery Gates

Tasks:
- Task 16 — remove obsolete live paths/authorities and update current references
- Task 17 — run focused/core/package final compatibility gates
- Task 18 — run workspace delivery gates and close the state ledger

Why this is a checkpoint:
C5 does not establish new language semantics. It proves the migration is complete, no obsolete authority can silently execute, and the changed test organization is compatible with the rest of the workspace.

Entry conditions:
- C4 COMPLETE;
- no active incident;
- all focused semantic/runtime evidence green.

Working set:

Primary:
- live `phalcom-core/tests/core/typing_integration/**`
- `phalcom-core/tests/core/mod.rs`
- current package docs
- implementation state file

Secondary — inspect only if evidence requires it:
- repository-wide compile/test failures outside this package for classification

Out of scope for this checkpoint:
- opportunistic cleanup unrelated to this package;
- historical-doc rewrites.

Semantic contract established by this checkpoint:
- old live package authorities are gone;
- one top-level test package owns all three sub-suites;
- all deferred compatibility/delivery gates are discharged;
- no unresolved implementation-state incident remains.

Semantic risks:
- stale old directory/module still compiles unnoticed;
- current docs point to obsolete law catalog;
- workspace gate exposes unrelated pre-existing failures and implementer misattributes them.

Hostile cases:
- `rg` finds a second live `Either` declaration;
- old top-level path registrations remain;
- both child support files remain;
- a broad gate fails in a subsystem untouched by the program: classify before patching.

Required evidence:
1. final negative/deletion gates in Section 19.
2. focused full package:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration:: -- --nocapture
```

3. full `phalcom-core` core integration target:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core
```

4. final broad gates in Section 18.

Do not run yet:
- nothing remains deferred after C5.

Escalate immediately if:
- deletion searches find an unexplained live duplicate;
- workspace failures are outside the planned boundary and cannot be shown to predate/ignore this program;
- clippy/format failures require unrelated code edits.

Checkpoint completion:
- [ ] Tasks 16–18 implemented
- [ ] deletion gates pass
- [ ] full typing integration package passes
- [ ] full core integration target passes
- [ ] format/check/workspace test/clippy gates pass or documented release blocker exists
- [ ] deferred-evidence audit empty
- [ ] state file final
- [ ] no active incident remains

Suggested commit grouping:

```text
docs(test): finalize typing integration migration
```

If no doc/code edits remain after evidence, C5 may be evidence-only and require no additional commit.

## Task 16 — Remove Obsolete Live Paths and Update Current References

Purpose:
Guarantee the migration replacement is exclusive, not merely preferred.

Risk:
- Semantic: LOW
- Implementation fanout: multi-file

Owned files and symbols:
- old `phalcom-core/tests/core/either` path
- old `phalcom-core/tests/core/monads` path
- `core/mod.rs`
- current `typing_integration` docs

Inspect before editing:
- scoped searches in Section 19.

Do not inspect unless evidence forces expansion:
- historical docs under completed plans/logs.

Dependencies:
- all prior checkpoints

Source of truth:
- live `typing_integration` package

Implementation boundary:

Changes:
- remove only genuine obsolete live remnants;
- update current documentation references.

Must not:
- rewrite historical evidence just to satisfy a broad text search.

Current implementation:
Old paths should already have been moved/deleted in C1/C2, but this task verifies closure.

Target implementation:
Only the new package is executable/current.

Edit operations:

1. Run all final negative searches.
2. Delete/repair unexplained live remnants.
3. For repository-wide old-path hits, classify each as:
   - historical evidence intentionally retained;
   - current stale reference that must be updated.
4. Update only current stale references.

Code instructions:

STRUCTURAL:
No semantic code changes are expected.

Testing classification:
- deletion/negative-search gate at C5.

Checkpoint state update:
Record every intentional historical old-path occurrence category, not necessarily every line.

## Task 17 — Run Package and Core Compatibility Gates

Purpose:
Prove the unified module registration and shared fixture source do not disturb other `phalcom-core` integration tests.

Risk:
- Semantic: MEDIUM
- Implementation fanout: package-wide evidence

Owned files and symbols:
- none unless a failure is diagnosed within planned scope

Inspect before editing:
- no new files; run tests first.

Do not inspect unless evidence forces expansion:
- unrelated production subsystem.

Dependencies:
- Task 16 deletion gates

Source of truth:
- Cargo test results

Implementation boundary:

Changes:
- evidence only unless a scoped migration defect is reproduced.

Must not:
- patch outward immediately on unrelated failure.

Edit operations:

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core typing_integration:: -- --nocapture
RUSTFLAGS='' cargo test -p phalcom-core --test core
```

First command proves:
- unified package semantic/runtime tests all execute under one namespace.

Second command proves:
- the `core` integration root remains compatible with all sibling modules after registration changes.

It does **not** independently prove the detailed GEX/MON/GEN invariants; those were established by focused checkpoint evidence.

Testing classification:
- Required C5 compatibility evidence.

Checkpoint state update:
Record exact command/result/counts.

## Task 18 — Run Workspace Delivery Gates and Finalize State

Purpose:
Discharge all broad deferred evidence and leave a reviewable completion record.

Risk:
- Semantic: LOW
- Implementation fanout: workspace-wide evidence

Owned files and symbols:
- implementation state ledger

Inspect before editing:
- no code unless a gate fails and Failure Protocol identifies an in-scope defect.

Dependencies:
- Task 17 green

Source of truth:
- actual delivery command outputs

Implementation boundary:

Changes:
- final state ledger only unless a scoped defect exists.

Must not:
- claim success for a skipped/failed broad gate;
- convert unrelated baseline failures into unplanned cleanup.

Edit operations:
Run final broad gates from Section 18, record each result, perform deferred-evidence audit, mark all checkpoints final.

Code instructions:

STRUCTURAL:
No new semantic implementation is expected.

Testing classification:
- Final delivery gate.

Checkpoint state update:
Record final evidence table, no unresolved incident, and roadmap next action if further complex typing tests are planned.

---

# 16. Expression Fixture Design Notes for the Implementing Agent

This section constrains C4 without attempting to turn the plan into the later full Expression technical specification.

## 16.1 Why `Expression<F,T>` rather than only `Expr<T>`

A simple GADT evaluator already exists in the semantic ownership suite. The integration value comes from making `F` independently higher-kinded:

```text
Expression<F,T>
    T = statically proven value/index
    F = effect constructor used by evaluation
```

Then:

```text
eval : Monad<F> × Expression<F,T> -> F<T>
```

forces equality evidence about `T` to survive application under `F`.

## 16.2 Why Expression itself also has a Monad instance

The unary constructor:

```text
<X> =>> Expression<F,X>
```

captures the outer `F` while binding `X`. Specializing this through the existing Monad hierarchy tests a different captured type-lambda structure than the current:

```text
<X> =>> Either<E,X>
```

Both should remain in the suite because they fail for different substitution/capture bugs.

## 16.3 Keep two Monads conceptually distinct

In a concrete `Either` scenario:

```text
outer evaluation effect Monad:
    Monad<<X> =>> Either<String,X>>

expression-construction Monad:
    Monad<<X> =>> Expression<<Y> =>> Either<String,Y>, X>>
```

The test should intentionally exercise both. Do not name them both `monad` in Rust assertions where confusion is likely.

## 16.4 Contextual typing for failed `Either::Left`

A left-only constructor does not intrinsically determine the right type. For runtime tests, avoid accidentally turning an unrelated underconstraint challenge into the main test by writing:

```phalcom
let failed: Either<String, Int> = Either::Left("boom")
```

before `Expression::Lift(failed)`.

A separate semantic probe may later test deeper contextual inference through `Lift`, but it is not required for the first runtime vertical proof.

## 16.5 Do not duplicate GADT proof ownership

The core integration test should observe exact outcomes such as:

```text
Expression<Either<String,_>,Int>
    -> Either<String,Int>

Expression<Either<String,_>,Bool>
    -> Either<String,Bool>
```

and ensure evaluator branch expressions are Ready. Low-level assertions that a match arm's proof map literally stores `T = Int` already belong in `phalcom-semantic/tests/semantic/adts/matching/gadt_refinement.rs`.

---

# 17. Repository Drift Protocol

Before each checkpoint:

1. verify every primary file still exists;
2. verify the primary symbol still has the responsibility stated in this plan;
3. inspect changes from earlier checkpoints;
4. search for newly introduced callers if a shared helper signature is about to change;
5. adapt path/import mechanics only as necessary.

Do not redo the full repository investigation unless:

- a primary symbol no longer exists;
- test source ownership changed materially;
- a production subsystem already introduced a shared integration fixture that supersedes this plan;
- an expected test behavior contradicts the inspected law catalog/spec.

Allowed mechanical adaptation example:

```text
shared helper function changed from taking &str to Arc<str> upstream;
adapt builder call sites.
```

Not allowed semantic adaptation example:

```text
constructor inference became inconvenient;
replace exact constructor identity with name equality or Dynamic.
```

If the semantic design is contradicted, mark the current checkpoint `INCIDENT` and report evidence.

---

# 18. Final Broad Gates

Run only after C5 focused/core evidence is green.

The repository pins a nightly toolchain, so normally use the repository-selected toolchain rather than forcing stable for compilation. Formatting is configured to use stable-compatible rustfmt options.

## 18.1 Format

```bash
cargo fmt --all -- --check
```

Proves:
- Rust edits satisfy repository formatting expectations.

Does not prove:
- typing semantics.

## 18.2 Workspace check

```bash
cargo check --workspace --all-targets
```

Proves:
- Rust module/API migration compiles across all workspace targets.

Does not prove:
- runtime or inference behavior.

## 18.3 Workspace tests

```bash
cargo test --workspace --all-targets
```

Proves:
- no broad test-target regression across the workspace after the migration and new integration fixtures.

Does not replace:
- the focused checkpoint evidence that proves GEN/MON/GEX/INT invariants.

## 18.4 Clippy

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Proves:
- changed Rust code introduces no clippy warnings under workspace policy.

If repository/environment convention requires `RUSTFLAGS=''` for a specific broad command due local compiler-wrapper behavior, record that exact invocation in the state ledger rather than silently changing the command.

---

# 19. Final Negative / Deletion Gates

Run from repository root.

## 19.1 One live canonical `Either`

```bash
rg -n '^\s*enum Either<L, R>' phalcom-core/tests/core/typing_integration
```

Expected:

```text
exactly one hit:
phalcom-core/tests/core/typing_integration/sources/either.ph
```

Any second live hit is a release blocker unless explicitly justified by a newly approved test requiring a deliberately distinct homonym (none is planned here).

## 19.2 One shared support authority

```bash
find phalcom-core/tests/core/typing_integration -name support.rs -print
```

Expected:

```text
phalcom-core/tests/core/typing_integration/support.rs
```

only.

## 19.3 Old top-level core registrations absent

```bash
rg -n '#\[path = "(either|monads)/mod\.rs"\]|^mod (either|monads);$' \
  phalcom-core/tests/core/mod.rs
```

Expected: zero hits.

Note: `typing_integration/mod.rs` is expected to contain nested `mod either;` / `mod monads;`; the gate is intentionally scoped to `core/mod.rs`.

## 19.4 Old directories absent

```bash
test ! -d phalcom-core/tests/core/either
test ! -d phalcom-core/tests/core/monads
```

Expected: both succeed.

## 19.5 Child source includes no longer own moved fixtures

```bash
rg -n 'include_str!\("(either|monads|semantic_probes|runtime_probes)\.ph"\)' \
  phalcom-core/tests/core/typing_integration/either \
  phalcom-core/tests/core/typing_integration/monads
```

Expected: zero hits. Shared source inclusion belongs in root `support.rs`.

## 19.6 Current documentation path audit

```bash
rg -n 'phalcom-core/tests/core/(either|monads)' \
  phalcom-core/tests/core/typing_integration
```

Expected: zero stale current-package ownership paths.

Repository-wide old-path hits under `docs/work/...` or completed implementation records may intentionally remain as historical evidence. Classify them; do not delete history.

## 19.7 No temporary compatibility adapters

Search for migration-only names introduced during execution, if any, such as:

```text
legacy_either_source
old_monads_source
compat_support
```

Expected: no production/live test use at final delivery. If a compatibility helper is intentionally retained, list its exact reason and removal condition in the state file.

---

# 20. Failure Protocol

When required evidence fails, stop scope expansion. The checkpoint becomes:

```text
C<N> — INCIDENT
```

not “mostly complete.”

Before repair, record:

## 20.1 Exact reproduction

```text
command
failing test/check
important output/assertion
```

## 20.2 Direct path

Examples for this program:

```text
Rust fixture
→ source builder
→ parse
→ analyze_single_module
→ CallableAnalysis / TypeStore assertion
```

or:

```text
runtime probe source
→ semantic preflight
→ ProgramCompiler
→ VM
→ exported slot assertion
```

## 20.3 Passing comparator

Examples:

```text
Either direct map passes, Monad bind fails;

basic Expr<T> ownership-layer GADT evaluator passes,
Expression<F,T> evaluator fails;

Expression semantic test passes,
runtime lowering fails.
```

## 20.4 Classify

Use exactly one initial classification:

```text
PRODUCT
    production semantics/runtime is wrong

FIXTURE
    source builder/probe/test did not establish the intended precondition

DEPENDENCY/PUBLICATION
    correct semantic product exists but is unavailable/stale to a consumer

BACKEND/HARNESS
    test helper/compiler/VM harness fails outside the intended semantic claim

BASELINE
    failure predates this checkpoint/program

PLAN DRIFT
    repository architecture now contradicts this plan
```

## 20.5 Narrow repair boundary

State the specific file/symbol allowed to change before editing.

For C1/C2 migration defects this should normally stay inside:

```text
phalcom-core/tests/core/typing_integration/**
phalcom-core/tests/core/mod.rs
```

For C4, if diagnosis points into `phalcom-semantic` or compiler/VM, **do not automatically edit it**. Produce a short incident note with evidence and obtain/derive a separate bounded remediation decision before crossing the production boundary.

## 20.6 Rejected broad fixes

Never repair by:

```text
restoring duplicate Either source;
weakening exact ID assertions to names;
turning a conflict into Dynamic;
disabling a hostile test;
skipping semantic preflight for VM tests;
adding parser syntax not already ratified;
creating an Expression-specific Monad algorithm copy;
special-casing test class names in production code.
```

---

# 21. Working-State File Protocol

Use:

```text
docs/impl/semantic/typing-integration/typing-integration-implementation-state.md
```

After every checkpoint, keep only reviewable facts, evidence, and decisions.

Required structure:

```md
# Typing Integration Implementation State

## Baseline
- Plan baseline: `9f046812...`
- Execution branch: ...
- Execution HEAD at start: ...
- Working-tree note: ...

## Established invariants
- I-01: ...
- I-02: ...

## Decisions
- D-01: ...

## Evidence ledger

| Checkpoint | Command | Result | Proves |
|---|---|---|---|
| C0 | `...` | PASS/FAIL | ... |

## Negative/deletion evidence
- command → result

## Deferred gates
- command → C5

## Active incident
None.

## Next resume action
Begin C<N> Task <M>.
```

Do not store private scratch reasoning or a verbose diary. Record only:

- facts;
- decisions;
- code anchors;
- results;
- rejected approaches that matter to later work.

---

# 22. Checkpoint Completion Report Template

At the end of each checkpoint, the implementing agent should produce:

```text
Checkpoint C<N> COMPLETE

Established:
    <dominant semantic contract>

Changed:
    <paths / key symbols>

Evidence:
    <command> — PASS
    <command> — PASS

Hostile cases:
    <case> — PASS

Negative gates:
    <search> — expected result

Deferred:
    <broad gate> → C5

Unexpected findings:
    none | concise finding

Next:
    C<N+1> — <name>
```

If required evidence is red, report `INCIDENT`, not COMPLETE.

---

# 23. Checkpoint Evidence Summary

This table is a plan-time ledger. Status is intentionally `PENDING`; no implementation test was executed by this planning session.

| Checkpoint | Semantic contract | Required evidence | Status |
|---|---|---|---|
| C0 | local baseline is known | Git state + old Either + old Monad suites | PENDING |
| C1 | one Rust package namespace, no semantics changed | moved Either + moved Monad suites with count parity | PENDING |
| C2 | one canonical Either + one shared support authority | hostile negatives + both complete sub-suites + deletion searches | PENDING |
| C3 | explicit package-level cross-feature contract | unified integration test + full package | PENDING |
| C4 | effectful Expression GADT composes GADT/HKT/HOF/Monad/Either and executes | semantic GADT dependency suite + GEX positives/negatives + runtime + flagship integration | PENDING |
| C5 | migration/delivery complete | deletion gates + core target + format/check/workspace tests/clippy | PENDING |

No row may be changed to COMPLETE without recording the actual required command evidence in the state file.

---

# 24. Deferred-Evidence Audit

At C5, inspect the state ledger and require:

```text
No deferred command remains unless it is:
1. executed successfully;
2. explicitly removed from scope with a concrete reason;
3. recorded as a known release blocker with an active INCIDENT.
```

In particular, confirm that these planned deferred gates are accounted for:

```text
full phalcom-core core integration target
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

---

# 25. Staged Commit Groups

Recommended grouping:

```text
C1
test(core): nest Either and monads under typing integration

C2
test(core): share typing integration fixtures and Either source

C3
test(core): establish unified typing integration contract

C4
test(core): add effectful GADT expression typing integration

C5
docs(test): finalize typing integration migration
```

Do not force one commit per patch-grade task. A checkpoint may contain multiple tasks because its evidence only becomes meaningful after their integration.

If C4 is large enough for review clarity, split it after a green internal semantic boundary:

```text
test(core): add typed Expression GADT semantic integration
test(core): execute typed Expression over Either Monad
```

but keep the checkpoint incomplete until both parts satisfy its required evidence.

---

# 26. Known Scope Exclusions

This program deliberately does **not** include:

- production generic inference changes;
- production GADT proof-engine changes;
- parser/AST grammar changes;
- new public syntax for exact-case types;
- rank-2 polymorphism;
- Yoneda / van Laarhoven encodings;
- Free Monad;
- `EitherT` / Monad transformers;
- recursion schemes / `Fix<F>`;
- natural transformation package;
- F-bounds / typestate package;
- structural record-row tests beyond whatever future integration may consume;
- category-theoretic proof of Functor/Applicative/Monad laws;
- LSP/editor testing;
- incremental-analysis changes or cold/incremental parity tests (this program changes test fixtures, not semantic query publication);
- rewriting completed historical planning/log artifacts to new paths;
- moving generic semantic ownership tests out of `phalcom-semantic`.

If the Expression fixture exposes a production defect, fixing it is not silently absorbed into this scope. Use the Failure Protocol and create/approve a bounded remediation slice.

---

# 27. Release-Complete Criteria

The implementation is complete only when all of the following are true:

- [ ] C0 through C5 are `COMPLETE`;
- [ ] local branch/HEAD/working-tree baseline was recorded before edits;
- [ ] every existing Either test was preserved or intentionally superseded with documented equivalent/stronger evidence;
- [ ] every existing MON law remains mapped to a live test and retains its ID;
- [ ] `sources/either.ph` is the only live `Either<L,R>` declaration in the package;
- [ ] `support.rs` is the only live fixture support authority in the package;
- [ ] Monads consume the same canonical full Either used by direct Either tests;
- [ ] old top-level `either` and `monads` registrations/directories are absent;
- [ ] all existing hostile Either/Monad no-Dynamic cases pass;
- [ ] unified cross-feature direct-Either + Monad test passes with exact identity/provenance assertions;
- [ ] Expression GADT positive tests pass;
- [ ] Expression hostile GADT/HKT/HOF tests reject without Dynamic escape;
- [ ] ExpressionMonad type-lambda/capture specialization passes;
- [ ] concrete `Expression<Either<String,_>,T> -> Either<String,T>` semantic evidence passes;
- [ ] Expression runtime success and failure/short-circuit probes pass after semantic preflight;
- [ ] flagship existing-`MonadAlgorithms` + Expression + Either scenario passes;
- [ ] final negative/deletion searches pass;
- [ ] full `phalcom-core --test core` gate passes;
- [ ] `cargo fmt --all -- --check` passes;
- [ ] `cargo check --workspace --all-targets` passes;
- [ ] `cargo test --workspace --all-targets` passes;
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes;
- [ ] no deferred evidence is forgotten;
- [ ] no unresolved `INCIDENT` remains in the state file;
- [ ] current README/LAWS/walkthrough paths describe the unified package accurately.

---

# 28. Final Implementation Guidance

The intended layering after this program is:

```text
FOUNDATION
    Either<L,R>
    Box<T>

        ↓

ABSTRACTIONS
    Functor<F>
    Applicative<F>
    Monad<F>
    MonadAlgorithms

        ↓

CONCRETE HKT SPECIALIZATION
    EitherMonad<E>
    StringEitherMonad

        ↓

GADT INTEGRATION
    Expression<F,T>
    ExpressionEvaluation
    ExpressionMonad<F>

        ↓

CROSS-FEATURE PROOF
    Expression<<X> =>> Either<String,X>,T>
        ↓ eval
    Either<String,T>

    MonadAlgorithms.traverse / kleisli / bind
        over <X> =>> Expression<F,X>
```

The lower layers stay individually testable. The upper layers intentionally compose them. That is the core architectural value of the unified package: when a brutal integration scenario fails, focused lower-level evidence shows whether the fault is in ordinary generic substitution, HKT/type-lambda specialization, GADT refinement, higher-order inference, or cross-layer execution.

The program should leave `typing_integration` as Phalcom's expandable “typing torture suite” without turning it into a monolith or a parallel type checker.
