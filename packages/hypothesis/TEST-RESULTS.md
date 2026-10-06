# Current verification status

**Lifecycle: `BLOCKED` / `PARTIAL` / `FOCUSED_TESTED`.** The focused utility
fixtures pass, but the end-to-end property search is blocked by Phalcom generic
trait dispatch. See the detailed [incident report](INCIDENT-REPORT.md).

## Verified on 2026-10-06

Using the existing `target/debug/phalcom` executable:

- Choice snapshots/copies/variants, settings defaults, failure-origin/status
  handling, and complexity ordering: **8 passed** across the focused runtime
  runs.
- Integer shrink-pass fixture: **passed** when re-probed; it has been moved
  from `blocked/` into the active engine fixtures.
- Deterministic property search: the fixture compiles and begins execution, but
  returns `Errored` because the VM reports `no active conformance target` for
  `Strategy.draw`.
- Integer replay probe: the compiler panics while lowering the generic strategy
  conformance because the conformance callable has no declaration owner
  (`phalcom-semantic/src/identity.rs:303`). A focused blocked-fixture probe
  reproduced the same panic and remains marked blocked.
- Full active runtime corpus: **9 passed, 1 failed, 1 blocked**. The sole active
  failure is the deterministic property search above; the blocked fixture is
  integer replay.

The package's independent choice and result helpers therefore have executable
coverage. Property generation, replay, shrinking, and verification do not yet
have a passing vertical runtime result.

The active fixtures can be run from the repository root with:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build -p phalcom-core --bin phalcom
python3 packages/hypothesis/scripts/run_runtime_tests.py \
  --phalcom target/debug/phalcom --probe-blocked
```
