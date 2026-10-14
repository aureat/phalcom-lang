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

Through `UNIV002.C2.P38`, the 174 shadow source paths were reconciled against
the MOD1 baseline at source revision
`26e70625d5bd131f679096dcbfb8f6365161b7b4`. Twenty-two source files changed since baseline; their
SHA-256 values are recorded in `INVENTORY.md`. Other source hashes still match
baseline. Its textual inventory heuristics remain baseline data and are not a
current opportunity audit.

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

`UNIV002.C2.P25` completed one comment above `IntegerMath.isqrt` in
`math/integer.ph`, documenting the floor-root result and the live
negative-input `ArgumentError` path. The integer algorithm and C9 verification
status are unchanged.

`UNIV002.C2.P26` completed one comment above `IntegerMath.gcd` in
`math/integer.ph`, documenting the nonnegative result and zero/zero case from
`docs/specs/numeric-math.md`. The algorithm and C9 verification status are
unchanged.

`UNIV002.C2.P27` completed one comment above `IntegerMath.lcm` in
`math/integer.ph`, documenting the nonnegative result and zero-input case from
`docs/specs/numeric-math.md`. The algorithm and C9 verification status are
unchanged.

`UNIV002.C2.P28` completed one comment above `IntegerMath.factorial` in
`math/integer.ph`, documenting the exact product and negative-input
`ArgumentError` contract from `docs/specs/numeric-math.md`. The algorithm and
C9 verification status are unchanged.

`UNIV002.C2.P29` completed one comment above `IntegerMath.choose` in
`math/integer.ph`, documenting the exact coefficient, negative-`n`
`ArgumentError`, and zero result for out-of-range `k`, as specified in
`docs/specs/numeric-math.md`. The algorithm and C9 verification status are
unchanged.

`UNIV002.C2.P30` completed one comment above `IntegerMath.fallingFactorial`
in `math/integer.ph`, documenting the exact falling product, negative-input
`ArgumentError`, and zero result when `k > n`, as specified in
`docs/specs/numeric-math.md`. The algorithm and C9 verification status are
unchanged.

`UNIV002.C2.P31` completed one comment above `IntegerMath.risingFactorial` in
`math/integer.ph`, documenting the exact rising product, negative-input
`ArgumentError`, and one when `k == 0`, as specified in
`docs/specs/numeric-math.md`. The algorithm and C9 verification status are
unchanged.

`UNIV002.C2.P32` completed one comment above the native
`FloatMathKernel.hypot` declaration in `math/float_kernel.ph`, documenting the
Float backend's binary64 contract from `docs/specs/numeric-math.md` §2. No
declaration or runtime behavior changed; C9 verification status is unchanged.

`UNIV002.C2.P33` completed a Category A comment replacement above
`FloatReduction.mean`, documenting the finite-Float mean and empty-input
`ArgumentError` contract from `docs/specs/numeric-math.md` §3. The source diff
contained only the comment. The inventory overlay is current through source
commit `e013305787c37873a08bbf24ba57150e60109b41`; no semantic or runtime
claim is made.

`UNIV002.C2.P34` completed one Category A comment replacement above
`FloatReduction.sum`, documenting its compensated finite-Float sum, empty
input result, non-finite input error, and non-finite result error. The source
diff contains no executable changes. The inventory overlay is current through
source commit `9ecf607c4395a7be119c992d99fbf1e1e0004d8f`; no semantic or
runtime claim is made.

`UNIV002.C2.P35` completed one Category A comment replacement above
`FloatReduction.logAddExp`, documenting NaN propagation, positive infinity
dominance, and negative infinity as the additive identity. The source diff
contains no executable changes. The inventory overlay is current through
source commit `465ee0ce3bd1663c77976bb95f03e4521ab549ed`; no semantic or
runtime claim is made.

`UNIV002.C2.P36` completed one Category A comment replacement above
`FloatReduction.log1mexp`, documenting its domain and special cases from the
numeric spec. The source diff contains no executable changes. The inventory
overlay is current through source commit
`0fb428dd36208dad578e53f090f0f68efbebeb0a`; no semantic or runtime claim is
made.

`UNIV002.C2.P37` completed one Category A comment replacement above
`NumericalRangeError`, documenting that it reports a valid argument or result
outside the supported numeric range. The source diff contains no executable
changes. The inventory overlay is current through source commit
`b2dc59cc07394a85affb6f10159991247cfca8a3`; no semantic or runtime claim is
made.

`UNIV002.C2.P38` completed one Category A comment replacement above
`NumericalEvaluationError`, documenting the bounded-evaluation convergence
contract. The source diff contains no executable changes. The inventory
overlay is current through source commit
`26e70625d5bd131f679096dcbfb8f6365161b7b4`; no semantic or runtime claim is
made.

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

`UNIV002.C2.P1`–P38 are complete: thirty-seven Category A documentation
slices and one Category B Gregorian predicate rewrite. P33 documents the
finite-Float mean, P34 documents the finite-Float sum contract, and P35
documents log-add-exp's NaN and infinity behavior; P36 documents log1mexp's
domain and special cases; P37 and P38 document the numeric error contracts,
without changing execution.

The 174 source paths match MOD1, with twenty-two changed hashes reconciled
through source commit `26e70625d5bd131f679096dcbfb8f6365161b7b4`. The committed source
inventory is exact at that revision; its heuristic columns remain
baseline-only. The shared worktree also contains uncommitted C3 crypto source
additions and a `package.ph` edit, which are outside this C2 reconciliation.
Select a bounded P39 Category A source-quality candidate from the inventory.
Continue to avoid held identity/spec work, iteration ownership, reflection,
S5 concurrency boundaries, production providers, compiler, and VM.

`STDL001.C9.P1` established the Float-based `universe.math` boundary and moved
optional special functions, statistics, and probability to the `numerics`
project. Its source/native edits are implemented, but Phalcom source compilation
and runtime acceptance remain unverified; P24 and P25 do not advance that gate.

Keep the architect-guided Option/Result/Unit identity and normative-spec work
separate. Preserve active MODL001 P4 changes and C3 crypto work. Exclude
iteration ownership, reflection/typing, S5 concurrency boundaries, production
providers, bootstrap, native mappings, compiler, and VM changes.
