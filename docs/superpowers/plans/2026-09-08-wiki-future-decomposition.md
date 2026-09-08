# Phalcom Future-Decomposed Wiki Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild `docs/wiki/` as a concept- and architecture-oriented knowledge base whose articles follow Phalcom's future decomposition, while package directories remain only source provenance.

**Architecture:** Keep `docs/wiki/index.md` as the domain map and `docs/wiki/log.md` as the append-only operation record. Organize compiled articles under capability domains such as language, semantic analysis, runtime, collections, native surfaces, diagnostics, editor, tooling, performance, and standard library; keep immutable raw snapshots under `docs/wiki/raw/<source-area>/`. One source area may feed multiple concept articles, and one article may cite multiple source areas.

**Tech Stack:** Markdown with relative links; repository source/spec/implementation records as evidence; `rg` for navigation; the Karpathy Wiki evidence checker for scoped raw-grounding checks; read-only link/inventory checks.

**Spec:** `docs/implementation/INDEX.md`, `docs/implementation/README.md`, and the existing decomposition pattern in `docs/wiki/modules/`.

## Global Constraints

- The canonical compiled wiki root is `docs/wiki/`; do not recreate root-level `wiki/` or `raw/` directories.
- Immutable raw material lives under `docs/wiki/raw/<source-area>/`; compiled articles live under one-level topic directories.
- Article titles, filenames, and index sections name concepts or architecture seams, not Cargo packages.
- Package names may appear in Sources and Raw metadata, implementation pointers, or source provenance notes.
- Every load-bearing number, date, or direct quote in a compiled article must be present in its linked raw source material.
- `docs/spec/` is normative for language behavior; `docs/implementation/` describes implementation plans and lifecycle, not implemented behavior by itself.
- Preserve existing modules articles and unrelated dirty work until their replacement article has inbound links and evidence coverage.
- Each migration updates `docs/wiki/index.md` and appends to `docs/wiki/log.md`; raw snapshots are never rewritten.
- Report implemented, focused-tested, baseline-blocked, and release-complete states separately; a documentation migration is not a code-release claim.

---

## Target taxonomy

| Domain | Purpose | Initial concept articles | Primary planning anchors |
|---|---|---|---|
| `language/` | Source syntax and language surface | lexical structure; expressions and control flow; blocks/closures; strings/interpolation; patterns/matching; annotations and Phaldoc | LANG001, LANG002, LANG003, DOCS003 |
| `type-system/` | Type structures and callable contracts | symbolic type notation; callable/generic contracts; ADT/GADT families; records/rows; type metadata | TYPE001, TYPE002, TYPE003, TYPE004, SEMA001 |
| `modules/` | Project/module topology | retain the current overview, identity, manifests, resolution, interfaces, linking, graphs, sessions articles | MODL001 |
| `semantic/` | Canonical semantic authority and products | type formation/inference; authority/identity; workspace incrementality; capabilities/flow; products/reflection; coverage; constructors | SEMA001–SEMA009 |
| `collections/` | Product and collection behavior | product model; maps; indexed access/ranges; traversal; argument expansion | COLL001–COLL005 |
| `runtime/` | Execution and object model | runtime representation; compiler/runtime boundary; VM execution; object/value lifecycle; errors/cleanup | RUNT001, RUNT002, COMP001, COMP002 |
| `concurrency/` | Cooperative execution | fibers; scheduler/reactor; reflection; futures | CONC001 |
| `native/` | Declarative native and host contracts | universe catalog; primitive contracts; declaration pipeline; canonical surface; generated drift; host capabilities | UNIV001, NATV001, NATV002 |
| `diagnostics/` | User-facing error products | report model; terminal rendering; source snippets/locations; result/error/traceback surfaces | DIAG001, LSPX003 |
| `editor/` | IDE-facing semantic products | LSP architecture; workspace snapshots; editor presentation; LSP performance | LSPX001–LSPX004 |
| `tooling/` | Interactive and documentation tooling | REPL sessions; REPL intelligence; commands; Phaldoc; documentation organization | CLIT001, DOCS001, DOCS003 |
| `performance/` | Measurement and optimization evidence | benchmark corpus; measurement protocol; hot paths; inline caches; runtime instrumentation | PERF001–PERF003, TEST001 |
| `standard-library/` | Library contracts | numeric model; numeric literals/arithmetic; floats/text; numeric errors; collection library buildout | STDL001, STDL002 |

### Package-to-concept placement

- `benchmarks/` becomes `performance/benchmark-corpus.md`, with benchmark-family details split into subarticles only when the evidence has a distinct contract.
- `phalcom-common/src/range.rs` feeds `diagnostics/source-locations.md`; `phalcom-common/src/selector.rs` feeds `runtime/selector-identity.md` or a callable/dispatch article.
- `phalcom-diagnostics/` feeds report, snippet, terminal-style, and error-surface articles; it does not become a package-level domain.
- `phalcom-native-meta/`, `phalcom-native-decl/`, `phalcom-native-macros/`, `phalcom-native-surface/`, and `phalcom-native-surface-gen/` feed the native declaration-to-catalog pipeline articles.
- Existing `type-system/` pages become concept pages such as `symbolic-type-notation.md` and `semantic-type-metadata.md`; their package names remain only in provenance.
- Existing `modules/` pages already follow the desired conceptual pattern and are the migration template, not a reason to duplicate package overviews elsewhere.

---

## Task 1: Lock the domain map and article contract

**Files:**
- Modify: `docs/wiki/index.md`
- Modify: `docs/wiki/log.md`
- Create: `docs/wiki/<domain>/overview.md` for each domain that receives content
- Create: `docs/wiki/raw/<source-area>/...` only for newly ingested source material

**Interfaces:**
- Consumes: `docs/implementation/INDEX.md` program names and statuses; current `docs/wiki/modules/` article pattern.
- Produces: one navigable domain map, stable concept filenames, and a one-paragraph purpose statement per domain.

- [x] Extract the program categories and ownership statements from `docs/implementation/INDEX.md`.
- [x] For each domain, write an overview whose first paragraph states the domain boundary and its neighboring domains.
- [x] Make every index entry point to a concept article or domain overview; do not add package names as new top-level sections.
- [x] Add cross-links at domain boundaries: language → type-system/semantic; semantic → modules/editor; native → runtime/standard-library; diagnostics → editor/tooling; performance → runtime/concurrency.
- [x] Append one log entry for the taxonomy decision and record any deliberately deferred domain.

**Acceptance checks:**
- The index can reach every domain overview and every current article.
- No new article title is merely a Cargo package name.
- The domain overview boundaries do not claim implementation completion from a `PROPOSED` program.

---

## Task 2: Preserve and re-home the existing conceptual core

**Files:**
- Modify: `docs/wiki/modules/*.md`
- Modify: `docs/wiki/type-system/*.md`
- Modify: `docs/wiki/index.md`
- Modify: `docs/wiki/log.md`
- Create: `docs/wiki/type-system/symbolic-type-notation.md`
- Create: `docs/wiki/type-system/semantic-type-metadata.md`

**Interfaces:**
- Consumes: current modules articles; current type-syntax and type-metadata articles; raw snapshots under `docs/wiki/raw/type-system/`.
- Produces: stable conceptual names and cross-links that later semantic, runtime, and editor pages can cite.

- [x] Keep the modules article decomposition for identity, project/manifest, resolution, interfaces/linking, dependency graphs, and sessions.
- [x] Rename or merge the current type articles by concept, preserving their Raw links and source-grounded claims.
- [x] Add explicit boundary sections explaining which facts belong to syntax, semantic type authority, module topology, or runtime projection.
- [x] Search all wiki articles for old article paths and update inbound links before retiring any duplicate page.
- [x] Append the migration record without rewriting the raw snapshots.

**Acceptance checks:**
- Existing modules links remain valid.
- Type-system articles link to semantic and module concepts without importing package ownership into their titles.
- A cold reader can navigate from module identity to semantic metadata without using Cargo package names as the mental model.

---

## Task 3: Compile the native contract pipeline as one architecture

**Files:**
- Create: `docs/wiki/native/universe-catalog.md`
- Create: `docs/wiki/native/primitive-contracts.md`
- Create: `docs/wiki/native/declaration-pipeline.md`
- Create: `docs/wiki/native/canonical-surface.md`
- Create: `docs/wiki/native/generated-surface-drift.md`
- Create/update: `docs/wiki/raw/native-*/...`
- Modify: `docs/wiki/index.md`
- Modify: `docs/wiki/log.md`

**Interfaces:**
- Consumes: `phalcom-native-meta`, `phalcom-native-decl`, `phalcom-native-macros`, `phalcom-native-surface`, `phalcom-native-surface-gen`, `phalcom-type-syntax`, and UNIV001/NATV001/NATV002 implementation records.
- Produces: a source-to-catalog narrative: authored attribute → normalized declaration → generated record → runtime/tooling lookup.

- [x] Split the current package articles into the five pipeline concepts.
- [x] Put UniverseKey, superclass relations, bindings, and type forms in the universe article.
- [x] Put visibility, lifecycle, ABI, effects, raises, flow, trust, and callable contracts in primitive-contracts.
- [x] Put selector parsing, normalization, documentation capture, and validation in declaration-pipeline.
- [x] Put catalog lookup, deterministic selector fallback, fingerprints, intrinsic checks, and return shapes in canonical-surface.
- [x] Put source census, deterministic ordering, Rust emission, rustfmt, and `--check` drift behavior in generated-surface-drift.
- [x] Place host bytes/path/resource/stream/network contracts in native host-capability articles, not in the declaration pipeline.

**Acceptance checks:**
- No native package article is needed to explain the end-to-end contract.
- Every native concept page cross-links its producer and consumer seam.
- Generated-file excerpts are clearly labeled; omitted generated records are not treated as evidence for claims they do not contain.

---

## Task 4: Build language, type-system, and semantic tracks

**Files:**
- Create: `docs/wiki/language/*.md`
- Create: `docs/wiki/type-system/*.md`
- Create: `docs/wiki/semantic/*.md`
- Create/update: `docs/wiki/raw/ast/`, `docs/wiki/raw/semantic/`, and relevant type raw sources
- Modify: `docs/wiki/index.md`
- Modify: `docs/wiki/log.md`

**Interfaces:**
- Consumes: `phalcom-ast/`, `phalcom-semantic/`, `phalcom-type-meta/`, type syntax/native type specifications, docs/spec indexes, and LANG001–LANG003/TYPE001–TYPE004/SEMA001–SEMA009.
- Produces: a source-to-proof-to-product narrative that keeps parser facts, formal semantic authority, and consumer projections distinct.

- [x] Write language pages from normative specs first, then annotate implementation status from live source and program metadata.
- [x] Separate type formation/inference from published metadata, editor products, and runtime type projection.
- [x] Give callable/generic contracts, ADT/GADT families, row types, coverage, and constructor completion their own concept pages when they have independent invariants.
- [x] Record syntax migrations and proposals as historical or proposed material with explicit status blocks.
- [x] Cross-link semantic workspace/incrementality to module sessions and LSP snapshot coherence.

**Acceptance checks:**
- Normative language claims link to `docs/spec/` evidence.
- Formal semantic claims are not inferred from advisory editor or compiler observations.
- Every page states its implementation/verification boundary when the underlying program is partial or unverified.

---

## Task 5: Build runtime, collections, and concurrency tracks

**Files:**
- Create: `docs/wiki/runtime/*.md`
- Create: `docs/wiki/collections/*.md`
- Create: `docs/wiki/concurrency/*.md`
- Create/update: `docs/wiki/raw/core/`, `docs/wiki/raw/collections/`, and `docs/wiki/raw/concurrency/`
- Modify: `docs/wiki/index.md`
- Modify: `docs/wiki/log.md`

**Interfaces:**
- Consumes: `phalcom-core/`, collection implementation/spec records, CONC001, RUNT001/RUNT002, COMP001/COMP002, and the native surface/runtime links.
- Produces: capability pages for object/value representation, VM execution, collection products, fibers, scheduling, GC/lifecycle, and runtime errors.

- [x] Split collection documentation by product model, map model, indexed/range behavior, traversal, and argument expansion.
- [x] Split runtime documentation by representation, execution/compiler boundary, memory/GC, and lifecycle/error behavior.
- [x] Split concurrency documentation by fiber execution, scheduler/reactor, reflection, and future library.
- [x] Link each runtime consumer to the owning semantic or native contract instead of duplicating type/effect facts.
- [x] Use benchmark and test-corpus pages as verification references, not as runtime design authorities.

**Acceptance checks:**
- A reader can trace one feature from semantic contract to compiler lowering to VM behavior.
- Collection and concurrency pages do not collapse distinct ownership or lifecycle invariants into a package overview.
- Runtime claims distinguish focused behavior from baseline-blocked or unverified evidence.

---

## Task 6: Build diagnostics, editor, tooling, performance, and standard-library tracks

**Files:**
- Create: `docs/wiki/diagnostics/*.md`
- Create: `docs/wiki/editor/*.md`
- Create: `docs/wiki/tooling/*.md`
- Create: `docs/wiki/performance/*.md`
- Create: `docs/wiki/standard-library/*.md`
- Create/update: matching `docs/wiki/raw/<source-area>/`
- Modify: `docs/wiki/index.md`
- Modify: `docs/wiki/log.md`

**Interfaces:**
- Consumes: `phalcom-diagnostics/`, `phalcom-lsp/`, `phalcom-repl/`, Phaldoc records, benchmark docs/results, standard-library programs, and DIAG001/LSPX001–LSPX004/CLIT001/DOCS001/DOCS003/PERF001–PERF003/STDL001–STDL002/TEST001.
- Produces: user-facing product articles and measurement articles that explain ownership, evidence, and limitations.

- [x] Split diagnostics into report model, terminal rendering, source snippets/locations, and result/error/traceback surfaces.
- [x] Split editor into LSP architecture, semantic products, workspace snapshots, source locations, and performance.
- [x] Split tooling into REPL execution/session state, interactive intelligence, commands, and Phaldoc/documentation generation.
- [x] Split performance into benchmark corpus/measurement protocol, hot paths, inline caches, and instrumentation/session ledgers.
- [x] Split standard-library pages by numeric contract, literal/arithmetic behavior, float/text behavior, and error/collection surfaces.
- [x] Preserve machine-specific benchmark caveats and distinguish measured, pending, and unmeasured comparisons.

**Acceptance checks:**
- Diagnostics and editor articles share source-location terminology without duplicating ownership.
- Performance pages never turn a benchmark result into a universal implementation guarantee.
- Tooling pages identify whether a feature is specified, implemented, focused-tested, or baseline-blocked.

---

## Task 7: Migrate raw provenance, index integrity, and legacy package pages

**Files:**
- Modify: all touched `docs/wiki/*.md` metadata and relative links
- Modify: `docs/wiki/index.md`
- Modify: `docs/wiki/log.md`
- Preserve: `docs/wiki/raw/**`
- Retire only after migration: package-named compiled pages that have no remaining unique concept content

**Interfaces:**
- Consumes: all compiled concept pages and their Raw fields.
- Produces: one canonical article per concept, no dead links, and a complete raw-to-article inventory.

- [x] For each package-named page, search the full wiki for inbound links and unique claims.
- [x] Merge unique claims into the owning concept page, retaining source attribution and status annotations.
- [x] Replace inbound links with concept paths before removing or converting package pages.
- [x] Keep package names in Raw filenames and Sources metadata so provenance remains searchable.
- [x] Update the index for every new, merged, retired, or renamed page.
- [x] Append a log entry for each source ingest and one consolidation entry per domain migration.

**Acceptance checks:**
- Every non-archive compiled article has a valid Raw field.
- Every non-disposed raw snapshot is referenced by at least one article.
- No index entry points to a retired page.
- No concept page is orphaned when another article materially depends on it.

---

## Task 8: Run the documentation quality gates

**Files:**
- Read-only: all `docs/wiki/` articles and raw snapshots
- Modify only for confirmed mechanical fixes: `docs/wiki/index.md`, affected article links, `docs/wiki/log.md`

**Interfaces:**
- Consumes: the completed domain taxonomy and all migrated articles.
- Produces: evidence-backed completion report with remaining baseline issues called out separately.

- [x] Run the Karpathy evidence checker against the changed article paths using a temporary root with `wiki -> docs/wiki` and `raw -> docs/wiki/raw` symlinks.
- [x] Run a full relative-link inventory; report legacy module-link issues separately from newly introduced failures.
- [x] Compare index entries against actual compiled article files and verify Updated dates.
- [x] Search for package-shaped article titles, duplicate concept titles, dead See Also links, missing cross-topic links, malformed status blocks, and orphan pages.
- [x] Run `rg` checks for trailing whitespace and accidental raw modifications.
- [x] Report documentation status as: implemented (articles written), focused-tested (scoped evidence/link gates pass), baseline-blocked (legacy pages or fixed-layout tooling prevents a whole-tree clean result), and release-complete only if the requested release gates are explicitly run and pass.

**Expected verification commands:**

```sh
python3 .agents/skills/karpathy-llm-wiki/scripts/check_evidence.py <temporary-root> <changed-article-paths>
rg --files docs/wiki | sort
rg -n '\[.*\]\(' docs/wiki
rg -n '[[:blank:]]+$' docs/wiki
```

Cargo tests are out of scope for documentation-only migration unless a source change is introduced.

---

## Scope review

This parent plan intentionally spans independent domain tracks. Execute Tasks 2–6 as separate reviewable subplans or work units; do not attempt one package-by-package bulk rewrite. The current request authorizes planning and reporting, not implementation, commits, or broad cleanup.
