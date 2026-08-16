# Verification record

## Passing scope

The following commands passed against commit `9af96f6a0960470e280e2719ff49e1eb3ae135a8` with `CARGO_TARGET_DIR=target`:

```text
CARGO_TARGET_DIR=target cargo test -p phalcom-core --test integration modules_runtime -- --nocapture
5 passed, 0 failed

CARGO_TARGET_DIR=target cargo test -p phalcom-modules -- --nocapture
all package tests passed

CARGO_TARGET_DIR=target cargo test -p phalcom-repl -- --nocapture
19 passed, 0 failed

CARGO_TARGET_DIR=target cargo test -p phalcom-core --test modules_compile -- --nocapture
2 passed, 0 failed

git diff HEAD^ HEAD --check
passed with no output
```

The focused module runtime tests cover diamond materialization/execution, sticky initializer failure, context-free inline import rejection, module export getter dispatch, and a single file-backed execution path. The module package tests cover graph, interface, linker, and package-level behavior. The REPL lane confirms that the compatibility migration still passes its existing 19 integration tests.

The already-built CLI also passed these smoke checks: `target/debug/phalcom examples/core_new.ph` exited 0 and printed `System`; `target/debug/phalcom --source 'let x = 1'` exited 0. These validate file and inline paths only.

## Confirmed failure

The new directory entry path fails before the selection branch:

```text
$ target/debug/phalcom phalcom-core/tests/fixtures/modules_v1/diamond_app
Error: Failed to read file phalcom-core/tests/fixtures/modules_v1/diamond_app
exit 74
```

The same fixture compiles and runs when the test directly passes `EntrySelection::Project`, isolating the defect to the CLI's pre-selection source read rather than the project compiler itself.

## Baseline or unrelated scope

`CARGO_TARGET_DIR=target cargo clippy -p phalcom-core --all-targets -- -D warnings` did not provide a clean commit gate. It reported the repository's broad pre-existing `result-large-err` debt around `PhError`, plus older `bind_instead_of_map`, `unwrap_or_default`, and related lints across unchanged code. Changed module functions also appear in the large-error lint because they use the existing `PhResult` type; this report does not promote that architectural baseline into a finding. A changed-file lint pass with baseline comparison remains unverified.

The old physical-import language test was ignored by design because Modules v1 retires that surface; it was not counted as a regression. A combined Cargo invocation naming unavailable standalone test targets was rejected by Cargo before running tests; the valid package/test-target commands above are the evidence used here.

## Unverified scope

No full workspace acceptance run, release-mode performance run, sanitizer/Miri run, CLI process-test suite, package-directory smoke test, module-object re-export fixture, invalid synthetic-cycle test, or linked-`ModuleId` API test was available in this review. Those lanes should remain explicitly open until the P1 findings and their regression tests are addressed.

