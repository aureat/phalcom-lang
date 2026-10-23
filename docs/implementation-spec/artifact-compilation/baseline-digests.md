# Baseline digests (slice 0, VER-R1)

Recorded 2026-10-23 at anchor `62ddc6acd` by `cargo run -p phalcom-core --example artifact_baseline_digests` (debug build).

**VER-R1 is defined per section, not per whole artifact**: the METADATA section is not deterministic across processes (see `incident-metadata-nondeterminism.md`). Module world, semantic world and query image are, and these digests must not change when the Universe builder is touched.

```
interface_digest=b52fd6ec5264e5383aea16b3cfda4f0bb1cf80974fbd8e0c9b40b3e49fffa3ed
semantic_digest=4fa012dc4bdd8d67f1f502c27c8f9c41a0aa36bdae686077ca1b87cda6b4dbc8
query_digest=4e7c6bbd01f9a712a64dc1a5e0b1a2f5888092b130514694d3e054504ada6e2f
sections module=428417 semantic=18204276 query=283553 metadata=2565857
```
