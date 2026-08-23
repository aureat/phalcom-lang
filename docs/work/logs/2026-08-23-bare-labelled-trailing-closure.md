# Bare labelled trailing closures

- Date: 2026-08-23
- Commit: `bae08ba8` (`fix(parser): accept bare labelled trailing closures`)
- Scope: `phalcom-ast` trailing-closure parsing and regression coverage
- Trigger: the parser rejected:
  `self.ifTrue { "true" } ifFalse: { "false" }`

## 1. Cause

The parser already accepted a bare positional trailing block after an eligible
member send and accepted labelled trailing closures written with an explicit
pipe closure, such as `ifFalse: || { "false" }`.

Labelled trailing-closure lookahead only recognized `label: |...| { ... }`.
When it saw `ifFalse: { ... }`, it left `ifFalse` unconsumed, and the enclosing
statement parser reported an expected semicolon/newline at the label.

## 2. Change

Updated [`phalcom-ast/src/parser.rs`](../../../phalcom-ast/src/parser.rs) so
labelled trailing-closure lookahead also recognizes `label: { ... }`. The
parser routes that zero-argument form through the existing `parse_brace_block`
path and attaches it as a labelled `PackItem`.

The change remains restricted to eligible member-send trailing arguments.
Explicit pipe closures and bitwise-OR parsing are unchanged.

## 3. Verification

- `cargo test -p phalcom-ast --no-fail-fast` — 50 unit tests and 137 integration tests passed.
- CLI parse of the reported source — passed.
- `git diff --check -- phalcom-ast/src/parser.rs` — passed.
- `graphify update .` — passed; HTML visualization skipped because the graph exceeded 5,000 nodes.
- Workspace format check remains blocked by pre-existing formatting drift in unrelated dirty files.

## 4. Worktree scope

Commit `bae08ba8` contains only the parser fix and regression test. Existing
unrelated tracked and untracked changes were preserved and remain unstaged.
