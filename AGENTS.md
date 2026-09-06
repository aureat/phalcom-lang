# Working on Phalcom

This guidance applies throughout the repository. Follow more specific directory
guidance when present and the user's explicit task scope and instructions.

## Start with the task

- Read the named plan, specification, supplied failure, and affected code first.
  Use `rg` for targeted navigation; avoid broad repository scans when the relevant
  paths are already known.
- Check `git status --short` and the relevant diff before editing. This checkout
  may contain ongoing work from the user or other agents.
- Preserve unrelated modified, staged, and untracked files. Do not reset, clean,
  overwrite, or broadly format them. Stage only the authorized work unit.
- A request for a plan or verification alone does not authorize implementation.
  Honor explicit checkpoint stops and limits on delegation.
- Read applicable skills when useful to the task; do not load every skill or
  rebuild an existing code index for routine read-only exploration.

## Repository map

| Location | Responsibility |
| --- | --- |
| `phalcom-ast/` | Lexer, parser, and syntax tree |
| `phalcom-semantic/` | Canonical static semantics, types, inference, semantic database, snapshots, and editor products |
| `phalcom-modules/` | Project and module resolution infrastructure |
| `phalcom-core/` | Compiler, bytecode, VM, heap, native primitives, and Universe bootstrap |
| `phalcom-repl/` | Command-line and REPL entry points |
| `phalcom-lsp/` | Language server |
| `tools/vsphalcom/` | VS Code extension and extension-host tests |
| `phalcom-diagnostics/` | Shared diagnostic infrastructure |
| `phalcom-type-*`, `phalcom-native-*` | Type syntax/metadata and native declaration/surface infrastructure |
| `phalcom-test-support/` | Shared testing support |
| `docs/spec/` | Normative language specification; begin with its README |

## Specification and implementation

- Follow `docs/spec/README.md` for documentation authority and migration rules.
  Follow topic indexes to the effective specification; a proposal or historical
  plan does not override a canonical language rule.
- Verify implementation claims against live source and relevant tests. A ratified
  specification can describe behavior that has not yet been implemented.
- Keep language rules, design rationale, implementation status, and future plans
  distinct. Record divergences explicitly instead of silently changing the rule.
- Trace source through parsing, semantic analysis, compilation, and execution as
  needed. Distinguish invalid fixtures from checker or runtime defects.
- Keep formal semantic authority in the semantic layer. Compiler and editor
  consumers should use canonical products rather than reconstructing type facts.
  Advisory observations must not strengthen formal proofs.
- Preserve stable identity and ownership boundaries. Keep inference-local
  variables out of published types and snapshots. Incremental changes must
  preserve unaffected products and diagnostics and match cold-analysis behavior.
- Prefer a focused correction at the owning layer over consumer-specific patches.
  Avoid unrelated refactors and speculative abstractions.

## Validation

Use the toolchain pinned in `rust-toolchain.toml`. Check `.cargo/config.toml` and
`.github/workflows/ci.yml` before changing build settings. CI clears `RUSTFLAGS`
and `RUSTC_WRAPPER`; the examples below clear local compiler flags while retaining
the pinned toolchain.

Run the smallest meaningful regression first, then the affected suite at a
coherent checkpoint. Run Cargo validation commands serially to avoid competing
builds. Confirm that a filter actually selected tests; zero tests is not evidence.

```sh
RUSTFLAGS='' cargo check -p phalcom-semantic
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic incremental::db
RUSTFLAGS='' cargo test -p phalcom-core --test core language::algebraic_data
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus booleans
RUSTFLAGS='' cargo test -p phalcom-core --test cli-smoke
```

- Consult `phalcom-semantic/tests/semantic/README.md` and
  `phalcom-core/tests/README.md` for test placement and responsibility.
- Semantic integration tests share `tests/semantic.rs`; use the existing module
  tree. For exact test selection, inspect the full name with `-- --list` first.
  Cargo accepts one positional filter; output capture is disabled with
  `-- --nocapture`.
- Test formal identities and proofs in semantic tests, executable projection in
  compiler tests, and runtime behavior in VM tests. Choose the lowest VM bootstrap
  tier that actually supplies the tested behavior; source-language tests generally
  need the full Universe.
- Add regression coverage for behavior changes. Do not add tests that merely
  mirror implementation details or run compilation for prose-only edits.
- For extension changes, use the scripts in `tools/vsphalcom/package.json` and
  the extension CI lane; extension-host tests need a working language server.

When the task calls for workspace or release verification, run its named gates.
The standard format, build, test, and lint checks are:

```sh
cargo fmt --all -- --check
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --workspace --all-targets
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --all-targets
RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy --workspace --all-targets -- -D warnings
```

Do not weaken assertions, skip failing tests, or change expected output merely to
make a gate green. Investigate a suspected baseline on a clean predecessor when
needed, without disturbing the current checkout. Report canceled or timed-out
checks as incomplete. Broaden or repeat validation only when required by the task
or justified by changed code, failures, or unresolved concerns.

## Delivery

- Review the scoped diff and check whitespace before finishing.
- Report what changed, why, what was verified, and any remaining blocker.
  Distinguish implemented, focused-tested, baseline-blocked, and release-complete.
- A clean or pushed tree is delivery evidence, not behavioral certification.
- Commit and push when requested. Keep related source, fixtures, and expected
  outputs in one coherent work unit; do not stage files just because they share
  an extension. Use `codex/` for new branches unless instructed otherwise.
- Keep progress updates brief and concrete. Do not claim a gate passed unless
  its completed result supports that claim.
