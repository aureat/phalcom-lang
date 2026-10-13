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

Through `UNIV002.C2.P24`, the 174 shadow source paths were reconciled against
the MOD1 baseline at source revision
`0e6c058a1dc5f90849f91adf4a16ad3f53eea6ad`. Nineteen source files changed since
baseline; their current SHA-256 values are recorded in `INVENTORY.md`. Other
source hashes still match baseline. Its textual inventory heuristics remain
baseline data and are not a current opportunity audit.

`UNIV002.C2.P17` completed Category A comments above `Either.mapRight`,
`Either.bimap`, and `Either.swap`. The source hash overlay is current through
its source commit; no semantic or runtime claim is made.

`UNIV002.C2.P18` completed Category A comments above `Arrow.then` and
`Arrow.compose`. The inventory is current through its source commit; no
semantic or runtime claim is made.

`UNIV002.C2.P19` completed Category A comments above `Arrow.dimap` and
`Arrow.second`. The inventory is current through its source commit; no
semantic or runtime claim is made.

`UNIV002.C2.P20` completed one Category A comment above `Iso.inverse` in
`functional/optics.ph`. It documents the callback exchange only and preserves
the existing Product/reflection constraints and callback-law caveat.

`UNIV002.C2.P21` completed comments above `Iso.compose` and `Iso.modify` in
`functional/optics.ph`. They document the existing forward/reverse callback
order and focus update only; the inventory is current through the source
commit and no semantic or runtime claim is made.

`UNIV002.C2.P22` completed comments above `Optional.set`, `Optional.modify`,
and `Prism.modify` in `functional/optics.ph`. They document the callback paths
and unchanged-source branches without promising callback effects; the
inventory is current through the source commit and no semantic or runtime
claim is made.

`UNIV002.C2.P23` completed six comments above `Iso.asLens`,
`Optional.compose`, `Prism.compose`, and `Prism.asOptional` in
`functional/optics.ph`. They document callback paths and composition order
visible in the method bodies; the inventory overlay now records the updated
source hash.

`UNIV002.C2.P24` completed three comments above `FloatReduction.logSumExp` in
`math/reduction.ph`, grounded in `docs/specs/numeric-math.md`. They describe
the one-pass stable operation and the normative empty, infinity, and NaN
cases. The algorithm and C9 verification status are unchanged.

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
## Next action

`UNIV002.C2.P1`–P24 are complete: twenty-three Category A documentation slices
and one Category B Gregorian predicate rewrite. P25 is active for one comment
above `IntegerMath.isqrt` in `math/integer.ph`, documenting the floor-root
result and live negative-input error path from `docs/specs/numeric-math.md`.
The plan is comments-only; no tests are required, and no behavior claim will be
made.

The 174 source paths match MOD1, with nineteen changed hashes reconciled
through source commit `0e6c058a1dc5f90849f91adf4a16ad3f53eea6ad`. P25 must
refresh the changed hash and recompare the full path/hash set. Inventory
heuristic columns remain baseline-only.

`STDL001.C9.P1` established the Float-based `universe.math` boundary and moved
optional special functions, statistics, and probability to the `numerics`
project. Its source/native edits are implemented, but Phalcom source compilation
and runtime acceptance remain unverified; P24 and P25 do not advance that gate.

Keep the architect-guided Option/Result/Unit identity and normative-spec work
separate. Preserve active MODL001 P4 changes and C3 crypto work. Exclude
iteration ownership, reflection/typing, S5 concurrency boundaries, production
providers, bootstrap, native mappings, compiler, and VM changes.
