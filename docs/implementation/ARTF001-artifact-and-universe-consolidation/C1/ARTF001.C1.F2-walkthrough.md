---
id: ARTF001.C1.F2
kind: implementation-slice-walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
base_revision: 7329c71da
implementation_revision: 93cc9f2fc
checkpoint: ARTF001.C1
program: ARTF001
prepared: 2026-10-23
---

# ARTF001.C1.F2 — Walkthrough: Package compiler image

## Outcome

F2 adds `PackageCompilerImage` capture over a loaded artifact base. Capture
reuses module dependency discovery and linking, initializes prelude state from
the loaded Universe snapshot, and keeps the artifact-backed semantic session
alive through scoped query freezing and dependency fingerprint aggregation.
The compiler product request is selected on that session before its first
update. Private provenance retains the base header, composed semantic base, and
Universe availability for the builder and loader checks in later slices.

Package capture restricts filesystem source access to its owning resolved
project. Foreign dependency projects must be provided by the external artifact
authority; otherwise discovery fails before source parsing. Capture rejects
owned error diagnostics and any scoped Failed/Blocked query census.

Scoped query freezing keeps Universe `whole_image()` output unchanged. Owned
dependencies resolve locally, while external dependency fingerprints come
only from the immutable base catalogue; missing and mismatched fingerprints
fail closed. Package metadata applies `ArtifactProjectionScope` before pool ID
assignment and uses a source fingerprint derived from the path-free package
source digest. The Q6 decision and regression are recorded in
`incident-metadata-package-projection.md`.

An initial source-denied run caught a helper constructing a temporary default
`SemanticWorkspaceSession`, which attempted canonical source bootstrap. The
capture helper now uses the already initialized external-backed session. The
fresh-process test passes with source denial active and the Universe build
counter at zero.

## Verification

- `RUSTFLAGS='' cargo check -p phalcom-core --lib`: PASS after the final query-terminal guard.
- `RUSTFLAGS='' cargo test -p phalcom-core --lib package_capture_rejects_foreign_dependency_source_before_parsing -- --nocapture`: PASS, 1 test. A filesystem-backed foreign project is rejected before the resolver can parse it.
- `RUSTFLAGS='' cargo test -p phalcom-core --test core modules_canonical_artifact::fresh_process_source_denied_artifact_compilation -- --nocapture`: PASS, 1 test in 121.88s. Package capture ran under `PHALCOM_DENY_SOURCE_BOOTSTRAP=1`; the canonical Universe build count remained zero, and scope, interfaces, topology, linked authority, and executable module keys each matched the four SlotMap modules.
- `RUSTFLAGS='' cargo test -p phalcom-semantic --lib package_projection_tests -- --nocapture`: PASS, 1 test. Owned callable metadata remains, dependency-owned rows are filtered, and source/interface fingerprints match across different registered source paths for the same package identity and source digest.
- `scoped_freeze_keeps_owned_rows_with_immutable_external_dependencies`: PASS. Positive, missing-fingerprint, and mismatched-fingerprint cases were covered.
- `RUSTFLAGS='' cargo run -p phalcom-core --example artifact_baseline_digests`: PASS. Universe interface, semantic, and query digests remain unchanged:

  ```text
  interface_digest=b52fd6ec5264e5383aea16b3cfda4f0bb1cf80974fbd8e0c9b40b3e49fffa3ed
  semantic_digest=4fa012dc4bdd8d67f1f502c27c8f9c41a0aa36bdae686077ca1b87cda6b4dbc8
  query_digest=4e7c6bbd01f9a712a64dc1a5e0b1a2f5888092b130514694d3e054504ada6e2f
  ```

Metadata digest and size varied in this run, consistent with
`incident-metadata-nondeterminism.md`. No broad core or workspace suite was
run. `git diff --check` passed. The new `package_image.rs` was formatted; a
check over existing large Rust files reported unrelated pre-existing format
differences, so no broad format rewrite was applied. Known baseline failures
remain recorded in `baseline-failures.md`.
