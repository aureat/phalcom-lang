# IMPLEMENTER INSTRUCTION PACKET — Universe Shadow Migration, Next Increment

**Packet:** `UNIVERSE-S4-NEXT`  
**Program:** Telos Universe source architecture and parallel migration  
**Working repository:** `aureat/phalcom-lang`  
**Authorized branch:** `feat/telos-collections`  
**Observed remote HEAD (2026-10-13):** `a1621a7cec577952612685508a4c87c90aea86cd`  
**Concurrent performance branch (observed):** `perf/universe-characterization` at `c0155584972ec8c9f0960e411a9501a9c003de92`  
**Production authority:** `phalcom-core/core/universe/` — **unchanged**  
**Shadow authority under development:** `packages/universe/` — **not production-registered**  
**Primary increment:** S4, collection iteration organization; preceded by a bounded specification correction and source-validation prerequisite  
**Production cutover:** **NOT AUTHORIZED**

> **Execution directive:** Continue the *duplicate-tree migration*. Do not replace, register, initialize, or delete the legacy Universe. The work of this packet must be reviewable and mergeable independently of the ongoing compiler-performance branch. Treat structural source checks, interface checks, semantic analysis, and runtime parity as different gates. Do not report unexecuted gates as passed.

---

## 0. Scope, evidence, and task authority

### 0.1 What is already present

At the observed branch revision, six migration commits follow the original legacy baseline `9be1ed1eba3f94ba05e8e4aca1ee55f5ccf13131`:

| Commit | Delivery |
|---|---|
| `4a219874` | Duplicated legacy Universe into `packages/universe/`. |
| `4d98eb5` | Recorded initial source provenance and baseline. |
| `8e8078a` | Split shadow `Number`, `Int`, `Float` into distinct modules. |
| `f62a8c2` | Recorded scalar split. |
| `089083c` | Moved shadow `Option`, `Result`, and `Unit`; adjusted facades and the Universe architecture spec. |
| `a1621a7` | Recorded foundational-owner migration. |

The shadow presently has 169 `.ph` files; the original has 168. Existing ledgers report **364 conservatively scanned type-like declarations**, **77 top-level values**, and **216 distinct authored import edges**. These are source inventories, **not compiler-semantic verification**. The latest `packages/universe/docs/migration-status.md` explicitly says no compiler checks or runtime tests have been performed against the new layout.

Completed shadow owners:

| Symbol | Legacy owner | New shadow owner |
|---|---|---|
| `Number` | `universe.scalar.number` | unchanged |
| `Int` | `universe.scalar.number` | `universe.scalar.int` |
| `Float` | `universe.scalar.number` | `universe.scalar.float` |
| `Option` | `universe.option.option` | `universe.option` |
| `Result` | `universe.errors.result` | `universe.result` |
| `Unit` | `universe.option.unit` | `universe.scalar.unit` |

**No production native mapping has been migrated.** Production `UniverseKey::source_path()`, the compiler prelude, source provider, and VM bootstrap still use the legacy module identities. This is intentional until cutover.

### 0.2 Explicitly in scope

1. Correct ambiguous **production-active vs future-target** identity assertions in `docs/specs/universe.md` without rejecting the approved target architecture.
2. Establish a **non-invasive shadow parser and source-interface verification** mechanism, capable of inspecting the new physical tree without activating it in the compiler or VM.
3. Inventory the entire existing `collections/iterator_laws.ph` and its source clients, then reorganize its interfaces and adapters under `universe.collections.iteration`.
4. Update shadow-only import/expose/export relationships, package facades, declaration identity ledgers, and provenance.
5. Detect—and explain or resolve at the correct architectural layer—the protocol/adapter dependency cycles that a physical split may introduce.
6. Inventory additional collection-related organizational debt (`collections/bytes.ph`, `scalar/string.ph` iterator implementations); **do not fold their full redesign into S4**.
7. Document verifiable acceptance gates and a precise implementation handoff for the next increment.

### 0.3 Explicitly out of scope

- Any change to `phalcom-core/core/universe/**`, except unrelated work separately authorized by its owner.
- Production modifications to `phalcom-modules/src/builtin.rs`, `phalcom-modules/src/builtin_interface.rs`, `phalcom-semantic/src/prelude.rs`, `phalcom-native-meta/src/universe.rs`, `phalcom-core/src/native/**`, `phalcom-core/src/modules/canonical_universe.rs`, or `phalcom-core/src/vm/bootstrap.rs`.
- Runtime loading/initialization of the shadow Universe; changes to GC, scheduler, native ABI, or VM class identity.
- Implementing `universe.telosc` or backporting `PERF003.C2` compiler optimization code.
- Changing iterator operation behavior, renaming public traits or adapters, replacing signatures with `Dynamic`, or weakening associated-type constraints.
- Full `collections/bytes.ph` extraction, System/Future separation, `numbers`→`math` consolidation, or source-authored prelude activation. These are later increments.
- Removing the original `packages/universe/src/collections/iterator_laws.ph` *from the legacy tree*. The shadow copy may be removed once all its declarations and references have been accounted for.

### 0.4 Sources to inspect first

Use the actual branch HEAD, not only this packet. Prioritize:

```text
packages/universe/docs/migration-status.md
packages/universe/docs/identity-migration.md
packages/universe/docs/module-map.md
packages/universe/docs/dependency-graph.md
packages/universe/docs/legacy-sync.md

docs/specs/universe.md
packages/universe/src/package.ph
packages/universe/src/collections/package.ph
packages/universe/src/collections/iterator_laws.ph
packages/universe/src/collections/{list,array,vector,slice,buffer,map,set,range,tuple,...}.ph
packages/universe/src/functional/{chain,fold,folds,effects,...}.ph
packages/universe/src/math/reduction.ph
packages/universe/src/scalar/string.ph
packages/universe/src/collections/bytes.ph

phalcom-modules/src/interface.rs
phalcom-modules/src/linker.rs
phalcom-modules/src/builtin.rs
phalcom-modules/src/builtin_interface.rs
phalcom-modules/src/identity.rs
phalcom-modules/src/source.rs
phalcom-core/src/native/source.rs
```

Record new findings as evidence, not assumptions. **Never use the shadow manifest alone as proof of builtin identity:** ordinary `ProjectUniverse` filesystem loading can yield `ProjectIdentity::Resolved`, not the builtin `ProjectIdentity::Universe`.

---

## 1. Non-negotiable invariants

| ID | Invariant |
|---|---|
| **S4-INV-01** | One production Universe. The legacy provider remains the sole production-authoritative source. |
| **S4-INV-02** | Shadow and legacy are alternative layouts of the *same conceptual builtin project*, never two runtime universes in one VM. |
| **S4-INV-03** | Exactly one **shadow** defining module per declaration. Re-exports preserve `DeclarationId`; they do not manufacture competing nominal identities. |
| **S4-INV-04** | All preexisting iterator protocols, 14 adapter classes, blanket conformance, method signatures, and observable semantics remain present. |
| **S4-INV-05** | Every outward-facing module and declaration has intentional `expose` / `export` policy. No special native metadata fallback substitutes for a missing authored export. |
| **S4-INV-06** | A source-only or type/interface dependency is not silently treated as a runtime initialization dependency. Runtime SCCs must not be waved away. |
| **S4-INV-07** | Cold/warm production compilation and bootstrap cannot read or initialize the shadow tree through ordinary execution paths. |
| **S4-INV-08** | No global `ModuleId`-only cache is used to mix incompatible legacy and shadow products. |
| **S4-INV-09** | No original production source file is removed, rewritten, or redirected by this packet. |
| **S4-INV-10** | Shadow compiler/semantic/runtime validation is reported precisely as passed, failed, or blocked; source-text scanning does not count as semantic proof. |
| **S4-INV-11** | No false equivalence between Git mergeability and architecture/runtime cutover readiness. |

If an invariant requires currently absent compiler functionality, isolate the gap, produce a bounded incident, continue independent migration work, and mark the blocked gate honestly. **Do not make a speculative compiler patch to force the shadow tree to pass.**

---

## 2. S4.T0 / G0 — Capture current state and protect isolation

**Tasks**

1. Record `git status --short`, branch, HEAD, upstream, and the latest legacy source revision. Do not discard preexisting dirty files or assume they belong to this work.
2. Reconfirm the latest `main` and `perf/universe-characterization` revisions; record them separately. These may advance during S4.
3. Inventory shadow and legacy files. Compare all unchanged shadow paths against their original provenance hashes; explain all planned differences.
4. Check whether the new physical `packages/universe/project.toml` has accidentally entered any registered project roots, workspace initialization, test fixture resolver configuration, or production source manifest.
5. Establish a baseline of **zero S4 changes** to production compiler/runtime and legacy `.ph` files.
6. Confirm the current shadow `option.ph` is the only owner of `universe.option`. Do not introduce the ambiguous `option/package.ph` path again.

**G0 acceptance**

- Recorded exact HEAD and clean/dirty state.
- Legacy source authority unchanged.
- Shadow remains non-production and unregistered.
- Complete S4 candidate declaration/reference census can be reproduced.

**Failure condition:** If the shadow package is production-discoverable as a second `universe`, stop and isolate that registration path before proceeding.

---

## 3. S4.T1 / G1 — Correct the active-vs-target Universe specification boundary

### 3.1 Confirmed defect

`docs/specs/universe.md` currently declares target identities such as `universe.scalar.int::Int`, `universe.option::Option`, and `universe.result::Result` as canonical without clearly stating that production code at this branch still uses their prior modules. The migration ledger correctly acknowledges the discrepancy. This can mislead agents implementing `PERF003` against current production metadata.

### 3.2 Required correction

**Do not revert the approved target architecture.** Add an unambiguous, prominently positioned source-authority/migration-status section, and annotate identity examples so the reader can determine whether they describe:

- **ACTIVE:** current production runtime/compiler source identity at the branch revision; or
- **TARGET:** the approved post-cutover shadow identity, not yet active.

Suggested minimum table (check current production source before writing):

| Declaration | ACTIVE production owner | TARGET shadow owner |
|---|---|---|
| `Number` | `universe.scalar.number` | same |
| `Int` | `universe.scalar.number` | `universe.scalar.int` |
| `Float` | `universe.scalar.number` | `universe.scalar.float` |
| `Option` | `universe.option.option` | `universe.option` |
| `Result` | `universe.errors.result` | `universe.result` |
| `Unit` | `universe.option.unit` | `universe.scalar.unit` |
| `Iterator` | `universe.collections.iterator_laws` | `universe.collections.iteration.protocols` (proposed; unactivated) |
| `Iterable` | `universe.collections.iterator_laws` | `universe.collections.iteration.protocols` (proposed; unactivated) |

Explicitly state that `ProjectIdentity::Universe` applies to both *conceptual* source layouts, but only legacy is currently authoritative for the running compiler/VM. No file path is canonical merely because a duplicated manifest calls itself `universe`.

Review sections 3, 6, and 10 for examples whose declarative tense could be mistaken for a statement about the active build. Preserve the final single-authority invariants. Do **not** reword the approved target as a second alternate permanent authority.

Prefer a small, isolated documentation commit, with minimal overlap against the concurrently evolving perf branch.

**G1 acceptance**

- Readers can distinguish active from target module IDs at every important identity example.
- Updated spec agrees with `packages/universe/docs/identity-migration.md`.
- Changes are documentation-only and do not affect production compiler code.

---

## 4. S4.T2 / G2 — Establish non-invasive shadow source/interface validation

### 4.1 Required architectural distinction

The current shadow ledgers are based on source scanning. Real interface extraction already exists in `phalcom-modules/src/interface.rs` (`InterfaceBuilder::build`), while `phalcom-ast::parse` constructs the AST. The normal builtin `UniverseSourceProvider` embeds the legacy tree and a static list of nodes. The `with_test_source_override` seam covers known canonical modules, and is **not** an automatic alternate package-tree loader; `BuiltinInterfaceBuilder` has process-wide caches keyed by `ModuleId`.

Do not use the legacy builtin provider to pretend the shadow layout has been linked. Do not mix shadow and legacy AST/interfaces within the process-global caches.

### 4.2 Minimal tool design

Implement or reuse a small **shadow-only verification harness**. Prefer a separate test utility or isolated test binary that does not modify production crate logic, production `Cargo.toml` membership, or builtin registration. Reuse actual parser and interface-builder APIs instead of inventing a regex parser.

The validator shall:

1. Walk `packages/universe/src` deterministically.
2. Derive physical nodes: `package.ph` denotes the containing package; other `*.ph` files denote modules. Root `src/package.ph` is `universe:<root>`.
3. Reject collisions where a source module `foo.ph` and a directory package `foo/package.ph` would own the same logical module.
4. Parse every `.ph` file with `phalcom-ast::parse`, aggregate errors with module IDs and physical paths, and **fail** on parser diagnostics.
5. Build unlinked interfaces using `InterfaceBuilder::build` and the actual `ModuleKind`. Detect source-local namespace conflicts and invalid exports; fail rather than suppressing errors.
6. Index authored declarations, top-level value bindings, imports, exports, re-exports, and exposed children. Verify target nodes and package exposure using the repository's documented relative-import semantics.
7. Validate, where the actual linker supports it without using legacy caches, the linked import/export targets. Distinguish **source existence** from **target binding exported by that source**.
8. Report separately the candidate *runtime* dependency graph. Do not claim an acyclic runnable graph from an import-path scan alone.
9. Emit reproducible JSON or Markdown diagnostic output with stable sort order and machine-testable fail conditions.
10. Produce canonical **conceptual** `ModuleId::universe` mappings in the shadow report, not resolved filesystem-project IDs as purported builtin identities.

**An isolated full linker test is desirable, not assumed.** Inspect linker constructors/inputs before invoking them. A parser + `InterfaceBuilder` pass is an honest initial gate, but it cannot certify semantic type resolution, conformance, native anchoring, bootstrap, or runtime semantics.

### 4.3 Anti-false-positive examples

The following tests must fail correctly:

- A missing exported declaration in an otherwise existing module.
- A missing `expose` for a referenced public child.
- A collision between a directory package and a same-name source module.
- Duplicate exported names or duplicate local declarations.
- A missing local binding targeted by `export`.
- An import path that exists in legacy only, not shadow.
- An intentionally malformed Telos source file.

The following must not spuriously fail:

- Valid implicit prelude references **when only parsing/unlinked-interface validation is being claimed**. Their semantic resolution remains a separate gate.
- Different files containing identical declaration short names under distinct module IDs.
- Runtime-support `Some`/`None` identities that are not ordinary independent source declarations.

### 4.4 Scope isolation

If a standalone Rust harness requires adding workspace membership or changing production crate code, first try an existing test-only integration route. If no safe route exists, produce a short design note with alternative harness locations and defer implementation; **do not** smuggle a second production project provider into the compiler.

**G2 acceptance**

- At least parser and unlinked-interface checks execute against the entire shadow source graph and yield recorded reproducible results; or the gate is explicitly **BLOCKED** with exact blocker and demonstration.
- No production native/source provider modification.
- No shadow/legacy cache aliasing.
- Validation levels are labeled `PARSE`, `INTERFACE`, `LINK`, `SEMANTIC`, `NATIVE`, and `RUNTIME`, each with `PASS`, `FAIL`, or `BLOCKED/NOT_RUN`.

---

## 5. S4.T3 / G3 — Create a complete `iterator_laws` extraction map

### 5.1 Baseline

`packages/universe/src/collections/iterator_laws.ph` is currently **910 lines**. The name falsely suggests a laws/test module, while the file owns production protocols and concrete adapters. It contains **16 top-level declarations**: two protocols and fourteen adapters.

**Protocols:** `Iterator`, `Iterable`.

**Adapters:** `MappedIterator`, `FilterIterator`, `FilterMapIterator`, `FlatMapIterator`, `FlattenIterator`, `ChainIterator`, `SkipIterator`, `SkipWhileIterator`, `TakeIterator`, `TakeWhileIterator`, `MapWhileIterator`, `InspectIterator`, `StrictZipIterator`, `IndexedIterator`.

It also owns the blanket `impl<I> Iterable for I where I |= Iterator`. Record this implementation as a first-class migration obligation, not merely a line of text.

### 5.2 Target ownership

```text
packages/universe/src/collections/
├── package.ph
├── iteration/
│   ├── package.ph
│   ├── protocols.ph       # Iterator, Iterable; blanket Iterable-for-Iterator impl
│   ├── transform.ph       # Mapped, Filter, FilterMap, MapWhile
│   ├── flatten.ph         # FlatMap, Flatten
│   ├── combine.ph         # Chain, StrictZip
│   ├── limit.ph           # Skip, SkipWhile, Take, TakeWhile
│   └── inspect.ph         # Inspect, Indexed
├── list.ph
├── array.ph
└── ... existing collections
```

**Canonical target names** (subject to successful cycle/interface analysis):

```text
universe.collections.iteration.protocols::Iterator
universe.collections.iteration.protocols::Iterable
universe.collections.iteration.transform::MappedIterator
universe.collections.iteration.transform::FilterIterator
universe.collections.iteration.transform::FilterMapIterator
universe.collections.iteration.transform::MapWhileIterator
universe.collections.iteration.flatten::FlatMapIterator
universe.collections.iteration.flatten::FlattenIterator
universe.collections.iteration.combine::ChainIterator
universe.collections.iteration.combine::StrictZipIterator
universe.collections.iteration.limit::SkipIterator
universe.collections.iteration.limit::SkipWhileIterator
universe.collections.iteration.limit::TakeIterator
universe.collections.iteration.limit::TakeWhileIterator
universe.collections.iteration.inspect::InspectIterator
universe.collections.iteration.inspect::IndexedIterator
```

Do not create `collections/iteration.ph` alongside `collections/iteration/package.ph`. The package path itself is `universe.collections.iteration`.

### 5.3 Inventory every method and implementation

Produce a table with the **source span, declaration owner, method selector, parameter labels, return type, associated-type constraints, visibility, and target module** for all methods in both protocols and all adapters.

At minimum, check the existing `Iterator` methods:

```text
next, iterator, map, filter, filterMap, flatMap, flatten, chain,
skip, skipWhile, take, takeWhile, mapWhile, inspect, zip
```

And existing `Iterable`/terminal operations including:

```text
iterator, iter, indexed, each, contains, includes, count, find,
index, any, all, none, nth, last, fold, reduce, join,
group, partition, toSet, toMap, toList
```

These lists are checksums of the current intended API, **not substitutes for an exact AST-based method inventory**. Overload sets, labeled forms (`each(indexed ...)`, `count(where ...)`, `toMap(merging ...)`), inferred return types, and generic where-clauses must remain equivalent.

Preserve all special behavior, including:

- `next -> Option<Self::Item>` and type associated-item propagation.
- `Iterable` relation `Self::Iterator |= Iterator` and item equality.
- Blanket `Iterable` for types conforming to `Iterator`.
- Adapter exhaustion/terminal behavior; `next` must not continue advancing after terminal state where the original adapter contracts prohibit it.
- Strict-zip mismatch behavior; do not replace with truncating zip.
- `toMap` duplicate-key/error propagation, and `toMap(merging:)` behavior.
- Iterator-to-collection conversion semantics, predicate call counts, order, and error behavior.
- Existing handling of `StringBuilder` in `join`.
- The distinction between generic `ChainIterator<I,J>` and `functional.chain::ChainIterator<T>`; do not conflate same-spelling declarations.

**G3 acceptance:** A complete line/declaration/method migration map exists and sums to the pre-move inventory. No adapter, overload, or implementation has been silently lost.

---

## 6. S4.T4 / G4 — Resolve protocol/adapter and facade dependency hazards *before* splitting

### 6.1 Required graph evidence

The existing protocol definitions reference adapter constructors directly, for example:

```telos
trait Iterator {
  type Item
  next -> Option<Self::Item>
  map(_ f) { MappedIterator.new(self, f) }
}
```

Every migrated adapter also depends on `Iterator` and often implements it. Naively splitting into `protocols.ph` plus `transform.ph` with ordinary selective imports may cause:

```text
iteration.protocols  --runtime selective import--> iteration.transform
iteration.transform  --runtime selective import--> iteration.protocols
```

`phalcom-modules/src/linker.rs` records `RuntimeDependencyReason::SelectiveValueImport` and re-export runtime dependencies; runtime SCC detection is strict. The legacy `phalcom-core/src/native/source.rs` initialization planner treats authored imports/re-exports as execution-order edges and rejects cycles. **The shadow physical split is not proof such a cycle is executable.**

A second danger is the higher-level `Iterable` methods. `join` depends on `StringBuilder`; `group`, `partition`, `toMap`, `toSet`, `toList` depend on collections. Moving those methods must not create new foundational↔collections cycles or force eager text/collection initialization during bootstrap.

### 6.2 Required design decision record

Before writing imports, produce a small ADR or section in `docs/dependency-graph.md` explaining:

1. The **declaration/interface dependency graph**, separately from the **runtime initialization graph**.
2. Strongly connected components (SCCs) of the proposed split.
3. Which edges arise only from type references, which from method bodies, and which from executable top-level initialization.
4. The current compiler/linker's actual handling of `from ... import`, direct re-export syntax, package facade imports, and prelude fallback.
5. Whether the compiler already has safe interface-only imports, deferred/lazy binding support, or implementation extension mechanisms **with test evidence**.
6. A chosen solution that preserves ordinary Telos module semantics and does not weaken runtime cycle validation.
7. The minimal compiler capability gap, if any, assigned to a **separate future integration plan**, not quietly implemented in S4.

### 6.3 Acceptable strategies

- **Preferred:** Use an already-implemented, tested interface/type-only resolution mechanism to preserve original class/trait identities while keeping executable initialization edges acyclic.
- **Alternative:** If Telos provides an existing extension/conformance mechanism allowing adapter methods to be defined outside the foundational trait *without changing member lookup or inference*, use it with targeted semantic parity tests.
- **Conservative staged fallback:** Keep a verified cohesive source module under an accurately named `iteration` package while extracting adapters into documented proposed modules only as far as independent checks permit. Mark the full six-module split **BLOCKED/PARTIAL**, not complete. This fallback must not be misrepresented as the final organizational goal and should not reintroduce the misleading `iterator_laws` name.

Do **not**:

- Silence cycle diagnostics.
- Invent `type import` or new extension syntax without compiler support.
- Introduce duplicate `Iterator`/`Iterable` nominal declarations.
- Rewrite default methods into unrelated APIs that change user call sites.
- Convert precise item constraints to `Dynamic`.
- Replace canonical imports with broad unspecified builtin lookup.
- Treat parsed-only acceptance as proof of linkability or runtime safety.

**G4 acceptance:** The chosen source decomposition has a supported, explained dependency model. If no valid execution model exists, record an incident and continue source-only work within safe limits; do not mark S4 executable-complete.

---

## 7. S4.T5 / G5 — Implement the shadow iteration package

### 7.1 Source move rules

- Modify **only** `packages/universe/src/**` for Telos source.
- Preserve every declaration body and method implementation in the first relocation pass. Restrict changes to module location, necessary imports, direct exports, package exposures, and minimal syntactic repairs proven necessary.
- Do not optimize algorithms, change naming, or improve functionality in the same move commit.
- Place every adapter in its designated owner **only when the dependency model has been validated**. A blocked adapter move should be recorded as pending, not hidden.
- Avoid placeholder modules claiming public API without declarations.

### 7.2 Package and public-API contract

Eventually, the public interface should permit:

```telos
from universe.collections.iteration import Iterator, Iterable
from universe.collections import Iterator, Iterable
from universe.collections.iteration.protocols import Iterator
```

All three `Iterator` routes must resolve to the exact same canonical `DeclarationId`:

```text
universe.collections.iteration.protocols::Iterator
```

For the adapters, package-level convenience exports may be desirable, but *do not blindly export internal adapter implementation details merely because the old oversized file exported them*. Here the old module **does** export all 14 adapters. Accordingly, inventory current external API use and preserve compatibility through a deliberate public facade unless a separately approved breaking-API decision is recorded. Exporting through the facade must never create a second nominal declaration.

`collections/package.ph` must:

- `expose .iteration`, and keep existing public children exposed.
- Remove `expose .iterator_laws` **in the shadow tree once all references are migrated and the new interface is validated**.
- Re-export `Iterator` and `Iterable` through the correct canonical owner, subject to the runtime dependency/initialization design.

The `iteration/package.ph` should expose its actual implemented children. Use only documented, parser-supported import/export/re-export syntax; do not invent an interface-only keyword.

### 7.3 Full consumer rewrite, shadow-only

Find **every** shadow `.ph` reference to:

```text
universe.collections.iterator_laws
```

Known consumers include:

```text
packages/universe/src/collections/list.ph
packages/universe/src/collections/array.ph
packages/universe/src/collections/bag.ph
packages/universe/src/collections/bijection.ph
packages/universe/src/collections/buffer.ph
packages/universe/src/collections/bytes.ph
packages/universe/src/collections/indexed_dict.ph
packages/universe/src/collections/map.ph
packages/universe/src/collections/mutable_bag.ph
packages/universe/src/collections/ordered_set.ph
packages/universe/src/collections/range.ph
packages/universe/src/collections/range_map.ph
packages/universe/src/collections/set.ph
packages/universe/src/collections/slice.ph
packages/universe/src/collections/tuple.ph
packages/universe/src/collections/vector.ph
packages/universe/src/scalar/string.ph
packages/universe/src/functional/chain.ph
packages/universe/src/functional/effects.ph
packages/universe/src/functional/fold.ph
packages/universe/src/functional/folds.ph
packages/universe/src/math/reduction.ph
```

This is an observed representative set, not an exhaustive substitute for repository-wide search. Audit remaining references in all shadow source, tests, examples, docs, and APIs. **Do not rewrite legacy production imports**, which must still point to the old module.

Preserve aliasing if necessary (`Iterator as CanonicalIterator`) unless the resolver and source semantics demonstrate the alias is redundant. Do not produce name collisions with collection-specific iterator classes.

### 7.4 Boundary with the prelude

Do not add `Iterator` or `Iterable` to the implicit prelude without an explicit API policy decision. Public package exports and implicit prelude membership are separate. The ultimate `universe.prelude` package is a later task; S4 should not change the production prelude.

### 7.5 Safe deletion rule

The shadow `iterator_laws.ph` may be removed only after:

- All sixteen original declarations have unique shadow owners.
- The blanket conformance and all methods are accounted for.
- No surviving shadow source/module interface imports the old module.
- Source-preservation diffs and parser/interface checks pass.
- No collision/identity duplication remains.

**G5 acceptance:** All extractable iteration declarations and imports are source-correct and accounted for; any link/runtime limitation is explicitly described and does not cause unauthorized production changes.

---

## 8. S4.T6 / G6 — Tests, counterexamples, and semantic parity gates

### 8.1 Immediate automated checks (must run)

Run reproducible checks applicable to the shadow without switching the builtin provider:

1. Source path/name collisions (including `package.ph` versus peer `.ph` ownership).
2. All shadow file parse diagnostics.
3. AST-based unlinked-interface building, source-local export/import collision checks.
4. Exists/exposed/exported checks on imported symbols.
5. Unique owning module for each moved declaration.
6. Full declaration/method/conformance preservation inventory for the split.
7. No surviving shadow `iterator_laws` imports after final extraction.
8. No modifications to the legacy source/provider/metadata/bootstrap.
9. `git diff --check` and a scoped changed-path audit.
10. Provenance and identity-ledger consistency checks.

### 8.2 Semantic checks (run **only** when an isolated canonical shadow frontend is valid)

Check exact canonical references and generic obligations for:

- `Iterator::Item` and `Iterable::Iterator` / `Iterable::Item` equality.
- The blanket `Iterable` for every conforming `Iterator`.
- Every adapter's declared `Item`, `next`, generic bounds, and constructors.
- `List`, `Array`, `Map`, `Set`, `Range`, `Tuple`, `Bytes`, `String`, functional `Chain`, and math reductions that use the protocols.
- Explicit import and package-facade routes resolving to the same declaration identities.
- Negative tests for absent exports/exposure, missing associated type, invalid conformance, and import collisions.

If an isolated compiler requires a different source authority under the same canonical `ProjectIdentity::Universe`, obtain a separate integration plan before changing the production provider. Report this gate **BLOCKED** until testable; do not run a plain filesystem-project compile and call its `Resolved` identities canonical builtin identities.

### 8.3 Runtime parity (not in scope to claim PASS)

Record expected future parity fixtures for:

- Iterator exhaustion (including repeated `next` after terminal).
- Mapping, filtering, flattening, chaining, skipping, taking, inspecting, and index enumeration.
- Strict zip and length mismatch behavior.
- Side effect/predicate counts and laziness.
- `Iterable.each`, `fold`, `reduce`, `group`, `partition`, `join`, `toList`, `toSet`, and both `toMap` forms.
- Runtime exception propagation and specialized iterator identities.

**These fixtures are future cutover evidence.** The production runtime is still driven by the old source; running existing legacy tests proves only that legacy remains intact. Record that distinction.

### 8.4 Verification matrix

| Verification | Required status for source-delivery S4 | Required before production cutover |
|---|---|---|
| Legacy source untouched | PASS | PASS |
| Shadow source full scan | PASS | PASS |
| Shadow parser | PASS or explicit blocker, no invented success | PASS |
| Shadow unlinked interfaces | PASS or explicit blocker | PASS |
| Shadow linked exports/exposures | PASS or explicit blocker | PASS |
| Shadow semantic type/conformance analysis | May be BLOCKED, with cause | PASS |
| Native canonical association | NOT RUN / BLOCKED | PASS |
| VM initialization DAG | NOT RUN / BLOCKED with SCC analysis | PASS |
| Runtime behavioral parity | NOT RUN / BLOCKED | PASS |
| Cold/warm compilation comparison | NOT RUN / BLOCKED | PASS where applicable |
| `universe.telosc` identity and fingerprint integration | OUT OF SCOPE | Separate future plan |

An S4 source-delivery commit is permitted with honest semantic/runtime blockers, but do not claim **S4 fully integrated/complete** or remove a still-needed bridge until the corresponding interface/graph gates are established.

---

## 9. S4.T7 / G7 — Extended ownership inventory (design only)

Produce a bounded next-phase report on related oversized modules; do not mix their code changes into S4.

### 9.1 `collections/bytes.ph` (approximately 820 lines)

Declarations observed:

```text
Bytes, BytesIterator, OpenMode, Path, Resource,
UnflushedError, ConcurrentOperationError, StreamProtocolError, WriteZeroError,
BytesReader, BytesWriter, BufferedWriter, BufferedReader
```

Evaluate separating:

- Byte values and `BytesIterator` under `collections`.
- Resource and byte I/O abstractions under the most semantically appropriate `io` modules.
- Filesystem `Path` and path operations under `path` **only if behavior and canonical native association support the move**.
- I/O error classes according to their error domain.

`UniverseKey::Bytes` and `UniverseKey::Resource` currently share the legacy module path. They will need distinct future association updates if split. Do not relocate native owner mappings in S4.

### 9.2 `scalar/string.ph` (approximately 1,120 lines)

Contains `String`, `StringChars`, `StringCharIterator`, `StringByteSequence`, `StringByteIterator`, `StringCodePointSequence`, and `StringCodePointIterator`. Decide whether cohesive iterator/view helpers belong near `String` or in focused `scalar/string_*` modules. Avoid moving them into the *generic adapter* package just because they are iterators; their identity and implementation depend on `String`.

### 9.3 Later domains

Keep a queue of the previously authorized future reorganizations: `concurrency/fiber.ph` (System/Future/Fiber split), `callable/method.ph` and `callable/family.ph`, error classes, `time/clock.ph` and `time/timestamp.ph`, `numbers` into `math`, and special `universe.prelude` source interface.

**G7 acceptance:** A specific next-stage inventory/ownership proposal exists without unapproved source edits in those domains.

---

## 10. S4.T8 / G8 — Migration ledgers, merge safety, and reporting

Update the existing files **in place**; do not invent redundant parallel status systems:

```text
packages/universe/docs/identity-migration.md
packages/universe/docs/module-map.md
packages/universe/docs/dependency-graph.md
packages/universe/docs/legacy-sync.md
packages/universe/docs/migration-status.md
```

Add an S4 design note/incident in `packages/universe/docs/` only if its findings materially require a separate decision record. The ledgers must capture:

- Branch, exact commit(s), upstream, and source baseline.
- All 16 iteration declarations, previous and proposed owners.
- Full set of moved implementation/method bodies and their equivalence evidence.
- All affected shadow import sites, written and verified.
- Created/deleted shadow module files, `expose` and export changes.
- Interface graph, runtime graph, SCCs, and any blocked edge category.
- Former and target visible APIs; deliberate alias/re-export policy.
- Tests run with command, revision, result, and output location; blocked tests explicitly distinguished.
- Production files audited for unintended modification.
- Pending sync changes from legacy and expected merge constraints.

### 10.1 Integration branch discipline

The performance branch has independently advanced to `c0155584972ec8c9f0960e411a9501a9c003de92` at inspection. It contains ongoing PERF003.C2/P4 compiler metadata and source/semantic product changes. Refresh its HEAD before proposing integration. Do not cherry-pick or port P4/P5 internal changes in this S4 task.

- The shadow tree is the primary source-edit surface; this minimizes direct Git conflicts.
- A full merge can still encounter preexisting unrelated changes elsewhere in `feat/telos-collections`.
- Newly authored `docs/specs/universe.md` status clarification should be narrowly scoped, not a wholesale normative rewrite.
- The actual cutover owner must later reconcile `ModuleId`/prelude/native mappings and compiled artifact fingerprints against the **then-consolidated** compiler architecture.
- Never use `git merge` success as evidence that semantic identity migration is safe.

### 10.2 Recommended atomic commit progression

1. `docs(universe): distinguish active and target source identities` — S4.T1.
2. `test(universe): add isolated shadow source-interface checks` — S4.T2, only if implementable without production intrusion.
3. `docs(universe): map iteration declarations and dependency SCCs` — S4.T3/T4.
4. `refactor(universe): organize shadow iteration protocols and adapters` — S4.T5, bounded to validated relocations.
5. `docs(universe): record S4 validation and migration ledger` — S4.T6–T8.

Do not commit interim states that would make the active production compiler consume the shadow tree. Commit a blocked/incomplete shadow slice only when its incompleteness is explicit and its layout remains structurally inspectable; otherwise keep design-only artifacts until a valid boundary is found.

### 10.3 Required final implementer report

Use this structure:

```markdown
# Universe Shadow Migration — S4 Delivery

- Branch / HEAD / upstream:
- Legacy baseline / provenance:
- Perf branch checked at:
- Commits delivered:
- Files added / moved / removed (shadow only):
- Any production/legacy/compiler changes (expect: none; spec-note exception):
- Declared canonical target identities:
- Declaration, method, overload, conformance preservation evidence:
- Import/export/expose surface changes:
- Interface graph SCCs and chosen cycle policy:
- Verification PASS / FAIL / BLOCKED / NOT_RUN, with exact commands:
- Active-vs-target documentation correction:
- Legacy divergence / synchronization findings:
- Remaining architectural incidents:
- Next implementation slice recommendation:
- Explicit confirmation: PRODUCTION CUTOVER NOT PERFORMED.
```

**G8 acceptance:** Documentation accurately tracks source and semantic progress, baseline is intact, and remaining issues are assigned an explicit next owner/gate.

---

## 11. Mandatory escalation

Open a separate incident report (or stop the affected slice with an equivalent evidence-based consultation packet) if any of these occurs:

1. `protocols.ph` ↔ adapter modules form an unavoidable import/runtime SCC under the current linker.
2. Default trait implementations cannot reference adapters from other modules without changing public language semantics.
3. The available tooling cannot derive the correct shadow canonical identity without invoking global legacy caches.
4. A private/helper adapter is found to be part of published API and removal would break external import semantics.
5. A broad prelude fallback is needed to hide unresolved imports.
6. A source module resolves only because native metadata synthesizes an un-authored export contrary to the target module contract.
7. A proposed import/alias creates duplicate or conflicting namespace bindings.
8. A source refactor changes generic signatures, trait conformance, strict zip behavior, iterator exhaustion, or terminal operation evaluation order.
9. The new source tree is accidentally included in production bootstrap or discovery.
10. The active-vs-target spec correction conflicts with another authorized document change and cannot be resolved narrowly.
11. The implementer is tempted to change `PERF003.C2` compiler internals, native bootstrap paths, or global caching to satisfy a shadow-only test.

A valid incident includes required invariant, exact current revision, relevant code paths and symbols, minimized reproducer or graph cycle, observed versus expected behavior, safe alternatives, recommended architectural ruling, and work that can continue independently. Do not silently downgrade a blocked gate or modify a test expectation.

---

## 12. Final instruction / success criteria

**Proceed now:** capture fresh state, correct the active-vs-target specification language, implement the smallest safe shadow validation mechanism, inventory the 910-line iteration module fully, characterize interface/runtime SCCs, and migrate the iteration source in bounded reviewable units **only after the dependency model is defensible**.

At the conclusion, S4 must deliver:

- An accurately named, progressively organized `universe.collections.iteration` domain in the shadow tree, with a documented owner for every original protocol/adapter.
- No loss of source declarations, generic constraints, overloads, method implementations, or blanket conformances.
- Explicit source-interface status and visibility, without pretending that automatic cache/native exports are authored API.
- An honest graph/verification assessment and, if necessary, a precise compiler-integration blocker.
- No production compiler, VM, prelude, native catalog, or legacy Universe modification.
- A complete migration/merge-reconciliation trail and a defensible next-step handoff.

**Do not cut over. Do not delete the legacy Universe. Do not redirect production compiler authority. Do not merge compiler optimizations to make a shadow-only milestone pass.**
