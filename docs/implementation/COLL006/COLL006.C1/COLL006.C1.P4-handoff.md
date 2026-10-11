---
id: COLL006.C1.P4
program: COLL006
checkpoint: COLL006.C1
kind: handoff
status: COMPLETE
completion: IMPLEMENTED
verification: BASELINE_BLOCKED
---

# COLL006.C1.P4 — Completed handoff to P5

P4 is `COMPLETE / IMPLEMENTED / BASELINE_BLOCKED` on
`feat/telos-collection-upgrade` in the `telos-collections-upgrade` worktree.
The implementation started from committed `d4fe4473` (T10/G6) and continued
inline through T11/G7/T12. The final continuation is in this worktree; use the
live branch revision as the next P5 starting anchor.

## Stable T10 behavior

- A bare runtime bracket literal defaults to `Array<T>`.
- A bare static/`const` bracket literal defaults to exact-extent `Vector<T,N>`.
- Explicit heads and unique contextual family selection retain precedence over
  the phase default. Explicit List use remains supported.
- A static empty literal has known extent zero but does not invent element type
  evidence; underconstrained `T` is diagnosed and no value is published.
- The checker sets `EvaluationPhase::Static` while checking a `const`
  initializer, then restores the caller phase. This keeps ordinary module
  binding facts aligned with the canonical static-constant product. Runtime
  `let`/`let mut` inherits the enclosing phase.
- Headless and explicit Vector literals fail closed in the context-free
  compiler without semantic lowering. Syntax-explicit `<T>[...]` Array remains
  the supported compatibility path.
- Current Phalcom specs do not permit selector overloads distinguished by
  parameter/result types. CV-25 is covered by ambiguous trait dispatch; the
  Array phase default does not choose a candidate. Do not add an overload
  system as part of this cutover.

## G6 evidence

All required G6 commands passed:

| Command/filter | Result |
|---|---|
| `cargo test -p phalcom-semantic --test semantic coll006` | 49 passed |
| `cargo test -p phalcom-core --test core core_array` | 11 passed |
| `cargo test -p phalcom-core --test core core_vector` | 8 passed |
| `cargo test -p phalcom-core --test core static_const` | 23 passed |

The supplementary semantic `static_constants` filter had 21 passes and the
same three D4 baseline failures: `test_callable_signature_projects_all_parameter_and_return_record_keys`,
`test_field_signature_projects_a_cold_computed_record_key`, and
`test_field_signature_key_edit_matches_cold_surface_and_invalidates_exactly`.
Each reports `RecordKeyNotStatic` for computed Record keys. The bare static
Vector, empty underconstraint, static List, and arbitrary-constructor boundary
cases in that filter passed. Keep G2 `BASELINE_BLOCKED`; do not repair D4 under
P4.

## Stable P5 takeover interfaces

- `StaticValueData::Sequence` in `phalcom-semantic/src/static_value/store.rs`
  retains the checked Array/Vector result type and ordered children. Its type,
  semantic equality/hash, and content fingerprint are canonical C4 products.
- `phalcom-semantic/src/static_eval/machine.rs` consumes checked literal
  resolutions; an open symbolic Vector identity spread remains symbolic and
  publishes no partial sequence.
- `MaterializationDescriptor::Sequence` is self-contained. The VM uses
  `RuntimeStaticConstantRef` and the canonical owner module cache to build
  immutable native Vector objects or frozen ordinary Array objects without
  runtime initializer execution. Intermediate heap children are temp-rooted
  until owner publication; failures publish no partial value.
- Direct const Array mutation has a semantic diagnostic, and the frozen runtime
  guard protects aliases. Ordinary Array element replacement remains available.
- P5 may rely on bare runtime Array/static Vector defaults and explicit List
  contexts. Parser/compiler do not independently select the family.

## T11/G7 evidence and limits

- Semantic `coll006`: 51 passed; `static_sequence`: 10 passed; exact COLL006
  incremental cases: 3 passed. Content/extent/phase/spread edits match cold
  analysis, invalidate current values/descriptors/fingerprints correctly, and
  reuse unrelated products.
- Core Array: 11 passed; Vector: 9 passed (including exactly-once source order
  and zero extent); static constants: 23 passed; module parity: 8 passed.
  Module tests cover selective/renamed/qualified/two-hop re-export reads,
  canonical owner identity, forced GC before reads, and cache rooting.
- Native surface `--check`: 417 declarations current. Workspace all-target
  `cargo check` and `git diff --check` passed.
- Semantic `static_constants`: 21 passed, three inherited D4 computed Record-key
  `RecordKeyNotStatic` failures; G7 is `BASELINE_BLOCKED`, not a clean pass.
  The initial broad `incremental` filter also exposed unrelated failures and
  timed out; the exact COLL006 incremental gate passed. Global `cargo fmt
  --all -- --check` reports inherited formatting drift in pre-existing files.
- The refreshed migration census records 505 classified bare nodes, zero ambiguous
  or unclassified; P4 introduces no outstanding List migration.

GC during recursive materialization is protected by the existing VM temp-root
seam and verified by nested post-construction collection and failed-publication
tests; there is no exposed test hook for forcing GC between individual
descriptor-child allocations. Controlled allocator failure also has no focused
injection hook; the implementation uses fallible reservations and errors before
owner-cache publication. No artificial global extent limit was added.

Next: implement P5 Slice/Buffer contiguous views and bounded storage, including
the LANG007.C4.P14 parser-snapshot convergence. Preserve P4's canonical static
value/owner model and do not re-derive collection family or static identity.

## P4 correctness follow-up closure — 2026-10-10

The bounded correctness follow-up is complete; P4 remains
`COMPLETE / IMPLEMENTED / BASELINE_BLOCKED`. Its key strengthened invariants are:

- An inference relation registered for a spread remains pending until its
  canonical owner solves it. A missing owner fails closed and publishes no
  construction product; concrete subtype checks stay in the canonical relation
  layer.
- Incremental multi-hop static spreads propagate content edits, invalidate
  extent-mismatched dependents, preserve independent products, and agree with
  cold analysis.
- Recursive sequence materialization keeps the existing temporary-root and
  canonical owner-cache boundaries. Failed publication restores root depth;
  nested frozen Arrays and imported/re-exported identity survive GC stress.
- `SequenceConstructionPlan::Fixed` retains host lengths as `usize`, with
  checked compiler conversion. The static evaluator remains the production
  `intern_sequence` caller and supplies committed semantic resolution.

The follow-up's focused gates and the three unchanged D4 static-constant
baseline failures are recorded in the [C1 checkpoint](COLL006.C1-CHECKPOINT.md).
P5 can proceed from the current live branch revision with these invariants.

## Residual ownership and P9 closure ruling

The user-provided consultation is recorded as A11 in the checkpoint and as
§16.7 in the detailed C1 plan. Treat D1/D2/D4 as verification residuals, not
P5 execution blockers:

| Residual | Current owner/classification | P5 effect | P9 action if it remains |
|---|---|---|---|
| D1 — `NonExportableRecordKeyDomain` in metadata fingerprint coverage | Semantic metadata / Record-key exportability; suspected unrelated, not proven external | None | Rerun the exact fingerprint fixture and compare with current `origin/main` |
| D2 — reflection, Universe exposure, and module/package/project fixture failures | Split by the actual reflection or fixture cause; suspected unrelated, not proven external | None | Rerun and classify each distinct cause against current `origin/main` |
| D4 — three computed-Record-key `RecordKeyNotStatic` cases | Static Record-key projection / static-constant semantic metadata; inherited non-P4 sequence residual | Prevents a clean aggregate `static_constants` PASS only | Rerun all three exact fixtures and compare with current `origin/main` |

At P9, current-main reproduction of the same failure is sufficient to classify
it `EXTERNAL_BASELINE`; do historical boundary comparison only when main passes
and the C1 candidate fails. Classify each residual as `RESOLVED`,
`EXTERNAL_BASELINE`, `C1_OWNED`, or `AMBIGUOUS`, splitting D2 when causes differ.
C1 closure requires no `AMBIGUOUS` residuals. Confirmed external failures may
leave C1 `COMPLETE / IMPLEMENTED / BASELINE_BLOCKED`; a C1-owned defect returns
to its owning plan or a corrective C1 plan. Do not add a P4 repair plan without
evidence of P4/C1 causality.

P9 also owns broad workspace and current-CI certification plus C1-specific
behavior, property, GC/rooting, re-entry, cold/incremental, performance,
migration, documentation, and lifecycle gates. The detailed plan lists the
current repository commands; re-read `scripts/verify.sh` and
`.github/workflows/ci.yml` when P9 begins because those gates may evolve.

## Records to read

- [P4 implementation plan](COLL006.C1.P4-static-homogeneous-sequences-and-atomic-bracket-cutover-luna-patch-grade-implementation-plan.md)
- [C1 checkpoint](COLL006.C1-CHECKPOINT.md)
- [G5 migration census](COLL006.C1.P4-bracket-migration-census.md)
