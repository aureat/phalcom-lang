# Handoff — LANG005.C2.P4 Exact-Case Conditional Dispatch Precedence

```yaml
program: LANG005
checkpoint: LANG005.C2
plan: LANG005.C2.P4
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
next_plan: LANG005.C3.P1
worktree: dirty; preserve unrelated working-tree changes
```

## Inherited decision

Exact-case conditional lookup is performed before ordinary enum-root result
handling. The checker publishes the existing conditional selection product;
receiver-effective/editor lookup uses the matching exact-case overlay.

The semantic index deliberately contains exact-case identity domains for
unconditional and covering implementations so exact-case source/member lookup
is available. A separate semantic applicability flag tells lowering that only
receiver-dependent domains need conditional runtime treatment. Unconditional
and covering exact-case members install as `VariantMethod` on their hidden
variant class; specialized/constrained members remain out of shared runtime
classes and use the established conditional fallback path.

## Evidence

The two DEF-006 regressions pass, the complete core inherent-impl lane passes
15/15, semantic inherent-impl coverage passes 41/41, and the named C1
product/reification/Record/optimizer consolidation passes. C2 is complete at
focused verification strength; broad workspace/release certification remains
separate.

## Next action

Continue `LANG005.C3.P1.T3` with shared `MemberBody` normalization for indexed
behavior members. Preserve C2 identities, conditional applicability
ownership, and the no-runtime-trait boundary while implementing C3.
