---
kind: verification-record
title: "Main consolidation of math and Char corrective work"
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
branch: main
date: 2026-10-07
main_parent: e0de68b2e6394cc28a21dc3ca6cf0dae7c7963f4
math_parent: 617d99f540a43f79ddc2aade5898a8118545a4c8
---

# Main consolidation

Delivery anchor: merge commit titled `merge: consolidate math and Char corrective work`.
The merge retains both parent histories and all nine commits from each side
since their shared `615f5887` base. No squash, cherry-pick selection, or
whole-file replacement of overlapping semantic code was used.

## Resolutions

- `LANG007.C4-CHECKPOINT.md`: preserve main's `parallel_follow_on` and P11–P13
  completion state together with math's completion/evidence note. The primary
  C4/P4 structural/static acceptance state remains unchanged.
- `phalcom-native-surface/src/generated.rs`: regenerate from merged authored
  declarations; 399 records. Both math and Char/String entries are retained.
- `phalcom-core/src/modules/static_constant.rs`: handle the newly introduced
  `MaterializationDescriptor::Char` using immediate `Value::char`. Canonical
  owner/name references and the single materialization authority are unchanged.
- `phalcom-core/tests/core/modules/parity.rs`: verify an exported non-BMP Char
  through selective, qualified, renamed re-export, and reflection reads after
  forced GC; aliases retain the source reference and no global const slot exists.

The two edited Rust files were formatted locally. Unrelated formatting drift
was not rewritten. The original checkout's staged/untracked proposals, chars
notes, and similarity package were not included in this isolated merge.

## Preservation evidence

Compared the combined source against both parent trees and the prior simulated
merge tree `24d0f86c`:

- Main's Char/String implementations, semantic pattern resolver/statement
  rules, compiler pattern execution, match fail-closed path, and pattern-site
  lowering are unchanged from `e0de68b2`.
- Math's callable-body/session/module-binding/query changes, Family invocation
  regressions, module send behavior, and GC tracing are unchanged from `617d99f5`.
- Automatically combined checking context, expression checker, fingerprints,
  Universe registration, and native owner metadata match the simulated merged
  tree. Both module-value facts/dependencies and pattern-site products survive.
- Focused Family tests exercise selective and qualified calls in member bodies,
  wrong arguments, provider mutation, and warm/cold parity. Their canonical
  semantic owner remains the module-value boundary and ordinary Family checking.

## Executed verification

All Cargo commands used `RUSTFLAGS='' RUSTC_WRAPPER=''` and the pinned toolchain.

| Command | Result |
|---|---|
| `cargo check --workspace` | PASS |
| `cargo test -p phalcom-core --test core exported_static_char_constants_keep_canonical_identity_across_reexports_and_gc -- --nocapture` | 1 PASS |
| `cargo test -p phalcom-core --test core audit_probes::math -- --nocapture` | 11 PASS |
| `cargo test -p phalcom-core --test core language::chars -- --nocapture` | 11 PASS |
| `cargo test -p phalcom-semantic --test semantic families::invocation -- --nocapture` | 11 PASS |
| `cargo test -p phalcom-core --test core modules_parity -- --nocapture` | 7 PASS |
| `cargo test -p phalcom-core --test language-corpus math -- --nocapture` | 1 PASS |
| `cargo test -p phalcom-semantic --test semantic adts::matching -- --nocapture` | 158 PASS, 16 existing ignored |
| `cargo test -p phalcom-semantic --test semantic capabilities::patterns -- --nocapture` | 6 PASS |
| `cargo test -p phalcom-core --test core algebraic_data_pattern_context -- --nocapture` | 0 executed, 8 existing ignored; no acceptance evidence |
| `cargo test -p phalcom-core --lib native:: -- --nocapture` | 11 PASS |
| `cargo test -p phalcom-core --test language-corpus strings_negative -- --nocapture` | 1 PASS |
| `cargo test -p phalcom-ast --test integration character_literal -- --nocapture` | 4 PASS |
| `cargo run -p phalcom-native-surface-gen -- --root . --check` | PASS, 399 declarations |
| targeted `rustfmt --edition 2024 --check` on the two edited Rust files | PASS |
| `git diff --check` and staged diff check | PASS |

## Verification limits

This is focused consolidation evidence, not release certification. The math
parent's three broad core failures, six wider module/fixture failures, and
workspace formatting drift remain inherited records; those unchanged failures
were not rerun or repaired during this merge. No assertions were weakened and
no additional tests were ignored. Full workspace tests/build/clippy release
gates were not run. The math plan retains `BASELINE_BLOCKED`; P11–P13 retain
their recorded focused verification, and C4/P4/R1 progress is preserved.

Random planning documents are retained with the full branch history. Random,
secure random, and distributions remain unimplemented successor work.
