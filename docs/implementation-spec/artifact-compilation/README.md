---
id: ARTC
program: ARTF001
kind: implementation-spec
status: PROPOSED
branch: feat/artifact-universe-consolidation
anchor: 605bc94d9
date: 2026-10-23
---

# General artifact compilation — implementation spec

This folder specifies **general artifact compilation**: compiling an ordinary
package (first target: `packages/slotmap`) into its own compiler artifact
(`.telosc`) that owns only the package's modules and references Universe (and
other packages) as external dependencies. The current pipeline can build
artifacts for Universe only, as a single closed image.

The documents are normative. "MUST", "MUST NOT", "SHOULD" and "MAY" have their
RFC 2119 meanings. Every rule carries a stable identifier (for example
`MW5-V7`) so tests, commits and review comments can cite it.

## Status at the anchor revision

| Slice | Content | State |
|---|---|---|
| A | `ArtifactProjectionScope` + `ArtifactSemanticProjection` | DONE `48435bb71` |
| B | Ownership-scoped root registration | DONE `48435bb71` |
| C | Ownership-scoped semantic world export (`export_semantic_world*_scoped`) | DONE `c6ff478c4`; only exercised with the Universe scope (see 04) |
| D | Ownership-scoped query image export (`export_query_image_v4_scoped`) | DONE `605bc94d9`; same caveat |
| 0 | Restore `phalcom-semantic` lib-test compilation; run A–D unit tests | NOT STARTED |
| E | `ModuleWorldV5`: external module references | SPECIFIED (03) |
| F1 | Context-threaded stable identity in export + publishing identity | SPECIFIED (04), **newly found blocker** |
| F2 | Package compiler image capture | SPECIFIED (05) |
| F3 | `build_package_telosc` | SPECIFIED (05) |
| G | External-aware loading and composition | SPECIFIED (06) |
| H | SlotMap package artifact proof | SPECIFIED (07) |
| I | Hostile, determinism and regression matrix | SPECIFIED (07) |

## Reading order

1. [01-architecture.md](01-architecture.md): the model, its terms, and the invariants that every slice preserves.
2. [02-decision-module-world-v5.md](02-decision-module-world-v5.md): the ruling on how dependency modules are referenced, with the rejected alternative.
3. [03-spec-module-world-v5.md](03-spec-module-world-v5.md): normative wire, validation, fingerprint and lift rules (slice E).
4. [04-spec-export-identity.md](04-spec-export-identity.md): stable identity of package-owned rows during export (slice F1).
5. [05-spec-package-builder.md](05-spec-package-builder.md): package image capture and the artifact builder (slices F2, F3).
6. [06-spec-package-loading.md](06-spec-package-loading.md): loading a package artifact against its dependencies (slice G).
7. [07-verification.md](07-verification.md): SlotMap proof, hostile matrix, determinism (slices H, I).
8. [08-implementation-plan.md](08-implementation-plan.md): slice order, files, gates, commit discipline.
9. [09-open-questions.md](09-open-questions.md): questions outside this spec's scope and the risks that remain.

## Handoff prompt for the implementing agent

> You are implementing general artifact compilation on branch
> `feat/artifact-universe-consolidation` in the worktree `.worktrees/artifact`.
> Read `docs/implementation-spec/artifact-compilation/README.md`, then
> `01`–`08` in order. Implement the slices in the order of `08`: start with
> slice 0, and never start a slice until the previous slice's gate is green.
> Rules are cited by ID. Do not re-derive any decision marked RULED. If the
> code contradicts a rule, stop and write an incident note into this folder
> (`incident-<slug>.md`) instead of improvising. Commit once per green
> checkpoint, staging narrow paths only. Every public Rust item ships with
> rustdoc (`docs/rust-documentation-guidelines.md`). Code anchors in these
> documents are greppable symbol names, not line numbers. Re-locate them with
> `rg` before editing.

## Conventions used here

- **Owned / external.** Owned modules belong to the artifact being built.
  External modules belong to another artifact (a dependency). Ownership is read
  from `ModuleId.project`, never from names or paths (rule `ARCH-I1`).
- **Local index.** `ArtifactModuleId(u32)`: a dense index into the artifact's
  own module rows.
- **External index.** `ExternalModuleIndex(u32)`: an index into the artifact's
  `external_modules` table (new in V5).
- **Universe artifact.** The builtin `universe` project artifact built by
  `build_universe_telosc` / `build_full_universe_telosc`.
- **Package artifact.** An artifact whose header identity is
  `TeloscPackageIdentity::Package { package, version }`.
