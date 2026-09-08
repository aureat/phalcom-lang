# Generated surface drift

> Sources: native surface generator and macro census
> Raw: [native surface generator source snapshot](../raw/native-surface-gen/2026-09-07-phalcom-native-surface-gen.md)
> Updated: 2026-09-08

Generation scans the native declaration census, orders records deterministically, lowers metadata, and emits Rust catalog output. Formatting and a `--check` mode make generated-file drift a reviewable failure.

## Evidence boundary

The generator can prove source-to-output determinism and detect stale generated files; it cannot prove that an implementation's runtime behavior matches its declaration. Runtime and semantic tests own that claim. Generated-file excerpts are evidence only for the records actually captured in the raw snapshot.

[Declaration pipeline](declaration-pipeline.md) feeds the generator. [Canonical surface](canonical-surface.md) consumes its records. The macro layer is an expansion mechanism, not a separate public architecture domain.
