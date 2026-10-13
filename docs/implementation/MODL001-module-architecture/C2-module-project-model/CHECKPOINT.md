---
id: MODL001.C2
program: MODL001
kind: implementation
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# MODL001.C2 — module project model

This checkpoint contains pending module syntax, project resolution, linking,
compiler/LSP integration, runtime initialization, and stabilization records.
Patch artifacts are retained as historical inputs, not treated as applied
source changes.

## Plans

| Plan | Scope | State |
|---|---|---|
| `MODL001.C2.P1` | Module syntax and project resolution | Proposed |
| `MODL001.C2.P2` | Module linking graphs and LSP integration | Proposed; follows P1 |
| `MODL001.C2.P3` | Module runtime initialization and execution | Proposed; follows P2 |
| `MODL001.C2.P4` | Phase-aware source dependencies for the S4 iteration-split prerequisite | Blocked before start; P2/P3 ordering, phase evidence ownership, stage boundary, and T1 proof tests require architect decision; see [architecture incident](MODL001.C2.P4-architecture-incident.md) |

## P4 architecture incident state — 2026-10-13

`MODL001.C2.P4` remains `BLOCKED / NOT_STARTED / UNVERIFIED`. The [incident report](MODL001.C2.P4-architecture-incident.md) is the current decision packet. It records live linker/semantic/compiler evidence, the revision mismatch between the P4 plan's recorded basis and the inspected branch HEAD, and the dirty-tree qualifier for the independent shadow Universe edits. Those edits do not change the Rust linker, semantic, compiler, or runtime paths cited by the incident.

The architect must decide whether P2/P3 must be completed before P4 or whether the P4 dependency/order must be amended; select the source phase model, canonical evidence owner, and provisional-to-executable link boundary; and require T1 tests for symbolic import resolution before phase finalization and fail-closed rejection of provisional products by executable compile/bootstrap APIs. The existing tree has no test proving either stage-boundary property. Two supplementary analysis drafts, [`C2P4-SLICE-1.md`](C2P4-SLICE-1.md) and [`C2P4-SLICE-2.md`](C2P4-SLICE-2.md), were committed in `48099fbc` as review inputs; neither is recorded as an adopted architecture decision. Do not execute P4 or reopen the iteration child split before the architect's decision is recorded and the plan is amended. The acyclic S5 shadow source moves documented in `packages/universe/docs/migration-status.md` are separate and remain independent of this hold.
