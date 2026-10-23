# Incident: Universe artifact METADATA section is not deterministic across processes

- **Trigger:** slice 0 step 4 (record Universe digests). Five runs of `build_universe_telosc` in separate processes gave five different `artifact_digest` values and different total lengths (21481906 .. 21483140 bytes).
- **Observed:** `interface_digest`, `semantic_digest` and `query_digest` are identical in every run; only `metadata_digest` (section size 2565977 .. 2566310) varies. Within one process the build is byte-identical, which is all the existing test asserts (`canonical_artifact.rs`: "bitwise deterministic").
- **Predates this program:** slices A–D changed 4 files, none touching metadata or the Universe builder (`git diff 141e01442..605bc94d9 --stat`).
- **Rules involved:** VER-R1 (byte-identical Universe output) and VER-D1 (determinism), which assume whole-artifact reproducibility.
- **Decision taken (not improvised around a rule):** VER-R1 is gated per section on the three stable digests recorded in `baseline-digests.md`. Metadata nondeterminism is left unfixed (outside this program's scope: cause is in `MetadataSectionV1::from_bundle` / the metadata bundle source, likely hash-order iteration). Any VER-D* whole-artifact determinism check for the **package** artifact must either compare per section or first fix this.
- **Options for the owner:** (a) fix metadata ordering at its source, then restore whole-artifact VER-R1; (b) keep per-section gating.
