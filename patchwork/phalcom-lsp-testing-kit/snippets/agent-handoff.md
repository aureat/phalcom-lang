# Agent handoff instructions

1. Reconcile this kit against the implementation branch; do not blindly overwrite newer files.
2. Run `fixture_syntax` first.
3. If a semantic acceptance test fails, inspect SemanticDb facts before loosening assertions.
4. Prefer adding a pure semantic unit test next to the subsystem, while keeping the RPC test as user-facing proof.
5. Preserve negative assertions:
   - no child-only member through `super`;
   - no A.User/B.User identity leakage;
   - no stale method after provider edit;
   - no visible `Unknown` pseudo-type;
   - no class-side/instance-side leakage.
6. Do not assert completion ordering unless it becomes specified behavior.
7. Avoid sleeps; fix synchronization/revision handling instead.
8. Keep `phalcom-lsp` VM-free.
9. Add fixtures whenever syntax changes editor semantics or highlighting.
10. Add every new integration module to `tests/integration.rs` because `autotests = false`.
