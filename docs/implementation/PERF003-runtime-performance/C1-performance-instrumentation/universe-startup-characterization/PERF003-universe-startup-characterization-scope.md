# Characterization scope and source-change record

Baseline: `b00d75b3ac9f50d37cf9abfddfde00917e099031`

Changes added in this investigation worktree:

- `phalcom-core/examples/perf_universe_characterization.rs`: feature-gated control harness for cold, warm, VM-tier, and decomposition experiments. The no-feature fallback only reports that the feature is required.
- `measure_process_startup.py`: five repeated Python and `python3 -S` startup samples plus one Telos blocker probe.
- `analyze_performance_timings.py`: strict parser and CSV/summary writer; malformed timing lines are fatal, and unobserved VM phases remain blank when a fixture fails before VM creation.
- `analyze_process_startup.py`: strict parser for subprocess, exit status, wall time, command, and RSS samples.
- `fixtures/tiny.py` and `fixtures/tiny.ph`: minimal process-startup inputs.
- Reports, raw logs, parsed data, and environment/command records in this directory.

No edits were made to `phalcom-core/src/`, `phalcom-semantic/src/`, test support, canonical source indexing, runtime bootstrap, production instrumentation, or the active `main` worktree. No artifact, snapshot, cache, compiler, or runtime architecture was implemented.

Verification performed:

- `cargo check -p phalcom-core --all-targets` — PASS after adding a no-feature fallback `main` to the example.
- `cargo check -p phalcom-core --all-targets --features test-timings` — PASS.
- Feature-enabled corpus test binary build (`--no-run`) — PASS.
- `cargo build -p phalcom-core --bin phalcom` — PASS.
- Corpus focused runs — executed, but failed before VM construction on the baseline metadata-export error.
- VM-tier characterization — kernel/native controls completed; full constructor failed on canonical Universe ProductKind source indexing.

An initial all-target check exposed that the example had no `main` without `test-timings`; the fallback was added, and the subsequent default and feature-enabled checks passed. No test established runtime behavior because the baseline paths fail before that point.
