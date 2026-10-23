---
id: XLNK
program: ARTF001
kind: implementation-spec
status: PROPOSED
branch: feat/artifact-universe-consolidation
anchor: 6489f0a8e
depends-on: ARTC (docs/implementation-spec/artifact-compilation), slices F3 through G
date: 2026-10-23
---

# Executable products and cross-artifact linking — implementation spec

This folder specifies how a package's bytecode is frozen into its own
`.telosc` artifact, how that bytecode links against the bytecode of the
artifacts it depends on, and what `telos run` executes. It closes the gap
recorded as Q1 in [`../artifact-compilation/09-open-questions.md`](../artifact-compilation/09-open-questions.md)
and tracked by VER-H6 there.

The first proof target is the same as ARTC's: `packages/slotmap`, compiled
over the Universe artifact. The end state this spec makes possible is:

```text
telos compile --universe universe.telosc --project packages/slotmap --emit artifact -o slotmap.telosc
telos compile --universe universe.telosc --artifact slotmap.telosc \
              --target bin:slotmap=slotmap.main --emit product -o slotmap.telosx
telos run slotmap.telosx --artifact universe.telosc --artifact slotmap.telosc
```

The syntax is illustrative (the toolchain spec owns it). The run step
parses, checks and emits nothing. Every byte of bytecode it executes comes
from a validated artifact.

The documents are normative. "MUST", "MUST NOT", "SHOULD" and "MAY" have
their RFC 2119 meanings. Every rule carries a stable identifier (for example
`XS-V4`) so tests, commits and review comments can cite it. Rule prefixes:

| Prefix | Document | Subject |
|---|---|---|
| `XL-I` | 01 | Invariants every slice preserves |
| `D-XL` | 02 | Rulings |
| `XS` | 03 | Linked executable section: wire (`XS-W`), validation (`XS-V`), capture (`XS-C`) |
| `XP` | 04 | Executable product file |
| `XM`, `XR` | 05 | Multi-artifact VM materialization (`XM`) and run semantics (`XR`) |
| `XT`, `XA` | 06 | `telos` (`XT`) and `archon` (`XA`) command contracts |
| `VER-X`, `LXH` | 07 | Verification and hostile matrix |

## Status at the anchor revision

| Item | State |
|---|---|
| Universe executable section (schema 1, closed world) | DONE. Built by `build_full_universe_telosc`, materialized by `bootstrap_vm`. Source-free SlotMap run passes (VER-R2), with SlotMap compiled from source in process. |
| Package compiler artifact (`build_package_telosc`) | CODE PRESENT, TEST RED at the anchor: `IncompletePublication("limit exceeded: module 0 has empty path")` from `ModuleWorldV5` validation. |
| Package compiler artifact loading (ARTC slice G) | NOT STARTED |
| Everything in this folder | SPECIFIED, NOT STARTED |

Implementation past slice X2 MUST NOT start before ARTC slice G's gate is
green (see `08`, Prerequisites). Slices X0–X2 (census, wire types, capture
refactor) MAY start earlier because they do not depend on package loading.

## Reading order

1. [01-architecture.md](01-architecture.md): what exists, the key observation that makes linking tractable, the target pipeline, and the invariants.
2. [02-decisions.md](02-decisions.md): rulings, each with the rejected alternatives.
3. [03-spec-linked-executable-section.md](03-spec-linked-executable-section.md): executable schema 2, the import table, validation and capture.
4. [04-spec-executable-product.md](04-spec-executable-product.md): the `.telosx` product file.
5. [05-spec-loading-and-execution.md](05-spec-loading-and-execution.md): validating an artifact chain, materializing it into one VM, and running an entry.
6. [06-spec-cli-contract.md](06-spec-cli-contract.md): what `telos compile`, `telos run` and `archon run` must do.
7. [07-verification.md](07-verification.md): SlotMap proofs, parity, determinism, hostile matrix.
8. [08-implementation-plan.md](08-implementation-plan.md): slice order, files, gates.
9. [09-open-questions.md](09-open-questions.md): unruled questions and risks.

## Relationship to other specifications

- **ARTC (`../artifact-compilation/`).** This spec extends ARTC's package
  artifact with an optional executable section. It reuses ARTC's
  `ModuleWorldV5` external table, publishing identity (EXP-*), package
  loading and composition (LD-*). It supersedes ARTC's non-goal "Executable
  package artifacts" (01 §6) and rewrites LDH-13 (see `07` §4).
- **Toolchain (`docs/specs/toolchain/`).** That tree owns the user-facing
  command surface, the package model, caches and the registry. It leaves the
  executable product format open (`telos-cli.md` §7: "The exact executable
  file format may evolve"). This spec fills that hole and nothing else.
  Where this spec states a command contract (`06`), it is the minimum the
  product requires; the toolchain spec remains the authority for syntax.

## Handoff prompt for the implementing agent

> You are implementing executable products and cross-artifact linking on
> branch `feat/artifact-universe-consolidation` in the worktree
> `.worktrees/artifact`. Slices X0–X2 may run at any time; before slice X3,
> confirm that ARTC slice G's gate is green, and stop if it is not. Read `docs/implementation-spec/executable-products/README.md`,
> then `01`–`08` in order. Implement the slices of `08` in order, and never
> start a slice until the previous slice's gate is green. Rules are cited by
> ID. Do not re-derive any decision in `02`. If the code contradicts a rule,
> stop and write an incident note into this folder (`incident-<slug>.md`:
> trigger, observed code, rule involved, options) instead of improvising.
> Commit once per green checkpoint, staging narrow paths only; never
> `git add -A`; never create or switch branches. Every public Rust item ships
> with rustdoc (`docs/rust-documentation-guidelines.md`). Code anchors here
> are greppable symbol names, not line numbers; re-locate them with `rg`
> before editing.

## Conventions used here

- **Owned / external.** As in ARTC: owned rows belong to the artifact being
  built; external rows belong to a dependency artifact. Ownership is read
  from `StableModuleRef.project` / `ModuleId.project`, never from names.
- **Closed executable.** An executable section with schema 1
  (`EXECUTABLE_SCHEMA_V1`). Only the Universe artifact carries one.
- **Linked executable.** An executable section with schema 2
  (`EXECUTABLE_SCHEMA_V2`, new). Only package artifacts carry one.
- **Artifact chain.** The ordered list of artifacts a run needs: Universe
  first, then packages in the order `plan_package_dependencies` produces.
- **Product.** A `.telosx` file (`04`): the selected entry, the artifact
  chain by digest, and the ABI it was built for. It contains no bytecode.
