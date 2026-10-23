---
id: ARTF001.C1.F1
kind: implementation-slice-handoff
status: READY
completion: IMPLEMENTED
verification: FOCUSED_TESTED
base_revision: 6513f9467
checkpoint: ARTF001.C1
program: ARTF001
prepared: 2026-10-23
---

# ARTF001.C1.F1 — Handoff: Export identity

## Current state

Continue from branch `feat/artifact-universe-consolidation` in `.worktrees/artifact`, at commit `6513f9467`. F1 is complete and focused-tested. See `ARTF001.C1.F1-walkthrough.md` and the checkpoint for the test results and VER-R1 digests.

## Next slice: F2 package compiler image

The active contract is `docs/implementation-spec/artifact-compilation/05-spec-package-builder.md` §1–2, with ordering and gates in `08-implementation-plan.md`. Limit rereading to those sections and relevant live functions.

- Add scoped `CanonicalSemanticQueryImage::freeze` while preserving `whole_image()` Universe output. Keep local transitive dependency rejection and cycle checks.
- Resolve external or ownerless dependency edges from the immutable base published fingerprint catalogue. If the base has no published fingerprint, return an error; do not prune the row or use a recomputed composed-snapshot fingerprint.
- Build the package capture from the loaded Universe/composed base. Reuse dependency discovery, external module linking, and `link_analyze_finalize`.
- Capture while the semantic session/database is alive, with `canonical_compiler_image` requested before updating the session. Do not reconstruct formal queries from `AnalyzedProgram` after the database is dropped.
- Package scope and linked/interface/topology key sets must equal the four SlotMap modules.
- Retain private base header, composed semantic base, and Universe availability required for the later builder ABI-copy and self-check contracts.
- Reject package-owned diagnostics and Failed/Blocked analysis states.
- Required F2 gate: fresh process with `PHALCOM_DENY_SOURCE_BOOTSTRAP=1`, and `canonical_universe_formal_image_build_count()` remains zero before and after capture.

The aggregate dependency fingerprint helper recomputes rows across the composed snapshot. Restrict owned aggregation to package scope; external and ownerless dependencies must use the base published catalogue as their sole fingerprint authority.

If package-owned metadata must be projected through a new scope-aware `MetadataExporter` API before pool IDs are assigned, send the architect a bounded Q6 proposal before changing that API. Do not filter a completed metadata bundle's type/node arrays without remapping.

Do not start F3 until F2's focused capture gate is green and committed. Preserve the documented baseline failures; do not run broad core auto-discovered examples or the missing semantic-artifact target.
