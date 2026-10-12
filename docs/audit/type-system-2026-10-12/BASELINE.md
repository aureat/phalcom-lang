# Audit Baseline

Assessment date: 2026-10-12  
Reference revision: `5942666901b387519c5cd502bed2611cf97b53c4`  
Branch: `main`  
Platform: macOS (`darwin`)  
Rust toolchain: `nightly-2026-07-10` (local `rustc 1.99.0-nightly (1a98b1e13 2026-08-07)`)  

## Worktree at baseline

The worktree was already dirty before the audit. Existing user work is present
in the `LANG007.C4.P7` implementation records and in
`phalcom-semantic/src/static_eval/{application.rs,error.rs,machine.rs,mod.rs,
residual.rs,specialize.rs}` plus the corresponding semantic tests. These files
are outside the audit's implementation scope and are being preserved.

## Workspace inventory

`cargo metadata --format-version 1 --no-deps` reported 16 workspace members:

`phalcom-ast`, `phalcom-common`, `phalcom-type-syntax`, `phalcom-native-meta`,
`phalcom-native-decl`, `phalcom-native-macros`, `phalcom-native-surface`,
`phalcom-native-surface-gen`, `phalcom-core`, `phalcom-repl`, `phalcom-lsp`,
`phalcom-modules`, `phalcom-semantic`, `phalcom-test-support`,
`phalcom-diagnostics`, and `phalcom-type-meta`.

## Commands and results

| Command | Result | Evidence |
|---|---|---|
| `cargo metadata --format-version 1 --no-deps` | PASS | 16 workspace members; no metadata error |
| `cargo check --workspace --all-targets` | PASS | Finished dev profile; 0 errors |
| `cargo test --workspace` | BASELINE BLOCKED | 300 passed, 7 failed, 1 ignored |

The failed workspace tests were:

1. `parser::record_pattern_remainder_is_terminal_and_map_patterns_are_explicit`
   — parser returned `SyntaxError { kind: UnrecognizedToken { token: "name",
   expected: ["\"#\""] }, range: 65..69 }`.
2. Six AST snapshot tests rejected output containing the new
   `is_primitive: false` field and wrote `.snap.new` files. The affected tests
   are `class_declaration_with_trailing_newline_parses`,
   `bare_attribute_with_no_args_parses`, `ensures_attribute_on_getter`,
   `multiple_attributes_attach_to_same_member_in_order`,
   `standalone_invariant_diverts_to_class_invariants`, and
   `requires_attribute_attaches_to_following_method`.

The snapshot failures are stale expected-output failures rather than type-system
failures. The record-pattern failure is a source/parser baseline issue. Neither
is being changed as part of this read-only audit.

## Baseline interpretation

The workspace compiles, but full test certification is `BASELINE_BLOCKED`.
Focused audit commands will be reported separately and will not be promoted to
workspace certification. Test-generated `.snap.new` files are audit artifacts
from the baseline command and should not be treated as user changes.
