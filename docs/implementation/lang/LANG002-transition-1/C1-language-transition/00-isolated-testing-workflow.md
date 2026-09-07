# Transition 1 — Isolated Testing Workflow

## Purpose

Transition 1 changes grammar and selector identity. During its flag day, a
failure in an old `.ph` fixture can mean either:

1. implementation regression; or
2. fixture still uses retired syntax.

Do not collapse those signals by repeatedly running `cargo test --workspace`.
Use separate lanes until Task 2 migrates declaration syntax across the corpus.

## Lanes before fixture migration

| Lane | Command | What it establishes |
|---|---|---|
| Rust compilation | `cargo check --workspace` | AST/compiler/VM/LSP Rust changes type-check without compiling test targets. |
| Focused new Rust test | `scripts/test-transition-1.sh rust ast` (or `core`, `lsp`) | A test named `transition_1_*` covers one new rule without running historical integration fixtures. The command fails if no matching test exists. |
| Legacy behavior | `scripts/test-transition-1.sh legacy <label/case>` | One unchanged existing `.ph` fixture still behaves correctly. This is valid regression evidence only while its syntax is accepted. |
| New surface behavior | `scripts/test-transition-1.sh probe <task/name>` | One new canonical-syntax probe under `phalcom-core/tests/transition-1/` compiles and has the expected behavior. |

The last two lanes build only `phalcom-core`'s `phalcom` binary, then execute a
single source file. They do not build every integration test binary and they
never scan an unrelated fixture directory.

### Bootstrap gate

Every source-level lane compiles `core.ph` before its requested case. If the
current parser can no longer parse `core.ph`, no legacy case or new probe is
valid evidence yet: the runner reports `bootstrap blocked before <case>`.

That is current Task 1 risk. Do not migrate broad tests to hide it. First add
the narrow compatibility path required by Task 1, or make the deliberately
scoped core migration in its assigned task. Until this gate is green, use only
the Rust compilation and `transition_1_*` unit-test lanes to separate AST,
selector, and compiler representation faults from fixture syntax debt.

## Rules

1. Keep all existing `tests/lang/**/*.ph` files unchanged until Task 2's
   declaration migration. Run them one at a time through the `legacy` lane.
2. Put every new source-level Transition 1 test in
   `phalcom-core/tests/transition-1/<task>/`, with a sibling `.expected` file.
   This prevents new syntax from mixing with legacy fixtures.
3. Name isolated in-crate Rust tests `transition_1_*`. The `rust` lane rejects
   an empty filter, so a green result always represents at least one test.
4. For a source-error probe, pass `--negative`; `.expected` then contains the
   required diagnostic substring instead of stdout.
5. Record a legacy failure as **syntax debt** only after the bootstrap gate and
   the same behavior's canonical probe are green. Otherwise treat it as a
   possible regression.

## Examples

```bash
# Task 1: lexer/AST/selector representation, no corpus migration.
cargo check --workspace
scripts/test-transition-1.sh rust ast
scripts/test-transition-1.sh rust core

# Existing legacy behavior, individually isolated.
scripts/test-transition-1.sh legacy classes/class_labeled_arg_method_definition
scripts/test-transition-1.sh legacy compile-errors/compile_error_let_reassignment --negative

# Task 2: canonical declarations and distinct indexing setter identity.
scripts/test-transition-1.sh probe task-02/setter_and_subscript
scripts/test-transition-1.sh probe task-02/malformed_old_label --negative
```

## When to migrate and widen

Task 2 is the semantic flag day. First add and pass canonical probes for each
changed declaration family: positional marker, external-label/local-name,
ordinary setter, subscript getter, and subscript setter. Then migrate the
matching legacy fixture slice and run its ordinary label test, for example:

```bash
cargo test -p phalcom-core --test lang classes
```

Only after every fixture slice, core source, snapshots, and tooling inputs use
canonical syntax does `cargo test --workspace` become a single meaningful
completion signal again. It remains the final gate; it is not the development
loop during migration.
