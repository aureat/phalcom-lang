---
id: ARTF001.C1.F2
kind: implementation-slice-handoff
status: READY
completion: IMPLEMENTED
verification: FOCUSED_TESTED
base_revision: 93cc9f2fc
checkpoint: ARTF001.C1
program: ARTF001
prepared: 2026-10-23
---

# ARTF001.C1.F2 — Handoff: Package compiler image

## Current state

F2 is complete and focused-tested at `93cc9f2fc` on
`feat/artifact-universe-consolidation` in `.worktrees/artifact`. The evidence
and exact Universe section digests are in `ARTF001.C1.F2-walkthrough.md` and
`CHECKPOINT.md`.

`PackageCompilerImage::capture_from_universe` covers Universe-only package
dependencies. The crate-private `capture_from_base` accepts a composed
artifact base for later package dependency support. Capture retains private
base header, composed semantic base, and Universe availability; F3 should use
these values instead of regenerating dependencies or hard-coding ABI fields.
The package-owned source provider rejects foreign projects before lookup or
parsing. Scoped query freezing and metadata projection are ready for the
builder.

## Next slice: F3 package module-world projection and builder

The architect assigned F3. Read `docs/implementation-spec/artifact-compilation/05-spec-package-builder.md`
§3–4 and `07-verification.md` H1/H2, then implement the bounded projection and
builder work.

- Add `project_package_module_world_v5` in `phalcom-modules/src/compiler_artifact.rs` with two-pass deterministic external `StableModuleRef` indexing. The modules crate must not depend on `phalcom-semantic`; use a borrowed stable-module conversion callback supplied by core, where the publishing `StableIdentityContext` is authoritative.
- Require local-only entry, initialization, topology, semantic-row, and witness-importer references to belong to the package scope. Fail on foreign inputs.
- Keep package bootstrap fields empty. Validate external Universe availability witnesses against the exact proofs retained on `PackageCompilerImage`.
- Build exactly four sections with package identity `Package{slotmap, 0.1.1}`, module-world schema V5, and no `PORTABLE_EXECUTABLE` capability. Use scoped root, semantic, query, and metadata exporters with the same publishing identity.
- Check project closure across semantic, query, and metadata owner references. If closure would change the external-project identity inputs used by F1's fingerprint, stop and report the inconsistency.
- Add focused VER-H1/H2 and captured-image projection tests, followed by the unchanged Universe section-digest gate.
- Defer builder self-check to G as assigned. Do not weaken it to use a closed-world loader. Do not change package import paths; the adopted LD-C4 catalog-threading amendment is a later G prerequisite.

The crate-boundary clarification is recorded in
`incident-package-projection-crate-boundary.md` and the PB-M1 amendment in
05. For the later G loader, preserve Universe's empty-path root-entry rule.
Package artifacts instead require the header identity to own the actual entry
project and a local root-module row for that project; the entry module itself
may have a nonempty path such as `slotmap.main`. Do not edit loader paths
during F3. Preserve the existing baseline failures; do not run broad core
auto-discovered examples or the missing semantic-artifact target.
