# AUD-V1 — C4.1 lacks broad current-head certification evidence

## Classification

- **Program:** LANG007
- **Checkpoint:** LANG007.C4.1 corrective closure
- **Severity:** Medium; release and checkpoint evidence gap
- **Status:** Confirmed verification gap at `ee7e8ee7`
- **Scope:** certification only; no production defect is asserted by this file

## Issue

The recorded C4.1 work has useful focused evidence, but it has not established
workspace or release certification for the current head. The patch changed both
covering-body semantic coordinates and callable-body incremental fingerprints.
Focused semantic tests do not cover the complete compiler, core, LSP, format,
and clippy surfaces that consume those products.

The current checkpoint records should therefore distinguish:

```text
implementation: substantially complete for the C4.1 feature
focused verification: useful and partly reproduced
broad certification: not established
```

## Evidence in the live tree

The C4.1 walkthrough records focused commands including:

- `cargo check -p phalcom-semantic`;
- semantic `impls::bodies`;
- semantic `impls::signatures`;
- record-row foundation tests;
- `git diff --check`.

The current CI contract in `.github/workflows/ci.yml` also includes:

- workspace build and tests with the pinned nightly;
- VS Code extension-host tests;
- rustfmt;
- clippy with `-D warnings`;
- the AST Miri lane.

No current-head run was available in the audit record. The broad local format
gate has also been reported as containing unrelated pre-existing drift, so its
result must be recorded precisely rather than promoted to a pass.

## Consequences

Without the broader gates:

- a semantic API or product change may break compiler projection or lowering;
- the narrowed fingerprint may fail under another crate's dependency path;
- diagnostics or source ranges may regress in LSP/editor consumers;
- formatting, clippy, or pinned-nightly compatibility may remain unknown;
- checkpoint records can overstate verification strength.

This gap does not invalidate a focused semantic pass. It limits the status that
can honestly be claimed.

## Correct architectural approach

Treat certification as an explicit gate after A1/A2 and the required focused
regressions are complete. Keep focused and broad evidence separate in the
checkpoint and walkthrough.

Use the repository's pinned toolchain and CI environment. The local repository
has `.cargo/config.toml` rustflags that are intentionally different from CI, so
the certification record must state which environment each command used.

Do not repair unrelated baseline formatting, clippy, or test failures during
this issue. Classify them as A/B/C/D under the repository workflow and record
the reproduction and scope.

## Testing and certification sequence

### Focused semantic gate

```sh
RUSTFLAGS='' cargo check -p phalcom-semantic
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::matching::gadt_refinement
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::matching::exhaustiveness
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic impls::bodies
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic impls::signatures
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic incremental::traits
```

Confirm each selector runs at least one test.

### Directly affected integration gate

Run the semantic ADT/matching and incremental groups named by the final plan,
then the directly affected core compiler tests for match lowering and semantic
body projection.

### Workspace/release gate

At the explicit certification point, run the CI-equivalent commands:

```sh
cargo fmt --all -- --check
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --workspace --all-targets
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --all-targets
RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy --workspace --all-targets -- -D warnings
```

Run extension-host and Miri lanes when the environment supports them; otherwise
record them as incomplete rather than implying release certification.

## Investigation leads

- Compare local commands with `.github/workflows/ci.yml` and
  `rust-toolchain.toml`.
- Consult `phalcom-semantic/tests/semantic/README.md` for test ownership.
- Consult `phalcom-core/tests/README.md` for semantic versus compiler/runtime
  responsibility.
- Update `LANG007.C4.1-walkthrough.md` and the checkpoint state only with
  completed evidence.

## Acceptance condition

The issue is closed when the named focused and broad gates have completed on
the current implementation, results are recorded with exact commands and
environment, unrelated failures are classified, and the checkpoint uses
`FOCUSED_TESTED` or `RELEASE_COMPLETE` truthfully.
