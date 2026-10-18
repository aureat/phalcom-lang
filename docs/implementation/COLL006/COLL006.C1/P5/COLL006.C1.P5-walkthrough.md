---
id: COLL006.C1.P5
program: COLL006
checkpoint: COLL006.C1
kind: walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# COLL006.C1.P5 — Slice, Buffer, and parser snapshot convergence

## Implementation and scope

P5 introduces canonical unary Collection families Slice and Buffer, with typed
source APIs, native storage, reflection, tracing, iteration, equality, range
views, and Map/Set-key rejection. Slice stores a strong normalized root owner,
start, and length. Nested views are zero-copy and O(1); live Array/Buffer changes
are visible while later Buffer append does not extend existing views. Buffer
reserves logical capacity once, stores only initialized values, rejects implicit
growth/shrink, and preserves atomic capacity failures. Value/Object remain
16/48 bytes on this host. Native declarations increase from 417 to 429 (+12).

`ListSource<T>` constructs a private Array from the caller List and retains it
through Slice. P14's historical private List copy is accurately reconciled;
no ImmutableSequence runtime type or runtime-sized Vector snapshot is added.
The bulk implementation is committed in `f3403d70`; this audit continuation
started with a clean feature worktree at that revision.

## Audit amendment and acceptance proof

The user supplied and approved disposition 2, splitting G6.A P5-owned acceptance
from G6.B full-package certification. The P5 plan, checkpoint, historical
consultation resolution, P9 obligations, and parse docs now reflect that ruling.

`core/list-source-snapshot-test.ph` imports no TextSource or parse.text module.
It tests empty/singleton/populated sources, separately observable append,
replacement, and middle-removal isolation, Cursor boundaries/offsets, satisfy
and select success/rejection, prefix parsing, ordered complete input, and
incomplete/trailing input. A factory-local List expires before `System.gc`;
its heap-backed ADT token remains readable and parseable through the source.

The selectable Rust fixture test requires normal exit and the exact marker
`PASS core-list-source-snapshot`. The native owner test invokes the real
ListSource constructor with two freshly allocated Strings. It inspects the
single Slice field and Array owner, roots only the source, proves the caller
List is collected while Slice/Array/elements survive forced GC, and executes
real Cursor and Parse.runAll after collection. This establishes the complete
source graph rather than relying solely on generic Slice tracing tests.

The audit continuation also adds Buffer reservation reuse coverage required by
P5's remaining performance checklist. Snapshotting preserves references, not
recursive deep copies. That continuation was committed and pushed as
`dea2cd1d`; the audit follow-up below is a later, user-directed correction.

**Current final state (2026-10-18):** P5 is
`COMPLETE / IMPLEMENTED / FOCUSED_TESTED`. G6.A, G6.B, and aggregate G6 pass.
The complete parse-package gate passes 4/4 Rust integration checks and all 13
runtime fixtures. This current state supersedes the historical blocked result
recorded below.

## Initial P5 T9 verification snapshot — 2026-10-11

All Cargo commands used `RUSTFLAGS=''` and the pinned toolchain.

| Command/filter | Result |
|---|---|
| `cargo run -p phalcom-core --bin phalcom -- packages/parse/tests/runtime/src/core/list-source-snapshot-test.ph` | normal exit, exact PASS marker |
| core `parse_list_source_snapshot` | 2 passed |
| core lib `heap::buffer::tests` | 5 passed (includes reservation reuse) |
| core lib `heap::slice::tests` | 5 passed |
| core `slice` | 9 passed (includes P5 range/mutation/GC/identity evidence) |
| core `core_buffer` | 1 passed |
| core `core_array` | 11 passed |
| core `core_vector` | 9 passed, including stable Map/Set keys |
| core `static_const` | 24 passed, including frozen Array controls |
| semantic `coll006` | 54 passed |
| semantic `static_sequence` | 10 passed |
| native meta `slice_and_buffer` | 1 passed |
| native surface generator `--root . --check` | 429 declarations current |
| `cargo check --workspace --all-targets` | passed |
| scoped rustfmt for both changed Rust files; `git diff --check` | passed |
| core `core_collections` | 11 passed, 1 inherited failure |
| core `modules_parse_package` | 2 passed, 2 failed; G6.B remains blocked |

The package command selects four Rust tests. Both new focused tests pass.
Project compilation fails at TextSource.slice's String/Substring mismatch.
The runtime corpus has 12 fixtures: the new fixture passes; 10 inherited text
fixtures fail at the return mismatch and the operator fixture fails comparing
Some with Int. The audit establishes the byte API returns Option<Int>.

`map_key_overwrite_and_remove_idempotence` fails its absent-key None assertion
identically on the untouched pre-P5 worktree at `78cfd920`. This is a D-class
baseline (`R-MAP-REMOVE`), not a P5 repair. Workspace rustfmt reports inherited
drift in unchanged files, including semantic tests; scoped changed files pass.
The recorded P4 D1/D2/D4 failures are inherited, not re-certified here.

Fixture bring-up corrected trailing-closure placement, the explicit Cursor
offset, and value-position `and` expressions (a documented existing limitation).
No collection or parser production contract was changed to make tests pass.

## Lifecycle and remaining integration

T7 is IMPLEMENTED / FOCUSED_TESTED and G6.A is PASS. T9's P5-owned stabilization
is complete. P5 is COMPLETE / IMPLEMENTED / BASELINE_BLOCKED; G6.B and aggregate
G6 remain BASELINE_BLOCKED. C1 remains IN_PROGRESS / PARTIAL / BASELINE_BLOCKED.

At the initial P5 T9 closeout, R-PARSE-TEXT and R-PARSE-BYTES were assigned to
the bounded parser-migration follow-up recommended by the audit
(LANG007.C4.P16), and their main runtime reproduction had not been verified.
The user-directed follow-up below later corrected both adapter issues. P9 must
still rerun the complete package gate after the associated-invocation owner
decision and compare any surviving residuals against contemporaneous main.
Full parser certification requires a clean package result.

Full owning-crate/workspace tests, Clippy, release/performance certification,
and unrelated baseline repairs are deferred to their named owners/P9. P6
inherits the stable interfaces in the [handoff](COLL006.C1.P5-handoff.md).

## User-directed audit follow-up — 2026-10-11

On the clean P6 T1 revision `80a889f0`, the user requested that all remaining
items in the supplied P5 implementation audit be addressed. The parser adapter
defects were local contract mismatches, so the follow-up now:

- materializes `String.slice`'s `Substring` in `TextSource.slice`, preserving
  the declared `String` result;
- handles `Option<Int>` explicitly in `TextSource.at`, `lineColAt`, the
  `Text` combinators, and the operator fixture (using `-1` for absent bytes);
- materializes the operator fixture's byte-sized slice before String
  concatenation;
- corrects `map_key_overwrite_and_remove_idempotence` to match the existing
  public `Map.remove(_)` wrapper, which returns the Map for both absent and
  present keys while still removing a present association.

Verification on this follow-up:

| Command/filter | Result |
|---|---|
| `RUSTFLAGS='' cargo test -p phalcom-core --test core core_collections` | 13 passed, including `map_key_overwrite_and_remove_idempotence` |
| `RUSTFLAGS='' cargo test -p phalcom-core --test core modules_parse_package` | 4 Rust tests selected: 3 passed, 1 failed; 11 of 12 runtime fixtures passed |

The original project compile errors from the String/Substring and
`Option<Int>`/`Int` mismatches are cleared. The remaining package failure is in
`packages/parse/tests/runtime/src/core/source-test.ph`, in the token-backed
Cursor `current`/`advance` assertions involving `Option<Token>`. The compiler
reports `associated family invocation lowering is not implemented until
associated semantic resolution lands` from the associated-invocation lowering
fallback. The exact single expression was not isolated from the paired cursor
assertions. This is an owning compiler-layer blocker, not a P5 collection
representation change; the follow-up consultation is recorded separately.

P5 remains `COMPLETE / IMPLEMENTED / BASELINE_BLOCKED`: G6.A is `PASS`, while
G6.B and aggregate G6 remain blocked by this remaining full-package failure.
The earlier `R-PARSE-TEXT` and `R-PARSE-BYTES` adapter issues are corrected;
P9 must rerun the package gate after the associated-invocation owner work and
complete its residual comparison. Workspace tests, Clippy, and release
certification remain deferred to P9.

## Final package-gate closure — 2026-10-18

The prior status above records the then-current failure and is superseded by
this closeout. The final package gate and its runtime corpus now pass:

| Evidence | Result |
|---|---|
| `RUSTFLAGS='' cargo test -p phalcom-core --test core modules_parse_package -- --nocapture` | 4/4 Rust integration checks passed |
| Runtime fixture corpus selected by the package gate | 13/13 fixtures passed, including Mini-TOML and token-backed Cursor coverage |
| `cargo test -p phalcom-ast --test integration interpolated_string_synthetic_getters_use_outer_range` | 1 passed |
| `RUSTFLAGS='' cargo test -p phalcom-core --test core interpolation_of_generic_data_projections_matches_with_product_optimization_modes -- --nocapture` | Passed |
| `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic generic_immutable_field_accepts_a_call_valued_constructor_write -- --nocapture` | Passed |

The package corpus exercises recursive typed Mini-TOML parser composition,
generic `select<U>`, `Cursor<ListSource<Token>>::current`, exact nested
`Option<Token>` variant construction, and the Array/Slice snapshot path. The
snapshot owner's forced-GC evidence is also covered by the separate G6.A Rust
selector recorded earlier in this walkthrough.

The remaining fixture failure was not evidence of a missing runtime or generic
associated-invocation mechanism. `Some(...)` in the equality expression had no
expected enum owner, so semantic analysis could not resolve its nested
`Token` constructor. Explicit `Option<Token>::Some(...)` supplies that context
and follows the canonical static variant path. Structural matching checks the
variant and payload without requiring fixture-defined `Token` equality. The
semantic-to-compiler associated-lowering contract remains intact. Dynamic
associated-family invocation is a separate, still-open compiler capability and
does not block the static construction path covered here.

`ListSource<T>` retains the P5 snapshot design: caller `List<T>` is copied into
a private `Array<T>`, and the source retains a `Slice<T>` view whose owner
survives collection. No recursive payload cloning is implied. No compiler
fallback, AST-spelling reconstruction, generic erasure, or weakened lowering
invariant was introduced.

No workspace test suite, Clippy gate, or release certification ran for this
closeout. The package is complete at its declared integration gate; verification
is `FOCUSED_TESTED`, not `RELEASE_COMPLETE`.
