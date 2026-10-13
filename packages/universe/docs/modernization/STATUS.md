# Shadow Universe modernization status

## MOD1 inventory baseline

- **Program:** MOD1 — Baseline, Guidelines, and Modernization Inventory
- **Branch:** `feat/telos-collections`
- **HEAD:** `18aaf8b568a0b45dd450befc35932bb7a1fdd442`
- **Source files:** 174 `.ph` files under `packages/universe/src`.
- **Dirty scope at baseline capture:** `phalcom-semantic/src/session.rs` and `phalcom-semantic/src/workspace.rs` only (3 insertions total); preserved untouched. The final status also includes these four task docs and the parent-created P1 plan/checkpoint records as untracked.
- **Baseline classification:** Inventory, guidelines, and language-feature matrix reviewed. MOD1 is **COMPLETE / IMPLEMENTED / FOCUSED_TESTED**. The baseline predates the C1.P2 source pilot recorded below.
- **Parser evidence:** `migration-status.md` records the S4 source validator as `FOCUSED_TESTED` for parsing all then-current shadow sources and building unlinked interfaces. It is prior evidence, not rerun at this HEAD. It does not prove semantic identity, linked semantic correctness, runtime behavior, or production cutover.
- **P1 checks:** filesystem count, per-source SHA-256 capture, targeted source inspection, independent exact inventory-path/hash comparison (174 paths, no differences), and `git diff --check` for the modernization docs. No tests, parser, compiler, semantic, or runtime commands were run for P1.

## Current source reconciliation

Through `UNIV002.C2.P15`, the 174 shadow source paths were reconciled against
the MOD1 baseline at source revision
`88a46e226ea2d899df654b85b8d526aa9b6da92c`. Sixteen source files changed since
baseline; their current SHA-256 values are recorded in `INVENTORY.md`. Other
source hashes still match baseline. Its textual inventory heuristics remain
baseline data and are not a current opportunity audit.

## Active work and exclusions

- S4 moved the iteration protocols and adapters into `collections/iteration/package.ph`; that cohesive move is recorded implemented/focused-tested, while its finer split is deferred and blocked by module/runtime-cycle concerns. Exclude edits to `collections/iteration/**` and any ownership/split work. The proposed `List`/`Array`/`Vector` slice-call-site review does not move those modules, but must preserve their dependency on the cohesive iteration owner.
- Shadow semantic-linking/P4 work is active outside this source-quality inventory. `phalcom-semantic/src/resolver.rs`, `session.rs`, `workspace.rs`, and `tests/semantic/integration/resolver.rs` are concurrently modified, with an untracked P4 handoff. Exclude reflection/typing, semantic-link-dependent source/interface changes, and any change requiring a new linker or canonical shadow identity.
- S5's System/Future separation is explicitly held for an architecture decision. Exclude `concurrency/fiber.ph` and related module-boundary changes.
- Production source providers, native mappings, prelude construction, bootstrap, VM, and legacy source are out of scope.

## Pilot state

`UNIV002.C1.P2` modernized `Chain.fromIterable`, `Chain.partition`,
`List.sliceByRange`, `Array.slice`, and `Vector.slice`. The exact shadow
parse/unlinked-interface test passed, and the scoped diff passed `git diff
--check`. The test does not typecheck method bodies or establish canonical
semantic identity. The pilot is `SOURCE_READY` and `PARSE_VERIFIED`; semantic
verification remains `UNVERIFIED` and shadow runtime parity remains `BLOCKED`
because no complete-shadow semantic or execution path exists.

## Pilot candidate (not an approved edit)

### Candidate A — completed `Result` match at the shared range-slice boundary

The proposed source-only pilot replaces the callback `.match` calls in:

- `collections/list.ph::List.sliceByRange`
- `collections/array.ph::Array.slice`
- `collections/vector.ph::Vector.slice`

All three call `range.rawSliceBounds(self.size)`. `Range.rawSliceBounds` currently constructs `Result::Error(SliceError)` on invalid coordinates and `Result::Ok((start, end))` on success (`collections/range.ph`). The three callers retain the same public methods and module ownership; List performs its existing element-copy loop, while Array and Vector call their existing `rawSlice` methods.

Acceptance invariants for any later P2 edit:

- Evaluate `rawSliceBounds` once and preserve the same `Result` branch selection.
- Preserve successful start/end values and the `start > end` normalization (`end = start`).
- Preserve `SliceError` propagation/raising for the failing branch.
- Preserve each collection's existing raw slicing or copy behavior and evaluation order.
- Keep all method signatures and module paths unchanged.


P2 completed these three source rewrites after documenting that no focused validator links the complete shadow graph. The exact shadow parser/unlinked-interface check passed; it does not typecheck these bodies or prove exact-site exhaustiveness. Live `phalcom-core/src/modules/semantic_lowering.rs::project_match_resolution` rejects match lowering unless exhaustiveness is proven, so semantic verification remains unavailable and runtime parity remains blocked. The rewrites preserve the existing bounds call, branches, and module ownership.

### Candidate B — completed single-iterator `Option` consumption in Chain

`functional/chain.ph::Chain.fromIterable` and `Chain.partition` hold one
iterator, then use an `isSome` condition plus a `match` to extract each item.
Their current structure supports a source-level `while let` candidate while
preserving one `iterator.next` evaluation per attempt. Required invariants are
one iterator construction, one predicate call per item, original item order,
and no change to exhaustion/error behavior. P2 completed these two rewrites as
source-level modernization. The exact parser/interface check passed, but no
linked semantic or runtime evidence is available for these generic functions.

## Data candidates — HIGH / DEFER

| Candidate | Live source/runtime contract observation | Disposition |
|---|---|---|
| `reflection/package-author.ph::PackageAuthor` | Shadow source declares `email -> String` and `url -> Uri`. Live runtime storage is `Option<String>` and `Option<ObjRef>`; native getters return `Option` values. | `HIGH / DEFER`. Contract mismatch blocks data conversion; resolve source/runtime API authority and optionality first. |
| `reflection/package-requirement.ph::PackageRequirement` | Shadow source declares `package -> PackageInfo`; live runtime storage is a package-name `String`, and its native getter returns that string. | `HIGH / DEFER`. Contract mismatch blocks data conversion; determine canonical component meaning and public API before conversion. |

Live evidence: `phalcom-core/src/heap/reflection.rs` (`PackageAuthorObject`, `PackageRequirementObject`), `phalcom-core/src/primitive/reflection.rs` (`package_author_email`, `package_author_url`, `package_requirement_package`), and `phalcom-core/src/modules/reflection_cache.rs` (direct construction). These custom heap objects/cache paths make representation and conversion work native-boundary changes. MOD1 records this blocker only; MOD5 must perform the full constructor, identity, cache, GC, and host-conversion audit.

The inventory excludes `collections/iteration/**` edits/ownership changes and active reflection/typing or semantic-link-dependent work. S5 `concurrency/fiber.ph` source-boundary changes remain held for architecture guidance. P2 changed only the five functions listed above; production, compiler, semantic, and runtime sources remain untouched by UNIV002.

## Required status vocabulary

Use distinct states; do not collapse them into “done”:

`NOT_STARTED`, `IN_PROGRESS`, `SOURCE_READY`, `PARSE_VERIFIED`, `INTERFACE_VERIFIED`, `SEMANTIC_VERIFIED`, `RUNTIME_VERIFIED`, `BLOCKED`, `DEFERRED`.

A source-only change may be `SOURCE_READY`; parser/interface checks do not imply semantic/runtime verification. Use `BLOCKED` only for a specific prerequisite or conflict and `DEFERRED` for an intentionally postponed slice.

## Next action

`UNIV002.C2.P1`–`P4`, `P6`–`P15` completed Category A
documentation/formatting slices. `UNIV002.C2.P5` completed a Category B
source-level Gregorian leap-year expression
rewrite in `time/calendar/month.ph`; its exact shadow parser/unlinked-interface
test passed. This does not typecheck or execute the method, so semantic linking
remains `UNVERIFIED` and shadow runtime parity remains `BLOCKED`. P7 documents
the observed `UtcOffset.seconds` input/range/error contract without changing
code. The current 174 paths are reconciled against the MOD1 inventory baseline
through P15's source commit; sixteen changed-file hashes are recorded in
`INVENTORY.md`, while its textual heuristics remain baseline-only.
P8 documents the maximum element count returned by `Buffer.capacity`; no
executable code changed.
P9 documents that `Bag.count` returns the number of occurrences of the supplied
value; the source wrapper and native lookup remain unchanged.
P10 documents that `Reader.run` runs the deferred computation with the supplied
environment; the method body and signature remain unchanged.
P11 documents that the mapped Reader transforms the produced value and retains
the same environment; the method body and signature remain unchanged.
`UNIV002.C2.P6` documented only the live `ZoneTransition` fields and excludes
unimplemented derived members. Keep source modernization independent of
Option/Result/Unit identity, collection iteration ownership, reflection, and
active linker work. `PackageAuthor` and `PackageRequirement` remain
`HIGH / DEFER` pending the dedicated constructor, identity, cache, GC,
reflection, and host-conversion audit.
Continue with a bounded source-quality slice that respects the active
identity, iteration, reflection, and linker boundaries. No safe additional
slice was selected from the remaining stable collections modules: the
remaining candidates depend on the held iteration boundary or lack a precise
interval behavior contract. P10 advanced the next priority package, Functional.
P11 completed a comment-only `Reader.map` slice. The MOD1 hash overlay is
current through its source commit; no semantic or runtime claim is made.
P12 completed a comment-only `Writer.map` slice. The inventory is current
through its source commit; no semantic or runtime claim is made.
P13 completed a comment-only `State.run` slice documenting the initial state
and returned transition values. The inventory is current through its source
commit; no semantic or runtime claim is made.
P14 documented that `Writer.mapLog` transforms the log and preserves the
stored value. The inventory is current through its source commit; no semantic
or runtime claim is made.
P15 documented that `Either.mapLeft` transforms a Left payload and preserves a
Right payload. The inventory is current through its source commit; no semantic
or runtime claim is made.
