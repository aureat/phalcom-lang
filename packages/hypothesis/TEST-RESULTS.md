# Final Test Results — Hypothesis for Phalcom 0.1.0

## Environment

```text
Verification date: 2026-07-23
Python source/static verifier: available
Phalcom executable: not installed
```

Observed evidence is limited to source/static verification. No `.ph` file was parsed, compiled, or executed by a real Phalcom toolchain in this environment.

## Interim runtime harness (2026-10-06)

The package now has a separate consumer project at `tests/runtime` and a thin
host runner at `scripts/run_runtime_tests.py`. The runner executes each runnable
`*-test.ph` fixture in a fresh Phalcom process and checks its exit status and
expected `PASS` marker. Fixtures under `src/blocked/` are reported as `BLOCKED`
and are rejected by `--strict`.

Observed run on 2026-10-06:

```text
Phalcom build: PASS (deprecation warnings)
Runtime tests: 0 passed, 8 failed, 2 blocked
```

All eight runnable processes panicked during canonical Universe startup, before
the fixture body, with a lowering error for a non-proven match in
`universe:errors.result`. The two blocked fixtures are `engine.integer_shrink_pass`,
which currently fails to compile the qualified `Choice` cases in the Hypothesis
shrink-pass implementation, and `strategies.integer_replay`, which is held at
the known `DrawData` / `ChoiceProvider` boundary. The runtime run therefore
does not establish behavior for the eight fixture bodies. The 2026-07-23
release evidence below is a historical static-verification snapshot.

## Phase 12 red-green evidence

Initial Phase 12 verifier against the unchanged Phase 11 checkpoint:

```text
PASS internal imports
FAIL release metadata and artifacts
FAIL historical implementation removed
FAIL public façade inventory
FAIL release integration fixtures
FAIL source hygiene
FAIL release documentation

1 passed, 6 failed
```

Final Phase 12 verifier:

```text
PASS release metadata and artifacts
PASS historical implementation removed
PASS public façade inventory
PASS release integration fixtures
PASS source hygiene
PASS complete test matrix
PASS release documentation
PASS internal imports

8 passed, 0 failed
```

## Complete release gate

`python3 scripts/verify_release.py` ran the entire gate twice. Each pass produced:

```text
Mutation verification: 5 passed, 0 failed
Phase 01: 2 passed, 0 failed
Phase 02: 3 passed, 0 failed
Phase 03: 4 passed, 0 failed
Phase 04: 5 passed, 0 failed
Phase 05: 6 passed, 0 failed
Phase 06: 6 passed, 0 failed
Phase 07: 8 passed, 0 failed
Phase 08: 8 passed, 0 failed
Phase 09: 8 passed, 0 failed
Phase 10: 7 passed, 0 failed
Phase 11: 7 passed, 0 failed
Phase 12: 8 passed, 0 failed
```

Two-pass aggregate:

```text
Mutation checks executed: 10
Phase verifier checks executed: 144
Observed failures: 0
```

The aggregate counts each named verifier lane on both complete passes.

## Mutation verification

Each mutation was applied to an isolated temporary copy, and the Phase 11 verifier was required to reject it:

```text
PASS mutation provider-normalization
PASS mutation candidate-deduplication
PASS mutation directory-lock
PASS mutation reporter-boundary
PASS mutation linear-span-order

5 passed, 0 failed
```

This mutation suite also ran twice through the final release gate.

## Final source audit

```text
Phase 11 archive files: 215
Final project files: 221
Manifest entries: 220
Unexpected Phase 11 baseline files missing: 0
Intentional historical removals: 5
Missing internal imports: 0
Imbalanced active Phalcom files: 0
Retired construct/control-flow syntax files: 0
Placeholder implementation files: 0
Python cache artifacts: 0
Root façade exports: 67
Package import fixture exports: 67
Documented public exports: 67
Export sets equal: yes
Executable examples present: 7
```

## Runtime command in the 2026-07-23 release plan

The 2026-07-23 release plan documented and required:

```sh
phalcom test --all
```

That command was not run because no `phalcom` executable was installed at the time. It is not implemented by the interim runner. Runtime behavior therefore remained unobserved in that historical snapshot.

The current interim runtime command is documented in `README.md`. It must be run before treating the new `.ph` fixtures as verified. The following release-level evidence also remains unobserved:

- Phalcom parsing and compilation;
- unit, property, stateful, database, golden, and integration runtime behavior;
- example execution as executable documentation;
- repeated fixed-seed runtime execution;
- persistence across fresh Phalcom processes;
- source-package versus installed-package runtime equivalence;
- benchmark timing, allocation, and throughput measurements.

No simulated output is reported as runtime evidence.

## Checksum and clean extraction

The final release procedure regenerates `SHA256SUMS`, verifies every listed entry, builds `phalcom-hypothesis-complete.zip`, extracts it into a clean directory, and reruns:

```text
sha256sum -c SHA256SUMS
python3 -m py_compile scripts/verify_phase*.py scripts/verify_release.py
python3 scripts/verify_release.py
```

The final archive file count and SHA-256 are reported with the delivered artifact after construction.
