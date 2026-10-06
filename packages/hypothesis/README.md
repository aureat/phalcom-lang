# Hypothesis

Hypothesis is a standalone property-testing library package for Phalcom.

The package is being reconstructed around the current language model. The
active source tree contains scripted, replay, and seeded random choice
providers; integer and boolean strategies; property evaluation; bounded
search; integer shrinking; and repeated replay of minimized failures through
the explicit `Property.check` API. The earlier implementation is retained
under `legacy-src/` as a design reference; it is not part of the active package
surface.

The modernization follows the requirements analysis supplied with this task:
algebraic runtime state uses `data` and `match`, strategies keep their generated
type, examples defensively copy their choice tape, and the engine core avoids
reflection-driven `Any` transport. Composite strategies, semantic spans,
automatic derivation, reporting, persistence, and stateful testing remain later
work.

The package manifest is `project.toml`. The executable entry currently only
identifies the package. Executable runtime tests live in the separate
`tests/runtime` project, which consumes Hypothesis through a path dependency.

Build Phalcom once, then run the runtime corpus from the repository root:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build -p phalcom-core --bin phalcom
python3 packages/hypothesis/scripts/run_runtime_tests.py \
  --phalcom target/debug/phalcom
```

Each `*-test.ph` file runs in its own Phalcom process. The kebab-case suffix
matches Phalcom's module naming rules. The runner reports a
test as passed only when the program exits successfully and prints its expected
`PASS` marker. Known compiler blockers live under `tests/runtime/src/blocked/`
and are reported as `BLOCKED`; `--strict` makes any blocked case fail the run.
A runnable fixture can also be run directly with the Phalcom executable.

`legacy-src/`, the top-level `tests/` corpus, and the existing design
documents are retained as historical material. The `verify_phase*.py`,
mutation, and release scripts perform source or artifact checks; only the
runtime runner compiles and executes Telos programs.
