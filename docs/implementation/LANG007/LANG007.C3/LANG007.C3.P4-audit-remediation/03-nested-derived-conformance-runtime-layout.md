# LANG007.C3.P4 audit remediation — nested derived conformance runtime layout

Date: 2026-09-27

## Finding

The audit's CV50 regression has a conditional `Combined` conformance whose
selected implementation requires two distinct nested conformances (`A |= First`
and `B |= Second`). Core projected each nested proof into runtime slot `0`, so
the runtime environment could not provide the selected implementation with its
full evidence layout. The failure occurred when `Forwarder.s` requested the
second witness after `Forwarder.f` had completed.

## Correction

The runtime recipe builder now obtains the selected source impl's generic
signature (or the guarded rule's signature), builds its canonical
`GenericEvidenceLayout`, and aligns the `DerivedConformanceEvidence.nested`
proofs with the original source-constraint sequence. Equivalent and subtype
proof entries remain in alignment but do not consume conformance slots. Each
executable requirement is mapped by its canonical
`GenericEvidenceSlotKey`; selector spelling and iteration order are not used as
slot identity. Non-callable `DataComponent` selections remain absent from
runtime method recipes.

This is projection of established semantic evidence. The compiler does not
rerun conformance proof search or synthesize a second evidence authority.

## Evidence

Commits:

- `4f0edc63` — preserve nested conditional evidence slots;
- `250c4ad4` — preserve omission of non-callable nested evidence slots.

Focused command:

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core language::conditional_conformance::cv50_slot_isolation_multiple_evidence_values -- --exact --nocapture
```

Result: 1 test selected, 1 passed, 681 filtered. The supervisor independently
reran this exact selector after both commits. No broad core, workspace, or
release gate was run.

## Boundary and remaining work

The separate rest-method probe
`p4_specialized_rest_dispatch_preserves_conformance_environment` does not
currently exercise runtime environment transport: semantic analysis publishes
`Unknown(DynamicMessageSend)` with no callable or `CallSpecialization`, so core
emits ordinary `Invoke` with no runtime recipe. Do not use its runtime failure
as evidence of an environment-drop bug until its semantic dispatch product is
established.

The CV51 environment-reuse failure and the remaining audit runtime failures are
not closed by this evidence. Continue Priority 2 classification against the
named predecessor revisions. P4 lifecycle remains
`IN_PROGRESS / PARTIAL / UNVERIFIED`.
