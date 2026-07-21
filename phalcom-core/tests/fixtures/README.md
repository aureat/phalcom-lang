# Phalcom Test Fixtures

This directory contains the test fixtures for the Phalcom programming language.

## Diagnostic Field-Assert Contract

As specified in `docs/spec/current/traceback/implementation-spec.md` §11 ("Testability"), traceback and diagnostic fixtures must adhere to the **field-assert contract**:

1. **Structure is the Contract:**
   - Traceback fixtures must assert the structural fields of the JSON output (`--trace-format=json`) rather than freezing the pretty-printed, colored, or styled text representation on stderr.
   - Assertions on traceback JSON lines should check:
     - The frame sequence: `module`, `name`, `line`, `core`, and `fiber`.
     - The error properties: `message` and `kind` (e.g., `#doesNotUnderstand`, `#concurrentMutation`, etc.).
   - Trace events (such as fiber switches) must assert specific event fields (e.g., `ev`, `from`, `to`, `fiber`, `at`, etc.).

2. **Output Streams:**
   - Standard output (`stdout`) must remain byte-exact and untouched for user program output.
   - All diagnostic, error, and trace stream outputs must be written to standard error (`stderr`).

3. **Color-Off Invariance:**
   - A styled diagnostic render (e.g. `--color=always`) stripped of ANSI escape (SGR) sequences must be byte-equivalent to the clean render using `--color=never`. Color is purely for emphasis and must never carry structural information.

4. **Negative-Control Rule:**
   - Every negative fixture (asserting compiler errors, syntax errors, or runtime errors) must be negative-controlled. That is, it must be verified that the test case fails when the fix is not present, ensuring the assertion is active.
