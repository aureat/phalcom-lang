# Handoff for Phase 2

Continue the Telos numerical/mathematical design in this folder only.

- Treat the existing files as the Phase 1 baseline; extend them rather than creating a competing report.
- Preserve the distinction between **observed current behavior**, **existing intended architecture**, **recommendation**, and **future direction**.
- Ground every claim in live repository evidence; cite files, symbols, tests, or specifications.
- The repository is named Phalcom while the requested language is Telos. Preserve that documented naming assumption.
- Use current Telos/Phalcom syntax only. Mark unsupported or proposed syntax explicitly.
- Keep the design capability-oriented: avoid Rust-style trait inheritance, global implicit promotion, and unsupported language features.
- Keep implementation out of scope unless the Phase 2 prompt explicitly changes that boundary.
- Maintain the bounded package structure and the explicit V1/deferred boundaries.
- Add new material as focused numbered Markdown files and update `README.md` when the report map changes.

## Phase 2 report state — 2026-10-07

The Phase 2 architecture report is in files 12–22. Start at
[12-phase2-recommendation-and-evidence.md](12-phase2-recommendation-and-evidence.md)
and use the updated README report map. Files 01–11 remain the Phase 1 baseline;
file 13 records the explicit foundation amendments.

The recommendation uses two optional consumer packages (`statistics` and
`probability`), independent capabilities, Float statistical kernels with explicit
conversion, immutable validated descriptors, explicit seeded RNG state, typed
conjugate updates, and bounded Monte Carlo. The V1 distribution set is nine
families; Multinomial/Geometric/Dirichlet and generalized inference remain later
work. File 22 owns the decisive boundary and bounded unresolved questions.

Important live finding: `universe.random` is exported but has no generator
implementation. The report proposes a pinned Pcg32 source generator; the
Hypothesis package's small SeededRandom is only a source-language precedent.
Special-function, direct tail/log-tail, and inverse requirements are explicit
dependencies, not facilities already supplied by current elementary math.

The report was reviewed as documentation. No source implementation or Cargo
validation was performed, and proposed API blocks were not compiled. A later
implementation needs focused generic Family/evidence checks and independent
numerical/sampler validation before making behavioral claims.
