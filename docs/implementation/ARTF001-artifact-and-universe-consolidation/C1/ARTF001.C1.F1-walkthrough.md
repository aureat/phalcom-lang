---
id: ARTF001.C1.F1
kind: implementation-slice-walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
base_revision: c8c6ab0b2
implementation_revision: 6513f9467
checkpoint: ARTF001.C1
program: ARTF001
prepared: 2026-10-23
---

# ARTF001.C1.F1 — Walkthrough: Export identity

## Outcome

F1 carries a publishing identity through semantic, query, type, and static-value artifact projection. Package-owned rows use the package identity before a resolver receipt exists. The package fingerprint includes the source digest, sorted external project identities, and artifact ABI fields. The source digest uses sorted owned module identities and exact source text, without filesystem paths.

The implementation is in commit `72c73cfe0` (`feat(artifact): project package rows with publishing identity`), followed by formatting commit `6513f9467`. EXP-T1's context implementation is in `c8c6ab0b2`.

## Verification

- `RUSTFLAGS='' cargo test -p phalcom-semantic --lib publishing_identity_overrides_only_its_project_in_both_fingerprint_modes -- --nocapture`: PASS, 1 selected test (EXP-T1).
- `RUSTFLAGS='' cargo test -p phalcom-core --lib package_identity::tests -- --nocapture`: PASS, 2 tests (EXP-T2: package fingerprint inputs and path-free source digest behavior).
- `RUSTFLAGS='' cargo test -p phalcom-semantic --lib package_owned_identity_tests -- --nocapture`: PASS, 1 test (EXP-T3). It exports a package-owned callable-signature row and a nonempty query root with a Package owner.
- `RUSTFLAGS='' cargo run -p phalcom-core --example artifact_baseline_digests`: PASS. VER-R1 section digests match the recorded baseline:

  ```text
  interface_digest=b52fd6ec5264e5383aea16b3cfda4f0bb1cf80974fbd8e0c9b40b3e49fffa3ed
  semantic_digest=4fa012dc4bdd8d67f1f502c27c8f9c41a0aa36bdae686077ca1b87cda6b4dbc8
  query_digest=4e7c6bbd01f9a712a64dc1a5e0b1a2f5888092b130514694d3e054504ada6e2f
  ```

The run also reported `metadata_digest=fd8ca68befc27808426159b25f3815a6ad0dbdd4b27a15e4792b3a7d1e0ba905`, with section lengths `module=428417`, `semantic=18204276`, `query=283553`, and `metadata=2565721`. Metadata digest and length are observations for this run only; metadata nondeterminism is documented in `incident-metadata-nondeterminism.md`.

`rustfmt --edition 2024 --check --config skip_children=true` passed for `package_identity.rs`, `identity.rs`, and `lib.rs`; `git diff --check` passed before both commits. Repository-wide format checking reported formatting differences across the checkout, and no broad formatting pass was applied. No broad workspace, crate-wide test, or release suite was rerun for F1. Existing baseline failures are preserved in `docs/implementation-spec/artifact-compilation/baseline-failures.md`.

## Export identity invariants

- Production exporter paths use context-aware stable identity conversions; remaining context-free conversion calls are test-only.
- Query-mode metadata conversion preserves the publishing override.
- The package source digest includes module path components and exact source bytes, but no checkout path.
- Universe interface, semantic, and query section digests remain byte-identical to the recorded VER-R1 baseline.
